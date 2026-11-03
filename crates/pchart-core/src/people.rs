//! The people and teams of a workspace (pictures 64): who is in which team and when they are away, and
//! what that means across the projects of the workspace.

use jiff::civil::{Date, Weekday};
use serde::{Deserialize, Deserializer};

use crate::analysis::{Analysis, Check, Next, Severity, analyse};
use crate::calendar::Calendar;
use crate::model::Plan;
use crate::schedule::{Schedule, Span};
use crate::Error;

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Workspace {
    #[serde(default)]
    pub name: Option<String>,
    /// Who answers reports (pictures 65); without one, everyone may.
    #[serde(default)]
    pub lead: Option<String>,
    #[serde(default)]
    pub teams: Vec<Team>,
    #[serde(default)]
    pub people: Vec<Person>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Team {
    pub name: String,
    #[serde(default)]
    pub city: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Person {
    pub name: String,
    #[serde(default)]
    pub team: Option<String>,
    /// How much of their time they can work on projects, in percent; none is all of it.
    #[serde(default)]
    pub capacity: Option<u32>,
    #[serde(default)]
    pub away: Vec<Away>,
}

/// Days a person is away, the first and the last.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Away {
    #[serde(deserialize_with = "day")]
    pub from: Date,
    #[serde(deserialize_with = "day")]
    pub to: Date,
}

fn day<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Date, D::Error> {
    crate::model::date(deserializer)?.ok_or_else(|| serde::de::Error::custom("away needs from and to"))
}

impl Workspace {
    pub fn parse(text: &str) -> Result<Workspace, Error> {
        toml::from_str(text).map_err(|err| Error::File(err.to_string()))
    }

    pub fn person(&self, name: &str) -> Option<&Person> {
        self.people.iter().find(|person| person.name == name)
    }

    /// The city of `team`, as `workspace.toml` names it.
    pub fn city(&self, team: &str) -> Option<&str> {
        self.teams.iter().find(|other| other.name == team)?.city.as_deref()
    }
}

/// A project of the workspace with its dates and the calendars of its lanes.
#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub plan: Plan,
    pub schedule: Schedule,
    pub calendars: Vec<Calendar>,
}

impl Project {
    fn dated(&self) -> bool {
        self.plan.without_length().is_empty()
    }

    /// Whether `item` is worked on on `day`: within its dates, and a working day where its lane is.
    pub fn on(&self, item: usize, day: Date) -> bool {
        let span = self.schedule.slots[item].planned;
        span.start <= day && day <= span.finish && self.calendars.get(self.plan.lane(&self.plan.items[item].lane).unwrap_or(0)).is_none_or(|calendar| calendar.is_working(day))
    }
}

/// The workspace a project is in (pictures 64): its name, its people, and its projects with that one
/// at `here`.
#[derive(Debug, Clone)]
pub struct Crew {
    pub name: String,
    pub workspace: Workspace,
    pub projects: Vec<Project>,
    pub here: usize,
}

impl Crew {
    /// The analysis of the project `here` as `plan` and `schedule` are now, with the checks of people
    /// across the projects among the warnings and in what is next. They take the place of the plan's own
    /// check of two items at once for the people the workspace names.
    pub fn analyse(&mut self, plan: &Plan, calendars: &[Calendar], schedule: &Schedule, today: Date) -> Analysis {
        let mut analysis = analyse(plan, calendars, schedule, today);
        let kept: Vec<bool> = analysis.checks.iter().map(|check| !matches!(check, Check::Overlap { who, .. } if self.workspace.person(who).is_some())).collect();
        let mut at = 0;
        let moved: Vec<Option<usize>> = kept
            .iter()
            .map(|&keep| {
                at += usize::from(keep);
                keep.then(|| at - 1)
            })
            .collect();
        analysis.next.retain_mut(|next| match next {
            Next::Check(index) => moved[*index].map(|to| *index = to).is_some(),
            _ => true,
        });
        let mut keep = kept.iter();
        analysis.checks.retain(|_| *keep.next().unwrap_or(&true));
        self.projects[self.here].plan = plan.clone();
        self.projects[self.here].schedule = schedule.clone();
        self.projects[self.here].calendars = calendars.to_vec();
        let found = checks(&self.workspace, &self.projects, self.here, calendars);
        let at = analysis.checks.iter().position(|check| check.severity() == Severity::Hint).unwrap_or(analysis.checks.len());
        let after = analysis.next.iter().rposition(|next| matches!(next, Next::Check(_) | Next::StartBy { .. })).map_or(0, |last| last + 1);
        for (offset, check) in found.into_iter().enumerate() {
            analysis.checks.insert(at + offset, check);
            analysis.next.insert(after + offset, Next::Check(at + offset));
        }
        analysis
    }
}

