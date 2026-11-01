//! What can be done, by name (picture 50): the finder searches these by the words people use and runs
//! them as their keys would.

use pchart_draw::{Level, View};

use crate::app::{App, FORMATS, Screen};
use crate::ui::day;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ChangeDate,
    AddItem,
    Rename,
    WaitsOn,
    Remove,
    TryLength,
    AddNote,
    Important,
    Show,
    Undo,
    Detail,
    Details,
    Situation,
    Roadmap,
    Checks,
    Holidays,
    Export(View),
    Help,
    Quit,
}

/// A line the finder offers: an action, and the item it works on.
pub type Found = (Action, Option<usize>);

impl Action {
    fn all(app: &App) -> Vec<Action> {
        let mut all = vec![Action::ChangeDate, Action::AddItem, Action::Rename, Action::WaitsOn, Action::Remove, Action::TryLength, Action::AddNote, Action::Important, Action::Undo, Action::Detail, Action::Details, Action::Situation, Action::Roadmap, Action::Checks, Action::Holidays];
        all.push(Action::Export(app.export.view));
        all.extend(View::ALL.into_iter().filter(|&view| view != app.export.view).map(Action::Export));
        all.extend([Action::Help, Action::Quit]);
        all
    }

    fn on_item(self) -> bool {
        matches!(self, Action::ChangeDate | Action::AddItem | Action::Rename | Action::WaitsOn | Action::Remove | Action::TryLength | Action::AddNote | Action::Important | Action::Show)
    }

    fn fits(self, app: &App, item: usize) -> bool {
        match self {
            Action::TryLength => !app.plan.items[item].milestone,
            Action::Important => app.plan.items[item].milestone,
            _ => true,
        }
    }

