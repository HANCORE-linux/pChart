//! The weeks of people across the projects of a workspace (pictures 64): a row for each person with
//! their work from every project and the share of their time each takes, in yellow where together it is
//! more than they can do, hatched where they are away, and under it how busy they are each week.

use jiff::civil::{Date, Weekday};

use crate::scene::{Anchor, Color, Fill, Scene, Shape, Stroke};
use crate::style::{ADVANCE, FONT, Palette};

/// A piece of work of a person: its name, its project, its days and the share of their time it takes,
/// in percent.
#[derive(Debug, Clone, PartialEq)]
pub struct Work {
    pub name: String,
    pub project: String,
    pub start: Date,
    pub finish: Date,
    pub share: u32,
    /// Weekdays within its dates that are not worked where it is done: holidays.
    pub off: Vec<Date>,
}

/// A person: their name, their team, their work, the days they are away, and how much of their time
/// they can work, in percent.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub name: String,
    pub team: String,
    pub work: Vec<Work>,
    pub away: Vec<(Date, Date)>,
    pub capacity: u32,
}

fn weekday(day: Date) -> bool {
    !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday)
}

impl Row {
    fn away_on(&self, day: Date) -> bool {
        self.away.iter().any(|&(from, to)| from <= day && day <= to)
    }

    /// How much of their time is planned on `day`, in percent.
    pub fn load(&self, day: Date) -> u32 {
        self.work.iter().filter(|work| work.start <= day && day <= work.finish && !work.off.contains(&day)).map(|work| work.share).sum()
    }

    /// How full the week from `monday` is, in percent of what they can do: 100 is full.
    pub fn full(&self, monday: Date) -> u32 {
        self.week(monday) * 100 / self.capacity.max(1)
    }

    /// The most of their time planned on a weekday of the week from `monday`, in percent.
    pub fn week(&self, monday: Date) -> u32 {
        (0..5).map(|offset| (0..offset).fold(monday, |day, _| next(day))).filter(|&day| !self.away_on(day)).map(|day| self.load(day)).max().unwrap_or(0)
    }

    /// The stretches of weekdays they are not away on where more than they can do is planned.
    pub fn over(&self) -> Vec<(Date, Date)> {
        let (Some(first), Some(last)) = (self.work.iter().map(|work| work.start).min(), self.work.iter().map(|work| work.finish).max()) else { return Vec::new() };
        let mut out: Vec<(Date, Date)> = Vec::new();
        let mut open = false;
        let mut day = first;
        while day <= last {
            if weekday(day) {
                let over = !self.away_on(day) && self.load(day) > self.capacity;
                match (over, open, out.last_mut()) {
                    (true, true, Some(stretch)) => stretch.1 = day,
                    (true, _, _) => out.push((day, day)),
                    _ => {}
                }
                open = over;
            }
            day = next(day);
        }
        out
    }
}

fn days(from: Date, to: Date) -> i32 {
    from.until(to).map(|span| span.get_days()).unwrap_or(0)
}

fn next(day: Date) -> Date {
    day.tomorrow().expect("a date in range")
}

fn fit(text: &str, size: f32, room: f32) -> String {
    let count = (room / (size * ADVANCE)).floor().max(0.0) as usize;
    if text.chars().count() <= count { text.to_string() } else { text.chars().take(count.saturating_sub(1)).collect::<String>() + "…" }
}

/// The name of a piece of work with its share when it takes less than all of the time: "UX concept 80 %".
fn label(work: &Work) -> String {
    if work.share == 100 { work.name.clone() } else { format!("{} {} %", work.name, work.share) }
}

/// The rows of bars each piece of work of `row` sits in, so that neither bars nor names cover each other.
fn stack(row: &Row, x: &dyn Fn(Date) -> f32, label: &dyn Fn(&Work) -> f32) -> Vec<usize> {
    let mut ends: Vec<f32> = Vec::new();
    row.work
        .iter()
        .map(|work| {
            let (x0, x1) = (x(work.start), x(next(work.finish)).max(x(work.start) + label(work)));
            let at = ends.iter().position(|&end| end <= x0).unwrap_or(ends.len());
            if at == ends.len() {
                ends.push(x1);
            } else {
                ends[at] = x1;
            }
            at
        })
        .collect()
}

