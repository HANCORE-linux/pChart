use jiff::civil::Date;

use crate::Error;
use crate::calendar::{Calendar, next, previous};
use crate::model::Plan;

/// First and last day of an item; both are the same day for a milestone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Date,
    pub finish: Date,
}

#[derive(Debug, Clone)]
pub struct Slot {
    /// Where the item is drawn: as early as possible from the plan start.
    pub planned: Span,
    /// As early as possible from today, or from the fixed plan start.
    pub earliest: Span,
    /// As late as possible without moving a fixed date; for an item no fixed date waits on,
    /// without moving the planned last day.
    pub latest: Span,
    /// Working days between earliest and latest, counted in the item's lane calendar.
    pub slack: i32,
    /// What the item has to spare: on the deciding chain, what the chain has; elsewhere, working
    /// days from where it is planned to its latest, counted in the item's lane calendar.
    pub spare: i32,
    /// The predecessor that sets the earliest start.
    pub driver: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Schedule {
    pub slots: Vec<Slot>,
    pub order: Vec<usize>,
    /// The chain that decides the date, first item first.
    pub chain: Vec<usize>,
    /// The fixed milestone or deadline with the least to spare. A task fixed on a date is a constraint, not a target.
    pub target: Option<usize>,
    /// The day the planned dates start from.
    pub start: Date,
    /// The item of the chain on which what the chain has to spare is measured: its first, or the first
    /// after the last fixed date on it, since a fixed date has nothing to spare.
    pub first: Option<usize>,
    /// The latest day `first` can start and still hold the target.
    pub start_by: Option<Date>,
    room: Option<i32>,
}

impl Schedule {
    /// Working days to spare before the target; negative when the target cannot be held.
    pub fn spare(&self) -> Option<i32> {
        self.target?;
        self.room
    }

