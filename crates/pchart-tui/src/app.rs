use std::path::PathBuf;

use jiff::civil::Date;
use pchart_core::{Analysis, Calendar, Plan, Schedule, analyse};
use pchart_draw::{Audience, Level, Palette, Settings, View};
use ratatui::layout::Rect;

/// The screens, as tabs in the title bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Situation,
    Roadmap,
    Checks,
    Export,
}

/// The part of the Export screen that `↑↓` moves between and `←→` changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    View,
    Audience,
    Format,
}

/// The formats of the Export screen, in its order.
pub const FORMATS: [&str; 4] = ["PDF", "SVG", "PNG", "Markdown"];

/// What the Export screen writes, and what it wrote last.
pub struct Export {
    pub view: View,
    pub audience: Audience,
    pub formats: [bool; 4],
    pub part: Part,
    /// The format `Space` switches on or off.
    pub format: usize,
    pub busy: bool,
    /// The files written last, and the time they were written.
    pub written: Option<(Vec<PathBuf>, String)>,
}

impl Default for Export {
    fn default() -> Self {
        Export { view: View::Milestones, audience: Audience::Team, formats: [true, true, false, false], part: Part::View, format: 0, busy: false, written: None }
    }
}

pub struct App {
    pub plan: Plan,
    pub calendars: Vec<Calendar>,
    pub today: Date,
    pub path: String,
    pub schedule: Schedule,
    pub analysis: Analysis,
    pub screen: Screen,
    /// The statement chosen in "What is burning next".
    pub statement: usize,
    /// The check chosen on the Checks screen, in the order it lists them.
    pub check: usize,
    pub export: Export,
    pub selected: usize,
    /// The detail level the user picked; a small picture may show less.
    pub level: Level,
    pub cell: (u16, u16),
    pub message: Option<String>,
    pub palette: Palette,
}

impl App {
    pub fn new(plan: Plan, calendars: Vec<Calendar>, today: Date, path: String, schedule: Schedule, cell: (u16, u16)) -> App {
        let selected = plan.item("frontend").or_else(|| schedule.chain.iter().copied().find(|&index| !plan.items[index].milestone)).unwrap_or(0);
        let analysis = analyse(&plan, &calendars, &schedule, today);
        App { plan, calendars, today, path, schedule, analysis, screen: Screen::Situation, statement: 0, check: 0, export: Export::default(), selected, level: Level::Standard, cell, message: None, palette: Palette::labra() }
    }

    /// The roadmap picture for `rect`.
    pub fn settings(&self, rect: Rect) -> Settings {
        let (cw, ch) = self.cell;
        Settings {
            width: f32::from(rect.width) * f32::from(cw),
            height: f32::from(rect.height) * f32::from(ch),
            unit: f32::from(ch) / 21.0,
            today: self.today,
            selected: Some(self.selected),
            level: self.level,
            palette: self.palette,
        }
    }

    /// The roadmap picture for `rect`, without a ground of its own.
    pub fn svg(&self, rect: Rect) -> String {
        let mut scene = pchart_draw::roadmap(&self.plan, &self.schedule, &self.calendars, &self.settings(rect));
        scene.background = None;
        scene.svg()
    }

    /// The next or previous item in the order of the schedule; at the Overview level, the first item
    /// of the next or previous lane.
    pub fn select(&mut self, step: isize) {
        let order = &self.schedule.order;
        if self.level == Level::Overview {
            let lane = |index: usize| self.plan.lane(&self.plan.items[index].lane).unwrap_or(0);
            let lanes = self.plan.lanes.len() as isize;
            let next = (lane(self.selected) as isize + step).rem_euclid(lanes.max(1)) as usize;
            if let Some(&first) = order.iter().find(|&&index| lane(index) == next) {
                self.selected = first;
            }
            return;
        }
        let count = order.len() as isize;
        let position = order.iter().position(|&index| index == self.selected).unwrap_or(0) as isize;
        self.selected = order[(position + step).rem_euclid(count) as usize];
    }

