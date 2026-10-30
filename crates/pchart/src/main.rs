use std::path::{Path, PathBuf};
use std::process::ExitCode;

use jiff::civil::Date;
use pchart_core::{Calendar, Holidays, Plan, Schedule};
use pchart_draw::{Output, Palette, Settings};

const SAMPLE_HOLIDAYS: &str = include_str!("../../../samples/holidays.tsv");

const USAGE: &str = "usage:
  pchart PLAN                      open the plan
  pchart schedule PLAN [--json]    print the calculated dates
  pchart export PLAN [--svg FILE] [--png FILE] [--pdf FILE] [--size WxH]
  pchart bench PLAN --frames N --out FILE

options for every command:
  --today YYYY-MM-DD    the day the plan is calculated from (default: today)
  --holidays FILE       holiday data, lines of REGION<TAB>DATE<TAB>NAME";

struct Args {
    command: String,
    plan: PathBuf,
    options: Vec<(String, String)>,
    flags: Vec<String>,
}

impl Args {
    fn parse() -> Result<Args, String> {
        let mut words: Vec<String> = std::env::args().skip(1).collect();
        if words.is_empty() || words.iter().any(|word| word == "-h" || word == "--help") {
            return Err(USAGE.into());
        }
        let command = if ["schedule", "export", "bench"].contains(&words[0].as_str()) { words.remove(0) } else { "open".into() };
        let (mut plan, mut options, mut flags) = (None, Vec::new(), Vec::new());
        let mut rest = words.into_iter();
        while let Some(word) = rest.next() {
            match word.as_str() {
                "--json" => flags.push(word),
                option if option.starts_with("--") => {
                    let value = rest.next().ok_or(format!("{option} needs a value\n\n{USAGE}"))?;
                    options.push((option.to_string(), value));
                }
                _ if plan.is_none() => plan = Some(PathBuf::from(word)),
                _ => return Err(format!("unexpected \"{word}\"\n\n{USAGE}")),
            }
        }
        Ok(Args { command, plan: plan.ok_or(USAGE)?, options, flags })
    }

    fn option(&self, name: &str) -> Option<&str> {
        self.options.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("{message}");
            ExitCode::from(code)
        }
    }
}

fn run() -> Result<(), (u8, String)> {
    let args = Args::parse().map_err(|message| (3, message))?;
    let plan = pchart_store::load(&args.plan).map_err(|err| (4, err.to_string()))?;
    let today = match args.option("--today") {
        Some(text) => text.parse::<Date>().map_err(|_| (3, format!("\"{text}\" is not a date like 2026-10-29")))?,
        None => jiff::Zoned::now().date(),
    };
    let holidays = match args.option("--holidays") {
        Some(path) => Holidays::parse(&std::fs::read_to_string(path).map_err(|err| (4, format!("{path}: {err}")))?),
        None => Holidays::parse(SAMPLE_HOLIDAYS),
    };
    for lane in &plan.lanes {
        if let Some(place) = &lane.place
            && !holidays.knows(place)
        {
            eprintln!("no holiday data for {place} (lane {}); counting weekends only", lane.name);
        }
    }
    let calendars: Vec<Calendar> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
    let schedule = pchart_core::schedule(&plan, &calendars, today).map_err(|err| (1, err.to_string()))?;

    match args.command.as_str() {
        "schedule" => {
            print_schedule(&plan, &schedule, args.flags.iter().any(|flag| flag == "--json"));
            Ok(())
        }
        "export" => export(&args, &plan, &schedule, &calendars, today),
        "bench" => {
            let frames = args.option("--frames").unwrap_or("40").parse().map_err(|_| (3, "--frames needs a number".to_string()))?;
            let out = args.option("--out").ok_or((3, format!("bench needs --out FILE\n\n{USAGE}")))?;
            pchart_tui::bench(plan, calendars, today, frames, Path::new(out)).map_err(|err| (4, err.to_string()))
        }
        _ => pchart_tui::run(plan, calendars, today, &args.plan).map_err(|err| (4, err.to_string())),
    }
}

fn print_schedule(plan: &Plan, schedule: &Schedule, json: bool) {
    let day = |date: Date| date.to_string();
    if json {
        let items: Vec<serde_json::Value> = plan
            .items
            .iter()
            .zip(&schedule.slots)
            .map(|(item, slot)| {
                serde_json::json!({
                    "id": item.id,
                    "start": day(slot.planned.start),
                    "finish": day(slot.planned.finish),
                    "earliest": [day(slot.earliest.start), day(slot.earliest.finish)],
                    "latest": [day(slot.latest.start), day(slot.latest.finish)],
                    "slack_wd": slot.slack,
                })
            })
            .collect();
        let chain: Vec<&str> = schedule.chain.iter().map(|&index| plan.items[index].id.as_str()).collect();
        let out = serde_json::json!({
            "schema": 1,
            "start": day(schedule.start),
            "target": schedule.target.map(|index| plan.items[index].id.clone()),
            "spare_wd": schedule.spare(),
            "chain": chain,
            "items": items,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        return;
    }
    let short = |date: Date| date.strftime("%a %d %b").to_string();
    for &index in &schedule.order {
        let (item, slot) = (&plan.items[index], &schedule.slots[index]);
        let mark = if schedule.chain.contains(&index) { '*' } else { ' ' };
        println!(
            "{mark} {:<16} {:<11} {} – {}   latest {} – {}   slack {:>3} wd",
            item.name,
            item.lane,
            short(slot.planned.start),
            short(slot.planned.finish),
            short(slot.latest.start),
            short(slot.latest.finish),
            slot.slack
        );
    }
    if let (Some(target), Some(spare)) = (schedule.target, schedule.spare()) {
        println!("\n{}: {} working days to spare, plan starts {}", plan.items[target].name, spare, short(schedule.start));
    }
}

fn export(args: &Args, plan: &Plan, schedule: &Schedule, calendars: &[Calendar], today: Date) -> Result<(), (u8, String)> {
    let (width, height) = match args.option("--size") {
        Some(size) => size.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))).ok_or((3, format!("\"{size}\" is not a size like 1220x756")))?,
        None => (1220.0, 756.0),
    };
    let settings = Settings { width, height, unit: 1.0, today, selected: None, palette: Palette::labra() };
    let svg = pchart_draw::roadmap(plan, schedule, calendars, &settings).svg();
    let output = Output::default();
    let tree = output.tree(&svg).map_err(|err| (4, err))?;
    let write = |path: &str, bytes: &[u8]| std::fs::write(path, bytes).map_err(|err| (4, format!("{path}: {err}")));
    if let Some(path) = args.option("--svg") {
        write(path, svg.as_bytes())?;
    }
    if let Some(path) = args.option("--png") {
        write(path, &output.png(&tree, 1.0).map_err(|err| (4, err))?)?;
    }
    if let Some(path) = args.option("--pdf") {
        write(path, &output.pdf(&tree).map_err(|err| (4, err))?)?;
    }
    Ok(())
}
