use std::path::PathBuf;

use jiff::civil::Date;
use pchart_core::{Analysis, Calendar, Edit, Fix, Plan, Schedule, analyse};
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

/// The answers of the date question (picture 37), in its order: calculated after what it waits on, on a
/// fixed date, no earlier than, no later than.
pub const ANSWERS: [Option<Fix>; 4] = [None, Some(Fix::On), Some(Fix::NoEarlierThan), Some(Fix::NoLaterThan)];

/// The open date question: about which item, the answer chosen, and the date typed so far.
pub struct Question {
    pub item: usize,
    pub choice: usize,
    pub typed: String,
}

/// The plan as it was before a what-if, which Esc goes back to.
pub struct Before {
    pub plan: Plan,
    pub schedule: Schedule,
    pub analysis: Analysis,
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
    /// The plans before each change, newest last, with what the change was; `u` goes back one.
    pub history: Vec<(Plan, String)>,
    /// The plan changed and is still to be written.
    pub unsaved: bool,
    pub question: Option<Question>,
    /// Set while a what-if is tried: the plan, schedule and analysis hold the tried plan, which is not
    /// written until Enter keeps it.
    pub before: Option<Before>,
}

impl App {
    pub fn new(plan: Plan, calendars: Vec<Calendar>, today: Date, path: String, schedule: Schedule, cell: (u16, u16)) -> App {
        let selected = plan.item("frontend").or_else(|| schedule.chain.iter().copied().find(|&index| !plan.items[index].milestone)).unwrap_or(0);
        let analysis = analyse(&plan, &calendars, &schedule, today);
        App { plan, calendars, today, path, schedule, analysis, screen: Screen::Situation, statement: 0, check: 0, export: Export::default(), selected, level: Level::Standard, cell, message: None, palette: Palette::labra(), history: Vec::new(), unsaved: false, question: None, before: None }
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

    /// The roadmap picture for `rect`, without a ground of its own; in a what-if with the old places.
    pub fn svg(&self, rect: Rect) -> String {
        let mut scene = match &self.before {
            Some(before) => pchart_draw::what_if(&self.plan, &self.schedule, &self.calendars, &self.settings(rect), &before.schedule),
            None => pchart_draw::roadmap(&self.plan, &self.schedule, &self.calendars, &self.settings(rect)),
        };
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

    fn take(&mut self, plan: Plan, schedule: Schedule) {
        self.analysis = analyse(&plan, &self.calendars, &schedule, self.today);
        self.plan = plan;
        self.schedule = schedule;
        self.unsaved = true;
    }

    /// Makes `edit`, keeps the plan before it for `u`, and marks the plan to be written.
    pub fn apply(&mut self, edit: &Edit, label: String) -> Result<(), String> {
        let plan = edit.apply(&self.plan).map_err(|err| err.to_string())?;
        let schedule = pchart_core::schedule(&plan, &self.calendars, self.today).map_err(|err| err.to_string())?;
        let before = self.plan.clone();
        self.take(plan, schedule);
        self.history.push((before, label));
        self.message = None;
        Ok(())
    }

    /// Goes back to the plan before the last change.
    pub fn undo(&mut self) {
        let Some((plan, label)) = self.history.pop() else {
            self.message = Some("Nothing to undo".into());
            return;
        };
        match pchart_core::schedule(&plan, &self.calendars, self.today) {
            Ok(schedule) => {
                self.take(plan, schedule);
                self.message = Some(format!("Undone: {label}"));
            }
            Err(err) => self.message = Some(err.to_string()),
        }
    }

    /// Opens the date question for the selected item, with its present answer chosen.
    pub fn ask(&mut self) {
        let item = &self.plan.items[self.selected];
        let (choice, date) = [(1, item.on), (2, item.no_earlier_than), (3, item.no_later_than)].into_iter().find(|(_, date)| date.is_some()).unwrap_or((0, None));
        self.question = Some(Question { item: self.selected, choice, typed: date.map(|date| date.strftime("%d %b %Y").to_string()).unwrap_or_default() });
    }

    /// The change the open question would make: None while nothing would change or the date is not
    /// understood yet.
    pub fn answer(&self) -> Option<Edit> {
        let question = self.question.as_ref()?;
        let item = &self.plan.items[question.item];
        match ANSWERS[question.choice] {
            None => (item.on.is_some() || item.no_earlier_than.is_some() || item.no_later_than.is_some()).then(|| Edit::Unfix { item: item.id.clone() }),
            Some(kind) => pchart_core::input::date(&question.typed, self.today).map(|date| Edit::Fix { item: item.id.clone(), kind, date }),
        }
    }

    /// Makes the answer of the open question and closes it.
    pub fn confirm(&mut self) {
        let Some(question) = &self.question else { return };
        let item = &self.plan.items[question.item];
        let label = format!("{} {}", item.name, if item.milestone { "date" } else { "start" });
        match self.answer() {
            Some(edit) => match self.apply(&edit, label) {
                Ok(()) => self.question = None,
                Err(err) => self.message = Some(err),
            },
            None if ANSWERS[question.choice].is_none() => self.question = None,
            None => self.message = Some("Type a date, such as 4 jan or next friday".into()),
        }
    }

    /// Tries `edit` as what-if, on top of what was tried before: nothing is written.
    fn try_edit(&mut self, edit: &Edit) {
        let tried = edit.apply(&self.plan).map_err(|err| err.to_string()).and_then(|plan| pchart_core::schedule(&plan, &self.calendars, self.today).map(|schedule| (plan, schedule)).map_err(|err| err.to_string()));
        match tried {
            Ok((plan, schedule)) => {
                if self.before.is_none() {
                    self.before = Some(Before { plan: self.plan.clone(), schedule: self.schedule.clone(), analysis: self.analysis.clone() });
                }
                self.analysis = analyse(&plan, &self.calendars, &schedule, self.today);
                self.plan = plan;
                self.schedule = schedule;
                self.message = None;
            }
            Err(err) => self.message = Some(err),
        }
    }

    /// `←→` on the roadmap: the selected item a working day longer or shorter, as what-if.
    pub fn try_length(&mut self, days: i32) {
        let item = &self.plan.items[self.selected];
        if item.milestone {
            self.message = Some(format!("{} is a milestone; it has no duration", item.name));
            return;
        }
        let days = (item.duration as i32 + days).max(1) as u32;
        self.try_edit(&Edit::Duration { item: item.id.clone(), days });
    }

    /// Tab in the date question: its answer as what-if.
    pub fn try_answer(&mut self) {
        match self.answer() {
            Some(edit) => {
                self.try_edit(&edit);
                if self.message.is_none() {
                    self.question = None;
                }
            }
            None => self.message = Some("Type a date, such as 4 jan or next friday".into()),
        }
    }

    /// What the what-if changed, item by item: "Frontend +12 wd", "Frontend start".
    pub fn changes(&self) -> Vec<String> {
        let Some(before) = &self.before else { return Vec::new() };
        before
            .plan
            .items
            .iter()
            .zip(&self.plan.items)
            .filter_map(|(old, new)| {
                if old.duration != new.duration {
                    Some(format!("{} {:+} wd", new.name, i64::from(new.duration) - i64::from(old.duration)))
                } else if (old.on, old.no_earlier_than, old.no_later_than) != (new.on, new.no_earlier_than, new.no_later_than) {
                    Some(format!("{} {}", new.name, if new.milestone { "date" } else { "start" }))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Enter in a what-if: the tried plan becomes the plan and is written; `u` takes it back.
    pub fn keep(&mut self) {
        let changes = self.changes();
        let Some(before) = self.before.take() else { return };
        if !changes.is_empty() {
            self.history.push((before.plan, changes.join(", ")));
            self.unsaved = true;
        }
    }

    /// Esc in a what-if: back to the plan as it was.
    pub fn discard(&mut self) {
        if let Some(before) = self.before.take() {
            self.plan = before.plan;
            self.schedule = before.schedule;
            self.analysis = before.analysis;
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
}