/// The work of `who` across the projects, by start: the project, the item and its dates. Milestones and
/// work that is done do not count.
pub fn work(projects: &[Project], who: &str) -> Vec<(usize, usize, Span)> {
    let mut out: Vec<(usize, usize, Span)> = projects
        .iter()
        .enumerate()
        .filter(|(_, project)| project.dated())
        .flat_map(|(at, project)| {
            project.plan.items.iter().enumerate().filter(|(_, item)| item.who.as_deref() == Some(who) && !item.milestone && !item.done).map(move |(index, _)| (at, index, project.schedule.slots[index].planned))
        })
        .collect();
    out.sort_by_key(|&(_, _, span)| span.start);
    out
}

/// The share of `item` in `plan`, in percent.
pub fn share(plan: &Plan, item: usize) -> u32 {
    plan.items[item].share.unwrap_or(100)
}

fn weekday(day: Date) -> bool {
    !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday)
}

fn days(from: Date, to: Date) -> impl Iterator<Item = Date> {
    std::iter::successors(Some(from), |day| day.tomorrow().ok()).take_while(move |day| *day <= to)
}

/// How much of their time `who` is planned at on `day`, in percent, leaving out the item `skip`
/// (project, item).
pub fn load(projects: &[Project], who: &str, day: Date, skip: Option<(usize, usize)>) -> u32 {
    work(projects, who).into_iter().filter(|&(project, item, _)| Some((project, item)) != skip && projects[project].on(item, day)).map(|(project, item, _)| share(&projects[project].plan, item)).sum()
}

impl Workspace {
    /// How much of their time `who` can work, in percent: 100 unless `workspace.toml` says less.
    pub fn capacity(&self, who: &str) -> u32 {
        self.person(who).and_then(|person| person.capacity).unwrap_or(100)
    }

    /// Whether `who` is away on `day`.
    pub fn away(&self, who: &str, day: Date) -> Option<Away> {
        self.person(who)?.away.iter().find(|away| away.from <= day && day <= away.to).copied()
    }
}

/// How someone stands from `from` to `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stand {
    /// Away on some of the days.
    Away(Away),
    /// The most of their time other work takes on one of the weekdays, in percent, and what they can do.
    Taken { most: u32, capacity: u32 },
}

/// How `who` stands from `from` to `to`, leaving out the item `skip` (project, item).
pub fn stand(workspace: &Workspace, projects: &[Project], who: &str, from: Date, to: Date, skip: Option<(usize, usize)>) -> Stand {
    if let Some(away) = workspace.person(who).and_then(|person| person.away.iter().find(|away| away.from <= to && from <= away.to)) {
        return Stand::Away(*away);
    }
    let most = days(from, to).filter(|&day| weekday(day)).map(|day| load(projects, who, day, skip)).max().unwrap_or(0);
    Stand::Taken { most, capacity: workspace.capacity(who) }
}

/// Someone else of the team of `who` who has `share` of their time from `from` to `to`.
fn free(workspace: &Workspace, projects: &[Project], who: &str, from: Date, to: Date, share: u32) -> Option<String> {
    let team = workspace.person(who)?.team.as_deref()?;
    workspace
        .people
        .iter()
        .find(|other| other.name != who && other.team.as_deref() == Some(team) && matches!(stand(workspace, projects, &other.name, from, to, None), Stand::Taken { most, capacity } if most + share <= capacity))
        .map(|other| other.name.clone())
}

fn working(calendar: &Calendar, from: Date, to: Date) -> i32 {
    days(from, to).filter(|&day| calendar.is_working(day)).count() as i32
}

