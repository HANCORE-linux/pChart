mod cli;
mod report;

use std::io::Read;
use std::path::Path;
use std::process::ExitCode;

use jiff::civil::Date;
use pchart_core::analysis::Severity;
use pchart_core::edit::{Edit, Fix};
use pchart_core::model::{Item, parse_workdays};
use pchart_core::{Calendar, Holidays, Plan, Schedule, analyse, text};
use pchart_draw::{Output, Palette, Settings};
use serde_json::json;

use crate::cli::Args;

const SAMPLE_HOLIDAYS: &str = include_str!("../../../samples/holidays.tsv");

/// Why a command failed: exit code, kind for programs, message and a hint.
struct Failure {
    code: u8,
    kind: &'static str,
    message: String,
    hint: Option<String>,
}

impl Failure {
    fn usage(message: impl Into<String>, args: &Args) -> Failure {
        let usage = args.usage();
        Failure { code: 3, kind: "usage", message: message.into(), hint: (!usage.is_empty()).then(|| format!("usage: {usage}")) }
    }

    fn file(message: impl Into<String>) -> Failure {
        Failure { code: 4, kind: "file", message: message.into(), hint: None }
    }

    fn edit(message: impl Into<String>) -> Failure {
        Failure { code: 3, kind: "edit", message: message.into(), hint: Some("pchart help --json lists every command and edit".into()) }
    }
}

fn main() -> ExitCode {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let json = words.iter().any(|word| word == "--json");
    match Args::parse(words).map_err(|message| Failure { code: 3, kind: "usage", message, hint: Some("pchart help".into()) }).and_then(|args| run(&args)) {
        Ok(code) => ExitCode::from(code),
        Err(failure) => {
            if json {
                eprintln!("{}", json!({ "schema": 1, "error": { "kind": failure.kind, "message": failure.message, "hint": failure.hint } }));
            } else {
                eprintln!("{}", failure.message);
                if let Some(hint) = failure.hint {
                    eprintln!("{hint}");
                }
            }
            ExitCode::from(failure.code)
        }
    }
}

struct Context {
    path: std::path::PathBuf,
    plan: Plan,
    calendars: Vec<Calendar>,
    holidays: Holidays,
    today: Date,
    schedule: Schedule,
}

fn context(args: &Args) -> Result<Context, Failure> {
    let path = std::path::PathBuf::from(args.words.first().ok_or_else(|| Failure::usage("which plan? Name the plan file.", args))?);
    let plan = pchart_store::load(&path).map_err(|err| Failure::file(err.to_string()))?;
    let today = match args.option("--today") {
        Some(text) => date(text, args)?,
        None => jiff::Zoned::now().date(),
    };
    let holidays = match args.option("--holidays") {
        Some(file) => Holidays::parse(&std::fs::read_to_string(file).map_err(|err| Failure::file(format!("{file}: {err}")))?),
        None => Holidays::parse(SAMPLE_HOLIDAYS),
    };
    if !args.flag("--json") {
        for lane in &plan.lanes {
            if let Some(place) = &lane.place
                && !holidays.knows(place)
            {
                eprintln!("No holiday data for {place} (lane {}); counting weekends only.", lane.name);
            }
        }
    }
    let calendars: Vec<Calendar> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
    let schedule = pchart_core::schedule(&plan, &calendars, today).map_err(|err| Failure::file(format!("{}: {err}", path.display())))?;
    Ok(Context { path, plan, calendars, holidays, today, schedule })
}

fn date(text: &str, args: &Args) -> Result<Date, Failure> {
    text.parse::<Date>().map_err(|_| Failure::usage(format!("\"{text}\" is not a date like 2027-03-01"), args))
}

fn print_json(value: &serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
}