/// The height of the picture of `rows` at `unit`.
pub fn height(rows: &[Row], unit: f32, width: f32, start: Date, end: Date) -> f32 {
    let geometry = Geometry::new(width, unit, start, end);
    geometry.top + 52.0 * unit + rows.iter().map(|row| geometry.row_h(row)).sum::<f32>()
}

struct Geometry {
    start: Date,
    top: f32,
    x0: f32,
    x1: f32,
    px: f32,
    u: f32,
    /// How busy each week is, under each person.
    numbers: bool,
}

impl Geometry {
    fn new(width: f32, u: f32, start: Date, end: Date) -> Geometry {
        let x0 = if width >= 1000.0 * u { 220.0 * u } else { 160.0 * u };
        Geometry::between(x0, width - 8.0 * u, u, start, end)
    }

    fn between(x0: f32, x1: f32, u: f32, start: Date, end: Date) -> Geometry {
        Geometry { start, top: 26.0 * u, x0, x1, px: (x1 - x0) / (days(start, end) + 1) as f32, u, numbers: true }
    }

    fn x(&self, day: Date) -> f32 {
        (self.x0 + days(self.start, day) as f32 * self.px).clamp(self.x0, self.x1)
    }

    fn label(&self, work: &Work) -> f32 {
        (label(work).chars().count() + 2 + work.project.chars().count()) as f32 * 13.5 * self.u * ADVANCE + 10.0 * self.u
    }

    fn row_h(&self, row: &Row) -> f32 {
        let levels = stack(row, &|day| self.x(day), &|work| self.label(work)).into_iter().max().map_or(1, |level| level + 1);
        if self.numbers { (48.0 + 36.0 * levels as f32).max(84.0) * self.u } else { (24.0 + 36.0 * levels as f32).max(64.0) * self.u }
    }
}

/// The people of `rows` from `start` to `end`, with the line of `today` and the person `chosen` on the
/// raised band with the pink edge.
#[allow(clippy::too_many_arguments)]
pub fn people(rows: &[Row], start: Date, end: Date, today: Date, chosen: Option<usize>, width: f32, height: f32, unit: f32, palette: &Palette) -> Scene {
    draw(rows, start, end, today, chosen, Geometry::new(width, unit, start, end), width, height, palette)
}

/// The work of the people of `rows` over `weeks` weeks from `start`, in the columns of the capacity
/// table, without how busy each week is, which the table says.
#[allow(clippy::too_many_arguments)]
pub fn work_under(rows: &[Row], start: Date, weeks: usize, today: Date, width: f32, height: f32, unit: f32, palette: &Palette) -> Scene {
    let end = (1..weeks * 7).fold(start, |day, _| next(day));
    let x0 = if width >= 1000.0 * unit { 240.0 * unit } else { 170.0 * unit };
    let mut g = Geometry::between(x0, x0 + weeks as f32 * week_width(unit), unit, start, end);
    g.numbers = false;
    draw(rows, start, end, today, None, g, width, height, palette)
}

/// The height of the work of `rows` under the capacity table.
pub fn work_height(rows: &[Row], unit: f32, weeks: usize, width: f32, start: Date) -> f32 {
    let end = (1..weeks.max(1) * 7).fold(start, |day, _| next(day));
    let x0 = if width >= 1000.0 * unit { 240.0 * unit } else { 170.0 * unit };
    let mut g = Geometry::between(x0, x0 + weeks as f32 * week_width(unit), unit, start, end);
    g.numbers = false;
    g.top + 52.0 * unit + rows.iter().map(|row| g.row_h(row)).sum::<f32>()
}

