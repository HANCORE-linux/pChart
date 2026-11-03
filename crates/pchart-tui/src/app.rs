use std::path::{Path, PathBuf};

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
    /// In a workspace, everyone's work across its projects (picture 64).
    People,
    /// In a workspace, what the team reports about the plan (pictures 65).
    Reports,
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

/// What the notes cursor stands on: an item, or a lane on a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    Item(usize),
    Place(usize, Date),
}

/// The notes mode (picture 43): the cursor's lane by its index in the plan, its day, and the note being
/// written (picture 40).
pub struct Notes {
    pub lane: usize,
    pub day: Date,
    pub typed: Option<String>,
}

/// The finder (Ctrl+P, picture 50): what is typed so far, and the line chosen in what it found.
#[derive(Default)]
pub struct Finder {
    pub typed: String,
    pub chosen: usize,
}

/// Adding an item (picture 51): its name, work or a milestone, its length as typed, its lane by its
/// index, one past the last for a new lane with the name typed for it (pictures 72), where it goes
/// among the places `places` offers, and the question `↑↓` is on.
pub struct Adding {
    pub from: usize,
    pub name: String,
    pub milestone: bool,
    pub length: String,
    pub lane: usize,
    pub lane_name: String,
    pub place: usize,
    pub at: usize,
}

/// Where a new item goes beside the item it was added from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// After it, and what waited on it waits on the new item.
    Between,
    After,
    /// Before it: the new item waits on what it waited on, and it waits on the new item.
    Before,
}

/// A change to the plan's items asked in a dialog over the roadmap (picture 51).
pub enum Form {
    Add(Adding),
    /// The name of `item` and of its lane as typed (pictures 72), and the row `↑↓` is on.
    Rename { item: usize, typed: String, lane: String, at: usize },
    /// What `item` waits on: one box for each item, and the one `↑↓` is on.
    WaitsOn { item: usize, on: Vec<bool>, at: usize },
    Remove { item: usize },
    /// How long `item` takes, as typed.
    Length { item: usize, typed: String },
    /// Who does `item` (picture 64): the people `App::who_rows` offers, the one `↑↓` is on, and the
    /// share of their time it takes, which `←→` changes.
    /// One row past `App::who_rows` is someone new, with their name as typed (pictures 72).
    Who { item: usize, at: usize, share: u32, typed: String },
    /// A report on `item` (pictures 65): how it stands by its place in `STATUSES`, the days more it needs
    /// as typed, the comment, who reports by their place in `App::reporters`, and the question `↑↓` is on.
    Report { item: usize, status: usize, days: String, comment: String, by: usize, at: usize, answering: Option<PathBuf> },
    /// Keeping how the plan stands today (pictures 67), with a name as typed, which may stay empty.
    Snapshot { typed: String },
    /// The phases (pictures 81): each with its name, the item it ends with and its short name; a new one
    /// as typed; and the row `↑↓` is on.
    Phases { rows: Vec<(String, usize, Option<String>)>, typed: String, at: usize },
    /// `q` while a what-if is not kept (pictures 78): keep it and quit, or discard it and quit.
    Leave,
}

pub use pchart_core::people::Crew;

/// A step of "Next" on the situation screen: one of the analysis, or a report to answer by its place
/// in `App::reports`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Next(usize),
    Report(usize),
}

/// How a piece of work can stand in a report, in the order of the report dialog.
pub const STATUSES: [(pchart_core::report::Status, &str); 4] = [
    (pchart_core::report::Status::Done, "Done"),
    (pchart_core::report::Status::OnTrack, "On track"),
    (pchart_core::report::Status::Risk, "At risk"),
    (pchart_core::report::Status::Later, "Needs more time"),
];

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
    /// The theme's page is painted under everything, as a theme named with --theme needs.
    pub ground: bool,
    /// The plans before each change, newest last, with what the change was; `u` goes back one.
    pub history: Vec<(Plan, String)>,
    /// The plan changed and is still to be written.
    pub unsaved: bool,
    pub question: Option<Question>,
    pub notes: Option<Notes>,
    /// Set while a what-if is tried: the plan, schedule and analysis hold the tried plan, which is not
    /// written until Enter keeps it.
    pub before: Option<Before>,
    /// The help of this screen is open (`?`).
    pub help: bool,
    pub finder: Option<Finder>,
    pub form: Option<Form>,
    /// The details beside the roadmap are shown (picture 52).
    pub side: bool,
    /// Why this date? (picture 56): the roadmap shows the chain that sets the selection's date.
    pub why: bool,
    /// The roadmap compared with the last snapshot (pictures 67).
    pub compare: bool,
    /// `o`: leave for the start, to begin or open another plan.
    pub to_start: bool,
    /// The detail level the roadmap could show at the last draw, which a small window may lower.
    pub shown: std::cell::Cell<Option<Level>>,
    pub crew: Option<Crew>,
    /// The line and the week chosen on the People screen.
    pub person: usize,
    pub week: usize,
    /// The workspace's folder, where reports go, and the reports about this plan, newest first
    /// (pictures 65); the one chosen on the Reports screen.
    pub root: Option<PathBuf>,
    /// Present mode (pictures 74): the slide shown, by its place in `present::SLIDES`.
    pub present: Option<usize>,
    /// The answer to the question of `Form::Leave`: quit now.
    pub quit: bool,
    /// In present mode, the entry of the slide `↑↓` is on, and whether Space shows its details.
    pub present_at: usize,
    pub present_details: bool,
    pub reports: Vec<(PathBuf, pchart_core::report::Report)>,
    pub report: usize,
    /// Who uses pChart here (`pchart_store::me`).
    pub me: Option<String>,
    /// The reports taken into the plan, each with the length `history` had after it, so `u` opens
    /// it again.
    pub taken: Vec<(usize, PathBuf)>,
}

impl App {
    pub fn new(plan: Plan, calendars: Vec<Calendar>, today: Date, path: String, schedule: Schedule, cell: (u16, u16)) -> App {
        let selected = plan.item("frontend").or_else(|| schedule.chain.iter().copied().find(|&index| !plan.items[index].milestone)).unwrap_or(0);
        let analysis = analyse(&plan, &calendars, &schedule, today);
        let mut app = App { plan, calendars, today, path, schedule, analysis, screen: Screen::Situation, statement: 0, check: 0, export: Export::default(), selected, level: Level::Standard, cell, message: None, palette: Palette::labra(), ground: false, history: Vec::new(), unsaved: false, question: None, notes: None, before: None, help: false, finder: None, form: None, side: false, why: false, compare: false, to_start: false, shown: std::cell::Cell::new(None), crew: None, person: 0, week: 0, root: None, present: None, quit: false, present_at: 0, present_details: false, reports: Vec::new(), report: 0, me: None, taken: Vec::new() };
        if !app.dated() || app.plan.items.iter().all(|item| item.milestone) {
            app.screen = Screen::Roadmap;
            app.export.view = View::Network;
            app.export.formats[3] = false;
        }
        app
    }

