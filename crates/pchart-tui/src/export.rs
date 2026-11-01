//! The Export screen (pictures 33 and 35): the view, the audience and the formats on the left, and the
//! page as the file will look on the right. Files go to `export/` next to the plan.

use std::path::{Path, PathBuf};

use jiff::civil::Date;
use pchart_core::{Analysis, Plan, Schedule};
use pchart_draw::bits::{self, Icon};
use pchart_draw::scene::{Fill, Shape};
use pchart_draw::style::ADVANCE;
use pchart_draw::{Audience, Output, View, markdown};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::{App, FORMATS, Part};
use crate::pictures::Placement;
use crate::ui::{Inks, home, panel};

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
    let left = if wide { Rect { width: (area.width * 46 / 170).max(44), ..body } } else { body };
    panel(frame, inks, left, "EXPORT", true);
    let (cw, ch) = app.cell;
    let gap = u16::from(area.height >= 30);
    let rect = Rect { x: left.x + 1, y: left.y + 2 + gap, width: left.width.saturating_sub(2), height: left.height.saturating_sub(3 + gap) };
    let (w, h) = (f32::from(rect.width) * f32::from(cw), f32::from(rect.height) * f32::from(ch));
    placements.push(Placement { rect, svg: options(app, w, h, f32::from(ch) / 21.0, area.height < 30, wide).svg(), paper: false });
    if !wide {
        return;
    }
    let right = Rect { x: left.width, width: body.width - left.width, ..body };
    panel(frame, inks, right, "PREVIEW · EXACTLY WHAT THE FILE LOOKS LIKE", false);
    let rect = Rect { x: right.x + 2, y: right.y + 2, width: right.width.saturating_sub(4), height: right.height.saturating_sub(3) };
    if rect.width >= 10 && rect.height >= 4 {
        let (px_w, px_h) = (f32::from(rect.width) * f32::from(cw), f32::from(rect.height) * f32::from(ch));
        placements.push(Placement { rect, svg: bits::sheet(&page, px_w, px_h, f32::from(ch) / 21.0), paper: true });
    }
}

