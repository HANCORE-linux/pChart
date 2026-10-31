//! English sentences for statements. Plain language: "to spare", "waits on", "decides the date".
//! Item names can be singular or plural ("Frontend", "Requirements"), so no sentence makes a verb agree with one.

use jiff::civil::{Date, Weekday};

use crate::analysis::{Analysis, Check, Handoff, Next, State, Verdict, analyse};
use crate::calendar::Calendar;
use crate::edit::{Edit, apply_all};
use crate::model::{Item, Plan};
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
        State::Holds => (format!("{target} is on track."), format!("{} to spare, with {first} starting by {}.", working_days(verdict.spare), day(verdict.latest_start))),
        State::Tight => (format!("{target} is at risk."), format!("Only {} to spare, with {first} starting by {}.", working_days(verdict.spare), day(verdict.latest_start))),
        State::DoesNotHold => (format!("{target} is late."), format!("{} late, even with {first} starting today.", working_days(-verdict.spare))),
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
        Check::TooLate { target, late } => (format!("{} is late.", name(plan, *target)), format!("The plan ends {} too late.", working_days(*late))),
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

/// A run of text; `true` marks a number or date that stands out.
pub type Run = (String, bool);

/// What a check means and what can be done about it. Each line of `means` and `can_do` starts on a
/// line of its own; `short` says both in one line each, for small windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Advice {
    pub means: Vec<Vec<Run>>,
    pub can_do: Vec<Vec<Run>>,
    pub short: [Vec<Run>; 2],
}

/// The buffer the Checks screen suggests before a fixed date, in working days.
pub const BUFFER: u32 = 5;

fn plain(text: String) -> Vec<Run> {
    vec![(text, false)]
}

/// What `check` means for this plan and what can be done about it. A missing buffer is answered with
/// the plan as it would be with one.
pub fn advice(plan: &Plan, calendars: &[Calendar], schedule: &Schedule, today: Date, check: &Check) -> Advice {
    let planned = |item: usize| &schedule.slots[item].planned;
    let place = |item: usize| {
        let lane = &plan.lanes[plan.lane(&plan.items[item].lane).unwrap_or(0)];
        lane.city.clone().or_else(|| lane.place.clone()).unwrap_or_else(|| lane.name.clone())
    };
    let both = |means: Vec<String>, can_do: String, short: [String; 2]| Advice { means: means.into_iter().map(plain).collect(), can_do: vec![plain(can_do)], short: short.map(plain) };
    match *check {
        Check::TooLate { target, late } => {
            let t = name(plan, target);
            both(
                vec![format!("{t} is fixed on {}, but the work before it ends {} later.", day(planned(target).finish), working_days(late))],
                format!("Shorten a step of the chain that decides the date, let steps overlap, or move {t}."),
                [format!("The work before {t} ends {} too late.", working_days(late)), "Shorten the chain that decides the date.".into()],
            )
        }
        Check::MissedDeadline { item, by, finish } => {
            let i = name(plan, item);
            both(vec![format!("{i} has to be done by {}. As planned, it ends {}.", day(by), day(finish))], "Start the work before it earlier, or make it shorter.".into(), [format!("{i} ends {}, after its deadline.", day(finish)), "Start earlier or make it shorter.".into()])
        }
        Check::FixedTooEarly { item, waits_on, ready } => {
            let (i, w) = (name(plan, item), name(plan, waits_on));
            let fixed = plan.items[item].on.map(day).unwrap_or_default();
            both(vec![format!("{i} is fixed on {fixed}, but it waits on {w}, which is ready on {} at the earliest.", day(ready))], format!("Move {i} to {} or later, or let it start when {w} is ready.", day(ready)), [format!("{w} is not ready before {}.", day(ready)), format!("Move {i} to {} or later.", day(ready))])
        }
        Check::NoBuffer { before, target } => {
            let (b, t) = (name(plan, before), name(plan, target));
            let means = vec![format!("{t} is fixed on {}. {b} ends on {}, the working day before.", day(planned(target).finish), day(planned(before).finish)), format!("If {b} or anything before it slips by a day, {t} is late.")];
            let mut can_do = vec![(format!("A buffer of {} before {t} takes small delays.", working_days(BUFFER as i32)), false)];
            let mut short = vec![(format!("A {BUFFER} wd buffer before it"), false)];
            match with_buffer(plan, calendars, today, before, target) {
                Some(verdict) if verdict.state == State::DoesNotHold => {
                    can_do.extend([(" The plan is then ".into(), false), (working_days(-verdict.spare), true), (" late.".into(), false)]);
                    short.extend([(" makes it ".into(), false), (format!("{} wd", -verdict.spare), true), (" late.".into(), false)]);
                }
                Some(verdict) => {
                    let first = name(plan, verdict.first);
                    can_do.extend([(" The plan is then still on track, with ".into(), false), (verdict.spare.to_string(), true), (format!(" working {} to spare, with {first} starting by ", if verdict.spare == 1 { "day" } else { "days" }), false), (day(verdict.latest_start), true), (".".into(), false)]);
                    short.extend([(" leaves ".into(), false), (verdict.spare.to_string(), true), (" wd to spare.".into(), false)]);
                }
                None => short.push((" takes small delays.".into(), false)),
            }
            Advice { means: means.into_iter().map(plain).collect(), can_do: vec![can_do], short: [plain(format!("If {b} slips by a day, {t} is late.")), short] }
        }
        Check::DayOff { item, date, ref holiday } => {
            let i = name(plan, item);
            let off = match holiday {
                Some(holiday) => format!("{holiday} in {}", place(item)),
                None if matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday) => "a weekend".into(),
                None => format!("a day off in {}", place(item)),
            };
            both(vec![format!("{i} is fixed on {}, which is {off}.", day(date))], format!("Move it to a working day in {}.", place(item)), [format!("{} is {off}.", day(date)), "Move it to a working day.".into()])
        }
        Check::Overdue { item, finish } => {
            let i = name(plan, item);
            both(vec![format!("{i} was planned to end {} and is not marked done.", day(finish))], "Mark it done, or give it the days it still needs.".into(), [format!("Planned to end {}, not marked done.", day(finish)), "Mark it done or give it more days.".into()])
        }
        Check::Overlap { ref who, first, second } => {
            let (a, b) = (name(plan, first), name(plan, second));
            both(vec![format!("{who} is planned on {a} and {b} at the same time.")], "Let one wait on the other, or give one of them to someone else.".into(), [format!("{who} is on {a} and {b} at once."), "Let one wait on the other.".into()])
        }
        Check::LongItem { item } => {
            let i = name(plan, item);
            both(vec![format!("{i} takes {}. Over six weeks, it is hard to see whether it is on track.", working_days(plan.items[item].duration as i32))], "Split it into parts of six weeks or less.".into(), ["Hard to see whether it is on track.".into(), "Split it into parts of six weeks or less.".into()])
        }
        Check::PhaseEndsWithoutMilestone { phase, item } => {
            let (p, i) = (&plan.phases[phase].name, name(plan, item));
            both(vec![format!("The phase {p} ends with {i}, a piece of work, so nothing marks when the phase is done.")], format!("Add a milestone after {i}, such as \"{p} done\"."), [format!("Nothing marks when {p} is done."), format!("Add a milestone after {i}.")])
        }
    }
}