    /// The next or previous statement in "What is burning next".
    pub fn choose(&mut self, step: isize) {
        let count = self.analysis.next.len() as isize;
        if count > 0 {
            self.statement = (self.statement as isize + step).rem_euclid(count) as usize;
        }
    }

    /// The roadmap, with the item of the chosen statement selected.
    pub fn show_in_roadmap(&mut self) {
        use pchart_core::analysis::Next;
        let item = self.analysis.next.get(self.statement).and_then(|next| match next {
            Next::StartBy { item, .. } | Next::Due { item, .. } => Some(*item),
            Next::Check(index) => self.analysis.checks[*index].items().first().copied(),
        });
        if let Some(item) = item {
            self.selected = item;
        }
        self.screen = Screen::Roadmap;
    }

    /// The checks by weight: problems, warnings, hints.
    pub fn checks(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.analysis.checks.len()).collect();
        order.sort_by_key(|&index| self.analysis.checks[index].severity());
        order
    }

    /// The next or previous check on the Checks screen.
    pub fn choose_check(&mut self, step: isize) {
        let count = self.analysis.checks.len() as isize;
        if count > 0 {
            self.check = (self.check as isize + step).rem_euclid(count) as usize;
        }
    }

    /// The roadmap, with the first item of the chosen check selected.
    pub fn show_check_in_roadmap(&mut self) {
        if let Some(item) = self.checks().get(self.check).and_then(|&index| self.analysis.checks[index].items().first().copied()) {
            self.selected = item;
        }
        self.screen = Screen::Roadmap;
    }

    /// `↑↓` on the Export screen.
    pub fn export_part(&mut self, step: isize) {
        let parts = [Part::View, Part::Audience, Part::Format];
        let at = parts.iter().position(|&part| part == self.export.part).unwrap_or(0) as isize;
        self.export.part = parts[(at + step).rem_euclid(parts.len() as isize) as usize];
    }

    /// `←→` on the Export screen: another view, another audience, or the next format.
    pub fn export_change(&mut self, step: isize) {
        let next = |at: usize, count: usize| (at as isize + step).rem_euclid(count as isize) as usize;
        let e = &mut self.export;
        match e.part {
            Part::View => e.view = View::ALL[next(View::ALL.iter().position(|&view| view == e.view).unwrap_or(0), View::ALL.len())],
            Part::Audience => {
                let audiences = [Audience::Team, Audience::Management, Audience::Client];
                e.audience = audiences[next(audiences.iter().position(|&audience| audience == e.audience).unwrap_or(0), audiences.len())];
            }
            Part::Format => e.format = next(e.format, FORMATS.len()),
        }
    }

    /// `Space` on the Export screen: the chosen format on or off.
    pub fn export_toggle(&mut self) {
        if self.export.part == Part::Format {
            self.export.formats[self.export.format] ^= true;
        }
    }

    /// Overview, Standard, Full, and round again.
    pub fn detail(&mut self) {
        self.level = match self.level {
            Level::Overview => Level::Standard,
            Level::Standard => Level::Full,
            Level::Full => Level::Overview,
        };
    }

    /// Lengthens or shortens the selected item by working days.
    pub fn stretch(&mut self, days: i32) {
        let item = &self.plan.items[self.selected];
        if item.milestone {
            self.message = Some(format!("{} is a milestone; it has no duration", item.name));
            return;
        }
        let mut plan = self.plan.clone();
        plan.items[self.selected].duration = (item.duration as i32 + days).max(1) as u32;
        match pchart_core::schedule(&plan, &self.calendars, self.today) {
            Ok(schedule) => {
                self.analysis = analyse(&plan, &self.calendars, &schedule, self.today);
                self.plan = plan;
                self.schedule = schedule;
                self.message = None;
            }
            Err(err) => self.message = Some(err.to_string()),
        }
    }
}
