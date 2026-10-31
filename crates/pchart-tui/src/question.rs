//! The date question (picture 37), drawn into the roadmap picture over the dimmed chart: the question,
//! the four answers with only the chosen one explained, the date as typed and as understood, and what
//! would follow.

use pchart_core::analysis::State;
use pchart_core::input;
use pchart_draw::bits::{self, Icon};
use pchart_draw::scene::{Anchor, Fill, Scene, Shape, Stroke};
use pchart_draw::situation::state_words;
use pchart_draw::style::ADVANCE;

use crate::app::{ANSWERS, App};
use crate::ui::day;

/// What each answer means, shown under the chosen one; "after" names what the item waits on.
fn meaning(choice: usize, after: &str) -> String {
    match choice {
        0 => format!("calculated, moves when {after} moves"),
        1 => "never moves; pChart warns if the plan cannot hold it".into(),
        2 => "not before this date, later if the plan needs it".into(),
        _ => "must be done by this date; a deadline".into(),
    }
}

/// Draws the open question into `s` at `x`, `y`, `width` × `height` pixels.
pub fn dialog(app: &App, s: &mut Scene, x: f32, y: f32, width: f32, height: f32, unit: f32) {
    let Some(question) = &app.question else { return };
    let (p, u) = (app.palette, unit);
    let item = &app.plan.items[question.item];
    s.add(Shape::Rect { x, y, w: width, h: height, radius: 0.0, fill: Fill::Color(p.page), stroke: Some(Stroke { color: p.rule, width: u, dash: None }) });
    let corner = 16.0 * u;
    for (cx, cy, dx, dy) in [(x, y, 1.0, 1.0), (x + width, y, -1.0, 1.0), (x, y + height, 1.0, -1.0), (x + width, y + height, -1.0, -1.0)] {
        s.add(Shape::Path { points: vec![(cx + dx * corner, cy), (cx, cy), (cx, cy + dy * corner)], closed: false, fill: Fill::None, stroke: Some(Stroke { color: p.ink, width: 2.0 * u, dash: None }) });
    }
    let fit = |text: &str, size: f32, room: f32| {
        let count = (room / (size * ADVANCE)).floor().max(0.0) as usize;
        if text.chars().count() <= count { text.to_string() } else { text.chars().take(count.saturating_sub(1)).collect::<String>() + "…" }
    };
    let left = x + 26.0 * u;
    let room = width - 52.0 * u;
    bits::runs(s, x + 20.0 * u, y + 29.0 * u, 16.6 * u, &[(&fit(&format!(":: CHANGE A DATE · {}", item.name.to_uppercase()), 16.6 * u, room), p.dim, false)]);
    let ask = if item.milestone { format!("When is {}?", item.name) } else { format!("When does {} start?", item.name) };
    bits::runs(s, left, y + 74.0 * u, (24.0 * u).min(room / (ask.chars().count() as f32 * ADVANCE)), &[(&ask, p.bright, true)]);

    let after: Vec<&str> = item.after.iter().filter_map(|id| app.plan.item(id)).map(|index| app.plan.items[index].name.as_str()).collect();
    let after = if after.is_empty() { "the start of the plan".to_string() } else { after.join(", ") };
    let current = [(1, item.on), (2, item.no_earlier_than), (3, item.no_later_than)].into_iter().find(|(_, date)| date.is_some()).map_or(0, |(choice, _)| choice);
    let names = [format!("After {after}"), "On a fixed date".into(), "No earlier than".into(), "No later than".into()];
    let mut top = y + 100.0 * u;
    for (choice, name) in names.iter().enumerate() {
        let chosen = choice == question.choice;
        let tall = if chosen { 50.0 * u } else { 33.0 * u };
        if chosen {
            s.add(Shape::Rect { x: x + 8.0 * u, y: top, w: width - 16.0 * u, h: tall - 6.0 * u, radius: 0.0, fill: Fill::Color(p.raised), stroke: None });
            s.add(Shape::Rect { x: x + 8.0 * u, y: top, w: 4.0 * u, h: tall - 6.0 * u, radius: 0.0, fill: Fill::Color(p.accent), stroke: None });
            bits::runs(s, left, top + 39.0 * u, 15.0 * u, &[(&fit(&meaning(choice, &after), 15.0 * u, room), p.dim, false)]);
        }
        bits::runs(s, left, top + 18.0 * u, 17.0 * u, &[(&fit(name, 17.0 * u, room - 60.0 * u), if chosen { p.bright } else { p.ink }, chosen)]);
        if choice == current {
            s.add(Shape::Text { x: x + width - 26.0 * u, y: top + 18.0 * u, text: "now".into(), size: 15.0 * u, color: p.muted, bold: false, anchor: Anchor::End });
        }
        top += tall;
    }

    top += 18.0 * u;
    if ANSWERS[question.choice].is_some() {
        bits::runs(s, left, top + 17.0 * u, 16.6 * u, &[("Date", p.muted, false)]);
        let box_w = 150.0 * u;
        s.add(Shape::Rect { x: left + 70.0 * u, y: top, w: box_w, h: 25.0 * u, radius: 3.0 * u, fill: Fill::None, stroke: Some(Stroke { color: p.ink, width: 1.2 * u, dash: None }) });
        let typed = fit(&question.typed, 16.6 * u, box_w - 24.0 * u);
        let end = bits::runs(s, left + 80.0 * u, top + 18.0 * u, 16.6 * u, &[(&typed, p.bright, false)]);
        s.add(Shape::Rect { x: end + u, y: top + 4.0 * u, w: 2.0 * u, h: 17.0 * u, radius: 0.0, fill: Fill::Color(p.bright), stroke: None });
        let echo = match input::date(&question.typed, app.today) {
            Some(date) => (format!("→  {}", date.strftime("%a %d %b %Y")), p.bright, true),
            None if question.typed.trim().is_empty() => ("such as 4 jan, next friday, W12".to_string(), p.muted, false),
            None => ("not a date yet".to_string(), p.muted, false),
        };
        bits::runs(s, left + 70.0 * u + box_w + 20.0 * u, top + 18.0 * u, 16.6 * u, &[(&fit(&echo.0, 16.6 * u, width - 300.0 * u), echo.1, echo.2)]);
    }
    top += 63.0 * u;
    s.add(Shape::Line { x1: x + 16.0 * u, y1: top - 10.0 * u, x2: x + width - 16.0 * u, y2: top - 10.0 * u, stroke: Stroke { color: p.rule, width: u, dash: None } });
    if top + 60.0 * u > y + height {
        return;
    }
    let Some(edit) = app.answer() else {
        let words = if ANSWERS[question.choice].is_none() { "Nothing changes." } else { "Type the date to see what follows." };
        bits::runs(s, left, top + 28.0 * u, 16.6 * u, &[(words, p.muted, false)]);
        return;
    };
    follows(app, s, &[edit], left, top, room, u);
}

