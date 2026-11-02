//! What each screen shows, and what gives way in a small window (see "Small windows" in the plan).

use jiff::civil::{Date, Weekday};
use pchart_core::analysis::{Severity, State};
use pchart_draw::bits::{self, Icon, MixedPart, Part};
use pchart_draw::scene::Color as Ink;
use pchart_draw::scene::{Fill, Shape, Stroke};
use pchart_draw::{Level, Palette, shown};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::{App, Form, Screen, Spot};
use crate::pictures::Placement;

/// The smallest window pChart draws in; below it says how much it needs.
pub const MIN: (u16, u16) = (60, 16);
const TILES: u16 = 6;

fn color(ink: Ink) -> Color {
    Color::Rgb(ink.0, ink.1, ink.2)
}

pub(crate) struct Inks {
    pub ink: Style,
    pub bright: Style,
    pub dim: Style,
    pub muted: Style,
    pub rule: Style,
    pub warn: Style,
    pub alert: Style,
    pub accent: Color,
    pub raised: Color,
}

impl Inks {
    pub(crate) fn of(p: Palette) -> Inks {
        let style = |ink: Ink| Style::new().fg(color(ink));
        Inks { ink: style(p.ink), bright: style(p.bright), dim: style(p.dim), muted: style(p.muted), rule: style(p.rule), warn: style(p.warn), alert: style(p.alert), accent: color(p.accent), raised: color(p.raised) }
    }
}

pub(crate) fn bold(style: Style) -> Style {
    style.add_modifier(Modifier::BOLD)
}

pub(crate) fn width(spans: &[Span]) -> u16 {
    spans.iter().map(|span| span.content.chars().count() as u16).sum()
}

pub(crate) fn put(frame: &mut Frame, x: u16, y: u16, room: u16, spans: Vec<Span<'static>>) {
    let area = frame.area();
    if y >= area.height || x >= area.width {
        return;
    }
    let room = room.min(area.width - x);
    frame.render_widget(Paragraph::new(Line::from(spans)), Rect { x, y, width: room, height: 1 });
}

/// `text` cut to `room` characters, ending in "…" when it was cut.
pub(crate) fn cut(text: &str, room: usize) -> String {
    if text.chars().count() <= room {
        return text.to_string();
    }
    text.chars().take(room.saturating_sub(1)).collect::<String>() + "…"
}

/// A panel: a thin frame with its title; the panel with the focus gets brighter corners.
pub(crate) fn panel(frame: &mut Frame, inks: &Inks, rect: Rect, title: &str, focus: bool) {
    if rect.width < 4 || rect.height < 3 {
        return;
    }
    frame.render_widget(Block::bordered().border_style(inks.rule), rect);
    if focus {
        let (x0, y0, x1, y1) = (rect.x, rect.y, rect.x + rect.width - 1, rect.y + rect.height - 1);
        let buffer = frame.buffer_mut();
        for (x, y, symbol) in [(x0, y0, "┌"), (x0 + 1, y0, "─"), (x0, y0 + 1, "│"), (x1, y0, "┐"), (x1 - 1, y0, "─"), (x1, y0 + 1, "│"), (x0, y1, "└"), (x0 + 1, y1, "─"), (x0, y1 - 1, "│"), (x1, y1, "┘"), (x1 - 1, y1, "─"), (x1, y1 - 1, "│")] {
            buffer[(x, y)].set_symbol(symbol).set_style(inks.ink);
        }
    }
    if !title.is_empty() {
        put(frame, rect.x + 2, rect.y + 1, rect.width.saturating_sub(4), vec![Span::styled(format!(":: {title}"), inks.dim)]);
    }
}

/// Draws the screen and returns the pictures it needs.
pub fn draw(app: &App, frame: &mut Frame) -> Vec<Placement> {
    let area = frame.area();
    let inks = Inks::of(app.palette);
    if app.ground {
        frame.buffer_mut().set_style(area, Style::new().bg(color(app.palette.page)).fg(color(app.palette.ink)));
    }
    if area.width < MIN.0 || area.height < MIN.1 {
        too_small(frame, &inks, area);
        return Vec::new();
    }
    let mut placements = Vec::new();
    title_bar(app, frame, &inks, area);
    let under = Inks::of(app.shown());
    match app.screen {
        Screen::Roadmap if !app.dated() => network(app, frame, &under, area, &mut placements),
        Screen::Situation | Screen::Checks if !app.dated() => undated(app, frame, &under, area),
        Screen::Situation => crate::situation::draw(app, frame, &under, area, &mut placements),
        Screen::Checks => crate::checks::draw(app, frame, &under, area, &mut placements),
        Screen::Export => crate::export::draw(app, frame, &under, area, &mut placements),
        Screen::People => crate::people::draw(app, frame, &under, area, &mut placements),
        Screen::Reports => crate::reports::draw(app, frame, &under, area, &mut placements),
        Screen::Roadmap => {
            let palette = if app.question.is_some() || app.form.is_some() || writing(app) || app.overlay() { app.palette.dimmed() } else { app.palette };
            let under = Inks::of(palette);
            let top = if area.width >= 120 && area.height >= 44 {
                tiles(app, &palette, frame, &under, area, &mut placements);
                1 + TILES
            } else {
                summary(app, frame, &under, area);
                2
            };
            let body = Rect { x: 0, y: top, width: area.width, height: area.height - 1 - top };
            if app.side && area.width >= 150 {
                let side = (area.width * 26 / 100).clamp(40, 56);
                roadmap(app, &palette, frame, &under, Rect { width: body.width - side, ..body }, &mut placements);
                details(app, &palette, frame, &under, Rect { x: body.width - side, width: side, ..body }, &mut placements);
            } else {
                roadmap(app, &palette, frame, &under, body, &mut placements);
            }
        }
    }
    if app.overlay() {
        placements.retain(|placement| !placement.paper);
    }
    if app.help {
        crate::help::draw(app, frame, area, &mut placements);
    }
    if app.finder.is_some() {
        crate::finder::draw(app, frame, area, &mut placements);
    }
    keys(app, frame, &inks, area);
    placements
}

fn too_small(frame: &mut Frame, inks: &Inks, area: Rect) {
    let size = |value: u16, need: u16| Span::styled(value.to_string(), bold(if value < need { inks.alert } else { inks.bright }));
    let lines = [
        vec![Span::styled("Terminal size too small:", inks.ink)],
        vec![Span::styled("Width = ", inks.ink), size(area.width, MIN.0), Span::styled(" Height = ", inks.ink), size(area.height, MIN.1)],
        vec![],
        vec![Span::styled("Needed:", inks.ink)],
        vec![Span::styled("Width = ", inks.ink), Span::styled(MIN.0.to_string(), bold(inks.bright)), Span::styled(" Height = ", inks.ink), Span::styled(MIN.1.to_string(), bold(inks.bright))],
    ];
    let top = area.y + (area.height / 2).saturating_sub(2);
    for (y, spans) in (top..area.bottom()).zip(lines) {
        let x = area.x + area.width.saturating_sub(width(&spans)) / 2;
        put(frame, x, y, area.width.saturating_sub(x), spans);
    }
}

fn title_bar(app: &App, frame: &mut Frame, inks: &Inks, area: Rect) {
    let version: String = env!("CARGO_PKG_VERSION").split('.').take(2).collect::<Vec<_>>().join(".");
    let left = vec![Span::styled("pchart", bold(inks.bright)), Span::styled(format!(" {version} · {}", app.plan.name), inks.dim)];
    let mut x = width(&left);
    put(frame, 0, 0, area.width, left);
    x = (x + 5).max(36.min(area.width / 4));
    let warnings = app.analysis.checks.iter().filter(|check| check.severity() != Severity::Hint).count();
    let problems = app.analysis.checks.iter().any(|check| check.severity() == Severity::Problem);
    let full = area.width >= 100;
    let second = if app.dated() { "Roadmap" } else { "Network" };
    let tabs = [("1", "Situation", Screen::Situation), ("2", second, Screen::Roadmap), ("3", "Checks", Screen::Checks), ("4", "Export", Screen::Export), ("5", "People", Screen::People), ("6", "Reports", Screen::Reports)];
    let open = app.open_reports().len();
    for (number, name, screen) in tabs.into_iter().take(if app.crew.is_some() { 6 } else { 4 }) {
        let current = app.screen == screen;
        let mut spans = if current {
            let under = |style: Style| style.add_modifier(Modifier::UNDERLINED).underline_color(inks.accent).bg(inks.raised);
            vec![Span::styled(" ", under(inks.ink)), Span::styled(number, under(bold(Style::new().fg(inks.accent)))), Span::styled(format!(" {name} "), under(bold(inks.bright)))]
        } else if full {
            vec![Span::styled(format!(" {number}"), bold(inks.ink)), Span::styled(format!(" {name} "), inks.dim)]
        } else {
            vec![Span::styled(format!(" {number} "), bold(inks.ink))]
        };
        if screen == Screen::Checks && warnings > 0 {
            spans.push(Span::styled(format!("▲{warnings} "), if problems { inks.alert } else { inks.warn }));
        }
        if screen == Screen::Reports && open > 0 {
            spans.push(Span::styled(format!("●{open} "), inks.warn));
        }
        let w = width(&spans);
        put(frame, x, 0, w, spans);
        x += w + 1;
    }
    let modes: Vec<String> = match (app.screen, &app.question, &app.before) {
        (Screen::Roadmap, None, None) if writing(app) => vec!["NOTE · nothing changed yet".into(), "NOTE".into()],
        (Screen::Roadmap, None, None) if app.notes.is_some() => vec!["ADD NOTES".into()],
        (Screen::Roadmap, Some(_), _) if area.width >= 100 => vec!["CHANGE A DATE · nothing changed yet".into()],
        (Screen::Roadmap, Some(_), _) => vec!["CHANGE A DATE".into()],
        (Screen::Roadmap, None, Some(_)) => {
            let changes = app.changes();
            match changes.len() {
                0 => vec!["WHAT-IF · nothing changed yet".into(), "WHAT-IF".into()],
                1 => vec![format!("WHAT-IF · {} · not saved", changes[0]), "WHAT-IF · not saved".into(), "WHAT-IF".into()],
                count => vec![format!("WHAT-IF · {count} changes · not saved"), "WHAT-IF · not saved".into(), "WHAT-IF".into()],
            }
        }
        _ => Vec::new(),
    };
    if !modes.is_empty() {
        if let Some(mode) = modes.into_iter().find(|mode| x + mode.chars().count() as u16 + 2 < area.width) {
            let w = mode.chars().count() as u16;
            put(frame, area.width - w - 1, 0, w, vec![Span::styled(mode, inks.warn)]);
        }
        return;
    }
    let right = if area.width >= 135 { format!("{} · saved", home(&app.path)) } else { "saved".to_string() };
    let right = if (right.chars().count() as u16) + x + 2 > area.width { "saved".to_string() } else { right };
    let w = right.chars().count() as u16;
    if x + w < area.width {
        put(frame, area.width - w - 1, 0, w, vec![Span::styled(right, inks.dim)]);
    }
}

/// `path` with the home folder as "~".
pub(crate) fn home(path: &str) -> String {
    match std::env::var("HOME").ok().filter(|home| !home.is_empty() && path.starts_with(home.as_str())) {
        Some(home) => format!("~{}", &path[home.len()..]),
        None => path.to_string(),
    }
}

/// The verdict words and their colour: neutral for on track, yellow for at risk, red for late.
pub(crate) fn state(app: &App, inks: &Inks) -> Option<(&'static str, Style)> {
    app.analysis.verdict.as_ref().map(|verdict| match verdict.state {
        State::Holds => ("on track", inks.bright),
        State::Tight => ("at risk", inks.warn),
        State::DoesNotHold => ("late", inks.alert),
    })
}

pub(crate) fn holidays(app: &App) -> Vec<(usize, Date, String)> {
    let weekday = |day: Date| !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday);
    let mut out = Vec::new();
    for (index, item) in app.plan.items.iter().enumerate().filter(|(_, item)| !item.milestone) {
        let lane = app.plan.lane(&item.lane).unwrap_or(0);
        let slot = &app.schedule.slots[index].planned;
        out.extend(app.calendars[lane].holidays(slot.start, slot.finish).filter(|(day, _)| weekday(*day)).map(|(day, name)| (index, day, name.to_string())));
    }
    out
}

