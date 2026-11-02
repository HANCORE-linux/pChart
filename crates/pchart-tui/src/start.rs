//! The start (picture 54): `pchart` without a plan. Pick a template or a plan opened before, give the
//! new plan its name and its target or start date, and Enter writes it to the folder pChart was started
//! in and opens it.

use std::path::PathBuf;

use jiff::civil::Date;
use pchart_core::analysis::State;
use pchart_core::{Calendar, Holidays, Plan, Schedule};
use pchart_draw::bits::{self, Icon};
use pchart_draw::scene::Shape;
use pchart_draw::style::ADVANCE;
use pchart_draw::{Level, Palette, Scene, Settings};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Span;

use crate::pictures::Placement;
use crate::ui::{Inks, bold, home, panel, put};

pub struct Start {
    pub templates: Vec<Plan>,
    /// Plans opened before, or in a workspace its projects: where they are, the plan, and the
    /// calendars of its lanes.
    pub recent: Vec<(PathBuf, Plan, Vec<Calendar>)>,
    /// The workspace pChart was started in, by name: its projects come first, before the templates.
    pub workspace: Option<String>,
    /// The line `↑↓` is on: the templates, then the plans opened before; in a workspace its projects,
    /// then the templates.
    pub chosen: usize,
    pub name: String,
    /// Scheduled forwards from a start date instead of backwards from a target date.
    pub forwards: bool,
    /// Only how it is built: the work without lengths, so the plan has no dates yet.
    pub undated: bool,
    pub typed: String,
    /// The field typing goes to: the date, or the name.
    pub naming: bool,
    pub dir: PathBuf,
    pub today: Date,
    pub palette: Palette,
    pub cell: (u16, u16),
    pub message: Option<String>,
}

impl Start {
    pub fn new(dir: PathBuf, recent: Vec<PathBuf>, calendars: &dyn Fn(&Plan) -> Vec<Calendar>, today: Date, palette: Palette, cell: (u16, u16)) -> Start {
        let templates = pchart_store::templates().into_iter().map(|(_, plan)| plan).collect();
        let recent = recent.into_iter().filter_map(|path| pchart_store::load(&path).ok().map(|plan| (path, calendars(&plan), plan)).map(|(path, calendars, plan)| (path, plan, calendars))).collect();
        Start { templates, recent, workspace: None, chosen: 0, name: String::new(), forwards: false, undated: false, typed: String::new(), naming: false, dir, today, palette, cell, message: None }
    }

    pub fn template(&self) -> Option<&Plan> {
        self.templates.get(self.template_at()?)
    }

    /// Which template the chosen line is, if it is one.
    pub fn template_at(&self) -> Option<usize> {
        let at = if self.workspace.is_some() { self.chosen.checked_sub(self.recent.len())? } else { self.chosen };
        (at < self.templates.len()).then_some(at)
    }

    /// Which plan opened before, or which project, the chosen line is, if it is one.
    pub fn plan_at(&self) -> Option<usize> {
        let at = if self.workspace.is_some() { self.chosen } else { self.chosen.checked_sub(self.templates.len())? };
        (at < self.recent.len()).then_some(at)
    }

    pub fn choose(&mut self, step: isize) {
        let count = (self.templates.len() + self.recent.len()) as isize;
        self.chosen = (self.chosen as isize + step).rem_euclid(count) as usize;
    }

    /// The date as typed, understood.
    /// `←→`: from a target date, from a start date, or without dates.
    pub fn from(&mut self, step: isize) {
        let at = (if self.undated { 2 } else { isize::from(self.forwards) } + step).rem_euclid(3);
        (self.forwards, self.undated) = (at == 1, at == 2);
    }

    pub fn date(&self) -> Option<Date> {
        pchart_core::input::date(&self.typed, self.today)
    }

    /// The name the new plan gets: as typed, else the template's.
    pub fn name(&self) -> String {
        let typed = self.name.trim();
        if typed.is_empty() { self.template().map_or_else(String::new, |plan| plan.name.clone()) } else { typed.to_string() }
    }

