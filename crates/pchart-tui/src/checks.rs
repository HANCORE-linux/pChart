//! The Checks screen (pictures 33 and 35): how many things to look at, every check as one line with its
//! symbol, one quiet line for the checks that found nothing and one for the holidays; for the chosen
//! check the roadmap around it, what happens, and what can be done.

use pchart_core::analysis::{Check, Severity};
use pchart_core::text;
use pchart_draw::bits::{self, Icon};
use pchart_draw::{Cut, Level, part};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;
use crate::pictures::Placement;
use crate::ui::{Inks, day, holidays, panel};

/// "CHECKS · 1 WARNING · 1 HINT".
fn title(app: &App) -> String {
    let mut parts = vec!["CHECKS".to_string()];
    for (severity, one, many) in [(Severity::Problem, "PROBLEM", "PROBLEMS"), (Severity::Warning, "WARNING", "WARNINGS"), (Severity::Hint, "HINT", "HINTS")] {
        match app.analysis.checks.iter().filter(|check| check.severity() == severity).count() {
            0 => {}
            1 => parts.push(format!("1 {one}")),
            count => parts.push(format!("{count} {many}")),
        }
    }
    parts.join(" · ")
}

/// A check in the list: its title, and where it is.
fn words(app: &App, check: &Check) -> (String, String) {
    let name = |index: usize| app.plan.items[index].name.clone();
    let finish = |index: usize| day(app.schedule.slots[index].planned.finish);
    match *check {
        Check::NoBuffer { before, target } => ("No buffer before a fixed date".into(), format!("{} {}  →  {} {}", name(before), finish(before), name(target), finish(target))),
        Check::PhaseEndsWithoutMilestone { phase, item } => (format!("{} phase ends without a milestone", app.plan.phases[phase].name), format!("Its last item is {}, {}", name(item), finish(item))),
        Check::LongItem { item } => (text::check(&app.plan, check).0.trim_end_matches('.').to_string(), format!("{} – {}", day(app.schedule.slots[item].planned.start), finish(item))),
        _ => {
            let (title, detail) = text::check(&app.plan, check);
            (title.trim_end_matches('.').to_string(), detail)
        }
    }
}

fn symbol(app: &App, check: &Check) -> (Icon, pchart_draw::scene::Color) {
    let p = app.palette;
    match check.severity() {
        Severity::Problem => (Icon::Cross, p.alert),
        Severity::Warning => (Icon::Warn, p.warn),
        Severity::Hint => (Icon::Info, p.dim),
    }
}

/// How many rules ran and found nothing; the check of two items at once runs only when the plan
/// names who does what.
fn fine(app: &App) -> usize {
    let found = |rule: &str| app.analysis.checks.iter().any(|check| check.rule() == rule);
    let who = app.plan.items.iter().any(|item| item.who.is_some());
    ["target-too-late", "deadline-missed", "fixed-before-predecessor", "no-buffer-before-fixed-date", "fixed-date-on-day-off", "overdue", "overlap", "long-item", "phase-without-milestone"]
        .into_iter()
        .filter(|&rule| !found(rule) && (rule != "overlap" || who))
        .count()
}

fn picture(app: &App, placements: &mut Vec<Placement>, rect: Rect, scene: impl FnOnce(f32, f32, f32) -> pchart_draw::Scene) {
    let (w, h) = (f32::from(rect.width) * f32::from(app.cell.0), f32::from(rect.height) * f32::from(app.cell.1));
    placements.push(Placement { rect, svg: scene(w, h, f32::from(app.cell.1) / 21.0).svg(), paper: false });
}