fn tiles(app: &App, palette: &Palette, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    let (cw, ch) = app.cell;
    let quarter = area.width / 4;
    let big = ch as f32 * 46.0 / 21.0;
    let p = *palette;
    let target = app.schedule.target;
    let first = app.schedule.first;
    let spare = app.schedule.spare();
    let warnings = app.analysis.checks.iter().filter(|check| check.severity() == Severity::Warning).count();
    let problems = app.analysis.checks.iter().filter(|check| check.severity() == Severity::Problem).count();
    let tone = state(app, inks).map_or(p.bright, |(word, _)| match word {
        "at risk" => p.warn,
        "late" => p.alert,
        _ => p.bright,
    });
    let end = app.schedule.slots.iter().enumerate().max_by_key(|(_, slot)| slot.planned.finish).map(|(index, slot)| (index, slot.planned.finish));
    let day = |date: Date| (date.strftime("%d %b").to_string(), date.strftime("%a · %Y").to_string());
    let tiles: [(String, String, Ink, String, Style, String); 4] = [
        match target {
            Some(index) => {
                let item = &app.plan.items[index];
                let date = app.schedule.slots[index].planned.finish;
                let kind = if item.on.is_some() { "TARGET · FIXED" } else { "TARGET · DEADLINE" };
                let (side, side_style) = match state(app, inks).filter(|_| app.before.is_some()) {
                    Some((word, style)) => (word.to_string(), style),
                    None => (day(date).1, inks.dim),
                };
                (kind.into(), day(date).0, p.bright, side, side_style, format!("{} · scheduled backwards", item.name))
            }
            None => match end {
                Some((index, date)) => ("END".into(), day(date).0, p.bright, day(date).1, inks.dim, format!("{} · scheduled forwards", app.plan.items[index].name)),
                None => ("END".into(), String::new(), p.bright, String::new(), inks.dim, String::new()),
            },
        },
        match first {
            Some(index) => {
                let date = app.schedule.start_by.unwrap_or(app.schedule.slots[index].latest.start);
                let name = app.plan.items[index].name.clone();
                match app.before {
                    Some(_) if date < app.today => ("LATEST START".into(), day(date).0, p.alert, day(date).1, inks.dim, name),
                    Some(_) if date == app.today => ("LATEST START".into(), day(date).0, p.warn, "today".into(), inks.warn, name),
                    _ => ("LATEST START".into(), day(date).0, p.bright, day(date).1, inks.dim, name),
                }
            }
            None => ("LATEST START".into(), String::new(), p.bright, String::new(), inks.dim, String::new()),
        },
        match spare {
            Some(spare) => {
                let foot = app.before.as_ref().and_then(|before| before.schedule.spare()).map_or("until the latest start".to_string(), |was| format!("was {was}"));
                ("TO SPARE".into(), spare.to_string(), tone, "working days".into(), inks.dim, foot)
            }
            None => ("TO SPARE".into(), String::new(), p.bright, String::new(), inks.dim, String::new()),
        },
        {
            let count = holidays(app).len();
            let foot = if count == 1 { "1 holiday accounted for".to_string() } else { format!("{count} holidays accounted for") };
            if problems > 0 {
                ("CHECKS".into(), problems.to_string(), p.alert, if problems == 1 { "problem".into() } else { "problems".into() }, inks.alert, foot)
            } else if warnings > 0 {
                ("CHECKS".into(), warnings.to_string(), p.warn, if warnings == 1 { "warning".into() } else { "warnings".into() }, inks.warn, foot)
            } else {
                ("CHECKS".into(), "0".into(), p.bright, "warnings".into(), inks.dim, foot)
            }
        },
    ];
    for (index, (title, value, value_ink, side, side_style, foot)) in tiles.into_iter().enumerate() {
        let x = index as u16 * quarter;
        let w = if index == 3 { area.width - x } else { quarter };
        panel(frame, inks, Rect { x, y: 1, width: w, height: TILES }, &title, false);
        let mut side_x = x + 2;
        if !value.is_empty() {
            let cols = ((value.chars().count() as f32 * 0.6 * big) / cw as f32).ceil() as u16 + 1;
            let rect = Rect { x: x + 2, y: 3, width: cols.min(w.saturating_sub(4)), height: 2 };
            let (px_w, px_h) = (f32::from(rect.width) * f32::from(cw), f32::from(rect.height) * f32::from(ch));
            placements.push(Placement { rect, svg: bits::big(&value, big, value_ink, px_w, px_h, palette).svg(), paper: false });
            side_x = rect.x + rect.width + 2;
        }
        put(frame, side_x, 4, (x + w).saturating_sub(side_x + 2), vec![Span::styled(side, side_style)]);
        put(frame, x + 2, 5, w.saturating_sub(4), vec![Span::styled(cut(&foot, w.saturating_sub(4) as usize), inks.dim)]);
    }
}

/// One line instead of the tiles: the target, the verdict, the latest start, the checks.
fn summary(app: &App, frame: &mut Frame, inks: &Inks, area: Rect) {
    let room = area.width.saturating_sub(2);
    let mut groups: Vec<Vec<Span<'static>>> = Vec::new();
    let long = |date: Date| date.strftime("%a %d %b %Y").to_string();
    if let Some(index) = app.schedule.target {
        let item = &app.plan.items[index];
        let mut group = vec![Span::styled(format!("{} ", item.name), inks.ink), Span::styled(long(app.schedule.slots[index].planned.finish), bold(inks.bright))];
        if item.on.is_some() {
            group.push(Span::styled(" · fixed", inks.dim));
        }
        groups.push(group);
    }
    if let (Some((word, style)), Some(spare)) = (state(app, inks), app.schedule.spare()) {
        groups.push(vec![Span::styled(word, bold(style)), Span::styled(" · ", inks.dim), Span::styled(spare.to_string(), bold(inks.bright)), Span::styled(" wd to spare", inks.dim)]);
    }
    if let Some(by) = app.schedule.start_by
        && app.schedule.target.is_some()
    {
        groups.push(vec![Span::styled("start by ", inks.dim), Span::styled(by.strftime("%a %d %b").to_string(), bold(inks.bright))]);
    }
    let warnings = app.analysis.checks.iter().filter(|check| check.severity() != Severity::Hint).count();
    if warnings > 0 {
        let problems = app.analysis.checks.iter().any(|check| check.severity() == Severity::Problem);
        let noun = if problems { if warnings == 1 { "problem" } else { "problems" } } else if warnings == 1 { "warning" } else { "warnings" };
        groups.push(vec![Span::styled(format!("▲ {warnings} {noun}"), if problems { inks.alert } else { inks.warn })]);
    }
    let join = |groups: &[Vec<Span<'static>>]| {
        let mut line = Vec::new();
        for (index, group) in groups.iter().enumerate() {
            if index > 0 {
                line.push(Span::raw("   "));
            }
            line.extend(group.iter().cloned());
        }
        line
    };
    while groups.len() > 1 && width(&join(&groups)) > room {
        groups.pop();
    }
    put(frame, 1, 1, room, join(&groups));
}

/// Screen 2 of a plan whose work has no length yet (picture 60): the network diagram with the selection,
/// moved with the arrow keys and scrolled to keep it in sight, the dialogs over it, and a line about the
/// selection under it.
fn network(app: &App, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    let p = app.palette;
    let box_rect = Rect { x: 0, y: 1, width: area.width, height: area.height.saturating_sub(3) };
    panel(frame, inks, box_rect, "NETWORK DIAGRAM", true);
    let work = app.plan.items.iter().filter(|item| !item.milestone).count();
    let missing = vec![Span::styled(format!("{} of {work} without a length", app.plan.without_length().len()), inks.dim)];
    let w = width(&missing);
    if box_rect.width > w + 24 {
        put(frame, box_rect.x + box_rect.width - w - 2, box_rect.y + 1, w, missing);
    }
    let chart = Rect { x: box_rect.x + 2, y: box_rect.y + 2, width: box_rect.width.saturating_sub(4), height: box_rect.height.saturating_sub(3) };
    let (cw, ch) = app.cell;
    let (pw, ph, u) = (f32::from(chart.width) * f32::from(cw), f32::from(chart.height) * f32::from(ch), f32::from(ch) / 21.0);
    if chart.width > 10 && chart.height > 4 {
        let mut page = pchart_draw::network(&app.plan, &app.schedule, app.today);
        let (_, boxes, _) = pchart_draw::structure::layout(&app.plan, &app.schedule);
        let [x0, y0, x1, y1] = boxes[app.selected];
        let (x0, y0, x1, y1, arm) = (x0 - 6.0, y0 - 6.0, x1 + 6.0, y1 + 6.0, 9.0);
        for (cx, cy, sx, sy) in [(x0, y0, 1.0, 1.0), (x1, y0, -1.0, 1.0), (x0, y1, 1.0, -1.0), (x1, y1, -1.0, -1.0)] {
            page.add(Shape::Path { points: vec![(cx + sx * arm, cy), (cx, cy), (cx, cy + sy * arm)], closed: false, fill: Fill::None, stroke: Some(Stroke { color: pchart_draw::page::INK, width: 2.0, dash: None }) });
        }
        let tall = page.width * ph / pw;
        let top = (y1 + 120.0 - tall).clamp(0.0, (page.height - tall).max(0.0));
        if app.form.is_none() {
            placements.push(Placement { rect: chart, svg: page.window(top, pw, ph), paper: true });
        } else {
            page.add(Shape::Rect { x: 0.0, y: 0.0, w: page.width, h: page.height, radius: 0.0, fill: Fill::Color(p.page), stroke: None });
            page.marks.last_mut().expect("the shade").opacity = 0.78;
            let mut s = bits::canvas(pw, ph, &p);
            s.background = None;
            let w = chart.width.saturating_sub(4).min(94);
            let h = chart.height.min(match app.form {
                Some(Form::Rename { .. } | Form::Length { .. }) => 9,
                Some(Form::Remove { item }) => match app.removal(item) {
                    Ok((_, words)) => (255.0 + 28.0 * words.len().max(1) as f32) / 21.0,
                    Err(_) => 8.0,
                }
                .ceil() as u16,
                Some(Form::Add(_)) => 15,
                Some(Form::Who { item, .. }) => who_height(app, item),
                Some(Form::Report { .. }) => 16,
                _ => 32,
            });
            let x = (chart.width - w) / 2;
            crate::item::dialog(app, &mut s, f32::from(x) * f32::from(cw), 0.0, f32::from(w) * f32::from(cw), f32::from(h) * f32::from(ch), u);
            let mut svg = page.window(top, pw, ph);
            let end = svg.rfind("</svg>").unwrap_or(svg.len());
            svg.insert_str(end, &s.nested(0.0, top, page.width, tall));
            placements.push(Placement { rect: chart, svg, paper: false });
        }
    }
    let rect = Rect { x: 2, y: area.height - 2, width: area.width.saturating_sub(4), height: 1 };
    let line = bits::line(&built(app, &p), f32::from(rect.width) * f32::from(cw), f32::from(ch), u, &p);
    placements.push(Placement { rect, svg: line.svg(), paper: false });
}