/// The verdict with a buffer of `BUFFER` working days between `before` and the fixed `target`.
fn with_buffer(plan: &Plan, calendars: &[Calendar], today: Date, before: usize, target: usize) -> Option<Verdict> {
    let (from, to) = (plan.items[before].id.clone(), plan.items[target].id.clone());
    let id = (0..).map(|n| if n == 0 { format!("buffer-before-{to}") } else { format!("buffer-before-{to}-{n}") }).find(|id| plan.item(id).is_none())?;
    let buffer = Item { id: id.clone(), lane: plan.items[before].lane.clone(), name: "Buffer".into(), duration: BUFFER, after: vec![from.clone()], buffer: true, ..Item::default() };
    let plan = apply_all(plan, &[Edit::Add { item: buffer }, Edit::Unlink { from: from.clone(), to: to.clone() }, Edit::Link { from: id, to }]).ok()?;
    let schedule = crate::schedule(&plan, calendars, today).ok()?;
    analyse(&plan, calendars, &schedule, today).verdict
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
        assert_eq!(headline, "Launch on Mon 01 Mar is on track.");
        assert_eq!(detail, "12 working days to spare, with Requirements starting by Mon 16 Nov.");
        assert_eq!(handoff(&plan, &analysis.handoffs[2]), "Frontend (Bangalore) waits on Client (New York): Design sign-off due Fri 18 Dec.");
        assert_eq!(next(&plan, &analysis, &analysis.next[0]), "Start Requirements by Mon 16 Nov: 12 working days from today.");
        assert_eq!(next(&plan, &analysis, &analysis.next[1]), "No buffer before Launch on Mon 01 Mar.");
        let line = |runs: &[Run]| runs.iter().map(|(text, _)| text.as_str()).collect::<String>();
        let buffer = advice(&plan, &calendars, &schedule, today, &analysis.checks[0]);
        assert_eq!(line(&buffer.means[0]), "Launch is fixed on Mon 01 Mar. Acceptance ends on Fri 26 Feb, the working day before.");
        assert_eq!(line(&buffer.can_do[0]), "A buffer of 5 working days before Launch takes small delays. The plan is then still on track, with 7 working days to spare, with Requirements starting by Mon 09 Nov.");
        assert_eq!(line(&buffer.short[1]), "A 5 wd buffer before it leaves 7 wd to spare.");
        assert!(buffer.can_do[0].iter().any(|(text, strong)| text == "Mon 09 Nov" && *strong));
    }
}
