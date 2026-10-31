//! The pictures of the situation screen: the chain that decides the date, the next six weeks, and the
//! scale from "does not hold" to "holds". `unit` is one pixel of a 21 px tall cell.

use jiff::civil::{Date, Weekday};
use pchart_core::calendar::next;
use pchart_core::{Plan, Schedule};

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

/// One segment of the scale; the one that applies is filled.
pub fn segment(on: bool, width: f32, height: f32, unit: f32, palette: &Palette) -> Scene {
    let mut s = scene(width, height, palette);
    let (h, y) = (6.0 * unit, height / 2.0 - 3.0 * unit);
    let (fill, stroke) = if on { (Fill::Color(palette.ink), None) } else { (Fill::Color(palette.raised), Some(Stroke { color: palette.muted, width: unit, dash: None })) };
    s.add(Shape::Rect { x: unit, y, w: width - 2.0 * unit, h, radius: 1.5 * unit, fill, stroke });
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

/// The next six weeks from `today`: week marks, the today line, the first step of the deciding chain
/// in the window with its dates above it, and the step after it, fading out, below.
pub fn weeks(plan: &Plan, schedule: &Schedule, today: Date, width: f32, height: f32, unit: f32, palette: &Palette) -> Scene {
    let (p, u) = (palette, unit);
    let mut s = scene(width, height, p);
    let end = (0..42).fold(today, |day, _| next(day));
    let span = 42.0;
    let x = |day: Date| today.until(day).map_or(0.0, |gap| gap.get_days() as f32) / span * width;
    let ly = 63.0 * u;
    s.add(Shape::Line { x1: 0.0, y1: ly, x2: width, y2: ly, stroke: Stroke { color: p.muted, width: 1.5 * u, dash: None } });
    let mut monday = today;
    while monday.weekday() != Weekday::Monday {
        monday = next(monday);
    }
    while monday <= end {
        let xm = x(monday);
        s.add(Shape::Line { x1: xm, y1: ly - 4.0 * u, x2: xm, y2: ly + 4.0 * u, stroke: Stroke { color: p.muted, width: u, dash: None } });
        s.add(text(xm + 3.0 * u, ly + 20.0 * u, format!("W{:02}", monday.iso_week_date().week()), 12.5 * u, p.muted, false, Anchor::Start));
        monday = (0..7).fold(monday, |day, _| next(day));
    }
    s.add(Shape::Line { x1: 0.75 * u, y1: ly - 14.0 * u, x2: 0.75 * u, y2: ly + 8.0 * u, stroke: Stroke { color: p.ink, width: 1.5 * u, dash: None } });
    s.add(text(4.0 * u, ly - 18.0 * u, "today".into(), 13.0 * u, p.bright, true, Anchor::Start));
    let window: Vec<usize> = schedule.chain.iter().copied().filter(|&index| !plan.items[index].milestone && schedule.slots[index].planned.finish >= today && schedule.slots[index].planned.start <= end).collect();
    if let Some(&first) = window.first() {
        let slot = &schedule.slots[first].planned;
        let (xs, xe) = (x(slot.start.max(today)), x(next(slot.finish).min(end)));
        s.marks.push(Mark { shape: Shape::Rect { x: xs, y: ly - 5.0 * u, w: xe - xs, h: 10.0 * u, radius: 2.0 * u, fill: Fill::Color(p.ink), stroke: None }, opacity: 0.95, item: Some(first) });
        let label = format!("{} {} – {}", plan.items[first].name, slot.start.strftime("%d %b"), slot.finish.strftime("%d %b"));
        let w = label.chars().count() as f32 * 13.0 * u * ADVANCE;
        s.add_for(first, 1.0, text(xs.min(width - w).max(0.0), ly - 12.0 * u, label, 13.0 * u, p.ink, false, Anchor::Start));
        if let Some(&then) = window.get(1) {
            let start = schedule.slots[then].planned.start;
            let xu = x(start);
            s.marks.push(Mark { shape: Shape::Rect { x: xu, y: ly - 5.0 * u, w: width - xu, h: 10.0 * u, radius: 2.0 * u, fill: Fill::Color(p.ink), stroke: None }, opacity: 0.35, item: Some(then) });
            let label = format!("{} from {}", plan.items[then].name, start.strftime("%d %b"));
            let w = label.chars().count() as f32 * 13.0 * u * ADVANCE;
            s.add_for(then, 1.0, text((xu + 4.0 * u).min(width - w).max(0.0), ly + 40.0 * u, label, 13.0 * u, p.dim, false, Anchor::Start));
        }
    }
    s
}
