//! The Reports screen (pictures 65): in a workspace, what the team reports about the plan, newest first,
//! and for the chosen report what taking it would do, in the roadmap around it and in words.

use pchart_core::report::{Answer, Report, Status};
use pchart_core::text::{self, Advice};
use pchart_draw::bits::{self, Icon};
use pchart_draw::scene::Color;
use pchart_draw::{Cut, Level, what_if_part};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;
use crate::pictures::Placement;
use crate::ui::{Inks, day, panel};

/// The symbol of a report and its colour.
pub(crate) fn symbol(app: &App, report: &Report) -> (Icon, Color) {
    let p = app.shown();
    match report.status {
        Status::Done => (Icon::Check, p.bright),
        Status::OnTrack => (Icon::Clock, p.dim),
        Status::Risk | Status::Later => (Icon::Warn, p.warn),
    }
}

/// "Priya Nair: Frontend needs 3 more working days".
pub(crate) fn title(app: &App, report: &Report) -> String {
    let name = app.plan.item(&report.item).map_or(report.item.as_str(), |index| app.plan.items[index].name.as_str());
    format!("{}: {name} {}", report.by, report.words())
}

/// When, and what they say: "Mon 02 Nov 11:45 · Backend API comes late".
pub(crate) fn place(report: &Report) -> String {
    let when = report.at.strftime("%a %d %b %H:%M").to_string();
    match &report.comment {
        Some(comment) => format!("{when} · {comment}"),
        None => when,
    }
}

fn picture(app: &App, placements: &mut Vec<Placement>, rect: Rect, scene: impl FnOnce(f32, f32, f32) -> pchart_draw::Scene) {
    let (w, h) = (f32::from(rect.width) * f32::from(app.cell.0), f32::from(rect.height) * f32::from(app.cell.1));
    placements.push(Placement { rect, svg: scene(w, h, f32::from(app.cell.1) / 21.0).svg(), paper: false });
}

/// What happens, the step, and the plan then, for `report`.
fn advice(app: &App, report: &Report) -> Advice {
    let Some(index) = app.plan.item(&report.item) else {
        let step = if app.answers() { "x declines it." } else { "The project lead declines it." };
        return Advice { happens: vec![(format!("{} is no longer in the plan.", report.item), false)], step: step.into(), then: Vec::new() };
    };
    let name = app.plan.items[index].name.clone();
    let happens = match report.status {
        Status::Done => vec![(format!("{name} is marked done."), false)],
        Status::Later if app.plan.items[index].milestone => vec![(format!("{name} is a point in time; nothing gets longer."), false)],
        Status::Later => {
            let now = app.plan.items[index].duration;
            vec![(format!("{name} then takes "), false), (text::working_days((now + report.days) as i32), true), (format!(" instead of {now}."), false)]
        }
        Status::Risk | Status::OnTrack if report.comment.is_some() => vec![(format!("Nothing moves; what they say becomes the note of {name}."), false)],
        Status::Risk | Status::OnTrack => vec![("Nothing in the plan changes.".to_string(), false)],
    };
    let then = match report.then(&app.plan, &app.calendars, app.today).and_then(|(plan, _, analysis)| analysis.verdict.map(|verdict| (plan, verdict))) {
        Some((plan, verdict)) => {
            let target = format!("{} on {} ", plan.items[verdict.target].name, day(verdict.date));
            match verdict.state {
                pchart_core::State::DoesNotHold => vec![(format!("{target}is then "), false), (text::working_days(-verdict.spare), true), (" late.".into(), false)],
                _ => vec![(format!("{target}then holds, with "), false), (text::working_days(verdict.spare), true), (" to spare.".into(), false)],
            }
        }
        None => Vec::new(),
    };
    let step = match app.lead().filter(|_| !app.answers()) {
        Some(lead) => format!("{lead} answers it; if they take it:"),
        None => "Enter takes it into the plan, x declines it:".into(),
    };
    Advice { happens, step, then }
}

