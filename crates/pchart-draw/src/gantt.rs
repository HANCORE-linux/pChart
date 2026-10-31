use std::collections::HashSet;

use jiff::civil::Date;
use pchart_core::analysis::Check;
use pchart_core::{Plan, Schedule};

use crate::page::{Axis, FIXED, GRAY, INK, Key, LATE, LIGHT, LINK, MID, Page, SHADES, SOFT, X0, X1, dates, fit, wrap};
use crate::scene::{Anchor, Scene};
use crate::style::sans_width;
use crate::views::Audience;

/// A phase with its dates and its items, in the order they start.
pub struct Phase {
    pub name: String,
    pub from: Date,
    pub to: Date,
    pub items: Vec<usize>,
}

/// The items of each phase: everything its closing item waits on, directly or through others, that no
/// earlier phase took; and the items no phase takes. Both in the order they start.
pub fn phase_items(plan: &Plan, schedule: &Schedule) -> (Vec<Phase>, Vec<usize>) {
    let by_start = |list: &mut Vec<usize>| list.sort_by_key(|&index| (schedule.slots[index].planned.start, schedule.slots[index].planned.finish));
    let mut taken = HashSet::new();
    let mut out = Vec::new();
    let phases = schedule.phases(plan);
    for (phase, (name, from, to)) in plan.phases.iter().zip(phases) {
        let Some(until) = plan.item(&phase.until) else { continue };
        let mut stack = vec![until];
        let mut mine = Vec::new();
        while let Some(index) = stack.pop() {
            if taken.insert(index) {
                mine.push(index);
                stack.extend(plan.items[index].after.iter().filter_map(|id| plan.item(id)));
            }
        }
        by_start(&mut mine);
        out.push(Phase { name, from, to, items: mine });
    }
    let mut rest: Vec<usize> = (0..plan.items.len()).filter(|index| !taken.contains(index)).collect();
    by_start(&mut rest);
    (out, rest)
}

enum Row {
    Phase(usize),
    Item(usize),
}