fn run(args: &Args) -> Result<u8, Failure> {
    if args.command == "help" {
        if args.flag("--json") {
            print_json(&cli::help_json());
        } else {
            print!("{}", cli::help());
        }
        return Ok(0);
    }
    let known: &[&str] = match args.command {
        "set" => &["--duration", "--on", "--no-earlier-than", "--no-later-than", "--name", "--lane", "--who"],
        "add" => &["--lane", "--name", "--duration", "--after", "--on", "--no-earlier-than", "--no-later-than", "--who"],
        "export" => &["--svg", "--png", "--pdf", "--size"],
        "bench" => &["--frames", "--out"],
        _ => &[],
    };
    if let Some(option) = args.unknown_options(known) {
        return Err(Failure::usage(format!("{} does not know {option}", args.command), args));
    }
    let ctx = context(args)?;
    let json = args.flag("--json");
    let item = |position: usize| -> Result<&str, Failure> {
        let id = args.words.get(position).map(String::as_str).ok_or_else(|| Failure::usage("which item? Name it by its id.", args))?;
        ctx.plan.item(id).map(|_| id).ok_or_else(|| Failure::edit(format!("there is no item \"{id}\"")))
    };
    let analysis = || analyse(&ctx.plan, &ctx.calendars, &ctx.schedule, ctx.today);

    match args.command {
        "situation" => {
            let analysis = analysis();
            if json {
                print_json(&report::situation_json(&ctx.plan, &ctx.schedule, &analysis));
            } else {
                print!("{}", report::situation_text(&ctx.plan, &ctx.schedule, &analysis));
            }
            Ok(0)
        }
        "why" => {
            let index = ctx.plan.item(item(1)?).unwrap_or(0);
            let chain = pchart_core::why(&ctx.schedule, index);
            if json {
                let steps: Vec<_> = chain
                    .iter()
                    .map(|&step| {
                        let slot = &ctx.schedule.slots[step];
                        json!({ "item": ctx.plan.items[step].id, "lane": ctx.plan.items[step].lane, "start": report::iso(slot.planned.start), "finish": report::iso(slot.planned.finish), "slack_wd": slot.slack })
                    })
                    .collect();
                print_json(&json!({ "schema": 1, "item": ctx.plan.items[index].id, "chain": steps }));
            } else {
                let slot = &ctx.schedule.slots[index];
                println!("{} on {} is set by this chain:", ctx.plan.items[index].name, text::day(slot.planned.finish));
                for line in text::chain_steps(&ctx.plan, &ctx.schedule, &chain) {
                    println!("  {line}");
                }
                let first = &ctx.schedule.slots[chain[0]];
                if chain.len() > 1 {
                    println!("{} to spare: {} can start {} at the earliest and {} at the latest.", text::working_days(first.slack), ctx.plan.items[chain[0]].name, text::day(first.earliest.start), text::day(first.latest.start));
                }
            }
            Ok(0)
        }
        "waits" => {
            let analysis = analysis();
            if json {
                print_json(&json!({ "schema": 1, "waits": analysis.handoffs.iter().map(|h| report::handoff(&ctx.plan, h)).collect::<Vec<_>>() }));
            } else if analysis.handoffs.is_empty() {
                println!("No lane waits on another.");
            } else {
                for handoff in &analysis.handoffs {
                    println!("{}{}", if handoff.deciding { "" } else { "  " }, text::handoff(&ctx.plan, handoff));
                }
            }
            Ok(0)
        }
        "next" => {
            let analysis = analysis();
            if json {
                print_json(&json!({ "schema": 1, "next": analysis.next.iter().map(|n| report::next(&ctx.plan, &analysis, n)).collect::<Vec<_>>() }));
            } else if analysis.next.is_empty() {
                println!("Nothing is due.");
            } else {
                for (number, next) in analysis.next.iter().enumerate() {
                    println!("{}. {}", number + 1, text::next(&ctx.plan, &analysis, next));
                }
            }
            Ok(0)
        }
        "check" => {
            let analysis = analysis();
            if json {
                print_json(&json!({ "schema": 1, "checks": analysis.checks.iter().map(|c| report::check(&ctx.plan, c)).collect::<Vec<_>>() }));
            } else {
                print!("{}", report::checks_text(&ctx.plan, &analysis));
            }
            let worst = analysis.checks.iter().map(|check| check.severity()).min();
            Ok(match worst {
                Some(Severity::Problem) => 2,
                Some(Severity::Warning) => 1,
                _ => 0,
            })
        }
        "schedule" => {
            print_schedule(&ctx.plan, &ctx.schedule, json);
            Ok(0)
        }
        "holidays" => holidays(args, &ctx, json),
        "set" | "add" | "remove" | "link" | "unlink" | "apply" => {
            let edits = edits(args, &ctx)?;
            change(args, &ctx, &edits, json)
        }
        "export" => export(args, &ctx).map(|()| 0),
        "bench" => {
            let frames = args.option("--frames").unwrap_or("40").parse().map_err(|_| Failure::usage("--frames needs a number", args))?;
            let out = args.option("--out").ok_or_else(|| Failure::usage("bench needs --out FILE", args))?;
            pchart_tui::bench(ctx.plan, ctx.calendars, ctx.today, frames, Path::new(out)).map_err(|err| Failure::file(err.to_string()))?;
            Ok(0)
        }
        _ => {
            pchart_tui::run(ctx.plan, ctx.calendars, ctx.today, &ctx.path).map_err(|err| Failure::file(err.to_string()))?;
            Ok(0)
        }
    }
}

