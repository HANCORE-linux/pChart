mod cli;
mod report;

use std::io::{IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use jiff::civil::Date;
use pchart_core::analysis::Severity;
use pchart_core::edit::{Edit, Fix};
use pchart_core::model::{Item, parse_workdays};
use pchart_core::{Calendar, Holidays, Plan, Schedule, analyse, text};
use pchart_draw::{Audience, Level, Output, Palette, Settings, View};
use serde_json::json;

use crate::cli::Args;

/// `println!` that ends quietly when the reader has gone, as in `pchart schedule PLAN | head`.
macro_rules! say {
    ($($arg:tt)*) => {
        emit(format_args!("{}\n", format_args!($($arg)*)))
    };
}

/// Holidays of every country and subdivision python-holidays knows, 2006–2051 (see `data/README.md`).
const HOLIDAYS: &[u8] = include_bytes!("../../../data/holidays.tsv.gz");

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

    /// A plan with work whose length is not known yet: it has no dates.
    fn undated(plan: &Plan, what: &str) -> Failure {
        let work = plan.items.iter().filter(|item| !item.milestone).count();
        Failure { code: 3, kind: "dates", message: format!("{what} needs dates, and {} of {work} pieces of work have no length yet", plan.without_length().len()), hint: Some("pchart situation PLAN names them; pchart set PLAN ID --duration DAYS gives one its length".into()) }
    }

    fn edit(message: impl Into<String>) -> Failure {
        Failure { code: 3, kind: "edit", message: message.into(), hint: Some("pchart help --json lists every command and edit".into()) }
    }
}

/// What a plan without dates says about itself: how much work has no length yet, and which.
fn undated(plan: &Plan) -> Vec<String> {
    let missing = plan.without_length();
    let work = plan.items.iter().filter(|item| !item.milestone).count();
    let names: Vec<&str> = missing.iter().map(|&index| plan.items[index].name.as_str()).collect();
    vec![format!("{} has no dates yet: {} of {work} pieces of work have no length.", plan.name, missing.len()), format!("Without a length: {}.", names.join(", "))]
}

fn main() -> ExitCode {
    let words: Vec<String> = std::env::args().skip(1).collect();
    let words = if words.is_empty() && !std::io::stdout().is_terminal() { vec!["help".into()] } else { words };
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
    path: PathBuf,
    plan: Plan,
    calendars: Vec<Calendar>,
    holidays: Holidays,
    today: Date,
    schedule: Schedule,
}

fn context(args: &Args) -> Result<Context, Failure> {
    let path = PathBuf::from(args.words.first().ok_or_else(|| Failure::usage("which plan? Name the plan file.", args))?);
    context_of(path, args)
}

fn context_of(path: PathBuf, args: &Args) -> Result<Context, Failure> {
    let plan = pchart_store::load(&path).map_err(|err| Failure::file(err.to_string()))?;
    let today = today(args)?;
    let holidays = holidays_for(&holiday_text(args)?, &plan);
    if !args.flag("--json") {
        for lane in &plan.lanes {
            if let Some(place) = &lane.place
                && !holidays.knows(place)
            {
                eprintln!("No holiday data for {place} (lane {}); counting weekends only.", lane.name);
            }
        }
    }
    let calendars = calendars(&holidays, &plan);
    let schedule = pchart_core::schedule(&plan, &calendars, today).map_err(|err| Failure::file(format!("{}: {err}", path.display())))?;
    Ok(Context { path, plan, calendars, holidays, today, schedule })
}

/// The analysis of `plan` as the plan of `ctx`, with the checks of people across the projects of its
/// workspace when it is in one (pictures 64).
fn analysis_of(ctx: &Context, plan: &Plan, schedule: &Schedule, args: &Args) -> Result<pchart_core::Analysis, Failure> {
    if pchart_store::workspace(ctx.path.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new("."))).is_none() {
        return Ok(analyse(plan, &ctx.calendars, schedule, ctx.today));
    }
    let text = holiday_text(args)?;
    let lanes = |plan: &Plan| calendars(&holidays_for(&text, plan), plan);
    Ok(match pchart_store::crew(&ctx.path, &ctx.plan, &ctx.schedule, ctx.today, &lanes) {
        Some(mut crew) => crew.analyse(plan, &ctx.calendars, schedule, ctx.today),
        None => analyse(plan, &ctx.calendars, schedule, ctx.today),
    })
}

