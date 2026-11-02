mod actions;
mod app;
mod checks;
mod export;
mod finder;
mod help;
mod item;
mod note;
mod people;
mod reports;
mod pictures;
mod question;
mod situation;
mod sixel;
mod start;
pub mod theme;
mod ui;

use std::fmt::Write as _;
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use jiff::civil::Date;
use pchart_core::{Calendar, Plan};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_image::picker::cap_parser::QueryStdioOptions;
use ratatui_image::picker::{Picker, ProtocolType};

use crate::app::{App, Screen};
use crate::pictures::{Painter, Pictures};
pub use crate::theme::Theme;

fn picker() -> io::Result<Picker> {
    let picker = Picker::from_query_stdio_with_options(QueryStdioOptions { kitty_compression: true, ..QueryStdioOptions::default() }).map_err(|err| io::Error::other(err.to_string()))?;
    if matches!(picker.protocol_type(), ProtocolType::Halfblocks | ProtocolType::Iterm2) {
        return Err(io::Error::other("this terminal shows no kitty or sixel pictures; pChart runs in kitty, Ghostty and foot"));
    }
    Ok(picker)
}

pub fn run(plan: Plan, calendars: Vec<Calendar>, today: Date, path: &Path, mut theme: Theme, lanes: &dyn Fn(&Plan) -> Vec<Calendar>) -> io::Result<()> {
    let picker = picker()?;
    let schedule = pchart_core::schedule(&plan, &calendars, today).map_err(io::Error::other)?;
    let mut pictures = Pictures::start(Painter::new(picker, &theme.palette));
    let crew = pchart_store::crew(path, &plan, &schedule, today, lanes);
    let mut app = App::new(plan, calendars, today, path.display().to_string(), schedule, pictures.cell());
    if let Some(crew) = crew {
        app.join(crew);
        app.me = pchart_store::me();
        app.root = pchart_store::workspace(path.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new("."))).map(|(root, _)| root);
        app.refresh_reports();
        app.message = None;
    }
    let mut looked = Instant::now();
    (app.palette, app.ground) = (theme.palette, theme.ground);
    let (written, results) = mpsc::channel();
    let (save, saves) = mpsc::channel::<Plan>();
    let (failed, failures) = mpsc::channel::<String>();
    let target = path.to_path_buf();
    let saver = std::thread::spawn(move || {
        let mut last = Ok(());
        while let Ok(mut plan) = saves.recv() {
            while let Ok(newer) = saves.try_recv() {
                plan = newer;
            }
            last = pchart_store::save(&target, &plan).map_err(|err| err.to_string());
            if let Err(err) = &last {
                let _ = failed.send(err.clone());
            }
        }
        last
    });
    let mut terminal = ratatui::init();
    let mut trace = std::env::var_os("PCHART_TRACE").and_then(|path| std::fs::File::create(path).ok());
    let begun = Instant::now();
    let mut note = |text: String| {
        if let Some(file) = trace.as_mut() {
            let _ = writeln!(file, "{:>8.1} {text}", begun.elapsed().as_secs_f64() * 1000.0);
        }
    };
    let result = loop {
        if theme.refresh(Instant::now()) {
            app.palette = theme.palette;
            pictures.set_palette(theme.palette);
            note("theme changed".into());
        }
        if pictures.receive() {
            note("pictures done".into());
        }
        if app.unsaved && app.before.is_none() {
            app.unsaved = false;
            let _ = save.send(app.plan.clone());
        }
        if let Ok(err) = failures.try_recv() {
            app.message = Some(err);
        }
        if app.root.is_some() && looked.elapsed() >= Duration::from_secs(3) {
            looked = Instant::now();
            app.refresh_reports();
        }
        if let Ok(result) = results.try_recv() {
            app.export.busy = false;
            match result {
                Ok(files) => app.export.written = Some((files, jiff::Zoned::now().strftime("%H:%M").to_string())),
                Err(err) => app.message = Some(err),
            }
        }
        let mut waiting = false;
        if let Err(err) = terminal.draw(|frame| {
            let placements = ui::draw(&app, frame);
            waiting = pictures.draw(frame, placements);
        }) {
            break Err(err);
        }
        let idle = if theme.live() { Duration::from_secs(1) } else if app.root.is_some() { Duration::from_secs(3) } else { Duration::from_secs(60) };
        match event::poll(if waiting || app.export.busy { Duration::from_millis(8) } else { idle }) {
            Ok(false) => continue,
            Ok(true) => {}
            Err(err) => break Err(err),
        }
        let read = event::read();
        note(format!("event {read:?}"));
        match read {
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release && app.finder.is_some() => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => app.finder = None,
                KeyCode::Esc => app.finder = None,
                KeyCode::Up | KeyCode::Down => {
                    let count = app.finder.as_ref().map_or(0, |finder| actions::find(&app, &finder.typed).len());
                    if let Some(finder) = app.finder.as_mut().filter(|_| count > 0) {
                        finder.chosen = (finder.chosen + if key.code == KeyCode::Up { count - 1 } else { 1 }) % count;
                    }
                }
                KeyCode::Enter => {
                    let finder = app.finder.take().unwrap_or_default();
                    if let Some(&(action, item)) = actions::find(&app, &finder.typed).get(finder.chosen)
                        && action.run(&mut app, item, &finder.typed)
                    {
                        break Ok(());
                    }
                }
                KeyCode::Backspace => app.finder.iter_mut().for_each(|finder| {
                    finder.typed.pop();
                    finder.chosen = 0;
                }),
                KeyCode::Char(c) => app.finder.iter_mut().for_each(|finder| {
                    finder.typed.push(c);
                    finder.chosen = 0;
                }),
                _ => {}
            },
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release && app.help => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.help = false;
                    app.finder = Some(app::Finder::default());
                }
                KeyCode::Char('q') => break Ok(()),
                KeyCode::Esc | KeyCode::Char('?') => app.help = false,
                _ => {}
            },
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release && app.form.is_some() => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                KeyCode::Esc => app.close_form(),
                KeyCode::Enter => app.keep_form(),
                KeyCode::Up | KeyCode::BackTab => app.form_move(-1),
                KeyCode::Down | KeyCode::Tab => app.form_move(1),
                KeyCode::Left => app.form_change(-1),
                KeyCode::Right => app.form_change(1),
                KeyCode::Backspace => app.form_back(),
                KeyCode::Char(c) => app.form_type(c),
                _ => {}
            },
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release && app.question.is_some() => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                KeyCode::Esc => app.question = None,
                KeyCode::Enter => app.confirm(),
                KeyCode::Tab => app.try_answer(),
                KeyCode::Up => app.question.as_mut().into_iter().for_each(|question| question.choice = (question.choice + 3) % 4),
                KeyCode::Down => app.question.as_mut().into_iter().for_each(|question| question.choice = (question.choice + 1) % 4),
                KeyCode::Backspace => app.question.as_mut().into_iter().for_each(|question| {
                    question.typed.pop();
                }),
                KeyCode::Char(c) => app.question.as_mut().into_iter().for_each(|question| question.typed.push(c)),
                _ => {}
            },
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release && app.notes.as_ref().is_some_and(|notes| notes.typed.is_some()) => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                KeyCode::Esc => app.notes.iter_mut().for_each(|notes| notes.typed = None),
                KeyCode::Enter => app.keep_note(),
                KeyCode::Backspace => app.notes.iter_mut().filter_map(|notes| notes.typed.as_mut()).for_each(|typed| {
                    typed.pop();
                }),
                KeyCode::Char(c) => app.notes.iter_mut().filter_map(|notes| notes.typed.as_mut()).for_each(|typed| typed.push(c)),
                _ => {}
            },
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release && app.notes.is_some() => {
                let week = if key.modifiers.contains(KeyModifiers::SHIFT) { 7 } else { 1 };
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                    KeyCode::Char('q') => break Ok(()),
                    KeyCode::Esc => app.notes = None,
                    KeyCode::Left => app.move_cursor(-week),
                    KeyCode::Right => app.move_cursor(week),
                    KeyCode::Up => app.cursor_lane(-1),
                    KeyCode::Down => app.cursor_lane(1),
                    KeyCode::Enter => app.write(),
                    KeyCode::Char('i') => app.toggle_important(),
                    KeyCode::Char('u') => app.undo(),
                    _ => {}
                }
            }
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release && app.why => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.why = false;
                    app.finder = Some(app::Finder::default());
                }
                KeyCode::Char('q') => break Ok(()),
                KeyCode::Char('?') => app.help = true,
                KeyCode::Esc | KeyCode::Char('y') => app.why = false,
                KeyCode::Up => app.select(-1),
                KeyCode::Down => app.select(1),
                _ => {}
            },
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release && app.before.is_some() => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                KeyCode::Char('q') => break Ok(()),
                KeyCode::Enter => app.keep(),
                KeyCode::Esc => app.discard(),
                KeyCode::Left if app.level != pchart_draw::Level::Overview => app.try_length(-1),
                KeyCode::Right if app.level != pchart_draw::Level::Overview => app.try_length(1),
                KeyCode::Up => app.select(-1),
                KeyCode::Down => app.select(1),
                KeyCode::Char('d') => app.detail(),
                _ => {}
            },
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release => match (app.screen, key.code) {
                (_, KeyCode::Char('q')) => break Ok(()),
                (_, KeyCode::Char('c')) if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                (_, KeyCode::Char('p')) if key.modifiers.contains(KeyModifiers::CONTROL) => app.finder = Some(app::Finder::default()),
                (_, KeyCode::Char('?')) => app.help = true,
                (_, KeyCode::Char('1')) => app.screen = Screen::Situation,
                (_, KeyCode::Char('2')) => app.screen = Screen::Roadmap,
                (_, KeyCode::Char('3')) => app.screen = Screen::Checks,
                (_, KeyCode::Char('4')) => app.screen = Screen::Export,
                (_, KeyCode::Char('5')) if app.crew.is_some() => app.screen = Screen::People,
                (_, KeyCode::Char('6')) if app.crew.is_some() => app.screen = Screen::Reports,
                (Screen::Reports, KeyCode::Up) => app.report = app.report.saturating_sub(1),
                (Screen::Reports, KeyCode::Down) => app.report = (app.report + 1).min(app.open_reports().len().saturating_sub(1)),
                (Screen::Reports, KeyCode::Enter) => app.take_report(),
                (Screen::Reports, KeyCode::Char('x')) => app.decline_report(),
                (Screen::Reports, KeyCode::Char('e')) => app.edit_report(),
                (Screen::Situation | Screen::Roadmap, KeyCode::Char('t')) if app.dated() => app.start_snapshot(),
                (Screen::Situation | Screen::Roadmap, KeyCode::Char('c')) if app.dated() => app.toggle_compare(),
                (Screen::Roadmap, KeyCode::Esc) if app.compare => app.compare = false,
                (Screen::Roadmap, KeyCode::Char('s')) => app.start_report(),
                (Screen::People, KeyCode::Up) => app.person = app.person.saturating_sub(1),
                (Screen::People, KeyCode::Down) => app.person = (app.person + 1).min(app.crew.as_ref().map_or(0, |crew| people::lines(&people::rows(crew, None)).len().saturating_sub(1))),
                (Screen::People, KeyCode::Left) => app.week = app.week.saturating_sub(1),
                (Screen::People, KeyCode::Right) => app.week = (app.week + 1).min(app.crew.as_ref().map_or(0, |crew| people::all_weeks(&people::rows(crew, None), app.today).len().saturating_sub(1))),
                (Screen::Roadmap, KeyCode::Up) if !app.dated() => app.move_in_network(0.0, -1.0),
                (Screen::Roadmap, KeyCode::Down) if !app.dated() => app.move_in_network(0.0, 1.0),
                (Screen::Roadmap, KeyCode::Left) if !app.dated() => app.move_in_network(-1.0, 0.0),
                (Screen::Roadmap, KeyCode::Right) if !app.dated() => app.move_in_network(1.0, 0.0),
                (Screen::Roadmap, KeyCode::Enter) if !app.dated() => app.start_length(),
                (Screen::Roadmap, KeyCode::Char('a')) if !app.dated() => app.start_add(),
                (Screen::Roadmap, KeyCode::Char('r')) if !app.dated() => app.start_rename(),
                (Screen::Roadmap, KeyCode::Char('w')) if !app.dated() => app.start_waits(),
                (Screen::Roadmap, KeyCode::Char('x') | KeyCode::Delete) if !app.dated() => app.start_remove(),
                (Screen::Roadmap, KeyCode::Char('u')) if !app.dated() => app.undo(),
                (Screen::Situation | Screen::Roadmap | Screen::Checks, _) if !app.dated() => {}
                (Screen::Situation, KeyCode::Up) => app.choose(-1),
                (Screen::Situation, KeyCode::Down) => app.choose(1),
                (Screen::Situation, KeyCode::Enter) => app.show_in_roadmap(),
                (Screen::Roadmap, KeyCode::Char('p')) => app.start_who(),
                (Screen::Checks, KeyCode::Up) => app.choose_check(-1),
                (Screen::Checks, KeyCode::Down) => app.choose_check(1),
                (Screen::Checks, KeyCode::Enter) => app.show_check_in_roadmap(),
                (Screen::Checks, KeyCode::Char('b')) if app.buffer_here() => app.add_buffer(),
                (Screen::Export, KeyCode::Up) => app.export_part(-1),
                (Screen::Export, KeyCode::Down) => app.export_part(1),
                (Screen::Export, KeyCode::Left) => app.export_change(-1),
                (Screen::Export, KeyCode::Right) => app.export_change(1),
                (Screen::Export, KeyCode::Char(' ')) => app.export_toggle(),
                (Screen::Export, KeyCode::Enter) if !app.export.busy => {
                    if app.export.formats.contains(&true) {
                        let job = export::job(&app);
                        let written = written.clone();
                        app.export.busy = true;
                        app.message = None;
                        std::thread::spawn(move || {
                            let _ = written.send(export::write(&job));
                        });
                    } else {
                        app.message = Some("Space switches a format on".into());
                    }
                }
                (_, KeyCode::Esc) => app.screen = Screen::Situation,
                (_, KeyCode::Up) => app.select(-1),
                (_, KeyCode::Down) => app.select(1),
                (_, KeyCode::Char('d')) => app.detail(),
                (_, KeyCode::Char('u')) => app.undo(),
                (Screen::Roadmap, KeyCode::Enter) if app.level != pchart_draw::Level::Overview => app.ask(),
                (Screen::Roadmap, KeyCode::Char('y')) if app.level != pchart_draw::Level::Overview => app.why = true,
                (Screen::Roadmap, KeyCode::Char('n')) if app.level != pchart_draw::Level::Overview => app.add_notes(),
                (Screen::Roadmap, KeyCode::Char(' ')) => app.toggle_details(terminal.size().map_or(0, |size| size.width)),
                (Screen::Roadmap, KeyCode::Char('a')) if app.level != pchart_draw::Level::Overview => app.start_add(),
                (Screen::Roadmap, KeyCode::Char('r')) if app.level != pchart_draw::Level::Overview => app.start_rename(),
                (Screen::Roadmap, KeyCode::Char('w')) if app.level != pchart_draw::Level::Overview => app.start_waits(),
                (Screen::Roadmap, KeyCode::Char('x') | KeyCode::Delete) if app.level != pchart_draw::Level::Overview => app.start_remove(),
                (Screen::Roadmap, KeyCode::Left) if app.level != pchart_draw::Level::Overview => app.try_length(-1),
                (Screen::Roadmap, KeyCode::Right) if app.level != pchart_draw::Level::Overview => app.try_length(1),
                _ => {}
            },
            Ok(_) => {}
            Err(err) => break Err(err),
        }
    };
    ratatui::restore();
    drop(save);
    let saved = saver.join().unwrap_or_else(|_| Err("the plan could not be saved".into()));
    result.and(saved.map_err(|err| io::Error::other(format!("the last change was not saved: {err}"))))
}

