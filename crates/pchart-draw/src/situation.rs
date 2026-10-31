//! The pictures of the situation screen (picture 32): the verdict, the steps of "Next" and the chain
//! that decides the date. `unit` is one pixel of a 21 px tall cell.

use pchart_core::analysis::{State, Verdict};
use pchart_core::{Plan, Schedule};

use crate::bits::{Icon, draw_icon, runs};
use crate::scene::{Anchor, Color, Fill, Mark, Scene, Shape, Stroke};
use crate::style::{ADVANCE, FONT, Palette};

fn scene(width: f32, height: f32, palette: &Palette) -> Scene {
    Scene { width, height, background: None, hatch: (palette.raised, palette.muted), hatch_alert: (palette.alert_ground, palette.alert), spare: palette.dim, font: FONT, marks: Vec::new() }
}

fn text(x: f32, y: f32, text: String, size: f32, color: Color, bold: bool, anchor: Anchor) -> Shape {
    Shape::Text { x, y, text, size, color, bold, anchor }
}

/// `text` cut to `room` pixels at `size`, ending in "…" when it was cut.
fn fit(text: &str, size: f32, room: f32) -> String {
    let fits = |count: usize| count as f32 * size * ADVANCE <= room;
    let count = text.chars().count();
    if fits(count) {
        return text.to_string();
    }
    let keep = (0..count).rev().find(|&keep| fits(keep + 1)).unwrap_or(0);
    text.chars().take(keep).collect::<String>() + "…"
}

/// What the verdict says: on track, at risk, late.
pub fn state_words(state: State) -> &'static str {
    match state {
        State::Holds => "on track",
        State::Tight => "at risk",
        State::DoesNotHold => "late",
    }
}

/// The verdict as picture 32 shows it: a symbol, "Launch on Mon 01 Mar is on track." and under it the
/// number once. Text that does not fit `width` gets smaller.
pub fn verdict(plan: &Plan, verdict: &Verdict, width: f32, height: f32, unit: f32, palette: &Palette) -> Scene {
    let (p, u) = (palette, unit);
    let mut s = scene(width, height, p);
    let k = (height / (8.0 * 21.0 * u)).min(1.0);
    let (icon, tone) = match verdict.state {
        State::Holds => (Icon::Check, p.bright),
        State::Tight => (Icon::Clock, p.warn),
        State::DoesNotHold => (Icon::Cross, p.alert),
    };
    draw_icon(&mut s, icon, 26.0 * u * k, 47.0 * u * k, 52.0 * u * k, tone, p.page);
    let lead = format!("{} on {} is ", plan.items[verdict.target].name, verdict.date.strftime("%a %d %b"));
    let words = format!("{}.", state_words(verdict.state));
    let x = 74.0 * u * k;
    let room = width - x - 4.0 * u;
    let size = (46.0 * u * k).min(room / ((lead.chars().count() + words.chars().count()) as f32 * ADVANCE));
    runs(&mut s, x, 63.0 * u * k, size, &[(&lead, p.bright, true), (&words, tone, true)]);
    let (number, rest) = if verdict.spare < 0 { (-verdict.spare, "working days late") } else { (verdict.spare, "working days to spare") };
    let rest = if number == 1 { rest.replacen("days", "day", 1) } else { rest.to_string() };
    let number = number.to_string();
    let end = runs(&mut s, x, 147.0 * u * k, size, &[(&number, if verdict.state == State::Holds { p.bright } else { tone }, true)]);
    runs(&mut s, end + 16.0 * u * k, 143.0 * u * k, (22.0 * u * k).min(size), &[(&rest, p.dim, false)]);
    s
}

/// One step of "Next": a symbol, a title, and one line of symbols and words under it.
pub struct Statement {
    pub icon: Icon,
    pub tone: Color,
    pub title: String,
    pub strong: bool,
    pub line: Vec<(Option<Icon>, String)>,
}

/// `statement` in a picture four cells high, as in picture 32; the chosen one lies on a raised band
/// with a pink edge, as the chosen row of a list (picture 36, B). Its symbol stands 40 units in.
pub fn statement(statement: &Statement, chosen: bool, width: f32, height: f32, unit: f32, palette: &Palette) -> Scene {
    let (p, u) = (palette, unit);
    let mut s = scene(width, height, p);
    let ground = if chosen { p.raised } else { p.page };
    if chosen {
        s.add(Shape::Rect { x: 0.0, y: 2.0 * u, w: width, h: 68.0 * u, radius: 0.0, fill: Fill::Color(p.raised), stroke: None });
        s.add(Shape::Rect { x: 0.0, y: 2.0 * u, w: 4.0 * u, h: 68.0 * u, radius: 0.0, fill: Fill::Color(p.accent), stroke: None });
    }
    let x0 = 40.0 * u;
    let size = if statement.strong { 24.0 } else { 20.0 } * u;
    draw_icon(&mut s, statement.icon, x0 + 26.0 * u, 21.0 * u, if statement.strong { 28.0 } else { 26.0 } * u, statement.tone, ground);
    let room = width - x0 - 78.0 * u;
    let title = fit(&statement.title, size, room);
    runs(&mut s, x0 + 74.0 * u, 29.0 * u, size, &[(&title, if statement.strong { p.bright } else { p.ink }, statement.strong)]);
    let mut x = x0 + 74.0 * u;
    for (icon, text) in &statement.line {
        if let Some(icon) = icon {
            draw_icon(&mut s, *icon, x + 8.0 * u, 57.0 * u, 16.0 * u, p.dim, ground);
            x += 24.0 * u;
        }
        let text = fit(text, 16.0 * u, (width - x).max(0.0));
        x = runs(&mut s, x, 62.0 * u, 16.0 * u, &[(&text, p.dim, false)]) + 32.0 * u;
    }
    s
}