fn holidays(args: &Args, ctx: &Context, json: bool) -> Result<u8, Failure> {
    let id = args.words.get(1).ok_or_else(|| Failure::usage("which lane? Name it by its id.", args))?;
    let lane = ctx.plan.lane(id).ok_or_else(|| Failure::edit(format!("there is no lane \"{id}\"")))?;
    let from = ctx.schedule.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(ctx.today).min(ctx.today);
    let to = ctx.schedule.slots.iter().map(|slot| slot.latest.finish.max(slot.planned.finish)).max().unwrap_or(ctx.today);
    let days: Vec<(Date, &str)> = ctx.calendars[lane].holidays(from, to).collect();
    let place = ctx.plan.lanes[lane].place.as_deref();
    if json {
        let list: Vec<_> = days.iter().map(|(day, name)| json!({ "date": report::iso(*day), "name": name, "working_day_lost": ctx.calendars[lane].between(*day, pchart_core::calendar::next(*day)) == 0 && !matches!(day.weekday(), jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday) })).collect();
        print_json(&json!({ "schema": 1, "lane": id, "place": place, "known": place.is_some_and(|place| ctx.holidays.knows(place)), "holidays": list }));
    } else if days.is_empty() {
        println!("No holidays for {} between {} and {}.", ctx.plan.lanes[lane].name, text::day(from), text::day(to));
    } else {
        for (day, name) in days {
            println!("{}  {name}", day.strftime("%a %d %b %Y"));
        }
    }
    Ok(0)
}