/// The line about the selection in the network diagram: its name, its team, its length or that it has
/// none yet, what it waits on and what waits on it.
fn built(app: &App, p: &Palette) -> Vec<Part> {
    let item = &app.plan.items[app.selected];
    let lane = &app.plan.lanes[app.plan.lane(&item.lane).unwrap_or(0)];
    let team = [Some(lane.name.as_str()), lane.city.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
    let names = |list: Vec<&str>| list.join(", ");
    let mut parts = vec![(None, item.name.clone(), p.bright, true), (Some((Icon::Person, p.dim)), team, p.dim, false)];
    if !item.milestone {
        parts.push(if item.duration > 0 { (Some((Icon::Clock, p.dim)), format!("{} wd", item.duration), p.ink, false) } else { (Some((Icon::Clock, p.dim)), "no length yet".into(), p.muted, false) });
    }
    let before: Vec<&str> = item.after.iter().filter_map(|id| app.plan.item(id)).map(|index| app.plan.items[index].name.as_str()).collect();
    let then: Vec<&str> = app.plan.items.iter().filter(|other| other.after.contains(&item.id)).map(|other| other.name.as_str()).collect();
    parts.push((None, if before.is_empty() { "nothing comes before it".into() } else { format!("after {}", names(before)) }, p.ink, false));
    if !then.is_empty() {
        parts.push((None, format!("then {}", names(then)), p.ink, false));
    }
    parts
}

/// The situation, the roadmap and the checks of a plan whose work has no length yet: what is missing.
fn undated(app: &App, frame: &mut Frame, inks: &Inks, area: Rect) {
    let rect = Rect { x: 0, y: 1, width: area.width, height: area.height - 2 };
    panel(frame, inks, rect, "NO DATES YET", false);
    let missing = app.plan.without_length();
    let work = app.plan.items.iter().filter(|item| !item.milestone).count();
    let (x, room) = (4, area.width.saturating_sub(8));
    put(frame, x, 4, room, vec![Span::styled(format!("{} of {work} pieces of work have no length yet.", missing.len()), bold(inks.bright))]);
    put(frame, x, 5, room, vec![Span::styled("Dates, the roadmap and the checks come once each has one.", inks.dim)]);
    put(frame, x, 7, room, vec![Span::styled("Without a length", inks.dim)]);
    let rows = usize::from(rect.height.saturating_sub(9));
    for (row, &index) in missing.iter().enumerate().take(rows) {
        let more = missing.len() - row - 1;
        let text = if row + 1 == rows && more > 0 { format!("and {} more", more + 1) } else { app.plan.items[index].name.clone() };
        put(frame, x + 2, 8 + row as u16, room, vec![Span::styled(cut(&text, usize::from(room)), inks.ink)]);
    }
}

fn keys(app: &App, frame: &mut Frame, inks: &Inks, area: Rect) {
    let item = if app.level == Level::Overview { "lane" } else { "item" };
    let notes = |spot: Option<Spot>| {
        let milestone = matches!(spot, Some(Spot::Item(index)) if app.plan.items[index].milestone);
        let enter = match (spot, app.note_here()) {
            (_, Some(_)) => "change the note",
            (Some(Spot::Place(..)), None) => "write a note here",
            _ => "write a note",
        };
        let mut hints = vec![("←→", "day"), ("Shift ←→", "week"), ("↑↓", "lane"), ("Enter", enter)];
        if milestone {
            hints.push(("i", "important"));
        }
        hints.push(("Esc", "done"));
        hints
    };
    let mut hints: Vec<(&str, &str)> = match app.screen {
        _ if app.finder.is_some() => vec![("↑↓", "choose"), ("Enter", "run"), ("Esc", "close")],
        _ if app.help => vec![("Esc", "close"), ("Ctrl+P", "find an action")],
        Screen::Roadmap if !app.dated() && app.form.is_none() => vec![("↑↓←→", "item"), ("Enter", "length"), ("a", "add"), ("w", "waits on"), ("r", "rename"), ("x", "remove"), ("?", "help")],
        Screen::Roadmap if app.form.is_some() && matches!(app.form, Some(Form::Length { .. })) => vec![("Enter", "keep the length"), ("Esc", "keep as it is")],
        Screen::Situation | Screen::Checks if !app.dated() => vec![("2", "how it is built"), ("?", "help")],
        Screen::Situation if app.since().is_some() => vec![("↑↓", "choose"), ("Enter", "show in roadmap"), ("c", "what changed"), ("?", "help")],
        Screen::Situation => vec![("↑↓", "choose"), ("Enter", "show in roadmap"), ("?", "help")],
        Screen::Roadmap if app.question.is_some() => vec![("↑↓", "choose"), ("Enter", "apply"), ("Tab", "try as what-if"), ("Esc", "keep as it is")],
        Screen::Roadmap if app.form.is_some() => match &app.form {
            Some(Form::Add(_)) => vec![("↑↓", "question"), ("←→", "change"), ("Enter", "add"), ("Esc", "keep as it is")],
            Some(Form::Rename { .. }) => vec![("Enter", "rename"), ("Esc", "keep as it is")],
            Some(Form::Snapshot { .. }) => vec![("Enter", "keep it"), ("Esc", "not now")],
            Some(Form::Remove { item }) if app.removal(*item).is_err() => vec![("Esc", "back")],
            Some(Form::Remove { .. }) => vec![("Enter", "remove"), ("Esc", "keep as it is")],
            Some(Form::Who { .. }) => vec![("↑↓", "person"), ("←→", "share"), ("Enter", "choose"), ("Esc", "keep as it is")],
            Some(Form::Report { answering: Some(_), .. }) => vec![("↑↓", "question"), ("←→", "change"), ("Enter", "take it"), ("Esc", "back to reports")],
            Some(Form::Report { .. }) => vec![("↑↓", "question"), ("←→", "change"), ("Enter", "send"), ("Esc", "keep as it is")],
            _ => vec![("↑↓", "item"), ("Space", "waits on it or not"), ("Enter", "keep"), ("Esc", "keep as it is")],
        },
        Screen::Roadmap if writing(app) => vec![("Enter", "keep the note"), ("Esc", "keep as it is")],
        Screen::Roadmap if app.notes.is_some() => notes(app.spot()),
        Screen::Roadmap if app.before.is_some() => vec![("Enter", "keep"), ("Esc", "discard"), ("←→", "keep changing"), ("↑↓", item)],
        Screen::Roadmap if app.why => vec![("↑↓", "another item"), ("Esc", "back"), ("?", "help")],
        Screen::Roadmap if app.compare && app.form.is_none() => vec![("↑↓", item), ("c", "back"), ("t", "snapshot"), ("?", "help")],
        Screen::Roadmap => vec![("↑↓", item), ("←→", "length"), ("Enter", "change a date"), ("p", "who does it"), ("s", "report"), ("n", "add notes"), ("u", "undo"), ("d", "detail"), ("?", "help")],
        Screen::Checks if app.buffer_here() => vec![("↑↓", "check"), ("Enter", "show in roadmap"), ("b", "add the buffer"), ("?", "help")],
        Screen::Checks if app.check < app.checks().len() => vec![("↑↓", "check"), ("Enter", "show in roadmap"), ("?", "help")],
        Screen::Checks => vec![("↑↓", "check"), ("?", "help")],
        Screen::People => vec![("↑↓", "team or person"), ("←→", "week"), ("?", "help")],
        Screen::Reports if app.open_reports().is_empty() => vec![("?", "help")],
        Screen::Reports if !app.answers() => vec![("↑↓", "report"), ("?", "help")],
        Screen::Reports => vec![("↑↓", "report"), ("Enter", "take it"), ("e", "change, then take"), ("x", "decline"), ("?", "help")],
        Screen::Export => vec![("↑↓", "option"), ("←→", "change"), ("Space", "on/off"), ("Enter", "export"), ("Esc", "back"), ("?", "help")],
    };
    let total = |hints: &[(&str, &str)]| hints.iter().map(|(key, label)| key.chars().count() + label.chars().count() + 5).sum::<usize>();
    let message = app.message.as_ref().map_or(0, |message| message.chars().count());
    while hints.len() > 1 && total(&hints) + message > usize::from(area.width.saturating_sub(1)) && hints.last().is_some_and(|(key, _)| *key == "?") {
        hints.remove(hints.len() - 2);
    }
    let mut spans = Vec::new();
    for (key, label) in hints {
        let next = [Span::styled(key, bold(inks.ink)), Span::styled(format!(" {label}"), inks.dim), Span::raw("    ")];
        if width(&spans) + width(&next) > area.width.saturating_sub(1) {
            break;
        }
        spans.extend(next);
    }
    if let Some(message) = &app.message {
        spans.push(Span::styled(message.clone(), inks.warn));
    }
    put(frame, 1, area.height - 1, area.width - 1, spans);
}

/// Where the parts of the roadmap screen go in `body` (picture 35): the chart in its frame, and under
/// it one line about the selection.
pub struct Layout {
    pub panel: Rect,
    pub chart: Rect,
    line: u16,
}

/// Where the roadmap picture goes on a screen of this size.
/// The details beside the roadmap (picture 52): what the line under the chart does not say about the
/// selection, and the checks.
fn details(app: &App, palette: &Palette, frame: &mut Frame, inks: &Inks, rect: Rect, placements: &mut Vec<Placement>) {
    let index = app.selected;
    let item = &app.plan.items[index];
    let slot = &app.schedule.slots[index];
    let name = |id: &String| app.plan.item(id).map(|other| app.plan.items[other].name.clone());
    let span = |span: pchart_core::Span| if item.milestone { day(span.finish) } else { format!("{} – {}", day(span.start), day(span.finish)) };
    let then: Vec<String> = app.plan.items.iter().filter(|other| other.after.contains(&item.id)).map(|other| other.name.clone()).collect();
    let date = match (item.on, item.no_earlier_than, item.no_later_than) {
        (Some(on), _, _) => format!("fixed on {}", day(on)),
        (_, Some(from), Some(by)) => format!("not before {}, by {}", day(from), day(by)),
        (_, Some(from), None) => format!("not before {}", day(from)),
        (_, None, Some(by)) => format!("by {}", day(by)),
        _ => "calculated".to_string(),
    };
    let after: Vec<String> = item.after.iter().filter_map(name).collect();
    let mut rows: Vec<(&str, String)> = vec![
        ("Waits on", if after.is_empty() { "nothing".into() } else { after.join(", ") }),
        ("Then", if then.is_empty() { "nothing".into() } else { then.join(", ") }),
        ("Earliest", span(slot.earliest)),
        ("Latest", span(slot.latest)),
        ("Date", date),
    ];
    let days: Vec<(Date, String)> = holidays(app).into_iter().filter(|(at, _, _)| *at == index).map(|(_, date, name)| (date, name)).collect();
    for (at, (date, name)) in days.iter().enumerate() {
        rows.push((if at == 0 { "Holidays" } else { "" }, format!("{} {name}", date.strftime("%d %b"))));
    }
    let width = rect.width.saturating_sub(4);
    let value_w = width.saturating_sub(11) as usize;
    let mut lines: Vec<(&str, String)> = Vec::new();
    for (label, value) in rows {
        let mut first = true;
        for chunk in wrap(&value, value_w) {
            lines.push((if first { label } else { "" }, chunk));
            first = false;
        }
    }
    if let Some(note) = &item.note {
        for (at, chunk) in wrap(note, value_w).into_iter().enumerate() {
            lines.push((if at == 0 { "Note" } else { "" }, chunk));
        }
    }
    let top_h = (lines.len() as u16 + 4).min(rect.height.saturating_sub(6));
    let top = Rect { height: top_h, ..rect };
    panel(frame, inks, top, &format!("DETAILS · {}", item.name.to_uppercase()), false);
    for (row, (label, value)) in lines.iter().enumerate() {
        let y = top.y + 3 + row as u16;
        if y + 1 >= top.y + top.height {
            break;
        }
        put(frame, top.x + 2, y, 10, vec![Span::styled(label.to_string(), inks.dim)]);
        put(frame, top.x + 13, y, width.saturating_sub(11), vec![Span::styled(value.clone(), inks.ink)]);
    }
    let bottom = Rect { y: top.y + top.height, height: rect.height - top.height, ..rect };
    if bottom.height < 5 {
        return;
    }
    panel(frame, inks, bottom, &crate::checks::title(app), false);
    let (cw, ch) = (f32::from(app.cell.0), f32::from(app.cell.1));
    let mut y = bottom.y + 3;
    for &check_index in &app.checks() {
        if y + 2 >= bottom.y + bottom.height {
            break;
        }
        let check = &app.analysis.checks[check_index];
        let (title, place) = crate::checks::words(app, check);
        let (icon, _) = crate::checks::symbol(app, check);
        let tone = match check.severity() {
            Severity::Problem => palette.alert,
            Severity::Warning => palette.warn,
            Severity::Hint => palette.dim,
        };
        let line = Rect { x: bottom.x + 2, y, width, height: 1 };
        placements.push(Placement { rect: line, svg: bits::line(&[(Some((icon, tone)), title, palette.ink, false)], f32::from(line.width) * cw, ch, ch / 21.0, palette).svg(), paper: false });
        let chunks = wrap(&place.split_whitespace().collect::<Vec<_>>().join(" "), width.saturating_sub(4) as usize);
        for (at, chunk) in chunks.iter().enumerate() {
            if y + 2 + (at as u16) < bottom.y + bottom.height {
                put(frame, bottom.x + 5, y + 1 + at as u16, width.saturating_sub(3), vec![Span::styled(chunk.clone(), inks.dim)]);
            }
        }
        y += chunks.len() as u16 + 2;
    }
}

/// `text` broken at spaces into lines of at most `count` characters.
fn wrap(text: &str, count: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in text.split(' ') {
        match out.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= count => {
                line.push(' ');
                line.push_str(word);
            }
            _ => out.push(word.to_string()),
        }
    }
    out
}

pub fn chart(area: Rect) -> Rect {
    let top = if area.width >= 120 && area.height >= 44 { 1 + TILES } else { 2 };
    layout(area, Rect { x: 0, y: top, width: area.width, height: area.height - 1 - top }).chart
}

pub fn layout(area: Rect, body: Rect) -> Layout {
    let gap = u16::from(area.height >= 30);
    let line = body.y + body.height - 1 - gap;
    let panel = Rect { x: body.x, y: body.y, width: body.width, height: line - body.y };
    let chart = Rect { x: panel.x + 1, y: panel.y + 2, width: panel.width.saturating_sub(2), height: panel.height.saturating_sub(3) };
    Layout { panel, chart, line }
}