    /// Puts the plan into its workspace, so the checks of people look across its projects.
    pub fn join(&mut self, crew: Crew) {
        self.crew = Some(crew);
        let (plan, schedule) = (self.plan.clone(), self.schedule.clone());
        self.analysis = self.analysed(&plan, &schedule);
    }

    /// The analysis of `plan`, with the checks of people across the workspace's projects.
    fn analysed(&mut self, plan: &Plan, schedule: &Schedule) -> Analysis {
        match self.crew.as_mut() {
            Some(crew) => crew.analyse(plan, &self.calendars, schedule, self.today),
            None => analyse(plan, &self.calendars, schedule, self.today),
        }
    }

    /// The people `p` offers for `item`: those of its lane's team first, then the others, then no one.
    /// Outside a workspace, the names the plan uses.
    pub fn who_rows(&self, item: usize) -> Vec<Option<String>> {
        let lane = self.plan.lane(&self.plan.items[item].lane).map_or("", |lane| self.plan.lanes[lane].name.as_str());
        let mut out: Vec<Option<String>> = match &self.crew {
            Some(crew) => {
                let (team, others): (Vec<_>, Vec<_>) = crew.workspace.people.iter().partition(|person| person.team.as_deref() == Some(lane));
                team.into_iter().chain(others).map(|person| Some(person.name.clone())).collect()
            }
            None => {
                let mut names: Vec<String> = self.plan.items.iter().filter_map(|other| other.who.clone()).collect();
                names.sort();
                names.dedup();
                names.into_iter().map(Some).collect()
            }
        };
        out.push(None);
        out
    }

    /// `p` on the roadmap: who does the selected item, starting at who does it now.
    pub fn start_who(&mut self) {
        if self.plan.items[self.selected].milestone {
            self.message = Some("A milestone is a point in time; work has someone doing it".into());
            return;
        }
        let rows = self.who_rows(self.selected);
        let now = self.plan.items[self.selected].who.clone();
        let at = rows.iter().position(|row| *row == now).unwrap_or(0);
        self.form = Some(Form::Who { item: self.selected, at, share: self.plan.items[self.selected].share.unwrap_or(100), typed: String::new() });
    }

    /// Enter on the People screen outside a workspace (pictures 73): makes the plan's folder a team,
    /// with the people the plan names, and shows them across its projects at once.
    pub fn make_team(&mut self, lanes: &dyn Fn(&Plan) -> Vec<Calendar>) {
        let path = PathBuf::from(&self.path);
        let dir = path.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_path_buf();
        let named: Vec<(String, Option<String>)> = crate::people::crew_of(self).workspace.people.iter().map(|person| (person.name.clone(), person.team.clone())).collect();
        if let Err(err) = pchart_store::make_team(&dir, &named) {
            self.message = Some(err.to_string());
            return;
        }
        if let Some(crew) = pchart_store::crew(&path, &self.plan, &self.schedule, self.today, lanes) {
            self.message = Some(format!("{} is a team now", crew.name));
            self.join(crew);
            self.root = pchart_store::workspace(&dir).map(|(root, _)| root);
            self.refresh_reports();
        }
    }