/// The Gantt chart for export: one row per item under its phase, with lane, planned dates and what it
/// has to spare in a table, bars on a time line, and who waits on whom.
pub fn gantt(plan: &Plan, schedule: &Schedule, today: Date, checks: &[Check], audience: Audience) -> Scene {
    let checks = if audience.warnings() { checks } else { &[] };
    let chain: HashSet<usize> = schedule.chain.iter().copied().collect();
    let (phases, rest) = phase_items(plan, schedule);
    let lane_name = |index: usize| plan.lane(&plan.items[index].lane).map_or(plan.items[index].lane.as_str(), |lane| plan.lanes[lane].name.as_str());
    let name_w = plan.items.iter().map(|item| sans_width(&item.name, 13.0, item.on.is_some())).fold(0.0, f32::max).min(190.0) + 12.0;
    let lane_x = X0 + name_w + 16.0;
    let lane_w = (0..plan.items.len()).map(|index| sans_width(lane_name(index), 12.0, false)).fold(0.0, f32::max).min(110.0);
    let lines = |index: usize| (wrap(&plan.items[index].name, 13.0, plan.items[index].on.is_some(), name_w - 4.0), wrap(lane_name(index), 12.0, false, lane_w));
    let tall = |index: usize| {
        let (name, lane) = lines(index);
        20.0 + 15.0 * (name.len().max(lane.len()) - 1) as f32
    };
    let mut rows: Vec<(Row, f32, f32)> = Vec::new();
    let mut y = 169.0;
    for (index, phase) in phases.iter().enumerate() {
        rows.push((Row::Phase(index), y, 19.0));
        y += 19.0;
        for &item in &phase.items {
            rows.push((Row::Item(item), y, tall(item)));
            y += tall(item);
        }
    }
    for &item in &rest {
        rows.push((Row::Item(item), y + 4.0, tall(item)));
        y += 4.0 + tall(item);
    }
    let bottom = y;
    let mut page = Page::new(plan, schedule, today, "Gantt chart", bottom + 96.0);

    let date_x = lane_x + lane_w + 16.0;
    let date_w = (0..plan.items.len()).map(|index| sans_width(&dates(plan, schedule, index, false), 12.0, false)).fold(0.0, f32::max);
    let spare_x = date_x + date_w + 16.0;
    let gx0 = if audience.spare() { spare_x + sans_width("To spare", 11.0, false).max(sans_width("19 wd", 12.0, false)) + 20.0 } else { spare_x + 4.0 };
    let axis = Axis::new(schedule, today, 13, gx0, X1);
    let row_y = |index: usize| rows.iter().find_map(|(row, y, h)| matches!(row, Row::Item(item) if *item == index).then_some(y + h / 2.0)).unwrap_or(0.0);
    let ends = |index: usize| {
        if plan.items[index].milestone {
            let x = axis.milestone(plan, schedule, index);
            (x, x)
        } else {
            let span = schedule.slots[index].planned;
            (axis.x(span.start), axis.xe(span.finish))
        }
    };

    page.months(&axis, 138.0, 122.0, bottom, 14.0);
    page.weeks(&axis, 156.0, 11.0);
    page.line(X0, 163.0, X1, 163.0, MID, 1.2);
    page.text(lane_x, 156.0, "Lane", 11.0, GRAY, false, Anchor::Start, None);
    page.text(date_x, 156.0, "Planned", 11.0, GRAY, false, Anchor::Start, None);
    if audience.spare() {
        page.text(spare_x, 156.0, "To spare", 11.0, GRAY, false, Anchor::Start, None);
    }
    let xt = axis.x(today);
    page.vline(xt, 163.0, bottom, GRAY, 1.2, None, None);
    page.text(xt - 5.0, 138.0, "today", 12.0, GRAY, true, Anchor::End, None);

    for (index, item) in plan.items.iter().enumerate() {
        for pred in item.after.iter().filter_map(|id| plan.item(id)) {
            let (ya, yb) = (row_y(pred), row_y(index));
            let x2 = ends(index).0 - if item.milestone { if item.on.is_some() { 7.0 } else { 6.0 } } else { 0.0 };
            let pred_end = ends(pred).1;
            let (mut xv, y0) = if plan.items[pred].milestone {
                let xv = pred_end.min(x2) - 4.0;
                (xv, ya + 6.0 - (pred_end - xv))
            } else {
                (pred_end.min(x2) - 6.0, ya + 5.0)
            };
            if x2 - xv < 5.0 {
                xv = x2 - 5.0;
            }
            page.link(xv, y0, x2, yb, if chain.contains(&pred) && chain.contains(&index) { LINK } else { MID });
        }
    }

    for (row, ry, h) in &rows {
        match row {
            Row::Phase(phase) => {
                let Phase { name, from, to, .. } = &phases[*phase];
                if *ry > rows[0].1 {
                    page.line(X0, *ry, X1, *ry, LIGHT, 1.2);
                }
                page.text(X0, ry + 14.0, &fit(name, 13.0, true, gx0 - X0 - 12.0), 13.0, INK, true, Anchor::Start, None);
                page.chevron(axis.x(*from), axis.xe(*to), ry + 5.0, 10.0, SHADES[phase % SHADES.len()], 5.0);
            }
            Row::Item(index) => {
                let index = *index;
                let item = &plan.items[index];
                let slot = &schedule.slots[index];
                let yc = ry + h / 2.0;
                let (on_chain, fixed, late) = (chain.contains(&index), item.on.is_some(), slot.spare < 0);
                let ink = if late {
                    LATE
                } else if fixed {
                    FIXED
                } else if on_chain {
                    INK
                } else {
                    GRAY
                };
                let (name, lane) = lines(index);
                for (line, text) in name.iter().enumerate() {
                    page.text(X0 + 12.0, yc + 4.5 - 7.5 * (name.len() - 1) as f32 + 15.0 * line as f32, text, 13.0, ink, fixed, Anchor::Start, Some(index));
                }
                for (line, text) in lane.iter().enumerate() {
                    page.text(lane_x, yc + 4.5 - 7.5 * (lane.len() - 1) as f32 + 15.0 * line as f32, text, 12.0, GRAY, false, Anchor::Start, Some(index));
                }
                page.text(date_x, yc + 4.5, &dates(plan, schedule, index, false), 12.0, if fixed || late { ink } else { GRAY }, false, Anchor::Start, Some(index));
                if !on_chain && audience.spare() {
                    page.text(spare_x, yc + 4.5, &format!("{} wd", slot.spare), 12.0, if late { LATE } else { GRAY }, false, Anchor::Start, Some(index));
                }
                let (x0, x1) = ends(index);
                if !item.milestone {
                    page.bar(x0 + 0.5, x1 - 0.5, yc, 10.0, if late { LATE } else if on_chain { INK } else { SOFT }, 2.0, Some(index));
                } else if fixed {
                    page.milestone(x0, yc, 7.0, late, false, true, Some(index));
                } else {
                    page.milestone(x0, yc, 6.0, late, !on_chain && !late, false, Some(index));
                }
            }
        }
    }

    for check in checks {
        let Check::NoBuffer { before, target } = *check else { continue };
        let xv = ends(before).1.min(ends(target).0 - 7.0) - 4.0;
        page.tag(xv - 8.0, row_y(target) + 4.5, "no buffer", Anchor::End);
    }

    let chain_label = match schedule.spare().filter(|_| audience.spare()) {
        Some(spare) => format!("decides the date · {spare} wd to spare"),
        None => "decides the date".to_string(),
    };
    let legend_y = page.height() - 76.0;
    page.legend(legend_y, &[(Key::Chain, &chain_label), (Key::Soft, "time to spare"), (Key::Link, "waits on"), (Key::Milestone, "milestone"), (Key::Fixed, "fixed date")]);
    page.finish()
}