    /// The plan Enter would write: the template with its name and its date. Without a date it starts today.
    pub fn plan(&self) -> Option<Plan> {
        let plan = self.template()?.new_plan(&self.name(), self.date().filter(|_| !self.undated), self.forwards);
        Some(if self.undated { plan.structure_only() } else { plan })
    }

    /// Where the new plan is written: its name in small letters, numbered when the file is there.
    pub fn file(&self) -> PathBuf {
        let base = pchart_core::model::slug(&self.name()).unwrap_or_else(|| "plan".into());
        (1..).map(|number| self.dir.join(if number == 1 { format!("{base}.pchart") } else { format!("{base}-{number}.pchart") })).find(|path| !path.exists()).unwrap_or_default()
    }

    /// Enter: writes the new plan and says where, or names the plan opened before.
    pub fn enter(&mut self) -> Option<PathBuf> {
        if let Some((path, _, _)) = self.plan_at().and_then(|at| self.recent.get(at)) {
            return Some(path.clone());
        }
        if !self.typed.trim().is_empty() && self.date().is_none() {
            self.message = Some("Type a date, such as 1 mar or W9".into());
            return None;
        }
        let plan = self.plan()?;
        let path = self.file();
        match pchart_store::create(&path, &plan) {
            Ok(()) => Some(path),
            Err(err) => {
                self.message = Some(err.to_string());
                None
            }
        }
    }
}

fn calendars(plan: &Plan) -> Vec<Calendar> {
    let holidays = Holidays::parse("");
    plan.lanes.iter().map(|_| holidays.calendar(None)).collect()
}

fn weeks(schedule: &Schedule) -> i64 {
    let first = schedule.slots.iter().map(|slot| slot.planned.start).min();
    let last = schedule.slots.iter().map(|slot| slot.planned.finish).max();
    match (first, last) {
        (Some(first), Some(last)) => (first.until(last).map(|span| span.get_days()).unwrap_or(0) as i64 + 6) / 7,
        _ => 0,
    }
}

fn picture(start: &Start, placements: &mut Vec<Placement>, rect: Rect, scene: impl FnOnce(f32, f32, f32) -> pchart_draw::Scene) {
    let (w, h) = (f32::from(rect.width) * f32::from(start.cell.0), f32::from(rect.height) * f32::from(start.cell.1));
    placements.push(Placement { rect, svg: scene(w, h, f32::from(start.cell.1) / 21.0).svg(), paper: false });
}

/// Draws the start and returns the pictures it needs.
pub fn draw(start: &Start, frame: &mut Frame) -> Vec<Placement> {
    let area = frame.area();
    let p = start.palette;
    let inks = Inks::of(p);
    let mut placements = Vec::new();
    let version: String = env!("CARGO_PKG_VERSION").split('.').take(2).collect::<Vec<_>>().join(".");
    put(frame, 0, 0, area.width, vec![Span::styled("pchart", bold(inks.bright)), Span::styled(format!(" {version} · new plan"), inks.dim)]);
    let right = if start.template_at().is_some() { "choose a template" } else { "open a plan" };
    put(frame, area.width.saturating_sub(right.len() as u16 + 1), 0, right.len() as u16 + 1, vec![Span::styled(right, inks.dim)]);
    let body = Rect { x: 0, y: 1, width: area.width, height: area.height.saturating_sub(2) };
    let left = Rect { width: (area.width * 30 / 100).clamp(34, 48).min(area.width), ..body };
    list(start, frame, &inks, left, &mut placements);
    let rest = Rect { x: left.width, width: body.width.saturating_sub(left.width), ..body };
    if rest.width >= 40 && rest.height >= 30 {
        let top = Rect { height: rest.height - 12, ..rest };
        preview(start, frame, &inks, top, &mut placements);
        how_to_start(start, frame, &inks, Rect { y: top.y + top.height, height: 12, ..rest }, &mut placements);
    } else if rest.width >= 40 {
        how_to_start(start, frame, &inks, rest, &mut placements);
    }
    let hints: Vec<(&str, &str)> = if start.template_at().is_some() {
        let mut hints = vec![("↑↓", "template"), ("←→", "target, start or none"), ("Tab", if start.naming { "date" } else { "name" }), ("Enter", "create"), ("Esc", "quit")];
        if start.undated {
            hints.remove(2);
        }
        hints
    } else {
        vec![("↑↓", "plan"), ("Enter", "open"), ("Esc", "quit")]
    };
    let mut spans = Vec::new();
    for (key, label) in hints {
        spans.extend([Span::styled(key, bold(inks.ink)), Span::styled(format!(" {label}"), inks.dim), Span::raw("    ")]);
    }
    if let Some(message) = &start.message {
        spans.push(Span::styled(message.clone(), inks.warn));
    }
    put(frame, 1, area.height.saturating_sub(1), area.width.saturating_sub(1), spans);
    placements
}