fn roadmap(app: &App, palette: &Palette, frame: &mut Frame, inks: &Inks, body: Rect, placements: &mut Vec<Placement>) {
    let area = frame.area();
    let l = layout(area, body);
    let mut settings = app.settings(l.chart);
    settings.palette = *palette;
    let (level, lanes) = shown(&app.plan, &app.schedule, &settings);
    let first = app.schedule.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(app.today);
    let last = app.schedule.slots.iter().map(|slot| slot.planned.finish).max().unwrap_or(app.today);
    let span = format!("{} – {}", first.strftime("%b %Y"), last.strftime("%b %Y")).to_uppercase();
    let switch_w = 34;
    let room = l.panel.width.saturating_sub(4 + switch_w) as usize;
    let compared;
    let mode = if app.before.is_some() {
        "WHAT-IF · "
    } else if app.why {
        "WHY THIS DATE · "
    } else if app.compare {
        compared = app.since().map(|(snapshot, _)| format!("SINCE {} · ", snapshot.title().to_uppercase())).unwrap_or_default();
        compared.as_str()
    } else {
        ""
    };
    let titles = if lanes.len() < app.plan.lanes.len() {
        let part = format!("LANES {}–{} OF {}", lanes.first().map_or(0, |lane| lane + 1), lanes.last().map_or(0, |lane| lane + 1), app.plan.lanes.len());
        vec![format!("{mode}ROADMAP · {part}"), format!("{mode}{part}")]
    } else {
        vec![format!("{mode}SWIMLANE ROADMAP · {span}"), format!("{mode}ROADMAP · {span}")]
    };
    let title = titles.into_iter().find(|title| title.chars().count() + 3 <= room).unwrap_or_else(|| format!("{mode}ROADMAP"));
    panel(frame, inks, l.panel, &title, true);

    let mut switch = vec![Span::styled("Detail ", inks.muted)];
    for (name, option) in [("Overview", Level::Overview), ("Standard", Level::Standard), ("Full", Level::Full)] {
        let style = if option == level {
            bold(inks.bright).bg(inks.raised)
        } else if option == app.level {
            inks.dim.add_modifier(Modifier::UNDERLINED)
        } else {
            inks.dim
        };
        switch.push(Span::styled(format!(" {name} "), style));
    }
    let w = width(&switch);
    if l.panel.width > w + 4 + 12 {
        put(frame, l.panel.x + l.panel.width - w - 2, l.panel.y + 1, w, switch);
    }

    let (cw, ch) = app.cell;
    if l.chart.width > 0 && l.chart.height > 0 {
        let mut scene = match &app.before {
            Some(before) => pchart_draw::what_if(&app.plan, &app.schedule, &app.calendars, &settings, &before.schedule),
            None if app.why => pchart_draw::why(&app.plan, &app.schedule, &app.calendars, &settings, app.selected),
            None if app.compare && app.then().is_some() => pchart_draw::what_if(&app.plan, &app.schedule, &app.calendars, &settings, &app.then().expect("a snapshot")),
            None => pchart_draw::roadmap(&app.plan, &app.schedule, &app.calendars, &settings),
        };
        scene.background = None;
        if app.question.is_some() {
            let w = l.chart.width.saturating_sub(4).min(94);
            let h = l.chart.height.min(24);
            let x = (l.chart.width - w) / 2;
            crate::question::dialog(app, &mut scene, f32::from(x) * f32::from(cw), 0.0, f32::from(w) * f32::from(cw), f32::from(h) * f32::from(ch), f32::from(ch) / 21.0);
        }
        if app.form.is_some() {
            let w = l.chart.width.saturating_sub(4).min(94);
            let h = l.chart.height.min(match app.form {
                Some(Form::Rename { .. }) => 9,
                Some(Form::Snapshot { .. }) => 11,
                Some(Form::Remove { item }) => match app.removal(item) {
                    Ok((_, words)) => (255.0 + 28.0 * words.len().max(1) as f32) / 21.0,
                    Err(_) => 8.0,
                }
                .ceil() as u16,
                Some(Form::Add(_)) => 20,
                Some(Form::Who { item, .. }) => who_height(app, item),
                Some(Form::Report { .. }) => 16,
                _ => 32,
            });
            let x = (l.chart.width - w) / 2;
            crate::item::dialog(app, &mut scene, f32::from(x) * f32::from(cw), 0.0, f32::from(w) * f32::from(cw), f32::from(h) * f32::from(ch), f32::from(ch) / 21.0);
        }
        if writing(app) {
            let w = l.chart.width.saturating_sub(4).min(94);
            let h = l.chart.height.min(if matches!(app.spot(), Some(Spot::Item(index)) if app.plan.items[index].milestone) { 14 } else { 12 });
            let x = (l.chart.width - w) / 2;
            crate::note::dialog(app, &mut scene, f32::from(x) * f32::from(cw), f32::from(ch) * 3.0_f32.min(f32::from(l.chart.height - h)), f32::from(w) * f32::from(cw), f32::from(h) * f32::from(ch), f32::from(ch) / 21.0);
        }
        placements.push(Placement { rect: l.chart, svg: scene.svg(), paper: false });
    }
    let rect = Rect { x: l.panel.x + 2, y: l.line, width: l.panel.width.saturating_sub(4), height: 1 };
    let (w, h, u) = (f32::from(rect.width) * f32::from(cw), f32::from(ch), f32::from(ch) / 21.0);
    let line = match app.spot() {
        _ if app.before.is_some() => bits::line_of_runs(&trial(app, palette), w, h, u, palette),
        _ if app.why => bits::line(&reason(app, palette), w, h, u, palette),
        Some(Spot::Place(lane, date)) => bits::line(&place(app, lane, date, palette), w, h, u, palette),
        _ => bits::line(&selection(app, level, palette), w, h, u, palette),
    };
    placements.push(Placement { rect, svg: line.svg(), paper: false });
}

/// The line under a what-if (picture 38): the selected item and how its length or its date changed,
/// then what the tried plan means for the target and when the work has to start.
fn trial(app: &App, palette: &Palette) -> Vec<MixedPart> {
    let p = *palette;
    let item = &app.plan.items[app.selected];
    let when = |schedule: &pchart_core::Schedule| {
        let span = schedule.slots[app.selected].planned;
        if item.milestone { span.finish } else { span.start }
    };
    let mut parts = vec![(None, vec![(item.name.clone(), p.bright, true)])];
    let before = app.before.as_ref();
    let length = before.map(|before| before.plan.items[app.selected].duration).filter(|&old| old != item.duration);
    let moved = before.map(|before| when(&before.schedule)).filter(|&old| old != when(&app.schedule));
    if let Some(old) = length {
        parts.push((Some((Icon::Clock, p.dim)), vec![(format!("{old} → "), p.dim, false), (format!("{} wd", item.duration), p.bright, true)]));
    } else if let Some(old) = moved {
        parts.push((Some((Icon::Calendar, p.dim)), vec![(format!("{} → ", day(old)), p.dim, false), (day(when(&app.schedule)), p.bright, true)]));
    } else if item.milestone {
        parts.push((Some((Icon::Calendar, p.dim)), vec![(day(when(&app.schedule)), p.ink, false)]));
    } else {
        parts.push((Some((Icon::Clock, p.dim)), vec![(format!("{} wd", item.duration), p.ink, false)]));
    }
    if let Some(verdict) = &app.analysis.verdict {
        let (icon, tone) = match verdict.state {
            State::Holds => (Icon::Check, p.bright),
            State::Tight => (Icon::Clock, p.warn),
            State::DoesNotHold => (Icon::Cross, p.alert),
        };
        let mut runs = vec![(format!("{} is ", app.plan.items[verdict.target].name), p.ink, false)];
        if verdict.state == State::DoesNotHold {
            runs.push((format!("{} wd late", -verdict.spare), p.alert, true));
        } else {
            runs.push((pchart_draw::situation::state_words(verdict.state).to_string(), tone, true));
            runs.push((format!(" · {} wd to spare", verdict.spare), p.ink, false));
        }
        if let (Some(first), Some(start)) = (app.schedule.first.filter(|_| verdict.state != State::DoesNotHold), app.schedule.start_by) {
            let name = &app.plan.items[first].name;
            let when = match start.cmp(&app.today) {
                std::cmp::Ordering::Equal => format!("{name} must start today"),
                std::cmp::Ordering::Greater => format!("{name} must start by {}", day(start)),
                std::cmp::Ordering::Less => format!("{name} should have started {}", day(start)),
            };
            runs.push((format!(" · {when}"), p.ink, false));
        }
        parts.push((Some((icon, tone)), runs));
    }
    parts
}

/// The line in "Why this date?" (picture 56): the selection and its date, the items before it that set
/// it, from the first, and what it has to spare. A long chain keeps its first and its last two items.
fn reason(app: &App, palette: &Palette) -> Vec<Part> {
    let p = *palette;
    let item = &app.plan.items[app.selected];
    let slot = &app.schedule.slots[app.selected];
    let chain = pchart_core::why(&app.schedule, app.selected);
    let names: Vec<&str> = chain[..chain.len() - 1].iter().map(|&index| app.plan.items[index].name.as_str()).collect();
    let when = if item.milestone { format!("on {}", day(slot.planned.finish)) } else { format!("starts {}", day(slot.planned.start)) };
    let after = match names.len() {
        0 => "nothing comes before it".to_string(),
        1..=4 => format!("after {}", names.join(" → ")),
        count => format!("after {} → … → {} → {}", names[0], names[count - 2], names[count - 1]),
    };
    let spare = if slot.spare < 0 {
        (Some((Icon::Bar, p.alert)), format!("{} wd late", -slot.spare), p.alert, false)
    } else if app.schedule.chain.contains(&app.selected) {
        (Some((Icon::Bar, p.ink)), format!("decides the date · {} wd to spare", slot.spare), p.ink, false)
    } else {
        (Some((Icon::Bar, p.muted)), format!("{} wd to spare", slot.spare), p.ink, false)
    };
    vec![(None, item.name.clone(), p.bright, true), (Some((Icon::Calendar, p.dim)), when, p.ink, false), (None, after, p.ink, false), spare]
}

/// The rows of the who dialog: its title, the people with a heading for each group, and air.
fn who_height(app: &App, item: usize) -> u16 {
    let groups = if app.crew.is_some() { 2.0 } else { 0.0 };
    ((170.0 + 30.0 * (app.who_rows(item).len() as f32 + groups)) / 21.0).ceil() as u16
}

