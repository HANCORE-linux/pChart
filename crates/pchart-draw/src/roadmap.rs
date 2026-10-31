use std::collections::HashSet;

use jiff::civil::{Date, Weekday};
use pchart_core::analysis::Check;
use pchart_core::calendar::{Calendar, next, previous};
use pchart_core::{Plan, Schedule, analyse};

use crate::scene::{Anchor, Color, Fill, Mark, Scene, Shape, Stroke};
use crate::style::{ADVANCE, FONT, Palette};

/// How much a roadmap shows. Overview folds each lane into one bar; Full adds dependencies, holidays,
/// free slack, the date of every milestone and the place of every lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Level {
    Overview,
    Standard,
    #[default]
    Full,
}

/// What a roadmap picture shows and at which size. `unit` is one pixel of a 21 px tall cell.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub width: f32,
    pub height: f32,
    pub unit: f32,
    pub today: Date,
    pub selected: Option<usize>,
    pub level: Level,
    pub palette: Palette,
}

/// A part of the roadmap: the days from `start` to `end`, and the lanes in `lanes` by their index in the plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cut {
    pub start: Date,
    pub end: Date,
    pub lanes: Vec<usize>,
}

impl Cut {
    /// The weeks around `items`: from five weeks before the week the first of them starts to the Friday
    /// of the week after the last one ends, with every lane that has work in these weeks.
    pub fn around(plan: &Plan, schedule: &Schedule, items: &[usize]) -> Option<Cut> {
        let monday = |mut day: Date| {
            while day.weekday() != Weekday::Monday {
                day = previous(day);
            }
            day
        };
        let first = items.iter().map(|&index| schedule.slots[index].planned.start).min()?;
        let last = items.iter().map(|&index| schedule.slots[index].planned.finish).max()?;
        let start = (0..35).fold(monday(first), |day, _| previous(day));
        let end = (0..11).fold(monday(last), |day, _| next(day));
        let lanes = (0..plan.lanes.len())
            .filter(|&lane| plan.items.iter().zip(&schedule.slots).any(|(item, slot)| plan.lane(&item.lane) == Some(lane) && slot.planned.start <= end && slot.planned.finish >= start))
            .collect();
        Some(Cut { start, end, lanes })
    }
}

/// Height of a lane with one row of bars, as in the approved design.
const LANE: f32 = 92.0;
/// Each further row of bars in a lane adds this share of a lane.
const ROW: f32 = 0.6;

/// Left, top, right and bottom of something drawn, to keep names off it.
type Area = [f32; 4];

fn hits(a: &Area, b: &Area) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

struct Grid {
    start: Date,
    end: Date,
    top: f32,
    x0: f32,
    x1: f32,
    px: f32,
    /// The top of each lane, and the bottom of the last one.
    tops: Vec<f32>,
    /// From the top of a lane to the middle of its first row of bars, and from row to row.
    bar: f32,
    pitch: f32,
    /// Height of a lane with one row of bars.
    unit: f32,
}

impl Grid {
    fn x(&self, day: Date) -> f32 {
        self.x0 + days(self.start, day) as f32 * self.px
    }

    fn lane_y(&self, lane: usize) -> f32 {
        self.tops[lane]
    }

    fn lane_h(&self, lane: usize) -> f32 {
        self.tops[lane + 1] - self.tops[lane]
    }

    fn bottom(&self) -> f32 {
        self.tops[self.tops.len() - 1]
    }
}

/// One roadmap being drawn: the plan, its schedule and where things go.
struct Chart<'a> {
    plan: &'a Plan,
    schedule: &'a Schedule,
    /// For each item, the position of its lane among the lanes shown, or `usize::MAX` when it is hidden.
    lanes: Vec<usize>,
    /// The lanes shown, by their index in the plan.
    visible: Vec<usize>,
    /// The row of bars an item sits in within its lane, so items at the same time do not cover each other.
    rows: Vec<usize>,
    chain: HashSet<usize>,
    g: Grid,
    p: Palette,
    u: f32,
    width: f32,
    height: f32,
    selected: Option<usize>,
    level: Level,
}