/// The start (picture 54), for `pchart` without a plan: returns the plan to open, a new one written to
/// `dir` or one opened before, or None when the user leaves.
/// `calendars` gives the calendars of a plan's lanes, for the plans opened before.
pub fn start(dir: &Path, today: Date, theme: Theme, calendars: &dyn Fn(&Plan) -> Vec<Calendar>) -> io::Result<Option<std::path::PathBuf>> {
    let picker = picker()?;
    let mut pictures = Pictures::start(Painter::new(picker, &theme.palette));
    let workspace = pchart_store::workspace(dir);
    let (folder, plans) = match &workspace {
        Some((root, _)) => (root.join("projects"), pchart_store::projects(root)),
        None => (dir.to_path_buf(), pchart_store::recent()),
    };
    let mut start = start::Start::new(folder, plans, calendars, today, theme.palette, pictures.cell());
    if let Some((root, _)) = &workspace {
        start.people = pchart_store::people(root);
        start.open = start.recent.iter().map(|(path, _, _)| pchart_store::reports(root, &path.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned())).iter().filter(|(_, report)| report.answer.is_none()).count()).collect();
        start.crew = start.recent.iter().filter_map(|(_, plan, lanes)| pchart_core::schedule(plan, lanes, today).ok().map(|schedule| pchart_core::people::Project { name: plan.name.clone(), plan: plan.clone(), schedule, calendars: lanes.clone() })).collect();
    }
    start.workspace = workspace.map(|(_, name)| name);
    let mut terminal = ratatui::init();
    let result = loop {
        pictures.receive();
        let mut waiting = false;
        if let Err(err) = terminal.draw(|frame| {
            let placements = start::draw(&start, frame);
            waiting = pictures.draw(frame, placements);
        }) {
            break Err(err);
        }
        match event::poll(if waiting { Duration::from_millis(8) } else { Duration::from_secs(60) }) {
            Ok(false) => continue,
            Ok(true) => {}
            Err(err) => break Err(err),
        }
        match event::read() {
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release => {
                start.message = None;
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(None),
                    KeyCode::Esc => break Ok(None),
                    KeyCode::Up => start.choose(-1),
                    KeyCode::Down => start.choose(1),
                    KeyCode::Left => start.from(-1),
                    KeyCode::Right => start.from(1),
                    KeyCode::Tab | KeyCode::BackTab => start.naming ^= true,
                    KeyCode::Enter => {
                        if let Some(path) = start.enter() {
                            break Ok(Some(path));
                        }
                    }
                    KeyCode::Backspace if start.naming || start.undated => drop(start.name.pop()),
                    KeyCode::Backspace => drop(start.typed.pop()),
                    KeyCode::Char(c) if start.naming || start.undated => start.name.push(c),
                    KeyCode::Char(c) => start.typed.push(c),
                    _ => {}
                }
            }
            Ok(_) => {}
            Err(err) => break Err(err),
        }
    };
    ratatui::restore();
    result
}