fn today(args: &Args) -> Result<Date, Failure> {
    match args.option("--today") {
        Some(text) => date(text, args),
        None => Ok(jiff::Zoned::now().date()),
    }
}

/// The holiday data: the file `--holidays` names, else the bundled data.
fn holiday_text(args: &Args) -> Result<String, Failure> {
    match args.option("--holidays") {
        Some(file) => std::fs::read_to_string(file).map_err(|err| Failure::file(format!("{file}: {err}"))),
        None => {
            let mut text = String::new();
            flate2::read::GzDecoder::new(HOLIDAYS).read_to_string(&mut text).expect("the bundled holiday data is gzip text");
            Ok(text)
        }
    }
}

/// The holidays of the places of the plan's lanes.
fn holidays_for(text: &str, plan: &Plan) -> Holidays {
    let places: Vec<&str> = plan.lanes.iter().filter_map(|lane| lane.place.as_deref()).collect();
    Holidays::parse_for(text, |region| places.iter().any(|place| *place == region || place.split_once('-').is_some_and(|(country, _)| country == region)))
}

fn calendars(holidays: &Holidays, plan: &Plan) -> Vec<Calendar> {
    plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect()
}

fn date(text: &str, args: &Args) -> Result<Date, Failure> {
    text.parse::<Date>().map_err(|_| Failure::usage(format!("\"{text}\" is not a date like 2027-03-01"), args))
}

fn emit(text: std::fmt::Arguments) {
    if let Err(err) = std::io::stdout().write_fmt(text) {
        if err.kind() == std::io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        eprintln!("stdout: {err}");
        std::process::exit(4);
    }
}

fn print_json(value: &serde_json::Value) {
    say!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
}

