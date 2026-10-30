//! The light export page that the milestone chart, the Gantt chart, the phase plan, Now / Next / Later
//! and the light roadmap share: a 16:9 slide in Noto Sans with the plan's name, the view, the day it
//! stands for, and where the dates come from. A page grows taller when a plan needs more rows.

use jiff::civil::{Date, Weekday};
use pchart_core::calendar::{next, previous};
use pchart_core::{Plan, Schedule};

use crate::scene::{Anchor, Color, Fill, Mark, Scene, Shape, Stroke};
use crate::style::{SANS, sans_width};

pub const WIDTH: f32 = 1124.0;
pub const HEIGHT: f32 = 632.0;
pub const X0: f32 = 56.0;
pub const X1: f32 = WIDTH - 56.0;

pub const PAPER: Color = Color(0xff, 0xff, 0xff);
pub const INK: Color = Color(0x1f, 0x24, 0x21);
pub const GRAY: Color = Color(0x6b, 0x70, 0x6b);
pub const LIGHT: Color = Color(0xee, 0xf0, 0xea);
pub const MID: Color = Color(0xc9, 0xcd, 0xc4);
/// Bars of items with time to spare.
pub const SOFT: Color = Color(0xb9, 0xbe, 0xb4);
/// "Waits on" lines along the chain that decides the date.
pub const LINK: Color = Color(0x7d, 0x82, 0x7b);
pub const CARD: Color = Color(0xf3, 0xf4, 0xef);
pub const SHADES: [Color; 4] = [Color(0xd9, 0xdd, 0xd3), Color(0xcd, 0xd2, 0xc6), Color(0xbf, 0xc5, 0xb7), Color(0xb1, 0xb8, 0xa8)];
/// A fixed date.
pub const FIXED: Color = Color(0xe3, 0x3d, 0x84);
/// Too late.
pub const LATE: Color = Color(0xee, 0x58, 0x5d);
/// Look: the ground of a warning tag, and the line of its bracket.
pub const WARN: Color = Color(0xd3, 0xdd, 0x2c);
pub const WARN_LINE: Color = Color(0xb4, 0xbe, 0x12);

/// Left, top, right and bottom.
pub type Area = [f32; 4];

fn hits(a: &Area, b: &Area, margin: f32) -> bool {
    a[0] < b[2] + margin && b[0] < a[2] + margin && a[1] < b[3] + margin && b[1] < a[3] + margin
}

/// The time line of a page: day → x.
pub struct Axis {
    pub start: Date,
    pub end: Date,
    pub x0: f32,
    pub x1: f32,
    pub px: f32,
}

impl Axis {
    /// From the Monday of the week of `today` or of the first item, whichever is earlier, to `after`
    /// days past the last day of the plan.
    pub fn new(schedule: &Schedule, today: Date, after: usize, x0: f32, x1: f32) -> Axis {
        let first = schedule.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(today).min(today);
        let last = schedule.slots.iter().map(|slot| slot.planned.finish).max().unwrap_or(today);
        let mut start = first;
        while start.weekday() != Weekday::Monday {
            start = previous(start);
        }
        let end = (0..after).fold(last, |day, _| next(day));
        let px = (x1 - x0) / (days(start, end) + 1) as f32;
        Axis { start, end, x0, x1, px }
    }

    pub fn x(&self, day: Date) -> f32 {
        self.x0 + days(self.start, day) as f32 * self.px
    }

    /// The end of a day.
    pub fn xe(&self, day: Date) -> f32 {
        self.x(day) + self.px
    }

    /// Calculated milestones sit at the end of their day, fixed ones in its middle.
    pub fn milestone(&self, plan: &Plan, schedule: &Schedule, index: usize) -> f32 {
        let day = schedule.slots[index].planned.finish;
        if plan.items[index].on.is_some() { self.x(day) + self.px / 2.0 } else { self.xe(day) }
    }
}