    fn words(self) -> &'static [&'static str] {
        match self {
            Action::ChangeDate => &["move", "date", "start", "when", "shift", "later", "earlier", "fix", "fixed", "change", "reschedule", "postpone"],
            Action::AddItem => &["add", "new", "item", "task", "create", "insert", "work", "milestone"],
            Action::Rename => &["rename", "name", "call", "title"],
            Action::WaitsOn => &["waits", "wait", "depends", "dependency", "dependencies", "link", "after", "before", "predecessor"],
            Action::Remove => &["remove", "delete", "drop", "cancel"],
            Action::TryLength => &["what", "if", "try", "longer", "shorter", "length", "duration", "extend", "days"],
            Action::AddNote => &["note", "notes", "comment", "remark", "add", "write"],
            Action::Important => &["important", "star", "mark", "key"],
            Action::Show => &[],
            Action::Undo => &["undo", "back", "revert", "take"],
            Action::Details => &["details", "side", "panel", "info", "column"],
            Action::Detail => &["detail", "level", "overview", "standard", "full", "zoom", "more", "less"],
            Action::Situation => &["situation", "where", "stand", "status", "verdict", "next", "chain", "track"],
            Action::Roadmap => &["roadmap", "chart", "timeline", "lanes", "show"],
            Action::Checks => &["checks", "check", "problems", "warnings", "look", "buffer", "lint"],
            Action::Holidays => &["holidays", "holiday", "days", "off", "christmas", "public"],
            Action::Export(view) => match view {
                View::Gantt => &["export", "share", "page", "file", "pdf", "svg", "png", "markdown", "md", "print", "send", "gantt"],
                View::Milestones => &["export", "share", "page", "file", "pdf", "svg", "png", "markdown", "md", "print", "send", "milestones", "milestone"],
                View::Roadmap => &["export", "share", "page", "file", "pdf", "svg", "png", "markdown", "md", "print", "send", "roadmap", "swimlane", "swimlanes"],
                View::Phases => &["export", "share", "page", "file", "pdf", "svg", "png", "markdown", "md", "print", "send", "phase", "phases"],
                View::NowNextLater => &["export", "share", "page", "file", "pdf", "svg", "png", "markdown", "md", "print", "send", "now", "next", "later"],
                View::Structure => &["export", "share", "page", "file", "pdf", "svg", "png", "markdown", "md", "print", "send", "structure", "breakdown", "wbs", "psp", "built", "tree"],
                View::Network => &["export", "share", "page", "file", "pdf", "svg", "png", "markdown", "md", "print", "send", "network", "netzplan", "dependencies", "graph", "pert"],
            },
            Action::Help => &["help", "explain", "keys", "legend", "symbols", "what"],
            Action::Quit => &["quit", "exit", "close", "leave"],
        }
    }

    /// Its name as the finder lists it.
    pub fn name(self, app: &App, item: Option<usize>) -> String {
        match self {
            Action::ChangeDate => "Change a date".into(),
            Action::AddItem => "Add an item".into(),
            Action::Rename => "Rename".into(),
            Action::WaitsOn => "What it waits on".into(),
            Action::Remove => "Remove".into(),
            Action::TryLength => "Try longer or shorter".into(),
            Action::AddNote => "Add a note".into(),
            Action::Important if item.is_some_and(|index| app.plan.items[index].important) => "Not important".into(),
            Action::Important => "Mark as important".into(),
            Action::Show => item.map_or_else(String::new, |index| app.plan.items[index].name.clone()),
            Action::Undo => "Undo".into(),
            Action::Details if app.side => "Hide the details".into(),
            Action::Details => "Show the details".into(),
            Action::Detail => "Change the detail".into(),
            Action::Situation => "Where we stand".into(),
            Action::Roadmap => "Show the roadmap".into(),
            Action::Checks => "What to look at".into(),
            Action::Holidays => "Holidays".into(),
            Action::Export(View::NowNextLater) => "Export Now / Next / Later".into(),
            Action::Export(View::Gantt) => "Export the Gantt chart".into(),
            Action::Export(view) => format!("Export the {}", view.name().to_lowercase()),
            Action::Help => "Explain this screen".into(),
            Action::Quit => "Quit".into(),
        }
    }

    /// What it would do here, with the item it works on.
    pub fn what(self, app: &App, item: Option<usize>, typed: &str) -> String {
        let item_of = |index: usize| &app.plan.items[index];
        match (self, item) {
            (Action::ChangeDate, Some(index)) => {
                let it = item_of(index);
                let slot = app.schedule.slots[index].planned;
                let fixed = if it.on.is_some() { ", fixed" } else if it.no_earlier_than.is_some() { ", no earlier" } else if it.no_later_than.is_some() { ", no later" } else { "" };
                if it.milestone { format!("when {} is · now {}{fixed}", it.name, day(slot.finish)) } else { format!("when {} starts · now {}{fixed}", it.name, day(slot.start)) }
            }
            (Action::AddItem, Some(index)) => format!("after {}, in its lane", item_of(index).name),
            (Action::Rename, Some(index)) => format!("{} gets another name", item_of(index).name),
            (Action::WaitsOn, Some(index)) => {
                let on: Vec<&str> = item_of(index).after.iter().filter_map(|id| app.plan.item(id)).map(|other| app.plan.items[other].name.as_str()).collect();
                if on.is_empty() { format!("{} waits on nothing", item_of(index).name) } else { format!("{} waits on {}", item_of(index).name, on.join(", ")) }
            }
            (Action::Remove, Some(index)) => format!("{} leaves the plan; u brings it back", item_of(index).name),
            (Action::TryLength, Some(index)) => format!("{} on trial, nothing is saved · now {} wd", item_of(index).name, item_of(index).duration),
            (Action::AddNote, Some(index)) => match item_of(index).note.as_deref() {
                Some(note) => format!("{}: {note}", item_of(index).name),
                None => format!("on {}, or anywhere in the roadmap", item_of(index).name),
            },
            (Action::Important, Some(index)) if item_of(index).important => format!("{} loses its star", item_of(index).name),
            (Action::Important, Some(index)) => format!("{} gets a star and stays in every export", item_of(index).name),
            (Action::Show, Some(index)) => {
                let it = item_of(index);
                let slot = app.schedule.slots[index].planned;
                let lane = app.plan.lane(&it.lane).map_or("", |lane| app.plan.lanes[lane].name.as_str());
                if it.milestone {
                    format!("{lane} · milestone · {}{}", slot.finish.strftime("%a %d %b %Y"), if it.on.is_some() { " · fixed" } else { "" })
                } else {
                    format!("{lane} · {} – {} · {} wd", day(slot.start), day(slot.finish), it.duration)
                }
            }
            (Action::Undo, _) => app.history.last().map_or_else(|| "nothing to undo".to_string(), |(_, label)| format!("takes back: {label}")),
            (Action::Detail, _) => format!("Overview, Standard or Full · now {}", match app.level {
                Level::Overview => "Overview",
                Level::Standard => "Standard",
                Level::Full => "Full",
            }),
            (Action::Situation, _) => match &app.analysis.verdict {
                Some(verdict) => {
                    let name = &app.plan.items[verdict.target].name;
                    let date = day(app.schedule.slots[verdict.target].planned.finish);
                    if verdict.spare < 0 {
                        format!("{name} on {date} is {} wd late", -verdict.spare)
                    } else {
                        format!("{name} on {date} is {} · {} wd to spare", pchart_draw::situation::state_words(verdict.state), verdict.spare)
                    }
                }
                None => "the verdict, what is next, and the chain".into(),
            },
            (Action::Details, _) => "what it waits on, its holidays and the checks, beside the roadmap".into(),
            (Action::Roadmap, _) => "every item in its lane, over time".into(),
            (Action::Checks, _) => {
                let looks = app.analysis.checks.iter().filter(|check| check.severity() != pchart_core::analysis::Severity::Hint).count();
                match looks {
                    0 => "nothing to look at".into(),
                    1 => "1 thing to look at".into(),
                    count => format!("{count} things to look at"),
                }
            }
            (Action::Holidays, _) => {
                let days = crate::ui::holidays(app);
                let mut names: Vec<String> = Vec::new();
                for (_, _, name) in &days {
                    if !names.contains(name) {
                        names.push(name.clone());
                    }
                }
                format!("{} accounted for: {}", days.len(), names.join(", "))
            }
            (Action::Export(_), _) => {
                let formats = formats(app, typed);
                let audience = match app.export.audience {
                    pchart_draw::Audience::Management => "management",
                    pchart_draw::Audience::Client => "the client",
                    _ => "the team",
                };
                format!("for {audience} · {}", FORMATS.iter().zip(formats).filter(|(_, on)| *on).map(|(name, _)| *name).collect::<Vec<_>>().join(", "))
            }
            (Action::Help, _) => "what this screen answers, its keys and symbols".into(),
            (Action::Quit, _) => "every change is saved already".into(),
            _ => String::new(),
        }
    }

    /// The key that does the same.
    pub fn key(self) -> &'static str {
        match self {
            Action::ChangeDate => "Enter",
            Action::AddItem => "a",
            Action::Rename => "r",
            Action::WaitsOn => "w",
            Action::Remove => "x",
            Action::TryLength => "←→",
            Action::AddNote => "n",
            Action::Important => "n i",
            Action::Show => "",
            Action::Undo => "u",
            Action::Details => "Space",
            Action::Detail => "d",
            Action::Situation => "1",
            Action::Roadmap => "2",
            Action::Checks | Action::Holidays => "3",
            Action::Export(_) => "4",
            Action::Help => "?",
            Action::Quit => "q",
        }
    }

    /// Does it; true when it quits.
    pub fn run(self, app: &mut App, item: Option<usize>, typed: &str) -> bool {
        let roadmap = |app: &mut App| {
            app.screen = Screen::Roadmap;
            if let Some(index) = item {
                app.selected = index;
            }
            if app.level == Level::Overview && self != Action::Show {
                app.level = Level::Standard;
            }
        };
        match self {
            Action::ChangeDate => {
                roadmap(app);
                app.ask();
            }
            Action::AddItem => {
                roadmap(app);
                app.start_add();
            }
            Action::Rename => {
                roadmap(app);
                app.start_rename();
            }
            Action::WaitsOn => {
                roadmap(app);
                app.start_waits();
            }
            Action::Remove => {
                roadmap(app);
                app.start_remove();
            }
            Action::TryLength => {
                roadmap(app);
                app.message = Some(format!("←→ tries {} longer or shorter", app.plan.items[app.selected].name));
            }
            Action::AddNote => {
                roadmap(app);
                app.add_notes();
                app.write();
            }
            Action::Important => {
                roadmap(app);
                app.mark_important(app.selected);
            }
            Action::Show => roadmap(app),
            Action::Undo => app.undo(),
            Action::Details => {
                app.screen = Screen::Roadmap;
                app.side ^= true;
            }
            Action::Detail => {
                app.screen = Screen::Roadmap;
                app.detail();
            }
            Action::Situation => app.screen = Screen::Situation,
            Action::Roadmap => app.screen = Screen::Roadmap,
            Action::Checks => app.screen = Screen::Checks,
            Action::Holidays => {
                app.screen = Screen::Checks;
                app.check = app.checks().len() + 1;
            }
            Action::Export(view) => {
                app.export.formats = formats(app, typed);
                app.export.view = view;
                app.screen = Screen::Export;
            }
            Action::Help => app.help = true,
            Action::Quit => return true,
        }
        false
    }
}