/// The options of picture 35 as one picture: the views as a list with the chosen one filled like a box
/// that is on, the audience and the formats as boxes, the folder and the files. The part `↑↓` is on has its label in brighter ink; `←→` rings a
/// format in pink. Rows are 21 units, as cells.
fn options(app: &App, width: f32, height: f32, unit: f32, compact: bool, wide: bool) -> pchart_draw::Scene {
    let (p, u, e) = (app.shown(), unit, &app.export);
    let mut s = bits::canvas(width, height, &p);
    let row = 21.0 * u;
    let size = 16.6 * u;
    let x = 16.0 * u;
    let fit = |text: &str, from: f32| {
        let count = ((width - from) / (size * ADVANCE)).floor().max(0.0) as usize;
        if text.chars().count() <= count { text.to_string() } else { text.chars().take(count.saturating_sub(1)).collect::<String>() + "…" }
    };
    let mut y = 0.0;
    let label = |s: &mut pchart_draw::Scene, y: f32, text: &str, part: Option<Part>| {
        bits::runs(s, x, y + 15.0 * u, 15.0 * u, &[(text, if part.is_some_and(|part| part == e.part) { p.ink } else { p.muted }, false)]);
    };
    label(&mut s, y, "View", Some(Part::View));
    y += row;
    for view in View::of(&app.plan) {
        let on = view == e.view;
        if on {
            let w = view.name().chars().count() as f32 * size * ADVANCE + 24.0 * u;
            s.add(Shape::Rect { x: x - 12.0 * u, y, w, h: row, radius: 5.0 * u, fill: Fill::Color(p.ink), stroke: None });
        }
        bits::runs(&mut s, x, y + 15.5 * u, size, &[(view.name(), if on { p.page } else { p.ink }, on)]);
        y += row;
    }
    y += if compact { 0.0 } else { row };
    label(&mut s, y, "For", Some(Part::Audience));
    y += row;
    let mut cx = x;
    for (audience, name) in [(Audience::Team, "Team"), (Audience::Management, "Management"), (Audience::Client, "Client")] {
        cx = bits::chip(&mut s, cx, y + 19.0 * u, name, audience == e.audience, false, u, &p) + 10.0 * u;
    }
    y += 1.5 * row;
    if !compact {
        let (verb, what) = hides(e.view, e.audience);
        let room = ((width - x) / (15.0 * u * ADVANCE)).floor().max(1.0) as usize;
        let mut lines: Vec<String> = vec![String::new()];
        for word in format!("{verb} {what}").split(' ') {
            let line = lines.last_mut().expect("a line");
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > room {
                lines.push(word.to_string());
            } else {
                *line = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            }
        }
        for line in &lines {
            bits::runs(&mut s, x, y + 15.0 * u, 15.0 * u, &[(line, p.dim, false)]);
            y += row;
        }
        y += row;
    }
    label(&mut s, y, "Format", Some(Part::Format));
    y += row;
    let mut cx = x;
    for (index, (name, on)) in FORMATS.into_iter().zip(e.formats).enumerate().take(app.formats()) {
        cx = bits::chip(&mut s, cx, y + 19.0 * u, name, on, e.part == Part::Format && e.format == index, u, &p) + 10.0 * u;
    }
    y += if compact { 1.5 * row } else { 2.5 * row };

    let files = files(app);
    let names: Vec<String> = files.iter().map(|file| file.file_name().map_or(String::new(), |name| name.to_string_lossy().into_owned())).collect();
    let state = if e.busy {
        Some("writing …".to_string())
    } else {
        e.written.as_ref().filter(|(written, _)| *written == files).map(|(_, time)| format!("written {time}"))
    };
    let indent = x + 24.0 * u;
    bits::draw_icon(&mut s, Icon::Folder, x + 8.0 * u, y + 10.0 * u, 16.0 * u, p.dim, p.page);
    let folder_text = format!("{}/", home(&folder(&app.path).display().to_string()));
    bits::runs(&mut s, indent, y + 15.0 * u, size, &[(&fit(&folder_text, indent), p.ink, false)]);
    y += row;
    if compact {
        let mut joined = String::new();
        let mut last = String::new();
        for name in &names {
            let (base, extension) = name.rsplit_once('.').unwrap_or((name, ""));
            joined += &if joined.is_empty() { name.clone() } else if base == last { format!(", .{extension}") } else { format!(", {name}") };
            last = base.to_string();
        }
        if let Some(state) = &state {
            joined += &format!(" · {state}");
        }
        bits::runs(&mut s, indent, y + 15.0 * u, size, &[(&fit(&joined, indent), p.ink, false)]);
        y += row;
    } else {
        for name in &names {
            bits::runs(&mut s, indent, y + 15.0 * u, size, &[(&fit(name, indent), p.ink, false)]);
            y += row;
        }
        if let Some(state) = &state {
            bits::draw_icon(&mut s, Icon::Check, x + 8.0 * u, y + 10.0 * u, 16.0 * u, p.dim, p.page);
            bits::runs(&mut s, indent, y + 15.0 * u, size, &[(state, p.dim, false)]);
            y += row;
        }
    }
    if !wide {
        bits::runs(&mut s, x, y + row + 15.0 * u, 15.0 * u, &[("The preview needs 110 columns.", p.muted, false)]);
    }
    s
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
        save(&picture, pchart_draw::milestones(&job.plan, &job.schedule, job.today, job.audience).svg().as_bytes())?;
        save(&format!("{}.md", job.stem), markdown(&job.plan, &job.schedule, &job.analysis, &job.source, job.today, &picture, job.audience).as_bytes())?;
    }
    Ok(names.into_iter().map(|name| job.folder.join(name)).collect())
}