fn list(start: &Start, frame: &mut Frame, inks: &Inks, rect: Rect, placements: &mut Vec<Placement>) {
    let p = start.palette;
    let title = start.workspace.as_ref().map_or_else(|| "START".to_string(), |name| format!("START · {}", name.to_uppercase()));
    panel(frame, inks, rect, &title, true);
    let bottom = rect.y + rect.height.saturating_sub(1);
    let mut y = rect.y + 3;
    let plans = |y: &mut u16, frame: &mut Frame, placements: &mut Vec<Placement>| {
        put(frame, rect.x + 3, *y, rect.width.saturating_sub(6), vec![Span::styled(if start.workspace.is_some() { "PROJECTS" } else { "RECENT" }, inks.dim)]);
        *y += 2;
        if start.recent.is_empty() {
            put(frame, rect.x + 3, *y, rect.width.saturating_sub(6), vec![Span::styled(if start.workspace.is_some() { "no projects yet" } else { "no plans yet" }, inks.muted)]);
            *y += 2;
        }
        for (at, (path, plan, calendars)) in start.recent.iter().enumerate() {
            let chosen = start.plan_at() == Some(at);
            let height = if chosen { 3 } else { 2 };
            if *y + height > bottom {
                break;
            }
            let (icon, tone) = match start.workspace.as_ref().and_then(|_| verdict(plan, calendars, start.today)) {
                Some(State::Holds) => (Icon::Check, p.dim),
                Some(State::Tight) => (Icon::Clock, p.warn),
                Some(State::DoesNotHold) => (Icon::Cross, p.alert),
                None => (Icon::Folder, p.dim),
            };
            let place = home(&path.display().to_string());
            let rect = Rect { x: rect.x + 1, y: *y, width: rect.width.saturating_sub(2), height };
            picture(start, placements, rect, |w, h, u| bits::row(icon, tone, &plan.name, chosen.then_some(place.as_str()), chosen, w, h, u, &p));
            *y += height;
        }
        *y += 1;
    };
    if start.workspace.is_some() {
        plans(&mut y, frame, placements);
    }
    if y + 2 > bottom {
        return;
    }
    put(frame, rect.x + 3, y, rect.width.saturating_sub(6), vec![Span::styled(if start.workspace.is_some() { "NEW PROJECT FROM" } else { "PROJECT TEMPLATES" }, inks.dim)]);
    y += 2;
    for (index, template) in start.templates.iter().enumerate() {
        let chosen = start.template_at() == Some(index);
        let height = if chosen { 3 } else { 2 };
        if y + height > bottom {
            break;
        }
        let items = template.items.iter().filter(|item| !item.milestone).count();
        let milestones = template.items.len() - items;
        let lanes = if template.lanes.len() == 1 { "1 lane".to_string() } else { format!("{} lanes", template.lanes.len()) };
        let place = format!("{lanes} · {items} items · {milestones} milestones");
        let rect = Rect { x: rect.x + 1, y, width: rect.width.saturating_sub(2), height };
        picture(start, placements, rect, |w, h, u| bits::row(Icon::Bar, p.dim, &template.name, chosen.then_some(place.as_str()), chosen, w, h, u, &p));
        y += height;
    }
    y += 1;
    if start.workspace.is_none() && y + 2 <= bottom {
        plans(&mut y, frame, placements);
    }
}

