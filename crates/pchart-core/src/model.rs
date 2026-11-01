use std::fmt::Write;

use jiff::civil::Date;
use serde::{Deserialize, Deserializer};

use crate::Error;

pub const FORMAT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Plan {
    pub format: u32,
    pub name: String,
    #[serde(default, deserialize_with = "date")]
    pub start: Option<Date>,
    #[serde(default)]
    pub phases: Vec<Phase>,
    pub lanes: Vec<Lane>,
    pub items: Vec<Item>,
    #[serde(default)]
    pub notes: Vec<Note>,
}

/// A note at a date in a lane; it stays there when the plan moves (picture 43).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Note {
    pub lane: String,
    #[serde(deserialize_with = "day")]
    pub date: Date,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Phase {
    pub name: String,
    pub until: String,
    /// What a phase arrow shows when the whole name does not fit, even small or on two lines.
    #[serde(default)]
    pub short: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Lane {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub place: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Item {
    pub id: String,
    pub lane: String,
    pub name: String,
    #[serde(default, deserialize_with = "workdays")]
    pub duration: u32,
    #[serde(default)]
    pub after: Vec<String>,
    #[serde(default)]
    pub milestone: bool,
    /// Time held back before a fixed date, so the work before it does not end right before it.
    #[serde(default)]
    pub buffer: bool,
    #[serde(default, deserialize_with = "date")]
    pub on: Option<Date>,
    #[serde(default, deserialize_with = "date")]
    pub no_earlier_than: Option<Date>,
    #[serde(default, deserialize_with = "date")]
    pub no_later_than: Option<Date>,
    #[serde(default)]
    pub done: bool,
    /// A milestone drawn as a star and never left out (pictures 39 and 41).
    #[serde(default)]
    pub important: bool,
    /// The person or team doing the item; two items of the same `who` must not overlap.
    #[serde(default)]
    pub who: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// The words of `text` in small letters joined by "-", or None when it has none.
pub fn slug(text: &str) -> Option<String> {
    let words: Vec<String> = text.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|word| !word.is_empty()).map(str::to_string).collect();
    (!words.is_empty()).then(|| words.join("-"))
}

impl Plan {
    /// An id for a new item called `name`: its words in small letters joined by "-", numbered when the id
    /// is taken.
    pub fn new_id(&self, name: &str) -> String {
        let base = slug(name).unwrap_or_else(|| "item".into());
        (1..).map(|number| if number == 1 { base.clone() } else { format!("{base}-{number}") }).find(|id| self.item(id).is_none()).unwrap_or(base)
    }

    /// A new plan from this template, called `name`, with `date` on its target, the last milestone
    /// nothing waits on, or as its start when `forwards` or when it has no such milestone. Without a
    /// date it starts on the day it is scheduled from.
    pub fn new_plan(&self, name: &str, date: Option<Date>, forwards: bool) -> Plan {
        let mut plan = self.clone();
        plan.name = name.to_string();
        let target = (0..plan.items.len()).rev().find(|&index| plan.items[index].milestone && !plan.items.iter().any(|other| other.after.contains(&plan.items[index].id)));
        if let Some(date) = date {
            match target.filter(|_| !forwards) {
                Some(target) => plan.items[target].on = Some(date),
                None => plan.start = Some(date),
            }
        }
        plan
    }

    /// The work whose length is not known yet. Until every piece of work has one, the plan has no dates.
    pub fn without_length(&self) -> Vec<usize> {
        (0..self.items.len()).filter(|&index| !self.items[index].milestone && self.items[index].duration == 0).collect()
    }

    /// This plan with its structure only: the work without lengths and nothing fixed on a date.
    pub fn structure_only(&self) -> Plan {
        let mut plan = self.clone();
        plan.start = None;
        for item in &mut plan.items {
            item.duration = 0;
            (item.on, item.no_earlier_than, item.no_later_than) = (None, None, None);
        }
        plan
    }

    pub fn parse(text: &str) -> Result<Plan, Error> {
        let plan: Plan = toml::from_str(text).map_err(|err| Error::File(err.to_string()))?;
        if plan.format > FORMAT {
            return Err(Error::File(format!("format {} is newer than this pChart (reads up to {FORMAT})", plan.format)));
        }
        plan.check()?;
        Ok(plan)
    }

    /// The plan file text: readable TOML in a fixed order, with only the fields that are set.
    pub fn to_toml(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "format = {}", self.format);
        let _ = writeln!(out, "name = {}", quote(&self.name));
        if let Some(start) = self.start {
            let _ = writeln!(out, "start = {start}");
        }
        for phase in &self.phases {
            let _ = write!(out, "\n[[phases]]\nname = {}\nuntil = {}\n", quote(&phase.name), quote(&phase.until));
            if let Some(short) = &phase.short {
                let _ = writeln!(out, "short = {}", quote(short));
            }
        }
        for lane in &self.lanes {
            let _ = write!(out, "\n[[lanes]]\nid = {}\nname = {}\n", quote(&lane.id), quote(&lane.name));
            for (key, value) in [("place", &lane.place), ("city", &lane.city)] {
                if let Some(value) = value {
                    let _ = writeln!(out, "{key} = {}", quote(value));
                }
            }
        }
        for item in &self.items {
            let _ = write!(out, "\n[[items]]\nid = {}\nlane = {}\nname = {}\n", quote(&item.id), quote(&item.lane), quote(&item.name));
            if item.milestone {
                out.push_str("milestone = true\n");
            } else if item.duration > 0 {
                let _ = writeln!(out, "duration = \"{}wd\"", item.duration);
            }
            if item.buffer {
                out.push_str("buffer = true\n");
            }
            if !item.after.is_empty() {
                let after: Vec<String> = item.after.iter().map(|id| quote(id)).collect();
                let _ = writeln!(out, "after = [{}]", after.join(", "));
            }
            for (key, value) in [("on", item.on), ("no_earlier_than", item.no_earlier_than), ("no_later_than", item.no_later_than)] {
                if let Some(day) = value {
                    let _ = writeln!(out, "{key} = {day}");
                }
            }
            if item.done {
                out.push_str("done = true\n");
            }
            if item.important {
                out.push_str("important = true\n");
            }
            for (key, value) in [("who", &item.who), ("note", &item.note)] {
                if let Some(value) = value {
                    let _ = writeln!(out, "{key} = {}", quote(value));
                }
            }
        }
        for note in &self.notes {
            let _ = write!(out, "\n[[notes]]\nlane = {}\ndate = {}\ntext = {}\n", quote(&note.lane), note.date, quote(&note.text));
        }
        out
    }

    pub fn item(&self, id: &str) -> Option<usize> {
        self.items.iter().position(|item| item.id == id)
    }

    pub fn lane(&self, id: &str) -> Option<usize> {
        self.lanes.iter().position(|lane| lane.id == id)
    }

    pub(crate) fn check(&self) -> Result<(), Error> {
        for (index, item) in self.items.iter().enumerate() {
            if self.items[..index].iter().any(|other| other.id == item.id) {
                return Err(Error::Plan(format!("the id \"{}\" is used twice", item.id)));
            }
            if self.lane(&item.lane).is_none() {
                return Err(Error::Plan(format!("\"{}\" is in the unknown lane \"{}\"", item.id, item.lane)));
            }
            if let Some(missing) = item.after.iter().find(|id| self.item(id).is_none()) {
                return Err(Error::Plan(format!("\"{}\" waits on the unknown item \"{missing}\"", item.id)));
            }
            if item.milestone && item.duration > 0 {
                return Err(Error::Plan(format!("the milestone \"{}\" has a duration", item.id)));
            }
            if item.milestone && item.buffer {
                return Err(Error::Plan(format!("the milestone \"{}\" cannot be a buffer; a buffer takes time", item.id)));
            }
            if item.important && !item.milestone {
                return Err(Error::Plan(format!("\"{}\" is not a milestone; only milestones are marked important", item.id)));
            }
        }
        if let Some(note) = self.notes.iter().find(|note| self.lane(&note.lane).is_none()) {
            return Err(Error::Plan(format!("the note on {} is in the unknown lane \"{}\"", note.date, note.lane)));
        }
        for phase in &self.phases {
            if self.item(&phase.until).is_none() {
                return Err(Error::Plan(format!("the phase \"{}\" ends at the unknown item \"{}\"", phase.name, phase.until)));
            }
        }
        Ok(())
    }
}

fn quote(text: &str) -> String {
    toml::Value::String(text.to_string()).to_string()
}

/// A TOML date (`2027-03-01`) in plan files, or a string in JSON edits.
fn date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Date>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Written {
        Toml(toml::value::Datetime),
        Text(String),
    }
    let not_a_date = |text: String| serde::de::Error::custom(format!("{text} is not a date like 2027-03-01"));
    match Option::<Written>::deserialize(deserializer)? {
        None => Ok(None),
        Some(Written::Text(text)) => text.parse::<Date>().map(Some).map_err(|_| not_a_date(text)),
        Some(Written::Toml(value)) => match (value.date, value.time) {
            (Some(day), None) => Date::new(day.year as i16, day.month as i8, day.day as i8).map(Some).map_err(serde::de::Error::custom),
            _ => Err(not_a_date(value.to_string())),
        },
    }
}