impl Chart<'_> {
    /// Left end, right end and middle height of an item; both ends are the same for a milestone.
    fn bar(&self, index: usize) -> (f32, f32, f32) {
        let lane = if self.shown(index) { self.lanes[index] } else { 0 };
        let y = self.g.lane_y(lane) + self.g.bar + self.rows[index] as f32 * self.g.pitch;
        if self.plan.items[index].milestone {
            let x = self.milestone_x(index);
            return (x, x, y);
        }
        let span = self.schedule.slots[index].planned;
        (self.g.x(span.start).max(self.g.x0), (self.g.x(span.finish) + self.g.px).min(self.g.x1), y)
    }

    fn shown(&self, index: usize) -> bool {
        self.lanes[index] != usize::MAX
    }

    /// Calculated milestones sit at the end of their day, fixed ones in its middle.
    fn milestone_x(&self, index: usize) -> f32 {
        let x = self.g.x(self.schedule.slots[index].planned.finish);
        if self.plan.items[index].on.is_some() { x + self.g.px / 2.0 } else { x + self.g.px }
    }

    /// What the bar or diamond of an item covers, with the pin of a fixed date.
    fn area(&self, index: usize) -> Vec<Area> {
        let u = self.u;
        let (x0, x1, y) = self.bar(index);
        if !self.plan.items[index].milestone {
            return vec![[x0, y - 7.0 * u, x1, y + 7.0 * u]];
        }
        let mut out = vec![[x0 - 8.0 * u, y - 8.0 * u, x0 + 8.0 * u, y + 8.0 * u]];
        if self.plan.items[index].on.is_some() {
            out.push([x0 - 4.0 * u, y - 27.0 * u, x0 + 4.0 * u, y - 8.0 * u]);
        }
        out
    }

    fn late(&self, index: usize) -> bool {
        self.schedule.slots[index].spare < 0
    }

    fn ink(&self, index: usize) -> Color {
        if self.late(index) {
            self.p.alert
        } else if self.selected == Some(index) || self.chain.contains(&index) {
            self.p.bright
        } else {
            self.p.dim
        }
    }

    fn rule(&self, color: Color, width: f32) -> Stroke {
        Stroke { color, width: width * self.u, dash: None }
    }

    fn text_width(&self, text: &str, size: f32) -> f32 {
        text.chars().count() as f32 * size * self.u * ADVANCE
    }

    /// The first of `places` where a name fits inside its lane without touching anything in `taken`, and
    /// the area it then takes, with room to keep the next name apart.
    fn place(&self, lane: usize, text: &str, size: f32, places: &[(f32, f32, Anchor)], taken: &[Area]) -> Option<(f32, f32, Anchor, Area)> {
        let (u, w) = (self.u, self.text_width(text, size));
        let (top, bottom) = (self.g.lane_y(lane) + u, self.g.lane_y(lane) + self.g.lane_h(lane) - u);
        places.iter().find_map(|&(x, y, anchor)| {
            let left = match anchor {
                Anchor::Start => x,
                Anchor::Middle => x - w / 2.0,
                Anchor::End => x - w,
            };
            let area = [left, y - 0.78 * size * u, left + w, y + 0.24 * size * u];
            let inside = area[0] >= self.g.x0 && area[2] <= self.width - 2.0 * u && area[1] >= top && area[3] <= bottom;
            (inside && !taken.iter().any(|other| hits(&area, other))).then_some((x, y, anchor, [area[0] - 6.0 * u, area[1], area[2] + 6.0 * u, area[3]]))
        })
    }
}

/// The swimlane roadmap at the detail level of `settings`.
pub fn roadmap(plan: &Plan, schedule: &Schedule, calendars: &[Calendar], settings: &Settings) -> Scene {
    draw(plan, schedule, calendars, settings, None, None)
}

/// The part of the swimlane roadmap within `cut`; bars that reach beyond it are cut at its edges.
pub fn part(plan: &Plan, schedule: &Schedule, calendars: &[Calendar], settings: &Settings, cut: &Cut) -> Scene {
    draw(plan, schedule, calendars, settings, Some(cut), None)
}

/// The roadmap of a plan tried as what-if, with the places items had in `before` dashed.
pub fn what_if(plan: &Plan, schedule: &Schedule, calendars: &[Calendar], settings: &Settings, before: &Schedule) -> Scene {
    draw(plan, schedule, calendars, settings, None, Some(before))
}