/// How a plan stands: holds, at risk or late; nothing when it has no fixed date.
fn verdict(plan: &Plan, calendars: &[Calendar], today: Date) -> Option<State> {
    let schedule = pchart_core::schedule(plan, calendars, today).ok()?;
    pchart_core::analyse(plan, calendars, &schedule, today).verdict.map(|verdict| verdict.state)
}

/// The plan the preview shows, with the calendars of its lanes.
fn shown(start: &Start) -> Option<(Plan, Vec<Calendar>)> {
    match start.plan() {
        Some(plan) => {
            let calendars = calendars(&plan);
            Some((plan, calendars))
        }
        None => start.plan_at().and_then(|at| start.recent.get(at)).map(|(_, plan, calendars)| (plan.clone(), calendars.clone())),
    }
}

fn preview(start: &Start, frame: &mut Frame, inks: &Inks, rect: Rect, placements: &mut Vec<Placement>) {
    let Some((plan, calendars)) = shown(start) else { return };
    let Ok(schedule) = pchart_core::schedule(&plan, &calendars, start.today) else { return };
    if !plan.without_length().is_empty() {
        panel(frame, inks, rect, &format!("PREVIEW · {} · PROJECT STRUCTURE", plan.name.to_uppercase()), false);
        let chart = Rect { x: rect.x + 2, y: rect.y + 2, width: rect.width.saturating_sub(4), height: rect.height.saturating_sub(3) };
        if chart.width < 20 || chart.height < 6 {
            return;
        }
        let (cw, ch) = start.cell;
        let page = pchart_draw::structure(&plan, &schedule, start.today);
        let svg = bits::sheet(&page, f32::from(chart.width) * f32::from(cw), f32::from(chart.height) * f32::from(ch), f32::from(ch) / 21.0);
        placements.push(Placement { rect: chart, svg, paper: true });
        return;
    }
    let title = format!("PREVIEW · {} · {} WEEKS", plan.name.to_uppercase(), weeks(&schedule));
    panel(frame, inks, rect, &title, false);
    let chart = Rect { x: rect.x + 1, y: rect.y + 2, width: rect.width.saturating_sub(2), height: rect.height.saturating_sub(3) };
    if chart.width < 20 || chart.height < 6 {
        return;
    }
    let (cw, ch) = start.cell;
    let settings = Settings {
        width: f32::from(chart.width) * f32::from(cw),
        height: f32::from(chart.height) * f32::from(ch),
        unit: f32::from(ch) / 21.0,
        today: start.today,
        selected: None,
        level: Level::Standard,
        palette: start.palette,
        cursor: None,
    };
    let mut scene = pchart_draw::roadmap(&plan, &schedule, &calendars, &settings);
    scene.background = None;
    placements.push(Placement { rect: chart, svg: scene.svg(), paper: false });
}

