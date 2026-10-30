use jiff::civil::Date;
use serde::{Deserialize, Deserializer};

use crate::Error;

pub const FORMAT: u32 = 1;

#[derive(Debug, Clone, Deserialize)]
pub struct Plan {
    pub format: u32,
    pub name: String,
    #[serde(default, deserialize_with = "date")]
    pub start: Option<Date>,
    #[serde(default)]
    pub phases: Vec<Phase>,
    pub lanes: Vec<Lane>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Phase {
    pub name: String,
    pub until: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Lane {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub place: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
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
    #[serde(default, deserialize_with = "date")]
    pub on: Option<Date>,
    #[serde(default, deserialize_with = "date")]
    pub no_earlier_than: Option<Date>,
    #[serde(default, deserialize_with = "date")]
    pub no_later_than: Option<Date>,
}

impl Plan {
    pub fn parse(text: &str) -> Result<Plan, Error> {
        let plan: Plan = toml::from_str(text).map_err(|err| Error::File(err.to_string()))?;
        if plan.format > FORMAT {
            return Err(Error::File(format!("format {} is newer than this pChart (reads up to {FORMAT})", plan.format)));
        }
        plan.check()?;
        Ok(plan)
    }

    pub fn item(&self, id: &str) -> Option<usize> {
        self.items.iter().position(|item| item.id == id)
    }

    pub fn lane(&self, id: &str) -> Option<usize> {
        self.lanes.iter().position(|lane| lane.id == id)
    }

    fn check(&self) -> Result<(), Error> {
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
            if !item.milestone && item.duration == 0 {
                return Err(Error::Plan(format!("\"{}\" has no duration", item.id)));
            }
        }
        for phase in &self.phases {
            if self.item(&phase.until).is_none() {
                return Err(Error::Plan(format!("the phase \"{}\" ends at the unknown item \"{}\"", phase.name, phase.until)));
            }
        }
        Ok(())
    }
}

fn date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Date>, D::Error> {
    let Some(value) = Option::<toml::value::Datetime>::deserialize(deserializer)? else {
        return Ok(None);
    };
    match (value.date, value.time) {
        (Some(day), None) => Date::new(day.year as i16, day.month as i8, day.day as i8).map(Some).map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom(format!("{value} is not a date like 2027-03-01"))),
    }
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
