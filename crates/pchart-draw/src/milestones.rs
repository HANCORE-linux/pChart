use jiff::civil::{Date, Weekday};
use pchart_core::calendar::{next, previous};
use pchart_core::{Plan, Schedule};

use crate::scene::{Anchor, Color, Fill, Scene, Shape, Stroke};
use crate::style::{SANS, sans_width};

/// Size of the page: a 16:9 slide.
pub const PAGE: (f32, f32) = (1124.0, 632.0);

const PAPER: Color = Color(0xff, 0xff, 0xff);
const INK: Color = Color(0x1f, 0x24, 0x21);
const GRAY: Color = Color(0x6b, 0x70, 0x6b);
const LIGHT: Color = Color(0xee, 0xf0, 0xea);
const MID: Color = Color(0xc9, 0xcd, 0xc4);
const SHADES: [Color; 4] = [Color(0xd9, 0xdd, 0xd3), Color(0xcd, 0xd2, 0xc6), Color(0xbf, 0xc5, 0xb7), Color(0xb1, 0xb8, 0xa8)];
const TARGET: Color = Color(0xe3, 0x3d, 0x84);
const LATE: Color = Color(0xee, 0x58, 0x5d);

/// The milestone chart for export: phases on a time line and every milestone with its date, on a
/// light page for slides and print.
pub fn milestones(plan: &Plan, schedule: &Schedule, today: Date) -> Scene {
    let (width, height) = PAGE;
    let mut s = Scene { width, height, background: PAPER, hatch: (LIGHT, MID), hatch_alert: (LIGHT, LATE), spare: MID, font: SANS, marks: Vec::new() };
    let text = |x: f32, y: f32, text: String, size: f32, color: Color, bold: bool, anchor: Anchor| Shape::Text { x, y, text, size, color, bold, anchor };
    s.add(text(56.0, 70.0, plan.name.clone(), 34.0, INK, true, Anchor::Start));
    s.add(text(56.0, 102.0, format!("Milestone chart · as of {}", today.strftime("%d %b %Y")), 17.0, GRAY, false, Anchor::Start));

    let first = schedule.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(today).min(today);
    let last = schedule.slots.iter().map(|slot| slot.planned.finish).max().unwrap_or(today);
    let mut start = first;
    while start.weekday() != Weekday::Monday {
        start = previous(start);
    }
    let end = (0..13).fold(last, |day, _| next(day));
    let (x0, x1) = (56.0, width - 56.0);
    let px = (x1 - x0) / (days(start, end) + 1) as f32;
    let x = |day: Date| x0 + days(start, day) as f32 * px;

    let ty = 190.0;
    let mut month = next(start.last_of_month());
    let mut labelled = false;
    while month <= end {
        s.add(Shape::Line { x1: x(month), y1: ty - 22.0, x2: x(month), y2: height - 90.0, stroke: Stroke { color: LIGHT, width: 1.5, dash: None } });
        let label = if !labelled || month.month() == 1 { month.strftime("%b %Y").to_string() } else { month.strftime("%b").to_string() };
        s.add(text(x(month) + 8.0, ty - 4.0, label, 15.0, GRAY, false, Anchor::Start));
        labelled = true;
        month = next(month.last_of_month());
    }
    s.add(Shape::Line { x1: x0, y1: ty + 6.0, x2: x1, y2: ty + 6.0, stroke: Stroke { color: MID, width: 1.5, dash: None } });

    for (index, (name, from, to)) in schedule.phases(plan).into_iter().enumerate() {
        chevron(&mut s, x(from), x(to) + px, ty + 34.0, 52.0, &name, SHADES[index % SHADES.len()]);
    }

    let my = ty + 214.0;
    s.add(Shape::Line { x1: x0, y1: my, x2: x1, y2: my, stroke: Stroke { color: MID, width: 2.0, dash: None } });
    let xt = x(today) + px / 2.0;
    s.add(Shape::Path { points: vec![(xt, ty + 14.0), (xt - 6.0, ty + 5.0), (xt + 6.0, ty + 5.0)], closed: true, fill: Fill::Color(GRAY), stroke: None });
    s.add(text(xt, ty + 30.0, "today".into(), 12.5, GRAY, false, Anchor::Middle));

    let point = |index: usize| {
        let day = schedule.slots[index].planned.finish;
        if plan.items[index].on.is_some() { x(day) + px / 2.0 } else { x(day) + px }
    };
    let mut order: Vec<usize> = (0..plan.items.len()).filter(|&index| plan.items[index].milestone).collect();
    order.sort_by_key(|&index| (Some(index) != schedule.target, schedule.slots[index].planned.finish));
    let mut taken: Vec<[f32; 4]> = Vec::new();
    for index in order {
        let item = &plan.items[index];
        let slot = &schedule.slots[index];
        let xm = point(index);
        let late = slot.spare < 0;
        let target = Some(index) == schedule.target;
        let (r, name_size, color) = if target { (13.0, 19.0, if late { LATE } else { TARGET }) } else { (10.0, 17.0, if late { LATE } else { INK }) };
        s.add_for(index, 1.0, Shape::Path { points: vec![(xm, my - r), (xm + r, my), (xm, my + r), (xm - r, my)], closed: true, fill: Fill::Color(color), stroke: None });
        let date = format!("{}{}", slot.planned.finish.strftime("%d %b %Y"), if item.on.is_some() { " · fixed" } else { "" });
        let w = sans_width(&item.name, name_size, true).max(sans_width(&date, 15.0, false));
        let cx = if target { xm + 4.0 } else { xm };
        let below = [cx - w / 2.0, my + 40.0 - name_size, cx + w / 2.0, my + 66.0];
        let above = [cx - w / 2.0, my - 46.0 - name_size, cx + w / 2.0, my - 20.0];
        let fits = |area: &[f32; 4]| area[0] >= 8.0 && area[2] <= width - 8.0 && !taken.iter().any(|other| area[0] < other[2] && other[0] < area[2] && area[1] < other[3] && other[1] < area[3]);
        let Some((area, name_y)) = [(below, my + 40.0), (above, my - 46.0)].into_iter().find(|(area, _)| fits(area)) else { continue };
        let name_y = if target { name_y + 6.0 } else { name_y };
        taken.push([area[0] - 6.0, area[1], area[2] + 6.0, area[3]]);
        s.add_for(index, 1.0, text(cx, name_y, item.name.clone(), name_size, color, true, Anchor::Middle));
        s.add_for(index, 1.0, text(cx, name_y + 22.0, date, 15.0, if target || late { color } else { GRAY }, false, Anchor::Middle));
    }

    s.add(Shape::Line { x1: 56.0, y1: height - 60.0, x2: width - 56.0, y2: height - 60.0, stroke: Stroke { color: LIGHT, width: 1.5, dash: None } });
    let source = match schedule.target {
        Some(target) => format!("Dates scheduled backwards from the fixed {} date", common(&plan.items[target].name)),
        None => "Dates scheduled forwards from the start of the plan".to_string(),
    };
    s.add(text(56.0, height - 30.0, source, 14.0, GRAY, false, Anchor::Start));
    s.add(text(width - 56.0, height - 30.0, "pChart".into(), 14.0, GRAY, true, Anchor::End));
    s
}