fn day<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Date, D::Error> {
    date(deserializer)?.ok_or_else(|| serde::de::Error::custom("a note needs a date"))
}

fn workdays<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    let text = String::deserialize(deserializer)?;
    parse_workdays(&text).ok_or_else(|| serde::de::Error::custom(format!("\"{text}\" is not a duration like \"25wd\" or \"3w\"")))
}

pub fn parse_workdays(text: &str) -> Option<u32> {
    let text = text.trim();
    let split = text.find(|c: char| !c.is_ascii_digit())?;
    let count: u32 = text[..split].parse().ok()?;
    match text[split..].trim() {
        "wd" => Some(count),
        "w" => Some(count * 5),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_read_working_days_and_weeks() {
        assert_eq!(parse_workdays("25wd"), Some(25));
        assert_eq!(parse_workdays("3w"), Some(15));
        assert_eq!(parse_workdays("3 w"), Some(15));
        assert_eq!(parse_workdays("3d"), None);
        assert_eq!(parse_workdays("wd"), None);
    }

    #[test]
    fn dates_are_toml_dates() {
        let plan = Plan::parse(
            r#"
            format = 1
            name = "x"
            [[lanes]]
            id = "a"
            name = "A"
            [[items]]
            id = "launch"
            lane = "a"
            name = "Launch"
            milestone = true
            on = 2027-03-01
            "#,
        )
        .unwrap();
        assert_eq!(plan.items[0].on, Some(jiff::civil::date(2027, 3, 1)));
    }

    #[test]
    fn the_sample_plan_survives_a_round_trip() {
        let text = include_str!("../../../samples/customer-portal.pchart");
        let plan = Plan::parse(text).unwrap();
        assert_eq!(plan.to_toml(), text);
        let mut changed = plan.clone();
        changed.items[0].who = Some("Anna \"A\" Berg".into());
        changed.items[0].done = true;
        changed.items[0].buffer = true;
        changed.items[1].no_later_than = Some(jiff::civil::date(2026, 12, 1));
        changed.start = Some(jiff::civil::date(2026, 11, 2));
        changed.phases[0].short = Some("Reqs".into());
        changed.items[3].important = true;
        changed.items[3].note = Some("Client signs the design in a workshop in New York.".into());
        changed.notes.push(Note { lane: "backend".into(), date: jiff::civil::date(2027, 1, 21), text: "Server move".into() });
        assert_eq!(Plan::parse(&changed.to_toml()).unwrap(), changed);
    }

    #[test]
    fn a_new_item_gets_an_id_from_its_name_that_is_not_taken() {
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        assert_eq!(plan.new_id("Code review"), "code-review");
        assert_eq!(plan.new_id("Frontend"), "frontend-2");
        assert_eq!(plan.new_id("Beta & acceptance!"), "beta-acceptance");
        assert_eq!(plan.new_id("…"), "item");
    }

    #[test]
    fn a_new_plan_puts_its_date_on_the_last_milestone_nothing_waits_on() {
        let mut template = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        let launch = template.item("launch").unwrap();
        template.items[launch].on = None;
        let day = Date::constant(2027, 3, 1);
        let backwards = template.new_plan("Portal", Some(day), false);
        assert_eq!((backwards.name.as_str(), backwards.items[launch].on, backwards.start), ("Portal", Some(day), None));
        let forwards = template.new_plan("Portal", Some(day), true);
        assert_eq!((forwards.items[launch].on, forwards.start), (None, Some(day)));
        assert_eq!(template.new_plan("Portal", None, false).items, template.items);
    }

    #[test]
    fn a_plan_can_be_built_before_its_lengths_are_known() {
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap().structure_only();
        let work: Vec<usize> = (0..plan.items.len()).filter(|&index| !plan.items[index].milestone).collect();
        assert_eq!(plan.without_length(), work);
        assert!(plan.start.is_none() && plan.items.iter().all(|item| item.on.is_none() && item.no_earlier_than.is_none() && item.no_later_than.is_none()));
        let text = plan.to_toml();
        assert!(!text.contains("duration"));
        assert_eq!(Plan::parse(&text).unwrap(), plan);
    }

    #[test]
    fn unknown_predecessors_are_reported() {
        let err = Plan::parse(
            r#"
            format = 1
            name = "x"
            [[lanes]]
            id = "a"
            name = "A"
            [[items]]
            id = "b"
            lane = "a"
            name = "B"
            duration = "2wd"
            after = ["nope"]
            "#,
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "\"b\" waits on the unknown item \"nope\"");
    }
}
