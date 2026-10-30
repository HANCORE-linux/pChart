use jiff::civil::Date;
use serde::Deserialize;

use crate::Error;
use crate::model::{Item, Plan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fix {
    On,
    NoEarlierThan,
    NoLaterThan,
}

/// One change to a plan. Agents send these as JSON (`{"op": "duration", "item": "frontend", "days": 30}`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Edit {
    Duration { item: String, days: u32 },
    Fix { item: String, kind: Fix, date: Date },
    /// Clears every fixed date, so the item is calculated again.
    Unfix { item: String },
    /// `to` waits on `from`.
    Link { from: String, to: String },
    Unlink { from: String, to: String },
    Add { item: Item },
    Remove { item: String },
    Rename { item: String, name: String },
    Lane { item: String, lane: String },
    Done { item: String, done: bool },
    Buffer { item: String, buffer: bool },
    Who { item: String, who: Option<String> },
}

impl Edit {
    pub fn apply(&self, plan: &Plan) -> Result<Plan, Error> {
        let mut next = plan.clone();
        let find = |id: &str| plan.item(id).ok_or_else(|| Error::Plan(format!("there is no item \"{id}\"")));
        match self {
            Edit::Duration { item, days } => {
                let index = find(item)?;
                if next.items[index].milestone {
                    return Err(Error::Plan(format!("\"{item}\" is a milestone; it has no duration")));
                }
                if *days == 0 {
                    return Err(Error::Plan(format!("\"{item}\" needs at least one working day")));
                }
                next.items[index].duration = *days;
            }
            Edit::Fix { item, kind, date } => {
                let target = &mut next.items[find(item)?];
                match kind {
                    Fix::On => (target.on, target.no_earlier_than, target.no_later_than) = (Some(*date), None, None),
                    Fix::NoEarlierThan => (target.on, target.no_earlier_than) = (None, Some(*date)),
                    Fix::NoLaterThan => (target.on, target.no_later_than) = (None, Some(*date)),
                }
            }
            Edit::Unfix { item } => {
                let target = &mut next.items[find(item)?];
                (target.on, target.no_earlier_than, target.no_later_than) = (None, None, None);
            }
            Edit::Link { from, to } => {
                let (first, then) = (find(from)?, find(to)?);
                if first == then || waits_on(plan, first, then) {
                    return Err(Error::Plan(format!("\"{from}\" already waits on \"{to}\"; linking them would make a circle")));
                }
                if !next.items[then].after.contains(from) {
                    next.items[then].after.push(from.clone());
                }
            }
            Edit::Unlink { from, to } => {
                let then = find(to)?;
                find(from)?;
                let before = next.items[then].after.len();
                next.items[then].after.retain(|id| id != from);
                if next.items[then].after.len() == before {
                    return Err(Error::Plan(format!("\"{to}\" does not wait on \"{from}\"")));
                }
            }
            Edit::Add { item } => {
                if plan.item(&item.id).is_some() {
                    return Err(Error::Plan(format!("the id \"{}\" is already used", item.id)));
                }
                next.items.push(item.clone());
            }
            Edit::Remove { item } => {
                let index = find(item)?;
                let waiting: Vec<&str> = plan.items.iter().filter(|other| other.after.contains(item)).map(|other| other.id.as_str()).collect();
                if !waiting.is_empty() {
                    return Err(Error::Plan(format!("\"{item}\" cannot be removed: {} wait on it", waiting.join(", "))));
                }
                if let Some(phase) = plan.phases.iter().find(|phase| &phase.until == item) {
                    return Err(Error::Plan(format!("\"{item}\" cannot be removed: the phase \"{}\" ends with it", phase.name)));
                }
                next.items.remove(index);
            }
            Edit::Rename { item, name } => next.items[find(item)?].name = name.clone(),
            Edit::Lane { item, lane } => {
                if plan.lane(lane).is_none() {
                    return Err(Error::Plan(format!("there is no lane \"{lane}\"")));
                }
                next.items[find(item)?].lane = lane.clone();
            }
            Edit::Done { item, done } => next.items[find(item)?].done = *done,
            Edit::Buffer { item, buffer } => next.items[find(item)?].buffer = *buffer,
            Edit::Who { item, who } => next.items[find(item)?].who = who.clone(),
        }
        next.check()?;
        Ok(next)
    }
}

/// Applies edits in order; all of them or none.
pub fn apply_all(plan: &Plan, edits: &[Edit]) -> Result<Plan, (usize, Error)> {
    edits.iter().enumerate().try_fold(plan.clone(), |plan, (index, edit)| edit.apply(&plan).map_err(|err| (index, err)))
}

/// Whether `item` waits on `other`, directly or through others.
fn waits_on(plan: &Plan, item: usize, other: usize) -> bool {
    let mut stack = vec![item];
    let mut seen = vec![false; plan.items.len()];
    while let Some(index) = stack.pop() {
        for pred in plan.items[index].after.iter().filter_map(|id| plan.item(id)) {
            if pred == other {
                return true;
            }
            if !std::mem::replace(&mut seen[pred], true) {
                stack.push(pred);
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Plan {
        Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap()
    }

    fn edits(json: &str) -> Vec<Edit> {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn edits_read_from_json() {
        let list = edits(
            r#"[{"op": "duration", "item": "frontend", "days": 30},
                {"op": "fix", "item": "frontend", "kind": "no_earlier_than", "date": "2027-01-04"},
                {"op": "add", "item": {"id": "buffer", "lane": "client", "name": "Buffer", "duration": "5wd", "after": ["acceptance"]}},
                {"op": "link", "from": "buffer", "to": "launch"},
                {"op": "buffer", "item": "buffer", "buffer": true}]"#,
        );
        let plan = apply_all(&sample(), &list).unwrap();
        let frontend = &plan.items[plan.item("frontend").unwrap()];
        assert_eq!((frontend.duration, frontend.no_earlier_than), (30, Some(jiff::civil::date(2027, 1, 4))));
        assert_eq!(plan.items[plan.item("launch").unwrap()].after, ["acceptance", "buffer"]);
        assert_eq!((plan.items[plan.item("buffer").unwrap()].duration, plan.items[plan.item("buffer").unwrap()].buffer), (5, true));
    }

    #[test]
    fn a_link_that_closes_a_circle_is_refused() {
        let err = Edit::Link { from: "launch".into(), to: "requirements".into() }.apply(&sample()).unwrap_err();
        assert_eq!(err.to_string(), "\"launch\" already waits on \"requirements\"; linking them would make a circle");
    }

    #[test]
    fn a_batch_is_all_or_nothing() {
        let list = edits(r#"[{"op": "rename", "item": "api", "name": "REST API"}, {"op": "remove", "item": "data-model"}]"#);
        let (index, err) = apply_all(&sample(), &list).unwrap_err();
        assert_eq!(index, 1);
        assert_eq!(err.to_string(), "\"data-model\" cannot be removed: api, sdk-integration wait on it");
    }

    #[test]
    fn fixing_on_a_date_replaces_the_other_kinds() {
        let plan = edits(r#"[{"op": "fix", "item": "api", "kind": "no_later_than", "date": "2027-01-29"}, {"op": "fix", "item": "api", "kind": "on", "date": "2026-12-14"}]"#);
        let plan = apply_all(&sample(), &plan).unwrap();
        let api = &plan.items[plan.item("api").unwrap()];
        assert_eq!((api.on, api.no_later_than), (Some(jiff::civil::date(2026, 12, 14)), None));
    }
}
