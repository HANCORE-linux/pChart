//! Who waits on whom, checked on random plans: lanes in several countries, work and milestones, fixed
//! dates, earliest and latest dates, buffers and phases. Every rule is stated on its own, not taken from
//! the scheduler, and a finding names its plan by the seed it was made from.

use jiff::civil::Date;
use pchart_core::calendar::{next, previous};
use pchart_core::{Calendar, Check, Edit, Holidays, Plan, Schedule, Span, analyse, apply_all, schedule, why};

struct Rng(u64);

impl Rng {
    fn step(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.step() % n.max(1)
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

const PLACES: [&str; 5] = ["DE-BE", "IN-KA", "CN", "US-NY", ""];

fn day(base: Date, offset: i64) -> Date {
    base.checked_add(jiff::Span::new().days(offset)).unwrap()
}

fn random_plan(rng: &mut Rng) -> (String, Date) {
    let base = day(jiff::civil::date(2026, 10, 1), rng.below(150) as i64);
    let today = day(base, rng.below(40) as i64 - 20);
    let lanes = 1 + rng.below(3) as usize;
    let count = 2 + rng.below(12) as usize;
    let mut out = String::from("format = 1\nname = \"Random\"\n");
    if rng.chance(30) {
        out += &format!("start = {base}\n");
    }
    let mut items = String::new();
    for k in 0..count {
        let milestone = rng.chance(35);
        items += &format!("\n[[items]]\nid = \"i{k}\"\nlane = \"l{}\"\nname = \"Item {k}\"\n", rng.below(lanes as u64));
        if milestone {
            items += "milestone = true\n";
        } else {
            items += &format!("duration = \"{}wd\"\n", 1 + rng.below(8));
            if rng.chance(10) {
                items += "buffer = true\n";
            }
        }
        let after: Vec<String> = (0..k).filter(|_| rng.chance(if k < 4 { 50 } else { 25 })).map(|j| format!("\"i{j}\"")).collect();
        if !after.is_empty() {
            items += &format!("after = [{}]\n", after.join(", "));
        }
        let fix = rng.below(100);
        let when = day(base, rng.below(160) as i64 - 10);
        if milestone && fix < 30 || !milestone && fix < 8 {
            items += &format!("on = {when}\n");
        } else if fix < 45 {
            items += &format!("no_earlier_than = {when}\n");
        } else if fix < 58 {
            items += &format!("no_later_than = {when}\n");
        }
    }
    for lane in 0..lanes {
        out += &format!("\n[[lanes]]\nid = \"l{lane}\"\nname = \"Lane {lane}\"\n");
        let place = PLACES[rng.below(PLACES.len() as u64) as usize];
        if !place.is_empty() {
            out += &format!("place = \"{place}\"\n");
        }
    }
    if rng.chance(50) && count > 2 {
        let ends: Vec<usize> = (0..count).filter(|_| rng.chance(20)).take(3).collect();
        for (number, end) in ends.iter().enumerate() {
            out += &format!("\n[[phases]]\nname = \"Phase {number}\"\nuntil = \"i{end}\"\n");
        }
    }
    (out + &items, today)
}

fn calendars(plan: &Plan) -> Vec<Calendar> {
    let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
    plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect()
}

/// Whether `item` waits on `other`, directly or through others, found on its own.
fn reaches(plan: &Plan, item: &str, other: &str) -> bool {
    let mut stack = vec![item.to_string()];
    let mut seen = std::collections::HashSet::new();
    while let Some(id) = stack.pop() {
        let Some(index) = plan.item(&id) else { continue };
        for pred in &plan.items[index].after {
            if pred == other {
                return true;
            }
            if seen.insert(pred.clone()) {
                stack.push(pred.clone());
            }
        }
    }
    false
}

const PLANS: u64 = 3000;

fn fixed_milestone(plan: &Plan, index: usize) -> bool {
    plan.items[index].milestone && plan.items[index].on.is_some()
}

/// The first day on which what waits on `index` may start, as the plan's rules say.
fn ready(plan: &Plan, index: usize, span: Span) -> Date {
    if fixed_milestone(plan, index) { span.start } else { next(span.finish) }
}

fn preds(plan: &Plan, index: usize) -> Vec<usize> {
    plan.items[index].after.iter().filter_map(|id| plan.item(id)).collect()
}

fn check_schedule(plan: &Plan, today: Date, found: &mut Vec<String>, seed: u64) {
    let cals = calendars(plan);
    let s: Schedule = match schedule(plan, &cals, today) {
        Ok(s) => s,
        Err(err) => return found.push(format!("{seed}: schedule failed on a plan without circles: {err}")),
    };
    let analysis = analyse(plan, &cals, &s, today);
    let lane = |index: usize| plan.lane(&plan.items[index].lane).unwrap();
    let mut say = |rule: &str, text: String| found.push(format!("{seed} [{rule}] {text}"));
    let name = |index: usize| plan.items[index].id.clone();
    for (label, base, spans) in [("earliest", plan.start.unwrap_or(today), s.slots.iter().map(|slot| slot.earliest).collect::<Vec<_>>()), ("planned", s.start, s.slots.iter().map(|slot| slot.planned).collect())] {
        for (index, item) in plan.items.iter().enumerate() {
            let cal = &cals[lane(index)];
            let span = spans[index];
            if item.milestone {
                if span.start != span.finish {
                    say("milestone-one-day", format!("{label} {} {span:?}", name(index)));
                }
            } else {
                if span.finish < span.start {
                    say("work-order", format!("{label} {} {span:?}", name(index)));
                }
                if item.on.is_none() && !cal.is_working(span.start) {
                    say("work-starts-on-working-day", format!("{label} {} starts {}", name(index), span.start));
                }
                if !cal.is_working(span.finish) {
                    say("work-ends-on-working-day", format!("{label} {} ends {}", name(index), span.finish));
                }
                let count = cal.between(span.start, next(span.finish));
                if count != item.duration as i32 {
                    say("work-length", format!("{label} {} takes {count} instead of {}", name(index), item.duration));
                }
            }
            if let Some(on) = item.on
                && span.start != on
            {
                say("fixed-stays", format!("{label} {} fixed {on} drawn {}", name(index), span.start));
            }
            if item.on.is_some() {
                continue;
            }
            let floor = item.no_earlier_than.map_or(base, |day| day.max(base));
            for pred in preds(plan, index) {
                let p = spans[pred];
                let after = if item.milestone { span.finish >= if fixed_milestone(plan, pred) { p.start } else { p.finish } } else { span.start >= ready(plan, pred, p) };
                if !after {
                    say("waits-on", format!("{label} {} {span:?} does not wait for {} {p:?}", name(index), name(pred)));
                }
            }
            let starts = if item.milestone { span.finish } else { span.start };
            if starts < floor && item.no_earlier_than.is_some() {
                say("no-earlier-than", format!("{label} {} on {starts}, not before {floor}", name(index)));
            }
            if item.milestone {
                let reached = preds(plan, index).into_iter().map(|pred| if fixed_milestone(plan, pred) { spans[pred].start } else { spans[pred].finish }).max();
                let floor = cal.on_or_after(floor);
                let expected = cal.on_or_after(reached.map_or(floor, |day| day.max(floor)));
                if span.finish != expected {
                    say("milestone-tight", format!("{label} {} on {} instead of {expected}", name(index), span.finish));
                }
            } else {
                let from = preds(plan, index).into_iter().map(|pred| ready(plan, pred, spans[pred])).max().map_or(floor, |day| day.max(floor));
                if span.start != cal.on_or_after(from) {
                    say("work-tight", format!("{label} {} starts {} instead of {}", name(index), span.start, cal.on_or_after(from)));
                }
            }
        }
    }
    for (index, item) in plan.items.iter().enumerate() {
        let slot = &s.slots[index];
        let cal = &cals[lane(index)];
        if !item.milestone && item.on.is_none() {
            let count = cal.between(slot.latest.start, next(slot.latest.finish));
            if count != item.duration as i32 {
                say("latest-length", format!("{} latest takes {count}", name(index)));
            }
        }
        if item.on.is_some() || plan.items[index].milestone && plan.items[index].on.is_some() {
            continue;
        }
        if let Some(limit) = item.no_later_than
            && slot.latest.finish > limit
        {
            say("no-later-than", format!("{} latest {} after {limit}", name(index), slot.latest.finish));
        }
        let bounded = |index: usize| -> bool {
            let mut stack = vec![index];
            let mut seen = vec![false; plan.items.len()];
            while let Some(at) = stack.pop() {
                let item = &plan.items[at];
                if item.on.is_some() || item.no_later_than.is_some() {
                    return true;
                }
                for succ in (0..plan.items.len()).filter(|&other| preds(plan, other).contains(&at)) {
                    if !std::mem::replace(&mut seen[succ], true) {
                        stack.push(succ);
                    }
                }
            }
            false
        };
        for succ in (0..plan.items.len()).filter(|&other| preds(plan, other).contains(&index) && (bounded(other) || !bounded(index))) {
            let t = s.slots[succ].latest;
            let by = if plan.items[succ].milestone && plan.items[succ].on.is_none() { t.finish } else { previous(t.start) };
            if slot.latest.finish > by {
                say("latest-waits", format!("{} latest {:?} after what {} needs ({by})", name(index), slot.latest, name(succ)));
            }
        }
        let problems = analysis.checks.iter().any(|check| check.severity() == pchart_core::Severity::Problem || matches!(check, Check::DayOff { .. }));
        if !problems && slot.planned.start > slot.latest.start {
            say("planned-after-latest", format!("{} planned {:?} latest {:?}", name(index), slot.planned, slot.latest));
        }
        if slot.planned.start < slot.earliest.start {
            say("planned-before-earliest", format!("{} planned {:?} earliest {:?}", name(index), slot.planned, slot.earliest));
        }
    }
    let late = |index: usize| {
        let item = &plan.items[index];
        match (item.on, item.no_later_than) {
            (Some(on), _) => preds(plan, index).into_iter().any(|pred| ready(plan, pred, s.slots[pred].earliest) > on),
            (None, Some(by)) => s.slots[index].earliest.finish > by,
            _ => false,
        }
    };
    let targets: Vec<usize> = (0..plan.items.len()).filter(|&index| plan.items[index].no_later_than.is_some() || plan.items[index].milestone && plan.items[index].on.is_some() && !preds(plan, index).is_empty()).collect();
    if let (Some(target), Some(spare)) = (s.target, s.spare()) {
        let any = targets.iter().any(|&index| late(index));
        if any != (spare < 0) {
            say("verdict-honest", format!("a target late {any} but {spare} wd to spare"));
        }
        if spare < 0 && !late(target) {
            say("verdict-names-the-late-date", format!("{} is named, its own date holds", name(target)));
        }
    }
    for pair in s.chain.windows(2) {
        if !preds(plan, pair[1]).contains(&pair[0]) || s.slots[pair[1]].driver != Some(pair[0]) {
            say("chain-links", format!("{} → {}", name(pair[0]), name(pair[1])));
        }
    }
    if let Some(&first) = s.chain.first()
        && s.slots[first].driver.is_some()
    {
        say("chain-starts", name(first));
    }
    if let Some(target) = s.target
        && s.chain.last() != Some(&target)
    {
        say("chain-ends-at-target", name(target));
    }
    for index in 0..plan.items.len() {
        let chain = why(&s, index);
        if chain.last() != Some(&index) || chain.windows(2).any(|pair| !preds(plan, pair[1]).contains(&pair[0])) || s.slots[chain[0]].driver.is_some() {
            say("why", format!("{} {chain:?}", name(index)));
        }
        if let Some(driver) = s.slots[index].driver
            && !preds(plan, index).contains(&driver)
        {
            say("driver-is-a-predecessor", name(index));
        }
    }
    for (index, item) in plan.items.iter().enumerate() {
        if let Some(on) = item.on
            && Some(index) != s.target
        {
            let late = preds(plan, index).into_iter().any(|pred| ready(plan, pred, s.slots[pred].earliest) > on);
            let flagged = analysis.checks.iter().any(|check| matches!(check, Check::FixedTooEarly { item, .. } if *item == index));
            if late != flagged {
                say("fixed-too-early-check", format!("{} late {late} flagged {flagged}", name(index)));
            }
        }
    }
    let position: Vec<usize> = {
        let mut at = vec![0; plan.items.len()];
        for (step, &index) in s.order.iter().enumerate() {
            at[index] = step;
        }
        at
    };
    if s.order.len() != plan.items.len() || (0..plan.items.len()).any(|index| preds(plan, index).iter().any(|&pred| position[pred] >= position[index])) {
        say("order", format!("{:?}", s.order));
    }
}

fn check_edits(plan: &Plan, rng: &mut Rng, found: &mut Vec<String>, seed: u64) {
    let ids: Vec<String> = plan.items.iter().map(|item| item.id.clone()).collect();
    let pick = |rng: &mut Rng| ids[rng.below(ids.len() as u64) as usize].clone();
    for _ in 0..6 {
        let (from, to) = (pick(rng), pick(rng));
        let circle = from == to || reaches(plan, &from, &to);
        match (Edit::Link { from: from.clone(), to: to.clone() }.apply(plan), circle) {
            (Ok(next), false) => {
                let after = &next.items[next.item(&to).unwrap()].after;
                if after.iter().filter(|id| **id == from).count() != 1 {
                    found.push(format!("{seed} [link-once] {from} → {to}: {after:?}"));
                }
                if schedule(&next, &calendars(&next), jiff::civil::date(2026, 11, 2)).is_err() {
                    found.push(format!("{seed} [link-schedules] {from} → {to}"));
                }
            }
            (Err(_), true) => {}
            (result, circle) => found.push(format!("{seed} [link-circle] {from} → {to}: circle {circle}, result ok {}", result.is_ok())),
        }
        let linked = plan.items[plan.item(&to).unwrap()].after.contains(&from);
        match (Edit::Unlink { from: from.clone(), to: to.clone() }.apply(plan), linked) {
            (Ok(next), true) => {
                if next.items[next.item(&to).unwrap()].after.contains(&from) {
                    found.push(format!("{seed} [unlink-gone] {from} → {to}"));
                }
            }
            (Err(_), false) => {}
            (result, linked) => found.push(format!("{seed} [unlink] {from} → {to}: linked {linked}, ok {}", result.is_ok())),
        }
        let gone = pick(rng);
        let blocked = plan.items.iter().any(|item| item.after.contains(&gone)) || plan.phases.iter().any(|phase| phase.until == gone);
        match (Edit::Remove { item: gone.clone() }.apply(plan), blocked) {
            (Ok(next), false) => {
                if next.items.iter().any(|item| item.after.contains(&gone)) || next.item(&gone).is_some() {
                    found.push(format!("{seed} [remove-clean] {gone}"));
                }
            }
            (Err(_), true) => {}
            (result, blocked) => found.push(format!("{seed} [remove] {gone}: blocked {blocked}, ok {}", result.is_ok())),
        }
    }
    let first = pick(rng);
    let batch = [Edit::Link { from: first.clone(), to: first.clone() }];
    if apply_all(plan, &batch).is_ok() {
        found.push(format!("{seed} [self-link] {first}"));
    }
    match Plan::parse(&plan.to_toml()) {
        Ok(again) if again == *plan => {}
        Ok(_) => found.push(format!("{seed} [roundtrip] differs")),
        Err(err) => found.push(format!("{seed} [roundtrip] {err}")),
    }
}

#[test]
fn every_rule_about_waiting_holds_in_random_plans() {
    let mut found = Vec::new();
    let mut circles = 0;
    let mut checked = 0;
    for seed in 1..=PLANS {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let (text, today) = random_plan(&mut rng);
        let plan = match Plan::parse(&text) {
            Ok(plan) => plan,
            Err(err) => {
                found.push(format!("{seed} [parse] {err}\n{text}"));
                continue;
            }
        };
        checked += 1;
        check_schedule(&plan, today, &mut found, seed);
        check_edits(&plan, &mut rng, &mut found, seed);
        let mut circle = plan.clone();
        if circle.items.len() >= 2 {
            let (a, b) = (circle.items[0].id.clone(), circle.items.last().unwrap().id.clone());
            circle.items[0].after.push(b);
            if reaches(&circle, &a, &a) {
                circles += 1;
                if schedule(&circle, &calendars(&circle), today).is_ok() {
                    found.push(format!("{seed} [circle-found] a circle was scheduled"));
                }
            }
        }
    }
    let mut rules: std::collections::BTreeMap<String, (usize, String)> = std::collections::BTreeMap::new();
    for line in &found {
        let rule = line.split('[').nth(1).and_then(|rest| rest.split(']').next()).unwrap_or("?").to_string();
        let entry = rules.entry(rule).or_insert((0, line.clone()));
        entry.0 += 1;
    }
    let summary: Vec<String> = rules.iter().map(|(rule, (count, first))| format!("{rule}: {count}×, e.g. {first}")).collect();
    assert!(checked == PLANS as usize && circles > PLANS as usize / 2, "{checked} plans, {circles} with a circle");
    assert!(found.is_empty(), "{} findings:\n{}", found.len(), summary.join("\n"));
}
