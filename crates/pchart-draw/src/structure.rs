//! How a project is built, without dates (pictures 57 and 58): the project structure (work breakdown,
//! DIN 69901), the network diagram (DIN 69900) and the dependency matrix (design structure matrix).

use jiff::civil::Date;
use pchart_core::{Plan, Schedule};

use crate::page::{CARD, GRAY, INK, Key, LATE, LINK, MID, PAPER, Page, SHADES, SOFT, X0, X1, fit};
use crate::scene::{Anchor, Fill, Scene, Shape, Stroke};
use crate::style::sans_width;

/// What an item is in a box: its lane and, for work, its length.
fn about(plan: &Plan, index: usize) -> String {
    let item = &plan.items[index];
    let lane = plan.lane(&item.lane).map_or("", |lane| plan.lanes[lane].name.as_str());
    if item.milestone || item.duration == 0 { lane.to_string() } else { format!("{lane} · {} wd", item.duration) }
}

/// The groups of the first level: the phases, each with the items that end in it, the items after the
/// last phase with it; without phases the lanes.
fn groups(plan: &Plan, schedule: &Schedule) -> Vec<(String, Vec<usize>)> {
    let mut order: Vec<usize> = (0..plan.items.len()).collect();
    order.sort_by_key(|&index| (schedule.slots[index].planned.start, schedule.slots[index].planned.finish));
    let phases = schedule.phases(plan);
    if phases.is_empty() {
        return plan.lanes.iter().map(|lane| (lane.name.clone(), order.iter().copied().filter(|&index| plan.items[index].lane == lane.id).collect())).collect();
    }
    let mut out: Vec<(String, Vec<usize>)> = phases.iter().map(|(name, _, _)| (name.clone(), Vec::new())).collect();
    for index in order {
        let finish = schedule.slots[index].planned.finish;
        let at = phases.iter().position(|(_, _, to)| finish <= *to).unwrap_or(phases.len() - 1);
        out[at].1.push(index);
    }
    out
}

/// The project structure: the phases as the first level, numbered, and under each the work and the
/// milestones that end in it, with their lane and length. A plan without phases is broken down by lane.
pub fn structure(plan: &Plan, schedule: &Schedule, today: Date) -> Scene {
    let groups = groups(plan, schedule);
    let (top, head, step, gap) = (178.0, 46.0, 56.0, 16.0);
    let most = groups.iter().map(|(_, items)| items.len()).max().unwrap_or(0) as f32;
    let mut page = Page::plain(plan, today, "Project structure", top + head + 20.0 + most * step + 100.0);
    let count = groups.len().max(1) as f32;
    let w = (X1 - X0 - gap * (count - 1.0)) / count;
    let x = |at: usize| X0 + at as f32 * (w + gap);
    if groups.len() > 1 {
        page.line(x(0) + w / 2.0, top - 18.0, x(groups.len() - 1) + w / 2.0, top - 18.0, MID, 1.5);
    }
    for (at, (name, items)) in groups.iter().enumerate() {
        let x0 = x(at);
        page.line(x0 + w / 2.0, top - 18.0, x0 + w / 2.0, top, MID, 1.5);
        page.scene.add(Shape::Rect { x: x0, y: top, w, h: head, radius: 4.0, fill: Fill::Color(SHADES[at % SHADES.len()]), stroke: None });
        page.shape([x0, top, x0 + w, top + head], None);
        let code = format!("{}", at + 1);
        let end = page.text(x0 + 14.0, top + 29.0, &code, 15.0, GRAY, true, Anchor::Start, None);
        page.text(end + 10.0, top + 29.0, &fit(name, 15.0, true, x0 + w - 12.0 - end - 10.0), 15.0, INK, true, Anchor::Start, None);
        let spine = x0 + 16.0;
        if let Some(last) = items.len().checked_sub(1) {
            page.line(spine, top + head, spine, top + head + 20.0 + last as f32 * step + 22.0, MID, 1.5);
        }
        for (row, &index) in items.iter().enumerate() {
            let item = &plan.items[index];
            let y = top + head + 20.0 + row as f32 * step;
            page.line(spine, y + 22.0, spine + 14.0, y + 22.0, MID, 1.5);
            let bx = spine + 14.0;
            let code = format!("{}.{}", at + 1, row + 1);
            if item.milestone {
                page.milestone(bx + 12.0, y + 22.0, 8.0, schedule.slots[index].spare < 0, false, item.on.is_some(), item.important, Some(index));
                let room = x0 + w - bx - 30.0;
                page.text(bx + 28.0, y + 19.0, &fit(&item.name, 14.0, true, room), 14.0, INK, true, Anchor::Start, Some(index));
                page.text(bx + 28.0, y + 37.0, &fit(&format!("{code} · {}", about(plan, index)), 12.0, false, room), 12.0, GRAY, false, Anchor::Start, Some(index));
                continue;
            }
            page.scene.add(Shape::Rect { x: bx, y, w: x0 + w - bx, h: 44.0, radius: 4.0, fill: Fill::Color(CARD), stroke: None });
            page.shape([bx, y, x0 + w, y + 44.0], Some(index));
            let room = x0 + w - bx - 20.0;
            page.text(bx + 10.0, y + 19.0, &fit(&item.name, 14.0, true, room), 14.0, INK, true, Anchor::Start, Some(index));
            page.text(bx + 10.0, y + 37.0, &fit(&format!("{code} · {}", about(plan, index)), 12.0, false, room), 12.0, GRAY, false, Anchor::Start, Some(index));
        }
    }
    page.finish()
}

