use jiff::civil::{Date, Weekday};
use pchart_core::calendar::next;
use pchart_core::{Plan, Schedule};

use crate::page::{Axis, FIXED, GRAY, HEIGHT, INK, LATE, MID, PAPER, Page, SHADES, X0, X1};
use crate::scene::{Anchor, Fill, Scene, Shape};
use crate::style::sans_width;

/// The phase plan for export: the phases to scale with their dates and length, the milestones that
/// close them as gates, and the fixed target.
pub fn phase_plan(plan: &Plan, schedule: &Schedule, today: Date) -> Scene {
    let mut page = Page::new(plan, schedule, today, "Phase plan", HEIGHT);
    let axis = Axis::new(schedule, today, 20, X0, X1);
    let ty = 190.0;
    page.months(&axis, ty - 4.0, ty - 22.0, HEIGHT - 90.0, 15.0);
    page.line(X0, ty + 6.0, X1, ty + 6.0, MID, 1.5);
    let xt = axis.x(today) + axis.px / 2.0;
    page.scene.add(Shape::Path { points: vec![(xt, ty + 14.0), (xt - 6.0, ty + 5.0), (xt + 6.0, ty + 5.0)], closed: true, fill: Fill::Color(GRAY), stroke: None });
    page.shape([xt - 6.0, ty + 5.0, xt + 6.0, ty + 14.0], None);
    page.text(xt, ty + 30.0, "today", 12.5, GRAY, false, Anchor::Middle, None);

    let (top, h) = (ty + 160.0, 90.0);
    let yc = top + h / 2.0;
    let target = schedule.target.filter(|&target| plan.items[target].milestone);
    let gate = |phase: &pchart_core::model::Phase| plan.item(&phase.until).filter(|&index| plan.items[index].milestone && Some(index) != target);
    let mut blocks: Vec<(f32, f32, String, String)> = Vec::new();
    for (index, ((name, from, to), phase)) in schedule.phases(plan).into_iter().zip(&plan.phases).enumerate() {
        let (x0, x1) = (axis.x(from), axis.xe(to));
        let tip = page.chevron(x0, x1, top, h, SHADES[index % SHADES.len()], 12.0);
        let right = x1 - if gate(phase).is_some() { 14.0 } else { 0.0 };
        let length = weeks(from, to);
        if sans_width(&length, 16.0, false) <= right - x0 - tip - 8.0 {
            page.text((x0 + tip + right) / 2.0, yc + 6.0, &length, 16.0, INK, false, Anchor::Middle, None);
        }
        let dates = format!("{} – {}", from.strftime("%d %b"), to.strftime("%d %b"));
        blocks.push(((x0 + x1) / 2.0 + tip / 2.0, sans_width(&name, 18.0, true).max(sans_width(&dates, 14.5, false)), name, dates));
    }
    for _ in 0..20 {
        for pair in 0..blocks.len().saturating_sub(1) {
            let short = blocks[pair].0 + blocks[pair].1 / 2.0 + 30.0 - (blocks[pair + 1].0 - blocks[pair + 1].1 / 2.0);
            if short > 0.0 {
                blocks[pair].0 -= short / 2.0;
                blocks[pair + 1].0 += short / 2.0;
            }
        }
        for block in &mut blocks {
            block.0 = block.0.clamp(X0 + block.1 / 2.0, X1 - block.1 / 2.0);
        }
    }
    for (x, _, name, dates) in &blocks {
        page.text(*x, top - 42.0, name, 18.0, INK, true, Anchor::Middle, None);
        page.text(*x, top - 17.0, dates, 14.5, GRAY, false, Anchor::Middle, None);
    }

    let gates: Vec<usize> = plan.phases.iter().filter_map(gate).collect();
    let ink = |index: usize| {
        if schedule.slots[index].spare < 0 {
            LATE
        } else if plan.items[index].on.is_some() {
            FIXED
        } else {
            INK
        }
    };
    for &index in &gates {
        let (x, color) = (axis.milestone(plan, schedule, index), if schedule.slots[index].spare < 0 { LATE } else { FIXED });
        if plan.items[index].important {
            page.star(x, yc, 13.0, color, Some((PAPER, 2.5)), Some(index));
        } else {
            page.diamond(x, yc, 11.0, color, Some((PAPER, 2.5)), Some(index));
        }
    }
    let bounds = [X0, top + h, X1, HEIGHT - 70.0];
    for &index in &gates {
        let x = axis.milestone(plan, schedule, index);
        let y = top + h + 30.0;
        let places = [(x + 4.0, y, Anchor::End), (x, y, Anchor::Middle), (x - 4.0, y, Anchor::Start), (x + 4.0, y + 22.0, Anchor::End), (x, y + 22.0, Anchor::Middle)];
        page.place(&places, &plan.items[index].name, 15.5, ink(index), true, Some(index), bounds);
    }

    if let Some(index) = target {
        let xl = axis.milestone(plan, schedule, index);
        let color = if schedule.slots[index].spare < 0 { LATE } else { FIXED };
        page.diamond(xl, yc, 14.0, color, Some((PAPER, 2.5)), Some(index));
        let day = schedule.slots[index].planned.finish;
        let fixed = if plan.items[index].on.is_some() { " · fixed" } else { "" };
        let (left, anchor) = if xl + 20.0 + sans_width(&plan.items[index].name, 19.0, true) <= X1 { (xl + 20.0, Anchor::Start) } else { (xl - 20.0, Anchor::End) };
        page.text(left, yc - 4.0, &plan.items[index].name, 19.0, color, true, anchor, Some(index));
        let date = format!("{}{fixed}", day.strftime("%d %b %Y"));
        if anchor == Anchor::End || left + sans_width(&date, 14.0, false) <= X1 {
            page.text(left, yc + 18.0, &date, 14.0, color, false, anchor, Some(index));
        } else {
            page.text(left, yc + 16.0, &day.strftime("%d %b %Y").to_string(), 14.0, color, false, anchor, Some(index));
            page.text(left, yc + 34.0, fixed.trim_start_matches(" · "), 14.0, color, false, anchor, Some(index));
        }
    }
    page.finish()
}

/// Length in weeks of five weekdays, to the half week: "2 weeks", "7½ weeks".
fn weeks(from: Date, to: Date) -> String {
    let mut weekdays = 0;
    let mut day = from;
    while day <= to {
        if !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday) {
            weekdays += 1;
        }
        day = next(day);
    }
    let halves = ((f64::from(weekdays) / 5.0 * 2.0).round() as i64).max(1);
    let whole = if halves == 1 { String::new() } else { (halves / 2).to_string() };
    let count = format!("{whole}{}", if halves % 2 == 1 { "½" } else { "" });
    if halves <= 2 { format!("{count} week") } else { format!("{count} weeks") }
}