#[allow(clippy::too_many_arguments)]
fn draw(rows: &[Row], start: Date, end: Date, today: Date, chosen: Option<usize>, g: Geometry, width: f32, height: f32, palette: &Palette) -> Scene {
    let (p, u) = (*palette, g.u);
    let mut s = Scene { width, height, background: None, hatch: (p.raised, p.muted), hatch_alert: (p.alert_ground, p.alert), spare: p.dim, font: FONT, marks: Vec::new() };
    let rule = |color: Color| Stroke { color, width: 1.0, dash: None };
    let text = |x: f32, y: f32, value: String, size: f32, color: Color, bold: bool| Shape::Text { x, y, text: value, size: size * u, color, bold, anchor: Anchor::Start };
    let top = g.top;
    let mut tops = vec![top + 52.0 * u];
    for row in rows {
        tops.push(tops[tops.len() - 1] + g.row_h(row));
    }
    let bottom = tops[tops.len() - 1].min(height);

    if let Some(chosen) = chosen.filter(|&chosen| chosen < rows.len()) {
        s.add(Shape::Rect { x: 0.0, y: tops[chosen], w: width, h: tops[chosen + 1] - tops[chosen], radius: 0.0, fill: Fill::Color(p.raised), stroke: None });
        s.add(Shape::Rect { x: 0.0, y: tops[chosen], w: 4.0 * u, h: tops[chosen + 1] - tops[chosen], radius: 0.0, fill: Fill::Color(p.accent), stroke: None });
    }
    let today_x = (start..=end).contains(&today).then(|| g.x(today) + g.px / 2.0);
    let today_w = 5.0 * 13.0 * u * ADVANCE;
    let clear = |from: f32, to: f32| today_x.is_none_or(|t| to < t - today_w / 2.0 - 8.0 * u || from > t + today_w / 2.0 + 8.0 * u);
    let mut month = start.first_of_month();
    let mut labelled = false;
    while month <= end {
        let following = next(month.last_of_month());
        let xa = g.x(month.max(start));
        if month >= start {
            s.add(Shape::Line { x1: xa, y1: top, x2: xa, y2: bottom, stroke: rule(p.rule) });
        }
        let label = if !labelled || month.month() == 1 { month.strftime("%b %Y").to_string() } else { month.strftime("%b").to_string() };
        let label_w = label.chars().count() as f32 * 14.5 * u * ADVANCE;
        if g.x(following.min(end)) - xa > label_w + 12.0 * u && clear(xa + 6.0 * u, xa + 6.0 * u + label_w) {
            s.add(text(xa + 6.0 * u, top + 14.0 * u, label, 14.5, p.ink, false));
            labelled = true;
        }
        month = following;
    }
    let mut monday = start;
    let mut count = 0;
    while monday <= end {
        let week = format!("{}{:02}", if count == 0 { "W" } else { "" }, monday.iso_week_date().week());
        let x = g.x(monday) + 2.0 * u;
        if x + week.chars().count() as f32 * 12.5 * u * ADVANCE <= g.x1 {
            s.add(text(x, top + 33.0 * u, week, 12.5, p.muted, false));
        }
        count += 1;
        monday = (0..7).fold(monday, |day, _| next(day));
    }
    s.add(Shape::Line { x1: g.x0, y1: top + 40.0 * u, x2: g.x1, y2: top + 40.0 * u, stroke: rule(p.rule) });

    let room = g.x0 - 24.0 * u;
    for (index, row) in rows.iter().enumerate() {
        let y = tops[index];
        if y >= height {
            break;
        }
        if index > 0 {
            s.add(Shape::Line { x1: 6.0 * u, y1: y, x2: g.x1, y2: y, stroke: rule(p.rule) });
        }
        let name = fit(&row.name, 16.6 * u, room);
        let after = 14.0 * u + (name.chars().count() + 1) as f32 * 16.6 * u * ADVANCE;
        s.add(text(14.0 * u, y + 31.0 * u, name, 16.6, if chosen == Some(index) { p.bright } else { p.ink }, chosen == Some(index)));
        if row.capacity != 100 && after + 5.0 * 14.0 * u * ADVANCE <= g.x0 - 10.0 * u {
            s.add(text(after, y + 31.0 * u, format!("{} %", row.capacity), 14.0, p.dim, false));
        }
        s.add(text(14.0 * u, y + 52.0 * u, fit(&row.team, 14.0 * u, room), 14.0, p.muted, false));
        let levels = stack(row, &|day| g.x(day), &|work| g.label(work));
        let overlaps = row.over();
        for (work, level) in row.work.iter().zip(levels) {
            if work.finish < start || work.start > end {
                continue;
            }
            let bar_y = y + 40.0 * u + level as f32 * 36.0 * u;
            let (x0, x1) = (g.x(work.start), g.x(next(work.finish)));
            s.marks.push(crate::scene::Mark { shape: Shape::Rect { x: x0 + u, y: bar_y - 7.0 * u, w: (x1 - x0 - 2.5 * u).max(2.0 * u), h: 14.0 * u, radius: 3.0 * u, fill: Fill::Color(p.dim), stroke: None }, opacity: 0.55, item: None });
            for &(from, to) in overlaps.iter().filter(|(from, to)| *from <= work.finish && work.start <= *to) {
                let (a, b) = (g.x(from.max(work.start)), g.x(next(to.min(work.finish))));
                s.add(Shape::Rect { x: a + u, y: bar_y - 7.0 * u, w: (b - a - 2.5 * u).max(2.0 * u), h: 14.0 * u, radius: 3.0 * u, fill: Fill::Color(p.warn), stroke: None });
            }
            let label_room = g.x1 - x0 - 4.0 * u;
            let name = fit(&work.name, 13.5 * u, label_room);
            let mut end_x = x0 + name.chars().count() as f32 * 13.5 * u * ADVANCE;
            s.add(text(x0, bar_y - 13.0 * u, name, 13.5, p.ink, false));
            if work.share != 100 && end_x + 7.0 * 13.5 * u * ADVANCE <= g.x1 {
                let share = format!("{} %", work.share);
                s.add(text(end_x + 13.5 * u * ADVANCE, bar_y - 13.0 * u, share.clone(), 13.5, p.ink, true));
                end_x += (share.chars().count() + 1) as f32 * 13.5 * u * ADVANCE;
            }
            let project = fit(&work.project, 13.5 * u, g.x1 - end_x - 14.0 * u);
            if !project.is_empty() && project != "…" {
                s.add(text(end_x + 13.5 * u * ADVANCE * 2.0, bar_y - 13.0 * u, project, 13.5, p.muted, false));
            }
        }
    }
    for (index, row) in rows.iter().enumerate() {
        let y = tops[index];
        if y >= height {
            break;
        }
        let base = tops[index + 1] - 12.0 * u;
        let mut monday = start;
        while g.numbers && monday <= end {
            let (load, full) = (row.week(monday), row.full(monday));
            let words = format!("{full} %");
            let x = g.x(monday) + 2.0 * u;
            if load > 0 && x + words.chars().count() as f32 * 12.5 * u * ADVANCE <= (g.x(monday) + 7.0 * g.px).min(g.x1) - 4.0 * u {
                let over = load > row.capacity;
                s.add(text(x, base, words, 12.5, if over { p.warn } else { p.dim }, over));
            }
            monday = (0..7).fold(monday, |day, _| next(day));
        }
        let bands = stack(row, &|day| g.x(day), &|work| g.label(work)).into_iter().max().map_or(1, |level| level + 1);
        for &(from, to) in &row.away {
            let mut day = from.max(start);
            while day <= to.min(end) {
                if !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday) {
                    for level in 0..bands {
                        let bar_y = y + 40.0 * u + level as f32 * 36.0 * u;
                        s.add(Shape::Rect { x: g.x(day), y: bar_y - 8.0 * u, w: g.px, h: 16.0 * u, radius: 0.0, fill: Fill::Hatch, stroke: None });
                    }
                }
                day = next(day);
            }
        }
    }
    if let Some(x) = today_x {
        s.add(Shape::Line { x1: x, y1: top + 40.0 * u, x2: x, y2: bottom, stroke: Stroke { color: p.ink, width: 1.4 * u, dash: None } });
        s.add(Shape::Text { x, y: top + 14.0 * u, text: "today".into(), size: 13.0 * u, color: p.bright, bold: true, anchor: Anchor::Middle });
    }
    s
}

