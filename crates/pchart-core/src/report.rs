//! Reports (pictures 65): someone in the team says how a piece of work stands, in a small file of its
//! own in the workspace's `reports` folder, so that no two people write the same file. The plan does not
//! change until the project lead takes a report.

use std::fmt::Write;

use jiff::civil::DateTime;
use serde::{Deserialize, Deserializer};

use crate::Error;
use crate::edit::Edit;
use crate::model::{Plan, slug};

/// How a piece of work stands, as someone reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Done,
    OnTrack,
    /// Something may make it late; it holds for now.
    Risk,
    /// It needs `days` working days more.
    Later,
}

/// What the project lead answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Answer {
    Taken,
    Declined,
}

/// What the project lead took when they changed a report before taking it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Taken {
    pub status: Status,
    #[serde(default)]
    pub days: u32,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Report {
    /// The plan file it is about, by its name in `projects`.
    pub project: String,
    pub item: String,
    pub by: String,
    #[serde(deserialize_with = "moment")]
    pub at: DateTime,
    pub status: Status,
    #[serde(default)]
    pub days: u32,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub answer: Option<Answer>,
    /// What was taken, when it is not what was reported.
    #[serde(default)]
    pub taken: Option<Taken>,
}

fn moment<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Written {
        Toml(toml::value::Datetime),
        Text(String),
    }
    let text = match Written::deserialize(deserializer)? {
        Written::Toml(value) => value.to_string(),
        Written::Text(text) => text,
    };
    text.trim_end_matches('Z').parse::<DateTime>().map_err(|_| serde::de::Error::custom(format!("{text} is not a time like 2026-11-02T14:15")))
}

fn name(status: Status) -> &'static str {
    match status {
        Status::Done => "done",
        Status::OnTrack => "on-track",
        Status::Risk => "risk",
        Status::Later => "later",
    }
}

fn quote(text: &str) -> String {
    toml::Value::String(text.to_string()).to_string()
}

impl Report {
    pub fn parse(text: &str) -> Result<Report, Error> {
        toml::from_str(text).map_err(|err| Error::File(err.to_string()))
    }

    /// The report file text.
    pub fn to_toml(&self) -> String {
        let status = name(self.status);
        let mut out = format!("project = {}\nitem = {}\nby = {}\nat = {}\nstatus = \"{status}\"\n", quote(&self.project), quote(&self.item), quote(&self.by), self.at.strftime("%Y-%m-%dT%H:%M:%S"));
        if self.days > 0 {
            let _ = writeln!(out, "days = {}", self.days);
        }
        if let Some(comment) = &self.comment {
            let _ = writeln!(out, "comment = {}", quote(comment));
        }
        if let Some(answer) = self.answer {
            let _ = writeln!(out, "answer = \"{}\"", if answer == Answer::Taken { "taken" } else { "declined" });
        }
        if let Some(taken) = &self.taken {
            let mut parts = vec![format!("status = \"{}\"", name(taken.status))];
            if taken.days > 0 {
                parts.push(format!("days = {}", taken.days));
            }
            if let Some(comment) = &taken.comment {
                parts.push(format!("comment = {}", quote(comment)));
            }
            let _ = writeln!(out, "taken = {{ {} }}", parts.join(", "));
        }
        out
    }

    /// This report as the project lead took it: with what they changed, if anything.
    pub fn as_taken(&self) -> Report {
        let mut out = self.clone();
        if let Some(taken) = &self.taken {
            (out.status, out.days, out.comment) = (taken.status, taken.days, taken.comment.clone());
        }
        out
    }

    /// The name of its file: when, who and what, as `2026-11-02-1415-priya-nair-frontend.toml`.
    pub fn file_name(&self) -> String {
        format!("{}-{}-{}.toml", self.at.strftime("%Y-%m-%d-%H%M"), slug(&self.by).unwrap_or_else(|| "someone".into()), slug(&self.item).unwrap_or_else(|| "item".into()))
    }

