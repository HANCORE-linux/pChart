pub struct Command {
    pub name: &'static str,
    pub usage: &'static str,
    pub about: &'static str,
}

pub const COMMANDS: &[Command] = &[
    Command { name: "open", usage: "pchart [PLAN] [--theme NAME]", about: "Open the plan in the terminal, in the colours of the current Omarchy theme, which it follows while it changes, or in Labra without Omarchy. --theme NAME takes a theme by name instead, such as labra or harbor. Without PLAN it starts: a new plan from a template, written to this folder, or a plan opened before." },
    Command { name: "new", usage: "pchart new PLAN --template ID [--name NAME] [--target DATE | --start DATE | --no-dates]", about: "Write a new plan from a template: new-application, website-relaunch, product-launch or empty. --target puts the date on its last milestone and plans backwards from it, --start plans forwards from it; without either it starts today. --no-dates keeps only how it is built: the work without lengths, so the plan has no dates until each has one. Never writes over a file." },
    Command { name: "list", usage: "pchart list", about: "The projects of the workspace pChart runs in (a folder with a workspace.toml), each with how it stands." },
    Command { name: "read", usage: "pchart read DIR [--out PLAN] [--name NAME]", about: "Read how a folder is built into a plan without dates: Rust crates and the files that use each other, and Hyprland, Lua and CSS files that take in others. A file that uses another waits on it. Writes NAME.pchart here and never writes over a file." },
    Command { name: "situation", usage: "pchart situation PLAN", about: "Where the plan stands: does the date hold, who waits on whom, what is next." },
    Command { name: "why", usage: "pchart why PLAN ITEM", about: "The chain of items that sets the date of ITEM." },
    Command { name: "waits", usage: "pchart waits PLAN", about: "Every lane that waits on another lane, and for what." },
    Command { name: "next", usage: "pchart next PLAN", about: "What is due next, most urgent first." },
    Command { name: "check", usage: "pchart check PLAN", about: "Problems, warnings and hints. Exit code 1 on warnings, 2 when a fixed date cannot be held." },
    Command { name: "schedule", usage: "pchart schedule PLAN", about: "Every calculated date, with what each item has to spare." },
    Command { name: "holidays", usage: "pchart holidays PLAN LANE", about: "The days off of a lane while the plan runs." },
    Command {
        name: "set",
        usage: "pchart set PLAN ITEM [--duration DAYS] [--on DATE | --no-earlier-than DATE | --no-later-than DATE | --calculated] [--done | --not-done] [--buffer | --no-buffer] [--important | --not-important] [--name NAME] [--lane LANE] [--who WHO] [--share PERCENT] [--note TEXT]",
        about: "Change an item.",
    },
    Command { name: "add", usage: "pchart add PLAN ID --lane LANE --name NAME ([--duration DAYS] [--buffer] | --milestone) [--after ID,ID] [--on DATE] [--who WHO]", about: "Add an item." },
    Command { name: "remove", usage: "pchart remove PLAN ITEM", about: "Remove an item that nothing waits on." },
    Command { name: "link", usage: "pchart link PLAN FROM TO", about: "Let TO wait on FROM." },
    Command { name: "unlink", usage: "pchart unlink PLAN FROM TO", about: "TO no longer waits on FROM." },
    Command { name: "apply", usage: "pchart apply PLAN EDITS.json", about: "Apply a JSON list of edits, all of them or none. \"-\" reads the list from stdin." },
    Command {
        name: "export",
        usage: "pchart export PLAN [--view roadmap|milestones|gantt|phases|now-next-later|structure|network|matrix] [--style dark|light] [--svg FILE] [--png FILE] [--pdf FILE] [--level overview|standard|full] [--size WxH] [--md FILE] [--dot FILE] [--audience team|management|client|agents]",
        about: "A view as pictures and PDF. The roadmap is dark like the terminal, at a detail level and size, or a light 16:9 page with --style light; the other views are light pages. --md writes the plan as Markdown with the milestone chart as SVG next to it. --audience leaves out what the audience does not see: management sees no warnings, the client no warnings and not what single items have to spare; agents get the item ids in Markdown. The default is team, which sees everything.",
    },
    Command { name: "bench", usage: "pchart bench PLAN --frames N --out FILE", about: "Inside a terminal: change an item N times and time every step." },
    Command { name: "help", usage: "pchart help [--json]", about: "This help; with --json for programs." },
];

pub const OPTIONS: &[(&str, &str)] = &[
    ("--json", "machine-readable output, with \"schema\": 1"),
    ("--dry-run", "with a change: show what would happen and write nothing"),
    ("--today DATE", "calculate from this day instead of today, so results repeat"),
    ("--holidays FILE", "holiday data, lines of REGION<TAB>YYYY-MM-DD<TAB>Name"),
];

pub const EXIT_CODES: &[(u8, &str)] = &[(0, "fine"), (1, "check found warnings"), (2, "check found that a fixed date cannot be held"), (3, "usage error or an edit that cannot be made"), (4, "file error")];

