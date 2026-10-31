//! The Checks screen (pictures 33 and 35): how many things to look at, every check as one line with its
//! symbol, one quiet line for the checks that found nothing and one for the holidays; for the chosen
//! check the roadmap around it, what happens, and what can be done; for a chosen quiet line its list,
//! as picture 22 showed them.

use pchart_core::analysis::{Check, Severity};
use pchart_core::text;
use pchart_draw::bits::{self, Icon};
use pchart_draw::{Cut, Level, part};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;
use crate::pictures::Placement;
use crate::ui::{Inks, cut, day, holidays, panel, put};

/// "CHECKS · 1 WARNING · 1 HINT".
pub(crate) fn title(app: &App) -> String {
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
pub(crate) fn words(app: &App, check: &Check) -> (String, String) {
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

pub(crate) fn symbol(app: &App, check: &Check) -> (Icon, pchart_draw::scene::Color) {
    let p = app.shown();
    match check.severity() {
        Severity::Problem => (Icon::Cross, p.alert),
        Severity::Warning => (Icon::Warn, p.warn),
        Severity::Hint => (Icon::Info, p.dim),
    }
}

/// The rules that ran and found nothing, in plain words; the check of two items at once runs only when
/// the plan names who does what.
fn fine(app: &App) -> Vec<&'static str> {
    let found = |rule: &str| app.analysis.checks.iter().any(|check| check.rule() == rule);
    let who = app.plan.items.iter().any(|item| item.who.is_some());
    [
        ("target-too-late", "Every fixed date holds"),
        ("deadline-missed", "Deadlines are kept"),
        ("fixed-before-predecessor", "Fixed dates come after their work"),
        ("no-buffer-before-fixed-date", "Every fixed date has a buffer"),
        ("fixed-date-on-day-off", "No fixed date is on a day off"),
        ("overdue", "Nothing is overdue"),
        ("overlap", "No one does two items at once"),
        ("long-item", "No item is longer than six weeks"),
        ("phase-without-milestone", "Every phase ends with a milestone"),
    ]
    .into_iter()
    .filter(|&(rule, _)| !found(rule) && (rule != "overlap" || who))
    .map(|(_, words)| words)
    .collect()
}

/// The list behind a chosen quiet line, in `rect`: the checks that found nothing, or the holidays the
/// plan accounts for, as many as fit.
fn quiet(app: &App, frame: &mut Frame, inks: &Inks, placements: &mut Vec<Placement>, rect: Rect, holidays_chosen: bool, spacing: u16) {
    let p = app.shown();
    let bottom = rect.y + rect.height;
    if !holidays_chosen {
        let who = app.plan.items.iter().any(|item| item.who.is_some());
        let mut lines: Vec<bits::Part> = fine(app).into_iter().map(|words| (Some((Icon::Check, p.dim)), words.to_string(), p.ink, false)).collect();
        if !who {
            lines.push((Some((Icon::Info, p.muted)), "Two items at once by one person: the plan names no one".to_string(), p.muted, false));
        }
        for (row, line) in lines.into_iter().enumerate() {
            let y = rect.y + row as u16 * spacing;
            if y >= bottom {
                break;
            }
            picture(app, placements, Rect { x: rect.x, y, width: rect.width, height: 1 }, |pw, ph, u| bits::line(&[line], pw, ph, u, &p));
        }
        return;
    }
    let rows: Vec<(String, String, String, String)> = holidays(app)
        .into_iter()
        .map(|(index, date, name)| {
            let lane = &app.plan.lanes[app.plan.lane(&app.plan.items[index].lane).unwrap_or(0)];
            (app.plan.items[index].name.clone(), day(date), name, lane.place.clone().unwrap_or_default())
        })
        .collect();
    let widest = |column: fn(&(String, String, String, String)) -> &String| rows.iter().map(|row| column(row).chars().count()).max().unwrap_or(0) as u16;
    let (item_w, date_w) = (widest(|row| &row.0), widest(|row| &row.1));
    let room = rect.height as usize;
    let shown = if rows.len() > room { room.saturating_sub(1) } else { rows.len() };
    for (row, (item, date, name, place)) in rows.iter().take(shown).enumerate() {
        let y = rect.y + row as u16;
        let place_x = rect.x + rect.width.saturating_sub(place.chars().count() as u16);
        let name_x = rect.x + item_w + 3 + date_w + 3;
        put(frame, rect.x, y, item_w + 3, vec![ratatui::text::Span::styled(item.clone(), inks.ink)]);
        put(frame, rect.x + item_w + 3, y, date_w + 3, vec![ratatui::text::Span::styled(date.clone(), inks.ink)]);
        put(frame, name_x, y, place_x.saturating_sub(name_x + 2), vec![ratatui::text::Span::styled(cut(name, place_x.saturating_sub(name_x + 2) as usize), inks.ink)]);
        put(frame, place_x, y, rect.width, vec![ratatui::text::Span::styled(place.clone(), inks.dim)]);
    }
    if shown < rows.len() {
        put(frame, rect.x, rect.y + shown as u16, rect.width, vec![ratatui::text::Span::styled(format!("and {} more", rows.len() - shown), inks.dim)]);
    }
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
    let spacing = if area.height >= 30 { 2 } else { 1 };
    let y = list(app, placements, left, spacing);
    let order = app.checks();
    if let Some(extra) = app.check.checked_sub(order.len()) {
        let holidays_chosen = extra == 1;
        let rect = if wide {
            let right = Rect { x: left.width, width: body.width - left.width, ..body };
            let title = if holidays_chosen { format!("HOLIDAYS ACCOUNTED FOR · {}", holidays(app).len()) } else { "CHECKED · NOTHING FOUND".to_string() };
            panel(frame, inks, right, &title, false);
            Rect { x: right.x + 3, y: right.y + 3, width: right.width.saturating_sub(6), height: right.height.saturating_sub(5) }
        } else {
            let top = y + u16::from(y + 2 < left.y + left.height);
            Rect { x: left.x + 3, y: top, width: left.width.saturating_sub(6), height: (left.y + left.height).saturating_sub(top + 1) }
        };
        quiet(app, frame, inks, placements, rect, holidays_chosen, spacing);
        return;
    }
    let Some(&index) = order.get(app.check) else { return };
    let check = &app.analysis.checks[index];
    let advice = text::advice(&app.plan, &app.calendars, app.today, check);
    let p = app.shown();
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
    let p = app.shown();
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
    let last = app.check.min(order.len().saturating_sub(1));
    let mut first = 0;
    while first < last && heights[first..=last].iter().sum::<u16>() > room {
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
    let fine = fine(app).len();
    let days = holidays(app).len();
    for (at, (icon, quiet)) in [(Icon::Check, format!("{fine} other {} found nothing", if fine == 1 { "check" } else { "checks" })), (Icon::Calendar, format!("{days} {} accounted for", if days == 1 { "holiday" } else { "holidays" }))].into_iter().enumerate() {
        if y >= bottom {
            break;
        }
        if app.check == order.len() + at && y + 2 <= bottom {
            picture(app, placements, Rect { x: rect.x + 1, y, width: rect.width.saturating_sub(2), height: 2 }, |pw, ph, u| bits::row(icon, p.dim, &quiet, None, true, pw, ph, u, &p));
            y += spacing.max(2);
            continue;
        }
        picture(app, placements, Rect { x: rect.x + 2, y, width: rect.width.saturating_sub(4), height: 1 }, |pw, ph, u| bits::line(&[(Some((icon, p.dim)), quiet, p.dim, false)], pw, ph, u, &p));
        y += spacing;
    }
    y
}