fn run(args: &Args) -> Result<u8, Failure> {
    if args.command == "help" {
        if args.flag("--json") {
            print_json(&cli::help_json());
        } else {
            emit(format_args!("{}", cli::help()));
        }
        return Ok(0);
    }
    let known: &[&str] = match args.command {
        "set" => &["--duration", "--on", "--no-earlier-than", "--no-later-than", "--name", "--lane", "--who", "--share", "--note"],
        "add" => &["--lane", "--name", "--duration", "--after", "--on", "--no-earlier-than", "--no-later-than", "--who"],
        "export" => &["--svg", "--png", "--pdf", "--size", "--level", "--view", "--style", "--md", "--dot", "--audience"],
        "bench" => &["--frames", "--out"],
        "open" => &["--theme"],
        "new" => &["--template", "--name", "--target", "--start"],
        "read" => &["--out", "--name"],
        _ => &[],
    };
    if let Some(option) = args.unknown_options(known) {
        return Err(Failure::usage(format!("{} does not know {option}", args.command), args));
    }
    if args.command == "new" {
        return new(args);
    }
    if args.command == "read" {
        return read(args);
    }
    if args.command == "list" {
        return list(args);
    }
    if args.command == "open" && args.words.is_empty() && std::io::stdout().is_terminal() {
        return start(args);
    }
    let ctx = context(args)?;
    let json = args.flag("--json");
    let item = |position: usize| -> Result<&str, Failure> {
        let id = args.words.get(position).map(String::as_str).ok_or_else(|| Failure::usage("which item? Name it by its id.", args))?;
        ctx.plan.item(id).map(|_| id).ok_or_else(|| Failure::edit(format!("there is no item \"{id}\"")))
    };
    let analysis = || analysis_of(&ctx, &ctx.plan, &ctx.schedule, args);
    let dated = ctx.plan.without_length().is_empty();
    if !dated && matches!(args.command, "situation" | "next" | "check") {
        if json {
            print_json(&json!({ "schema": 1, "dates": false, "without_length": ctx.plan.without_length().iter().map(|&index| &ctx.plan.items[index].id).collect::<Vec<_>>() }));
        } else {
            for line in undated(&ctx.plan) {
                say!("{line}");
            }
        }
        return Ok(0);
    }
    if !dated && matches!(args.command, "why" | "schedule" | "holidays") {
        return Err(Failure::undated(&ctx.plan, &format!("pchart {}", args.command)));
    }

    match args.command {
        "situation" => {
            let analysis = analysis()?;
            if json {
                print_json(&report::situation_json(&ctx.plan, &ctx.schedule, &analysis));
            } else {
                emit(format_args!("{}", report::situation_text(&ctx.plan, &ctx.schedule, &analysis)));
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
                        json!({ "item": ctx.plan.items[step].id, "lane": ctx.plan.items[step].lane, "start": report::iso(slot.planned.start), "finish": report::iso(slot.planned.finish), "spare_wd": slot.spare })
                    })
                    .collect();
                print_json(&json!({ "schema": 1, "item": ctx.plan.items[index].id, "spare_wd": ctx.schedule.slots[index].spare, "chain": steps }));
            } else {
                let slot = &ctx.schedule.slots[index];
                say!("{} on {} is set by this chain:", ctx.plan.items[index].name, text::day(slot.planned.finish));
                for line in text::chain_steps(&ctx.plan, &ctx.schedule, &chain) {
                    say!("  {line}");
                }
                let name = &ctx.plan.items[index].name;
                if slot.spare < 0 {
                    say!("{name} is {} late.", text::working_days(-slot.spare));
                } else {
                    say!("{name} has {} to spare.", text::working_days(slot.spare));
                }
                if chain.len() > 1 {
                    let first = &ctx.schedule.slots[chain[0]];
                    say!("{} can start {} at the earliest and {} at the latest.", ctx.plan.items[chain[0]].name, text::day(first.earliest.start), text::day(first.latest.start));
                }
            }
            Ok(0)
        }
        "waits" => {
            let analysis = analysis()?;
            if json {
                print_json(&json!({ "schema": 1, "waits": analysis.handoffs.iter().map(|h| report::handoff(&ctx.plan, h)).collect::<Vec<_>>() }));
            } else if analysis.handoffs.is_empty() {
                say!("No lane waits on another.");
            } else {
                for handoff in &analysis.handoffs {
                    say!("{}{}", if handoff.deciding { "" } else { "  " }, text::handoff(&ctx.plan, handoff));
                }
            }
            Ok(0)
        }
        "next" => {
            let analysis = analysis()?;
            if json {
                print_json(&json!({ "schema": 1, "next": analysis.next.iter().map(|n| report::next(&ctx.plan, &analysis, n)).collect::<Vec<_>>() }));
            } else if analysis.next.is_empty() {
                say!("Nothing is due.");
            } else {
                for (number, next) in analysis.next.iter().enumerate() {
                    say!("{}. {}", number + 1, text::next(&ctx.plan, &analysis, next));
                }
            }
            Ok(0)
        }
        "check" => {
            let analysis = analysis()?;
            if json {
                print_json(&json!({ "schema": 1, "checks": analysis.checks.iter().map(|c| report::check(&ctx.plan, c)).collect::<Vec<_>>() }));
            } else {
                emit(format_args!("{}", report::checks_text(&ctx.plan, &analysis)));
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
        _ => open(ctx, args),
    }
}

/// Opens the plan in the terminal and puts it first among the plans opened last.
fn open(ctx: Context, args: &Args) -> Result<u8, Failure> {
    let theme = pchart_tui::Theme::pick(args.option("--theme")).map_err(|err| Failure::usage(err, args))?;
    pchart_store::remember(&ctx.path);
    let text = holiday_text(args)?;
    let lanes = |plan: &Plan| calendars(&holidays_for(&text, plan), plan);
    pchart_tui::run(ctx.plan, ctx.calendars, ctx.today, &ctx.path, theme, &lanes).map_err(|err| Failure::file(err.to_string()))?;
    Ok(0)
}

/// `pchart` without a plan, in a terminal: the start, which writes a new plan or picks one opened before,
/// and then opens it.
fn start(args: &Args) -> Result<u8, Failure> {
    let theme = pchart_tui::Theme::pick(args.option("--theme")).map_err(|err| Failure::usage(err, args))?;
    let text = holiday_text(args)?;
    let lanes = |plan: &Plan| calendars(&holidays_for(&text, plan), plan);
    let dir = std::env::current_dir().map_err(|err| Failure::file(err.to_string()))?;
    match pchart_tui::start(&dir, today(args)?, theme, &lanes).map_err(|err| Failure::file(err.to_string()))? {
        Some(path) => open(context_of(path, args)?, args),
        None => Ok(0),
    }
}

