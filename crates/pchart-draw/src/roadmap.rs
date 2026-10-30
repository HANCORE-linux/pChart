use std::collections::HashSet;

use jiff::civil::{Date, Weekday};
use pchart_core::calendar::{Calendar, next, previous};
use pchart_core::{Plan, Schedule};

use crate::scene::{Anchor, Color, Fill, Scene, Shape, Stroke};
use crate::style::{ADVANCE, FONT, Palette};

/// What a roadmap picture shows and at which size. `unit` is one pixel of a 21 px tall cell.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub width: f32,
    pub height: f32,
    pub unit: f32,
    pub today: Date,
    pub selected: Option<usize>,
    pub palette: Palette,
}

struct Grid {
    start: Date,
    x0: f32,
    x1: f32,
    px: f32,
    rows_y: f32,
    lane_h: f32,
    bar: f32,
}

impl Grid {
    fn x(&self, day: Date) -> f32 {
        self.x0 + days(self.start, day) as f32 * self.px
    }

    fn lane_y(&self, lane: usize) -> f32 {
        self.rows_y + lane as f32 * self.lane_h
    }
}

/// The swimlane roadmap at the Full detail level.
pub fn roadmap(plan: &Plan, schedule: &Schedule, calendars: &[Calendar], settings: &Settings) -> Scene {
    let (p, u, today) = (settings.palette, settings.unit, settings.today);
    let mut s = Scene { width: settings.width, height: settings.height, background: p.page, hatch: (p.raised, p.muted), hatch_alert: (p.alert_ground, p.alert), font: FONT, marks: Vec::new() };
    let lanes: Vec<usize> = plan.items.iter().map(|item| plan.lane(&item.lane).unwrap_or(0)).collect();
    let chain: HashSet<usize> = schedule.chain.iter().copied().collect();

    let first = schedule.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(today).min(today);
    let last = schedule.slots.iter().map(|slot| slot.planned.finish.max(slot.latest.finish)).max().unwrap_or(today);
    let mut start = first;
    while start.weekday() != Weekday::Monday {
        start = previous(start);
    }
    let end = (0..20).fold(last, |day, _| next(day));
    let top = 26.0 * u;
    let rows_y = top + 64.0 * u;
    let lane_h = ((settings.height - rows_y - 22.0 * u) / plan.lanes.len().max(1) as f32).max(24.0 * u);
    let x0 = 220.0 * u;
    let x1 = settings.width - 8.0 * u;
    let px = (x1 - x0) / (days(start, end) + 1) as f32;
    let g = Grid { start, x0, x1, px, rows_y, lane_h, bar: (40.0 * u).min(lane_h * 0.45) };
    let bottom = g.lane_y(plan.lanes.len());
    let rule = |color: Color, width: f32| Stroke { color, width, dash: None };
    let text = |x: f32, y: f32, text: String, size: f32, color: Color| Shape::Text { x, y, text, size: size * u, color, bold: false, anchor: Anchor::Start };

    let mut month = start.first_of_month();
    let mut labelled = false;
    while month <= end {
        let following = next(month.last_of_month());
        let xa = g.x(month.max(start));
        if month >= start {
            s.add(Shape::Line { x1: xa, y1: top, x2: xa, y2: bottom, stroke: rule(p.rule, 1.0) });
        }
        let label = if !labelled || month.month() == 1 { month.strftime("%b %Y").to_string() } else { month.strftime("%b").to_string() };
        if g.x(following.min(end)) - xa > (label.chars().count() as f32 * 14.5 * ADVANCE + 12.0) * u {
            s.add(text(xa + 6.0 * u, top + 14.0 * u, label, 14.5, p.ink));
            labelled = true;
        }
        month = following;
    }
    let mut monday = start;
    let mut first_week = true;
    while monday <= end {
        let week = format!("{}{:02}", if first_week { "W" } else { "" }, monday.iso_week_date().week());
        s.add(text(g.x(monday) + 2.0 * u, top + 33.0 * u, week, 12.5, p.muted));
        first_week = false;
        monday = (0..7).fold(monday, |day, _| next(day));
    }
    s.add(Shape::Line { x1: x0, y1: top + 40.0 * u, x2: x1, y2: top + 40.0 * u, stroke: rule(p.rule, 1.0) });

    for (name, from, to) in schedule.phases(plan) {
        chevron(&mut s, &p, u, g.x(from), g.x(to) + px, top + 44.0 * u, 18.0 * u, &name);
    }
    s.add(text(12.0 * u, top + 58.0 * u, "Phases".into(), 14.0, p.muted));

    for (index, lane) in plan.lanes.iter().enumerate() {
        let y = g.lane_y(index);
        s.add(Shape::Line { x1: 6.0 * u, y1: y, x2: x1, y2: y, stroke: rule(p.rule, 1.0) });
        s.add(text(12.0 * u, y + 31.0 * u, lane.name.clone(), 16.6, p.ink));
        if lane_h >= 60.0 * u {
            let place = [lane.city.as_deref(), lane.place.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
            s.add(text(12.0 * u, y + 52.0 * u, place, 14.0, p.muted));
        }
        for (day, _) in calendars[index].holidays(start, end) {
            if !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday) {
                s.add(Shape::Rect { x: g.x(day), y: y + 8.0 * u, w: px, h: lane_h - 16.0 * u, radius: 0.0, fill: Fill::Hatch, stroke: None });
            }
        }
    }

    if let Some(target) = schedule.target {
        let xt = g.x(schedule.slots[target].planned.finish) + px / 2.0;
        s.add(Shape::Line { x1: xt, y1: rows_y + 2.0 * u, x2: xt, y2: bottom, stroke: Stroke { color: p.bright, width: 1.0, dash: Some((4.0 * u, 4.0 * u)) } });
    }

    let ends = |index: usize| {
        let slot = &schedule.slots[index].planned;
        (g.x(slot.start), g.x(slot.finish) + px, g.lane_y(lanes[index]) + g.bar)
    };
    let point = |index: usize| milestone_x(plan, schedule, &g, index);
    for (index, item) in plan.items.iter().enumerate() {
        for pred in item.after.iter().filter_map(|id| plan.item(id)) {
            let (_, ax1, ay) = ends(pred);
            let (bx0, _, by) = ends(index);
            let from = if plan.items[pred].milestone { point(pred) + 7.0 * u } else { ax1 };
            let to = if item.milestone { point(index) - 7.0 * u } else { bx0 };
            let deciding = chain.contains(&pred) && chain.contains(&index);
            let (color, opacity) = if deciding { (p.dim, 0.9) } else { (p.muted, 0.7) };
            let stroke = Some(Stroke { color, width: 1.2 * u, dash: None });
            let points = if (ay - by).abs() < 1.0 {
                vec![(from, ay), (to - u, by)]
            } else {
                let xm = from + ((to - from) / 2.0).clamp(3.0 * u, 6.0 * u);
                vec![(from, ay), (xm, ay), (xm, by), (to - u, by)]
            };
            s.marks.push(crate::scene::Mark { shape: Shape::Path { points, closed: false, fill: Fill::None, stroke }, opacity, item: None });
            let head = vec![(to, by), (to - 5.0 * u, by - 3.0 * u), (to - 5.0 * u, by + 3.0 * u)];
            s.marks.push(crate::scene::Mark { shape: Shape::Path { points: head, closed: true, fill: Fill::Color(color), stroke: None }, opacity, item: None });
        }
    }

    for (index, item) in plan.items.iter().enumerate() {
        let slot = &schedule.slots[index];
        let (bx0, bx1, y) = ends(index);
        let late = slot.slack < 0;
        if item.milestone {
            let x = point(index);
            if item.on.is_some() {
                diamond(&mut s, index, x, y, 8.0 * u, Fill::Color(p.bright), None);
                s.add_for(index, 1.0, Shape::Line { x1: x, y1: y - 10.0 * u, x2: x, y2: y - 20.0 * u, stroke: rule(p.bright, 1.6 * u) });
                s.add_for(index, 1.0, Shape::Circle { x, y: y - 23.0 * u, r: 4.0 * u, fill: p.bright });
            } else if late {
                diamond(&mut s, index, x, y, 7.0 * u, Fill::Color(p.alert), None);
            } else if chain.contains(&index) {
                diamond(&mut s, index, x, y, 7.0 * u, Fill::Color(p.ink), None);
            } else {
                diamond(&mut s, index, x, y, 7.0 * u, Fill::Color(p.page), Some(rule(p.dim, 1.5 * u)));
            }
            continue;
        }
        let free = free_until(plan, schedule, index);
        if !chain.contains(&index) && free > slot.planned.finish {
            let xs = g.x(next(slot.planned.finish));
            let xe = g.x(free) + px;
            s.add_for(index, 0.7, Shape::Rect { x: xs, y: y - 5.0 * u, w: xe - xs, h: 10.0 * u, radius: 0.0, fill: Fill::Hatch, stroke: None });
        }
        let (color, opacity) = if late {
            (p.alert, 0.92)
        } else if chain.contains(&index) {
            (p.ink, 0.95)
        } else {
            (p.dim, 0.5)
        };
        s.add_for(index, opacity, Shape::Rect { x: bx0, y: y - 7.0 * u, w: bx1 - bx0, h: 14.0 * u, radius: 3.0 * u, fill: Fill::Color(color), stroke: None });
        if settings.selected == Some(index) {
            let k = 6.0 * u;
            for (cx, cy, dx, dy) in [(bx0 - 4.0 * u, y - 11.0 * u, 1.0, 1.0), (bx1 + 4.0 * u, y - 11.0 * u, -1.0, 1.0), (bx0 - 4.0 * u, y + 11.0 * u, 1.0, -1.0), (bx1 + 4.0 * u, y + 11.0 * u, -1.0, -1.0)] {
                s.add(Shape::Path { points: vec![(cx + dx * k, cy), (cx, cy), (cx, cy + dy * k)], closed: false, fill: Fill::None, stroke: Some(rule(p.bright, 1.6 * u)) });
            }
        }
    }

    labels(&mut s, plan, schedule, &g, &lanes, &chain, settings);

    let xt = g.x(today);
    s.add(Shape::Line { x1: xt, y1: top + 42.0 * u, x2: xt, y2: bottom, stroke: rule(p.ink, 1.5 * u) });
    s.add(Shape::Text { x: xt - 5.0 * u, y: top + 14.0 * u, text: "today".into(), size: 13.5 * u, color: p.bright, bold: true, anchor: Anchor::End });
    s
}