/// A line of the capacity table: a team with the rows of its people, or one person by their row.
#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Team(String, Vec<usize>),
    Person(usize),
}

/// How busy the team of `members` is in the week from `monday`: the most planned on a weekday against
/// what those of them who are there can do, in percent of that, with the planned and the capacity.
pub fn team_week(rows: &[Row], members: &[usize], monday: Date) -> (u32, u32) {
    (0..5)
        .map(|offset| (0..offset).fold(monday, |day, _| next(day)))
        .map(|day| {
            let there: Vec<&Row> = members.iter().map(|&at| &rows[at]).filter(|row| !row.away_on(day)).collect();
            (there.iter().map(|row| row.load(day)).sum::<u32>(), there.iter().map(|row| row.capacity).sum::<u32>())
        })
        .max_by_key(|&(planned, capacity)| (planned * 1000) / capacity.max(1))
        .unwrap_or((0, 0))
}

/// Whether the person of `row` is away on every weekday of the week from `monday`, on some, or on none.
fn away_in(row: &Row, monday: Date) -> (bool, bool) {
    let days: Vec<bool> = (0..5).map(|offset| row.away_on((0..offset).fold(monday, |day, _| next(day)))).collect();
    (days.iter().all(|&away| away), days.iter().any(|&away| away))
}