/// `pchart list`: the projects of the workspace pChart runs in, each with how it stands.
fn list(args: &Args) -> Result<u8, Failure> {
    let dir = std::env::current_dir().map_err(|err| Failure::file(err.to_string()))?;
    let (root, name) = pchart_store::workspace(&dir).ok_or_else(|| Failure::usage("not in a workspace: a folder with a workspace.toml, or one inside it", args))?;
    let text = holiday_text(args)?;
    let today = today(args)?;
    let mut rows = Vec::new();
    for path in pchart_store::projects(&root) {
        let plan = pchart_store::load(&path).map_err(|err| Failure::file(err.to_string()))?;
        let calendars = calendars(&holidays_for(&text, &plan), &plan);
        let schedule = pchart_core::schedule(&plan, &calendars, today).map_err(|err| Failure::edit(err.to_string()))?;
        let verdict = analyse(&plan, &calendars, &schedule, today).verdict;
        rows.push((path, plan, verdict, calendars, schedule));
    }
    let people = pchart_store::people(&root);
    let projects: Vec<pchart_core::people::Project> = rows.iter().map(|(_, plan, _, calendars, schedule)| pchart_core::people::Project { name: plan.name.clone(), plan: plan.clone(), schedule: schedule.clone(), calendars: calendars.clone() }).collect();
    let mut found: Vec<(String, String, jiff::civil::Date, jiff::civil::Date, String)> = Vec::new();
    for (here, (_, plan, _, calendars, _)) in rows.iter().enumerate() {
        for check in pchart_core::people::checks(&people, &projects, here, calendars) {
            match check {
                pchart_core::Check::Overload { who, item, from, to, load, capacity, with, .. } => {
                    if !found.iter().any(|seen| seen.0 == who && seen.1 == "overload" && seen.2 <= to && from <= seen.3) {
                        let work: Vec<String> = std::iter::once(format!("{} in {}", plan.items[item].name, plan.name)).chain(with.into_iter().map(|other| other.replace(" here", &format!(" in {}", plan.name)))).collect();
                        found.push((who, "overload".into(), from, to, format!("{load} % of {capacity} %: {}", work.join(", "))));
                    }
                }
                pchart_core::Check::Away { who, item, from, to, days } => found.push((who, "away".into(), from, to, format!("{} of {} in {}", text::working_days(days), plan.items[item].name, plan.name))),
                _ => {}
            }
        }
    }
    if args.flag("--json") {
        let projects: Vec<_> = rows
            .iter()
            .map(|(path, plan, verdict, _, _)| {
                let state = verdict.as_ref().map(|verdict| pchart_draw::situation::state_words(verdict.state));
                json!({ "file": path.strip_prefix(&root).unwrap_or(path).display().to_string(), "name": plan.name, "state": state, "target": verdict.as_ref().map(|verdict| plan.items[verdict.target].id.clone()), "date": verdict.as_ref().map(|verdict| report::iso(verdict.date)), "spare_wd": verdict.as_ref().map(|verdict| verdict.spare), "dates": plan.without_length().is_empty() })
            })
            .collect();
        let people: Vec<_> = found.iter().map(|(who, kind, from, to, what)| json!({ "who": who, "kind": kind, "from": report::iso(*from), "to": report::iso(*to), "what": what })).collect();
        print_json(&json!({ "schema": 1, "workspace": name, "projects": projects, "people": people }));
        return Ok(0);
    }
    say!("{name}: {}", if rows.len() == 1 { "1 project".to_string() } else { format!("{} projects", rows.len()) });
    for (path, plan, verdict, _, _) in &rows {
        let file = path.file_name().map_or_else(String::new, |file| file.to_string_lossy().into_owned());
        let stands = match verdict {
            Some(verdict) if verdict.spare < 0 => format!("{} on {} is {} late", plan.items[verdict.target].name, text::day(verdict.date), text::working_days(-verdict.spare)),
            Some(verdict) => format!("{} on {} is {}, {} to spare", plan.items[verdict.target].name, text::day(verdict.date), pchart_draw::situation::state_words(verdict.state), text::working_days(verdict.spare)),
            None if !plan.without_length().is_empty() => "no dates yet".into(),
            None => "no fixed target date".into(),
        };
        say!("  {:<28} {stands}  ({file})", plan.name);
    }
    if !found.is_empty() {
        say!("People to look at:");
        for (who, kind, from, to, what) in &found {
            let words = if kind == "away" { format!("away {}: {what}", text::span(*from, *to)) } else { format!("over capacity {}, {what}", text::span(*from, *to)) };
            say!("  {who:<28} {words}");
        }
    }
    Ok(0)
}