pub struct Page {
    pub scene: Scene,
    texts: Vec<Area>,
    shapes: Vec<(Area, Option<usize>)>,
    /// Month lines, drawn under everything and around every text when the page is finished.
    grids: Vec<(usize, Vec<f32>, f32, f32)>,
}

impl Page {
    /// A page with the plan's name, the view and the day as title, and where the dates come from at
    /// the foot. `height` is at least that of a slide.
    pub fn new(plan: &Plan, schedule: &Schedule, today: Date, view: &str, height: f32) -> Page {
        let height = height.max(HEIGHT);
        let scene = Scene { width: WIDTH, height, background: Some(PAPER), hatch: (LIGHT, MID), hatch_alert: (LIGHT, LATE), spare: MID, font: SANS, marks: Vec::new() };
        let mut page = Page { scene, texts: Vec::new(), shapes: Vec::new(), grids: Vec::new() };
        page.text(X0, 70.0, &plan.name, 34.0, INK, true, Anchor::Start, None);
        page.text(X0, 102.0, &format!("{view} · as of {}", today.strftime("%d %b %Y")), 17.0, GRAY, false, Anchor::Start, None);
        page.line(X0, height - 60.0, X1, height - 60.0, LIGHT, 1.5);
        let source = match schedule.target {
            Some(target) => format!("Dates scheduled backwards from the fixed {} date", common(&plan.items[target].name)),
            None => "Dates scheduled forwards from the start of the plan".to_string(),
        };
        page.text(X0, height - 30.0, &source, 14.0, GRAY, false, Anchor::Start, None);
        page.text(X1, height - 30.0, "pChart", 14.0, GRAY, true, Anchor::End, None);
        page
    }

    pub fn height(&self) -> f32 {
        self.scene.height
    }

    fn area(x: f32, y: f32, text: &str, size: f32, bold: bool, anchor: Anchor) -> Area {
        let w = sans_width(text, size, bold);
        let left = match anchor {
            Anchor::Start => x,
            Anchor::Middle => x - w / 2.0,
            Anchor::End => x - w,
        };
        [left, y - 0.74 * size, left + w, y + 0.24 * size]
    }

    /// Writes `text` with its baseline at `y` and returns where it ends on the right.
    #[allow(clippy::too_many_arguments)]
    pub fn text(&mut self, x: f32, y: f32, text: &str, size: f32, color: Color, bold: bool, anchor: Anchor, item: Option<usize>) -> f32 {
        let area = Page::area(x, y, text, size, bold, anchor);
        self.texts.push(area);
        self.scene.marks.push(Mark { shape: Shape::Text { x, y, text: text.to_string(), size, color, bold, anchor }, opacity: 1.0, item });
        area[2]
    }

    /// Writes `text` at the first of `places` where it stays inside `bounds` and touches no other text
    /// and no shape but those of `item`. Returns false when it fits nowhere.
    #[allow(clippy::too_many_arguments)]
    pub fn place(&mut self, places: &[(f32, f32, Anchor)], text: &str, size: f32, color: Color, bold: bool, item: Option<usize>, bounds: Area) -> bool {
        let free = |area: &Area| {
            area[0] >= bounds[0] && area[2] <= bounds[2] && area[1] >= bounds[1] && area[3] <= bounds[3] && !self.texts.iter().any(|other| hits(area, other, 2.0)) && !self.shapes.iter().any(|(other, owner)| (item.is_none() || *owner != item) && hits(area, other, 1.5))
        };
        let Some(&(x, y, anchor)) = places.iter().find(|&&(x, y, anchor)| free(&Page::area(x, y, text, size, bold, anchor))) else { return false };
        self.text(x, y, text, size, color, bold, anchor, item);
        true
    }