fn how_to_start(start: &Start, frame: &mut Frame, inks: &Inks, rect: Rect, placements: &mut Vec<Placement>) {
    let p = start.palette;
    if let Some((path, plan, calendars)) = start.plan_at().and_then(|at| start.recent.get(at)) {
        panel(frame, inks, rect, "OPEN", true);
        let inner = Rect { x: rect.x + 2, y: rect.y + 2, width: rect.width.saturating_sub(4), height: rect.height.saturating_sub(3) };
        let verdict = pchart_core::schedule(plan, calendars, start.today).ok().and_then(|schedule| {
            let analysis = pchart_core::analyse(plan, calendars, &schedule, start.today);
            analysis.verdict.map(|verdict| (verdict.state, plan.items[verdict.target].name.clone(), schedule.slots[verdict.target].planned.finish, verdict.spare))
        });
        let changed = std::fs::metadata(path).and_then(|meta| meta.modified()).ok().and_then(|time| jiff::Timestamp::try_from(time).ok()).map(|time| time.to_zoned(jiff::tz::TimeZone::system()).strftime("%a %d %b, %H:%M").to_string());
        let place = home(&path.display().to_string());
        picture(start, placements, inner, |w, h, u| {
            let mut s = bits::canvas(w, h, &p);
            let size = 16.6 * u;
            let mut y = 22.0 * u;
            if let Some((state, name, date, spare)) = &verdict {
                let (icon, tone) = match state {
                    State::Holds => (Icon::Check, p.bright),
                    State::Tight => (Icon::Clock, p.warn),
                    State::DoesNotHold => (Icon::Cross, p.alert),
                };
                bits::draw_icon(&mut s, icon, 18.0 * u, y - 6.0 * u, 20.0 * u, tone, p.page);
                let words = if *state == State::DoesNotHold { format!("{} wd late", -spare) } else { format!("{} · {spare} wd to spare", pchart_draw::situation::state_words(*state)) };
                bits::runs(&mut s, 38.0 * u, y, 18.0 * u, &[(&format!("{name} on {} is ", date.strftime("%a %d %b")), p.bright, true), (&words, tone, true)]);
                y += 36.0 * u;
            }
            bits::draw_icon(&mut s, Icon::Folder, 18.0 * u, y - 5.0 * u, 15.0 * u, p.dim, p.page);
            bits::runs(&mut s, 38.0 * u, y, size, &[(&place, p.ink, false)]);
            y += 30.0 * u;
            if let Some(changed) = &changed {
                bits::draw_icon(&mut s, Icon::Clock, 18.0 * u, y - 5.0 * u, 15.0 * u, p.dim, p.page);
                bits::runs(&mut s, 38.0 * u, y, size, &[(&format!("changed {changed}"), p.ink, false)]);
            }
            s
        });
        return;
    }
    panel(frame, inks, rect, "HOW TO START", true);
    let inner = Rect { x: rect.x + 2, y: rect.y + 2, width: rect.width.saturating_sub(4), height: rect.height.saturating_sub(3) };
    let plan = start.plan();
    let schedule = plan.as_ref().and_then(|plan| pchart_core::schedule(plan, &calendars(plan), start.today).ok());
    let analysis = plan.as_ref().zip(schedule.as_ref()).map(|(plan, schedule)| pchart_core::analyse(plan, &calendars(plan), schedule, start.today));
    let file = start.file();
    let placeholder = start.template().map_or_else(String::new, |plan| plan.name.clone());
    picture(start, placements, inner, |w, h, u| {
        let mut s = bits::canvas(w, h, &p);
        let size = 16.6 * u;
        let (left, value) = (8.0 * u, 8.0 * u + 9.0 * size * ADVANCE);
        let field = |s: &mut Scene, y: f32, width: f32, text: &str, hint: &str, caret: bool| {
            s.add(Shape::Rect { x: value, y, w: width, h: 27.0 * u, radius: 3.0 * u, fill: pchart_draw::scene::Fill::None, stroke: Some(pchart_draw::scene::Stroke { color: if caret { p.ink } else { p.muted }, width: 1.2 * u, dash: None }) });
            let (shown, ink, x) = if text.is_empty() { (hint, p.muted, value + 16.0 * u) } else { (text, p.bright, value + 10.0 * u) };
            let end = bits::runs(s, x, y + 19.0 * u, size, &[(shown, ink, false)]);
            if caret {
                let x = if text.is_empty() { value + 10.0 * u } else { end };
                s.add(Shape::Rect { x: x + u, y: y + 5.0 * u, w: 2.0 * u, h: 17.0 * u, radius: 0.0, fill: pchart_draw::scene::Fill::Color(p.bright), stroke: None });
            }
        };
        let naming = start.naming || start.undated;
        bits::runs(&mut s, left, 22.0 * u, size, &[("Name", if naming { p.bright } else { p.dim }, naming)]);
        field(&mut s, 3.0 * u, (w - value - 8.0 * u).min(420.0 * u), &start.name, &placeholder, naming);
        bits::runs(&mut s, left, 62.0 * u, size, &[("From", p.dim, false)]);
        let long = value + 29.0 * size * ADVANCE + 96.0 * u <= w;
        let end = bits::chip(&mut s, value, 62.0 * u, if long { "Target date" } else { "Target" }, !start.forwards && !start.undated, false, u, &p);
        let end = bits::chip(&mut s, end + 12.0 * u, 62.0 * u, if long { "Start date" } else { "Start" }, start.forwards, false, u, &p);
        bits::chip(&mut s, end + 12.0 * u, 62.0 * u, "No dates", start.undated, false, u, &p);
        let mut y = 110.0 * u;
        if start.undated {
            bits::draw_icon(&mut s, Icon::Calendar, left + 10.0 * u, y - 6.0 * u, 20.0 * u, p.dim, p.page);
            let words = "Dates come once every piece of work has a length.";
            bits::runs(&mut s, left + 30.0 * u, y, size.min((w - left - 40.0 * u) / (words.chars().count() as f32 * ADVANCE)), &[(words, p.ink, false)]);
            y += 34.0 * u;
        } else {
            bits::runs(&mut s, left, 102.0 * u, size, &[("Date", if start.naming { p.dim } else { p.bright }, !start.naming)]);
            let box_w = 160.0 * u;
            field(&mut s, 83.0 * u, box_w, &start.typed, "such as 1 mar", !start.naming);
            let echo = match start.date() {
                Some(date) => (format!("→  {}", date.strftime("%a %d %b %Y")), p.bright, true),
                None if start.typed.trim().is_empty() => (if start.forwards { "starts today".to_string() } else { "no target yet: it starts today".to_string() }, p.muted, false),
                None => ("not a date yet".to_string(), p.muted, false),
            };
            bits::runs(&mut s, value + box_w + 20.0 * u, 102.0 * u, size, &[(&echo.0, echo.1, echo.2)]);
            y = 150.0 * u;
        }
        if let (false, Some(plan), Some(schedule), Some(analysis)) = (start.undated, &plan, &schedule, &analysis) {
            let first = schedule.first.or_else(|| schedule.order.first().copied());
            let last = (0..plan.items.len()).max_by_key(|&index| schedule.slots[index].planned.finish);
            let (icon, tone, words): (Icon, _, Vec<(String, _, bool)>) = match (&analysis.verdict, start.date().filter(|_| !start.forwards)) {
                (Some(verdict), Some(_)) => {
                    let start_by = first.map(|index| (plan.items[index].name.clone(), schedule.start_by.unwrap_or(schedule.slots[index].latest.start)));
                    match verdict.state {
                        State::DoesNotHold => (Icon::Cross, p.alert, vec![(format!("{} would be ", plan.items[verdict.target].name), p.bright, true), (format!("{} working days late.", -verdict.spare), p.alert, true)]),
                        _ => match start_by {
                            Some((name, day)) => (Icon::Check, p.bright, vec![(format!("{name} must start by "), p.bright, true), (day.strftime("%a %d %b %Y").to_string(), p.bright, true), (".".into(), p.bright, true)]),
                            None => (Icon::Check, p.bright, Vec::new()),
                        },
                    }
                }
                _ => match last {
                    Some(index) => (Icon::Calendar, p.dim, vec![(format!("{} on ", plan.items[index].name), p.bright, true), (schedule.slots[index].planned.finish.strftime("%a %d %b %Y").to_string(), p.bright, true), (".".into(), p.bright, true)]),
                    None => (Icon::Calendar, p.dim, Vec::new()),
                },
            };
            bits::draw_icon(&mut s, icon, left + 10.0 * u, y - 6.0 * u, 20.0 * u, tone, p.page);
            let parts: Vec<(&str, _, bool)> = words.iter().map(|(text, color, bold)| (text.as_str(), *color, *bold)).collect();
            let count = words.iter().map(|(text, _, _)| text.chars().count()).sum::<usize>().max(1) as f32;
            bits::runs(&mut s, left + 30.0 * u, y, (18.0 * u).min((w - left - 40.0 * u) / (count * ADVANCE)), &parts);
            y += 34.0 * u;
        }
        bits::draw_icon(&mut s, Icon::Folder, left + 10.0 * u, y - 5.0 * u, 15.0 * u, p.dim, p.page);
        let shown = format!("{}{}", if start.workspace.is_some() { "projects/" } else { "./" }, file.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default());
        let end = bits::runs(&mut s, left + 30.0 * u, y, size, &[(&shown, p.ink, false)]);
        bits::runs(&mut s, end, y, size, &[("  is written here, then pChart opens it", p.dim, false)]);
        s
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pchart-start-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn start(dir: &std::path::Path, recent: Vec<PathBuf>) -> Start {
        let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
        let calendars = |plan: &Plan| plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        Start::new(dir.to_path_buf(), recent, &calendars, jiff::civil::date(2026, 10, 29), Palette::labra(), (7, 16))
    }

    fn screen(start: &Start, width: u16, height: u16) -> (String, String) {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let mut placements = Vec::new();
        terminal.draw(|frame| placements = draw(start, frame)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        let rows = (0..height).map(|y| (0..width).map(|x| buffer[(x, y)].symbol().to_string()).collect::<String>() + "\n").collect();
        (rows, placements.iter().map(|placement| placement.svg.as_str()).collect())
    }

    #[test]
    fn a_template_becomes_a_plan_in_the_folder_with_its_name_and_its_date() {
        let dir = folder("new");
        let mut start = start(&dir, Vec::new());
        let (rows, svg) = screen(&start, 160, 47);
        for text in [":: START", "PROJECT TEMPLATES", "RECENT", "no plans yet", ":: PREVIEW · NEW APPLICATION · ", ":: HOW TO START", "choose a template", "↑↓ template", "Enter create"] {
            assert!(rows.contains(text), "{text}");
        }
        for text in ["New application", "7 lanes · 11 items · 5 milestones", "Website relaunch", "Empty plan", "no target yet: it starts today", "./new-application.pchart"] {
            assert!(svg.contains(text), "{text}");
        }
        start.naming = true;
        "Customer portal v1".chars().for_each(|c| start.name.push(c));
        start.naming = false;
        start.typed = "soon".into();
        assert!(start.enter().is_none() && start.message.as_deref() == Some("Type a date, such as 1 mar or W9"));
        start.typed = "1 mar".into();
        let (_, svg) = screen(&start, 160, 47);
        assert!(svg.contains("Mon 01 Mar 2027") && svg.contains("Requirements must start by ") && svg.contains("./customer-portal-v1.pchart"));
        std::fs::write(dir.join("customer-portal-v1.pchart"), "").unwrap();
        let path = start.enter().unwrap();
        assert_eq!(path, dir.join("customer-portal-v1-2.pchart"));
        let plan = pchart_store::load(&path).unwrap();
        assert_eq!((plan.name.as_str(), plan.items[plan.item("launch").unwrap()].on), ("Customer portal v1", Some(jiff::civil::date(2027, 3, 1))));
        assert_eq!(std::fs::read_to_string(dir.join("customer-portal-v1.pchart")).unwrap(), "");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_plan_without_dates_starts_with_its_structure_only() {
        let dir = folder("undated");
        let mut start = start(&dir, Vec::new());
        start.typed = "1 mar".into();
        start.from(1);
        start.from(1);
        assert!(start.undated && !start.forwards);
        let (rows, svg) = screen(&start, 160, 47);
        assert!(rows.contains(":: PREVIEW · NEW APPLICATION · PROJECT STRUCTURE") && rows.contains("←→ target, start or none") && !rows.contains("Tab "), "{rows}");
        assert!(svg.contains("No dates") && svg.contains("Dates come once every piece of work has a length.") && !svg.contains("such as 1 mar"));
        let plan = start.plan().unwrap();
        assert!(plan.without_length().len() == plan.items.iter().filter(|item| !item.milestone).count() && plan.items.iter().all(|item| item.on.is_none()));
        start.from(1);
        assert!(!start.undated && !start.forwards);
        start.from(-1);
        assert!(start.undated);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn in_a_workspace_its_projects_come_first_and_a_new_one_goes_into_projects() {
        let dir = folder("workspace");
        let path = dir.join("portal.pchart");
        std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples/customer-portal.pchart"), &path).unwrap();
        let mut start = start(&dir, vec![path.clone()]);
        start.workspace = Some("Acme".into());
        assert_eq!((start.plan_at(), start.template_at()), (Some(0), None));
        let (rows, svg) = screen(&start, 160, 47);
        assert!(rows.contains(":: START · ACME") && rows.contains("PROJECTS") && rows.contains("NEW PROJECT FROM") && !rows.contains("RECENT") && rows.contains(":: OPEN"), "{rows}");
        assert!(svg.contains("Customer portal v1") && svg.contains("Launch on Mon 01 Mar is "));
        assert!(rows.find("PROJECTS").unwrap() < rows.find("NEW PROJECT FROM").unwrap());
        start.choose(1);
        assert_eq!((start.plan_at(), start.template_at()), (None, Some(0)));
        assert!(screen(&start, 160, 47).1.contains("projects/new-application.pchart"));
        start.choose(-1);
        assert_eq!(start.enter(), Some(path));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_plan_opened_before_says_how_it_stands_and_opens_as_it_is() {
        let dir = folder("recent");
        let path = dir.join("portal.pchart");
        std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples/customer-portal.pchart"), &path).unwrap();
        let mut start = start(&dir, vec![path.clone()]);
        start.choose(-1);
        assert_eq!(start.chosen, 4);
        let (rows, svg) = screen(&start, 160, 47);
        assert!(rows.contains("open a plan") && rows.contains(":: OPEN") && rows.contains("↑↓ plan") && !rows.contains("no plans yet"));
        assert!(svg.contains("Customer portal v1") && svg.contains("Launch on Mon 01 Mar is ") && svg.contains("changed "));
        assert_eq!(start.enter(), Some(path));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn no_text_runs_over_the_edge_of_its_shape_on_the_start() {
        let dir = folder("edges");
        let path = dir.join("portal.pchart");
        std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples/customer-portal.pchart"), &path).unwrap();
        let mut start = start(&dir, vec![path]);
        let output = pchart_draw::Output::default();
        let mut found = Vec::new();
        for (cell, chosen) in [(7, 16), (8, 17), (10, 21)].into_iter().flat_map(|cell| (0..5).map(move |chosen| (cell, chosen))) {
            start.cell = cell;
            start.chosen = chosen;
            for (typed, forwards, undated, workspace) in [("", false, false, false), ("1 mar", false, false, false), ("1 jan", false, false, false), ("2 nov", true, false, false), ("", false, true, false), ("", false, false, true)] {
                (start.typed, start.forwards, start.undated) = (typed.into(), forwards, undated);
                start.workspace = workspace.then(|| "Acme".to_string());
                for (width, height) in [(160, 47), (200, 60), (120, 40), (91, 43), (80, 24)] {
                    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
                    let mut placements = Vec::new();
                    terminal.draw(|frame| placements = draw(&start, frame)).unwrap();
                    for placement in placements {
                        found.extend(output.crossings(&placement.svg).unwrap().into_iter().map(|crossing| format!("{chosen} \"{typed}\" {width}x{height} {cell:?}: {crossing}")));
                    }
                }
            }
        }
        std::fs::remove_dir_all(dir).unwrap();
        assert!(found.is_empty(), "{} crossings:\n{}", found.len(), found.join("\n"));
    }

    #[test]
    fn a_small_window_leaves_out_the_preview() {
        let dir = folder("small");
        let (rows, _) = screen(&start(&dir, Vec::new()), 80, 24);
        assert!(!rows.contains("PREVIEW") && rows.contains(":: HOW TO START") && rows.contains("PROJECT TEMPLATES"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