    /// Each phase from its first item to the item that ends it.
    pub fn phases(&self, plan: &Plan) -> Vec<(String, Date, Date)> {
        let first_after = |day: Option<Date>| self.slots.iter().map(|slot| slot.planned.start).filter(|start| day.is_none_or(|day| *start > day)).min();
        let mut from = first_after(None).unwrap_or(self.start);
        let mut out = Vec::new();
        for phase in &plan.phases {
            let Some(until) = plan.item(&phase.until) else { continue };
            let to = self.slots[until].planned.finish;
            out.push((phase.name.clone(), from, to));
            from = first_after(Some(to)).unwrap_or(next(to));
        }
        out
    }
}

/// Schedules every item that has no fixed date. `calendars` holds one calendar per lane.
pub fn schedule(plan: &Plan, calendars: &[Calendar], today: Date) -> Result<Schedule, Error> {
    let lanes: Vec<usize> = plan.items.iter().map(|item| plan.lane(&item.lane).expect("checked when the plan was read")).collect();
    let preds: Vec<Vec<usize>> = plan.items.iter().map(|item| item.after.iter().filter_map(|id| plan.item(id)).collect()).collect();
    let mut succs = vec![Vec::new(); plan.items.len()];
    for (index, list) in preds.iter().enumerate() {
        for &pred in list {
            succs[pred].push(index);
        }
    }
    let order = order(plan, &preds)?;
    let cal = |index: usize| &calendars[lanes[index]];

    let base = plan.start.unwrap_or(today);
    let earliest = forward(plan, &preds, &order, &cal, base);
    let bound = backward(plan, &succs, &order, &cal, None);
    let gap = |index: usize, from: Span, to: Span| {
        if plan.items[index].milestone { cal(index).between(from.finish, to.finish) } else { cal(index).between(from.start, to.start) }
    };
    let bound_slack = |index: usize| bound[index].map(|late| gap(index, earliest[index].0, late));

    let trace = |mut index: usize| {
        let mut chain = vec![index];
        while let Some(driver) = earliest[index].1 {
            chain.push(driver);
            index = driver;
        }
        chain.reverse();
        chain
    };
    let targets = (0..plan.items.len()).filter(|&index| {
        let item = &plan.items[index];
        item.no_later_than.is_some() || (item.milestone && item.on.is_some() && !preds[index].is_empty())
    });
    let after_fixed = |chain: &[usize]| chain.split_last().map_or(0, |(_, before)| before.iter().rposition(|&index| plan.items[index].on.is_some()).map_or(0, |at| at + 1));
    let fixed_room = |index: usize| {
        plan.items[index].on.map(|day| {
            let ready = preds[index].iter().map(|&pred| available(plan, pred, earliest[pred].0)).max().unwrap_or(day);
            if ready > day { cal(index).between(ready, day).min(-1) } else { cal(index).between(ready, day) }
        })
    };
    let holds = |target: usize| {
        let chain = trace(target);
        let first = chain[after_fixed(&chain)];
        let late = toward(plan, &succs, &order, &cal, target)[first].expect("the chain leads to the target");
        (fixed_room(first).unwrap_or_else(|| gap(first, earliest[first].0, late)), late.start)
    };
    let target = targets.min_by_key(|&index| holds(index).0);
    let chain = match target {
        Some(target) => trace(target),
        None => (0..plan.items.len()).max_by_key(|&index| earliest[index].0.finish).map(trace).unwrap_or_default(),
    };

    let shifted = match (plan.start, target, chain.first()) {
        (None, Some(_), Some(&first)) if bound_slack(first).is_some_and(|slack| slack > 0) => bound[first].expect("the chain leads to the target").start,
        _ => base,
    };
    let fits = |planned: &[(Span, Option<usize>)]| (0..plan.items.len()).all(|index| bound[index].is_none_or(|late| planned[index].0.start <= late.start.max(earliest[index].0.start)));
    let (mut early, mut late) = (0, base.until(shifted).map_or(0, |span| span.get_days()));
    let mut planned = if shifted == base { earliest.clone() } else { forward(plan, &preds, &order, &cal, shifted) };
    let mut start = shifted;
    if !fits(&planned) {
        while late - early > 1 {
            let middle = (early + late) / 2;
            let day = base.checked_add(jiff::Span::new().days(middle)).unwrap_or(base);
            if fits(&forward(plan, &preds, &order, &cal, day)) { early = middle } else { late = middle }
        }
        start = base.checked_add(jiff::Span::new().days(early)).unwrap_or(base);
        planned = forward(plan, &preds, &order, &cal, start);
    }
    let end = planned.iter().map(|(span, _)| span.finish).max().unwrap_or(start);
    let soft = backward(plan, &succs, &order, &cal, Some(end));
    let latest: Vec<Span> = bound.iter().zip(&soft).map(|(bound, soft)| bound.or(*soft).expect("the planned last day limits every item")).collect();
    let slack: Vec<i32> = (0..plan.items.len()).map(|index| gap(index, earliest[index].0, latest[index])).collect();
    let from = after_fixed(&chain);
    let first = chain.get(from).copied();
    let (room, start_by) = match (target, first) {
        (Some(target), _) => {
            let (room, by) = holds(target);
            (Some(room), Some(by))
        }
        (None, Some(first)) => (Some(slack[first]), Some(latest[first].start)),
        _ => (None, None),
    };

    let slots = (0..plan.items.len())
        .map(|index| Slot {
            planned: planned[index].0,
            earliest: earliest[index].0,
            latest: latest[index],
            slack: slack[index],
            spare: match (chain.iter().position(|&step| step == index), room) {
                (Some(at), Some(room)) if at >= from => room,
                _ => gap(index, planned[index].0, latest[index]),
            },
            driver: earliest[index].1,
        })
        .collect();
    Ok(Schedule { slots, order, chain, target, start, first, start_by, room })
}

fn order(plan: &Plan, preds: &[Vec<usize>]) -> Result<Vec<usize>, Error> {
    fn visit(index: usize, preds: &[Vec<usize>], state: &mut [u8], path: &mut Vec<usize>, out: &mut Vec<usize>) -> Result<(), Vec<usize>> {
        match state[index] {
            2 => return Ok(()),
            1 => {
                let from = path.iter().position(|&step| step == index).unwrap_or(0);
                return Err(path[from..].to_vec());
            }
            _ => {}
        }
        state[index] = 1;
        path.push(index);
        for &pred in &preds[index] {
            visit(pred, preds, state, path, out)?;
        }
        path.pop();
        state[index] = 2;
        out.push(index);
        Ok(())
    }
    let mut state = vec![0u8; plan.items.len()];
    let mut out = Vec::with_capacity(plan.items.len());
    for index in 0..plan.items.len() {
        let mut path = Vec::new();
        visit(index, preds, &mut state, &mut path, &mut out).map_err(|cycle| {
            let names: Vec<&str> = cycle.iter().map(|&step| plan.items[step].name.as_str()).collect();
            Error::Plan(format!("these items wait on each other in a circle: {}", names.join(" → ")))
        })?;
    }
    Ok(out)
}

/// The first day on which successors of an item may start.
pub(crate) fn available(plan: &Plan, index: usize, span: Span) -> Date {
    let item = &plan.items[index];
    if item.milestone && item.on.is_some() { span.start } else { next(span.finish) }
}

/// The day by which an item is reached: a fixed milestone on its day, everything else at the end of its
/// last day.
fn reached(plan: &Plan, index: usize, span: Span) -> Date {
    let item = &plan.items[index];
    if item.milestone && item.on.is_some() { span.start } else { span.finish }
}

/// The last day by which predecessors of an item must have finished.
fn required(plan: &Plan, index: usize, span: Span) -> Date {
    let item = &plan.items[index];
    if item.milestone && item.on.is_none() { span.finish } else { previous(span.start) }
}

fn forward<'a>(plan: &Plan, preds: &[Vec<usize>], order: &[usize], cal: &impl Fn(usize) -> &'a Calendar, base: Date) -> Vec<(Span, Option<usize>)> {
    let mut out = vec![(Span { start: base, finish: base }, None); plan.items.len()];
    for &index in order {
        let item = &plan.items[index];
        let mut from = base.max(item.no_earlier_than.unwrap_or(base));
        if item.milestone && item.on.is_none() {
            from = cal(index).on_or_after(from);
            let reached = preds[index].iter().map(|&pred| (reached(plan, pred, out[pred].0), pred)).max_by_key(|(day, _)| *day);
            let (day, driver) = match reached {
                Some((day, pred)) if day >= from => (cal(index).on_or_after(day), Some(pred)),
                _ => (from, None),
            };
            out[index] = (Span { start: day, finish: day }, driver);
            continue;
        }
        let driver = preds[index].iter().map(|&pred| (available(plan, pred, out[pred].0), pred)).max_by_key(|(day, _)| *day).filter(|(day, _)| *day >= from).map(|(day, pred)| {
            from = day;
            pred
        });
        let span = match item.on {
            Some(day) if item.milestone => Span { start: day, finish: day },
            fixed => {
                let start = fixed.unwrap_or_else(|| cal(index).on_or_after(from));
                Span { start, finish: cal(index).finish(start, item.duration) }
            }
        };
        out[index] = (span, driver);
    }
    out
}