    pub fn shape(&mut self, area: Area, item: Option<usize>) {
        self.shapes.push(([area[0].min(area[2]), area[1].min(area[3]), area[0].max(area[2]), area[1].max(area[3])], item));
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, width: f32) {
        self.scene.add(Shape::Line { x1, y1, x2, y2, stroke: Stroke { color, width, dash: None } });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn vline(&mut self, x: f32, y0: f32, y1: f32, color: Color, width: f32, dash: Option<(f32, f32)>, item: Option<usize>) {
        self.scene.add(Shape::Line { x1: x, y1: y0, x2: x, y2: y1, stroke: Stroke { color, width, dash } });
        self.shape([x - width / 2.0, y0, x + width / 2.0, y1], item);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn bar(&mut self, x0: f32, x1: f32, y: f32, h: f32, color: Color, radius: f32, item: Option<usize>) {
        let shape = Shape::Rect { x: x0, y: y - h / 2.0, w: x1 - x0, h, radius, fill: Fill::Color(color), stroke: None };
        self.scene.marks.push(Mark { shape, opacity: 1.0, item });
        self.shape([x0, y - h / 2.0, x1, y + h / 2.0], item);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn diamond(&mut self, x: f32, y: f32, r: f32, fill: Color, stroke: Option<(Color, f32)>, item: Option<usize>) {
        let stroke = stroke.map(|(color, width)| Stroke { color, width, dash: None });
        self.scene.marks.push(Mark { shape: Shape::Path { points: vec![(x, y - r), (x + r, y), (x, y + r), (x - r, y)], closed: true, fill: Fill::Color(fill), stroke }, opacity: 1.0, item });
        self.shape([x - r, y - r, x + r, y + r], item);
    }

    /// A phase arrow; returns its tip.
    pub fn chevron(&mut self, x0: f32, x1: f32, y: f32, h: f32, fill: Color, tip: f32) -> f32 {
        let tip = tip.min((x1 - x0) / 2.0);
        let points = vec![(x0, y), (x1 - tip, y), (x1, y + h / 2.0), (x1 - tip, y + h), (x0, y + h), (x0 + tip, y + h / 2.0)];
        self.scene.add(Shape::Path { points, closed: true, fill: Fill::Color(fill), stroke: None });
        self.shape([x0, y, x1, y + h], None);
        tip
    }

    /// A warning: a triangle and `text` in ink on a yellow tag, since yellow text does not read on
    /// white. The triangle is drawn, as Noto Sans has none. Returns the tag's left and right end.
    pub fn tag(&mut self, x: f32, y: f32, text: &str, anchor: Anchor) -> (f32, f32) {
        let w = 20.0 + if text.is_empty() { 0.0 } else { sans_width(text, 12.0, true) + 4.0 };
        let x0 = if anchor == Anchor::End { x - w } else { x };
        self.scene.add(Shape::Rect { x: x0, y: y - 13.0, w, h: 18.0, radius: 3.0, fill: Fill::Color(WARN), stroke: None });
        self.scene.add(Shape::Path { points: vec![(x0 + 10.0, y - 8.5), (x0 + 14.0, y - 1.0), (x0 + 6.0, y - 1.0)], closed: true, fill: Fill::Color(INK), stroke: None });
        self.shape([x0, y - 13.0, x0 + w, y + 5.0], None);
        if !text.is_empty() {
            self.text(x0 + 18.0, y, text, 12.0, INK, true, Anchor::Start, None);
        }
        (x0, x0 + w)
    }

    /// A "waits on" line: down from the end of one item, then right to the start of the next.
    pub fn link(&mut self, xv: f32, y0: f32, x2: f32, y2: f32, color: Color) {
        let stroke = Some(Stroke { color, width: 1.2, dash: None });
        self.scene.add(Shape::Path { points: vec![(xv, y0), (xv, y2), (x2 - 5.0, y2)], closed: false, fill: Fill::None, stroke });
        self.scene.add(Shape::Path { points: vec![(x2, y2), (x2 - 6.0, y2 - 3.5), (x2 - 6.0, y2 + 3.5)], closed: true, fill: Fill::Color(color), stroke: None });
        self.shape([xv - 0.6, y0.min(y2) + 2.0, xv + 0.6, y0.max(y2)], None);
    }

    /// Month names along `y` with "Nov 2026" on the first and on January; with `grid`, a line for each
    /// month from `top` to `bottom`.
    pub fn months(&mut self, axis: &Axis, y: f32, top: f32, bottom: f32, size: f32) {
        let mut xs = Vec::new();
        let mut month = if axis.start.day() == 1 { axis.start } else { next(axis.start.last_of_month()) };
        let mut first = true;
        while month <= axis.end {
            let following = next(month.last_of_month());
            xs.push(axis.x(month));
            let label = if first || month.month() == 1 { month.strftime("%b %Y").to_string() } else { month.strftime("%b").to_string() };
            if axis.x(following.min(next(axis.end))) - axis.x(month) > sans_width(&label, size, false) + 9.0 {
                self.text(axis.x(month) + 7.0, y, &label, size, GRAY, false, Anchor::Start, None);
                first = false;
            }
            month = following;
        }
        self.grids.push((self.scene.marks.len(), xs, top, bottom));
    }

    /// Calendar week numbers along `y`, the first one with a "W".
    pub fn weeks(&mut self, axis: &Axis, y: f32, size: f32) {
        let mut monday = axis.start;
        let mut first = true;
        while monday <= axis.end {
            let label = format!("{}{:02}", if first { "W" } else { "" }, monday.iso_week_date().week());
            if axis.xe(axis.end) - axis.x(monday) > sans_width(&label, size, false) + 4.0 {
                self.text(axis.x(monday) + 3.0, y, &label, size, if first { GRAY } else { MID }, false, Anchor::Start, None);
            }
            first = false;
            monday = (0..7).fold(monday, |day, _| next(day));
        }
    }

    /// A legend along `y`, left to right.
    pub fn legend(&mut self, y: f32, entries: &[(Key, &str)]) {
        let mut x = X0;
        for (key, label) in entries {
            match key {
                Key::Chain | Key::Soft => {
                    self.bar(x, x + 22.0, y - 4.0, 10.0, if *key == Key::Chain { INK } else { SOFT }, 2.0, None);
                    x += 30.0;
                }
                Key::Edge => {
                    self.bar(x, x + 4.0, y - 4.0, 16.0, INK, 1.0, None);
                    x += 12.0;
                }
                Key::Link => {
                    let stroke = Some(Stroke { color: LINK, width: 1.2, dash: None });
                    self.scene.add(Shape::Path { points: vec![(x, y - 12.0), (x, y - 4.0), (x + 16.0, y - 4.0)], closed: false, fill: Fill::None, stroke });
                    self.scene.add(Shape::Path { points: vec![(x + 21.0, y - 4.0), (x + 15.0, y - 7.5), (x + 15.0, y - 0.5)], closed: true, fill: Fill::Color(LINK), stroke: None });
                    x += 29.0;
                }
                Key::Milestone => {
                    self.diamond(x + 7.0, y - 4.0, 6.0, INK, None, None);
                    x += 21.0;
                }
                Key::Fixed => {
                    self.diamond(x + 7.0, y - 4.0, 7.0, FIXED, None, None);
                    x += 22.0;
                }
                Key::Today => {
                    self.vline(x + 5.0, y - 13.0, y + 4.0, GRAY, 1.5, None, None);
                    x += 16.0;
                }
                Key::Warning => x = self.tag(x, y, "", Anchor::Start).1 + 8.0,
            }
            x = self.text(x, y, label, 13.0, GRAY, false, Anchor::Start, None) + 26.0;
        }
    }

    /// The scene, with the month lines under everything, broken around every text.
    pub fn finish(mut self) -> Scene {
        for (at, xs, top, bottom) in std::mem::take(&mut self.grids).into_iter().rev() {
            let mut lines = Vec::new();
            for x in xs {
                let mut cuts: Vec<(f32, f32)> = self.texts.iter().filter(|text| text[0] - 3.0 < x && x < text[2] + 3.0).map(|text| (text[1] - 3.0, text[3] + 3.0)).collect();
                cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
                cuts.push((bottom, bottom));
                let mut y = top;
                for (from, to) in cuts {
                    if from.min(bottom) - y > 2.0 {
                        lines.push(Mark { shape: Shape::Line { x1: x, y1: y, x2: x, y2: from.min(bottom), stroke: Stroke { color: LIGHT, width: 1.5, dash: None } }, opacity: 1.0, item: None });
                    }
                    y = y.max(to);
                }
            }
            self.scene.marks.splice(at..at, lines);
        }
        self.scene
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Chain,
    Soft,
    Edge,
    Link,
    Milestone,
    Fixed,
    Today,
    Warning,
}

/// `text` cut to `room` pixels in Noto Sans, ending in "…" when it was cut.
pub fn fit(text: &str, size: f32, bold: bool, room: f32) -> String {
    if sans_width(text, size, bold) <= room {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let keep = (0..chars.len()).rev().find(|&keep| sans_width(&(chars[..keep].iter().collect::<String>() + "…"), size, bold) <= room).unwrap_or(0);
    chars[..keep].iter().collect::<String>().trim_end().to_string() + "…"
}

/// A name inside a sentence: "Launch" becomes "launch", "SAP cutover" stays as it is.
pub fn common(name: &str) -> String {
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(first), Some(second)) if first.is_uppercase() && second.is_lowercase() => first.to_lowercase().chain(name.chars().skip(1)).collect(),
        _ => name.to_string(),
    }
}

/// "16 Nov – 27 Nov" for a bar, "18 Dec" for a milestone, "01 Mar · fixed" for a fixed date.
pub fn dates(plan: &Plan, schedule: &Schedule, index: usize, weekday: bool) -> String {
    let format = if weekday { "%a %d %b" } else { "%d %b" };
    let slot = &schedule.slots[index].planned;
    let item = &plan.items[index];
    if item.milestone {
        let day = slot.finish.strftime(format).to_string();
        if item.on.is_some() { format!("{day} · fixed") } else { day }
    } else {
        format!("{} – {}", slot.start.strftime(format), slot.finish.strftime(format))
    }
}

/// Rows within lanes, first come first served: an item goes into the first row of its lane where it
/// covers nothing. `items` holds lane, left end, right end and whether it is a milestone; a milestone
/// reaches `reach` to either side and may sit at the end or start of a bar, but not inside it, and two
/// milestones may overlap by half.
pub fn rows(items: &[(usize, f32, f32, bool)], reach: f32) -> Vec<usize> {
    let clash = |a: &(usize, f32, f32, bool), b: &(usize, f32, f32, bool)| match (a.3, b.3) {
        (false, false) => a.1 < b.2 - 0.5 && b.1 < a.2 - 0.5,
        (true, true) => (a.1 - b.1).abs() < reach,
        (true, false) => a.1 > b.1 + 0.5 && a.1 < b.2 - 0.5,
        (false, true) => b.1 > a.1 + 0.5 && b.1 < a.2 - 0.5,
    };
    let mut order: Vec<usize> = (0..items.len()).collect();
    order.sort_by(|&a, &b| (items[a].1 - if items[a].3 { reach } else { 0.0 }).total_cmp(&(items[b].1 - if items[b].3 { reach } else { 0.0 })));
    let mut rows = vec![0; items.len()];
    let mut placed: Vec<usize> = Vec::new();
    for index in order {
        rows[index] = (0..).find(|&row| !placed.iter().any(|&other| items[other].0 == items[index].0 && rows[other] == row && clash(&items[index], &items[other]))).unwrap_or(0);
        placed.push(index);
    }
    rows
}

pub fn days(from: Date, to: Date) -> i32 {
    from.until(to).map(|span| span.get_days()).unwrap_or(0)
}
