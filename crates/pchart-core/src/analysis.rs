use jiff::civil::{Date, Weekday};

use crate::calendar::{Calendar, next};
use crate::model::Plan;
use crate::schedule::{Schedule, available};

/// Below this many working days to spare, a target is tight.
pub const TIGHT: i32 = 5;
/// Items longer than this many working days (six weeks) hide their progress.
pub const LONG: u32 = 30;
/// How far ahead, in working days, chain items count as due next.
pub const SOON: i32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Holds,
    Tight,
    DoesNotHold,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub state: State,
    pub target: usize,
    pub date: Date,
    pub spare: i32,
    /// The first item of the chain that decides the date, and its latest start.
    pub first: usize,
    pub latest_start: Date,
}

/// One lane waiting on another for an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handoff {
    pub waiting: usize,
    pub on: usize,
    pub item: usize,
    pub due: Date,
    pub deciding: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Problem,
    Warning,
    Hint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    TooLate { target: usize, late: i32 },
    MissedDeadline { item: usize, by: Date, finish: Date },
    FixedTooEarly { item: usize, waits_on: usize, ready: Date },
    NoBuffer { before: usize, target: usize },
    DayOff { item: usize, date: Date, holiday: Option<String> },
    Overdue { item: usize, finish: Date },
    Overlap { who: String, first: usize, second: usize },
    LongItem { item: usize },
    PhaseEndsWithoutMilestone { phase: usize, item: usize },
}

impl Check {
    pub fn severity(&self) -> Severity {
        match self {
            Check::TooLate { .. } | Check::MissedDeadline { .. } | Check::FixedTooEarly { .. } => Severity::Problem,
            Check::NoBuffer { .. } | Check::DayOff { .. } | Check::Overdue { .. } | Check::Overlap { .. } => Severity::Warning,
            Check::LongItem { .. } | Check::PhaseEndsWithoutMilestone { .. } => Severity::Hint,
        }
    }

    /// The items a check is about.
    pub fn items(&self) -> Vec<usize> {
        match self {
            Check::TooLate { target: item, .. }
            | Check::MissedDeadline { item, .. }
            | Check::DayOff { item, .. }
            | Check::Overdue { item, .. }
            | Check::LongItem { item }
            | Check::PhaseEndsWithoutMilestone { item, .. } => vec![*item],
            Check::FixedTooEarly { item, waits_on, .. } => vec![*item, *waits_on],
            Check::NoBuffer { before, target } => vec![*before, *target],
            Check::Overlap { first, second, .. } => vec![*first, *second],
        }
    }