fn edits(args: &Args, ctx: &Context) -> Result<Vec<Edit>, Failure> {
    let word = |position: usize, what: &str| args.words.get(position).cloned().ok_or_else(|| Failure::usage(format!("which {what}?"), args));
    let days = |text: &str| text.parse::<u32>().ok().or_else(|| parse_workdays(text)).ok_or_else(|| Failure::usage(format!("\"{text}\" is not a number of working days"), args));
    let fixes = [("--on", Fix::On), ("--no-earlier-than", Fix::NoEarlierThan), ("--no-later-than", Fix::NoLaterThan)];
    Ok(match args.command {
        "set" => {
            let item = word(1, "item")?;
            let mut list = Vec::new();
            if let Some(text) = args.option("--duration") {
                list.push(Edit::Duration { item: item.clone(), days: days(text)? });
            }
            if args.flag("--calculated") {
                list.push(Edit::Unfix { item: item.clone() });
            }
            for (option, kind) in fixes {
                if let Some(text) = args.option(option) {
                    list.push(Edit::Fix { item: item.clone(), kind, date: date(text, args)? });
                }
            }
            if args.flag("--done") || args.flag("--not-done") {
                list.push(Edit::Done { item: item.clone(), done: args.flag("--done") });
            }
            if let Some(name) = args.option("--name") {
                list.push(Edit::Rename { item: item.clone(), name: name.into() });
            }
            if let Some(lane) = args.option("--lane") {
                list.push(Edit::Lane { item: item.clone(), lane: lane.into() });
            }
            if let Some(who) = args.option("--who") {
                list.push(Edit::Who { item: item.clone(), who: (!who.is_empty()).then(|| who.into()) });
            }
            if list.is_empty() {
                return Err(Failure::usage("set needs at least one change", args));
            }
            list
        }
        "add" => {
            let id = word(1, "id for the new item")?;
            let milestone = args.flag("--milestone");
            let duration = match (args.option("--duration"), milestone) {
                (Some(text), false) => days(text)?,
                (None, true) => 0,
                _ => return Err(Failure::usage("give either --duration DAYS or --milestone", args)),
            };
            let mut item = Item {
                id,
                lane: args.option("--lane").ok_or_else(|| Failure::usage("add needs --lane", args))?.into(),
                name: args.option("--name").ok_or_else(|| Failure::usage("add needs --name", args))?.into(),
                duration,
                milestone,
                after: args.option("--after").map(|list| list.split(',').map(|id| id.trim().to_string()).filter(|id| !id.is_empty()).collect()).unwrap_or_default(),
                who: args.option("--who").map(str::to_string),
                ..Item::default()
            };
            for (option, kind) in fixes {
                if let Some(text) = args.option(option) {
                    let day = Some(date(text, args)?);
                    match kind {
                        Fix::On => item.on = day,
                        Fix::NoEarlierThan => item.no_earlier_than = day,
                        Fix::NoLaterThan => item.no_later_than = day,
                    }
                }
            }
            vec![Edit::Add { item }]
        }
        "remove" => vec![Edit::Remove { item: word(1, "item")? }],
        "link" => vec![Edit::Link { from: word(1, "item to wait on")?, to: word(2, "item that waits")? }],
        "unlink" => vec![Edit::Unlink { from: word(1, "item")?, to: word(2, "item that waits")? }],
        _ => {
            let source = word(1, "file of edits")?;
            let text = if source == "-" {
                let mut text = String::new();
                std::io::stdin().read_to_string(&mut text).map_err(|err| Failure::file(format!("stdin: {err}")))?;
                text
            } else {
                std::fs::read_to_string(&source).map_err(|err| Failure::file(format!("{source}: {err}")))?
            };
            let _ = ctx;
            serde_json::from_str(&text).map_err(|err| Failure::edit(format!("the edits are not a JSON list of edits: {err}")))?
        }
    })
}

fn change(args: &Args, ctx: &Context, edits: &[Edit], json: bool) -> Result<u8, Failure> {
    let plan = pchart_core::apply_all(&ctx.plan, edits).map_err(|(index, err)| Failure::edit(if edits.len() > 1 { format!("edit {}: {err}", index + 1) } else { err.to_string() }))?;
    let schedule = pchart_core::schedule(&plan, &ctx.calendars, ctx.today).map_err(|err| Failure::edit(err.to_string()))?;
    let analysis = analyse(&plan, &ctx.calendars, &schedule, ctx.today);
    let dry = args.flag("--dry-run");
    if !dry {
        pchart_store::save(&ctx.path, &plan).map_err(|err| Failure::file(err.to_string()))?;
    }
    let moved = report::moved((&ctx.plan, &ctx.schedule), (&plan, &schedule));
    if json {
        let moved: Vec<_> = moved
            .iter()
            .map(|change| {
                let span = |span: Option<(Date, Date)>| span.map(|(start, finish)| [report::iso(start), report::iso(finish)]);
                json!({ "item": change.id, "before": span(change.before), "after": span(change.after) })
            })
            .collect();
        let mut out = report::situation_json(&plan, &schedule, &analysis);
        out["written"] = json!(!dry);
        out["moved"] = json!(moved);
        out["checks"] = json!(analysis.checks.iter().map(|c| report::check(&plan, c)).collect::<Vec<_>>());
        print_json(&out);
    } else {
        println!("{}", if dry { "Dry run, nothing written." } else { "Saved." });
        let span = |span: Option<(Date, Date)>| span.map_or("—".to_string(), |(start, finish)| if start == finish { text::day(start) } else { format!("{} – {}", text::day(start), text::day(finish)) });
        if moved.is_empty() {
            println!("No date moved.");
        }
        for change in &moved {
            let name = plan.item(&change.id).map_or_else(|| ctx.plan.items[ctx.plan.item(&change.id).unwrap_or(0)].name.clone(), |index| plan.items[index].name.clone());
            println!("  {name}: {} → {}", span(change.before), span(change.after));
        }
        if let Some(verdict) = &analysis.verdict {
            let (headline, detail) = text::verdict(&plan, verdict);
            println!("{headline} {detail}");
        }
        for check in analysis.checks.iter().filter(|check| check.severity() != Severity::Hint && !matches!(check, pchart_core::Check::TooLate { .. })) {
            println!("{} {}", if check.severity() == Severity::Problem { "✖" } else { "▲" }, text::check(&plan, check).0);
        }
    }
    Ok(0)
}