/// The network diagram: every item as a box, below what it waits on, in the column of its lane, with
/// an arrow for each item it waits on; the chain that decides the date in ink.
pub fn network(plan: &Plan, schedule: &Schedule, today: Date) -> Scene {
    let mut rank = vec![0usize; plan.items.len()];
    for &index in &schedule.order {
        rank[index] = plan.items[index].after.iter().filter_map(|id| plan.item(id)).map(|pred| rank[pred] + 1).max().unwrap_or(0);
    }
    let lanes: Vec<usize> = plan.items.iter().map(|item| plan.lane(&item.lane).unwrap_or(0)).collect();
    let ranks = rank.iter().max().map_or(0, |&most| most + 1);
    let (top, row, boxh) = (200.0, 78.0, 42.0);
    let mut page = Page::plain(plan, today, "Network diagram", top + ranks as f32 * row + 120.0);
    let columns = plan.lanes.len().max(1);
    let cw = (X1 - X0) / columns as f32;
    for (lane, it) in plan.lanes.iter().enumerate() {
        let xc = X0 + cw * (lane as f32 + 0.5);
        page.text(xc, 156.0, &fit(&it.name, 14.0, true, cw - 8.0), 14.0, INK, true, Anchor::Middle, None);
        if let Some(city) = &it.city {
            page.text(xc, 174.0, &fit(city, 12.0, false, cw - 8.0), 12.0, GRAY, false, Anchor::Middle, None);
        }
    }
    let mut boxes = vec![[0.0f32; 4]; plan.items.len()];
    for lane in 0..columns {
        for r in 0..ranks {
            let here: Vec<usize> = (0..plan.items.len()).filter(|&index| lanes[index] == lane && rank[index] == r).collect();
            let share = (cw - 8.0) / here.len().max(1) as f32;
            for (at, &index) in here.iter().enumerate() {
                let x0 = X0 + cw * lane as f32 + 4.0 + share * at as f32 + 4.0;
                let y0 = top + r as f32 * row;
                boxes[index] = [x0, y0, x0 + share - 8.0, y0 + boxh];
            }
        }
    }
    let chain = |index: usize| schedule.chain.contains(&index);
    let mut gutters = vec![0usize; columns];
    for (index, item) in plan.items.iter().enumerate() {
        for pred in item.after.iter().filter_map(|id| plan.item(id)) {
            let (a, b) = (boxes[pred], boxes[index]);
            let (px, sx) = ((a[0] + a[2]) / 2.0, (b[0] + b[2]) / 2.0);
            let (color, width) = if chain(pred) && chain(index) { (INK, 1.6) } else { (LINK, 1.2) };
            let mut points = vec![(px, a[3])];
            let across = if rank[index] == rank[pred] + 1 {
                let across = b[1] - if item.milestone { 24.0 } else { 14.0 };
                points.push((px, across));
                across
            } else {
                let lane = lanes[pred];
                gutters[lane] += 1;
                let gx = X0 + cw * (lane as f32 + 1.0) - 2.0 - 3.0 * (gutters[lane] % 3) as f32;
                points.extend([(px, a[3] + 6.0), (gx, a[3] + 6.0), (gx, b[1] - 24.0)]);
                b[1] - 24.0
            };
            let head = if item.milestone {
                let (cy, tip) = (b[1] + 10.0, sx - 11.0);
                points.extend([(tip - 12.0, across), (tip - 12.0, cy), (tip - 5.0, cy)]);
                vec![(tip, cy), (tip - 6.0, cy - 3.5), (tip - 6.0, cy + 3.5)]
            } else {
                points.extend([(sx, across), (sx, b[1] - 5.0)]);
                vec![(sx, b[1]), (sx - 3.5, b[1] - 6.0), (sx + 3.5, b[1] - 6.0)]
            };
            page.scene.add(Shape::Path { points, closed: false, fill: Fill::None, stroke: Some(Stroke { color, width, dash: None }) });
            page.scene.add(Shape::Path { points: head, closed: true, fill: Fill::Color(color), stroke: None });
        }
    }
    for (index, item) in plan.items.iter().enumerate() {
        let [x0, y0, x1, y1] = boxes[index];
        let late = schedule.slots[index].spare < 0;
        let room = x1 - x0 - 10.0;
        if item.milestone {
            let xc = (x0 + x1) / 2.0;
            page.milestone(xc, y0 + 10.0, 8.0, late, false, item.on.is_some(), item.important, Some(index));
            page.text(xc, y0 + 36.0, &fit(&item.name, 12.5, true, room + 8.0), 12.5, if late { LATE } else { INK }, true, Anchor::Middle, Some(index));
            continue;
        }
        let (fill, stroke) = if late {
            (PAPER, Stroke { color: LATE, width: 1.6, dash: None })
        } else if chain(index) {
            (PAPER, Stroke { color: INK, width: 1.6, dash: None })
        } else {
            (CARD, Stroke { color: SOFT, width: 1.0, dash: None })
        };
        page.scene.add(Shape::Rect { x: x0, y: y0, w: x1 - x0, h: y1 - y0, radius: 4.0, fill: Fill::Color(fill), stroke: Some(stroke) });
        page.shape([x0, y0, x1, y1], Some(index));
        let xc = (x0 + x1) / 2.0;
        if item.duration == 0 {
            page.text(xc, y0 + 26.0, &fit(&item.name, 12.5, true, room), 12.5, INK, true, Anchor::Middle, Some(index));
            continue;
        }
        page.text(xc, y0 + 18.0, &fit(&item.name, 12.5, true, room), 12.5, INK, true, Anchor::Middle, Some(index));
        page.text(xc, y0 + 34.0, &fit(&format!("{} wd", item.duration), 11.5, false, room), 11.5, GRAY, false, Anchor::Middle, Some(index));
    }
    let y = page.height() - 92.0;
    if schedule.chain.is_empty() {
        page.legend(y, &[(Key::Link, "waits on"), (Key::Milestone, "milestone")]);
    } else {
        page.legend(y, &[(Key::Chain, "decides the date"), (Key::Soft, "has time to spare"), (Key::Link, "waits on"), (Key::Milestone, "milestone"), (Key::Fixed, "fixed date")]);
    }
    page.finish()
}

