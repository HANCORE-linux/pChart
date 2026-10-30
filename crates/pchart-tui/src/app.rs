use jiff::civil::Date;
use pchart_core::{Analysis, Calendar, Plan, Schedule, analyse};
use pchart_draw::{Level, Palette, Settings};
use ratatui::layout::Rect;

/// The screens, as tabs in the title bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Situation,
    Roadmap,
    Checks,
    Export,
}

pub struct App {
    pub plan: Plan,
    pub calendars: Vec<Calendar>,
    pub today: Date,
    pub path: String,
    pub schedule: Schedule,
    pub analysis: Analysis,
    pub screen: Screen,
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
        App { plan, calendars, today, path, schedule, analysis, screen: Screen::Roadmap, selected, level: Level::Standard, cell, message: None, palette: Palette::labra() }
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