/// The checks of people for the project `here`: someone planned at more of their time than they have,
/// and someone planned while they are away. `calendars` are those of `here`'s lanes.
pub fn checks(workspace: &Workspace, projects: &[Project], here: usize, calendars: &[Calendar]) -> Vec<Check> {
    let Some(project) = projects.get(here).filter(|project| project.dated()) else { return Vec::new() };
    let plan = &project.plan;
    let mut out = Vec::new();
    for (index, item) in plan.items.iter().enumerate() {
        let Some(who) = item.who.as_deref() else { continue };
        if item.milestone || item.done {
            continue;
        }
        let span = project.schedule.slots[index].planned;
        let calendar = &calendars[plan.lane(&item.lane).unwrap_or(0)];
        let capacity = workspace.capacity(who);
        let over: Vec<(Date, u32)> = days(span.start, span.finish).filter(|&day| calendar.is_working(day) && workspace.away(who, day).is_none()).map(|day| (day, load(projects, who, day, None))).filter(|&(_, load)| load > capacity).collect();
        let mut stretches: Vec<Vec<(Date, u32)>> = Vec::new();
        for (day, load) in over {
            match stretches.last_mut() {
                Some(stretch) if working(calendar, stretch[stretch.len() - 1].0, day) == 2 => stretch.push((day, load)),
                _ => stretches.push(vec![(day, load)]),
            }
        }
        for stretch in stretches {
            let (from, to) = (stretch[0].0, stretch[stretch.len() - 1].0);
            if out.iter().any(|check| matches!(check, Check::Overload { who: other, from: f, to: t, .. } if other == who && *f == from && *t == to)) {
                continue;
            }
            let most = stretch.iter().map(|&(_, load)| load).max().unwrap_or(0);
            let with = work(projects, who)
                .into_iter()
                .filter(|&(at, other, theirs)| (at, other) != (here, index) && theirs.start <= to && from <= theirs.finish)
                .map(|(at, other, _)| if at == here { format!("{} here", projects[at].plan.items[other].name) } else { format!("{} in {}", projects[at].plan.items[other].name, projects[at].name) })
                .collect();
            out.push(Check::Overload { who: who.to_string(), item: index, from, to, load: most, capacity, with, free: free(workspace, projects, who, span.start, span.finish, share(plan, index)) });
        }
        for away in workspace.person(who).map(|person| person.away.as_slice()).unwrap_or_default() {
            if away.from > span.finish || span.start > away.to {
                continue;
            }
            let (from, to) = (span.start.max(away.from), span.finish.min(away.to));
            let days = working(calendar, from, to);
            if days > 0 {
                out.push(Check::Away { who: who.to_string(), item: index, from: away.from, to: away.to, days });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::Holidays;

    fn project(name: &str, items: &str, holidays: &str) -> Project {
        let plan = Plan::parse(&format!("format = 1\nname = \"{name}\"\nstart = 2027-03-01\n[[lanes]]\nid = \"a\"\nname = \"A\"\nplace = \"X\"\n{items}")).unwrap();
        let calendars = vec![Holidays::parse(holidays).calendar(Some("X"))];
        let schedule = crate::schedule(&plan, &calendars, "2027-03-01".parse().unwrap()).unwrap();
        Project { name: name.into(), plan, schedule, calendars }
    }

    fn item(id: &str, days: u32, who: &str, share: u32, after: &str) -> String {
        let after = if after.is_empty() { String::new() } else { format!("after = [\"{after}\"]\n") };
        format!("[[items]]\nid = \"{id}\"\nlane = \"a\"\nname = \"{id}\"\nduration = \"{days}wd\"\nwho = \"{who}\"\nshare = {share}\n{after}")
    }

    fn workspace(text: &str) -> Workspace {
        Workspace::parse(text).unwrap()
    }

    fn overloads(checks: &[Check]) -> Vec<(Date, Date, u32)> {
        checks.iter().filter_map(|check| if let Check::Overload { from, to, load, .. } = check { Some((*from, *to, *load)) } else { None }).collect()
    }

    #[test]
    fn two_pieces_of_work_of_one_person_in_one_plan_are_one_warning() {
        let here = project("P", &(item("x", 10, "Lena", 60, "") + &item("y", 10, "Lena", 60, "")), "");
        let calendars = here.calendars.clone();
        let found = overloads(&checks(&workspace(""), &[here], 0, &calendars));
        assert_eq!(found, vec![("2027-03-01".parse().unwrap(), "2027-03-12".parse().unwrap(), 120)]);
    }

    #[test]
    fn every_stretch_of_too_much_is_its_own_warning() {
        let here = project("P", &item("long", 20, "Lena", 60, ""), "");
        let other = project("Q", &(item("a", 3, "Lena", 50, "") + &item("gap", 5, "Nobody", 100, "a") + &item("b", 3, "Lena", 50, "gap")), "");
        let calendars = here.calendars.clone();
        let found = overloads(&checks(&workspace(""), &[here, other], 0, &calendars));
        assert_eq!(found.len(), 2, "{found:?}");
    }

    #[test]
    fn someone_is_offered_only_when_they_have_the_time_for_all_of_the_work() {
        let here = project("P", &item("x", 10, "Lena", 80, ""), "");
        let other = project("Q", &(item("y", 2, "Lena", 50, "") + &item("wait", 5, "Nobody", 100, "") + &item("z", 5, "Mia", 100, "wait")), "");
        let team = workspace("people = [{ name = \"Lena\", team = \"T\" }, { name = \"Mia\", team = \"T\" }, { name = \"Tom\", team = \"T\", capacity = 50 }]");
        let calendars = here.calendars.clone();
        let found = checks(&team, &[here, other], 0, &calendars);
        assert!(found.iter().any(|check| matches!(check, Check::Overload { free: None, .. })), "Mia is busy later in x and Tom has only 50 %: {found:?}");
    }

    #[test]
    fn work_on_a_holiday_where_it_is_done_does_not_count() {
        let here = project("P", &item("x", 5, "Lena", 60, ""), "X\t2027-03-03\tFeast\n");
        let other = project("Q", &item("y", 5, "Lena", 60, ""), "Y\t2027-03-03\tOther\n");
        let projects = [here, other];
        let day: Date = "2027-03-03".parse().unwrap();
        assert_eq!(load(&projects, "Lena", day, None), 60);
        assert_eq!(load(&projects, "Lena", "2027-03-02".parse().unwrap(), None), 120);
    }
}