/// Changes the selected item `frames` times like a held arrow key and measures every step,
/// up to the moment the terminal has read the new picture. Writes a report to `out`.
pub fn bench(plan: Plan, calendars: Vec<Calendar>, today: Date, frames: usize, out: &Path) -> io::Result<()> {
    let picker = picker()?;
    let schedule = pchart_core::schedule(&plan, &calendars, today).map_err(io::Error::other)?;
    let mut painter = Painter::new(picker, &pchart_draw::Palette::labra());
    let mut app = App::new(plan, calendars, today, "bench".into(), schedule, painter.cell());
    app.level = pchart_draw::Level::Full;
    app.screen = Screen::Roadmap;
    let protocol_name = painter.protocol_name();
    let id = std::process::id() << 8;
    let (bytes, input) = mpsc::channel::<u8>();
    std::thread::spawn(move || {
        let mut stdin = io::stdin();
        let mut byte = [0u8; 1];
        while stdin.read(&mut byte).is_ok_and(|read| read == 1) {
            if bytes.send(byte[0]).is_err() {
                return;
            }
        }
    });
    let terminal_process = terminal_process();
    let mut terminal = ratatui::init();
    let mut area = terminal.size()?;
    let mut stable = Instant::now();
    let waiting = Instant::now();
    while stable.elapsed() < Duration::from_millis(400) && waiting.elapsed() < Duration::from_secs(3) {
        std::thread::sleep(Duration::from_millis(50));
        let now = terminal.size()?;
        if now != area {
            (area, stable) = (now, Instant::now());
        }
    }
    terminal.autoresize()?;
    let area = ratatui::layout::Rect::new(0, 0, area.width, area.height);
    let rect = ui::chart(area);
    let steps = ["schedule", "scene and SVG", "text and parse", "raster", "encode", "write", "terminal reads it", "change to picture"];
    let mut times: Vec<Vec<f64>> = vec![Vec::new(); steps.len()];
    let started = Instant::now();
    let (own_before, terminal_before) = (cpu_ticks(std::process::id()), terminal_process.as_ref().and_then(|(pid, _)| cpu_ticks(*pid)));
    let mut failure = None;
    for frame in 0..frames {
        let t0 = Instant::now();
        app.try_length(if frame % 2 == 0 { 1 } else { -1 });
        let t1 = Instant::now();
        let svg = app.svg(rect);
        let t2 = Instant::now();
        let Some((protocol, cost)) = painter.paint(&svg, rect, id, false) else {
            failure = Some("the picture could not be painted".to_string());
            break;
        };
        let t3 = Instant::now();
        terminal.draw(|frame| {
            ui::draw(&app, frame);
            frame.render_widget(ratatui_image::Image::new(&protocol), rect);
        })?;
        let t4 = Instant::now();
        let mut stdout = io::stdout();
        stdout.write_all(b"\x1b[c")?;
        stdout.flush()?;
        if !answered(&input, Duration::from_secs(3)) {
            failure = Some(format!("the terminal did not answer after frame {frame}"));
            break;
        }
        let t5 = Instant::now();
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        for (index, value) in [ms(t1 - t0), ms(t2 - t1), ms(cost.parse), ms(cost.raster), ms(cost.encode), ms(t4 - t3), ms(t5 - t4), ms(t5 - t0)].into_iter().enumerate() {
            times[index].push(value);
        }
    }
    let seconds = started.elapsed().as_secs_f64();
    let (own_after, terminal_after) = (cpu_ticks(std::process::id()), terminal_process.as_ref().and_then(|(pid, _)| cpu_ticks(*pid)));
    ratatui::restore();

    let (cw, ch) = app.cell;
    let mut report = String::new();
    let name = terminal_process.as_ref().map_or("unknown terminal".to_string(), |(pid, name)| format!("{name} (pid {pid})"));
    let _ = writeln!(report, "terminal  {name}, {protocol_name}");
    let _ = writeln!(report, "screen    {}×{} cells of {cw}×{ch} px", area.width, area.height);
    let _ = writeln!(report, "picture   {}×{} cells = {}×{} px", rect.width, rect.height, u32::from(rect.width) * u32::from(cw), u32::from(rect.height) * u32::from(ch));
    let _ = writeln!(report, "frames    {} in {seconds:.1} s", times[0].len());
    if let Some(failure) = failure {
        let _ = writeln!(report, "stopped   {failure}");
    }
    let _ = writeln!(report, "\n{:<20}{:>10}{:>10}{:>10}", "step (ms)", "median", "p90", "max");
    for (step, values) in steps.iter().zip(&mut times) {
        values.sort_by(f64::total_cmp);
        if values.is_empty() {
            continue;
        }
        let at = |q: f64| values[((values.len() - 1) as f64 * q).round() as usize];
        let _ = writeln!(report, "{step:<20}{:>10.1}{:>10.1}{:>10.1}", at(0.5), at(0.9), at(1.0));
    }
    let share = |before: Option<u64>, after: Option<u64>| match (before, after) {
        (Some(before), Some(after)) => format!("{:.0} % of one core", (after - before) as f64 / 100.0 / seconds * 100.0),
        _ => "unknown".into(),
    };
    let _ = writeln!(report, "\ncpu       pchart {}, terminal {}", share(own_before, own_after), share(terminal_before, terminal_after));
    let _ = writeln!(report, "memory    pchart peak {}", peak_memory().unwrap_or_else(|| "unknown".into()));
    std::fs::write(out, report)
}