/// `pchart new`: a plan from a template, written where no file is.
/// `pchart read DIR`: how a folder is built, as a plan without dates next to where pChart runs.
fn read(args: &Args) -> Result<u8, Failure> {
    let dir = PathBuf::from(args.words.first().ok_or_else(|| Failure::usage("which folder? Name it.", args))?);
    let folder = std::fs::canonicalize(&dir).ok().and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned())).unwrap_or_else(|| "folder".into());
    let name = args.option("--name").map_or(folder.clone(), str::to_string);
    let shown = dir.display().to_string();
    let shown = match std::env::var("HOME").ok().filter(|home| !home.is_empty() && shown.starts_with(home.as_str())) {
        Some(home) => format!("~{}", &shown[home.len()..]),
        None => shown,
    };
    let terminal = std::io::stderr().is_terminal() && !args.flag("--json");
    let started = std::time::Instant::now();
    let mut drawn = started;
    let mut frame = 0;
    let mut line = |text: String| {
        let now = std::time::Instant::now();
        if terminal && now.duration_since(started).as_millis() >= 150 && now.duration_since(drawn).as_millis() >= 80 {
            drawn = now;
            frame += 1;
            eprint!("\r\x1b[2K{} {text}", ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"][frame % 10]);
        }
    };
    let reading_from = std::cell::Cell::new(None::<std::time::Instant>);
    let line = std::cell::RefCell::new(&mut line);
    let read = pchart_store::read(
        &dir,
        &name,
        &mut |seen| (line.borrow_mut())(format!("Counting files in {shown} · {seen} so far")),
        &mut |progress| {
            let from = *reading_from.get().get_or_insert_with(std::time::Instant::now);
            reading_from.set(Some(from));
            let spent = from.elapsed().as_secs_f64();
            let left = if progress.bytes == 0 { 0.0 } else { spent * (progress.of_bytes - progress.bytes) as f64 / progress.bytes as f64 };
            let when = if left < 1.0 { "less than a second left".to_string() } else { format!("about {} s left", left.round()) };
            (line.borrow_mut())(format!("Reading {shown} · {} of {} files · {when}", progress.files, progress.of_files));
        },
    );
    if terminal && started.elapsed().as_millis() >= 150 {
        eprint!("\r\x1b[2K");
    }
    let read = read.map_err(|err| Failure { code: 3, kind: "usage", message: err.to_string(), hint: None })?;
    let path = PathBuf::from(args.option("--out").map_or_else(|| format!("{}.pchart", pchart_core::model::slug(&folder).unwrap_or_else(|| "folder".into())), str::to_string));
    let dry = args.flag("--dry-run");
    if !dry {
        pchart_store::create(&path, &read.plan).map_err(|err| Failure::file(err.to_string()))?;
    }
    let waits: usize = read.plan.items.iter().map(|item| item.after.len()).sum();
    if args.flag("--json") {
        print_json(&json!({ "schema": 1, "written": !dry, "file": path.display().to_string(), "items": read.plan.items.len(), "lanes": read.plan.lanes.len(), "waits": waits, "kinds": read.kinds, "left_out": read.circles.iter().map(|(from, to)| json!({ "item": from, "uses": to })).collect::<Vec<_>>() }));
        return Ok(0);
    }
    let kinds: Vec<String> = read.kinds.iter().map(|(kind, count)| format!("{count} {kind}")).collect();
    let lanes = match read.plan.lanes.len() {
        1 => "1 lane".to_string(),
        count => format!("{count} lanes"),
    };
    say!("Read {} files ({}) in {lanes}: {} of them use others, {} in all.", read.plan.items.len(), kinds.join(", "), read.plan.items.iter().filter(|item| !item.after.is_empty()).count(), if waits == 1 { "1 use".to_string() } else { format!("{waits} uses") });
    if !read.circles.is_empty() {
        say!("{} uses were left out, since a plan cannot wait on itself in a circle:", read.circles.len());
        for (from, to) in read.circles.iter().take(5) {
            say!("  {from} uses {to}");
        }
        if read.circles.len() > 5 {
            say!("  and {} more; --json lists them all", read.circles.len() - 5);
        }
    }
    say!("{} {}. Open it with: pchart {}", if dry { "Dry run, nothing written:" } else { "Wrote" }, path.display(), path.display());
    Ok(0)
}