fn labels(s: &mut Scene, plan: &Plan, schedule: &Schedule, g: &Grid, lanes: &[usize], chain: &HashSet<usize>, settings: &Settings) {
    let (p, u) = (settings.palette, settings.unit);
    let size = 13.5 * u;
    let width = |text: &str| text.chars().count() as f32 * size * ADVANCE;
    let mut order: Vec<usize> = (0..plan.items.len()).collect();
    order.sort_by_key(|&index| schedule.slots[index].planned.start);
    let mut above_end = vec![f32::MIN; plan.lanes.len()];
    let mut right_end = vec![f32::MIN; plan.lanes.len()];
    for (position, &index) in order.iter().enumerate() {
        let item = &plan.items[index];
        let slot = &schedule.slots[index];
        let lane = lanes[index];
        let x0 = g.x(slot.planned.start);
        let y = g.lane_y(lane) + g.bar;
        let selected = settings.selected == Some(index);
        let color = if slot.slack < 0 {
            p.alert
        } else if selected || chain.contains(&index) {
            p.bright
        } else {
            p.dim
        };
        if item.milestone {
            let x = milestone_x(plan, schedule, g, index);
            let label = format!("{}  {}", item.name, slot.planned.finish.strftime("%d %b"));
            let next_in_lane = order[position + 1..].iter().find(|&&other| lanes[other] == lane).map(|&other| g.x(schedule.slots[other].planned.start));
            let room = next_in_lane.unwrap_or(g.x1).min(g.x1);
            if x + 12.0 * u + width(&label) <= room {
                s.add_for(index, 1.0, Shape::Text { x: x + 12.0 * u, y: y + 5.0 * u, text: label, size, color, bold: selected, anchor: Anchor::Start });
                right_end[lane] = x + 12.0 * u + width(&item.name);
            } else {
                s.add_for(index, 1.0, Shape::Text { x: x - 12.0 * u, y: y - 12.0 * u, text: label, size, color, bold: selected, anchor: Anchor::End });
            }
            continue;
        }
        let below = x0 < above_end[lane] + 8.0 * u || x0 < right_end[lane] + 8.0 * u;
        let label_y = if below { y + 24.0 * u } else { y - 13.0 * u };
        if !below {
            above_end[lane] = x0 + width(&item.name);
        }
        s.add_for(index, 1.0, Shape::Text { x: x0, y: label_y, text: item.name.clone(), size, color, bold: selected, anchor: Anchor::Start });
    }
}