pub fn draw(app: &App, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    let p = app.shown();
    let body = Rect { x: 0, y: 1, width: area.width, height: area.height - 2 };
    let wide = area.width >= 150;
    let left = if wide { Rect { width: (area.width * 62 / 170).max(50), ..body } } else { body };
    let open = app.open_reports();
    let answered: Vec<usize> = (0..app.reports.len()).filter(|at| !open.contains(at)).collect();
    let heading = match open.len() {
        0 => "REPORTS".to_string(),
        1 => "REPORTS · 1 TO ANSWER".to_string(),
        count => format!("REPORTS · {count} TO ANSWER"),
    };
    panel(frame, inks, left, &heading, true);
    let (x, w) = (left.x + 2, left.width.saturating_sub(4));
    let headline = match open.len() {
        0 if app.reports.is_empty() => "No reports yet".to_string(),
        0 => "Every report is answered".to_string(),
        1 => "1 report to answer".to_string(),
        count => format!("{count} reports to answer"),
    };
    let (icon, tone) = if open.is_empty() { (Icon::Check, p.bright) } else { (Icon::Note, p.ink) };
    picture(app, placements, Rect { x, y: left.y + 2, width: w, height: 3 }, |pw, ph, u| bits::headline(icon, tone, &headline, pw, ph, u, &p));
    let bottom = left.y + left.height - 1;
    let mut y = left.y + 6;
    for (row, &at) in open.iter().enumerate() {
        let chosen = row == app.report;
        let height = if chosen { 3 } else { 2 };
        if y + height > bottom {
            break;
        }
        let report = &app.reports[at].1;
        let (icon, tone) = symbol(app, report);
        let (title, place) = (title(app, report), place(report));
        picture(app, placements, Rect { x: left.x + 1, y, width: left.width.saturating_sub(2), height }, |pw, ph, u| bits::row(icon, tone, &title, chosen.then_some(place.as_str()), chosen, pw, ph, u, &p));
        y += height;
    }
    if !answered.is_empty() {
        y += 1;
        for &at in &answered {
            if y >= bottom {
                break;
            }
            let report = &app.reports[at].1;
            let answer = match (report.answer, &report.taken) {
                (Some(Answer::Taken), Some(_)) => format!("taken as {}", report.as_taken().words()),
                (Some(Answer::Taken), None) => "taken".to_string(),
                _ => "declined".to_string(),
            };
            let words = format!("{answer} · {}", title(app, report));
            picture(app, placements, Rect { x: left.x + 2, y, width: left.width.saturating_sub(4), height: 1 }, |pw, ph, u| bits::line(&[(Some((Icon::Check, p.dim)), words, p.dim, false)], pw, ph, u, &p));
            y += 1;
        }
    }
    let Some(report) = open.get(app.report).map(|&at| &app.reports[at].1) else { return };
    let Advice { happens, step, then } = advice(app, report);
    if !wide {
        let rect = Rect { x: left.x + 3, y: y + 1, width: left.width.saturating_sub(6), height: 5 };
        if rect.y + rect.height < bottom {
            picture(app, placements, rect, |w, h, u| bits::advice(&happens, &step, &then, w, h, u, &p));
        }
        return;
    }
    let right = Rect { x: left.width, width: body.width - left.width, ..body };
    let tried = report.then(&app.plan, &app.calendars, app.today).filter(|_| report.status == Status::Later);
    let index = app.plan.item(&report.item);
    let cut = tried.as_ref().zip(index).and_then(|((plan, schedule, _), index)| Cut::around(plan, schedule, &[index]));
    let span = cut.as_ref().map(|cut| format!(" · {} – {}", cut.start.strftime("%d %b"), cut.end.strftime("%d %b %Y"))).unwrap_or_default();
    panel(frame, inks, right, &format!("IF YOU TAKE IT{span}").to_uppercase(), false);
    let text_h = 5;
    let room = right.height.saturating_sub(3 + text_h + 2);
    let rect = Rect { x: right.x + 1, y: right.y + 2, width: right.width.saturating_sub(2), height: if tried.is_some() { (right.height * 26 / 48).min(room) } else { 0 } };
    if let (Some((plan, schedule, _)), Some(cut), Some(index)) = (&tried, &cut, index)
        && rect.height >= 8
        && rect.width >= 30
    {
        let mut settings = app.settings(rect);
        settings.selected = Some(index);
        settings.level = Level::Full;
        let mut scene = what_if_part(plan, schedule, &app.calendars, &settings, cut, &app.schedule);
        scene.background = None;
        placements.push(Placement { rect, svg: scene.svg(), paper: false });
    }
    let text = Rect { x: right.x + 3, y: rect.y + rect.height + 2, width: right.width.saturating_sub(6), height: text_h };
    if text.y + text.height < right.y + right.height {
        picture(app, placements, text, |w, h, u| bits::advice(&happens, &step, &then, w, h, u, &p));
    }
}