fn new(args: &Args) -> Result<u8, Failure> {
    let path = PathBuf::from(args.words.first().ok_or_else(|| Failure::usage("where? Name the new plan file.", args))?);
    let templates = pchart_store::templates();
    let ids = templates.iter().map(|(id, _)| *id).collect::<Vec<_>>().join(", ");
    let id = args.option("--template").ok_or_else(|| Failure::usage(format!("which template? {ids}"), args))?;
    let (_, template) = templates.iter().find(|(other, _)| *other == id).ok_or_else(|| Failure::usage(format!("there is no template \"{id}\"; there are {ids}"), args))?;
    let date = match (args.option("--target"), args.option("--start")) {
        (Some(_), Some(_)) => return Err(Failure::usage("give either --target or --start", args)),
        (Some(text), None) | (None, Some(text)) => Some(date(text, args)?),
        (None, None) => None,
    };
    if args.flag("--no-dates") && date.is_some() {
        return Err(Failure::usage("a plan without dates has no --target or --start", args));
    }
    let plan = template.new_plan(args.option("--name").unwrap_or(&template.name), date, args.option("--start").is_some());
    let plan = if args.flag("--no-dates") { plan.structure_only() } else { plan };
    let today = today(args)?;
    let calendars = calendars(&holidays_for(&holiday_text(args)?, &plan), &plan);
    let schedule = pchart_core::schedule(&plan, &calendars, today).map_err(|err| Failure::edit(err.to_string()))?;
    let analysis = analyse(&plan, &calendars, &schedule, today);
    let dry = args.flag("--dry-run");
    if path.exists() {
        return Err(Failure::file(format!("{}: is there already", path.display())));
    }
    if !dry {
        pchart_store::create(&path, &plan).map_err(|err| Failure::file(err.to_string()))?;
    }
    if args.flag("--json") {
        let mut out = report::situation_json(&plan, &schedule, &analysis);
        out["written"] = json!(!dry);
        print_json(&out);
        return Ok(0);
    }
    say!("{}", if dry { "Dry run, nothing written." } else { "Saved." });
    if !plan.without_length().is_empty() {
        say!("{}", undated(&plan)[0]);
        return Ok(0);
    }
    match &analysis.verdict {
        Some(verdict) => {
            let (headline, detail) = text::verdict(&plan, verdict);
            say!("{headline} {detail}");
        }
        None => {
            if let Some(last) = (0..plan.items.len()).max_by_key(|&index| schedule.slots[index].planned.finish) {
                say!("{} on {}.", plan.items[last].name, text::day(schedule.slots[last].planned.finish));
            }
        }
    }
    Ok(0)
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
        say!("No holidays for {} between {} and {}.", ctx.plan.lanes[lane].name, text::day(from), text::day(to));
    } else {
        for (day, name) in days {
            say!("{}  {name}", day.strftime("%a %d %b %Y"));
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
            if args.flag("--buffer") || args.flag("--no-buffer") {
                list.push(Edit::Buffer { item: item.clone(), buffer: args.flag("--buffer") });
            }
            if args.flag("--important") || args.flag("--not-important") {
                list.push(Edit::Important { item: item.clone(), important: args.flag("--important") });
            }
            if let Some(note) = args.option("--note") {
                list.push(Edit::Note { item: item.clone(), note: Some(note.into()) });
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
            if let Some(share) = args.option("--share") {
                let share = share.trim().trim_end_matches('%').trim().parse::<u32>().map_err(|_| Failure::usage(format!("\"{share}\" is not a share like 30 or 30%"), args))?;
                list.push(Edit::Share { item: item.clone(), share: Some(share) });
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
                (None, _) => 0,
                (Some(_), true) => return Err(Failure::usage("a milestone has no --duration", args)),
            };
            let mut item = Item {
                id,
                lane: args.option("--lane").ok_or_else(|| Failure::usage("add needs --lane", args))?.into(),
                name: args.option("--name").ok_or_else(|| Failure::usage("add needs --name", args))?.into(),
                duration,
                milestone,
                buffer: args.flag("--buffer"),
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
    let analysis = analysis_of(ctx, &plan, &schedule, args)?;
    let dry = args.flag("--dry-run");
    if !dry {
        pchart_store::save(&ctx.path, &plan).map_err(|err| Failure::file(err.to_string()))?;
    }
    if !plan.without_length().is_empty() || !ctx.plan.without_length().is_empty() {
        if json {
            print_json(&json!({ "schema": 1, "written": !dry, "dates": plan.without_length().is_empty(), "without_length": plan.without_length().iter().map(|&index| &plan.items[index].id).collect::<Vec<_>>() }));
            return Ok(0);
        }
        say!("{}", if dry { "Dry run, nothing written." } else { "Saved." });
        match analysis.verdict.as_ref().filter(|_| plan.without_length().is_empty()) {
            Some(verdict) => {
                let (headline, detail) = text::verdict(&plan, verdict);
                say!("Every piece of work has a length now. {headline} {detail}");
            }
            None if plan.without_length().is_empty() => say!("Every piece of work has a length now."),
            None => say!("{}", undated(&plan)[0]),
        }
        return Ok(0);
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
        say!("{}", if dry { "Dry run, nothing written." } else { "Saved." });
        let span = |span: Option<(Date, Date)>| span.map_or("—".to_string(), |(start, finish)| if start == finish { text::day(start) } else { format!("{} – {}", text::day(start), text::day(finish)) });
        if moved.is_empty() {
            say!("No date moved.");
        }
        for change in &moved {
            let name = plan.item(&change.id).map_or_else(|| ctx.plan.items[ctx.plan.item(&change.id).unwrap_or(0)].name.clone(), |index| plan.items[index].name.clone());
            say!("  {name}: {} → {}", span(change.before), span(change.after));
        }
        if let Some(verdict) = &analysis.verdict {
            let (headline, detail) = text::verdict(&plan, verdict);
            say!("{headline} {detail}");
        }
        for check in analysis.checks.iter().filter(|check| check.severity() != Severity::Hint && !matches!(check, pchart_core::Check::TooLate { .. })) {
            say!("{} {}", if check.severity() == Severity::Problem { "✖" } else { "▲" }, text::check(&plan, check).0);
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
                    "spare_wd": slot.spare,
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
        say!(
            "{mark} {:<16} {:<11} {} – {}   latest {} – {}   to spare {:>3} wd",
            item.name,
            item.lane,
            text::day(slot.planned.start),
            text::day(slot.planned.finish),
            text::day(slot.latest.start),
            text::day(slot.latest.finish),
            slot.spare
        );
    }
    if let (Some(target), Some(spare)) = (schedule.target, schedule.spare()) {
        say!("\n{}: {} to spare, plan starts {}", plan.items[target].name, text::working_days(spare), text::day(schedule.start));
    }
}

fn export(args: &Args, ctx: &Context) -> Result<(), Failure> {
    if ["--svg", "--png", "--pdf", "--md", "--dot"].iter().all(|option| args.option(option).is_none()) {
        return Err(Failure::usage("export needs at least one of --svg, --png, --pdf, --md and --dot", args));
    }
    let audience = match args.option("--audience") {
        Some(id) => Audience::from_id(id).ok_or_else(|| Failure::usage(format!("\"{id}\" is not an audience; use team, management, client or agents"), args))?,
        None => Audience::Team,
    };
    let view = args.option("--view").unwrap_or("roadmap");
    if !ctx.plan.without_length().is_empty() {
        let pictures = ["--svg", "--png", "--pdf"].iter().any(|option| args.option(option).is_some());
        if pictures && View::from_id(view).is_none_or(View::dated) {
            return Err(Failure::undated(&ctx.plan, &format!("the {view} view")));
        }
        if args.option("--md").is_some() {
            return Err(Failure::undated(&ctx.plan, "Markdown"));
        }
    }
    let light = match args.option("--style").unwrap_or(if view == "roadmap" { "dark" } else { "light" }) {
        "light" => true,
        "dark" if view == "roadmap" => false,
        "dark" => return Err(Failure::usage(format!("the {view} view is a light page; only the roadmap has a dark style"), args)),
        other => return Err(Failure::usage(format!("\"{other}\" is not a style; use light or dark"), args)),
    };
    if light && let Some(option) = ["--size", "--level"].into_iter().find(|option| args.option(option).is_some()) {
        return Err(Failure::usage(format!("{option} is for the dark roadmap; light views are 16:9 pages"), args));
    }
    let checks = || analyse(&ctx.plan, &ctx.calendars, &ctx.schedule, ctx.today).checks;
    let scene = match view {
        "roadmap" if light => View::Roadmap.page(&ctx.plan, &ctx.schedule, ctx.today, &checks(), audience),
        "roadmap" => {
            let size = match args.option("--size") {
                Some(size) => Some(size.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))).ok_or_else(|| Failure::usage(format!("\"{size}\" is not a size like 1220x756"), args))?),
                None => None,
            };
            let level = match args.option("--level").unwrap_or("full") {
                "overview" => Level::Overview,
                "standard" => Level::Standard,
                "full" => Level::Full,
                other => return Err(Failure::usage(format!("\"{other}\" is not a detail level; use overview, standard or full"), args)),
            };
            let mut settings = Settings { width: size.map_or(1220.0, |(width, _)| width), height: 0.0, unit: 1.0, today: ctx.today, selected: None, level, palette: Palette::labra(), cursor: None };
            settings.height = size.map_or_else(|| pchart_draw::natural_height(&ctx.plan, &ctx.schedule, &settings), |(_, height)| height);
            pchart_draw::roadmap(&ctx.plan, &ctx.schedule, &ctx.calendars, &settings)
        }
        other => match View::from_id(other) {
            Some(view) => view.page(&ctx.plan, &ctx.schedule, ctx.today, &checks(), audience),
            None => return Err(Failure::usage(format!("\"{other}\" is not a view; use roadmap, milestones, gantt, phases, now-next-later, structure, network or matrix"), args)),
        },
    };
    let svg = scene.svg();
    let write = |path: &Path, bytes: &[u8]| std::fs::write(path, bytes).map_err(|err| Failure::file(format!("{}: {err}", path.display())));
    if args.option("--svg").is_some() || args.option("--png").is_some() || args.option("--pdf").is_some() {
        let output = Output::default();
        let tree = output.tree(&svg).map_err(Failure::file)?;
        if let Some(path) = args.option("--svg") {
            write(Path::new(path), svg.as_bytes())?;
        }
        if let Some(path) = args.option("--png") {
            write(Path::new(path), &output.png(&tree, 1.0).map_err(Failure::file)?)?;
        }
        if let Some(path) = args.option("--pdf") {
            write(Path::new(path), &output.pdf(&tree).map_err(Failure::file)?)?;
        }
    }
    if let Some(path) = args.option("--dot") {
        write(Path::new(path), pchart_draw::dot(&ctx.plan, &ctx.schedule).as_bytes())?;
    }
    if let Some(path) = args.option("--md") {
        let stem = ctx.path.file_stem().map_or("plan".into(), |stem| stem.to_string_lossy());
        let picture = format!("{stem}-milestones.svg");
        write(&Path::new(path).with_file_name(&picture), pchart_draw::milestones(&ctx.plan, &ctx.schedule, ctx.today, audience).svg().as_bytes())?;
        let analysis = analyse(&ctx.plan, &ctx.calendars, &ctx.schedule, ctx.today);
        let source = ctx.path.file_name().map_or("plan".into(), |name| name.to_string_lossy());
        write(Path::new(path), pchart_draw::markdown(&ctx.plan, &ctx.schedule, &analysis, &source, ctx.today, &picture, audience).as_bytes())?;
    }
    Ok(())
}