/// What would follow from `edits`, under a dialog at `left`, `top`: the verdict as it would be, and what
/// moves, or where a new item would lie.
#[allow(clippy::too_many_arguments)]
pub fn follows(app: &App, s: &mut Scene, edits: &[pchart_core::Edit], left: f32, top: f32, room: f32, u: f32) {
    let p = app.palette;
    let fit = |text: &str, size: f32, room: f32| {
        let count = (room / (size * ADVANCE)).floor().max(0.0) as usize;
        if text.chars().count() <= count { text.to_string() } else { text.chars().take(count.saturating_sub(1)).collect::<String>() + "…" }
    };
    let tried = pchart_core::apply_all(&app.plan, edits).map_err(|(_, err)| err.to_string()).and_then(|plan| pchart_core::schedule(&plan, &app.calendars, app.today).map(|schedule| (plan, schedule)).map_err(|err| err.to_string()));
    let (plan, schedule) = match tried {
        Ok(tried) => tried,
        Err(err) => {
            bits::runs(s, left, top + 28.0 * u, 16.6 * u, &[(&fit(&err, 16.6 * u, room), p.alert, false)]);
            return;
        }
    };
    let analysis = pchart_core::analyse(&plan, &app.calendars, &schedule, app.today);
    if let Some(verdict) = &analysis.verdict {
        let (icon, tone) = match verdict.state {
            State::Holds => (Icon::Check, p.bright),
            State::Tight => (Icon::Clock, p.warn),
            State::DoesNotHold => (Icon::Cross, p.alert),
        };
        bits::draw_icon(s, icon, left + 18.0 * u, top + 24.0 * u, 36.0 * u, tone, p.page);
        let target = &plan.items[verdict.target].name;
        let wd = |count: i32| if count.abs() == 1 { format!("{count} working day") } else { format!("{count} working days") };
        let parts: Vec<(String, pchart_draw::scene::Color)> = match verdict.state {
            State::DoesNotHold => vec![(format!("{target} would be "), p.bright), (format!("{} late.", wd(-verdict.spare)), p.alert)],
            State::Tight => vec![(format!("{target} would be {} ", state_words(verdict.state)), p.bright), (format!("with only {} to spare.", wd(verdict.spare)), p.warn)],
            State::Holds => vec![(format!("{target} stays {}, ", state_words(verdict.state)), p.bright), (format!("with {} to spare.", wd(verdict.spare)), p.bright)],
        };
        let size = (20.0 * u).min((room - 50.0 * u) / (parts.iter().map(|(text, _)| text.chars().count()).sum::<usize>() as f32 * ADVANCE));
        let parts: Vec<(&str, pchart_draw::scene::Color, bool)> = parts.iter().map(|(text, color)| (text.as_str(), *color, true)).collect();
        bits::runs(s, left + 50.0 * u, top + 32.0 * u, size, &parts);
    }
    let before = |index: usize| app.plan.item(&plan.items[index].id);
    let moved = (0..plan.items.len())
        .filter(|&index| Some(index) != schedule.target && before(index).is_some_and(|old| schedule.slots[index].planned != app.schedule.slots[old].planned))
        .max_by_key(|&index| schedule.slots[index].planned.finish);
    let moves = match moved {
        Some(index) => format!("{} moves to {}", plan.items[index].name, day(schedule.slots[index].planned.finish)),
        None => "Nothing else moves.".to_string(),
    };
    let words = match (0..plan.items.len()).find(|&index| before(index).is_none()) {
        Some(new) if plan.items[new].milestone => format!("{} on {} · {moves}", plan.items[new].name, day(schedule.slots[new].planned.finish)),
        Some(new) => format!("{} {} – {} · {moves}", plan.items[new].name, day(schedule.slots[new].planned.start), day(schedule.slots[new].planned.finish)),
        None => moves,
    };
    bits::runs(s, left + 50.0 * u, top + 58.0 * u, 16.0 * u, &[(&fit(&words, 16.0 * u, room - 50.0 * u), p.dim, false)]);
}