/// Waits for the answer to a primary device attributes query (`ESC [ ? … c`).
fn answered(input: &mpsc::Receiver<u8>, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let mut seen = Vec::new();
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        let Ok(byte) = input.recv_timeout(left) else { return false };
        seen.push(byte);
        if byte == b'c' && seen.windows(3).any(|window| window == b"\x1b[?") {
            return true;
        }
    }
    false
}

fn stat_fields(pid: u32) -> Option<Vec<String>> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    Some(stat.rsplit_once(')')?.1.split_whitespace().map(str::to_string).collect())
}

/// User and system time of a process in clock ticks (100 per second on Linux).
fn cpu_ticks(pid: u32) -> Option<u64> {
    let fields = stat_fields(pid)?;
    Some(fields.get(11)?.parse::<u64>().ok()? + fields.get(12)?.parse::<u64>().ok()?)
}

fn terminal_process() -> Option<(u32, String)> {
    let mut pid = std::process::id();
    for _ in 0..12 {
        let parent: u32 = stat_fields(pid)?.get(1)?.parse().ok()?;
        let name = std::fs::read_to_string(format!("/proc/{parent}/comm")).ok()?.trim().to_string();
        if ["kitty", "ghostty", "foot"].contains(&name.as_str()) {
            return Some((parent, name));
        }
        pid = parent;
    }
    None
}

fn peak_memory() -> Option<String> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmHWM:"))?;
    let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(format!("{:.0} MB", kb / 1024.0))
}