    /// The plan file's name, as reports name it.
    pub fn file_name(&self) -> String {
        std::path::Path::new(&self.path).file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned())
    }

    /// Who answers reports, when the workspace names one.
    pub fn lead(&self) -> Option<&str> {
        self.crew.as_ref()?.workspace.lead.as_deref()
    }

    /// Whether the one using pChart here may answer reports: the workspace's lead, or anyone when it
    /// names none.
    pub fn answers(&self) -> bool {
        self.lead().is_none_or(|lead| self.me.as_deref() == Some(lead))
    }

    /// The reports not answered yet, by their place in `reports`.
    pub fn open_reports(&self) -> Vec<usize> {
        (0..self.reports.len()).filter(|&at| self.reports[at].1.answer.is_none()).collect()
    }

    /// Reads the reports again; a new one that is not answered yet is named in the key bar.
    pub fn refresh_reports(&mut self) {
        let Some(root) = &self.root else { return };
        let found = pchart_store::reports(root, &self.file_name());
        let new = found.iter().find(|(path, report)| report.answer.is_none() && !self.reports.iter().any(|(known, _)| known == path));
        if let Some((_, report)) = new.filter(|_| !self.reports.is_empty() || self.message.is_none()) {
            let name = self.plan.item(&report.item).map_or(report.item.clone(), |index| self.plan.items[index].name.clone());
            self.message = Some(format!("{} reports: {name} {}", report.by, report.words()));
        }
        self.reports = found;
        self.report = self.report.min(self.open_reports().len().saturating_sub(1));
    }

    /// Enter on the Reports screen: takes the chosen report into the plan as one change for `u`, and
    /// writes the answer into its file.
    pub fn take_report(&mut self) {
        if !self.may_answer() {
            return;
        }
        let Some((path, report)) = self.open_reports().get(self.report).map(|&at| self.reports[at].clone()) else { return };
        self.take_into(&path, &report, None);
    }

    /// Enter in the report dialog opened with `e`: takes the report as changed there, and writes what
    /// was taken into its file, so the reporter sees it.
    fn take_edited(&mut self, path: &std::path::Path, edited: &pchart_core::report::Report) {
        let Some((_, reported)) = self.reports.iter().find(|(known, _)| known == path).cloned() else { return };
        let same = (edited.status, edited.days, &edited.comment) == (reported.status, reported.days, &reported.comment);
        let taken = (!same).then(|| pchart_core::report::Taken { status: edited.status, days: edited.days, comment: edited.comment.clone() });
        if self.take_into(path, edited, taken) {
            self.form = None;
            self.screen = Screen::Reports;
        }
    }

    /// Takes `report` into the plan as one change for `u` and writes the answer; whether it worked.
    fn take_into(&mut self, path: &std::path::Path, report: &pchart_core::report::Report, taken: Option<pchart_core::report::Taken>) -> bool {
        let Some(edits) = report.edits(&self.plan) else {
            self.message = Some(format!("{} is no longer in the plan", report.item));
            return false;
        };
        if !edits.is_empty() {
            if let Err(err) = self.apply_all(&edits, format!("{}'s report", report.by)) {
                self.message = Some(err);
                return false;
            }
            self.taken.push((self.history.len(), path.to_path_buf()));
        }
        let name = self.plan.item(&report.item).map_or(report.item.clone(), |index| self.plan.items[index].name.clone());
        let changed = taken.is_some();
        let done = match pchart_store::answer(path, pchart_core::report::Answer::Taken, taken) {
            Ok(()) => {
                self.message = Some(format!("{}: {name} {}", if changed { "Taken as changed" } else { "Taken" }, report.words()));
                true
            }
            Err(err) => {
                self.message = Some(err.to_string());
                false
            }
        };
        self.refresh_reports();
        done
    }

    /// `x` on the Reports screen: declines the chosen report; the plan stays as it is.
    pub fn decline_report(&mut self) {
        if !self.may_answer() {
            return;
        }
        let Some((path, _)) = self.open_reports().get(self.report).map(|&at| self.reports[at].clone()) else { return };
        self.message = Some(match pchart_store::answer(&path, pchart_core::report::Answer::Declined, None) {
            Ok(()) => "Declined; the plan stays as it is".to_string(),
            Err(err) => err.to_string(),
        });
        self.refresh_reports();
    }

    /// Whether the one here may answer; if not, the key bar says who does.
    fn may_answer(&mut self) -> bool {
        if self.answers() {
            return true;
        }
        self.message = Some(format!("{} answers reports", self.lead().unwrap_or("The project lead")));
        false
    }

    /// Who can report: the people of the workspace.
    pub fn reporters(&self) -> Vec<String> {
        self.crew.as_ref().map(|crew| crew.workspace.people.iter().map(|person| person.name.clone()).collect()).unwrap_or_default()
    }

    /// `s` on the roadmap: a report on the selected item, from who does it.
    pub fn start_report(&mut self) {
        if self.root.is_none() {
            self.message = Some("Reports go to a workspace".into());
            return;
        }
        let reporters = self.reporters();
        let who = self.me.clone().filter(|me| reporters.contains(me)).or_else(|| self.plan.items[self.selected].who.clone());
        let by = who.and_then(|who| reporters.iter().position(|name| *name == who)).unwrap_or(0);
        self.form = Some(Form::Report { item: self.selected, status: 1, days: String::new(), comment: String::new(), by, at: 0, answering: None });
    }

    /// `e` on the Reports screen: the chosen report in the report dialog over the roadmap, to change
    /// before taking it.
    pub fn edit_report(&mut self) {
        if !self.may_answer() {
            return;
        }
        let Some((path, report)) = self.open_reports().get(self.report).map(|&at| self.reports[at].clone()) else { return };
        let Some(item) = self.plan.item(&report.item) else {
            self.message = Some(format!("{} is no longer in the plan", report.item));
            return;
        };
        self.selected = item;
        self.screen = Screen::Roadmap;
        let status = STATUSES.iter().position(|(status, _)| *status == report.status).unwrap_or(1);
        let days = if report.days > 0 { report.days.to_string() } else { String::new() };
        self.form = Some(Form::Report { item, status, days, comment: report.comment.clone().unwrap_or_default(), by: 0, at: 1, answering: Some(path) });
    }

    /// Esc in a dialog: closes it; one opened from the Reports screen goes back there.
    pub fn close_form(&mut self) {
        if matches!(self.form, Some(Form::Report { answering: Some(_), .. })) {
            self.screen = Screen::Reports;
        }
        if matches!(self.form, Some(Form::Leave)) {
            self.discard();
            self.quit = true;
        }
        self.form = None;
    }

    /// The report the open dialog would send.
    pub fn form_report(&self) -> Result<pchart_core::report::Report, String> {
        let Some(Form::Report { item, status, days, comment, by, answering, .. }) = &self.form else { return Err(String::new()) };
        let (status, _) = STATUSES[*status];
        let days = if status == pchart_core::report::Status::Later { pchart_core::input::length(days).filter(|&days| days > 0).ok_or("Type how many days more, such as 3")? } else { 0 };
        let reported = answering.as_ref().and_then(|path| self.reports.iter().find(|(known, _)| known == path)).map(|(_, report)| report.by.clone());
        let by = match reported {
            Some(by) => by,
            None => self.reporters().get(*by).cloned().ok_or("The workspace names no people")?,
        };
        Ok(pchart_core::report::Report {
            project: self.file_name(),
            item: self.plan.items[*item].id.clone(),
            by,
            at: {
                let now = jiff::Zoned::now().datetime();
                self.today.at(now.hour(), now.minute(), 0, 0)
            },
            status,
            days,
            comment: Some(comment.trim().to_string()).filter(|comment| !comment.is_empty()),
            answer: None,
            taken: None,
        })
    }

    /// Enter in the report dialog: writes the report for the project lead.
    pub fn send_report(&mut self) {
        let (Some(root), Ok(report)) = (self.root.clone(), self.form_report()) else {
            if let Err(err) = self.form_report() {
                self.message = Some(err);
            }
            return;
        };
        match pchart_store::report(&root, &report) {
            Ok(_) => {
                self.form = None;
                self.refresh_reports();
                self.message = Some("Reported; the plan stays as it is until the project lead takes it".into());
            }
            Err(err) => self.message = Some(err.to_string()),
        }
    }

    /// Whether every piece of work has a length, so the plan has dates.
    pub fn dated(&self) -> bool {
        self.plan.without_length().is_empty()
    }

    /// The formats the Export screen offers: Markdown only with dates.
    pub fn formats(&self) -> usize {
        if self.dated() { FORMATS.len() } else { FORMATS.len() - 1 }
    }

    /// The colours of the screen under the help or the finder: dimmed while one of them is open.
    pub fn shown(&self) -> Palette {
        if self.overlay() { self.palette.dimmed() } else { self.palette }
    }

    pub fn overlay(&self) -> bool {
        self.help || self.finder.is_some()
    }

    /// The roadmap picture for `rect`.
    pub fn settings(&self, rect: Rect) -> Settings {
        let (cw, ch) = self.cell;
        Settings {
            width: f32::from(rect.width) * f32::from(cw),
            height: f32::from(rect.height) * f32::from(ch),
            unit: f32::from(ch) / 21.0,
            today: self.today,
            selected: if matches!(self.spot(), Some(Spot::Place(..))) { None } else { Some(self.selected) },
            level: self.level,
            palette: self.shown(),
            cursor: self.notes.as_ref().map(|notes| (notes.lane, notes.day)),
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
    /// The steps of "Next": what the analysis says, with the reports to answer after when to start.
    pub fn steps(&self) -> Vec<Step> {
        use pchart_core::analysis::Next;
        let reports = self.open_reports();
        let after = self.analysis.next.iter().take_while(|next| matches!(next, Next::StartBy { .. })).count();
        let mut out: Vec<Step> = (0..self.analysis.next.len()).map(Step::Next).collect();
        for (offset, at) in reports.into_iter().enumerate() {
            out.insert(after + offset, Step::Report(at));
        }
        out
    }

    pub fn choose(&mut self, step: isize) {
        let count = self.steps().len() as isize;
        if count > 0 {
            self.statement = (self.statement as isize + step).rem_euclid(count) as usize;
        }
    }

    /// The roadmap, with the item of the chosen statement selected.
    pub fn show_in_roadmap(&mut self) {
        use pchart_core::analysis::Next;
        let item = match self.steps().get(self.statement) {
            Some(Step::Report(at)) => {
                self.report = self.open_reports().iter().position(|open| open == at).unwrap_or(0);
                self.screen = Screen::Reports;
                return;
            }
            Some(Step::Next(index)) => match &self.analysis.next[*index] {
                Next::StartBy { item, .. } | Next::Due { item, .. } => Some(*item),
                Next::Check(index) => self.analysis.checks[*index].items().first().copied(),
            },
            None => None,
        };
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

    /// The next or previous line on the Checks screen: the checks, then the checks that found nothing and
    /// the holidays.
    pub fn choose_check(&mut self, step: isize) {
        let count = self.analysis.checks.len() as isize + 2;
        self.check = (self.check as isize + step).rem_euclid(count) as usize;
    }

    /// `b` on the Checks screen: the buffer the chosen check suggests, as one change that `u` takes back.
    pub fn add_buffer(&mut self) {
        let Some(&index) = self.checks().get(self.check) else { return };
        let pchart_core::Check::NoBuffer { before, target } = self.analysis.checks[index] else { return };
        let target_name = self.plan.items[target].name.clone();
        let Some(edits) = pchart_core::text::buffer_edits(&self.plan, before, target) else { return };
        match self.apply_all(&edits, format!("buffer before {target_name}")) {
            Ok(()) => {
                self.check = self.check.min((self.analysis.checks.len() + 1).saturating_sub(1));
                self.message = Some("Buffer added · u takes it back".into());
            }
            Err(err) => self.message = Some(err),
        }
    }

    /// Whether `b` adds a buffer: the chosen check is a missing one.
    pub fn buffer_here(&self) -> bool {
        self.checks().get(self.check).is_some_and(|&index| matches!(self.analysis.checks[index], pchart_core::Check::NoBuffer { .. }))
    }

    /// The roadmap, with the first item of the chosen check selected; nothing on the two quiet lines.
    pub fn show_check_in_roadmap(&mut self) {
        let Some(&index) = self.checks().get(self.check) else { return };
        if let Some(&item) = self.analysis.checks[index].items().first() {
            self.selected = item;
        }
        self.screen = Screen::Roadmap;
    }

    /// `↑↓` on the Export screen.
    pub fn export_part(&mut self, step: isize) {
        if self.export.part == Part::View {
            let views = View::of(&self.plan);
            let at = views.iter().position(|&view| view == self.export.view).unwrap_or(0) as isize + step;
            if (0..views.len() as isize).contains(&at) {
                self.export.view = views[at as usize];
                return;
            }
        }
        let parts = [Part::View, Part::Audience, Part::Format];
        let at = parts.iter().position(|&part| part == self.export.part).unwrap_or(0) as isize;
        self.export.part = parts[(at + step).rem_euclid(parts.len() as isize) as usize];
    }

    /// `←→` on the Export screen: another view, another audience, or the next format.
    pub fn export_change(&mut self, step: isize) {
        let next = |at: usize, count: usize| (at as isize + step).rem_euclid(count as isize) as usize;
        let formats = self.formats();
        let e = &mut self.export;
        match e.part {
            Part::View => {
                let views = View::of(&self.plan);
                e.view = views[next(views.iter().position(|&view| view == e.view).unwrap_or(0), views.len())];
            }
            Part::Audience => {
                let audiences = [Audience::Team, Audience::Management, Audience::Client];
                e.audience = audiences[next(audiences.iter().position(|&audience| audience == e.audience).unwrap_or(0), audiences.len())];
            }
            Part::Format => e.format = next(e.format, formats),
        }
    }

    /// `Space` on the Export screen: the chosen format on or off.
    pub fn export_toggle(&mut self) {
        if self.export.part == Part::Format {
            self.export.formats[self.export.format] ^= true;
        }
    }

    fn take(&mut self, plan: Plan, schedule: Schedule) {
        self.analysis = self.analysed(&plan, &schedule);
        self.plan = plan;
        self.schedule = schedule;
        self.selected = self.selected.min(self.plan.items.len().saturating_sub(1));
        self.unsaved = true;
    }

    /// Makes `edit`, keeps the plan before it for `u`, and marks the plan to be written.
    pub fn apply(&mut self, edit: &Edit, label: String) -> Result<(), String> {
        let plan = edit.apply(&self.plan).map_err(|err| err.to_string())?;
        self.calendars = self.calendars_for(&plan);
        let schedule = pchart_core::schedule(&plan, &self.calendars, self.today).map_err(|err| err.to_string())?;
        let before = self.plan.clone();
        self.take(plan, schedule);
        self.history.push((before, label));
        self.message = None;
        Ok(())
    }

    /// The calendars of the lanes of `plan`: a lane added in pChart has no place, so no holidays.
    pub fn calendars_for(&self, plan: &Plan) -> Vec<Calendar> {
        let mut calendars = self.calendars.clone();
        calendars.resize_with(plan.lanes.len().max(calendars.len()), Calendar::default);
        calendars
    }

    /// Someone new named in `p` (pictures 72): in a workspace they join its people, in `team`.
    fn welcome(&mut self, name: &str, team: Option<String>) {
        let (Some(root), Some(crew)) = (self.root.clone(), self.crew.as_ref()) else { return };
        if crew.workspace.person(name).is_some() {
            return;
        }
        if let Err(err) = pchart_store::add_person(&root, name, team.as_deref()) {
            self.message = Some(format!("{name} is in the plan but not in workspace.toml: {err}"));
            return;
        }
        if let Some(mut crew) = self.crew.take() {
            crew.workspace.people.push(pchart_core::people::Person { name: name.to_string(), team, capacity: None, away: Vec::new() });
            self.join(crew);
        }
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
        if let Some((_, path)) = self.taken.pop_if(|(after, _)| *after > self.history.len()) {
            if let Err(err) = pchart_store::reopen(&path) {
                self.message = Some(err.to_string());
            }
            self.refresh_reports();
        }
    }

    /// Makes `edits` as one change, which `u` takes back at once.
    pub fn apply_all(&mut self, edits: &[Edit], label: String) -> Result<(), String> {
        let plan = pchart_core::apply_all(&self.plan, edits).map_err(|(_, err)| err.to_string())?;
        self.calendars = self.calendars_for(&plan);
        let schedule = pchart_core::schedule(&plan, &self.calendars, self.today).map_err(|err| err.to_string())?;
        let before = self.plan.clone();
        self.take(plan, schedule);
        self.history.push((before, label));
        self.message = None;
        Ok(())
    }

    /// `a` on the roadmap: adds an item after the selected one, in its lane; before it when it is the
    /// milestone the plan ends with.
    pub fn start_add(&mut self) {
        let lane = self.plan.lane(&self.plan.items[self.selected].lane).unwrap_or(0);
        let length = if self.dated() { "5" } else { "" };
        let id = &self.plan.items[self.selected].id;
        let end = self.plan.items[self.selected].milestone && !self.plan.items.iter().any(|other| other.after.contains(id));
        let place = if end { self.places(self.selected).iter().position(|(place, _)| *place == Place::Before).unwrap_or(0) } else { 0 };
        self.form = Some(Form::Add(Adding { from: self.selected, name: String::new(), milestone: false, length: length.into(), lane, lane_name: String::new(), place, at: 0 }));
    }

    /// Enter in the network diagram: how long the selected item takes.
    pub fn start_length(&mut self) {
        let item = &self.plan.items[self.selected];
        if item.milestone {
            self.message = Some(format!("{} is a milestone; it has no length", item.name));
            return;
        }
        self.form = Some(Form::Length { item: self.selected, typed: String::new() });
    }

    /// `Tab` and `Shift+Tab`: the next screen of the title bar, or the one before.
    pub fn next_screen(&mut self, step: isize) {
        let mut screens = vec![Screen::Situation, Screen::Roadmap, Screen::Checks, Screen::Export, Screen::People];
        if self.crew.is_some() {
            screens.push(Screen::Reports);
        }
        let at = screens.iter().position(|&screen| screen == self.screen).unwrap_or(0) as isize;
        self.screen = screens[(at + step).rem_euclid(screens.len() as isize) as usize];
    }

    /// `t`: keep how the plan stands today.
    pub fn start_snapshot(&mut self) {
        self.screen = Screen::Roadmap;
        self.form = Some(Form::Snapshot { typed: String::new() });
    }

    /// The last snapshot up to today and what changed since.
    pub fn since(&self) -> Option<(&pchart_core::snapshot::Snapshot, Vec<pchart_core::snapshot::Change>)> {
        let snapshot = pchart_core::snapshot::last(&self.plan, self.today)?;
        Some((snapshot, pchart_core::snapshot::changes(&self.plan, &self.schedule, &self.calendars, snapshot)))
    }

    /// The schedule as the last snapshot holds it, for the roadmap to draw its dates dashed.
    pub fn then(&self) -> Option<Schedule> {
        let snapshot = pchart_core::snapshot::last(&self.plan, self.today)?;
        let mut schedule = self.schedule.clone();
        for (item, slot) in self.plan.items.iter().zip(schedule.slots.iter_mut()) {
            if let Some(&(start, finish)) = snapshot.items.get(&item.id) {
                slot.planned = pchart_core::Span { start, finish };
            }
        }
        Some(schedule)
    }

    /// `c`: the roadmap compared with the last snapshot, or back.
    pub fn toggle_compare(&mut self) {
        if pchart_core::snapshot::last(&self.plan, self.today).is_none() {
            self.message = Some("No snapshot yet; t keeps one".into());
            return;
        }
        self.compare = !self.compare;
        self.screen = Screen::Roadmap;
    }

    /// The arrow keys in the network diagram: the nearest item that way.
    pub fn move_in_network(&mut self, dx: f32, dy: f32) {
        let (_, boxes, _) = pchart_draw::structure::layout(&self.plan, &self.schedule);
        let middle = |index: usize| ((boxes[index][0] + boxes[index][2]) / 2.0, (boxes[index][1] + boxes[index][3]) / 2.0);
        let (x, y) = middle(self.selected);
        let near = (0..boxes.len()).filter(|&index| index != self.selected).filter_map(|index| {
            let (ox, oy) = middle(index);
            let (ahead, aside) = ((ox - x) * dx + (oy - y) * dy, ((ox - x) * dy).abs() + ((oy - y) * dx).abs());
            (ahead > 1.0).then_some((ahead + 3.0 * aside, index))
        });
        if let Some((_, index)) = near.min_by(|a, b| a.0.total_cmp(&b.0)) {
            self.selected = index;
        }
    }

    /// `e` on the roadmap: the phases of the plan, to rename, end elsewhere, remove or add.
    pub fn start_phases(&mut self) {
        let rows = self.plan.phases.iter().filter_map(|phase| self.plan.item(&phase.until).map(|until| (phase.name.clone(), until, phase.short.clone()))).collect();
        self.form = Some(Form::Phases { rows, typed: String::new(), at: 0 });
    }

    /// The items a phase can end with, in the order they finish.
    pub fn phase_ends(&self) -> Vec<usize> {
        let mut ends: Vec<usize> = (0..self.plan.items.len()).collect();
        ends.sort_by_key(|&index| (self.schedule.slots[index].planned.finish, !self.plan.items[index].milestone));
        ends
    }

    /// The phases the dialog would leave, in the order they end; the new one ends with the last item.
    pub fn form_phases(&self) -> Option<Vec<pchart_core::model::Phase>> {
        let Some(Form::Phases { rows, typed, .. }) = &self.form else { return None };
        let ends = self.phase_ends();
        let mut out: Vec<(usize, pchart_core::model::Phase)> = rows.iter().map(|(name, until, short)| (*until, pchart_core::model::Phase { name: name.trim().to_string(), until: self.plan.items[*until].id.clone(), short: short.clone() })).collect();
        if !typed.trim().is_empty() {
            let last = *ends.last()?;
            out.push((last, pchart_core::model::Phase { name: typed.trim().to_string(), until: self.plan.items[last].id.clone(), short: None }));
        }
        out.sort_by_key(|(until, _)| ends.iter().position(|end| end == until));
        Some(out.into_iter().map(|(_, phase)| phase).collect())
    }

    /// Ctrl+D in the phases (pictures 82): keeps them as they are in the dialog, then asks when the chosen
    /// phase ends, as Enter asks for the item it ends with on the roadmap.
    pub fn phase_date(&mut self) {
        let Some(Form::Phases { rows, at, .. }) = &self.form else { return };
        let Some(&(_, until, _)) = rows.get(*at) else { return };
        let until_id = self.plan.items[until].id.clone();
        self.keep_form();
        if self.form.is_some() {
            return;
        }
        if let Some(index) = self.plan.item(&until_id) {
            self.selected = index;
            self.ask();
        }
    }

    /// Delete in a dialog: removes the chosen phase.
    pub fn form_delete(&mut self) {
        if let Some(Form::Phases { rows, at, .. }) = self.form.as_mut()
            && *at < rows.len()
        {
            rows.remove(*at);
            *at = (*at).min(rows.len());
        }
    }

    /// `r` on the roadmap: renames the selected item.
    pub fn start_rename(&mut self) {
        let lane = self.plan.lane(&self.plan.items[self.selected].lane).map_or_else(String::new, |lane| self.plan.lanes[lane].name.clone());
        self.form = Some(Form::Rename { item: self.selected, typed: self.plan.items[self.selected].name.clone(), lane, at: 0 });
    }

    /// `w` on the roadmap: what the selected item waits on, starting at the first it waits on.
    pub fn start_waits(&mut self) {
        let item = &self.plan.items[self.selected];
        let on: Vec<bool> = self.plan.items.iter().map(|other| item.after.contains(&other.id)).collect();
        let order = self.waits_order(self.selected);
        let at = order.iter().position(|&index| on[index]).unwrap_or(0);
        self.form = Some(Form::WaitsOn { item: self.selected, on, at });
    }

    /// `x` on the roadmap: removes the selected item.
    pub fn start_remove(&mut self) {
        if self.plan.items.len() < 2 {
            self.message = Some("A plan keeps one item at least".into());
            return;
        }
        self.form = Some(Form::Remove { item: self.selected });
    }

    /// What removing `index` changes, as edits and in words: what waited on it waits on what it waited
    /// on, and a phase that ended with it ends with the item before it that ends last.
    pub fn removal(&self, index: usize) -> Result<(Vec<Edit>, Vec<String>), String> {
        let item = &self.plan.items[index];
        let name = |id: &String| self.plan.item(id).map_or_else(|| id.clone(), |other| self.plan.items[other].name.clone());
        let (mut edits, mut words) = (Vec::new(), Vec::new());
        for other in self.plan.items.iter().filter(|other| other.after.contains(&item.id)) {
            edits.push(Edit::Unlink { from: item.id.clone(), to: other.id.clone() });
            let added: Vec<&String> = item.after.iter().filter(|before| !other.after.contains(before)).collect();
            edits.extend(added.iter().map(|before| Edit::Link { from: (*before).clone(), to: other.id.clone() }));
            words.push(if added.is_empty() {
                format!("{} no longer waits on {}.", other.name, item.name)
            } else {
                format!("{} waits on {} instead of {}.", other.name, added.iter().map(|id| name(id)).collect::<Vec<_>>().join(", "), item.name)
            });
        }
        for phase in self.plan.phases.iter().filter(|phase| phase.until == item.id) {
            let last = item.after.iter().filter_map(|id| self.plan.item(id)).max_by_key(|&other| self.schedule.slots[other].planned.finish);
            let Some(last) = last else {
                return Err(format!("{} cannot be removed: the {} phase ends with it, and nothing comes before it.", item.name, phase.name));
            };
            edits.push(Edit::Phase { phase: phase.name.clone(), until: self.plan.items[last].id.clone() });
            words.push(format!("The {} phase ends with {} instead.", phase.name, self.plan.items[last].name));
        }
        if item.note.is_some() {
            words.push("Its note goes with it.".into());
        }
        edits.push(Edit::Remove { item: item.id.clone() });
        Ok((edits, words))
    }

    /// The items `item` could wait on, in the order of the schedule: not those that wait on it, which
    /// would make a circle.
    pub fn waits_order(&self, item: usize) -> Vec<usize> {
        self.schedule.order.iter().copied().filter(|&index| index != item && !pchart_core::edit::waits_on(&self.plan, index, item)).collect()
    }

    /// Where an item added from `from` can go, in words.
    pub fn places(&self, from: usize) -> Vec<(Place, String)> {
        let name = |index: usize| self.plan.items[index].name.clone();
        let then: Vec<String> = self.plan.items.iter().filter(|other| other.after.contains(&self.plan.items[from].id)).map(|other| other.name.clone()).collect();
        let mut places = Vec::new();
        if !then.is_empty() {
            places.push((Place::Between, format!("after {}, before {}", name(from), then.join(", "))));
        }
        places.push((Place::After, if then.is_empty() { format!("after {}", name(from)) } else { format!("after {}, nothing waits on it", name(from)) }));
        places.push((Place::Before, format!("before {}", name(from))));
        places
    }

    /// The changes the open dialog would make and what to call them for `u`; an error says what is
    /// still missing.
    pub fn form_edits(&self) -> Result<(Vec<Edit>, String), String> {
        match self.form.as_ref().ok_or_else(String::new)? {
            Form::Add(adding) => {
                let name = adding.name.trim();
                if name.is_empty() {
                    return Err("Type a name".into());
                }
                let duration = if adding.milestone || !self.dated() && adding.length.trim().is_empty() { 0 } else { pchart_core::input::length(&adding.length).ok_or("Type a length, such as 5 or 2w")? };
                let from = &self.plan.items[adding.from];
                let id = self.plan.new_id(name);
                let place = self.places(adding.from).get(adding.place).map_or(Place::After, |(place, _)| *place);
                let after = if place == Place::Before { from.after.clone() } else { vec![from.id.clone()] };
                let mut edits = Vec::new();
                let lane = match self.plan.lanes.get(adding.lane) {
                    Some(lane) => lane.id.clone(),
                    None => {
                        let lane_name = adding.lane_name.trim();
                        if lane_name.is_empty() {
                            return Err("Name the new lane".into());
                        }
                        let lane = self.plan.new_lane_id(lane_name);
                        edits.push(Edit::AddLane { lane: lane.clone(), name: lane_name.to_string() });
                        lane
                    }
                };
                let item = pchart_core::model::Item { id: id.clone(), lane, name: name.to_string(), duration, after, milestone: adding.milestone, ..Default::default() };
                edits.push(Edit::Add { item });
                match place {
                    Place::Between => {
                        for other in self.plan.items.iter().filter(|other| other.after.contains(&from.id)) {
                            edits.push(Edit::Unlink { from: from.id.clone(), to: other.id.clone() });
                            edits.push(Edit::Link { from: id.clone(), to: other.id.clone() });
                        }
                    }
                    Place::After => {}
                    Place::Before => {
                        for before in &from.after {
                            edits.push(Edit::Unlink { from: before.clone(), to: from.id.clone() });
                        }
                        edits.push(Edit::Link { from: id.clone(), to: from.id.clone() });
                    }
                }
                Ok((edits, format!("added {name}")))
            }
            Form::Rename { item, typed, lane, .. } => {
                let (name, lane_name) = (typed.trim(), lane.trim());
                if name.is_empty() {
                    return Err("Type a name".into());
                }
                if lane_name.is_empty() {
                    return Err("Name the lane".into());
                }
                let old = &self.plan.items[*item];
                let mut edits = Vec::new();
                if name != old.name {
                    edits.push(Edit::Rename { item: old.id.clone(), name: name.to_string() });
                }
                if let Some(at) = self.plan.lane(&old.lane).filter(|&at| self.plan.lanes[at].name != lane_name) {
                    edits.push(Edit::RenameLane { lane: self.plan.lanes[at].id.clone(), name: lane_name.to_string() });
                }
                Ok((edits, format!("{} renamed", old.name)))
            }
            Form::WaitsOn { item, on, .. } => {
                let it = &self.plan.items[*item];
                let mut edits = Vec::new();
                for (index, other) in self.plan.items.iter().enumerate() {
                    match (it.after.contains(&other.id), on[index]) {
                        (false, true) => edits.push(Edit::Link { from: other.id.clone(), to: it.id.clone() }),
                        (true, false) => edits.push(Edit::Unlink { from: other.id.clone(), to: it.id.clone() }),
                        _ => {}
                    }
                }
                Ok((edits, format!("what {} waits on", it.name)))
            }
            Form::Remove { item } => Ok((self.removal(*item)?.0, format!("removed {}", self.plan.items[*item].name))),
            Form::Length { item, typed } => {
                let it = &self.plan.items[*item];
                let days = pchart_core::input::length(typed).filter(|&days| days > 0).ok_or("Type a length, such as 5 or 2w")?;
                Ok((if days == it.duration { Vec::new() } else { vec![Edit::Duration { item: it.id.clone(), days }] }, format!("{} length", it.name)))
            }
            Form::Report { .. } | Form::Snapshot { .. } | Form::Leave => Ok((Vec::new(), String::new())),
            Form::Phases { rows, .. } => {
                if rows.iter().any(|(name, ..)| name.trim().is_empty()) {
                    return Err("Name every phase".into());
                }
                let phases = self.form_phases().unwrap_or_default();
                Ok((if phases == self.plan.phases { Vec::new() } else { vec![Edit::Phases { phases }] }, "phases".into()))
            }
            Form::Who { item, at, share, typed } => {
                let it = &self.plan.items[*item];
                let rows = self.who_rows(*item);
                let who = match rows.get(*at) {
                    Some(row) => row.clone(),
                    None if typed.trim().is_empty() => return Err("Type a name".into()),
                    None => Some(typed.trim().to_string()),
                };
                let mut edits = Vec::new();
                if who != it.who {
                    edits.push(Edit::Who { item: it.id.clone(), who: who.clone() });
                }
                let share = (*share != 100 && who.is_some()).then_some(*share);
                if share != it.share {
                    edits.push(Edit::Share { item: it.id.clone(), share });
                }
                Ok((edits, format!("who does {}", it.name)))
            }
        }
    }

    /// `↑↓` in a dialog: the question or the item above or below.
    pub fn form_move(&mut self, step: isize) {
        let count = match &self.form {
            Some(Form::Add(_)) => 5,
            Some(Form::Rename { .. }) => 2,
            Some(Form::Phases { rows, .. }) => rows.len() + 1,
            Some(Form::WaitsOn { item, .. }) => self.waits_order(*item).len(),
            Some(Form::Who { item, .. }) => self.who_rows(*item).len() + 1,
            Some(Form::Report { .. }) => 4,
            _ => return,
        };
        if let Some(Form::Add(Adding { at, .. }) | Form::Rename { at, .. } | Form::Phases { at, .. } | Form::WaitsOn { at, .. } | Form::Who { at, .. } | Form::Report { at, .. }) = self.form.as_mut() {
            *at = (*at as isize + step).rem_euclid(count.max(1) as isize) as usize;
        }
    }

    /// `←→` in the add dialog: work or a milestone, the lane, or where it goes.
    pub fn form_change(&mut self, step: isize) {
        let ends = self.phase_ends();
        if let Some(Form::Phases { rows, at, .. }) = self.form.as_mut() {
            if let Some((_, until, _)) = rows.get_mut(*at) {
                let now = ends.iter().position(|end| end == until).unwrap_or(0) as isize;
                *until = ends[(now + step).clamp(0, ends.len() as isize - 1) as usize];
            }
            return;
        }
        if matches!(self.form, Some(Form::Leave)) {
            self.form = None;
            return;
        }
        if let Some(Form::Who { share, .. }) = self.form.as_mut() {
            *share = (*share as isize + step * 10).clamp(10, 100) as u32;
            return;
        }
        let people = self.reporters().len().max(1) as isize;
        if let Some(Form::Report { status, by, at, .. }) = self.form.as_mut() {
            match at {
                0 => *status = (*status as isize + step).rem_euclid(STATUSES.len() as isize) as usize,
                3 => *by = (*by as isize + step).rem_euclid(people) as usize,
                _ => {}
            }
            return;
        }
        let lanes = self.plan.lanes.len() as isize;
        let places = match &self.form {
            Some(Form::Add(adding)) => self.places(adding.from).len() as isize,
            _ => return,
        };
        if let Some(Form::Add(adding)) = self.form.as_mut() {
            match adding.at {
                1 => adding.milestone ^= true,
                3 => adding.lane = (adding.lane as isize + step).rem_euclid(lanes + 1) as usize,
                4 => adding.place = (adding.place as isize + step).rem_euclid(places) as usize,
                _ => {}
            }
        }
    }

    /// A key typed in a dialog: a letter of the name or the length; Space in "waits on" switches the
    /// item it is on.
    pub fn form_type(&mut self, c: char) {
        let order = match &self.form {
            Some(Form::WaitsOn { item, .. }) => self.waits_order(*item),
            _ => Vec::new(),
        };
        let (lanes, people) = (self.plan.lanes.len(), match &self.form {
            Some(Form::Who { item, .. }) => self.who_rows(*item).len(),
            _ => 0,
        });
        match self.form.as_mut() {
            Some(Form::Add(adding)) if adding.at == 0 => adding.name.push(c),
            Some(Form::Add(adding)) if adding.at == 2 => adding.length.push(c),
            Some(Form::Add(adding)) if adding.at == 3 && adding.lane == lanes => adding.lane_name.push(c),
            Some(Form::Rename { lane, at: 1, .. }) => lane.push(c),
            Some(Form::Phases { rows, typed, at }) => match rows.get_mut(*at) {
                Some((name, _, short)) => {
                    name.push(c);
                    *short = None;
                }
                None => typed.push(c),
            },
            Some(Form::Who { at, typed, .. }) => {
                *at = people;
                typed.push(c);
            }
            Some(Form::Rename { typed, .. } | Form::Length { typed, .. } | Form::Snapshot { typed }) => typed.push(c),
            Some(Form::Report { days, at: 1, .. }) => days.push(c),
            Some(Form::Report { comment, at: 2, .. }) => comment.push(c),
            Some(Form::WaitsOn { on, at, .. }) if c == ' ' => {
                if let Some(&other) = order.get(*at) {
                    on[other] ^= true;
                }
            }
            _ => {}
        }
    }

    /// Backspace in a dialog.
    pub fn form_back(&mut self) {
        match self.form.as_mut() {
            Some(Form::Add(adding)) if adding.at == 0 => drop(adding.name.pop()),
            Some(Form::Add(adding)) if adding.at == 2 => drop(adding.length.pop()),
            Some(Form::Add(adding)) if adding.at == 3 => drop(adding.lane_name.pop()),
            Some(Form::Rename { lane, at: 1, .. }) => drop(lane.pop()),
            Some(Form::Phases { rows, typed, at }) => match rows.get_mut(*at) {
                Some((name, _, short)) => {
                    name.pop();
                    *short = None;
                }
                None => drop(typed.pop()),
            },
            Some(Form::Who { typed, .. }) => drop(typed.pop()),
            Some(Form::Rename { typed, .. } | Form::Length { typed, .. } | Form::Snapshot { typed }) => drop(typed.pop()),
            Some(Form::Report { days, at: 1, .. }) => drop(days.pop()),
            Some(Form::Report { comment, at: 2, .. }) => drop(comment.pop()),
            _ => {}
        }
    }

    /// Space on the roadmap: the details beside it on or off; a window of `width` columns narrower
    /// than 150 has no room for them.
    pub fn toggle_details(&mut self, width: u16) {
        self.side ^= true;
        if self.side && width < 150 {
            self.message = Some("The details need 150 columns".into());
        }
    }

    /// Enter in a dialog: makes its changes as one and closes it, or says what is missing.
    pub fn keep_form(&mut self) {
        if matches!(self.form, Some(Form::Leave)) {
            self.form = None;
            self.keep();
            self.quit = true;
            return;
        }
        if let Some(Form::Snapshot { typed }) = &self.form {
            let name = Some(typed.trim().to_string()).filter(|name| !name.is_empty());
            let snapshot = pchart_core::snapshot::Snapshot::take(&self.plan, &self.schedule, self.today, name);
            let title = snapshot.title();
            let before = self.plan.clone();
            let mut plan = self.plan.clone();
            plan.keep(snapshot);
            let schedule = self.schedule.clone();
            self.take(plan, schedule);
            self.history.push((before, format!("snapshot {title}")));
            self.form = None;
            self.message = Some(format!("Kept: {title} · u takes it back"));
            return;
        }
        if let Some(Form::Report { answering: Some(path), .. }) = &self.form {
            let path = path.clone();
            match self.form_report() {
                Ok(edited) => self.take_edited(&path, &edited),
                Err(err) => self.message = Some(err),
            }
            return;
        }
        if matches!(self.form, Some(Form::Report { .. })) {
            self.send_report();
            return;
        }
        match self.form_edits() {
            Ok((edits, _)) if edits.is_empty() => self.form = None,
            Ok((edits, label)) => {
                let added = matches!(self.form, Some(Form::Add(_)));
                let next = match self.form {
                    Some(Form::Remove { item }) => {
                        let id = &self.plan.items[item].id;
                        self.plan.items.iter().find(|other| other.after.contains(id)).or_else(|| self.plan.items[item].after.first().and_then(|before| self.plan.item(before)).map(|before| &self.plan.items[before])).map(|other| other.id.clone())
                    }
                    _ => None,
                };
                let undated = !self.dated();
                let newcomer = match &self.form {
                    Some(Form::Who { item, at, typed, .. }) if *at >= self.who_rows(*item).len() => Some((typed.trim().to_string(), self.plan.lane(&self.plan.items[*item].lane).map(|lane| self.plan.lanes[lane].name.clone()))),
                    _ => None,
                };
                match self.apply_all(&edits, label) {
                    Ok(()) => {
                        if let Some((name, team)) = newcomer {
                            self.welcome(&name, team);
                        }
                        if undated && self.dated() {
                            self.message = Some("Every piece of work has a length now".into());
                        }
                        if added {
                            self.selected = self.plan.items.len() - 1;
                        }
                        if let Some(id) = next {
                            self.selected = self.plan.item(&id).unwrap_or(0);
                        }
                        self.selected = self.selected.min(self.plan.items.len() - 1);
                        self.form = None;
                    }
                    Err(err) => self.message = Some(err),
                }
            }
            Err(err) => self.message = Some(err),
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
                self.analysis = self.analysed(&plan, &schedule);
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
    /// `q` or Ctrl+C: true when pChart may quit now; a what-if that is not kept asks first.
    pub fn may_quit(&mut self) -> bool {
        if self.before.is_none() || self.changes().is_empty() {
            return true;
        }
        self.present = None;
        self.screen = Screen::Roadmap;
        self.form = Some(Form::Leave);
        false
    }

    pub fn discard(&mut self) {
        if let Some(before) = self.before.take() {
            self.plan = before.plan;
            self.schedule = before.schedule;
            self.analysis = before.analysis;
        }
    }

    /// `n` on the roadmap: the notes cursor on the selected item.
    pub fn add_notes(&mut self) {
        let item = &self.plan.items[self.selected];
        let slot = &self.schedule.slots[self.selected];
        let day = if item.milestone { slot.planned.finish } else { slot.planned.start };
        self.notes = Some(Notes { lane: self.plan.lane(&item.lane).unwrap_or(0), day, typed: None });
    }

    /// What the notes cursor stands on: a milestone on its day first, then an item running that day.
    pub fn spot(&self) -> Option<Spot> {
        let notes = self.notes.as_ref()?;
        let here = |index: &usize| self.plan.lane(&self.plan.items[*index].lane) == Some(notes.lane);
        let span = |index: usize| self.schedule.slots[index].planned;
        let milestone = (0..self.plan.items.len()).filter(here).find(|&index| self.plan.items[index].milestone && span(index).finish == notes.day);
        let running = (0..self.plan.items.len()).filter(here).find(|&index| span(index).start <= notes.day && notes.day <= span(index).finish);
        Some(milestone.or(running).map_or(Spot::Place(notes.lane, notes.day), Spot::Item))
    }

    fn follow(&mut self) {
        if let Some(Spot::Item(index)) = self.spot() {
            self.selected = index;
        }
    }

    /// `←→` in the notes mode: the cursor by days, within the roadmap.
    pub fn move_cursor(&mut self, days: i64) {
        let first = self.schedule.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(self.today).min(self.today);
        let last = self.schedule.slots.iter().map(|slot| slot.planned.finish).max().unwrap_or(self.today);
        let Some(notes) = self.notes.as_mut() else { return };
        let day = notes.day.checked_add(jiff::Span::new().days(days)).unwrap_or(notes.day);
        let end = last.checked_add(jiff::Span::new().days(28)).unwrap_or(last);
        notes.day = day.clamp(first, end);
        self.follow();
    }

    /// `↑↓` in the notes mode: the cursor to the lane above or below.
    pub fn cursor_lane(&mut self, step: isize) {
        let lanes = self.plan.lanes.len();
        let Some(notes) = self.notes.as_mut() else { return };
        notes.lane = notes.lane.saturating_add_signed(step).min(lanes.saturating_sub(1));
        self.follow();
    }

    /// The note where the cursor stands, as written.
    pub fn note_here(&self) -> Option<&str> {
        match self.spot()? {
            Spot::Item(index) => self.plan.items[index].note.as_deref(),
            Spot::Place(lane, day) => self.plan.notes.iter().find(|note| self.plan.lane(&note.lane) == Some(lane) && note.date == day).map(|note| note.text.as_str()),
        }
    }

    /// Enter in the notes mode: writes the note where the cursor stands, starting from the one there.
    pub fn write(&mut self) {
        let typed = self.note_here().unwrap_or_default().to_string();
        if let Some(notes) = self.notes.as_mut() {
            notes.typed = Some(typed);
        }
    }

    /// Enter while writing: keeps the note; an empty one removes it.
    pub fn keep_note(&mut self) {
        let (Some(spot), Some(typed)) = (self.spot(), self.notes.as_ref().and_then(|notes| notes.typed.clone())) else { return };
        let note = Some(typed).filter(|text| !text.trim().is_empty());
        let (edit, label) = match spot {
            Spot::Item(index) => (Edit::Note { item: self.plan.items[index].id.clone(), note }, format!("note on {}", self.plan.items[index].name)),
            Spot::Place(lane, day) => (Edit::NoteAt { lane: self.plan.lanes[lane].id.clone(), date: day, note }, format!("note on {} {}", self.plan.lanes[lane].name, day.strftime("%a %d %b"))),
        };
        match self.apply(&edit, label) {
            Ok(()) => self.notes.iter_mut().for_each(|notes| notes.typed = None),
            Err(err) => self.message = Some(err),
        }
    }

    /// `i` in the notes mode: marks the milestone under the cursor important, or no longer.
    pub fn toggle_important(&mut self) {
        let Some(Spot::Item(index)) = self.spot() else { return };
        self.mark_important(index);
    }

    /// Marks the milestone `index` important, or no longer.
    pub fn mark_important(&mut self, index: usize) {
        let item = &self.plan.items[index];
        if !item.milestone {
            return;
        }
        let (important, label) = (!item.important, format!("{} {}", item.name, if item.important { "not important" } else { "important" }));
        if let Err(err) = self.apply(&Edit::Important { item: item.id.clone(), important }, label) {
            self.message = Some(err);
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