    pub fn rule(&self) -> &'static str {
        match self {
            Check::TooLate { .. } => "target-too-late",
            Check::MissedDeadline { .. } => "deadline-missed",
            Check::FixedTooEarly { .. } => "fixed-before-predecessor",
            Check::NoBuffer { .. } => "no-buffer-before-fixed-date",
            Check::DayOff { .. } => "fixed-date-on-day-off",
            Check::Overdue { .. } => "overdue",
            Check::Overlap { .. } => "overlap",
            Check::LongItem { .. } => "long-item",
            Check::PhaseEndsWithoutMilestone { .. } => "phase-without-milestone",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Next {
    StartBy { item: usize, date: Date, in_wd: i32 },
    Check(usize),
    Due { item: usize, date: Date, in_wd: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub verdict: Option<Verdict>,
    pub handoffs: Vec<Handoff>,
    pub checks: Vec<Check>,
    pub next: Vec<Next>,
}

pub fn analyse(plan: &Plan, calendars: &[Calendar], schedule: &Schedule, today: Date) -> Analysis {
    let lane = |index: usize| plan.lane(&plan.items[index].lane).unwrap_or(0);
    let cal = |index: usize| &calendars[lane(index)];
    let verdict = verdict(plan, schedule);

    let mut handoffs: Vec<Handoff> = Vec::new();
    for (index, item) in plan.items.iter().enumerate() {
        for pred in item.after.iter().filter_map(|id| plan.item(id)) {
            let (waiting, on) = (lane(index), lane(pred));
            if waiting == on || handoffs.iter().any(|handoff| handoff.waiting == waiting && handoff.item == pred) {
                continue;
            }
            let deciding = schedule.chain.contains(&pred) && schedule.chain.contains(&index);
            handoffs.push(Handoff { waiting, on, item: pred, due: schedule.slots[pred].planned.finish, deciding });
        }
    }
    handoffs.sort_by_key(|handoff| (!handoff.deciding, handoff.due));

    let checks = checks(plan, calendars, schedule, today, verdict.as_ref());

    let mut upcoming = Vec::new();
    if let Some(verdict) = &verdict
        && verdict.spare >= 0
        && !plan.items[verdict.first].done
    {
        upcoming.push(Next::StartBy { item: verdict.first, date: verdict.latest_start, in_wd: cal(verdict.first).between(today, verdict.latest_start) });
    }
    let mut serious: Vec<usize> = (0..checks.len()).filter(|&index| checks[index].severity() != Severity::Hint).collect();
    serious.sort_by_key(|&index| checks[index].severity());
    upcoming.extend(serious.into_iter().map(Next::Check));
    for &index in &schedule.chain {
        let finish = schedule.slots[index].planned.finish;
        let in_wd = cal(index).between(today, finish);
        let first = verdict.as_ref().is_some_and(|verdict| verdict.first == index);
        if !plan.items[index].done && !first && (0..=SOON).contains(&in_wd) {
            upcoming.push(Next::Due { item: index, date: finish, in_wd });
        }
    }
    Analysis { verdict, handoffs, checks, next: upcoming }
}

fn verdict(plan: &Plan, schedule: &Schedule) -> Option<Verdict> {
    let target = schedule.target?;
    let spare = schedule.spare()?;
    let first = schedule.first?;
    let state = if spare < 0 {
        State::DoesNotHold
    } else if spare < TIGHT {
        State::Tight
    } else {
        State::Holds
    };
    let item = &plan.items[target];
    let date = item.on.or(item.no_later_than).unwrap_or(schedule.slots[target].planned.finish);
    Some(Verdict { state, target, date, spare, first, latest_start: schedule.start_by? })
}

fn checks(plan: &Plan, calendars: &[Calendar], schedule: &Schedule, today: Date, verdict: Option<&Verdict>) -> Vec<Check> {
    let lane = |index: usize| plan.lane(&plan.items[index].lane).unwrap_or(0);
    let cal = |index: usize| &calendars[lane(index)];
    let preds = |index: usize| plan.items[index].after.iter().filter_map(|id| plan.item(id)).collect::<Vec<_>>();
    let target = verdict.map(|verdict| verdict.target);
    let mut out = Vec::new();

    if let Some(verdict) = verdict
        && verdict.spare < 0
    {
        out.push(Check::TooLate { target: verdict.target, late: -verdict.spare });
    }
    for (index, item) in plan.items.iter().enumerate() {
        let slot = &schedule.slots[index];
        if let Some(by) = item.no_later_than
            && Some(index) != target
            && slot.earliest.finish > by
        {
            out.push(Check::MissedDeadline { item: index, by, finish: slot.earliest.finish });
        }
        if let Some(day) = item.on
            && Some(index) != target
            && let Some((ready, pred)) = preds(index).into_iter().map(|pred| (available(plan, pred, schedule.slots[pred].earliest), pred)).max()
            && ready > day
        {
            out.push(Check::FixedTooEarly { item: index, waits_on: pred, ready });
        }
        if let Some(day) = item.on
            && !preds(index).is_empty()
            && let Some(&before) = preds(index).iter().max_by_key(|&&pred| schedule.slots[pred].planned.finish)
            && !plan.items[before].buffer
            && cal(index).between(next(schedule.slots[before].planned.finish), day) == 0
            && verdict.is_none_or(|verdict| verdict.spare >= 0 || verdict.target != index)
        {
            out.push(Check::NoBuffer { before, target: index });
        }
        for day in [item.on, item.no_earlier_than, item.no_later_than].into_iter().flatten() {
            if !cal(index).is_working(day) {
                let holiday = cal(index).holiday(day).map(str::to_string);
                let weekend = matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday);
                if holiday.is_some() || weekend {
                    out.push(Check::DayOff { item: index, date: day, holiday });
                }
            }
        }
        if !item.done && slot.planned.finish < today {
            out.push(Check::Overdue { item: index, finish: slot.planned.finish });
        }
        if !item.milestone && item.duration > LONG {
            out.push(Check::LongItem { item: index });
        }
    }
    for first in 0..plan.items.len() {
        for second in first + 1..plan.items.len() {
            let (a, b) = (&plan.items[first], &plan.items[second]);
            if let (Some(who), Some(other)) = (&a.who, &b.who)
                && who == other
                && !a.milestone
                && !b.milestone
            {
                let (x, y) = (schedule.slots[first].planned, schedule.slots[second].planned);
                if x.start <= y.finish && y.start <= x.finish {
                    out.push(Check::Overlap { who: who.clone(), first, second });
                }
            }
        }
    }
    for (index, phase) in plan.phases.iter().enumerate() {
        if let Some(item) = plan.item(&phase.until)
            && !plan.items[item].milestone
        {
            out.push(Check::PhaseEndsWithoutMilestone { phase: index, item });
        }
    }
    out.sort_by_key(Check::severity);
    out
}

/// The chain that sets the date of `item`, first item first.
pub fn why(schedule: &Schedule, item: usize) -> Vec<usize> {
    let mut chain = vec![item];
    let mut index = item;
    while let Some(driver) = schedule.slots[index].driver {
        chain.push(driver);
        index = driver;
    }
    chain.reverse();
    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::Edit;
    use crate::model::Item;
    use crate::{Holidays, schedule};
    use jiff::civil::date;

    fn sample() -> (Plan, Vec<Calendar>) {
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
        let calendars = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        (plan, calendars)
    }

    fn run(plan: &Plan, calendars: &[Calendar], today: Date) -> Analysis {
        let schedule = schedule(plan, calendars, today).unwrap();
        analyse(plan, calendars, &schedule, today)
    }

    #[test]
    fn the_sample_holds_with_twelve_days_to_spare() {
        let (plan, calendars) = sample();
        let analysis = run(&plan, &calendars, date(2026, 10, 29));
        let verdict = analysis.verdict.unwrap();
        assert_eq!((verdict.state, verdict.spare, verdict.latest_start), (State::Holds, 12, date(2026, 11, 16)));
        assert_eq!(plan.items[verdict.first].id, "requirements");
        assert_eq!(analysis.next[0], Next::StartBy { item: verdict.first, date: date(2026, 11, 16), in_wd: 12 });
    }

    #[test]
    fn handoffs_on_the_deciding_chain_come_first() {
        let (plan, calendars) = sample();
        let analysis = run(&plan, &calendars, date(2026, 10, 29));
        let deciding: Vec<(&str, &str, &str)> = analysis
            .handoffs
            .iter()
            .filter(|handoff| handoff.deciding)
            .map(|handoff| (plan.lanes[handoff.waiting].id.as_str(), plan.lanes[handoff.on].id.as_str(), plan.items[handoff.item].id.as_str()))
            .collect();
        assert_eq!(deciding, [("design", "product", "requirements"), ("client", "design", "ui-design"), ("frontend", "client", "design-sign-off"), ("qa", "frontend", "feature-freeze"), ("client", "qa", "beta-test")]);
        assert!(analysis.handoffs.iter().skip(deciding.len()).all(|handoff| !handoff.deciding));
    }

    #[test]
    fn the_sample_warns_about_no_buffer_before_launch() {
        let (plan, calendars) = sample();
        let analysis = run(&plan, &calendars, date(2026, 10, 29));
        let launch = plan.item("launch").unwrap();
        let acceptance = plan.item("acceptance").unwrap();
        let serious: Vec<&Check> = analysis.checks.iter().filter(|check| check.severity() != Severity::Hint).collect();
        assert_eq!(serious, [&Check::NoBuffer { before: acceptance, target: launch }]);
        assert!(analysis.checks.contains(&Check::PhaseEndsWithoutMilestone { phase: 0, item: plan.item("requirements").unwrap() }));
    }

    #[test]
    fn only_an_item_marked_as_buffer_ends_the_warning() {
        let (plan, calendars) = sample();
        let with = |buffer| {
            let item = Item { id: "spare".into(), lane: "client".into(), name: "Buffer".into(), duration: 5, after: vec!["acceptance".into()], buffer, ..Item::default() };
            crate::edit::apply_all(&plan, &[Edit::Add { item }, Edit::Link { from: "spare".into(), to: "launch".into() }]).unwrap()
        };
        let warns = |plan: &Plan| run(plan, &calendars, date(2026, 10, 29)).checks.iter().any(|check| matches!(check, Check::NoBuffer { .. }));
        assert!(warns(&with(false)));
        assert!(!warns(&with(true)));
    }

    #[test]
    fn a_late_start_does_not_hold() {
        let (plan, calendars) = sample();
        let analysis = run(&plan, &calendars, date(2026, 12, 3));
        let verdict = analysis.verdict.unwrap();
        assert_eq!(verdict.state, State::DoesNotHold);
        assert!(matches!(analysis.checks[0], Check::TooLate { late, .. } if late == -verdict.spare && late > 0));
        assert!(!analysis.checks.iter().any(|check| matches!(check, Check::NoBuffer { .. })));
    }

    #[test]
    fn the_same_person_on_two_items_at_once_overlaps() {
        let (mut plan, calendars) = sample();
        for id in ["api", "sdk-integration"] {
            let index = plan.item(id).unwrap();
            plan.items[index].who = Some("Anna".into());
        }
        let analysis = run(&plan, &calendars, date(2026, 10, 29));
        assert!(analysis.checks.iter().any(|check| matches!(check, Check::Overlap { who, .. } if who == "Anna")));
    }

    #[test]
    fn a_fixed_date_on_a_holiday_is_reported() {
        let (mut plan, calendars) = sample();
        let index = plan.item("security-review").unwrap();
        plan.items[index].no_earlier_than = Some(date(2026, 12, 25));
        let analysis = run(&plan, &calendars, date(2026, 10, 29));
        assert!(analysis.checks.contains(&Check::DayOff { item: index, date: date(2026, 12, 25), holiday: Some("Christmas Day".into()) }));
    }

    #[test]
    fn a_fixed_start_before_its_predecessor_is_ready_is_a_problem() {
        let (mut plan, calendars) = sample();
        let index = plan.item("frontend").unwrap();
        plan.items[index].on = Some(date(2026, 12, 1));
        let analysis = run(&plan, &calendars, date(2026, 10, 29));
        let sign_off = plan.item("design-sign-off").unwrap();
        assert!(analysis.checks.iter().any(|check| matches!(check, Check::FixedTooEarly { item, waits_on, .. } if *item == index && *waits_on == sign_off)));
        assert_eq!(analysis.checks[0].severity(), Severity::Problem);
    }

    #[test]
    fn why_follows_the_drivers() {
        let (plan, calendars) = sample();
        let schedule = schedule(&plan, &calendars, date(2026, 10, 29)).unwrap();
        let chain: Vec<&str> = why(&schedule, plan.item("integration").unwrap()).into_iter().map(|index| plan.items[index].id.as_str()).collect();
        assert_eq!(chain, ["requirements", "ux-concept", "ui-design", "design-sign-off", "frontend", "integration"]);
    }
}
