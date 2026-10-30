//! English sentences for statements. Plain language: "to spare", "waits on", "decides the date".
//! Item names can be singular or plural ("Frontend", "Requirements"), so no sentence makes a verb agree with one.

use jiff::civil::{Date, Weekday};

use crate::analysis::{Analysis, Check, Handoff, Next, State, Verdict};
use crate::model::Plan;
use crate::schedule::Schedule;

pub fn day(date: Date) -> String {
    date.strftime("%a %d %b").to_string()
}

pub fn working_days(count: i32) -> String {
    if count.abs() == 1 { format!("{count} working day") } else { format!("{count} working days") }
}

fn from_today(count: i32) -> String {
    if count == 0 { "today".into() } else { format!("{} from today", working_days(count)) }
}

fn name(plan: &Plan, item: usize) -> &str {
    &plan.items[item].name
}

/// A lane with its city: "Frontend (Bangalore)".
pub fn lane(plan: &Plan, lane: usize) -> String {
    let lane = &plan.lanes[lane];
    match &lane.city {
        Some(city) => format!("{} ({city})", lane.name),
        None => lane.name.clone(),
    }
}

/// The headline and the sentence below it.
pub fn verdict(plan: &Plan, verdict: &Verdict) -> (String, String) {
    let target = format!("{} on {}", name(plan, verdict.target), day(verdict.date));
    let first = name(plan, verdict.first);
    match verdict.state {
        State::Holds => (format!("{target} holds."), format!("{} to spare, with {first} starting by {}.", working_days(verdict.spare), day(verdict.latest_start))),
        State::Tight => (format!("{target} is tight."), format!("Only {} to spare, with {first} starting by {}.", working_days(verdict.spare), day(verdict.latest_start))),
        State::DoesNotHold => (format!("{target} does not hold."), format!("{} late, even with {first} starting today.", working_days(-verdict.spare))),
    }
}

pub fn handoff(plan: &Plan, handoff: &Handoff) -> String {
    format!("{} waits on {}: {} due {}.", lane(plan, handoff.waiting), lane(plan, handoff.on), name(plan, handoff.item), day(handoff.due))
}

pub fn next(plan: &Plan, analysis: &Analysis, next: &Next) -> String {
    match next {
        Next::StartBy { item, date, in_wd } => format!("Start {} by {}: {}.", name(plan, *item), day(*date), from_today(*in_wd)),
        Next::Due { item, date, in_wd } => format!("{} is due {}: {}.", name(plan, *item), day(*date), from_today(*in_wd)),
        Next::Check(index) => check(plan, &analysis.checks[*index]).0,
    }
}

/// The title of a check and the sentence that explains it.
pub fn check(plan: &Plan, check: &Check) -> (String, String) {
    let place = |item: usize| {
        let lane = &plan.lanes[plan.lane(&plan.items[item].lane).unwrap_or(0)];
        lane.city.clone().or_else(|| lane.place.clone()).unwrap_or_else(|| lane.name.clone())
    };
    match check {
        Check::TooLate { target, late } => (format!("{} does not hold.", name(plan, *target)), format!("The plan ends {} too late.", working_days(*late))),
        Check::MissedDeadline { item, by, finish } => (format!("{} misses its deadline {}.", name(plan, *item), day(*by)), format!("The earliest it can finish is {}.", day(*finish))),
        Check::FixedTooEarly { item, waits_on, ready } => {
            let fixed = plan.items[*item].on.map(day).unwrap_or_default();
            (format!("{} is fixed on {}, before {} is ready.", name(plan, *item), fixed, name(plan, *waits_on)), format!("{} is ready on {} at the earliest.", name(plan, *waits_on), day(*ready)))
        }
        Check::NoBuffer { before, target } => {
            let date = plan.items[*target].on.map(day).unwrap_or_default();
            (format!("No buffer before {} on {}.", name(plan, *target), date), format!("{} ends the working day before. Add a buffer?", name(plan, *before)))
        }
        Check::DayOff { item, date, holiday } => {
            let off = match holiday {
                Some(holiday) => format!("{holiday} in {}", place(*item)),
                None if matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday) => "a weekend".into(),
                None => format!("a day off in {}", place(*item)),
            };
            (format!("{} is fixed on a day off.", name(plan, *item)), format!("{} is {off}.", day(*date)))
        }
        Check::Overdue { item, finish } => (format!("{} is overdue.", name(plan, *item)), format!("It was planned to end {} and is not marked done.", day(*finish))),
        Check::Overlap { who, first, second } => (format!("{who} is planned on two items at once."), format!("{} and {} overlap.", name(plan, *first), name(plan, *second))),
        Check::LongItem { item } => {
            let days = plan.items[*item].duration as i32;
            (format!("{} takes {}.", name(plan, *item), working_days(days)), "Split it into parts of six weeks or less, so progress shows.".into())
        }
        Check::PhaseEndsWithoutMilestone { phase, item } => {
            (format!("The phase {} ends without a milestone.", plan.phases[*phase].name), format!("It ends with {}. A milestone marks when the phase is done.", name(plan, *item)))
        }
    }
}

pub fn chain(plan: &Plan, chain: &[usize]) -> String {
    chain.iter().map(|&item| name(plan, item)).collect::<Vec<_>>().join(" → ")
}

/// One line per step of a chain: name, lane and dates. What the chain has to spare is said once, for all of it.
pub fn chain_steps(plan: &Plan, schedule: &Schedule, chain: &[usize]) -> Vec<String> {
    let width = chain.iter().map(|&item| name(plan, item).chars().count()).max().unwrap_or(0);
    let lanes: Vec<String> = chain.iter().map(|&item| lane(plan, plan.lane(&plan.items[item].lane).unwrap_or(0))).collect();
    let lane_width = lanes.iter().map(|lane| lane.chars().count()).max().unwrap_or(0);
    chain
        .iter()
        .zip(&lanes)
        .map(|(&item, lane)| {
            let slot = &schedule.slots[item];
            let when = if plan.items[item].milestone { day(slot.planned.finish) } else { format!("{} – {}", day(slot.planned.start), day(slot.planned.finish)) };
            format!("{:<width$}  {:<lane_width$}  {when}", name(plan, item), lane)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Calendar, Holidays, analyse, schedule};
    use jiff::civil::date;

    #[test]
    fn the_sample_reads_in_plain_language() {
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
        let calendars: Vec<Calendar> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        let today = date(2026, 10, 29);
        let schedule = schedule(&plan, &calendars, today).unwrap();
        let analysis = analyse(&plan, &calendars, &schedule, today);
        let (headline, detail) = verdict(&plan, analysis.verdict.as_ref().unwrap());
        assert_eq!(headline, "Launch on Mon 01 Mar holds.");
        assert_eq!(detail, "12 working days to spare, with Requirements starting by Mon 16 Nov.");
        assert_eq!(handoff(&plan, &analysis.handoffs[2]), "Frontend (Bangalore) waits on Client (New York): Design sign-off due Fri 18 Dec.");
        assert_eq!(next(&plan, &analysis, &analysis.next[0]), "Start Requirements by Mon 16 Nov: 12 working days from today.");
        assert_eq!(next(&plan, &analysis, &analysis.next[1]), "No buffer before Launch on Mon 01 Mar.");
    }
}