fn print_schedule(plan: &Plan, schedule: &Schedule, json: bool) {
    if json {
        let items: Vec<serde_json::Value> = plan
            .items
            .iter()
            .zip(&schedule.slots)
            .map(|(item, slot)| {
                json!({
                    "id": item.id,
                    "start": report::iso(slot.planned.start),
                    "finish": report::iso(slot.planned.finish),
                    "earliest": [report::iso(slot.earliest.start), report::iso(slot.earliest.finish)],
                    "latest": [report::iso(slot.latest.start), report::iso(slot.latest.finish)],
                    "slack_wd": slot.slack,
                })
            })
            .collect();
        print_json(&json!({
            "schema": 1,
            "start": report::iso(schedule.start),
            "target": schedule.target.map(|index| plan.items[index].id.clone()),
            "spare_wd": schedule.spare(),
            "chain": schedule.chain.iter().map(|&index| plan.items[index].id.as_str()).collect::<Vec<_>>(),
            "items": items,
        }));
        return;
    }
    for &index in &schedule.order {
        let (item, slot) = (&plan.items[index], &schedule.slots[index]);
        let mark = if schedule.chain.contains(&index) { '*' } else { ' ' };
        println!(
            "{mark} {:<16} {:<11} {} – {}   latest {} – {}   slack {:>3} wd",
            item.name,
            item.lane,
            text::day(slot.planned.start),
            text::day(slot.planned.finish),
            text::day(slot.latest.start),
            text::day(slot.latest.finish),
            slot.slack
        );
    }
    if let (Some(target), Some(spare)) = (schedule.target, schedule.spare()) {
        println!("\n{}: {} to spare, plan starts {}", plan.items[target].name, text::working_days(spare), text::day(schedule.start));
    }
}

fn export(args: &Args, ctx: &Context) -> Result<(), Failure> {
    let (width, height) = match args.option("--size") {
        Some(size) => size.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))).ok_or_else(|| Failure::usage(format!("\"{size}\" is not a size like 1220x756"), args))?,
        None => (1220.0, 756.0),
    };
    let settings = Settings { width, height, unit: 1.0, today: ctx.today, selected: None, palette: Palette::labra() };
    let svg = pchart_draw::roadmap(&ctx.plan, &ctx.schedule, &ctx.calendars, &settings).svg();
    let output = Output::default();
    let tree = output.tree(&svg).map_err(Failure::file)?;
    let write = |path: &str, bytes: &[u8]| std::fs::write(path, bytes).map_err(|err| Failure::file(format!("{path}: {err}")));
    if let Some(path) = args.option("--svg") {
        write(path, svg.as_bytes())?;
    }
    if let Some(path) = args.option("--png") {
        write(path, &output.png(&tree, 1.0).map_err(Failure::file)?)?;
    }
    if let Some(path) = args.option("--pdf") {
        write(path, &output.pdf(&tree).map_err(Failure::file)?)?;
    }
    Ok(())
}