/// Calculated milestones sit at the end of their day, fixed ones in its middle.
fn milestone_x(plan: &Plan, schedule: &Schedule, g: &Grid, index: usize) -> f32 {
    let x = g.x(schedule.slots[index].planned.finish);
    if plan.items[index].on.is_some() { x + g.px / 2.0 } else { x + g.px }
}

/// The last day an item can finish without moving a successor (its free slack), at most its latest finish.
fn free_until(plan: &Plan, schedule: &Schedule, index: usize) -> Date {
    let id = &plan.items[index].id;
    plan.items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.after.contains(id))
        .map(|(succ, item)| {
            let span = schedule.slots[succ].planned;
            if item.milestone && item.on.is_none() { span.finish } else { previous(span.start) }
        })
        .min()
        .unwrap_or(schedule.slots[index].latest.finish)
        .min(schedule.slots[index].latest.finish)
}

fn diamond(s: &mut Scene, item: usize, x: f32, y: f32, r: f32, fill: Fill, stroke: Option<Stroke>) {
    s.add_for(item, 1.0, Shape::Path { points: vec![(x, y - r), (x + r, y), (x, y + r), (x - r, y)], closed: true, fill, stroke });
}

#[allow(clippy::too_many_arguments)]
fn chevron(s: &mut Scene, p: &Palette, u: f32, x0: f32, x1: f32, y: f32, h: f32, label: &str) {
    let tip = (8.0 * u).min((x1 - x0) / 2.0);
    let points = vec![(x0, y), (x1 - tip, y), (x1, y + h / 2.0), (x1 - tip, y + h), (x0, y + h), (x0 + tip, y + h / 2.0)];
    s.add(Shape::Path { points, closed: true, fill: Fill::Color(p.raised), stroke: Some(Stroke { color: p.muted, width: u, dash: None }) });
    let room = x1 - x0 - tip - 8.0 * u;
    let mut size = 13.5 * u;
    let mut label: Vec<char> = label.chars().collect();
    if label.len() as f32 * size * ADVANCE > room {
        size = (room / (label.len() as f32 * ADVANCE)).max(10.5 * u);
    }
    while label.len() as f32 * size * ADVANCE > room && label.len() > 3 {
        label.truncate(label.len() - 2);
        label.push('…');
    }
    s.add(Shape::Text { x: (x0 + x1) / 2.0 + tip / 2.0, y: y + h / 2.0 + size * 0.36, text: label.into_iter().collect(), size, color: p.dim, bold: false, anchor: Anchor::Middle });
}

fn days(from: Date, to: Date) -> i32 {
    from.until(to).map(|span| span.get_days()).unwrap_or(0)
}