/// The height of the capacity table of `lines`.
pub fn grid_height(lines: &[Line], unit: f32) -> f32 {
    (78.0 + lines.iter().map(|line| if matches!(line, Line::Team(..)) { 40.0 } else { 30.0 }).sum::<f32>()) * unit
}

/// The width of a week in the capacity table.
pub fn week_width(unit: f32) -> f32 {
    74.0 * unit
}

/// The capacity table (picture 64): a row for each team and under it its people, a column for each week
/// from `weeks`, and in each cell how full the week is on its busiest day, in percent of what they can
/// do, shaded by how full it is, yellow over 100 and hatched where they are away; the cell `chosen` (line, week) is
/// ringed in pink.
#[allow(clippy::too_many_arguments)]
pub fn grid(rows: &[Row], lines: &[Line], weeks: &[Date], today: Date, chosen: (usize, usize), width: f32, height: f32, unit: f32, palette: &Palette) -> Scene {
    let (p, u) = (*palette, unit);
    let mut s = Scene { width, height, background: None, hatch: (p.raised, p.muted), hatch_alert: (p.alert_ground, p.alert), spare: p.dim, font: FONT, marks: Vec::new() };
    let text = |x: f32, y: f32, value: String, size: f32, color: Color, bold: bool, anchor: Anchor| Shape::Text { x, y, text: value, size: size * u, color, bold, anchor };
    let x0 = if width >= 1000.0 * u { 240.0 * u } else { 170.0 * u };
    let cw = week_width(u);
    let top = 26.0 * u;
    let this = {
        let mut day = today;
        while day.weekday() != Weekday::Monday {
            day = day.yesterday().expect("a date in range");
        }
        day
    };
    let mut free_from = 0.0;
    for (column, &monday) in weeks.iter().enumerate() {
        let x = x0 + column as f32 * cw;
        if x + cw > width {
            break;
        }
        if column == 0 || monday.day() <= 7 {
            let label = if column == 0 || monday.month() == 1 { monday.strftime("%b %Y").to_string() } else { monday.strftime("%b").to_string() };
            if x + 6.0 * u >= free_from {
                free_from = x + 6.0 * u + (label.chars().count() + 1) as f32 * 14.5 * u * ADVANCE;
                s.add(text(x + 6.0 * u, top + 14.0 * u, label, 14.5, p.ink, false, Anchor::Start));
            }
            if column > 0 {
                s.add(Shape::Line { x1: x, y1: top, x2: x, y2: top + 40.0 * u, stroke: Stroke { color: p.rule, width: 1.0, dash: None } });
            }
        }
        let now = monday == this;
        s.add(text(x + cw / 2.0, top + 34.0 * u, if now { "today".into() } else { format!("W{:02}", monday.iso_week_date().week()) }, 12.5, if now { p.bright } else { p.muted }, now, Anchor::Middle));
    }
    s.add(Shape::Line { x1: 6.0 * u, y1: top + 42.0 * u, x2: width - 6.0 * u, y2: top + 42.0 * u, stroke: Stroke { color: p.rule, width: 1.0, dash: None } });
    let mut y = top + 52.0 * u;
    let room = x0 - 24.0 * u;
    for (index, line) in lines.iter().enumerate() {
        let h = if matches!(line, Line::Team(..)) { 40.0 * u } else { 30.0 * u };
        if y + h > height {
            break;
        }
        let picked = chosen.0 == index;
        if picked {
            s.add(Shape::Rect { x: 0.0, y, w: width, h: h - 2.0 * u, radius: 0.0, fill: Fill::Color(p.raised), stroke: None });
            s.add(Shape::Rect { x: 0.0, y, w: 4.0 * u, h: h - 2.0 * u, radius: 0.0, fill: Fill::Color(p.accent), stroke: None });
        }
        let base = y + h - 10.0 * u;
        match line {
            Line::Team(name, members) => {
                if index > 0 {
                    s.add(Shape::Line { x1: 6.0 * u, y1: y + 2.0 * u, x2: width - 6.0 * u, y2: y + 2.0 * u, stroke: Stroke { color: p.rule, width: 1.0, dash: None } });
                }
                s.add(text(14.0 * u, base, fit(&name.to_uppercase(), 14.0 * u, room), 14.0, if picked { p.bright } else { p.dim }, picked, Anchor::Start));
                for (column, &monday) in weeks.iter().enumerate() {
                    let x = x0 + column as f32 * cw;
                    if x + cw > width {
                        break;
                    }
                    let (planned, capacity) = team_week(rows, members, monday);
                    if planned > 0 {
                        let share = planned * 100 / capacity.max(1);
                        s.add(text(x + cw / 2.0, base, format!("{share} %"), 13.5, if planned > capacity { p.warn } else { p.dim }, planned > capacity, Anchor::Middle));
                    }
                }
            }
            Line::Person(at) => {
                let row = &rows[*at];
                let name = fit(&row.name, 16.6 * u, room - 12.0 * u);
                let after = 26.0 * u + (name.chars().count() + 1) as f32 * 16.6 * u * ADVANCE;
                s.add(text(26.0 * u, base, name, 16.6, if picked { p.bright } else { p.ink }, picked, Anchor::Start));
                if row.capacity != 100 && after + 5.0 * 14.0 * u * ADVANCE <= x0 - 10.0 * u {
                    s.add(text(after, base, format!("{} %", row.capacity), 14.0, p.dim, false, Anchor::Start));
                }
                for (column, &monday) in weeks.iter().enumerate() {
                    let x = x0 + column as f32 * cw;
                    if x + cw > width {
                        break;
                    }
                    let (cell_x, cell_y, cell_w, cell_h) = (x + 3.0 * u, y + 3.0 * u, cw - 6.0 * u, h - 8.0 * u);
                    let (all, some) = away_in(row, monday);
                    let load = row.week(monday);
                    if some {
                        s.add(Shape::Rect { x: cell_x, y: cell_y, w: cell_w, h: cell_h, radius: 2.0 * u, fill: Fill::Hatch, stroke: None });
                    }
                    if all {
                        s.add(text(x + cw / 2.0, base, "away".into(), 13.5, p.dim, false, Anchor::Middle));
                    } else if load > 0 {
                        let over = load > row.capacity;
                        let full = (load as f32 / row.capacity.max(1) as f32).min(1.0);
                        let fill = if over { p.warn } else { p.page.mix(p.dim, f64::from(0.14 + 0.36 * full)) };
                        if !some {
                            s.add(Shape::Rect { x: cell_x, y: cell_y, w: cell_w, h: cell_h, radius: 2.0 * u, fill: Fill::Color(fill), stroke: None });
                        }
                        s.add(text(x + cw / 2.0, base, format!("{} %", row.full(monday)), 13.5, if over && !some { p.page } else if over { p.warn } else { p.bright }, over, Anchor::Middle));
                    }
                }
            }
        }
        if picked {
            let x = x0 + chosen.1 as f32 * cw;
            if x + cw <= width {
                s.add(Shape::Rect { x: x + 2.0 * u, y: y + 2.0 * u, w: cw - 4.0 * u, h: h - 6.0 * u, radius: 3.0 * u, fill: Fill::None, stroke: Some(Stroke { color: p.accent, width: 2.0 * u, dash: None }) });
            }
        }
        y += h;
    }
    s
}
