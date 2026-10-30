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
    /// As late as possible without moving the target.
    pub latest: Span,
    /// Working days between earliest and latest, counted in the item's lane calendar.
    pub slack: i32,
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
}

impl Schedule {
    /// Working days to spare before the target; negative when the target cannot be held.
    pub fn spare(&self) -> Option<i32> {
        self.target?;
        self.chain.first().map(|&first| self.slots[first].slack)
    }

    pub fn phases(&self, plan: &Plan) -> Vec<(String, Date, Date)> {
        let mut from = self.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(self.start);
        let mut out = Vec::new();
        for phase in &plan.phases {
            let Some(until) = plan.item(&phase.until) else { continue };
            let to = self.slots[until].planned.finish;
            out.push((phase.name.clone(), from, to));
            from = next(to);
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
    let end = earliest.iter().map(|(span, _)| span.finish).max().unwrap_or(base);
    let latest = backward(plan, &succs, &order, &cal, end);
    let slack: Vec<i32> = (0..plan.items.len())
        .map(|index| {
            let (early, late) = (earliest[index].0, latest[index]);
            if plan.items[index].milestone { cal(index).between(early.finish, late.finish) } else { cal(index).between(early.start, late.start) }
        })
        .collect();

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
    let target = targets.min_by_key(|&index| slack[trace(index)[0]]);
    let chain = match target {
        Some(target) => trace(target),
        None => (0..plan.items.len()).max_by_key(|&index| earliest[index].0.finish).map(trace).unwrap_or_default(),
    };

    let start = match (plan.start, target, chain.first()) {
        (None, Some(_), Some(&first)) if slack[first] > 0 => latest[first].start,
        _ => base,
    };
    let planned = if start == base { earliest.clone() } else { forward(plan, &preds, &order, &cal, start) };

    let slots = (0..plan.items.len())
        .map(|index| Slot { planned: planned[index].0, earliest: earliest[index].0, latest: latest[index], slack: slack[index], driver: earliest[index].1 })
        .collect();
    Ok(Schedule { slots, order, chain, target, start })
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
        let driver = preds[index].iter().map(|&pred| (available(plan, pred, out[pred].0), pred)).max_by_key(|(day, _)| *day).filter(|(day, _)| *day >= from).map(|(day, pred)| {
            from = day;
            pred
        });
        let span = match (item.milestone, item.on) {
            (true, Some(day)) => Span { start: day, finish: day },
            (true, None) => {
                let day = if driver.is_some() { previous(from) } else { from };
                Span { start: day, finish: day }
            }
            (false, fixed) => {
                let start = fixed.unwrap_or_else(|| cal(index).on_or_after(from));
                Span { start, finish: cal(index).finish(start, item.duration) }
            }
        };
        out[index] = (span, driver);
    }
    out
}

fn backward<'a>(plan: &Plan, succs: &[Vec<usize>], order: &[usize], cal: &impl Fn(usize) -> &'a Calendar, end: Date) -> Vec<Span> {
    let mut out = vec![Span { start: end, finish: end }; plan.items.len()];
    for &index in order.iter().rev() {
        let item = &plan.items[index];
        let mut by = succs[index].iter().map(|&succ| required(plan, succ, out[succ])).min().unwrap_or(end);
        if let Some(limit) = item.no_later_than {
            by = by.min(limit);
        }
        out[index] = match (item.milestone, item.on) {
            (_, Some(day)) if item.milestone => Span { start: day, finish: day },
            (true, _) => {
                let day = cal(index).on_or_before(by);
                Span { start: day, finish: day }
            }
            (false, Some(day)) => Span { start: day, finish: cal(index).finish(day, item.duration) },
            (false, None) => {
                let finish = cal(index).on_or_before(by);
                Span { start: cal(index).start(finish, item.duration), finish }
            }
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