fn draw(plan: &Plan, schedule: &Schedule, calendars: &[Calendar], settings: &Settings, cut: Option<&Cut>, before: Option<&Schedule>) -> Scene {
    let (p, u, today) = (settings.palette, settings.unit, settings.today);
    let mut s = Scene { width: settings.width, height: settings.height, background: Some(p.page), hatch: (p.raised, p.muted), hatch_alert: (p.alert_ground, p.alert), spare: p.dim, font: FONT, marks: Vec::new() };
    let c = chart(plan, schedule, settings, Some(settings.height), cut);

    frame(&mut s, &c, calendars);
    if let Some(target) = schedule.target.filter(|&target| c.shown(target)) {
        let xt = c.g.x(schedule.slots[target].planned.finish);
        s.add(Shape::Line { x1: xt, y1: c.g.lane_y(0) + 2.0 * u, x2: xt, y2: c.g.bottom(), stroke: Stroke { color: p.bright, width: u, dash: Some((4.0 * u, 4.0 * u)) } });
    }
    if c.level == Level::Overview {
        folded(&mut s, &c);
    } else {
        let mut taken: Vec<Area> = (0..plan.items.len()).filter(|&index| c.shown(index)).flat_map(|index| c.area(index)).collect();
        if c.level == Level::Full {
            arrows(&mut s, &c);
            free_slack(&mut s, &c, &mut taken);
        }
        if let Some(before) = before {
            ghosts(&mut s, &c, before);
        }
        bars(&mut s, &c);
        buffers(&mut s, &c, &analyse(plan, calendars, schedule, today).checks, &mut taken);
        labels(&mut s, &c, &mut taken);
    }

    if (c.g.start..=c.g.end).contains(&today) {
        let xt = c.g.x(today);
        s.add(Shape::Line { x1: xt, y1: c.g.top + 42.0 * u, x2: xt, y2: c.g.bottom(), stroke: c.rule(p.ink, 1.5) });
        s.add(Shape::Text { x: xt - 5.0 * u, y: c.g.top + 14.0 * u, text: "today".into(), size: 13.5 * u, color: p.bright, bold: true, anchor: Anchor::End });
    }
    s
}

/// The height at which every lane of the roadmap gets the room of the approved design.
pub fn natural_height(plan: &Plan, schedule: &Schedule, settings: &Settings) -> f32 {
    chart(plan, schedule, settings, None, None).g.bottom() + 22.0 * settings.unit
}

/// Lays the roadmap out: the days it shows, a row for every item, the detail level that fits, and
/// the lanes that share `height`, or get their natural room without one. Full needs 4 rows of 21 units
/// per lane and steps down to Standard below that; below 2 rows per lane, only the lanes around the
/// selection are shown, always with the lane of the fixed target. A `cut` sets the days and the lanes.
fn chart<'a>(plan: &'a Plan, schedule: &'a Schedule, settings: &Settings, height: Option<f32>, cut: Option<&Cut>) -> Chart<'a> {
    let (u, today) = (settings.unit, settings.today);
    let (start, end) = match cut {
        Some(cut) => (cut.start, cut.end),
        None => {
            let first = schedule.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(today).min(today);
            let last = schedule.slots.iter().map(|slot| slot.planned.finish.max(slot.latest.finish)).max().unwrap_or(today);
            let mut start = first;
            while start.weekday() != Weekday::Monday {
                start = previous(start);
            }
            (start, (0..20).fold(last, |day, _| next(day)))
        }
    };
    let within = |index: usize| cut.is_none() || (schedule.slots[index].planned.start <= end && schedule.slots[index].planned.finish >= start);
    let top = 26.0 * u;
    let x0 = if settings.width >= 1000.0 * u { 220.0 * u } else { 140.0 * u };
    let x1 = settings.width - 8.0 * u;
    let px = (x1 - x0) / (days(start, end) + 1) as f32;
    let lane_of: Vec<usize> = plan.items.iter().map(|item| plan.lane(&item.lane).unwrap_or(0)).collect();
    let mut c = Chart {
        plan,
        schedule,
        lanes: (0..plan.items.len()).map(|index| if within(index) { lane_of[index] } else { usize::MAX }).collect(),
        visible: cut.map_or_else(|| (0..plan.lanes.len()).collect(), |cut| cut.lanes.clone()),
        rows: vec![0; plan.items.len()],
        chain: schedule.chain.iter().copied().collect(),
        g: Grid { start, end, top, x0, x1, px, tops: vec![top + 64.0 * u; plan.lanes.len() + 1], bar: 40.0 * u, pitch: LANE * ROW * u, unit: LANE * u },
        p: settings.palette,
        u,
        width: settings.width,
        height: height.unwrap_or(f32::MAX),
        selected: settings.selected,
        level: settings.level,
    };
    if c.level != Level::Overview {
        c.rows = rows(&c);
    }
    let weight = |c: &Chart, lane: usize| {
        let rows = (0..plan.items.len()).filter(|&index| lane_of[index] == lane && within(index)).map(|index| c.rows[index] + 1).max().unwrap_or(1);
        1.0 + ROW * (rows - 1) as f32
    };
    let rows_y = top + 64.0 * u;
    let mut unit = LANE * u;
    if let Some(height) = height {
        let available = height - rows_y - 22.0 * u;
        let share = |c: &Chart, lanes: &[usize], room: f32| room / lanes.iter().map(|&lane| weight(c, lane)).sum::<f32>().max(1.0);
        let all = c.visible.clone();
        if c.level == Level::Full && share(&c, &all, available) < 84.0 * u {
            c.level = Level::Standard;
        }
        let mut room = available;
        if cut.is_none() && share(&c, &all, available) < 42.0 * u {
            room += 18.0 * u;
            let fits = |lanes: &[usize]| share(&c, lanes, room) >= 42.0 * u;
            let selected = settings.selected.map_or(0, |index| lane_of[index]);
            let target = schedule.target.map(|index| lane_of[index]);
            let window = |from: usize, extra: Option<usize>| {
                let mut lanes: Vec<usize> = Vec::new();
                for lane in from..plan.lanes.len() {
                    let mut next: Vec<usize> = lanes.iter().copied().chain([lane]).chain(extra.filter(|&extra| extra != lane)).collect();
                    next.sort_unstable();
                    next.dedup();
                    if !fits(&next) {
                        break;
                    }
                    lanes.push(lane);
                }
                lanes.extend(extra);
                lanes.sort_unstable();
                lanes.dedup();
                lanes
            };
            c.visible = (0..plan.lanes.len())
                .map(|from| window(from, None))
                .find(|lanes| lanes.contains(&selected) && target.is_none_or(|target| lanes.contains(&target)))
                .or_else(|| (0..plan.lanes.len()).map(|from| window(from, target)).find(|lanes| lanes.contains(&selected) && fits(lanes)))
                .unwrap_or_else(|| vec![selected]);
        }
        unit = share(&c, &c.visible, room).max(24.0 * u);
    }
    c.lanes = lane_of.iter().enumerate().map(|(index, lane)| c.visible.iter().position(|shown| shown == lane).filter(|_| within(index)).unwrap_or(usize::MAX)).collect();
    let weights: Vec<f32> = c.visible.iter().map(|&lane| weight(&c, lane)).collect();
    c.g.tops = std::iter::once(rows_y).chain(weights.iter().scan(rows_y, |y, weight| {
        *y += unit * weight;
        Some(*y)
    })).collect();
    c.g.bar = if unit >= 84.0 * u {
        40.0 * u
    } else if unit >= 63.0 * u {
        35.0 * u
    } else {
        (31.0 * u).min(unit - 9.0 * u).max(unit * 0.5)
    };
    c.g.pitch = unit * ROW;
    c.g.unit = unit;
    c
}