pub fn draw(app: &App, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    let body = Rect { x: 0, y: 1, width: area.width, height: area.height - 2 };
    let wide = area.width >= 150;
    let left = if wide { Rect { width: (area.width * 62 / 170).max(50), ..body } } else { body };
    panel(frame, inks, left, &title(app), true);
    let y = list(app, placements, left, if area.height >= 30 { 2 } else { 1 });
    let order = app.checks();
    let Some(&index) = order.get(app.check) else { return };
    let check = &app.analysis.checks[index];
    let advice = text::advice(&app.plan, &app.calendars, app.today, check);
    let p = app.palette;
    if !wide {
        let gap = u16::from(y + 6 < left.y + left.height);
        let rect = Rect { x: left.x + 3, y: y + gap, width: left.width.saturating_sub(6), height: 5 };
        if rect.y + rect.height < left.y + left.height {
            picture(app, placements, rect, |w, h, u| bits::advice(&advice.happens, &advice.step, &advice.then, w, h, u, &p));
        }
        return;
    }

    let right = Rect { x: left.width, width: body.width - left.width, ..body };
    let Some(cut) = Cut::around(&app.plan, &app.schedule, &check.items()) else { return };
    let span = if cut.start.year() == cut.end.year() {
        format!("{} – {}", cut.start.strftime("%d %b"), cut.end.strftime("%d %b %Y"))
    } else {
        format!("{} – {}", cut.start.strftime("%d %b %Y"), cut.end.strftime("%d %b %Y"))
    };
    panel(frame, inks, right, &format!("IN THE ROADMAP · {}", span.to_uppercase()), false);
    let text = Rect { x: right.x + 3, y: 0, width: right.width.saturating_sub(6), height: 5 };
    let room = right.height.saturating_sub(3 + text.height + 2);
    let rect = Rect { x: right.x + 1, y: right.y + 2, width: right.width.saturating_sub(2), height: (right.height * 26 / 48).min(room) };
    if rect.height >= 8 && rect.width >= 30 {
        let mut settings = app.settings(rect);
        settings.selected = None;
        settings.level = Level::Full;
        let mut scene = part(&app.plan, &app.schedule, &app.calendars, &settings, &cut);
        scene.background = None;
        placements.push(Placement { rect, svg: scene.svg(), paper: false });
    }
    let text = Rect { y: rect.y + rect.height + 2, ..text };
    if text.y + text.height < right.y + right.height {
        picture(app, placements, text, |w, h, u| bits::advice(&advice.happens, &advice.step, &advice.then, w, h, u, &p));
    }
}

/// The headline, the checks by weight, and the two quiet lines. Returns the row after them.
fn list(app: &App, placements: &mut Vec<Placement>, rect: Rect, spacing: u16) -> u16 {
    let p = app.palette;
    let (x, w) = (rect.x + 2, rect.width.saturating_sub(4));
    let bottom = rect.y + rect.height - 1;
    let looks = app.analysis.checks.iter().filter(|check| check.severity() != Severity::Hint).count();
    let problems = app.analysis.checks.iter().any(|check| check.severity() == Severity::Problem);
    let (icon, tone, headline) = match looks {
        0 => (Icon::Check, p.bright, "Nothing to look at".to_string()),
        1 => (if problems { Icon::Cross } else { Icon::Warn }, if problems { p.alert } else { p.warn }, "1 thing to look at".to_string()),
        count => (if problems { Icon::Cross } else { Icon::Warn }, if problems { p.alert } else { p.warn }, format!("{count} things to look at")),
    };
    picture(app, placements, Rect { x, y: rect.y + 2, width: w, height: 3 }, |pw, ph, u| bits::headline(icon, tone, &headline, pw, ph, u, &p));

    let order = app.checks();
    let heights: Vec<u16> = (0..order.len()).map(|at| if at == app.check { 3 } else { 2 }).collect();
    let top = rect.y + 6;
    let quiet = 2 * spacing;
    let room = bottom.saturating_sub(top + quiet);
    let mut first = 0;
    while first < app.check && heights[first..=app.check].iter().sum::<u16>() > room {
        first += 1;
    }
    let mut y = top;
    for (at, &index) in order.iter().enumerate().skip(first) {
        if y + heights[at] > top + room {
            break;
        }
        let check = &app.analysis.checks[index];
        let (title, place) = words(app, check);
        let (icon, tone) = symbol(app, check);
        let chosen = at == app.check;
        picture(app, placements, Rect { x: rect.x + 1, y, width: rect.width.saturating_sub(2), height: heights[at] }, |pw, ph, u| bits::row(icon, tone, &title, chosen.then_some(place.as_str()), chosen, pw, ph, u, &p));
        y += heights[at];
    }
    y += 1;
    let fine = fine(app);
    let days = holidays(app).len();
    for (icon, quiet) in [(Icon::Check, format!("{fine} other {} found nothing", if fine == 1 { "check" } else { "checks" })), (Icon::Calendar, format!("{days} {} accounted for", if days == 1 { "holiday" } else { "holidays" }))] {
        if y >= bottom {
            break;
        }
        picture(app, placements, Rect { x: rect.x + 2, y, width: rect.width.saturating_sub(4), height: 1 }, |pw, ph, u| bits::line(&[(Some((icon, p.dim)), quiet, p.dim, false)], pw, ph, u, &p));
        y += spacing;
    }
    y
}