/// The dependency matrix (design structure matrix): the items as rows and as columns in the same order,
/// phase by phase, and a dot where the row waits on the column, in ink on the chain that decides the
/// date. A plan has no circles, so every dot lies below the diagonal; one far from it waits across phases.
pub fn matrix(plan: &Plan, schedule: &Schedule, today: Date) -> Scene {
    let groups = groups(plan, schedule);
    let order: Vec<usize> = groups.iter().flat_map(|(_, items)| items.iter().copied()).collect();
    let count = order.len().max(1);
    let label = order.iter().map(|&index| sans_width(&plan.items[index].name, 13.0, false)).fold(0.0, f32::max).min(260.0) + 40.0;
    let gx = X0 + label;
    let cell = ((X1 - gx) / count as f32).clamp(10.0, 32.0);
    let (top, head) = (176.0, 26.0);
    let height = top + 26.0 + count as f32 * cell + groups.len() as f32 * head + 130.0;
    let mut page = Page::plain(plan, today, "Dependency matrix", height);
    let at: Vec<usize> = {
        let mut at = vec![0; plan.items.len()];
        for (row, &index) in order.iter().enumerate() {
            at[index] = row;
        }
        at
    };
    let mut ys = vec![0.0f32; count];
    let mut y = top + 26.0;
    let mut row = 0;
    for (number, (name, items)) in groups.iter().enumerate() {
        page.text(X0, y + 18.0, &format!("{} {}", number + 1, fit(name, 13.0, true, label + count as f32 * cell)), 13.0, GRAY, true, Anchor::Start, None);
        y += head;
        let block = items.len() as f32 * cell;
        page.scene.add(Shape::Rect { x: gx, y, w: count as f32 * cell, h: block, radius: 0.0, fill: Fill::Color(CARD), stroke: None });
        page.scene.add(Shape::Rect { x: gx + row as f32 * cell, y, w: block, h: block, radius: 0.0, fill: Fill::Color(SHADES[number % SHADES.len()]), stroke: None });
        for _ in items {
            ys[row] = y;
            row += 1;
            y += cell;
        }
    }
    let x = |column: usize| gx + column as f32 * cell;
    let every = if cell >= 18.0 { 1 } else { 5 };
    for (column, &index) in order.iter().enumerate() {
        if (column + 1) % every == 0 || every == 1 {
            page.text(x(column) + cell / 2.0, top + 14.0, &format!("{}", column + 1), 11.0, GRAY, false, Anchor::Middle, None);
        }
        let row_y = ys[column];
        page.text(gx - 30.0, row_y + cell / 2.0 + 4.0, &fit(&plan.items[index].name, 13.0, false, label - 34.0), 13.0, INK, false, Anchor::End, Some(index));
        page.text(gx - 6.0, row_y + cell / 2.0 + 4.0, &format!("{}", column + 1), 11.0, GRAY, false, Anchor::End, None);
        page.scene.add(Shape::Rect { x: x(column), y: row_y, w: cell, h: cell, radius: 0.0, fill: Fill::Color(MID), stroke: None });
    }
    for (row, &index) in order.iter().enumerate() {
        page.line(gx, ys[row], gx + count as f32 * cell, ys[row], PAPER, 1.0);
        for pred in plan.items[index].after.iter().filter_map(|id| plan.item(id)) {
            let on = schedule.chain.contains(&index) && schedule.chain.contains(&pred);
            page.scene.add(Shape::Circle { x: x(at[pred]) + cell / 2.0, y: ys[row] + cell / 2.0, r: cell * 0.26, fill: if on { INK } else { LINK } });
        }
    }
    for column in 0..=count {
        page.line(x(column), top + 26.0 + head, x(column), y, PAPER, 1.0);
    }
    let ly = page.height() - 92.0;
    if schedule.chain.is_empty() {
        page.scene.add(Shape::Circle { x: X0 + 6.0, y: ly - 4.0, r: 5.0, fill: LINK });
        page.text(X0 + 18.0, ly, "the row waits on the column", 13.0, GRAY, false, Anchor::Start, None);
        return page.finish();
    }
    page.scene.add(Shape::Circle { x: X0 + 6.0, y: ly - 4.0, r: 5.0, fill: INK });
    let end = page.text(X0 + 18.0, ly, "the row waits on the column, on the chain that decides the date", 13.0, GRAY, false, Anchor::Start, None);
    page.scene.add(Shape::Circle { x: end + 32.0, y: ly - 4.0, r: 5.0, fill: LINK });
    page.text(end + 44.0, ly, "waits on, with time to spare", 13.0, GRAY, false, Anchor::Start, None);
    page.finish()
}