/// The detail level a roadmap shows at the size of `settings`, and the lanes it shows, by their index
/// in the plan.
pub fn shown(plan: &Plan, schedule: &Schedule, settings: &Settings) -> (Level, Vec<usize>) {
    let c = chart(plan, schedule, settings, Some(settings.height), None);
    (c.level, c.visible)
}

/// Rows within each lane, so items at the same time do not cover each other.
fn rows(c: &Chart) -> Vec<usize> {
    let items: Vec<(usize, f32, f32, bool)> = (0..c.plan.items.len())
        .map(|index| {
            let (x0, x1, _) = c.bar(index);
            (c.lanes[index], x0, x1, c.plan.items[index].milestone)
        })
        .collect();
    crate::page::rows(&items, 8.0 * c.u)
}

/// Months and weeks, phases, lanes, and in Full the holidays of each lane.
fn frame(s: &mut Scene, c: &Chart, calendars: &[Calendar]) {
    let (g, p, u) = (&c.g, c.p, c.u);
    let (top, end, bottom) = (g.top, g.end, g.bottom());
    let text = |x: f32, y: f32, text: String, size: f32, color: Color| Shape::Text { x, y, text, size: size * u, color, bold: false, anchor: Anchor::Start };

    let mut month = g.start.first_of_month();
    let mut labelled = false;
    while month <= end {
        let following = next(month.last_of_month());
        let xa = g.x(month.max(g.start));
        if month >= g.start {
            s.add(Shape::Line { x1: xa, y1: top, x2: xa, y2: bottom, stroke: c.rule(p.rule, 1.0 / u) });
        }
        let label = if !labelled || month.month() == 1 { month.strftime("%b %Y").to_string() } else { month.strftime("%b").to_string() };
        if g.x(following.min(end)) - xa > c.text_width(&label, 14.5) + 12.0 * u {
            s.add(text(xa + 6.0 * u, top + 14.0 * u, label, 14.5, p.ink));
            labelled = true;
        }
        month = following;
    }
    let columns = 7.0 * g.px / (10.0 * u);
    let step = if columns >= 4.0 { 1 } else if columns >= 2.0 { 2 } else { 0 };
    let mut monday = g.start;
    let mut count = 0;
    while step > 0 && monday <= end {
        if count % step == 0 {
            let week = format!("{}{:02}", if count == 0 { "W" } else { "" }, monday.iso_week_date().week());
            s.add(text(g.x(monday) + 2.0 * u, top + 33.0 * u, week, 12.5, p.muted));
        }
        count += 1;
        monday = (0..7).fold(monday, |day, _| next(day));
    }
    s.add(Shape::Line { x1: g.x0, y1: top + 40.0 * u, x2: g.x1, y2: top + 40.0 * u, stroke: c.rule(p.rule, 1.0 / u) });

    for (name, from, to) in c.schedule.phases(c.plan).into_iter().filter(|(_, from, to)| *to >= g.start && *from <= end) {
        chevron(s, &p, u, g.x(from.max(g.start)), g.x(to.min(end)) + g.px, top + 44.0 * u, 18.0 * u, &name);
    }
    s.add(text(12.0 * u, top + 58.0 * u, "Phases".into(), 14.0, p.muted));

    let room = g.x0 - 24.0 * u;
    for (index, &shown) in c.visible.iter().enumerate() {
        let lane = &c.plan.lanes[shown];
        let y = g.lane_y(index);
        if c.level == Level::Full {
            for (day, _) in calendars[shown].holidays(g.start, end) {
                if !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday) {
                    s.add(Shape::Rect { x: g.x(day), y: y + 8.0 * u, w: g.px, h: g.lane_h(index) - 16.0 * u, radius: 0.0, fill: Fill::Hatch, stroke: None });
                }
            }
        }
        s.add(Shape::Line { x1: 6.0 * u, y1: y, x2: g.x1, y2: y, stroke: c.rule(p.rule, 1.0 / u) });
        let name_y = if g.unit >= 63.0 * u { 31.0 * u } else { (36.0 * u).min(g.unit - 6.0 * u) };
        s.add(text(12.0 * u, y + name_y, fit(&lane.name, 16.6 * u, room), 16.6, p.ink));
        if c.level == Level::Full && g.unit >= 63.0 * u {
            let place = [lane.city.as_deref(), lane.place.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
            s.add(text(12.0 * u, y + 52.0 * u, fit(&place, 14.0 * u, room), 14.0, p.muted));
        }
    }
}