/// Latest spans that hold `target` alone: from its own date back through what waits before it. A fixed
/// date on the way does not move, so the target binds nothing before it through that date.
fn toward<'a>(plan: &Plan, succs: &[Vec<usize>], order: &[usize], cal: &impl Fn(usize) -> &'a Calendar, target: usize) -> Vec<Option<Span>> {
    let by = |index: usize, day: Date| {
        let item = &plan.items[index];
        let finish = cal(index).on_or_before(day);
        Span { start: if item.milestone { finish } else { cal(index).start(finish, item.duration) }, finish }
    };
    let mut out: Vec<Option<Span>> = vec![None; plan.items.len()];
    let item = &plan.items[target];
    out[target] = Some(match item.on {
        Some(day) => Span { start: day, finish: if item.milestone { day } else { cal(target).finish(day, item.duration) } },
        None => by(target, item.no_later_than.expect("a target is fixed or has a latest date")),
    });
    for &index in order.iter().rev() {
        if index != target && plan.items[index].on.is_none() {
            out[index] = succs[index].iter().filter_map(|&succ| out[succ].map(|span| required(plan, succ, span))).min().map(|day| by(index, day));
        }
    }
    out
}

/// Latest spans. Without `end`, only fixed dates limit them, and an item no fixed date waits on has none.
fn backward<'a>(plan: &Plan, succs: &[Vec<usize>], order: &[usize], cal: &impl Fn(usize) -> &'a Calendar, end: Option<Date>) -> Vec<Option<Span>> {
    let mut out: Vec<Option<Span>> = vec![None; plan.items.len()];
    for &index in order.iter().rev() {
        let item = &plan.items[index];
        let by = succs[index].iter().filter_map(|&succ| out[succ].map(|span| required(plan, succ, span))).min().or(end);
        let by = match (by, item.no_later_than) {
            (Some(by), Some(limit)) => Some(by.min(limit)),
            (by, limit) => by.or(limit),
        };
        out[index] = match (item.milestone, item.on) {
            (_, Some(day)) if item.milestone => Some(Span { start: day, finish: day }),
            (true, _) => by.map(|by| {
                let day = cal(index).on_or_before(by);
                Span { start: day, finish: day }
            }),
            (false, Some(day)) => Some(Span { start: day, finish: cal(index).finish(day, item.duration) }),
            (false, None) => by.map(|by| {
                let finish = cal(index).on_or_before(by);
                Span { start: cal(index).start(finish, item.duration), finish }
            }),
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn plan(items: &str) -> Plan {
        Plan::parse(&format!("format = 1\nname = \"t\"\n[[lanes]]\nid = \"a\"\nname = \"A\"\n{items}")).unwrap()
    }

    fn run(plan: &Plan, today: Date) -> Schedule {
        schedule(plan, &[Calendar::default()], today).unwrap()
    }

    const CHAIN: &str = r#"
[[items]]
id = "build"
lane = "a"
name = "Build"
duration = "5wd"
[[items]]
id = "done"
lane = "a"
name = "Done"
milestone = true
after = ["build"]
[[items]]
id = "test"
lane = "a"
name = "Test"
duration = "2wd"
after = ["done"]
[[items]]
id = "docs"
lane = "a"
name = "Docs"
duration = "1wd"
after = ["build"]
[[items]]
id = "launch"
lane = "a"
name = "Launch"
milestone = true
after = ["test", "docs"]
on = 2026-11-16
"#;

    #[test]
    fn a_calculated_milestone_takes_no_day() {
        let s = run(&plan(CHAIN), date(2026, 11, 2));
        let p = plan(CHAIN);
        let done = &s.slots[p.item("done").unwrap()];
        let test = &s.slots[p.item("test").unwrap()];
        assert_eq!(done.earliest.finish, date(2026, 11, 6));
        assert_eq!(test.earliest.start, date(2026, 11, 9));
    }

    #[test]
    fn a_milestone_after_a_fixed_milestone_is_on_its_day() {
        let p = plan(&format!("{CHAIN}[[items]]\nid = \"review\"\nlane = \"a\"\nname = \"Review\"\nmilestone = true\nafter = [\"launch\"]\n"));
        let s = run(&p, date(2026, 11, 2));
        let review = &s.slots[p.item("review").unwrap()];
        assert_eq!((review.planned.finish, review.earliest.finish, review.latest.finish), (date(2026, 11, 16), date(2026, 11, 16), date(2026, 11, 16)));
    }

    fn work(id: &str, days: u32, after: &[&str], extra: &str) -> String {
        let after: Vec<String> = after.iter().map(|id| format!("\"{id}\"")).collect();
        format!("[[items]]\nid = \"{id}\"\nlane = \"a\"\nname = \"{id}\"\nduration = \"{days}wd\"\nafter = [{}]\n{extra}", after.join(", "))
    }

    fn milestone(id: &str, after: &[&str], extra: &str) -> String {
        let after: Vec<String> = after.iter().map(|id| format!("\"{id}\"")).collect();
        format!("[[items]]\nid = \"{id}\"\nlane = \"a\"\nname = \"{id}\"\nmilestone = true\nafter = [{}]\n{extra}", after.join(", "))
    }

    #[test]
    fn a_milestone_is_on_the_last_day_it_waits_for_even_when_a_fixed_date_ties_with_work() {
        let p = plan(&[work("build", 1, &[], ""), milestone("fixed", &[], "on = 2026-11-03\n"), milestone("done", &["fixed", "build"], "")].concat());
        let s = run(&p, date(2026, 11, 2));
        assert_eq!(s.slots[p.item("done").unwrap()].planned.finish, date(2026, 11, 3));
    }

    #[test]
    fn a_milestone_is_not_before_its_earliest_date_nor_on_a_weekend() {
        let p = plan(&[work("build", 1, &[], ""), milestone("done", &["build"], "no_earlier_than = 2026-11-03\n"), milestone("kickoff", &[], "")].concat());
        let s = run(&p, date(2026, 11, 2));
        assert_eq!(s.slots[p.item("done").unwrap()].earliest.finish, date(2026, 11, 3));
        let s = run(&p, date(2026, 11, 7));
        assert_eq!(s.slots[p.item("kickoff").unwrap()].earliest.finish, date(2026, 11, 9));
    }

    #[test]
    fn a_milestone_is_on_a_working_day_of_its_own_lane() {
        let text = format!("format = 1\nname = \"t\"\n[[lanes]]\nid = \"a\"\nname = \"A\"\n[[lanes]]\nid = \"b\"\nname = \"B\"\n{}{}", work("build", 5, &[], ""), milestone("done", &["build"], "").replace("lane = \"a\"", "lane = \"b\""));
        let p = Plan::parse(&text).unwrap();
        let off = Calendar::new([(date(2026, 11, 6), "Day off".to_string())].into());
        let s = schedule(&p, &[Calendar::default(), off], date(2026, 11, 2)).unwrap();
        assert_eq!(s.slots[p.item("build").unwrap()].planned.finish, date(2026, 11, 6));
        assert_eq!(s.slots[p.item("done").unwrap()].planned.finish, date(2026, 11, 9));
    }

    #[test]
    fn a_fixed_start_of_the_chain_has_nothing_to_spare_so_the_work_after_it_is_measured() {
        let p = plan(&[milestone("kickoff", &[], "on = 2026-11-16\n"), work("build", 10, &["kickoff"], ""), milestone("launch", &["build"], "on = 2026-11-20\n")].concat());
        let s = run(&p, date(2026, 11, 2));
        assert_eq!((s.spare(), s.first.map(|first| p.items[first].id.as_str()), s.start_by), (Some(-6), Some("build"), Some(date(2026, 11, 6))));
        let p = plan(&[milestone("early", &[], "on = 2026-11-09\n"), milestone("launch", &["early"], "on = 2026-11-07\n")].concat());
        assert_eq!(run(&p, date(2026, 11, 2)).spare(), Some(-1));
    }

    #[test]
    fn the_plan_starts_no_later_than_every_item_allows() {
        let p = plan(&[work("prepare", 5, &[], ""), work("build", 5, &["prepare"], "no_earlier_than = 2026-11-30\n"), work("ship", 5, &["build"], "no_later_than = 2026-12-18\n")].concat());
        let s = run(&p, date(2026, 11, 2));
        assert_eq!(s.spare(), Some(5));
        assert!(s.slots[p.item("ship").unwrap()].planned.finish <= date(2026, 12, 18));
    }

    #[test]
    fn the_target_has_what_its_own_date_leaves_and_not_what_another_fixed_date_takes() {
        let p = plan(&[work("build", 5, &[], ""), work("fixed", 2, &["build"], "on = 2026-11-04\n"), milestone("launch", &["build"], "no_later_than = 2026-12-31\n")].concat());
        let s = run(&p, date(2026, 11, 2));
        assert!(s.spare().is_some_and(|spare| spare > 0), "{:?}", s.spare());
    }

    #[test]
    fn predecessors_of_a_fixed_milestone_finish_the_working_day_before() {
        let p = plan(CHAIN);
        let s = run(&p, date(2026, 11, 2));
        assert_eq!(s.slots[p.item("test").unwrap()].latest.finish, date(2026, 11, 13));
        assert_eq!(s.slots[p.item("build").unwrap()].latest.start, date(2026, 11, 5));
        assert_eq!(s.spare(), Some(3));
    }

    #[test]
    fn the_deciding_chain_follows_the_driving_predecessor() {
        let p = plan(CHAIN);
        let s = run(&p, date(2026, 11, 2));
        let names: Vec<&str> = s.chain.iter().map(|&index| p.items[index].id.as_str()).collect();
        assert_eq!(names, ["build", "done", "test", "launch"]);
    }

    #[test]
    fn items_with_slack_start_as_early_as_possible_from_the_latest_start_of_the_chain() {
        let p = plan(CHAIN);
        let s = run(&p, date(2026, 11, 2));
        assert_eq!(s.start, date(2026, 11, 5));
        let docs = &s.slots[p.item("docs").unwrap()];
        assert_eq!(docs.planned.start, date(2026, 11, 12));
        assert_eq!(docs.latest.start, date(2026, 11, 13));
    }

    #[test]
    fn off_the_chain_the_spare_counts_from_where_the_item_is_planned() {
        let p = plan(CHAIN);
        let s = run(&p, date(2026, 11, 2));
        let spare = |id: &str| s.slots[p.item(id).unwrap()].spare;
        assert_eq!((spare("build"), spare("test"), spare("launch")), (3, 3, 3));
        assert_eq!(spare("docs"), 1);
    }

    #[test]
    fn an_item_no_fixed_date_waits_on_does_not_change_the_spare() {
        let p = plan(&format!("{CHAIN}[[items]]\nid = \"manual\"\nlane = \"a\"\nname = \"Manual\"\nduration = \"30wd\"\nafter = [\"build\"]\n"));
        let s = run(&p, date(2026, 11, 2));
        assert_eq!(s.spare(), Some(3));
        assert_eq!(s.start, date(2026, 11, 5));
        let manual = &s.slots[p.item("manual").unwrap()];
        assert_eq!((manual.planned.start, manual.spare), (date(2026, 11, 12), 0));
    }

    #[test]
    fn a_target_that_cannot_be_held_has_negative_spare() {
        let p = plan(CHAIN);
        let s = run(&p, date(2026, 11, 9));
        assert_eq!(s.spare(), Some(-2));
        assert_eq!(s.start, date(2026, 11, 9));
    }

    #[test]
    fn circles_are_reported() {
        let p = plan("[[items]]\nid = \"x\"\nlane = \"a\"\nname = \"X\"\nduration = \"1wd\"\nafter = [\"y\"]\n[[items]]\nid = \"y\"\nlane = \"a\"\nname = \"Y\"\nduration = \"1wd\"\nafter = [\"x\"]\n");
        let err = schedule(&p, &[Calendar::default()], date(2026, 11, 2)).unwrap_err();
        assert_eq!(err.to_string(), "these items wait on each other in a circle: X → Y");
    }
}
