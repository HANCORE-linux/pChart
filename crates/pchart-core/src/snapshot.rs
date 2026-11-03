//! Snapshots (pictures 67): how a plan stood on a day, kept at the end of the plan file, holding only
//! dates: what the chain had to spare, when its first step had to start, and the planned dates of every
//! item. What changed since is worked out against the plan as it stands now.

use std::collections::BTreeMap;
use std::fmt::Write;

use jiff::civil::Date;
use serde::{Deserialize, Deserializer};

use crate::calendar::Calendar;
use crate::model::Plan;
use crate::schedule::Schedule;
use crate::text::{day, working_days};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Snapshot {
    #[serde(deserialize_with = "one")]
    pub day: Date,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub spare: Option<i32>,
    #[serde(default, deserialize_with = "maybe")]
    pub start_by: Option<Date>,
    /// The planned first and last day of each item, by its id.
    #[serde(default, deserialize_with = "spans")]
    pub items: BTreeMap<String, (Date, Date)>,
}

fn parse(value: &toml::value::Datetime) -> Result<Date, String> {
    value.to_string().parse::<Date>().map_err(|_| format!("{value} is not a date like 2027-03-01"))
}

fn one<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Date, D::Error> {
    parse(&toml::value::Datetime::deserialize(deserializer)?).map_err(serde::de::Error::custom)
}

fn maybe<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Date>, D::Error> {
    Option::<toml::value::Datetime>::deserialize(deserializer)?.map(|value| parse(&value)).transpose().map_err(serde::de::Error::custom)
}

fn spans<'de, D: Deserializer<'de>>(deserializer: D) -> Result<BTreeMap<String, (Date, Date)>, D::Error> {
    let raw = BTreeMap::<String, [toml::value::Datetime; 2]>::deserialize(deserializer)?;
    raw.into_iter().map(|(id, [from, to])| Ok((id, (parse(&from)?, parse(&to)?)))).collect::<Result<_, String>>().map_err(serde::de::Error::custom)
}

impl Snapshot {
    /// How `plan` stands on `today`.
    pub fn take(plan: &Plan, schedule: &Schedule, today: Date, name: Option<String>) -> Snapshot {
        let items = plan.items.iter().zip(&schedule.slots).map(|(item, slot)| (item.id.clone(), (slot.planned.start, slot.planned.finish))).collect();
        Snapshot { day: today, name, spare: schedule.spare(), start_by: schedule.start_by, items }
    }

    pub fn to_toml(&self) -> String {
        let mut out = format!("\n[[snapshots]]\nday = {}\n", self.day);
        if let Some(name) = &self.name {
            let _ = writeln!(out, "name = {}", toml::Value::String(name.clone()));
        }
        if let Some(spare) = self.spare {
            let _ = writeln!(out, "spare = {spare}");
        }
        if let Some(start_by) = self.start_by {
            let _ = writeln!(out, "start_by = {start_by}");
        }
        out.push_str("\n[snapshots.items]\n");
        for (id, (from, to)) in &self.items {
            let key = if id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') { id.clone() } else { toml::Value::String(id.clone()).to_string() };
            let _ = writeln!(out, "{key} = [{from}, {to}]");
        }
        out
    }

    /// "Thu 22 Oct", or "Steering committee · Thu 22 Oct".
    pub fn title(&self) -> String {
        match &self.name {
            Some(name) => format!("{name} · {}", day(self.day)),
            None => day(self.day),
        }
    }
}

/// What changed since a snapshot, the most telling first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// What the chain has to spare, by what changed in the plan, not by the days that passed: the first
    /// step of the chain must start `wd` working days later (more to spare) or earlier (less).
    Spare { item: usize, before: Date, now: Date, wd: i32 },
    /// A piece of work takes longer or shorter, in working days of its lane.
    Length { item: usize, before: i32, now: i32 },
    Added { item: usize },
    Removed { name: String },
    /// A milestone moved.
    Moved { item: usize, before: Date, now: Date },
}

/// The snapshot to compare with on `today`: the last one taken up to then.
pub fn last(plan: &Plan, today: Date) -> Option<&Snapshot> {
    plan.snapshots.iter().filter(|snapshot| snapshot.day <= today).max_by_key(|snapshot| snapshot.day)
}