/// Who waits on whom, as arrows from the end of an item to the start of the next.
fn arrows(s: &mut Scene, c: &Chart) {
    let u = c.u;
    for (index, item) in c.plan.items.iter().enumerate() {
        for pred in item.after.iter().filter_map(|id| c.plan.item(id)).filter(|&pred| c.shown(pred) && c.shown(index)) {
            let (ax0, ax1, ay) = c.bar(pred);
            let (bx0, _, by) = c.bar(index);
            let from_milestone = c.plan.items[pred].milestone;
            let x1 = ax1 + if from_milestone { 7.0 * u } else { 0.0 };
            let x2 = bx0 - if item.milestone { 7.0 * u } else { 0.0 };
            let (color, opacity) = if c.chain.contains(&pred) && c.chain.contains(&index) { (c.p.dim, 0.9) } else { (c.p.muted, 0.7) };
            let stroke = Some(c.rule(color, 1.2));
            let line = |points: Vec<(f32, f32)>| Mark { shape: Shape::Path { points, closed: false, fill: Fill::None, stroke }, opacity, item: None };
            if (ay - by).abs() < 1.0 {
                s.marks.push(line(vec![(x1, ay), (x2 - u, by)]));
                continue;
            }
            if from_milestone {
                let y1 = ay + if by > ay { 7.0 * u } else { -7.0 * u };
                s.marks.push(line(vec![(ax0, y1), (ax0, by), (x2 - u, by)]));
            } else {
                let xm = x1 + ((x2 - x1) / 2.0).clamp(3.0 * u, 6.0 * u);
                s.marks.push(line(vec![(x1, ay), (xm, ay), (xm, by), (x2 - u, by)]));
            }
            let head = vec![(x2, by), (x2 - 5.0 * u, by - 3.0 * u), (x2 - 5.0 * u, by + 3.0 * u)];
            s.marks.push(Mark { shape: Shape::Path { points: head, closed: true, fill: Fill::Color(color), stroke: None }, opacity, item: None });
        }
    }
}