/// The deciding chain as one line of steps: name above, bar or diamond, date below, and under them a
/// bracket for each lane with its name and city.
pub fn chain(plan: &Plan, schedule: &Schedule, width: f32, height: f32, unit: f32, palette: &Palette) -> Scene {
    let (p, u) = (palette, unit);
    let mut s = scene(width, height, p);
    let steps = &schedule.chain;
    let count = steps.len();
    if count == 0 {
        return s;
    }
    let (x0, x1) = (90.0 * u, width - 90.0 * u);
    let step = if count > 1 { (x1 - x0) / (count - 1) as f32 } else { 0.0 };
    let x = |position: usize| if count > 1 { x0 + position as f32 * step } else { width / 2.0 };
    let cy = 84.0 * u;
    let room = |position: usize| {
        let edge = 2.0 * x(position).min(width - x(position));
        match position {
            _ if count == 1 => width,
            0 => ((step - 6.0 * u) * 1.3).min(edge),
            last if last + 1 == count => ((step - 6.0 * u) * 1.3).min(edge),
            _ => step - 6.0 * u,
        }
    };
    for (position, &index) in steps.iter().enumerate() {
        let item = &plan.items[index];
        let slot = &schedule.slots[index].planned;
        let xs = x(position);
        if position + 1 < count {
            let reach = |index: usize| if plan.items[index].milestone { 12.0 * u } else { 40.0 * u };
            let (xa, xb) = (xs + reach(index), x(position + 1) - reach(steps[position + 1]));
            s.add(Shape::Line { x1: xa, y1: cy, x2: xb - 5.0 * u, y2: cy, stroke: Stroke { color: p.muted, width: 1.4 * u, dash: None } });
            s.add(Shape::Path { points: vec![(xb, cy), (xb - 6.0 * u, cy - 3.5 * u), (xb - 6.0 * u, cy + 3.5 * u)], closed: true, fill: Fill::Color(p.muted), stroke: None });
        }
        let late = schedule.slots[index].spare < 0;
        let ink = if late { p.alert } else { p.ink };
        let fixed = item.on.is_some();
        if fixed {
            let r = 9.0 * u;
            s.add_for(index, 1.0, Shape::Path { points: vec![(xs, cy - r), (xs + r, cy), (xs, cy + r), (xs - r, cy)], closed: true, fill: Fill::Color(if late { p.alert } else { p.bright }), stroke: None });
            s.add(Shape::Line { x1: xs, y1: cy - r - 2.0 * u, x2: xs, y2: cy - r - 12.0 * u, stroke: Stroke { color: p.bright, width: 1.6 * u, dash: None } });
            s.add(Shape::Circle { x: xs, y: cy - r - 15.0 * u, r: 4.0 * u, fill: p.bright });
        } else if item.milestone {
            let r = 8.0 * u;
            s.add_for(index, 1.0, Shape::Path { points: vec![(xs, cy - r), (xs + r, cy), (xs, cy + r), (xs - r, cy)], closed: true, fill: Fill::Color(ink), stroke: None });
        } else {
            s.marks.push(Mark { shape: Shape::Rect { x: xs - 34.0 * u, y: cy - 7.0 * u, w: 68.0 * u, h: 14.0 * u, radius: 3.0 * u, fill: Fill::Color(ink), stroke: None }, opacity: 0.95, item: Some(index) });
        }
        let name_y = if fixed { cy - 30.0 * u } else { cy - 20.0 * u };
        s.add_for(index, 1.0, text(xs, name_y, fit(&item.name, 14.5 * u, room(position)), 14.5 * u, if late { p.alert } else { p.bright }, true, Anchor::Middle));
        let date = if fixed {
            format!("{} · fixed", slot.finish.strftime("%a %d %b"))
        } else if item.milestone {
            slot.finish.strftime("%a %d %b").to_string()
        } else {
            format!("{} – {}", slot.start.strftime("%d %b"), slot.finish.strftime("%d %b"))
        };
        s.add_for(index, 1.0, text(xs, cy + 30.0 * u, fit(&date, 13.5 * u, room(position)), 13.5 * u, if fixed { p.bright } else { p.ink }, false, Anchor::Middle));
    }
    let mut groups: Vec<(usize, usize, usize)> = Vec::new();
    for (position, &index) in steps.iter().enumerate() {
        let lane = plan.lane(&plan.items[index].lane).unwrap_or(0);
        match groups.last_mut() {
            Some(group) if group.0 == lane => group.2 = position,
            _ => groups.push((lane, position, position)),
        }
    }
    let by = cy + 62.0 * u;
    for (lane, first, last) in groups {
        let (xa, xb) = (x(first) - 62.0 * u, x(last) + 62.0 * u);
        s.add(Shape::Path { points: vec![(xa, by - 6.0 * u), (xa, by), (xb, by), (xb, by - 6.0 * u)], closed: false, fill: Fill::None, stroke: Some(Stroke { color: p.muted, width: 1.2 * u, dash: None }) });
        let lane = &plan.lanes[lane];
        let label = [Some(lane.name.as_str()), lane.city.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
        let middle = (xa + xb) / 2.0;
        let span = (last - first + 1) as f32 * step.max(124.0 * u) - 2.0 * u;
        s.add(text(middle, by + 20.0 * u, fit(&label, 13.5 * u, span.min(2.0 * middle.min(width - middle))), 13.5 * u, p.dim, false, Anchor::Middle));
    }
    s
}