pub const EDITS: &[(&str, &str)] = &[
    ("duration", r#"{"op": "duration", "item": ID, "days": N}"#),
    ("fix", r#"{"op": "fix", "item": ID, "kind": "on" | "no_earlier_than" | "no_later_than", "date": "YYYY-MM-DD"}"#),
    ("unfix", r#"{"op": "unfix", "item": ID}"#),
    ("link", r#"{"op": "link", "from": ID, "to": ID}"#),
    ("unlink", r#"{"op": "unlink", "from": ID, "to": ID}"#),
    ("add", r#"{"op": "add", "item": {"id": ID, "lane": LANE, "name": NAME, "duration": "5wd" | "milestone": true, "after": [ID]}}"#),
    ("remove", r#"{"op": "remove", "item": ID}"#),
    ("rename", r#"{"op": "rename", "item": ID, "name": NAME}"#),
    ("lane", r#"{"op": "lane", "item": ID, "lane": LANE}"#),
    ("done", r#"{"op": "done", "item": ID, "done": true}"#),
    ("buffer", r#"{"op": "buffer", "item": ID, "buffer": true}"#),
    ("who", r#"{"op": "who", "item": ID, "who": NAME | null}"#),
    ("share", r#"{"op": "share", "item": ID, "share": PERCENT | null}"#),
    ("note", r#"{"op": "note", "item": ID, "note": TEXT | null}"#),
    ("important", r#"{"op": "important", "item": MILESTONE, "important": true}"#),
    ("note_at", r#"{"op": "note_at", "lane": LANE, "date": "YYYY-MM-DD", "note": TEXT | null}"#),
    ("phase", r#"{"op": "phase", "phase": NAME, "until": ID}"#),
];

const FLAGS: &[&str] = &["--json", "--dry-run", "--done", "--not-done", "--milestone", "--calculated", "--buffer", "--no-buffer", "--important", "--not-important", "--no-dates"];

pub struct Args {
    pub command: &'static str,
    pub words: Vec<String>,
    options: Vec<(String, String)>,
    flags: Vec<String>,
}

impl Args {
    pub fn parse(mut words: Vec<String>) -> Result<Args, String> {
        if words.iter().any(|word| word == "-h" || word == "--help") {
            words = vec!["help".into()];
        }
        let command = match COMMANDS.iter().find(|command| words.first().is_some_and(|word| command.name == word) && command.name != "open") {
            Some(command) => {
                words.remove(0);
                command.name
            }
            None => "open",
        };
        let (mut positional, mut options, mut flags) = (Vec::new(), Vec::new(), Vec::new());
        let mut rest = words.into_iter();
        while let Some(word) = rest.next() {
            if FLAGS.contains(&word.as_str()) {
                flags.push(word);
            } else if word.starts_with("--") {
                let value = rest.next().ok_or(format!("{word} needs a value"))?;
                options.push((word, value));
            } else {
                positional.push(word);
            }
        }
        Ok(Args { command, words: positional, options, flags })
    }

    pub fn option(&self, name: &str) -> Option<&str> {
        self.options.iter().rev().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
    }

    pub fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|flag| flag == name)
    }

    pub fn unknown_options(&self, known: &[&str]) -> Option<String> {
        self.options.iter().map(|(key, _)| key).find(|key| !known.contains(&key.as_str()) && !["--today", "--holidays"].contains(&key.as_str())).cloned()
    }

    pub fn usage(&self) -> &'static str {
        COMMANDS.iter().find(|command| command.name == self.command).map_or("", |command| command.usage)
    }
}

pub fn help() -> String {
    let mut out = String::from("pChart: project charts in the terminal that check themselves.\n\n");
    for command in COMMANDS.iter().filter(|command| command.name != "help") {
        out.push_str(&format!("  {}\n      {}\n", command.usage, command.about));
    }
    out.push_str("\nOptions for every command:\n");
    for (option, about) in OPTIONS {
        out.push_str(&format!("  {option:<18}{about}\n"));
    }
    out.push_str("\nExit codes: ");
    out.push_str(&EXIT_CODES.iter().map(|(code, about)| format!("{code} {about}")).collect::<Vec<_>>().join(", "));
    out.push_str(".\nItems and lanes are named by their ids in the plan file. Dates are YYYY-MM-DD.\n");
    out
}

pub fn help_json() -> serde_json::Value {
    serde_json::json!({
        "schema": 1,
        "commands": COMMANDS.iter().map(|command| serde_json::json!({ "name": command.name, "usage": command.usage, "about": command.about })).collect::<Vec<_>>(),
        "options": OPTIONS.iter().map(|(option, about)| serde_json::json!({ "option": option, "about": about })).collect::<Vec<_>>(),
        "exit_codes": EXIT_CODES.iter().map(|(code, about)| serde_json::json!({ "code": code, "about": about })).collect::<Vec<_>>(),
        "edits": EDITS.iter().map(|(op, example)| serde_json::json!({ "op": op, "example": example })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> Args {
        Args::parse(line.split_whitespace().map(str::to_string).collect()).unwrap()
    }

    #[test]
    fn a_plan_alone_opens_it() {
        let args = parse("portal.pchart --today 2026-10-29");
        assert_eq!((args.command, args.words.as_slice(), args.option("--today")), ("open", ["portal.pchart".to_string()].as_slice(), Some("2026-10-29")));
    }

    #[test]
    fn flags_take_no_value() {
        let args = parse("set portal.pchart frontend --duration 30 --done --dry-run --json");
        assert_eq!(args.command, "set");
        assert_eq!(args.words, ["portal.pchart", "frontend"]);
        assert!(args.flag("--done") && args.flag("--dry-run") && args.flag("--json"));
        assert_eq!(args.option("--duration"), Some("30"));
    }

    #[test]
    fn no_plan_means_the_start_and_help_means_help() {
        let args = parse("--today 2026-10-29");
        assert_eq!((args.command, args.words.len()), ("open", 0));
        assert_eq!(parse("").command, "open");
        assert_eq!(parse("situation x --help").command, "help");
    }
}