/// How far each bar can slip before it pushes a successor, hatched behind its end.
fn free_slack(s: &mut Scene, c: &Chart, taken: &mut Vec<Area>) {
    let u = c.u;
    for (index, item) in c.plan.items.iter().enumerate() {
        let slot = &c.schedule.slots[index];
        if item.milestone || c.chain.contains(&index) || !c.shown(index) {
            continue;
        }
        let free = free_until(c.plan, c.schedule, index);
        if free > slot.planned.finish {
            let (_, x1, y) = c.bar(index);
            let xe = (c.g.x(free) + c.g.px).min(c.g.x1);
            s.add_for(index, 1.0, Shape::Rect { x: x1, y: y - 7.0 * u, w: xe - x1, h: 14.0 * u, radius: 3.0 * u, fill: Fill::Spare, stroke: Some(c.rule(c.p.muted, 1.0)) });
            taken.push([x1, y - 7.0 * u, xe, y + 7.0 * u]);
        }
    }
}

fn bars(s: &mut Scene, c: &Chart) {
    let u = c.u;
    for (index, item) in c.plan.items.iter().enumerate().filter(|&(index, _)| c.shown(index)) {
        if item.milestone {
            milestone(s, c, index);
            continue;
        }
        let (x0, x1, y) = c.bar(index);
        let (color, opacity) = if c.late(index) {
            (c.p.alert, 0.92)
        } else if c.chain.contains(&index) {
            (c.p.ink, 0.95)
        } else {
            (c.p.dim, 0.5)
        };
        s.add_for(index, opacity, Shape::Rect { x: x0 + u, y: y - 7.0 * u, w: x1 - x0 - 2.5 * u, h: 14.0 * u, radius: 3.0 * u, fill: Fill::Color(color), stroke: None });
        if c.selected == Some(index) {
            corners(s, c, x0, x1, y, 11.0 * u);
        }
    }
}

/// The places items had before a what-if, dashed, for the items that moved.
fn ghosts(s: &mut Scene, c: &Chart, before: &Schedule) {
    let u = c.u;
    let stroke = Stroke { color: c.p.muted, width: 1.2 * u, dash: Some((3.0 * u, 3.0 * u)) };
    for (index, item) in c.plan.items.iter().enumerate().filter(|&(index, _)| c.shown(index)) {
        let Some(old) = before.slots.get(index).map(|slot| slot.planned) else { continue };
        if old == c.schedule.slots[index].planned {
            continue;
        }
        let y = c.bar(index).2;
        if item.milestone {
            let x = c.g.x(old.finish) + if item.on.is_some() { c.g.px / 2.0 } else { c.g.px };
            let r = 7.0 * u;
            s.add(Shape::Path { points: vec![(x, y - r), (x + r, y), (x, y + r), (x - r, y)], closed: true, fill: Fill::None, stroke: Some(stroke) });
        } else {
            let (x0, x1) = (c.g.x(old.start).max(c.g.x0), (c.g.x(old.finish) + c.g.px).min(c.g.x1));
            if x1 > x0 {
                s.add(Shape::Rect { x: x0 + u, y: y - 7.0 * u, w: x1 - x0 - 2.5 * u, h: 14.0 * u, radius: 3.0 * u, fill: Fill::None, stroke: Some(stroke) });
            }
        }
    }
}

/// A milestone in the theme's accent, or red when it is late, with room around it so it stands apart
/// from the bar it ends (picture 45, B).
fn milestone(s: &mut Scene, c: &Chart, index: usize) {
    let (p, u) = (c.p, c.u);
    let (x, _, y) = c.bar(index);
    let fixed = c.plan.items[index].on.is_some();
    diamond(s, index, x, y, if fixed { 11.0 } else { 10.0 } * u, Fill::Color(p.page), None);
    let tone = if c.late(index) { p.alert } else { p.accent };
    if fixed {
        diamond(s, index, x, y, 8.0 * u, Fill::Color(tone), None);
        s.add_for(index, 1.0, Shape::Line { x1: x, y1: y - 10.0 * u, x2: x, y2: y - 20.0 * u, stroke: c.rule(tone, 1.6) });
        s.add_for(index, 1.0, Shape::Circle { x, y: y - 23.0 * u, r: 4.0 * u, fill: tone });
    } else if c.late(index) || c.chain.contains(&index) {
        diamond(s, index, x, y, 7.0 * u, Fill::Color(tone), None);
    } else {
        diamond(s, index, x, y, 7.0 * u, Fill::Color(p.page), Some(c.rule(p.accent, 1.5)));
    }
}