pub fn changes(plan: &Plan, schedule: &Schedule, calendars: &[Calendar], snapshot: &Snapshot) -> Vec<Change> {
    let calendar = |index: usize| plan.lane(&plan.items[index].lane).and_then(|lane| calendars.get(lane));
    let length = |index: usize, (from, to): (Date, Date)| calendar(index).map_or(0, |calendar| calendar.between(from, crate::calendar::next(to)));
    let mut out = Vec::new();
    if let (Some(before), Some(now), Some(item)) = (snapshot.start_by, schedule.start_by, schedule.first)
        && before != now
    {
        let wd = calendar(item).map_or(0, |calendar| calendar.between(before, now));
        out.push(Change::Spare { item, before, now, wd });
    }
    let mut moved = Vec::new();
    for (index, item) in plan.items.iter().enumerate() {
        let now = (schedule.slots[index].planned.start, schedule.slots[index].planned.finish);
        match snapshot.items.get(&item.id) {
            None => out.push(Change::Added { item: index }),
            Some(&before) if item.milestone && before.1 != now.1 => moved.push(Change::Moved { item: index, before: before.1, now: now.1 }),
            Some(&before) if !item.milestone => {
                let (was, is) = (length(index, before), length(index, now));
                if was != is {
                    out.push(Change::Length { item: index, before: was, now: is });
                }
            }
            _ => {}
        }
    }
    for id in snapshot.items.keys().filter(|id| plan.item(id).is_none()) {
        out.push(Change::Removed { name: id.clone() });
    }
    out.extend(moved);
    out
}

/// A change in words: "3 working days less to spare (15 → 12)", "Frontend +3 wd (22 → 25)".
pub fn words(plan: &Plan, change: &Change) -> String {
    let name = |item: usize| plan.items[item].name.clone();
    match change {
        Change::Spare { item, before, now, wd } => format!("{} {} to spare: {} must now start by {} instead of {}", working_days(wd.abs()), if *wd < 0 { "less" } else { "more" }, name(*item), day(*now), day(*before)),
        Change::Length { item, before, now } => format!("{} {:+} wd ({before} → {now})", name(*item), now - before),
        Change::Added { item } => format!("{} is new", name(*item)),
        Change::Removed { name } => format!("{name} was removed"),
        Change::Moved { item, before, now } => format!("{} moved from {} to {}", name(*item), day(*before), day(*now)),
    }
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;
    use crate::Holidays;

    /// The sample as it stood on Thu 22 Oct with Frontend at 22 working days, kept as "Kick-off", and as
    /// it stands on Thu 29 Oct with Frontend at 25.
    fn kept() -> (Plan, Vec<Calendar>) {
        let mut plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
        let calendars: Vec<Calendar> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        let frontend = plan.item("frontend").unwrap();
        plan.items[frontend].duration = 22;
        let then = crate::schedule(&plan, &calendars, date(2026, 10, 22)).unwrap();
        let snapshot = Snapshot::take(&plan, &then, date(2026, 10, 22), Some("Kick-off".into()));
        plan.keep(snapshot);
        plan.items[frontend].duration = 25;
        (plan, calendars)
    }

    #[test]
    fn a_snapshot_is_kept_at_the_end_of_the_plan_file_and_read_back() {
        let (plan, _) = kept();
        let text = plan.to_toml();
        assert!(text.starts_with("format = 2\n") && text.contains("\n[[snapshots]]\nday = 2026-10-22\nname = \"Kick-off\"\nspare = 20\nstart_by = 2026-11-19\n") && text.contains("\nfrontend = [2026-12-24, 2027-01-27]\n"), "{text}");
        assert_eq!(Plan::parse(&text).unwrap(), plan);
        let old = include_str!("../../../samples/customer-portal.pchart");
        assert!(old.starts_with("format = 1\n") && Plan::parse(old).unwrap().to_toml().starts_with("format = 1\n"));
        assert!(Plan::parse(&text.replacen("format = 2", "format = 3", 1)).is_err());
    }

    #[test]
    fn what_changed_counts_the_plan_and_not_the_days_that_passed() {
        let (plan, calendars) = kept();
        let now = crate::schedule(&plan, &calendars, date(2026, 10, 29)).unwrap();
        let snapshot = last(&plan, date(2026, 10, 29)).unwrap();
        let said: Vec<String> = changes(&plan, &now, &calendars, snapshot).iter().map(|change| words(&plan, change)).collect();
        assert_eq!(said, ["3 working days less to spare: Requirements must now start by Mon 16 Nov instead of Thu 19 Nov", "Frontend +3 wd (22 → 25)", "Design sign-off moved from Wed 23 Dec to Fri 18 Dec"]);
        let mut same = plan.clone();
        let frontend = same.item("frontend").unwrap();
        same.items[frontend].duration = 22;
        let later = crate::schedule(&same, &calendars, date(2026, 11, 5)).unwrap();
        assert!(changes(&same, &later, &calendars, snapshot).is_empty(), "a week passing is not a change");
        let mut other = plan.clone();
        other.items.retain(|item| item.id != "security-review");
        for item in &mut other.items {
            item.after.retain(|id| id != "security-review");
        }
        other.items.push(crate::model::Item { id: "load-test".into(), lane: "qa".into(), name: "Load test".into(), duration: 3, after: vec!["integration".into()], ..Default::default() });
        let then = crate::schedule(&other, &calendars, date(2026, 10, 29)).unwrap();
        let said: Vec<String> = changes(&other, &then, &calendars, snapshot).iter().map(|change| words(&other, change)).collect();
        assert!(said.contains(&"Load test is new".to_string()) && said.contains(&"security-review was removed".to_string()), "{said:?}");
        assert!(last(&plan, date(2026, 10, 21)).is_none());
    }
}
