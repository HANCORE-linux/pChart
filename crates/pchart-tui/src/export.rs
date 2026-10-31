//! The Export screen (pictures 24 and 25): the view, the audience and the formats on the left, and the
//! page as the file will look on the right. Files go to `export/` next to the plan.

use std::path::{Path, PathBuf};

use jiff::civil::Date;
use pchart_core::{Analysis, Plan, Schedule};
use pchart_draw::{Audience, Output, View, bits, markdown};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;

use crate::app::{App, FORMATS, Part};
use crate::pictures::Placement;
use crate::ui::{Inks, bold, cut, home, panel, put};

/// The left column, as in picture 4.
const LEFT: u16 = 50;

fn folder(path: &str) -> PathBuf {
    Path::new(path).parent().map_or(PathBuf::from("export"), |parent| parent.join("export"))
}

fn stem(path: &str) -> String {
    Path::new(path).file_stem().map_or("plan".into(), |stem| stem.to_string_lossy().into_owned())
}

/// The files an export writes: the page in each format that is on, and the Markdown with the milestone
/// chart next to it.
fn names(stem: &str, view: View, formats: [bool; 4]) -> Vec<String> {
    let mut out: Vec<String> = ["pdf", "svg", "png"].into_iter().zip(formats).filter(|&(_, on)| on).map(|(extension, _)| format!("{stem}-{}.{extension}", view.id())).collect();
    if formats[3] {
        out.push(format!("{stem}.md"));
        let picture = format!("{stem}-milestones.svg");
        if !out.contains(&picture) {
            out.push(picture);
        }
    }
    out
}

pub fn files(app: &App) -> Vec<PathBuf> {
    let folder = folder(&app.path);
    names(&stem(&app.path), app.export.view, app.export.formats).into_iter().map(|name| folder.join(name)).collect()
}

/// What hides the audience for this view, or what it shows when it hides nothing.
fn hides(view: View, audience: Audience) -> (&'static str, String) {
    let mut hidden = Vec::new();
    if view == View::Gantt && !audience.spare() {
        hidden.push("what items have to spare");
    }
    if matches!(view, View::Gantt | View::Roadmap) && !audience.warnings() {
        hidden.push("warnings");
    }
    match (hidden.is_empty(), matches!(view, View::Gantt | View::Roadmap)) {
        (false, _) => ("hides", hidden.join(", ")),
        (true, true) => ("shows", "everything".into()),
        (true, false) => ("shows", "the same for every audience".into()),
    }
}

