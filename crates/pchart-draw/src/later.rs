use std::collections::HashSet;

use jiff::civil::{Date, Weekday};
use pchart_core::calendar::{next, previous};
use pchart_core::{Plan, Schedule};

use crate::page::{CARD, FIXED, GRAY, INK, Key, LATE, MID, Page, X0, X1, dates, fit};
use crate::scene::{Anchor, Fill, Scene, Shape};
use crate::style::sans_width;

const GAP: f32 = 28.0;
const CARD_H: f32 = 46.0;
const PITCH: f32 = 52.0;

/// Now / Next / Later for export, without a time line: Now is what runs or starts in this week and the
/// five after it, Next the six weeks after that, Later the rest. Items that are done are left out.
pub fn now_next_later(plan: &Plan, schedule: &Schedule, today: Date) -> Scene {
    let chain: HashSet<usize> = schedule.chain.iter().copied().collect();
    let mut monday = today;
    while monday.weekday() != Weekday::Monday {
        monday = previous(monday);
    }
    let weeks = |from: Date, count: usize| (0..count * 7).fold(from, |day, _| next(day));
    let now_end = previous(weeks(monday, 6));
    let next_end = previous(weeks(next(now_end), 6));
    let start = |index: usize| {
        let slot = &schedule.slots[index].planned;
        if plan.items[index].milestone { slot.finish } else { slot.start }
    };
    let mut items: Vec<usize> = (0..plan.items.len()).filter(|&index| !plan.items[index].done).collect();
    items.sort_by_key(|&index| (start(index), schedule.slots[index].planned.finish));
    let day = |date: Date| date.strftime("%a %d %b").to_string();
    let columns: [(&str, String, Vec<usize>); 3] = [
        ("Now", format!("running, or starts by {}", day(now_end)), items.iter().copied().filter(|&index| start(index) <= now_end).collect()),
        ("Next", format!("starts {} – {}", day(next(now_end)), day(next_end)), items.iter().copied().filter(|&index| start(index) > now_end && start(index) <= next_end).collect()),
        ("Later", format!("starts from {}", day(next(next_end))), items.iter().copied().filter(|&index| start(index) > next_end).collect()),
    ];
    let longest = columns.iter().map(|(_, _, cards)| cards.len()).max().unwrap_or(0) as f32;
    let mut page = Page::new(plan, schedule, today, "Now / Next / Later", 198.0 + longest * PITCH + 96.0);
    let width = (X1 - X0 - 2.0 * GAP) / 3.0;
    for (column, (name, rule, cards)) in columns.iter().enumerate() {
        let x = X0 + column as f32 * (width + GAP);
        page.text(x, 158.0, name, 24.0, INK, true, Anchor::Start, None);
        page.text(x, 182.0, &fit(rule, 13.5, false, width), 13.5, GRAY, false, Anchor::Start, None);
        let mut y = 198.0;
        for &index in cards {
            let item = &plan.items[index];
            let fixed = item.on.is_some();
            let late = schedule.slots[index].spare < 0;
            let ink = if late {
                LATE
            } else if fixed {
                FIXED
            } else {
                INK
            };
            page.scene.add(Shape::Rect { x, y, w: width, h: CARD_H, radius: 3.0, fill: Fill::Color(CARD), stroke: None });
            page.scene.add(Shape::Rect { x, y, w: 4.0, h: CARD_H, radius: 0.0, fill: Fill::Color(if chain.contains(&index) { INK } else { MID }), stroke: None });
            let mut tx = x + 18.0;
            if item.milestone {
                page.diamond(tx + 5.0, y + 17.0, if fixed { 6.0 } else { 5.0 }, if fixed || late { ink } else { INK }, None, Some(index));
                tx += 18.0;
            }
            let room = x + width - tx - 12.0;
            page.text(tx, y + 22.0, &fit(&item.name, 15.0, true, room), 15.0, ink, true, Anchor::Start, Some(index));
            let lane = plan.lane(&item.lane).map_or(item.lane.as_str(), |lane| plan.lanes[lane].name.as_str());
            let when = if item.milestone && fixed { format!("{} {} · fixed", day(schedule.slots[index].planned.finish), schedule.slots[index].planned.finish.year()) } else { dates(plan, schedule, index, true) };
            let lane = fit(lane, 13.0, false, (room - sans_width(&format!(" · {when}"), 13.0, false)).max(0.0));
            page.text(tx, y + 39.0, &fit(&format!("{lane} · {when}"), 13.0, false, room), 13.0, if fixed || late { ink } else { GRAY }, false, Anchor::Start, Some(index));
            y += PITCH;
        }
    }
    let legend_y = page.height() - 76.0;
    page.legend(legend_y, &[(Key::Edge, "decides the date"), (Key::Milestone, "milestone, in the column of its date")]);
    page.finish()
}
