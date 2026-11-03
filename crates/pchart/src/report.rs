use jiff::civil::Date;
use pchart_core::analysis::{Analysis, Check, Handoff, Next, Severity, State, Verdict};
use pchart_core::{Plan, Schedule, text};
use serde_json::{Value, json};

pub fn iso(date: Date) -> String {
    date.to_string()
}

fn id(plan: &Plan, item: usize) -> &str {
    &plan.items[item].id
}

pub fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Problem => "problem",
        Severity::Warning => "warning",
        Severity::Hint => "hint",
    }
}

pub fn verdict(plan: &Plan, verdict: &Verdict) -> Value {
    let state = match verdict.state {
        State::Holds => "holds",
        State::Tight => "tight",
        State::DoesNotHold => "does-not-hold",
    };
    let (headline, detail) = text::verdict(plan, verdict);
    let words = format!("{headline} {detail}");
    json!({
        "state": state,
        "target": id(plan, verdict.target),
        "date": iso(verdict.date),
        "spare_wd": verdict.spare,
        "latest_start": iso(verdict.latest_start),
        "item": id(plan, verdict.first),
        "text": words,
    })
}

pub fn handoff(plan: &Plan, handoff: &Handoff) -> Value {
    json!({
        "who": plan.lanes[handoff.waiting].id,
        "on": plan.lanes[handoff.on].id,
        "item": id(plan, handoff.item),
        "due": iso(handoff.due),
        "deciding": handoff.deciding,
        "text": text::handoff(plan, handoff),
    })
}

pub fn check(plan: &Plan, check: &Check) -> Value {
    let (title, detail) = text::check(plan, check);
    json!({
        "rule": check.rule(),
        "severity": severity(check.severity()),
        "items": check.items().into_iter().map(|item| id(plan, item)).collect::<Vec<_>>(),
        "title": title,
        "detail": detail,
    })
}

pub fn next(plan: &Plan, analysis: &Analysis, next: &Next) -> Value {
    let words = text::next(plan, analysis, next);
    match next {
        Next::StartBy { item, date, in_wd } => json!({ "kind": "start-by", "item": id(plan, *item), "date": iso(*date), "in_wd": in_wd, "text": words }),
        Next::Due { item, date, in_wd } => json!({ "kind": "due", "item": id(plan, *item), "date": iso(*date), "in_wd": in_wd, "text": words }),
        Next::Check(index) => {
            let check = &analysis.checks[*index];
            json!({ "kind": "check", "rule": check.rule(), "severity": severity(check.severity()), "items": check.items().into_iter().map(|item| id(plan, item)).collect::<Vec<_>>(), "text": words })
        }
    }
}

pub fn situation_json(plan: &Plan, schedule: &Schedule, analysis: &Analysis) -> Value {
    json!({
        "schema": 1,
        "verdict": analysis.verdict.as_ref().map(|v| verdict(plan, v)),
        "waits": analysis.handoffs.iter().map(|h| handoff(plan, h)).collect::<Vec<_>>(),
        "next": analysis.next.iter().map(|n| next(plan, analysis, n)).collect::<Vec<_>>(),
        "chain": schedule.chain.iter().map(|&item| id(plan, item)).collect::<Vec<_>>(),
    })
}

pub fn situation_text(plan: &Plan, schedule: &Schedule, analysis: &Analysis) -> String {
    let mut out = String::new();
    match &analysis.verdict {
        Some(verdict) => {
            let (headline, detail) = text::verdict(plan, verdict);
            out.push_str(&format!("{headline}\n{detail}\n"));
        }
        None => out.push_str(&format!("{} has no fixed target date; it ends {}.\n", plan.name, schedule.slots.iter().map(|slot| slot.planned.finish).max().map(text::day).unwrap_or_default())),
    }
    let deciding: Vec<&Handoff> = analysis.handoffs.iter().filter(|handoff| handoff.deciding).collect();
    if !analysis.handoffs.is_empty() {
        out.push_str("\nWho is waiting on whom\n");
        for handoff in &deciding {
            out.push_str(&format!("  {}\n", text::handoff(plan, handoff)));
        }
        let rest = analysis.handoffs.len() - deciding.len();
        if rest > 0 {
            out.push_str(&format!("  and {rest} more with time to spare (pchart waits)\n"));
        }
    }
    out.push_str("\nWhat is burning next\n");
    if analysis.next.is_empty() {
        out.push_str("  Nothing is due.\n");
    }
    for (number, item) in analysis.next.iter().enumerate() {
        let mark = match item {
            Next::Check(index) if analysis.checks[*index].severity() == Severity::Problem => "✖ ",
            Next::Check(_) => "▲ ",
            _ => "",
        };
        out.push_str(&format!("  {}. {mark}{}\n", number + 1, text::next(plan, analysis, item)));
    }
    if !schedule.chain.is_empty() {
        out.push_str(&format!("\nThis chain decides the date\n  {}\n", text::chain(plan, &schedule.chain)));
    }
    out
}

pub fn checks_text(plan: &Plan, analysis: &Analysis) -> String {
    if analysis.checks.is_empty() {
        return "No problems, no warnings.\n".into();
    }
    let mut out = String::new();
    for check in &analysis.checks {
        let (title, detail) = text::check(plan, check);
        let mark = match check.severity() {
            Severity::Problem => "✖",
            Severity::Warning => "▲",
            Severity::Hint => "·",
        };
        out.push_str(&format!("{mark} {title}\n  {detail}\n"));
    }
    out
}

/// An item whose planned first and last day changed; `None` where it did not exist.
pub struct Moved {
    pub id: String,
    pub before: Option<(Date, Date)>,
    pub after: Option<(Date, Date)>,
}

/// Items whose planned dates differ between two schedules of the same items.
pub fn moved(before: (&Plan, &Schedule), after: (&Plan, &Schedule)) -> Vec<Moved> {
    let span = |(plan, schedule): (&Plan, &Schedule), id: &str| plan.item(id).map(|index| (schedule.slots[index].planned.start, schedule.slots[index].planned.finish));
    let mut ids: Vec<&str> = before.0.items.iter().map(|item| item.id.as_str()).collect();
    ids.extend(after.0.items.iter().map(|item| item.id.as_str()).filter(|id| before.0.item(id).is_none()));
    ids.into_iter()
        .filter_map(|id| {
            let (old, new) = (span(before, id), span(after, id));
            (old != new).then(|| Moved { id: id.to_string(), before: old, after: new })
        })
        .collect()
}