pub fn draw(app: &App, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    let body = Rect { x: 0, y: 1, width: area.width, height: area.height - 2 };
    let e = &app.export;
    let page = e.view.page(&app.plan, &app.schedule, app.today, &app.analysis.checks, e.audience);
    let wide = area.width >= 110;
    let left = if wide { Rect { width: LEFT, ..body } } else { body };
    panel(frame, inks, left, "EXPORT", true);
    let (x, w) = (left.x, left.width);
    let bottom = left.y + left.height - 1;
    let compact = area.height < 30;
    let mut y = left.y + 2;
    let line = |frame: &mut Frame, y: &mut u16, indent: u16, spans: Vec<Span<'static>>| {
        if *y < bottom {
            put(frame, x + indent, *y, w.saturating_sub(indent + 2), spans);
        }
        *y += 1;
    };
    let heading = |part: Part, text: &'static str| Span::styled(text, if e.part == part { inks.ink } else { inks.muted });
    let choice = |frame: &mut Frame, y: &mut u16, name: &'static str, on: bool| {
        if on && *y < bottom {
            frame.buffer_mut().set_style(Rect { x: x + 1, y: *y, width: w.saturating_sub(2), height: 1 }, Style::new().bg(inks.raised));
            put(frame, x + 2, *y, 1, vec![Span::styled("›", bold(Style::new().fg(inks.accent)))]);
        }
        let style = if on { bold(inks.bright) } else { inks.ink };
        if *y < bottom {
            put(frame, x + 4, *y, w.saturating_sub(6), vec![Span::styled(name, style)]);
        }
        *y += 1;
    };

    line(frame, &mut y, 3, vec![heading(Part::View, "VIEW")]);
    for view in View::ALL {
        choice(frame, &mut y, view.name(), view == e.view);
    }
    y += 1;
    line(frame, &mut y, 3, vec![heading(Part::Audience, "AUDIENCE")]);
    for (audience, name) in [(Audience::Team, "Team"), (Audience::Management, "Management"), (Audience::Client, "Client")] {
        choice(frame, &mut y, name, audience == e.audience);
    }
    let (verb, what) = hides(e.view, e.audience);
    line(frame, &mut y, 4, vec![Span::styled(format!("{verb:<7}"), inks.muted), Span::styled(what, inks.dim)]);
    y += 1;
    line(frame, &mut y, 3, vec![heading(Part::Format, "FORMAT")]);
    let mut formats = Vec::new();
    for (index, (name, on)) in FORMATS.into_iter().zip(e.formats).enumerate() {
        let ground = |style: Style| if e.part == Part::Format && e.format == index { style.bg(inks.raised) } else { style };
        if index > 0 {
            formats.push(Span::raw("   "));
        }
        formats.push(Span::styled(if on { "[x] " } else { "[ ] " }, ground(if on { bold(inks.bright) } else { inks.muted })));
        formats.push(Span::styled(name, ground(if on { inks.ink } else { inks.dim })));
    }
    line(frame, &mut y, 4, formats);
    y += 1;

    let folder_text = format!("{}/", home(&folder(&app.path).display().to_string()));
    let size = if page.height <= 632.5 { "16:9 · slide".to_string() } else { format!("{:.0} × {:.0} px · taller than a slide", page.width, page.height) };
    let facts = [("Page", size), ("Style", "light · Noto Sans".into()), ("Holidays", "off".into())];
    for (key, value) in facts.into_iter().filter(|_| !compact).chain([("Folder", folder_text)]) {
        line(frame, &mut y, 3, vec![Span::styled(format!("{key:<11}"), inks.muted), Span::styled(cut(&value, w.saturating_sub(16) as usize), inks.ink)]);
    }
    if !compact {
        y += 1;
    }
    let files = files(app);
    let names: Vec<String> = files.iter().map(|file| file.file_name().map_or(String::new(), |name| name.to_string_lossy().into_owned())).collect();
    let state = if e.busy {
        Some("writing …".to_string())
    } else {
        e.written.as_ref().filter(|(written, _)| *written == files).map(|(_, time)| format!("written {time}"))
    };
    let room = w.saturating_sub(16) as usize;
    if compact {
        let mut joined = String::new();
        let mut last = String::new();
        for name in &names {
            let (base, extension) = name.rsplit_once('.').unwrap_or((name, ""));
            joined += &if joined.is_empty() { name.clone() } else if base == last { format!(", .{extension}") } else { format!(", {name}") };
            last = base.to_string();
        }
        let mut spans = vec![Span::styled("Files      ", inks.muted), Span::styled(cut(&joined, room), inks.ink)];
        if let Some(state) = state {
            spans.push(Span::styled(format!(" · {state}"), inks.dim));
        }
        line(frame, &mut y, 3, spans);
    } else {
        line(frame, &mut y, 3, vec![Span::styled("Files", inks.muted)]);
        y -= 1;
        for name in &names {
            line(frame, &mut y, 14, vec![Span::styled(cut(name, room), inks.ink)]);
        }
        if names.is_empty() {
            y += 1;
        }
        if let Some(state) = state {
            line(frame, &mut y, 14, vec![Span::styled(state, inks.dim)]);
        }
    }

    if !wide {
        line(frame, &mut y, 3, vec![Span::styled("The preview needs 110 columns.", inks.muted)]);
        return;
    }
    let right = Rect { x: LEFT, width: body.width - LEFT, ..body };
    panel(frame, inks, right, "PREVIEW · EXACTLY WHAT THE FILE LOOKS LIKE", false);
    let rect = Rect { x: right.x + 2, y: right.y + 2, width: right.width.saturating_sub(4), height: right.height.saturating_sub(3) };
    if rect.width >= 10 && rect.height >= 4 {
        let (cw, ch) = app.cell;
        let (px_w, px_h) = (f32::from(rect.width) * f32::from(cw), f32::from(rect.height) * f32::from(ch));
        placements.push(Placement { rect, svg: bits::sheet(&page, px_w, px_h, f32::from(ch) / 21.0), paper: true });
    }
}

/// Everything an export needs, so the files can be written away from the screen.
pub struct Job {
    plan: Plan,
    schedule: Schedule,
    analysis: Analysis,
    today: Date,
    source: String,
    stem: String,
    folder: PathBuf,
    view: View,
    audience: Audience,
    formats: [bool; 4],
}

pub fn job(app: &App) -> Job {
    let source = Path::new(&app.path).file_name().map_or("plan".into(), |name| name.to_string_lossy().into_owned());
    let e = &app.export;
    Job { plan: app.plan.clone(), schedule: app.schedule.clone(), analysis: app.analysis.clone(), today: app.today, source, stem: stem(&app.path), folder: folder(&app.path), view: e.view, audience: e.audience, formats: e.formats }
}

/// Writes the files of `job` and returns them.
pub fn write(job: &Job) -> Result<Vec<PathBuf>, String> {
    let names = names(&job.stem, job.view, job.formats);
    std::fs::create_dir_all(&job.folder).map_err(|err| format!("{}: {err}", job.folder.display()))?;
    let save = |name: &str, bytes: &[u8]| {
        let path = job.folder.join(name);
        std::fs::write(&path, bytes).map_err(|err| format!("{}: {err}", path.display()))
    };
    let svg = job.view.page(&job.plan, &job.schedule, job.today, &job.analysis.checks, job.audience).svg();
    if job.formats[..3].iter().any(|&on| on) {
        let output = Output::default();
        let tree = output.tree(&svg)?;
        let base = format!("{}-{}", job.stem, job.view.id());
        if job.formats[0] {
            save(&format!("{base}.pdf"), &output.pdf(&tree)?)?;
        }
        if job.formats[1] {
            save(&format!("{base}.svg"), svg.as_bytes())?;
        }
        if job.formats[2] {
            save(&format!("{base}.png"), &output.png(&tree, 1.0)?)?;
        }
    }
    if job.formats[3] {
        let picture = format!("{}-milestones.svg", job.stem);
        save(&picture, pchart_draw::milestones(&job.plan, &job.schedule, job.today).svg().as_bytes())?;
        save(&format!("{}.md", job.stem), markdown(&job.plan, &job.schedule, &job.analysis, &job.source, job.today, &picture, job.audience).as_bytes())?;
    }
    Ok(names.into_iter().map(|name| job.folder.join(name)).collect())
}