/// The line about the selection: its name, its team, its dates and length, and whether it decides the
/// date or what it has to spare; at the Overview level the same for its lane.
fn selection(app: &App, level: Level, palette: &Palette) -> Vec<Part> {
    let p = *palette;
    let lane = &app.plan.lanes[app.plan.lane(&app.plan.items[app.selected].lane).unwrap_or(0)];
    let team = [Some(lane.name.as_str()), lane.city.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
    let decides = |yes: bool| if yes { (Some((Icon::Bar, p.ink)), "decides the date".to_string(), p.ink, false) } else { (Some((Icon::Bar, p.muted)), "time to spare".to_string(), p.ink, false) };
    if level == Level::Overview {
        let (name, folded, from, to, _, on_chain) = folded_lane(app);
        return vec![(None, name, p.bright, true), (None, format!("Lane · {folded}"), p.muted, false), (Some((Icon::Calendar, p.dim)), format!("{} – {}", day(from), day(to)), p.ink, false), decides(on_chain)];
    }
    let item = &app.plan.items[app.selected];
    let slot = &app.schedule.slots[app.selected];
    let dates = if item.milestone { day(slot.planned.finish) } else { format!("{} – {}", day(slot.planned.start), day(slot.planned.finish)) };
    let star = item.important.then_some((Icon::Star, p.ink));
    let mut parts = vec![(star, item.name.clone(), p.bright, true)];
    if !item.milestone {
        match &item.who {
            Some(who) => {
                let share = item.share.filter(|&share| share != 100).map(|share| format!("{share} %"));
                parts.push((Some((Icon::Person, p.dim)), [Some(who.clone()), share, lane.city.clone()].into_iter().flatten().collect::<Vec<_>>().join(" · "), p.ink, false));
            }
            None => parts.push((Some((Icon::Person, p.dim)), team, p.dim, false)),
        }
    }
    parts.push((Some((Icon::Calendar, p.dim)), dates, p.ink, false));
    if app.compare
        && let Some((snapshot, _)) = app.since()
    {
        let was = match snapshot.items.get(&item.id) {
            None => Some("new since then".to_string()),
            Some(&(start, finish)) if (start, finish) != (slot.planned.start, slot.planned.finish) => Some(format!("was {}", if item.milestone { day(finish) } else { format!("{} – {}", day(start), day(finish)) })),
            Some(_) => None,
        };
        if let Some(was) = was {
            parts.push((None, was, p.muted, false));
        }
    }
    let note = item.note.clone().map(|note| (Some((Icon::Note, p.dim)), note, p.ink, false));
    if app.notes.is_some() {
        return parts.into_iter().chain(note).collect();
    }
    if !item.milestone {
        parts.push((Some((Icon::Clock, p.dim)), format!("{} wd", item.duration), p.ink, false));
    }
    if slot.spare < 0 {
        parts.push((Some((Icon::Bar, p.alert)), format!("{} wd late", -slot.spare), p.alert, false));
    } else if app.schedule.chain.contains(&app.selected) {
        parts.push(decides(true));
    } else {
        parts.push((Some((Icon::Bar, p.muted)), format!("{} wd to spare", slot.spare), p.ink, false));
    }
    parts.into_iter().chain(note).collect()
}

/// The line under the notes cursor on an empty place (picture 43): the lane and its city, the day and
/// its week, and the note there.
fn place(app: &App, lane: usize, date: Date, palette: &Palette) -> Vec<Part> {
    let p = *palette;
    let lane_of = &app.plan.lanes[lane];
    let team = [Some(lane_of.name.as_str()), lane_of.city.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
    let mut parts = vec![(Some((Icon::Person, p.dim)), team, p.bright, true), (Some((Icon::Calendar, p.dim)), format!("{} · W{:02}", date.strftime("%a %d %b %Y"), date.iso_week_date().week()), p.ink, false)];
    parts.extend(app.note_here().map(|note| (Some((Icon::Note, p.dim)), note.to_string(), p.ink, false)));
    parts
}

/// A note is being written.
fn writing(app: &App) -> bool {
    app.notes.as_ref().is_some_and(|notes| notes.typed.is_some())
}

pub(crate) fn day(date: Date) -> String {
    date.strftime("%a %d %b").to_string()
}

/// The selected item's lane at the Overview level: its name, what it folds, its span, the items it
/// stands for (its milestones when it has nothing else), and whether one of them decides the date.
fn folded_lane(app: &App) -> (String, String, Date, Date, Vec<usize>, bool) {
    let lane = app.plan.lane(&app.plan.items[app.selected].lane).unwrap_or(0);
    let mine: Vec<usize> = (0..app.plan.items.len()).filter(|&index| app.plan.lane(&app.plan.items[index].lane) == Some(lane)).collect();
    let tasks: Vec<usize> = mine.iter().copied().filter(|&index| !app.plan.items[index].milestone).collect();
    let folded = match (tasks.len(), mine.len()) {
        (0, 1) => "1 milestone".to_string(),
        (0, count) => format!("{count} milestones"),
        (1, _) => "1 item folded".to_string(),
        (count, _) => format!("{count} items folded"),
    };
    let shown = if tasks.is_empty() { mine } else { tasks };
    let from = shown.iter().map(|&index| app.schedule.slots[index].planned.start).min().unwrap_or(app.today);
    let to = shown.iter().map(|&index| app.schedule.slots[index].planned.finish).max().unwrap_or(app.today);
    let decides = shown.iter().any(|index| app.schedule.chain.contains(index));
    (app.plan.lanes[lane].name.clone(), folded, from, to, shown, decides)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pchart_core::{Holidays, Plan};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn app() -> App {
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
        let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        let today = jiff::civil::date(2026, 10, 29);
        let schedule = pchart_core::schedule(&plan, &calendars, today).unwrap();
        App::new(plan, calendars, today, "portal.pchart".into(), schedule, (7, 16))
    }

    fn undated() -> App {
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap().structure_only();
        let calendars = vec![pchart_core::Calendar::default(); plan.lanes.len()];
        let today = jiff::civil::date(2026, 10, 29);
        let schedule = pchart_core::schedule(&plan, &calendars, today).unwrap();
        App::new(plan, calendars, today, "portal.pchart".into(), schedule, (7, 16))
    }

    /// The sample kept as "Kick-off" on Thu 22 Oct with Frontend at 22 working days (pictures 67), and
    /// opened on Thu 29 Oct with Frontend at 25.
    fn snapshotted() -> App {
        let mut plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
        let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        let frontend = plan.item("frontend").unwrap();
        plan.items[frontend].duration = 22;
        let then = pchart_core::schedule(&plan, &calendars, jiff::civil::date(2026, 10, 22)).unwrap();
        let snapshot = pchart_core::snapshot::Snapshot::take(&plan, &then, jiff::civil::date(2026, 10, 22), Some("Kick-off".into()));
        plan.keep(snapshot);
        plan.items[frontend].duration = 25;
        let today = jiff::civil::date(2026, 10, 29);
        let schedule = pchart_core::schedule(&plan, &calendars, today).unwrap();
        App::new(plan, calendars, today, "portal.pchart".into(), schedule, (7, 16))
    }

    /// The sample plan in the workspace Acme of pictures 64: its people and teams, Lena Vogel on UX
    /// concept here and on Visual design in a website relaunch at once, and Priya Nair away during
    /// Frontend once she does it.
    fn crewed() -> App {
        let mut app = app();
        for (item, who, share) in [("requirements", "Jonas Weber", Some(30)), ("ux-concept", "Lena Vogel", Some(80)), ("ui-design", "Lena Vogel", None), ("api", "Tom Becker", Some(60)), ("integration", "Arjun Rao", None)] {
            let index = app.plan.item(item).unwrap();
            (app.plan.items[index].who, app.plan.items[index].share) = (Some(who.into()), share);
        }
        let template = pchart_store::templates().into_iter().find(|(id, _)| *id == "website-relaunch").unwrap().1;
        let mut website = template.new_plan("Website relaunch", Some(jiff::civil::date(2027, 1, 15)), false);
        for (item, who, share) in [("wireframes", "Mia Klein", Some(80)), ("visual-design", "Lena Vogel", Some(50)), ("templates", "Arjun Rao", Some(60))] {
            let index = website.item(item).unwrap();
            (website.items[index].who, website.items[index].share) = (Some(who.into()), share);
        }
        let calendars = vec![pchart_core::Calendar::default(); website.lanes.len()];
        let schedule = pchart_core::schedule(&website, &calendars, app.today).unwrap();
        let workspace = pchart_core::people::Workspace::parse(
            r#"name = "Acme"
            teams = [{ name = "Product", city = "Berlin" }, { name = "Design/UX", city = "Berlin" }, { name = "Backend/API", city = "Berlin" }, { name = "Frontend", city = "Bangalore" }]
            people = [
              { name = "Jonas Weber", team = "Product" },
              { name = "Lena Vogel", team = "Design/UX" },
              { name = "Mia Klein", team = "Design/UX", capacity = 80 },
              { name = "Tom Becker", team = "Backend/API" },
              { name = "Priya Nair", team = "Frontend", away = [{ from = 2027-01-04, to = 2027-01-08 }] },
              { name = "Arjun Rao", team = "Frontend" },
            ]"#,
        )
        .unwrap();
        let here = pchart_core::people::Project { name: app.plan.name.clone(), plan: app.plan.clone(), schedule: app.schedule.clone(), calendars: app.calendars.clone() };
        let other = pchart_core::people::Project { name: website.name.clone(), plan: website, schedule, calendars };
        app.join(crate::app::Crew { name: "Acme".into(), workspace, projects: vec![here, other], here: 0 });
        app
    }

    /// `crewed` with a workspace folder of its own and two reports in it: Priya Nair says Frontend needs
    /// 3 more working days, and Tom Becker that Data model is at risk.
    fn reported(name: &str) -> (App, std::path::PathBuf) {
        let mut app = crewed();
        let dir = std::env::temp_dir().join(format!("pchart-tui-reports-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let report = |item: &str, by: &str, at: &str, status, days, comment: &str| pchart_core::report::Report { project: "portal.pchart".into(), item: item.into(), by: by.into(), at: at.parse().unwrap(), status, days, comment: Some(comment.into()), answer: None, taken: None };
        pchart_store::report(&dir, &report("data-model", "Tom Becker", "2026-10-28T17:05", pchart_core::report::Status::Risk, 0, "The schema review moved to next week")).unwrap();
        pchart_store::report(&dir, &report("frontend", "Priya Nair", "2026-10-29T08:40", pchart_core::report::Status::Later, 3, "Backend API comes late")).unwrap();
        app.root = Some(dir.clone());
        app.refresh_reports();
        app.message = None;
        (app, dir)
    }

    fn screen(app: &App, width: u16, height: u16) -> (Vec<String>, Vec<Placement>) {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let mut placements = Vec::new();
        terminal.draw(|frame| placements = draw(app, frame)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        ((0..height).map(|y| (0..width).map(|x| buffer[(x, y)].symbol().to_string()).collect()).collect(), placements)
    }

    fn has(rows: &[String], text: &str) -> bool {
        rows.iter().any(|row| row.contains(text))
    }

    #[test]
    fn no_text_runs_over_the_edge_of_its_shape_on_any_screen() {
        let output = pchart_draw::Output::default();
        let mut found = Vec::new();
        let mut look = |app: &mut App, what: &str| {
            for cell in [(8, 17), (10, 21), (7, 16)] {
                app.cell = cell;
                for (width, height) in [(170, 50), (182, 43), (150, 44), (120, 40), (91, 43), (80, 24), (200, 60)] {
                    for placement in screen(app, width, height).1 {
                        found.extend(output.crossings(&placement.svg).unwrap().into_iter().map(|crossing| format!("{what} {width}x{height} {cell:?}: {crossing}")));
                    }
                }
            }
        };
        for shown in [Screen::Situation, Screen::Roadmap, Screen::Checks, Screen::Export] {
            let mut app = app();
            app.screen = shown;
            look(&mut app, &format!("{shown:?}"));
            app.help = true;
            look(&mut app, &format!("{shown:?} help"));
            app.help = false;
            for typed in ["", "move launch"] {
                app.finder = Some(crate::app::Finder { typed: typed.into(), chosen: 0 });
                look(&mut app, &format!("{shown:?} finder \"{typed}\""));
            }
        }
        for shown in [Screen::Situation, Screen::Roadmap] {
            let mut app = snapshotted();
            app.screen = shown;
            look(&mut app, &format!("since {shown:?}"));
        }
        let mut kept = snapshotted();
        kept.toggle_compare();
        kept.selected = kept.plan.item("frontend").unwrap();
        look(&mut kept, "compared");
        kept.start_snapshot();
        "Steering committee".chars().for_each(|c| kept.form_type(c));
        look(&mut kept, "snapshot dialog");
        for view in pchart_draw::View::ALL {
            let mut app = app();
            app.screen = Screen::Export;
            app.export.view = view;
            look(&mut app, &format!("export {view:?}"));
        }
        for shown in [Screen::Situation, Screen::Roadmap, Screen::Checks, Screen::Export] {
            let mut app = undated();
            app.screen = shown;
            look(&mut app, &format!("no dates {shown:?}"));
            app.finder = Some(crate::app::Finder { typed: String::new(), chosen: 0 });
            look(&mut app, &format!("no dates {shown:?} finder"));
            app.finder = None;
            app.help = true;
            look(&mut app, &format!("no dates {shown:?} help"));
        }
        let mut net = undated();
        for (dialog, start) in [("length", App::start_length as fn(&mut App)), ("add", App::start_add), ("rename", App::start_rename), ("waits on", App::start_waits), ("remove", App::start_remove)] {
            for item in ["frontend", "launch", "requirements"] {
                net.selected = net.plan.item(item).unwrap();
                start(&mut net);
                look(&mut net, &format!("network {dialog} {item}"));
                net.form = None;
            }
        }
        let mut app = app();
        app.screen = Screen::Roadmap;
        for level in [Level::Overview, Level::Full, Level::Standard] {
            app.level = level;
            look(&mut app, &format!("roadmap {level:?}"));
        }
        for item in ["frontend", "launch", "requirements", "sdk-integration"] {
            app.selected = app.plan.item(item).unwrap();
            look(&mut app, item);
            for (dialog, start) in [("add", App::start_add as fn(&mut App)), ("rename", App::start_rename), ("waits on", App::start_waits), ("remove", App::start_remove)] {
                start(&mut app);
                look(&mut app, &format!("{dialog} {item}"));
                app.form = None;
            }
            app.ask();
            look(&mut app, &format!("question {item}"));
            app.question = None;
            app.toggle_details(200);
            look(&mut app, &format!("details {item}"));
            app.side = false;
            app.why = true;
            look(&mut app, &format!("why {item}"));
            app.why = false;
        }
        let mut app = crewed();
        app.selected = app.plan.item("frontend").unwrap();
        app.start_who();
        app.form_move(1);
        app.keep_form();
        app.screen = Screen::Roadmap;
        look(&mut app, "workspace roadmap");
        for item in ["frontend", "ux-concept", "requirements"] {
            app.selected = app.plan.item(item).unwrap();
            app.start_who();
            look(&mut app, &format!("who {item}"));
            app.form_change(-3);
            look(&mut app, &format!("who {item} 70 %"));
            app.form = None;
        }
        app.screen = Screen::Checks;
        for check in 0..app.checks().len() {
            app.check = check;
            look(&mut app, &format!("workspace check {check}"));
        }
        app.screen = Screen::Situation;
        look(&mut app, "workspace situation");
        app.screen = Screen::People;
        for (line, week) in [(0, 0), (3, 5), (2, 5), (8, 9), (9, 30)] {
            (app.person, app.week) = (line, week);
            look(&mut app, &format!("people {line} {week}"));
        }
        app.help = true;
        look(&mut app, "people help");
        let (mut app, dir) = reported("edges");
        app.screen = Screen::Situation;
        look(&mut app, "situation with reports");
        app.screen = Screen::Reports;
        for report in 0..2 {
            app.report = report;
            look(&mut app, &format!("reports {report}"));
        }
        app.help = true;
        look(&mut app, "reports help");
        app.help = false;
        app.screen = Screen::Roadmap;
        app.selected = app.plan.item("frontend").unwrap();
        app.start_report();
        app.form_change(2);
        look(&mut app, "report dialog");
        app.form = None;
        app.screen = Screen::Reports;
        app.report = 0;
        app.edit_report();
        look(&mut app, "change, then take");
        app.close_form();
        app.take_report();
        app.decline_report();
        look(&mut app, "reports all answered");
        std::fs::remove_dir_all(dir).unwrap();
        assert!(found.is_empty(), "{} crossings:\n{}", found.len(), found.join("\n"));
    }

    #[test]
    fn in_a_workspace_people_are_chosen_with_their_free_time_and_checked_across_projects() {
        let mut app = crewed();
        let checks: Vec<String> = app.analysis.checks.iter().map(|check| pchart_core::text::check(&app.plan, check).0).collect();
        assert!(checks.contains(&"Lena Vogel is planned at 130 %.".to_string()), "{checks:?}");
        assert!(!app.analysis.checks.iter().any(|check| matches!(check, pchart_core::Check::Overlap { .. })));
        assert!(app.analysis.next.iter().any(|next| matches!(next, pchart_core::Next::Check(index) if matches!(app.analysis.checks[*index], pchart_core::Check::Overload { .. }))));

        app.screen = Screen::Roadmap;
        app.selected = app.plan.item("frontend").unwrap();
        app.start_who();
        assert_eq!(app.who_rows(app.selected)[..2], [Some("Priya Nair".to_string()), Some("Arjun Rao".to_string())]);
        app.form_change(-6);
        let (rows, placements) = screen(&app, 170, 50);
        assert!(has(&rows, "←→ share"));
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        for words in ["Who does Frontend?", "40 %", "FRONTEND · BANGALORE", "OTHER TEAMS", "away Mon 04 Jan – Fri 08 Jan", "40 % free then", "No one named"] {
            assert!(svg.contains(words), "{words}");
        }
        app.form_move(1);
        app.keep_form();
        let frontend = &app.plan.items[app.selected];
        assert_eq!((frontend.who.as_deref(), frontend.share), (Some("Priya Nair"), Some(40)));
        assert!(app.analysis.checks.iter().any(|check| matches!(check, pchart_core::Check::Away { who, days: 5, .. } if who == "Priya Nair")));
        let svg: String = screen(&app, 170, 50).1.iter().map(|placement| placement.svg.clone()).collect();
        assert!(svg.contains("Priya Nair · 40 % · Bangalore"));
        app.undo();
        assert_eq!(app.plan.items[app.selected].who, None);

        app.screen = Screen::People;
        (app.person, app.week) = (3, 5);
        let (rows, placements) = screen(&app, 170, 50);
        assert!(has(&rows, "5 People") && has(&rows, ":: CAPACITY · ACME") && has(&rows, ":: WORK · LENA VOGEL") && has(&rows, "←→ week"));
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(svg.contains("130 %") && svg.contains("DESIGN/UX · BERLIN") && svg.contains("Mia Klein") && svg.contains("130 % full · 130 % of 100 % planned, 30 % too much"), "{svg}");
        app.person = 2;
        let svg: String = screen(&app, 170, 50).1.iter().map(|placement| placement.svg.clone()).collect();
        assert!(svg.contains("72 % full · 130 % of 180 % planned, 50 % free"));
        app.crew = None;
        assert!(!has(&screen(&app, 170, 50).0, "5 People"));
    }

    #[test]
    fn reports_wait_for_the_project_lead_who_takes_changes_or_declines_them() {
        let (mut app, dir) = reported("answer");
        let frontend = app.plan.item("frontend").unwrap();
        let length = app.plan.items[frontend].duration;
        assert_eq!(app.open_reports().len(), 2);
        assert_eq!(app.steps()[..2], [crate::app::Step::Next(0), crate::app::Step::Report(0)]);
        let rows = screen(&app, 170, 50).0;
        assert!(has(&rows, "6 Reports ●2"));
        app.screen = Screen::Reports;
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, ":: REPORTS · 2 TO ANSWER") && has(&rows, ":: IF YOU TAKE IT") && has(&rows, "e change, then take"), "{rows:?}");
        assert!(svg.contains("Priya Nair: Frontend needs 3 more working days") && svg.contains(&format!("{} working days", length + 3)) && svg.contains("Tom Becker: Data model at risk"));

        app.take_report();
        assert_eq!((app.plan.items[frontend].duration, app.open_reports().len()), (length + 3, 1));
        assert!(app.reports.iter().any(|(_, report)| report.item == "frontend" && report.answer == Some(pchart_core::report::Answer::Taken)));
        app.undo();
        assert_eq!((app.plan.items[frontend].duration, app.open_reports().len()), (length, 2), "undoing it opens the report again");
        app.take_report();

        app.decline_report();
        let data_model = app.plan.item("data-model").unwrap();
        assert_eq!((app.open_reports().len(), app.plan.items[data_model].note.clone()), (0, None));

        std::fs::write(dir.join("reports/2026-10-30-0900-priya-nair-frontend.toml"), "project = \"portal.pchart\"\nitem = \"frontend\"\nby = \"Priya Nair\"\nat = 2026-10-30T09:00:00\nstatus = \"later\"\ndays = 3\n").unwrap();
        app.refresh_reports();
        assert_eq!(app.message.as_deref(), Some("Priya Nair reports: Frontend needs 3 more working days"));
        app.edit_report();
        assert_eq!(app.screen, Screen::Roadmap);
        let svg: String = screen(&app, 170, 50).1.iter().map(|placement| placement.svg.clone()).collect();
        assert!(svg.contains("What do you take from Priya Nair?") && svg.contains("reported needs 3 more working days"));
        app.form_back();
        app.form_type('2');
        app.keep_form();
        assert_eq!((app.screen, app.plan.items[frontend].duration), (Screen::Reports, length + 3 + 2));
        let changed = app.reports.iter().find(|(path, _)| path.ends_with("2026-10-30-0900-priya-nair-frontend.toml")).unwrap().1.clone();
        assert_eq!(changed.taken.map(|taken| taken.days), Some(2));

        app.screen = Screen::Roadmap;
        app.start_report();
        app.form_change(1);
        app.form_move(1);
        app.form_move(1);
        for c in "Tests start Monday".chars() {
            app.form_type(c);
        }
        let before = app.plan.clone();
        app.keep_form();
        assert_eq!(app.plan, before);
        assert!(app.reports.iter().any(|(_, report)| report.status == pchart_core::report::Status::Risk && report.comment.as_deref() == Some("Tests start Monday") && report.answer.is_none()));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn only_the_lead_answers_reports_when_the_workspace_names_one() {
        let (mut app, dir) = reported("lead");
        app.crew.as_mut().unwrap().workspace.lead = Some("Anna Lead".into());
        app.me = Some("Priya Nair".into());
        app.screen = Screen::Reports;
        let frontend = app.plan.item("frontend").unwrap();
        let length = app.plan.items[frontend].duration;
        app.take_report();
        app.edit_report();
        app.decline_report();
        assert_eq!((app.plan.items[frontend].duration, app.open_reports().len(), app.message.as_deref()), (length, 2, Some("Anna Lead answers reports")));
        let (rows, placements) = screen(&app, 170, 50);
        assert!(!has(&rows, "Enter take it") && placements.iter().any(|placement| placement.svg.contains("Anna Lead answers it; if they take it:")));
        app.me = Some("Anna Lead".into());
        app.take_report();
        assert_eq!(app.plan.items[frontend].duration, length + 3);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_plan_without_lengths_opens_with_how_it_is_built_and_says_what_is_missing() {
        let mut app = undated();
        assert_eq!((app.screen, app.export.view, app.formats()), (Screen::Roadmap, pchart_draw::View::Network, 3));
        let rows = screen(&app, 170, 50).0;
        assert!(has(&rows, "2 Network") && has(&rows, ":: NETWORK DIAGRAM") && has(&rows, "10 of 10 without a length") && has(&rows, "Enter length"));
        app.screen = Screen::Export;
        let (rows, placements) = screen(&app, 170, 50);
        assert!(has(&rows, ":: EXPORT") && placements.iter().any(|placement| placement.svg.contains("Network diagram")));
        assert!(placements.iter().any(|placement| !placement.svg.contains("Gantt chart") && placement.svg.contains("Dependency matrix")));
        assert!(!placements.iter().any(|placement| placement.svg.contains("Markdown")));
        for shown in [Screen::Situation, Screen::Checks] {
            app.screen = shown;
            let rows = screen(&app, 170, 50).0;
            assert!(has(&rows, ":: NO DATES YET") && has(&rows, "10 of 10 pieces of work have no length yet.") && has(&rows, "Requirements") && has(&rows, "2 how it is built"), "{shown:?}");
        }
        assert!(has(&screen(&app, 80, 24).0, "Security review"));
        let mut plan = app.plan.clone();
        let api = plan.items[plan.item("api").unwrap()].clone();
        plan.items.extend((0..10).map(|number| pchart_core::model::Item { id: format!("more-{number}"), name: format!("More {number}"), after: Vec::new(), ..api.clone() }));
        let calendars = vec![pchart_core::Calendar::default(); plan.lanes.len()];
        let schedule = pchart_core::schedule(&plan, &calendars, app.today).unwrap();
        let mut crowded = App::new(plan, calendars, app.today, "crowded.pchart".into(), schedule, (7, 16));
        crowded.screen = Screen::Situation;
        let rows = screen(&crowded, 80, 24).0;
        let more = crowded.plan.without_length().len() - 12;
        assert!(has(&rows, &format!("and {more} more")), "{rows:#?}");
        let found = crate::actions::find(&app, "");
        assert!(found.iter().all(|(action, _)| !matches!(action, crate::actions::Action::ChangeDate | crate::actions::Action::Why)));
        assert!(crate::actions::find(&app, "add").iter().any(|(action, _)| *action == crate::actions::Action::AddItem));
        assert!(crate::actions::find(&app, "gantt").iter().all(|(action, _)| *action != crate::actions::Action::Export(pchart_draw::View::Gantt)));
    }

    #[test]
    fn a_fixed_kickoff_before_the_chain_leaves_the_latest_start_to_the_work_after_it() {
        let mut app = app();
        let mut plan = app.plan.clone();
        plan.items.push(pchart_core::model::Item { id: "kickoff".into(), lane: "product".into(), name: "Kickoff".into(), milestone: true, on: Some(jiff::civil::date(2026, 11, 2)), ..Default::default() });
        let requirements = plan.item("requirements").unwrap();
        plan.items[requirements].after.push("kickoff".into());
        app.schedule = pchart_core::schedule(&plan, &app.calendars, app.today).unwrap();
        app.analysis = pchart_core::analyse(&plan, &app.calendars, &app.schedule, app.today);
        app.plan = plan;
        app.screen = Screen::Roadmap;
        assert_eq!(app.schedule.first, Some(requirements));
        let svgs: String = screen(&app, 170, 50).1.iter().map(|placement| placement.svg.clone()).collect();
        assert!(svgs.contains("16 Nov") && svgs.contains(">Requirements<"), "the tile");
        let rows = screen(&app, 110, 40).0;
        assert!(has(&rows, "start by Mon 16 Nov"), "{rows:#?}");
        app.try_length(1);
        let line = screen(&app, 170, 50).1.last().unwrap().svg.clone();
        assert!(line.contains("Requirements must start by") && !line.contains("Kickoff must"), "{line}");
    }

    #[test]
    fn in_the_network_the_arrows_move_and_enter_gives_a_length_until_the_dates_come() {
        let mut app = undated();
        let (_, boxes, _) = pchart_draw::structure::layout(&app.plan, &app.schedule);
        let start = app.plan.item("requirements").unwrap();
        app.selected = start;
        app.move_in_network(0.0, 1.0);
        assert!(boxes[app.selected][1] > boxes[start][1]);
        app.move_in_network(-1.0, 0.0);
        app.move_in_network(-1.0, 0.0);
        assert!(boxes[app.selected][0] <= boxes[start][0] + 1.0);
        app.selected = app.plan.item("launch").unwrap();
        app.start_length();
        assert!(app.form.is_none() && app.message.as_deref() == Some("Launch is a milestone; it has no length"));
        for index in app.plan.without_length() {
            assert!(!app.dated());
            app.selected = index;
            app.start_length();
            for c in "5".chars() {
                app.form_type(c);
            }
            app.keep_form();
            assert!(app.form.is_none(), "{:?}", app.message);
        }
        assert!(app.dated() && app.message.as_deref() == Some("Every piece of work has a length now"));
        assert!(has(&screen(&app, 170, 50).0, "SWIMLANE ROADMAP") && has(&screen(&app, 170, 50).0, "2 Roadmap"));
    }

    #[test]
    fn why_this_date_shows_the_chain_and_says_it_in_one_line() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        app.selected = app.plan.item("launch").unwrap();
        app.why = true;
        let (rows, placements) = screen(&app, 170, 50);
        let line = &placements.last().unwrap().svg;
        assert!(has(&rows, ":: WHY THIS DATE · SWIMLANE ROADMAP") && has(&rows, "↑↓ another item") && has(&rows, "Esc back"));
        assert!(line.contains("on Mon 01 Mar") && line.contains("after Requirements → … → Beta test → Acceptance") && line.contains("decides the date · 12 wd to spare"));
        app.selected = app.plan.item("sdk-integration").unwrap();
        let line = screen(&app, 170, 50).1.last().unwrap().svg.clone();
        assert!(line.contains("starts Thu 10 Dec") && line.contains("after Requirements → Data model") && line.contains("19 wd to spare"));
        app.selected = app.plan.item("requirements").unwrap();
        assert!(screen(&app, 170, 50).1.last().unwrap().svg.contains("nothing comes before it"));
        assert!(has(&screen(&app, 80, 24).0, ":: WHY THIS DATE · LANES"));
    }

    #[test]
    fn b_adds_the_buffer_the_check_suggests_and_u_takes_it_back() {
        let mut app = app();
        app.screen = Screen::Checks;
        app.check = 0;
        assert!(app.buffer_here() && has(&screen(&app, 170, 50).0, "b add the buffer"));
        let promised = pchart_core::text::advice(&app.plan, &app.calendars, app.today, &app.analysis.checks[app.checks()[0]]).then;
        app.add_buffer();
        let buffer = app.plan.item("buffer-before-launch").unwrap();
        assert!(app.plan.items[buffer].buffer && app.plan.items[buffer].duration == pchart_core::text::BUFFER);
        assert!(app.plan.items[app.plan.item("launch").unwrap()].after.contains(&"buffer-before-launch".to_string()));
        assert!(!app.analysis.checks.iter().any(|check| matches!(check, pchart_core::Check::NoBuffer { .. })));
        let spare = app.analysis.verdict.as_ref().unwrap().spare;
        assert!(promised.iter().any(|(text, _)| text == &format!("{spare} working days")), "{promised:?} {spare}");
        let rows = screen(&app, 80, 24).0;
        assert!(has(&rows, "Buffer added · u takes it back") && !app.buffer_here(), "{rows:#?}");
        app.undo();
        assert!(app.plan.item("buffer-before-launch").is_none() && app.buffer_here());
    }

    #[test]
    fn the_checks_screen_lists_by_weight_and_says_what_a_check_means() {
        let mut app = app();
        app.screen = Screen::Checks;
        let svg = |placements: &[Placement]| placements.iter().map(|placement| placement.svg.as_str()).collect::<String>();
        let (rows, placements) = screen(&app, 170, 50);
        let all = svg(&placements);
        assert!(has(&rows, ":: CHECKS · 1 WARNING · 1 HINT") && has(&rows, ":: IN THE ROADMAP · 18 JAN – 12 MAR 2027"));
        for text in ["1 thing to look at", "No buffer before a fixed date", "Acceptance Fri 26 Feb  →  Launch Mon 01 Mar", "Requirements phase ends without a milestone", "6 other checks found nothing", "5 holidays accounted for", "If Acceptance slips by a day, Launch is late.", "Add a 5-day buffer before Launch:", "still on track, with "] {
            assert!(all.contains(text), "{text}");
        }
        assert!(!all.contains("Its last item is Requirements") && placements.len() == 7);
        let (rows, placements) = screen(&app, 80, 24);
        assert!(!has(&rows, "IN THE ROADMAP") && svg(&placements).contains("Add a 5-day buffer before Launch:"));
        app.choose_check(1);
        let (rows, placements) = screen(&app, 170, 50);
        assert!(has(&rows, ":: IN THE ROADMAP · 12 OCT – 04 DEC 2026") && svg(&placements).contains("Add a milestone after Requirements.") && svg(&placements).contains("Its last item is Requirements, Fri 27 Nov"));
        app.show_check_in_roadmap();
        assert_eq!((app.screen, app.plan.items[app.selected].id.as_str()), (Screen::Roadmap, "requirements"));
    }

    #[test]
    fn the_export_screen_writes_what_its_preview_shows() {
        let dir = std::env::temp_dir().join(format!("pchart-export-{}", std::process::id()));
        let mut app = app();
        app.path = dir.join("portal.pchart").display().to_string();
        app.screen = Screen::Export;
        let chosen = |svg: &str, text: &str| svg.contains(&format!("font-weight=\"700\" xml:space=\"preserve\">{text}</text>"));
        let (rows, placements) = screen(&app, 170, 50);
        let options = &placements[0].svg;
        assert!(has(&rows, ":: EXPORT") && has(&rows, ":: PREVIEW · EXACTLY WHAT THE FILE LOOKS LIKE") && placements.len() == 2);
        assert!(chosen(options, "Milestone chart") && chosen(options, "Team") && chosen(options, "PDF") && !chosen(options, "PNG") && options.contains("Markdown"));
        assert!(options.contains("portal-milestones.pdf") && !options.contains("16:9"));
        app.export_change(-1);
        app.export_part(1);
        app.export_change(1);
        app.export_change(1);
        let (_, placements) = screen(&app, 170, 50);
        let options = &placements[0].svg;
        assert!(chosen(options, "Gantt chart") && chosen(options, "Client") && options.contains("hides what items have to spare,") && options.contains("warnings"));
        app.export_part(1);
        app.export_change(1);
        app.export_change(1);
        app.export_toggle();
        app.export_change(1);
        app.export_toggle();
        let names: Vec<String> = crate::export::files(&app).iter().map(|file| file.file_name().unwrap().to_string_lossy().into_owned()).collect();
        assert_eq!(names, ["portal-gantt.pdf", "portal-gantt.svg", "portal-gantt.png", "portal.md", "portal-milestones.svg"]);
        let (rows, placements) = screen(&app, 80, 24);
        assert!(placements.len() == 1 && placements[0].svg.contains("portal-gantt.pdf, .svg, .png, portal.md") && placements[0].svg.contains("The preview needs 110 columns.") && !has(&rows, "PREVIEW"));

        let written = crate::export::write(&crate::export::job(&app)).unwrap();
        assert_eq!(written, crate::export::files(&app));
        let read = |name: &str| std::fs::read(dir.join("export").join(name)).unwrap();
        assert!(read("portal-gantt.pdf").starts_with(b"%PDF") && read("portal-gantt.png").starts_with(b"\x89PNG"));
        let svg = String::from_utf8(read("portal-gantt.svg")).unwrap();
        assert!(svg.contains("Gantt chart") && !svg.contains("To spare") && !svg.contains("no buffer"));
        let md = String::from_utf8(read("portal.md")).unwrap();
        assert!(md.contains("**Launch on Mon 01 Mar is on track.**\n") && !md.contains("to spare, with") && md.contains("![Milestone chart](portal-milestones.svg)"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn small_windows_give_way_in_the_approved_order() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let line = |placements: &[Placement]| placements.last().map(|placement| placement.svg.clone()).unwrap_or_default();
        let (rows, placements) = screen(&app, 242, 61);
        assert!(rows[2].contains(":: TARGET · FIXED") && has(&rows, ":: SWIMLANE ROADMAP") && !has(&rows, ":: SELECTION") && !has(&rows, ":: CHECKS · 1 WARNING"));
        assert!(line(&placements).contains("Frontend · Bangalore") && line(&placements).contains("decides the date") && placements.len() == 6);
        let (rows, placements) = screen(&app, 120, 36);
        assert!(rows[1].contains("on track · 12 wd to spare") && line(&placements).contains("Mon 21 Dec – Wed 27 Jan"));
        let (rows, _) = screen(&app, 80, 24);
        assert!(rows.iter().any(|row| row.contains("ROADMAP · LANES") && row.contains("OF 7")));
        let (rows, placements) = screen(&app, 50, 14);
        assert!(placements.is_empty() && rows.iter().any(|row| row.contains("Terminal size too small:")) && rows.iter().any(|row| row.contains("Width = 60 Height = 16")));
    }

    #[test]
    fn a_lane_of_milestones_stands_for_its_milestones() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        app.level = Level::Overview;
        app.selected = app.plan.item("acceptance").unwrap();
        let (_, placements) = screen(&app, 170, 50);
        let line = &placements.last().unwrap().svg;
        assert!(line.contains("Lane · 3 milestones") && line.contains("Fri 18 Dec – Mon 01 Mar") && line.contains("decides the date") && !line.contains("0 items"));
    }

    #[test]
    fn enter_on_a_statement_shows_its_item_in_the_roadmap() {
        let mut app = app();
        app.choose(1);
        app.show_in_roadmap();
        assert_eq!((app.screen, app.plan.items[app.selected].id.as_str()), (Screen::Roadmap, "acceptance"));
        app.screen = Screen::Situation;
        app.choose(1);
        app.show_in_roadmap();
        assert_eq!(app.plan.items[app.selected].id, "requirements");
    }

    #[test]
    fn the_situation_says_where_we_stand_what_is_next_and_what_decides_the_date() {
        let app = app();
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, ":: WHERE DO WE STAND") && has(&rows, ":: NEXT") && has(&rows, ":: THIS CHAIN DECIDES THE DATE") && placements.len() == 4);
        for text in ["Launch on Mon 01 Mar is ", "on track.", "working days to spare", "Start Requirements by Mon 16 Nov", "in 12 working days", "Product · Berlin", "Add a buffer before Launch", "Acceptance Fri 26 Feb  →  Launch Mon 01 Mar"] {
            assert!(svg.contains(text), "{text}");
        }
        assert!(!has(&rows, "WHO IS WAITING") && !svg.contains("NEXT 6 WEEKS") && !svg.contains("does not hold"));
        let (rows, placements) = screen(&app, 80, 24);
        assert!(has(&rows, ":: NEXT") && !has(&rows, "THIS CHAIN DECIDES THE DATE") && placements.len() == 3);
    }

    #[test]
    fn the_date_question_changes_a_start_and_u_takes_it_back() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let frontend = app.plan.item("frontend").unwrap();
        app.selected = frontend;
        app.ask();
        assert!(app.answer().is_none() && app.question.as_ref().is_some_and(|question| question.choice == 0 && question.typed.is_empty()));
        let question = app.question.as_mut().unwrap();
        question.choice = 2;
        question.typed = "4 jan".into();
        let date = jiff::civil::date(2027, 1, 4);
        assert_eq!(app.answer(), Some(pchart_core::Edit::Fix { item: "frontend".into(), kind: pchart_core::Fix::NoEarlierThan, date }));
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, "CHANGE A DATE · nothing changed yet") && has(&rows, "keep as it is"));
        for text in ["When does Frontend start?", "After Design sign-off", "not before this date, later if the plan needs it", "Mon 04 Jan 2027", "Launch would be ", "9 working days late.", " moves to "] {
            assert!(svg.contains(text), "{text}");
        }

        app.confirm();
        assert!(app.question.is_none() && app.unsaved && app.history.len() == 1);
        assert_eq!(app.plan.items[frontend].no_earlier_than, Some(date));
        assert_eq!(app.analysis.verdict.as_ref().map(|verdict| (verdict.state, verdict.spare)), Some((pchart_core::analysis::State::DoesNotHold, -9)));
        app.ask();
        let question = app.question.as_mut().unwrap();
        assert!(question.choice == 2 && question.typed == "04 Jan 2027");
        question.choice = 0;
        assert_eq!(app.answer(), Some(pchart_core::Edit::Unfix { item: "frontend".into() }));
        app.question.as_mut().unwrap().choice = 1;
        app.question.as_mut().unwrap().typed = "soon".into();
        app.confirm();
        assert!(app.question.is_some() && app.history.len() == 1 && app.message.as_deref() == Some("Type a date, such as 4 jan or next friday"));
        app.question = None;

        app.unsaved = false;
        app.undo();
        assert!(app.unsaved && app.history.is_empty() && app.plan.items[frontend].no_earlier_than.is_none() && app.message.as_deref() == Some("Undone: Frontend start"));
        assert_eq!(app.analysis.verdict.as_ref().map(|verdict| verdict.state), Some(pchart_core::analysis::State::Holds));
        app.undo();
        assert_eq!(app.message.as_deref(), Some("Nothing to undo"));
    }

    #[test]
    fn a_what_if_writes_nothing_until_enter_keeps_it() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let frontend = app.plan.item("frontend").unwrap();
        app.selected = frontend;
        let (_, placements) = screen(&app, 170, 50);
        let dashes = |placements: &[Placement]| placements.iter().map(|placement| placement.svg.matches("stroke-dasharray").count()).sum::<usize>();
        let before = dashes(&placements);
        for _ in 0..12 {
            app.try_length(1);
        }
        assert!(app.before.is_some() && !app.unsaved && app.history.is_empty() && app.changes() == ["Frontend +12 wd"]);
        assert_eq!(app.analysis.verdict.as_ref().map(|verdict| (verdict.state, verdict.spare)), Some((pchart_core::analysis::State::Tight, 0)));
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        for text in ["WHAT-IF · Frontend +12 wd · not saved", ":: WHAT-IF · SWIMLANE ROADMAP", "today", "was 12", "Esc discard", "←→ keep changing"] {
            assert!(has(&rows, text), "{text}");
        }
        for text in ["25 → ", "37 wd", "Launch is ", "at risk", " · 0 wd to spare", " · Requirements must start today"] {
            assert!(svg.contains(text), "{text}");
        }
        assert!(dashes(&placements) > before + 5);

        app.discard();
        assert!(app.before.is_none() && app.plan.items[frontend].duration == 25 && app.analysis.verdict.as_ref().is_some_and(|verdict| verdict.spare == 12));
        app.try_length(1);
        app.try_length(1);
        app.keep();
        assert!(app.before.is_none() && app.unsaved && app.plan.items[frontend].duration == 27 && app.history.len() == 1);
        app.undo();
        assert!(app.plan.items[frontend].duration == 25 && app.message.as_deref() == Some("Undone: Frontend +2 wd"));
        app.unsaved = false;

        app.ask();
        let question = app.question.as_mut().unwrap();
        question.choice = 2;
        question.typed = "4 jan".into();
        app.try_answer();
        assert!(app.question.is_none() && app.before.is_some() && !app.unsaved && app.changes() == ["Frontend start"]);
        let svg: String = screen(&app, 170, 50).1.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(svg.contains("Mon 21 Dec → ") && svg.contains("Mon 04 Jan") && svg.contains("9 wd late") && !svg.contains("must start"));
        app.discard();
        app.selected = app.plan.item("launch").unwrap();
        app.try_length(1);
        assert!(app.before.is_none() && app.message.as_deref() == Some("Launch is a milestone; it has no duration"));
    }

    #[test]
    fn notes_go_on_items_and_places_and_a_milestone_takes_a_star() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let (frontend, sign_off) = (app.plan.item("frontend").unwrap(), app.plan.item("design-sign-off").unwrap());
        app.selected = frontend;
        app.add_notes();
        assert_eq!(app.spot(), Some(Spot::Item(frontend)));
        app.cursor_lane(2);
        let qa = app.plan.lane("qa").unwrap();
        assert_eq!(app.spot(), Some(Spot::Place(qa, jiff::civil::date(2026, 12, 21))));
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, "ADD NOTES") && has(&rows, "Enter write a note here") && !has(&rows, "i important"));
        assert!(svg.contains("Test/QA · Bangalore") && svg.contains("Mon 21 Dec 2026 · W52"));
        app.write();
        app.notes.as_mut().unwrap().typed = Some("Meet QA".into());
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, "NOTE · nothing changed yet") && has(&rows, "Enter keep the note") && svg.contains("What should people know about Mon 21 Dec?") && svg.contains("Meet QA"));
        app.keep_note();
        assert!(app.plan.notes.len() == 1 && app.plan.notes[0].text == "Meet QA" && app.unsaved && app.notes.as_ref().is_some_and(|notes| notes.typed.is_none()));

        let svg = |app: &App| screen(app, 170, 50).1.iter().map(|placement| placement.svg.clone()).collect::<String>();
        assert!(svg(&app).contains(">Meet QA<"));
        app.cursor_lane(1);
        app.move_cursor(-3);
        assert_eq!(app.spot(), Some(Spot::Item(sign_off)));
        assert!(has(&screen(&app, 170, 50).0, "i important"));
        app.toggle_important();
        app.write();
        app.notes.as_mut().unwrap().typed = Some("Client signs the design in a workshop in New York.".into());
        app.keep_note();
        let item = &app.plan.items[sign_off];
        assert!(item.important && item.note.as_deref() == Some("Client signs the design in a workshop in New York.") && app.history.len() == 3);

        assert_eq!(svg(&app).matches("Client signs the design").count(), 2);
        app.notes = None;
        assert!(svg(&app).matches("Client signs the design").count() == 1 && !svg(&app).contains(">Meet QA<"));
        let (rows, placements) = screen(&app, 170, 50);
        let line = &placements.last().unwrap().svg;
        assert!(has(&rows, "n add notes") && line.contains("Design sign-off") && line.contains("Client signs the design in a workshop in New York."));
        app.undo();
        assert!(app.plan.items[sign_off].note.is_none() && app.message.as_deref() == Some("Undone: note on Design sign-off"));
    }

    #[test]
    fn the_quiet_lines_open_the_checks_that_found_nothing_and_the_holidays() {
        let mut app = app();
        app.screen = Screen::Checks;
        app.choose_check(2);
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, ":: CHECKED · NOTHING FOUND") && !has(&rows, "Enter show in roadmap"));
        for text in ["Every fixed date holds", "No item is longer than six weeks", "Two items at once by one person: the plan names no one"] {
            assert!(svg.contains(text), "{text}");
        }
        app.choose_check(1);
        let rows = screen(&app, 170, 50).0;
        assert!(has(&rows, ":: HOLIDAYS ACCOUNTED FOR · 5") && has(&rows, "Christmas Day") && has(&rows, "DE-BE") && has(&rows, "Republic Day"));
        assert!(has(&screen(&app, 80, 24).0, "Republic Day"));
        app.show_check_in_roadmap();
        assert_eq!(app.screen, Screen::Checks);
        app.choose_check(1);
        assert_eq!(app.check, 0);
    }

    #[test]
    fn items_are_added_renamed_and_linked_each_as_one_change() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let (frontend, integration) = (app.plan.item("frontend").unwrap(), app.plan.item("integration").unwrap());
        app.selected = frontend;
        app.start_add();
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, "Enter add") && svg.contains("ADD AFTER FRONTEND") && svg.contains("after Frontend, before Integration") && svg.contains("Type a name to see what follows."));
        "Code review".chars().for_each(|c| app.form_type(c));
        app.form_move(2);
        app.form_back();
        app.form_type('3');
        let svg: String = screen(&app, 170, 50).1.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(svg.contains("3 working days") && svg.contains("Launch stays on track, ") && svg.contains("with 9 working days to spare."));
        app.keep_form();
        let review = app.plan.item("code-review").unwrap();
        assert!(app.form.is_none() && app.selected == review && app.plan.items[review].duration == 3 && app.plan.items[review].after == ["frontend"]);
        assert!(app.plan.items[integration].after.contains(&"code-review".to_string()) && !app.plan.items[integration].after.contains(&"frontend".to_string()));
        app.undo();
        assert!(app.plan.item("code-review").is_none() && app.plan.items[integration].after.contains(&"frontend".to_string()));

        app.selected = frontend;
        app.start_add();
        "Review".chars().for_each(|c| app.form_type(c));
        app.form_move(1);
        app.form_change(1);
        app.form_move(3);
        app.form_change(-1);
        app.keep_form();
        let review = app.plan.item("review").unwrap();
        assert!(app.plan.items[review].milestone && app.plan.items[review].after == ["design-sign-off"] && app.plan.items[frontend].after == ["review"]);

        app.selected = frontend;
        app.start_rename();
        app.form_back();
        "ds".chars().for_each(|c| app.form_type(c));
        app.keep_form();
        assert!(app.plan.items[frontend].name == "Frontends" && app.plan.items[frontend].id == "frontend");

        let sdk = app.plan.item("sdk-integration").unwrap();
        app.selected = sdk;
        app.start_waits();
        assert!(!app.waits_order(sdk).contains(&integration) && app.waits_order(sdk).contains(&app.plan.item("api").unwrap()));
        let svg: String = screen(&app, 170, 50).1.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(svg.contains("What does SDK integration wait on?") && svg.contains("Integration and 5 more wait on SDK integration, so they are not offered.") && svg.contains("Nothing changes."));
        let api = app.waits_order(sdk).iter().position(|&index| app.plan.items[index].id == "api").unwrap();
        let Some(crate::app::Form::WaitsOn { at, .. }) = app.form.as_mut() else { panic!() };
        *at = api;
        app.form_type(' ');
        app.keep_form();
        assert!(app.plan.items[sdk].after.contains(&"api".to_string()) && app.message.is_none());
        app.undo();
        assert_eq!(app.message.as_deref(), Some("Undone: what SDK integration waits on"));
    }

    #[test]
    fn removing_an_item_keeps_the_chain_and_its_phase_whole() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let frontend = app.plan.item("frontend").unwrap();
        app.selected = frontend;
        app.start_remove();
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, "Enter remove") && svg.contains("Remove Frontend?") && svg.contains("Integration waits on Design sign-off instead of Frontend.") && svg.contains("with 23 working days to spare."));
        app.keep_form();
        let integration = app.plan.item("integration").unwrap();
        assert!(app.plan.item("frontend").is_none() && app.selected == integration && app.plan.items[integration].after == ["api", "sdk-integration", "design-sign-off"]);
        app.undo();
        assert!(app.plan.item("frontend").is_some() && app.message.as_deref() == Some("Undone: removed Frontend"));

        app.selected = app.plan.item("feature-freeze").unwrap();
        let (_, words) = app.removal(app.selected).unwrap();
        assert_eq!(words, ["Beta test waits on Integration instead of Feature freeze.", "Security review waits on Integration instead of Feature freeze.", "The Development phase ends with Integration instead."]);
        app.start_remove();
        app.keep_form();
        assert!(app.plan.item("feature-freeze").is_none() && app.plan.phases[2].until == "integration");

        app.selected = app.plan.item("requirements").unwrap();
        app.start_remove();
        assert!(has(&screen(&app, 170, 50).0, "Esc back"));
        app.keep_form();
        assert!(app.plan.item("requirements").is_some() && app.form.is_some() && app.message.as_deref() == Some("Requirements cannot be removed: the Requirements phase ends with it, and nothing comes before it."));
    }

    #[test]
    fn space_shows_the_details_beside_the_roadmap_when_there_is_room() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        app.toggle_details(170);
        assert!(app.side && app.message.is_none());
        let (rows, placements) = screen(&app, 170, 50);
        assert!(has(&rows, ":: DETAILS · FRONTEND") && has(&rows, "Waits on") && has(&rows, "Design sign-off") && has(&rows, "26 Jan Republic Day") && has(&rows, ":: CHECKS · 1 WARNING · 1 HINT"));
        assert!(placements.iter().any(|placement| placement.svg.contains("Mon 21 Dec – Wed 27 Jan")) && !has(&rows, "Planned"));
        assert!(!has(&screen(&app, 120, 40).0, "DETAILS"));
        app.toggle_details(120);
        app.toggle_details(120);
        assert_eq!(app.message.as_deref(), Some("The details need 150 columns"));
    }

    #[test]
    fn the_help_explains_the_screen_and_the_finder_runs_what_was_typed() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let rows = screen(&app, 170, 50).0;
        assert!(has(&rows, "? help") && !has(&rows, "q quit") && has(&screen(&app, 60, 16).0, "? help"));
        app.help = true;
        let (rows, placements) = screen(&app, 170, 50);
        let help = &placements.last().unwrap().svg;
        assert!(has(&rows, "Esc close") && help.contains(":: HELP · ROADMAP") && help.contains("Find any action by typing") && help.contains("decides the date"));
        let chart = &placements[placements.len() - 3].svg;
        assert!(chart.contains(&app.palette.dimmed().ink.hex()) && !chart.contains(&app.palette.ink.hex()));
        app.help = false;
        app.finder = Some(crate::app::Finder { typed: "move launch".into(), chosen: 0 });
        let (rows, placements) = screen(&app, 170, 50);
        let finder = &placements.last().unwrap().svg;
        assert!(has(&rows, "Enter run") && finder.contains("2 matches") && finder.contains("Change a date") && finder.contains("when Launch is · now Mon 01 Mar, fixed"));
        let (action, item) = crate::actions::find(&app, "move launch")[0];
        app.finder = None;
        assert!(!action.run(&mut app, item, "move launch"));
        assert!(app.question.as_ref().is_some_and(|question| Some(question.item) == app.plan.item("launch")));
    }

    #[test]
    fn the_situation_says_what_changed_since_the_last_snapshot_and_c_compares_the_roadmap() {
        let mut kept = snapshotted();
        let rows = screen(&kept, 170, 50).0;
        for text in ["Since Kick-off · Thu 22 Oct", "3 working days less to spare: Requirements must now start by Mon 16 Nov instead of Thu 19 Nov", "Frontend +3 wd (22 → 25)", "Design sign-off moved from Wed 23 Dec to Fri 18 Dec", "c what changed"] {
            assert!(has(&rows, text), "{text}");
        }
        assert!(!has(&screen(&app(), 170, 50).0, "Since") && !has(&screen(&app(), 170, 50).0, "c what changed"));
        kept.toggle_compare();
        kept.selected = kept.plan.item("frontend").unwrap();
        let (rows, placements) = screen(&kept, 170, 50);
        assert!(kept.screen == Screen::Roadmap && has(&rows, ":: SINCE KICK-OFF · THU 22 OCT · SWIMLANE ROADMAP") && has(&rows, "c back"));
        assert!(placements.iter().any(|placement| placement.svg.contains("was Thu 24 Dec – Wed 27 Jan")));
        assert!(placements.iter().any(|placement| placement.svg.contains("stroke-dasharray")));
        kept.toggle_compare();
        assert!(!kept.compare && !has(&screen(&kept, 170, 50).0, "SINCE KICK-OFF"));
        let mut plain = app();
        plain.toggle_compare();
        assert_eq!((plain.compare, plain.message.as_deref()), (false, Some("No snapshot yet; t keeps one")));
    }

    #[test]
    fn t_keeps_a_snapshot_with_a_name_and_u_takes_it_back() {
        let mut kept = snapshotted();
        kept.screen = Screen::Situation;
        kept.start_snapshot();
        assert_eq!(kept.screen, Screen::Roadmap);
        let rows = screen(&kept, 170, 50);
        assert!(has(&rows.0, "Enter keep it") && rows.1.iter().any(|placement| placement.svg.contains("Keep how the plan stands today?") && placement.svg.contains("The last snapshot is Kick-off · Thu 22 Oct.")));
        "Steering committee".chars().for_each(|c| kept.form_type(c));
        kept.keep_form();
        assert_eq!(kept.plan.snapshots.iter().map(|snapshot| snapshot.title()).collect::<Vec<_>>(), ["Kick-off · Thu 22 Oct", "Steering committee · Thu 29 Oct"]);
        assert_eq!(kept.message.as_deref(), Some("Kept: Steering committee · Thu 29 Oct · u takes it back"));
        assert!(kept.unsaved && kept.since().is_some_and(|(snapshot, changes)| snapshot.name.as_deref() == Some("Steering committee") && changes.is_empty()));
        kept.screen = Screen::Situation;
        assert!(has(&screen(&kept, 170, 50).0, "Since Steering committee · Thu 29 Oct   nothing has moved"));
        kept.undo();
        assert_eq!(kept.plan.snapshots.len(), 1);
        kept.start_snapshot();
        kept.keep_form();
        assert_eq!(kept.plan.snapshots.last().map(|snapshot| snapshot.title()), Some("Thu 29 Oct".to_string()));
    }
}