/// A name inside a sentence: "Launch" becomes "launch", "SAP cutover" stays as it is.
fn common(name: &str) -> String {
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(first), Some(second)) if first.is_uppercase() && second.is_lowercase() => first.to_lowercase().chain(name.chars().skip(1)).collect(),
        _ => name.to_string(),
    }
}

fn chevron(s: &mut Scene, x0: f32, x1: f32, y: f32, h: f32, label: &str, fill: Color) {
    let tip = 14.0_f32.min((x1 - x0) / 2.0);
    let points = vec![(x0, y), (x1 - tip, y), (x1, y + h / 2.0), (x1 - tip, y + h), (x0, y + h), (x0 + tip, y + h / 2.0)];
    s.add(Shape::Path { points, closed: true, fill: Fill::Color(fill), stroke: Some(Stroke { color: fill, width: 1.0, dash: None }) });
    let room = x1 - x0 - tip - 8.0;
    let mut size = 17.0;
    let mut label = label.to_string();
    if sans_width(&label, size, false) > room {
        size = (size * room / sans_width(&label, size, false)).max(10.5);
    }
    while sans_width(&label, size, false) > room && label.chars().count() > 3 {
        let keep = label.chars().count() - 2;
        label = label.chars().take(keep).collect::<String>() + "…";
    }
    s.add(Shape::Text { x: (x0 + x1) / 2.0 + tip / 2.0, y: y + h / 2.0 + size * 0.36, text: label, size, color: INK, bold: false, anchor: Anchor::Middle });
}

fn days(from: Date, to: Date) -> i32 {
    from.until(to).map(|span| span.get_days()).unwrap_or(0)
}
