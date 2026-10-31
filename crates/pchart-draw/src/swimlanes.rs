use std::collections::HashSet;

use jiff::civil::Date;
use pchart_core::analysis::Check;
use pchart_core::{Plan, Schedule};

use crate::page::{Axis, FIXED, GRAY, INK, Key, LATE, LIGHT, MID, Page, SHADES, SOFT, WARN_LINE, X0, X1, fit, rows, wrap};
use crate::scene::{Anchor, Fill, Scene, Shape, Stroke};
use crate::style::{phase_label, sans_width};
use crate::views::Audience;

const LANE: f32 = 51.0;
const ROW: f32 = 0.6;
const ROWS_Y: f32 = 178.0;
const PHASES_Y: f32 = 156.0;

/// The swimlane roadmap for export on a light page: the Standard level with the place of every lane,
/// the fixed target, warnings and a legend.
pub fn swimlanes(plan: &Plan, schedule: &Schedule, today: Date, checks: &[Check], audience: Audience) -> Scene {
    let checks = if audience.warnings() { checks } else { &[] };
    let chain: HashSet<usize> = schedule.chain.iter().copied().collect();
    let lanes: Vec<usize> = plan.items.iter().map(|item| plan.lane(&item.lane).unwrap_or(0)).collect();
    let place = |index: usize| {
        let lane = &plan.lanes[index];
        [lane.city.as_deref(), lane.place.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ")
    };
    let label_w = (0..plan.lanes.len()).map(|index| sans_width(&plan.lanes[index].name, 14.0, false).max(sans_width(&place(index), 12.0, false))).fold(0.0, f32::max).min(180.0);
    let gx0 = X0 + label_w + 26.0;
    let axis = Axis::new(schedule, today, 20, gx0, X1);
    let ends = |index: usize| {
        if plan.items[index].milestone {
            let x = axis.milestone(plan, schedule, index);
            (x, x)
        } else {
            let span = schedule.slots[index].planned;
            (axis.x(span.start), axis.xe(span.finish))
        }
    };
    let extents: Vec<(usize, f32, f32, bool)> = (0..plan.items.len()).map(|index| (lanes[index], ends(index).0, ends(index).1, plan.items[index].milestone)).collect();
    let row = rows(&extents, 7.0);
    let lane_lines: Vec<Vec<String>> = plan.lanes.iter().map(|lane| wrap(&lane.name, 14.0, false, label_w)).collect();
    let mut tops = vec![ROWS_Y];
    for lane in 0..plan.lanes.len() {
        let count = (0..plan.items.len()).filter(|&index| lanes[index] == lane).map(|index| row[index] + 1).max().unwrap_or(1);
        tops.push(tops[lane] + (LANE * (1.0 + ROW * (count - 1) as f32)).max(LANE + 17.0 * (lane_lines[lane].len() - 1) as f32));
    }
    let bottom = tops[plan.lanes.len()];
    let y = |index: usize| tops[lanes[index]] + 29.5 + row[index] as f32 * LANE * ROW;

    let mut page = Page::new(plan, schedule, today, "Swimlane roadmap", bottom + 97.0);
    page.months(&axis, 130.0, 114.0, bottom, 14.0);
    page.weeks(&axis, 146.0, 11.0);
    page.line(X0, 152.0, X1, 152.0, MID, 1.2);

    page.text(X0, PHASES_Y + 13.0, "Phases", 12.0, GRAY, false, Anchor::Start, None);
    for (index, ((name, from, to), phase)) in schedule.phases(plan).into_iter().zip(&plan.phases).enumerate() {
        let (x0, x1) = (axis.x(from), axis.xe(to));
        let tip = page.chevron(x0, x1, PHASES_Y, 18.0, SHADES[index % SHADES.len()], 7.0);
        let (lines, size) = phase_label(&name, phase.short.as_deref(), x1 - x0 - tip - 4.0, 18.0, 12.0, 10.0, |text, size| sans_width(text, size, false));
        let pitch = size * 1.2;
        let first = PHASES_Y + 9.0 + size * 0.36 - pitch * (lines.len() - 1) as f32 / 2.0;
        for (line, text) in lines.iter().enumerate() {
            page.text((x0 + x1) / 2.0 + tip / 2.0, first + pitch * line as f32, text, size, INK, false, Anchor::Middle, None);
        }
    }

    for (index, lines) in lane_lines.iter().enumerate() {
        page.line(X0, tops[index], X1, tops[index], LIGHT, 1.2);
        for (line, text) in lines.iter().enumerate() {
            page.text(X0, tops[index] + 21.0 + 17.0 * line as f32, text, 14.0, INK, false, Anchor::Start, None);
        }
        page.text(X0, tops[index] + 38.0 + 17.0 * (lines.len() - 1) as f32, &fit(&place(index), 12.0, false, label_w), 12.0, GRAY, false, Anchor::Start, None);
    }
    if let Some(target) = schedule.target {
        page.vline(axis.milestone(plan, schedule, target), PHASES_Y + 22.0, bottom, FIXED, 1.0, Some((4.0, 4.0)), Some(target));
    }
    let xt = axis.x(today);
    page.vline(xt, 152.0, bottom, GRAY, 1.2, None, None);
    page.text(xt - 5.0, 130.0, "today", 12.0, GRAY, true, Anchor::End, None);

    let ink = |index: usize| {
        if schedule.slots[index].spare < 0 {
            LATE
        } else if plan.items[index].on.is_some() {
            FIXED
        } else if chain.contains(&index) {
            INK
        } else {
            GRAY
        }
    };
    for (index, item) in plan.items.iter().enumerate() {
        let (x0, x1) = ends(index);
        let late = schedule.slots[index].spare < 0;
        if !item.milestone {
            page.bar(x0 + 1.0, x1 - 1.5, y(index), 9.0, if late { LATE } else if chain.contains(&index) { INK } else { SOFT }, 2.0, Some(index));
        } else if item.on.is_some() {
            page.milestone(x0, y(index), 7.0, late, false, true, Some(index));
        } else {
            page.milestone(x0, y(index), 6.0, late, !chain.contains(&index) && !late, false, Some(index));
        }
    }

    for check in checks {
        let Check::NoBuffer { before, target } = *check else { continue };
        let xa = if plan.items[before].milestone { ends(before).0 } else { ends(before).1 };
        let (xl, yl) = (ends(target).0, y(target));
        page.scene.add(Shape::Path { points: vec![(xa, yl + 8.0), (xa, yl + 12.0), (xl, yl + 12.0), (xl, yl + 8.0)], closed: false, fill: Fill::None, stroke: Some(Stroke { color: WARN_LINE, width: 1.6, dash: None }) });
        page.shape([xa, yl + 8.0, xl, yl + 12.0], None);
        page.tag(xl + 12.0, yl + 15.0, "no buffer", Anchor::Start);
    }

    let mut order: Vec<usize> = (0..plan.items.len()).collect();
    order.sort_by(|&a, &b| (plan.items[a].on.is_none(), ends(a).0).partial_cmp(&(plan.items[b].on.is_none(), ends(b).0)).unwrap_or(std::cmp::Ordering::Equal));
    for index in order {
        let item = &plan.items[index];
        let (x0, x1) = ends(index);
        let yi = y(index);
        let bounds = [gx0, tops[lanes[index]] + 2.0, X1, tops[lanes[index] + 1] - 1.0];
        if item.milestone && item.on.is_some() {
            let text = format!("{}  {}", item.name, schedule.slots[index].planned.finish.strftime("%d %b"));
            page.place(&[(x0 + 6.0, yi - 12.0, Anchor::Start), (x0 - 6.0, yi - 12.0, Anchor::End), (x0 + 11.0, yi + 4.5, Anchor::Start)], &text, 12.5, ink(index), true, Some(index), bounds);
        } else if item.milestone {
            page.place(&[(x0 + 11.0, yi + 4.5, Anchor::Start), (x0 - 6.0, yi - 12.0, Anchor::End), (x0 + 6.0, yi - 12.0, Anchor::Start), (x0 - 11.0, yi + 4.5, Anchor::End)], &item.name, 12.5, ink(index), false, Some(index), bounds);
        } else {
            page.place(&[(x0, yi - 10.0, Anchor::Start), (x0, yi + 16.5, Anchor::Start), (x1, yi - 10.0, Anchor::End), (x1, yi + 16.5, Anchor::End)], &item.name, 12.5, ink(index), false, Some(index), bounds);
        }
    }

    let legend_y = page.height() - 76.0;
    let keys = [(Key::Chain, "decides the date"), (Key::Soft, "time to spare"), (Key::Milestone, "milestone"), (Key::Fixed, "fixed date"), (Key::Today, "today"), (Key::Warning, "warning")];
    page.legend(legend_y, &keys[..if audience.warnings() { keys.len() } else { keys.len() - 1 }]);
    page.finish()
}