/// Names of fixed dates first, then from left to right: a bar's name above it, else below it, else to
/// its right, else ending at its end; a milestone's name to its right, else to its left, above, level
/// or below. A name that fits nowhere is left out; the selection panel shows it.
fn labels(s: &mut Scene, c: &Chart, taken: &mut Vec<Area>) {
    let u = c.u;
    let size = 13.5;
    let mut order: Vec<usize> = (0..c.plan.items.len()).filter(|&index| c.shown(index)).collect();
    order.sort_by(|&a, &b| (c.plan.items[a].on.is_none(), c.bar(a).0).partial_cmp(&(c.plan.items[b].on.is_none(), c.bar(b).0)).unwrap_or(std::cmp::Ordering::Equal));
    for index in order {
        let item = &c.plan.items[index];
        let (x0, x1, y) = c.bar(index);
        let (text, places) = if item.milestone {
            let day = c.schedule.slots[index].planned.finish;
            let text = if item.on.is_some() || c.level == Level::Full { format!("{}  {}", item.name, day.strftime("%d %b")) } else { item.name.clone() };
            (text, [(x0 + 12.0 * u, y + 5.0 * u, Anchor::Start), (x0 - 12.0 * u, y - 12.0 * u, Anchor::End), (x0 - 12.0 * u, y + 5.0 * u, Anchor::End), (x0 - 12.0 * u, y + 24.0 * u, Anchor::End), (x0 + 12.0 * u, y + 24.0 * u, Anchor::Start)])
        } else {
            (item.name.clone(), [(x0, y - 13.0 * u, Anchor::Start), (x0, y + 24.0 * u, Anchor::Start), (x1 + 6.0 * u, y + 5.0 * u, Anchor::Start), (x1, y - 13.0 * u, Anchor::End), (x1, y + 24.0 * u, Anchor::End)])
        };
        if let Some((x, y, anchor, area)) = c.place(c.lanes[index], &text, size, &places, taken) {
            taken.push(area);
            s.add_for(index, 1.0, Shape::Text { x, y, text, size: size * u, color: c.ink(index), bold: c.selected == Some(index), anchor });
        }
    }
}

/// The "no buffer" warning, drawn as a bracket from the last item before a fixed date to that date.
fn buffers(s: &mut Scene, c: &Chart, checks: &[Check], taken: &mut Vec<Area>) {
    let u = c.u;
    for check in checks {
        let Check::NoBuffer { before, target } = *check else { continue };
        if !c.shown(before) || !c.shown(target) {
            continue;
        }
        let (bx0, bx1, _) = c.bar(before);
        let xa = if c.plan.items[before].milestone { bx0 } else { bx1 };
        let (xl, _, y) = c.bar(target);
        let y = y + (26.0 * u).min(c.g.unit - c.g.bar + 3.0 * u);
        let lane = c.lanes[target];
        let floor = if lane + 1 == c.visible.len() { (c.g.bottom() + 20.0 * u).min(c.height - 2.0 * u) } else { c.g.lane_y(lane + 1) - u };
        let text = if c.level == Level::Full { "▲ 0 wd buffer" } else { "▲ no buffer" };
        let w = c.text_width(text, 13.5);
        let x = if xl + 10.0 * u + w <= c.width - 2.0 * u { xl + 10.0 * u } else { xa.min(xl) - 10.0 * u - w };
        if y + 9.0 * u > floor || x < c.g.x0 {
            continue;
        }
        s.add(Shape::Path { points: vec![(xa, y - 5.0 * u), (xa, y), (xl, y), (xl, y - 5.0 * u)], closed: false, fill: Fill::None, stroke: Some(c.rule(c.p.warn, 1.6)) });
        taken.push([xa.min(xl) - u, y - 6.0 * u, xl.max(xa) + u, y + u]);
        taken.push([x, y - 6.0 * u, x + w, y + 9.0 * u]);
        s.add(Shape::Text { x, y: y + 5.0 * u, text: text.into(), size: 13.5 * u, color: c.p.warn, bold: true, anchor: Anchor::Start });
    }
}