    /// What taking it changes in `plan`: done marks the work done, more time makes it longer, and a
    /// risk or "on track" with a comment keeps it as the item's note. None when the item is gone.
    pub fn edits(&self, plan: &Plan) -> Option<Vec<Edit>> {
        let item = &plan.items[plan.item(&self.item)?];
        let note = self.comment.as_ref().map(|comment| Edit::Note { item: item.id.clone(), note: Some(format!("{} ({}, {}): {comment}", self.words(), self.by, self.at.strftime("%d %b"))) });
        Some(match self.status {
            Status::Done => vec![Edit::Done { item: item.id.clone(), done: true }],
            Status::Later if !item.milestone && self.days > 0 => std::iter::once(Edit::Duration { item: item.id.clone(), days: item.duration + self.days }).chain(note).collect(),
            _ => note.into_iter().collect(),
        })
    }

    /// The plan as it would be with this report taken: the plan, its dates and what they mean. None
    /// when its item is gone or it cannot be taken.
    pub fn then(&self, plan: &Plan, calendars: &[crate::Calendar], today: jiff::civil::Date) -> Option<(Plan, crate::Schedule, crate::Analysis)> {
        let next = crate::edit::apply_all(plan, &self.edits(plan)?).ok()?;
        let schedule = crate::schedule(&next, calendars, today).ok()?;
        let analysis = crate::analyse(&next, calendars, &schedule, today);
        Some((next, schedule, analysis))
    }

    /// The status in words: "done", "on track", "at risk", "needs 3 working days more".
    pub fn words(&self) -> String {
        match self.status {
            Status::Done => "done".into(),
            Status::OnTrack => "on track".into(),
            Status::Risk => "at risk".into(),
            Status::Later if self.days == 1 => "needs 1 more working day".into(),
            Status::Later => format!("needs {} more working days", self.days),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(status: Status, days: u32) -> Report {
        Report { project: "portal.pchart".into(), item: "frontend".into(), by: "Priya Nair".into(), at: "2026-11-02T14:15".parse().unwrap(), status, days, comment: Some("Backend API comes late".into()), answer: None, taken: None }
    }

    #[test]
    fn a_report_is_written_and_read_back() {
        let mut later = report(Status::Later, 3);
        assert_eq!(later.file_name(), "2026-11-02-1415-priya-nair-frontend.toml");
        assert_eq!(Report::parse(&later.to_toml()).unwrap(), later);
        later.answer = Some(Answer::Declined);
        assert_eq!(Report::parse(&later.to_toml()).unwrap(), later);
        later.answer = Some(Answer::Taken);
        later.taken = Some(Taken { status: Status::Later, days: 2, comment: Some("Two days, the API team helps".into()) });
        assert_eq!(Report::parse(&later.to_toml()).unwrap(), later);
        assert_eq!((later.as_taken().days, later.as_taken().words()), (2, "needs 2 more working days".to_string()));
        assert!(Report::parse("project = 'a'\nitem = 'b'\nby = 'c'\nat = 2026-11-02T14:15:00\nstatus = 'risk'\n").is_ok());
        assert!(Report::parse("project = 'a'\nitem = 'b'\nby = 'c'\nat = 2026-11-02T14:15:00\nstatus = 'maybe'\n").is_err());
    }

    #[test]
    fn taking_a_report_changes_only_what_it_says() {
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        let frontend = plan.items[plan.item("frontend").unwrap()].duration;
        assert_eq!(report(Status::Later, 3).edits(&plan).unwrap()[0], Edit::Duration { item: "frontend".into(), days: frontend + 3 });
        assert_eq!(report(Status::Done, 0).edits(&plan).unwrap(), [Edit::Done { item: "frontend".into(), done: true }]);
        assert!(matches!(report(Status::Risk, 0).edits(&plan).unwrap().as_slice(), [Edit::Note { .. }]));
        let mut gone = report(Status::Done, 0);
        gone.item = "nothing".into();
        assert!(gone.edits(&plan).is_none());
    }
}