/// The formats of the export, with those named in `typed` switched on.
fn formats(app: &App, typed: &str) -> [bool; 4] {
    let mut formats = app.export.formats;
    for word in typed.to_lowercase().split_whitespace() {
        match word {
            "pdf" => formats[0] = true,
            "svg" => formats[1] = true,
            "png" => formats[2] = true,
            "markdown" | "md" => formats[3] = true,
            _ => {}
        }
    }
    formats
}

fn words(text: &str) -> Vec<String> {
    text.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|word| !word.is_empty()).map(str::to_string).collect()
}

fn matches(word: &str, among: &[String]) -> bool {
    among.iter().any(|candidate| candidate.starts_with(word))
}

/// What the finder offers for `typed`: actions first, those that matched more words higher; then the
/// items. With nothing typed, the actions of this screen.
pub fn find(app: &App, typed: &str) -> Vec<Found> {
    let typed_words = words(typed);
    if typed_words.is_empty() {
        let selected = Some(app.selected);
        let mut found: Vec<Found> = match app.screen {
            Screen::Roadmap => vec![(Action::ChangeDate, selected), (Action::AddItem, selected), (Action::Rename, selected), (Action::WaitsOn, selected), (Action::TryLength, selected), (Action::AddNote, selected), (Action::Undo, None), (Action::Detail, None), (Action::Details, None)],
            Screen::Checks => vec![(Action::Holidays, None), (Action::Undo, None)],
            Screen::Export => vec![(Action::Export(app.export.view), None)],
            Screen::Situation => vec![(Action::Undo, None)],
        };
        found.retain(|&(action, _)| action != Action::Undo || !app.history.is_empty());
        found.retain(|&(action, item)| item.is_none_or(|index| action.fits(app, index)));
        found.extend([(Action::Situation, None), (Action::Roadmap, None), (Action::Checks, None), (Action::Export(app.export.view), None), (Action::Help, None), (Action::Quit, None)].into_iter().filter(|found_one| !(found_one.0 == Action::Situation && app.screen == Screen::Situation) && !(found_one.0 == Action::Roadmap && app.screen == Screen::Roadmap) && !(found_one.0 == Action::Checks && app.screen == Screen::Checks) && !(matches!(found_one.0, Action::Export(_)) && app.screen == Screen::Export)));
        return found;
    }
    let names: Vec<Vec<String>> = app.plan.items.iter().map(|item| words(&item.name)).collect();
    let mut scored: Vec<(usize, Found)> = Vec::new();
    for action in Action::all(app) {
        let own: Vec<String> = action.words().iter().map(|word| word.to_string()).chain(words(&action.name(app, None))).collect();
        let by_action = typed_words.iter().filter(|word| matches(word, &own)).count();
        let score = by_action + typed_words.iter().filter(|word| own.contains(word)).count();
        if action.on_item() {
            for &index in &app.schedule.order {
                if !action.fits(app, index) {
                    continue;
                }
                let by_item = typed_words.iter().filter(|word| matches(word, &names[index])).count();
                let all = typed_words.iter().all(|word| matches(word, &own) || matches(word, &names[index]));
                if by_item > 0 && all {
                    scored.push((score, (action, Some(index))));
                }
            }
            if by_action == typed_words.len() && action.fits(app, app.selected) && !scored.iter().any(|(_, (other, _))| *other == action) {
                scored.push((score, (action, Some(app.selected))));
            }
        } else if by_action == typed_words.len() {
            scored.push((score, (action, None)));
        }
    }
    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    let mut found: Vec<Found> = scored.into_iter().map(|(_, found)| found).collect();
    for &index in &app.schedule.order {
        let named = typed_words.iter().all(|word| matches(word, &names[index]));
        let paired = found.iter().any(|&(_, item)| item == Some(index)) && typed_words.iter().any(|word| matches(word, &names[index]));
        if named || paired {
            found.push((Action::Show, Some(index)));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_find_actions_and_the_items_they_work_on() {
        let plan = pchart_core::Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        let holidays = pchart_core::Holidays::parse(include_str!("../../../samples/holidays.tsv"));
        let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        let today = jiff::civil::date(2026, 10, 29);
        let schedule = pchart_core::schedule(&plan, &calendars, today).unwrap();
        let app = App::new(plan, calendars, today, "plan".into(), schedule, (10, 21));
        let launch = app.plan.item("launch").unwrap();
        assert_eq!(find(&app, "move launch"), [(Action::ChangeDate, Some(launch)), (Action::Show, Some(launch))]);
        assert!(find(&app, "pdf").iter().all(|(action, _)| matches!(action, Action::Export(_))));
        assert_eq!(find(&app, "holidays")[0], (Action::Holidays, None));
        assert_eq!(find(&app, "star sign")[0], (Action::Important, app.plan.item("design-sign-off")));
    }
}