/// Overview: each lane as one bar from its first to its last item, with its milestones.
fn folded(s: &mut Scene, c: &Chart) {
    let (p, u) = (c.p, c.u);
    let size = 14.5;
    let selected_lane = c.selected.map(|index| c.lanes[index]);
    let mut taken: Vec<Area> = Vec::new();
    let mut ends = vec![None; c.visible.len()];
    for (lane, end) in ends.iter_mut().enumerate() {
        let tasks: Vec<usize> = (0..c.plan.items.len()).filter(|&index| c.lanes[index] == lane && !c.plan.items[index].milestone).collect();
        let Some(&first) = tasks.first() else { continue };
        let y = c.g.lane_y(lane) + c.g.bar;
        let x0 = tasks.iter().map(|&index| c.bar(index).0).fold(f32::MAX, f32::min);
        let x1 = tasks.iter().map(|&index| c.bar(index).1).fold(f32::MIN, f32::max);
        *end = Some(x1);
        let (color, opacity) = if tasks.iter().any(|&index| c.late(index)) {
            (p.alert, 0.92)
        } else if tasks.iter().any(|index| c.chain.contains(index)) {
            (p.ink, 0.95)
        } else {
            (p.dim, 0.5)
        };
        s.add_for(first, opacity, Shape::Rect { x: x0, y: y - 10.0 * u, w: x1 - x0, h: 20.0 * u, radius: 4.0 * u, fill: Fill::Color(color), stroke: None });
        taken.push([x0, y - 10.0 * u, x1, y + 10.0 * u]);
        if selected_lane == Some(lane) {
            corners(s, c, x0, x1, y, 14.0 * u);
        }
        let count = if tasks.len() == 1 { "1 item".to_string() } else { format!("{} items", tasks.len()) };
        taken.push([x0, y - 27.0 * u, x0 + c.text_width(&count, 12.5), y - 14.0 * u]);
        s.add(Shape::Text { x: x0, y: y - 17.0 * u, text: count, size: 12.5 * u, color: p.muted, bold: false, anchor: Anchor::Start });
    }
    let mut milestones: Vec<usize> = (0..c.plan.items.len()).filter(|&index| c.plan.items[index].milestone && c.shown(index)).collect();
    milestones.sort_by(|&a, &b| c.bar(a).0.total_cmp(&c.bar(b).0));
    taken.extend(milestones.iter().flat_map(|&index| c.area(index)));
    for index in milestones {
        milestone(s, c, index);
        let item = &c.plan.items[index];
        let (x, _, y) = c.bar(index);
        let text = match item.on {
            Some(day) => format!("{}  {}", item.name, day.strftime("%d %b")),
            None => item.name.clone(),
        };
        let (right, left_above, left_below) = ((x + 14.0 * u, y + 5.0 * u, Anchor::Start), (x - 14.0 * u, y - 14.0 * u, Anchor::End), (x - 14.0 * u, y + 26.0 * u, Anchor::End));
        let near_end = ends[c.lanes[index]].is_some_and(|end: f32| (x - end).abs() < 12.0 * u);
        let places = if near_end { [left_above, right, left_below] } else { [right, left_above, left_below] };
        if let Some((x, y, anchor, area)) = c.place(c.lanes[index], &text, size, &places, &taken) {
            taken.push(area);
            let color = if c.late(index) { p.alert } else if c.chain.contains(&index) { p.bright } else { p.dim };
            s.add_for(index, 1.0, Shape::Text { x, y, text, size: size * u, color, bold: false, anchor });
        }
    }
}

fn corners(s: &mut Scene, c: &Chart, x0: f32, x1: f32, y: f32, h: f32) {
    let (u, k) = (c.u, 6.0 * c.u);
    for (cx, cy, dx, dy) in [(x0 - 4.0 * u, y - h, 1.0, 1.0), (x1 + 4.0 * u, y - h, -1.0, 1.0), (x0 - 4.0 * u, y + h, 1.0, -1.0), (x1 + 4.0 * u, y + h, -1.0, -1.0)] {
        s.add(Shape::Path { points: vec![(cx + dx * k, cy), (cx, cy), (cx, cy + dy * k)], closed: false, fill: Fill::None, stroke: Some(c.rule(c.p.bright, 1.6)) });
    }
}

/// `text` cut to `room` pixels at `size`, ending in "…" when it was cut.
fn fit(text: &str, size: f32, room: f32) -> String {
    let fits = |count: usize| count as f32 * size * ADVANCE <= room;
    let count = text.chars().count();
    if fits(count) {
        return text.to_string();
    }
    let keep = (0..count).rev().find(|&keep| fits(keep + 1)).unwrap_or(0);
    text.chars().take(keep).collect::<String>().trim_end().to_string() + "…"
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
    let count = label.chars().count();
    let size = (room / (count as f32 * ADVANCE)).clamp(10.5 * u, 13.5 * u);
    let fits = |chars: usize| chars as f32 * size * ADVANCE <= room;
    let label = if fits(count) {
        label.to_string()
    } else {
        match (4..count).rev().find(|&keep| fits(keep + 1)) {
            Some(keep) => label.chars().take(keep).collect::<String>() + "…",
            None => return,
        }
    };
    s.add(Shape::Text { x: (x0 + x1) / 2.0 + tip / 2.0, y: y + h / 2.0 + size * 0.36, text: label, size, color: p.dim, bold: false, anchor: Anchor::Middle });
}

fn days(from: Date, to: Date) -> i32 {
    from.until(to).map(|span| span.get_days()).unwrap_or(0)
}
