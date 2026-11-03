use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use pchart_core::{Error, Plan};

fn failed(path: &Path, err: impl std::fmt::Display) -> Error {
    Error::File(format!("{}: {err}", path.display()))
}

/// A workspace (see docs/PLAN.md): the nearest folder at or above `dir` with a `workspace.toml`, and
/// its name, from `name = "…"` in that file, else the folder's.
pub fn workspace(dir: &Path) -> Option<(PathBuf, String)> {
    let mut at = fs::canonicalize(dir).ok()?;
    loop {
        let manifest = at.join("workspace.toml");
        if manifest.is_file() {
            let text = fs::read_to_string(&manifest).unwrap_or_default();
            let named = text.lines().filter_map(|line| line.trim().strip_prefix("name")).find_map(|rest| Some(rest.trim_start().strip_prefix('=')?.trim().trim_matches('"').to_string()));
            let name = named.filter(|name| !name.is_empty()).unwrap_or_else(|| at.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned()));
            return Some((at, name));
        }
        if !at.pop() {
            return None;
        }
    }
}

/// The people and teams of the workspace at `root` (pictures 64), from its `workspace.toml`; none when
/// it names none or cannot be read.
pub fn people(root: &Path) -> pchart_core::people::Workspace {
    fs::read_to_string(root.join("workspace.toml")).ok().and_then(|text| pchart_core::people::Workspace::parse(&text).ok()).unwrap_or_default()
}

/// Adds `name` of `team` to the people of the workspace at `root` (pictures 72), keeping the rest of
/// its `workspace.toml` as it is written.
pub fn add_person(root: &Path, name: &str, team: Option<&str>) -> Result<(), Error> {
    let path = root.join("workspace.toml");
    let text = fs::read_to_string(&path).map_err(|err| failed(&path, err))?;
    pchart_core::people::Workspace::parse(&text).map_err(|err| failed(&path, err))?;
    let quoted = |text: &str| format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""));
    let entry = format!("{{ name = {}{} }}", quoted(name), team.map(|team| format!(", team = {}", quoted(team))).unwrap_or_default());
    let next = match people_end(&text) {
        Some((last, end, open)) => format!("{}{}{}\n  {entry},{}", &text[..last], if open { "" } else { "," }, &text[last..end], &text[end..]),
        None if text.lines().any(|line| line.trim_start().starts_with('[')) => format!("{}\n[[people]]\nname = {}{}\n", text.trim_end(), quoted(name), team.map(|team| format!("\nteam = {}", quoted(team))).unwrap_or_default()),
        None => format!("{}\npeople = [\n  {entry},\n]\n", text.trim_end()),
    };
    if pchart_core::people::Workspace::parse(&next).ok().and_then(|workspace| workspace.person(name).cloned()).is_none() {
        return Err(failed(&path, format!("could not add {name}")));
    }
    whole(&path, &next)
}

/// Where the last entry of `people = [ … ]` ends, the end of its line when only a comment follows it
/// there, and whether the list is empty or ends with a comma.
fn people_end(text: &str) -> Option<(usize, usize, bool)> {
    let start = text.match_indices("people").find(|&(at, _)| text[..at].rsplit('\n').next().is_some_and(|before| before.trim().is_empty()) && text[at + 6..].trim_start().starts_with('='))?.0;
    let open = start + text[start..].find('[')?;
    let (mut depth, mut last, mut chars) = (0, open + 1, text[open..].char_indices().map(|(at, c)| (open + at, c)));
    while let Some((at, c)) = chars.next() {
        match c {
            '#' => while chars.next().is_some_and(|(_, c)| c != '\n') {},
            '"' | '\'' => {
                let mut escaped = false;
                for (end, next) in chars.by_ref() {
                    if next == c && !escaped {
                        last = end + 1;
                        break;
                    }
                    escaped = c == '"' && next == '\\' && !escaped;
                }
            }
            '[' | '{' => {
                depth += 1;
                last = at + 1;
            }
            ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    let line = text[last..].find('\n').map_or(text.len(), |end| last + end);
                    let rest = text[last..line].trim_start();
                    let end = if rest.is_empty() || rest.starts_with('#') { line } else { last };
                    return Some((last, end, text[..last].ends_with('[') || text[..last].ends_with(',')));
                }
                last = at + 1;
            }
            c if !c.is_whitespace() => last = at + c.len_utf8(),
            _ => {}
        }
    }
    None
}

/// Makes `dir` a team folder (pictures 73): a `workspace.toml` with `people`, each with their team;
/// never over a file that is there.
pub fn make_team(dir: &Path, people: &[(String, Option<String>)]) -> Result<PathBuf, Error> {
    let path = dir.join("workspace.toml");
    let quoted = |text: &str| format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""));
    let lines: String = people.iter().map(|(name, team)| format!("  {{ name = {}{} }},\n", quoted(name), team.as_deref().map(|team| format!(", team = {}", quoted(team))).unwrap_or_default())).collect();
    let mut file = OpenOptions::new().write(true).create_new(true).open(&path).map_err(|err| if err.kind() == ErrorKind::AlreadyExists { failed(&path, "is there already") } else { failed(&path, err) })?;
    file.write_all(format!("people = [\n{lines}]\n").as_bytes()).map_err(|err| failed(&path, err))?;
    Ok(path)
}

/// The workspace the plan at `path` is in (pictures 64), with its people and its projects, `plan` among
/// them as it is now; `lanes` gives the calendars of a plan's lanes. A project that cannot be read or
/// scheduled is left out.
pub fn crew(path: &Path, plan: &Plan, schedule: &pchart_core::Schedule, today: jiff::civil::Date, lanes: &dyn Fn(&Plan) -> Vec<pchart_core::Calendar>) -> Option<pchart_core::people::Crew> {
    use pchart_core::people::{Crew, Project};
    let dir = path.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let (root, name) = workspace(dir)?;
    let this = fs::canonicalize(path).ok();
    let here_project = || Project { name: plan.name.clone(), plan: plan.clone(), schedule: schedule.clone(), calendars: lanes(plan) };
    let mut found = Vec::new();
    let mut here = None;
    for file in projects(&root) {
        if fs::canonicalize(&file).ok() == this {
            here = Some(found.len());
            found.push(here_project());
            continue;
        }
        let Ok(other) = load(&file) else { continue };
        let calendars = lanes(&other);
        let Ok(dates) = pchart_core::schedule(&other, &calendars, today) else { continue };
        found.push(Project { name: other.name.clone(), plan: other, schedule: dates, calendars });
    }
    let here = here.unwrap_or_else(|| {
        found.push(here_project());
        found.len() - 1
    });
    Some(Crew { name, workspace: people(&root), projects: found, here })
}

/// The reports about the plan file named `project` in the workspace at `root` (pictures 65), newest
/// first; files that are no reports are left out.
pub fn reports(root: &Path, project: &str) -> Vec<(PathBuf, pchart_core::report::Report)> {
    let Ok(entries) = fs::read_dir(root.join("reports")) else { return Vec::new() };
    let mut out: Vec<(PathBuf, pchart_core::report::Report)> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
        .filter_map(|path| fs::read_to_string(&path).ok().and_then(|text| pchart_core::report::Report::parse(&text).ok()).map(|report| (path, report)))
        .filter(|(_, report)| report.project == project)
        .collect();
    out.sort_by(|a, b| b.1.at.cmp(&a.1.at).then_with(|| b.0.cmp(&a.0)));
    out
}

/// Writes a new report into the workspace at `root`, never over another one. Returns its file.
pub fn report(root: &Path, report: &pchart_core::report::Report) -> Result<PathBuf, Error> {
    let dir = root.join("reports");
    fs::create_dir_all(&dir).map_err(|err| failed(&dir, err))?;
    let name = report.file_name();
    let stem = name.trim_end_matches(".toml");
    for number in 1..100 {
        let path = if number == 1 { dir.join(&name) } else { dir.join(format!("{stem}-{number}.toml")) };
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => return whole(&path, &report.to_toml()).map(|()| path),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(failed(&path, err)),
        }
    }
    Err(failed(&dir, "has too many reports of that minute"))
}

/// Writes the project lead's answer into the report at `path`, with what they took when they changed it.
pub fn answer(path: &Path, answer: pchart_core::report::Answer, taken: Option<pchart_core::report::Taken>) -> Result<(), Error> {
    let text = fs::read_to_string(path).map_err(|err| failed(path, err))?;
    let mut report = pchart_core::report::Report::parse(&text).map_err(|err| failed(path, err))?;
    report.answer = Some(answer);
    report.taken = taken;
    whole(path, &report.to_toml())
}

/// Takes the answer out of the report at `path` again, as when taking it is undone.
pub fn reopen(path: &Path) -> Result<(), Error> {
    let text = fs::read_to_string(path).map_err(|err| failed(path, err))?;
    let mut report = pchart_core::report::Report::parse(&text).map_err(|err| failed(path, err))?;
    (report.answer, report.taken) = (None, None);
    whole(path, &report.to_toml())
}

/// The projects of a workspace: the plans beside its `workspace.toml` and in its `projects` folder, by
/// file name.
pub fn projects(workspace: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = [workspace.to_path_buf(), workspace.join("projects")].iter().filter_map(|dir| fs::read_dir(dir).ok()).flatten().filter_map(|entry| entry.ok().map(|entry| entry.path())).filter(|path| path.extension().is_some_and(|extension| extension == "pchart")).collect();
    out.sort();
    out
}

pub fn load(path: &Path) -> Result<Plan, Error> {
    let text = fs::read_to_string(path).map_err(|err| failed(path, err))?;
    Plan::parse(&text).map_err(|err| match err {
        Error::File(message) => failed(path, message),
        other => other,
    })
}

/// Writes the plan so that the file is always whole: a temporary file next to it, flushed to disk,
/// then renamed over it. A lock keeps two writers (the TUI and an agent, say) from interleaving.
pub fn save(path: &Path, plan: &Plan) -> Result<(), Error> {
    whole(path, &plan.to_toml())
}

/// Writes `text` so that the file is always whole, as `save` does.
fn whole(path: &Path, text: &str) -> Result<(), Error> {
    let _lock = Lock::take(path)?;
    let temporary = sibling(path, &format!("{}.tmp", std::process::id()));
    let written = (|| {
        let mut file = File::create(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if written.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    written.map_err(|err| failed(path, err))
}

/// Writes a new plan, never over a file that is there.
pub fn create(path: &Path, plan: &Plan) -> Result<(), Error> {
    OpenOptions::new().write(true).create_new(true).open(path).map_err(|err| if err.kind() == ErrorKind::AlreadyExists { failed(path, "is there already") } else { failed(path, err) })?;
    save(path, plan).inspect_err(|_| {
        let _ = fs::remove_file(path);
    })
}

/// The templates pChart ships, by id.
const TEMPLATES: [(&str, &str); 4] = [
    ("new-application", include_str!("../templates/new-application.pchart")),
    ("website-relaunch", include_str!("../templates/website-relaunch.pchart")),
    ("product-launch", include_str!("../templates/product-launch.pchart")),
    ("empty", include_str!("../templates/empty.pchart")),
];

/// The templates pChart ships: their ids and plans.
pub fn templates() -> Vec<(&'static str, Plan)> {
    TEMPLATES.iter().map(|(id, text)| (*id, Plan::parse(text).expect("the bundled templates are plans"))).collect()
}

/// Where pChart keeps what it remembers between runs: `$XDG_STATE_HOME/pchart`, else
/// `~/.local/state/pchart`.
fn state() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME").filter(|dir| !dir.is_empty()).map(PathBuf::from).or_else(|| std::env::var_os("HOME").filter(|dir| !dir.is_empty()).map(|home| PathBuf::from(home).join(".local/state")))?;
    Some(base.join("pchart"))
}

/// Who uses pChart on this computer, as the people of a workspace name them (pictures 65):
/// `$PCHART_NAME`, else the first line of `$XDG_CONFIG_HOME/pchart/name`, else of `~/.config/pchart/name`.
pub fn me() -> Option<String> {
    let config = std::env::var_os("XDG_CONFIG_HOME").filter(|dir| !dir.is_empty()).map(PathBuf::from).or_else(|| std::env::var_os("HOME").filter(|dir| !dir.is_empty()).map(|home| PathBuf::from(home).join(".config")));
    me_from(std::env::var("PCHART_NAME").ok(), config.as_deref())
}

fn me_from(named: Option<String>, config: Option<&Path>) -> Option<String> {
    named.map(|name| name.trim().to_string()).filter(|name| !name.is_empty()).or_else(|| fs::read_to_string(config?.join("pchart/name")).ok()?.lines().next().map(|line| line.trim().to_string()).filter(|name| !name.is_empty()))
}

/// The plans opened last, newest first, those that still exist.
pub fn recent() -> Vec<PathBuf> {
    state().map(|dir| recent_in(&dir)).unwrap_or_default()
}

/// Puts `path` first among the plans opened last, keeping eight.
pub fn remember(path: &Path) {
    if let Some(dir) = state() {
        remember_in(&dir, path);
    }
}

fn recent_in(dir: &Path) -> Vec<PathBuf> {
    fs::read_to_string(dir.join("recent")).unwrap_or_default().lines().map(PathBuf::from).filter(|path| path.is_file()).collect()
}

fn remember_in(dir: &Path, path: &Path) {
    let Ok(path) = fs::canonicalize(path) else { return };
    let mut paths = recent_in(dir);
    paths.retain(|other| other != &path);
    paths.insert(0, path);
    paths.truncate(8);
    let text: String = paths.iter().map(|path| format!("{}\n", path.display())).collect();
    let _ = fs::create_dir_all(dir).and_then(|()| fs::write(dir.join("recent"), text));
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".");
    name.push(suffix);
    path.with_file_name(name)
}

struct Lock(PathBuf);

impl Lock {
    fn take(path: &Path) -> Result<Lock, Error> {
        let lock = sibling(path, "lock");
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&lock) {
                Ok(mut file) => {
                    let _ = write!(file, "{}", std::process::id());
                    return Ok(Lock(lock));
                }
                Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                    let holder = fs::read_to_string(&lock).ok().and_then(|text| text.trim().parse::<u32>().ok());
                    if holder.is_none_or(|pid| !Path::new(&format!("/proc/{pid}")).exists()) {
                        let _ = fs::remove_file(&lock);
                        continue;
                    }
                    if Instant::now() >= deadline {
                        return Err(failed(path, format!("is being changed by process {}; try again", holder.unwrap_or_default())));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(err) => return Err(failed(path, err)),
            }
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pchart-store-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn someone_new_joins_the_people_and_the_rest_of_the_file_stays_as_written() {
        let dir = folder("add-person");
        let file = dir.join("workspace.toml");
        let cases = [
            ("name = \"Acme\"\n# who is who\npeople = [\n  { name = \"Priya Nair\", team = \"Frontend\" }, # back in January\n]\n", "name = \"Acme\"\n# who is who\npeople = [\n  { name = \"Priya Nair\", team = \"Frontend\" }, # back in January\n  { name = \"Nora \\\"N\\\" Berg\", team = \"Design/UX\" },\n]\n"),
            ("people = [{ name = \"Lena\", team = \"T\" }]\n", "people = [{ name = \"Lena\", team = \"T\" },\n  { name = \"Nora \\\"N\\\" Berg\", team = \"Design/UX\" },]\n"),
            ("people = []\nlead = \"Lena\"\n", "people = [\n  { name = \"Nora \\\"N\\\" Berg\", team = \"Design/UX\" },]\nlead = \"Lena\"\n"),
            ("name = \"Acme\"\n", "name = \"Acme\"\npeople = [\n  { name = \"Nora \\\"N\\\" Berg\", team = \"Design/UX\" },\n]\n"),
            ("[[people]]\nname = \"Lena\"\n", "[[people]]\nname = \"Lena\"\n[[people]]\nname = \"Nora \\\"N\\\" Berg\"\nteam = \"Design/UX\"\n"),
        ];
        for (before, after) in cases {
            fs::write(&file, before).unwrap();
            add_person(&dir, "Nora \"N\" Berg", Some("Design/UX")).unwrap();
            assert_eq!(fs::read_to_string(&file).unwrap(), after);
        }
        fs::write(&file, "people = [").unwrap();
        assert!(add_person(&dir, "Nora", None).is_err());
        assert_eq!(fs::read_to_string(&file).unwrap(), "people = [");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_folder_becomes_a_team_and_its_plans_its_projects() {
        let dir = folder("make-team");
        fs::create_dir_all(dir.join("projects")).unwrap();
        for file in ["b.pchart", "projects/a.pchart", "notes.txt"] {
            fs::write(dir.join(file), "").unwrap();
        }
        let path = make_team(&dir, &[("Lena \"L\" Vogel".into(), Some("Design/UX".into())), ("Tom".into(), None)]).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "people = [\n  { name = \"Lena \\\"L\\\" Vogel\", team = \"Design/UX\" },\n  { name = \"Tom\" },\n]\n");
        assert_eq!(people(&dir).people.len(), 2);
        assert!(make_team(&dir, &[]).unwrap_err().to_string().ends_with("is there already"));
        assert_eq!(people(&dir).people.len(), 2);
        assert_eq!(projects(&dir), [dir.join("b.pchart"), dir.join("projects/a.pchart")]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_workspace_is_found_from_inside_it_and_lists_its_projects() {
        let dir = folder("workspace");
        fs::create_dir_all(dir.join("projects/old")).unwrap();
        fs::write(dir.join("workspace.toml"), "# people and places\nname = \"Acme\"\n").unwrap();
        for file in ["b.pchart", "a.pchart", "notes.md"] {
            fs::write(dir.join("projects").join(file), "").unwrap();
        }
        let root = fs::canonicalize(&dir).unwrap();
        assert_eq!(workspace(&dir.join("projects/old")), Some((root.clone(), "Acme".to_string())));
        assert_eq!(projects(&root), [root.join("projects/a.pchart"), root.join("projects/b.pchart")]);
        fs::write(dir.join("workspace.toml"), "").unwrap();
        assert_eq!(workspace(&dir).map(|(_, name)| name), root.file_name().map(|name| name.to_string_lossy().into_owned()));
        assert_eq!(workspace(&std::env::temp_dir().join("pchart-nowhere-at-all")), None);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reports_are_written_one_file_each_and_answered_in_their_file() {
        use pchart_core::report::{Answer, Report, Status, Taken};
        let dir = folder("reports");
        let made = |item: &str, at: &str| Report { project: "portal.pchart".into(), item: item.into(), by: "Priya Nair".into(), at: at.parse().unwrap(), status: Status::Later, days: 3, comment: None, answer: None, taken: None };
        let first = report(&dir, &made("frontend", "2026-10-29T08:40")).unwrap();
        let second = report(&dir, &made("frontend", "2026-10-29T08:40")).unwrap();
        assert_eq!((first.file_name().unwrap(), second.file_name().unwrap()), ("2026-10-29-0840-priya-nair-frontend.toml".as_ref(), "2026-10-29-0840-priya-nair-frontend-2.toml".as_ref()));
        report(&dir, &made("api", "2026-10-30T09:00")).unwrap();
        let mut other = made("frontend", "2026-10-31T09:00");
        other.project = "website.pchart".into();
        report(&dir, &other).unwrap();
        fs::write(dir.join("reports/notes.toml"), "not a report").unwrap();
        let found = reports(&dir, "portal.pchart");
        assert_eq!(found.iter().map(|(_, report)| report.item.as_str()).collect::<Vec<_>>(), ["api", "frontend", "frontend"]);
        answer(&first, Answer::Taken, Some(Taken { status: Status::Later, days: 2, comment: None })).unwrap();
        let answered = reports(&dir, "portal.pchart").into_iter().find(|(path, _)| *path == first).unwrap().1;
        assert_eq!((answered.answer, answered.as_taken().days), (Some(Answer::Taken), 2));
        assert!(reports(&dir.join("nowhere"), "portal.pchart").is_empty());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn who_uses_pchart_here_comes_from_pchart_name_or_the_config() {
        let dir = folder("me");
        assert_eq!(me_from(None, Some(&dir)), None);
        fs::create_dir_all(dir.join("pchart")).unwrap();
        fs::write(dir.join("pchart/name"), "  Anna Lead \nsecond line\n").unwrap();
        assert_eq!(me_from(None, Some(&dir)).as_deref(), Some("Anna Lead"));
        assert_eq!(me_from(Some(" Priya Nair ".into()), Some(&dir)).as_deref(), Some("Priya Nair"));
        assert_eq!(me_from(Some("  ".into()), Some(&dir)).as_deref(), Some("Anna Lead"));
        assert_eq!(me_from(None, None), None);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_plan_in_a_workspace_comes_with_its_people_and_the_other_projects() {
        let dir = folder("crew");
        fs::create_dir_all(dir.join("projects")).unwrap();
        fs::write(dir.join("workspace.toml"), "name = \"Acme\"\npeople = [{ name = \"Lena Vogel\", team = \"Design/UX\", capacity = 80, away = [{ from = 2027-01-04, to = 2027-01-08 }] }]\n").unwrap();
        let text = include_str!("../../../samples/customer-portal.pchart");
        for file in ["a.pchart", "b.pchart", "broken.pchart"] {
            fs::write(dir.join("projects").join(file), if file == "broken.pchart" { "not a plan" } else { text }).unwrap();
        }
        let plan = Plan::parse(text).unwrap();
        let lanes = |plan: &Plan| vec![pchart_core::Calendar::default(); plan.lanes.len()];
        let today = jiff::civil::date(2026, 10, 29);
        let schedule = pchart_core::schedule(&plan, &lanes(&plan), today).unwrap();
        let found = crew(&dir.join("projects/b.pchart"), &plan, &schedule, today, &lanes).unwrap();
        assert_eq!((found.name.as_str(), found.projects.len(), found.here), ("Acme", 2, 1));
        let lena = found.workspace.person("Lena Vogel").unwrap();
        assert_eq!((lena.team.as_deref(), found.workspace.capacity("Lena Vogel"), lena.away.len()), (Some("Design/UX"), 80, 1));
        let elsewhere = crew(&dir.join("plan.pchart"), &plan, &schedule, today, &lanes).unwrap();
        assert_eq!((elsewhere.projects.len(), elsewhere.here), (3, 2));
        fs::write(dir.join("workspace.toml"), "people = 3").unwrap();
        assert!(crew(&dir.join("projects/a.pchart"), &plan, &schedule, today, &lanes).unwrap().workspace.people.is_empty());
        assert!(crew(&std::env::temp_dir().join("pchart-nowhere-at-all/plan.pchart"), &plan, &schedule, today, &lanes).is_none());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_saved_plan_loads_again_and_leaves_nothing_behind() {
        let dir = folder("save");
        let path = dir.join("portal.pchart");
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        save(&path, &plan).unwrap();
        assert_eq!(load(&path).unwrap(), plan);
        let names: Vec<String> = fs::read_dir(&dir).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned()).collect();
        assert_eq!(names, ["portal.pchart"]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_new_plan_is_never_written_over_a_file() {
        let dir = folder("create");
        let path = dir.join("portal.pchart");
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        create(&path, &plan).unwrap();
        assert_eq!(load(&path).unwrap(), plan);
        let err = create(&path, &Plan { name: "Other".into(), ..plan.clone() }).unwrap_err();
        assert!(err.to_string().ends_with("portal.pchart: is there already"), "{err}");
        assert_eq!(load(&path).unwrap(), plan);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn the_templates_are_plans_that_check_clean() {
        let calendar = pchart_core::Holidays::parse("").calendar(None);
        let today = "2026-10-29".parse().unwrap();
        let ids: Vec<&str> = templates().iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, ["new-application", "website-relaunch", "product-launch", "empty"]);
        for (id, template) in templates() {
            let plan = template.new_plan(&template.name, Some("2027-03-01".parse().unwrap()), false);
            let calendars = vec![calendar.clone(); plan.lanes.len()];
            let schedule = pchart_core::schedule(&plan, &calendars, today).unwrap();
            let analysis = pchart_core::analyse(&plan, &calendars, &schedule, today);
            assert!(analysis.checks.is_empty(), "{id}: {:?}", analysis.checks);
        }
    }

    #[test]
    fn the_plans_opened_last_come_first_and_only_eight_are_kept() {
        let dir = folder("recent");
        let state = dir.join("state");
        assert!(recent_in(&state).is_empty());
        let paths: Vec<PathBuf> = (0..10).map(|number| dir.join(format!("plan-{number}.pchart"))).collect();
        for path in &paths {
            fs::write(path, "").unwrap();
            remember_in(&state, path);
        }
        remember_in(&state, &paths[5]);
        fs::remove_file(&paths[8]).unwrap();
        let canonical = |number: usize| fs::canonicalize(&paths[number]).unwrap();
        assert_eq!(recent_in(&state), [5, 9, 7, 6, 4, 3, 2].map(canonical));
        remember_in(&state, &dir.join("gone.pchart"));
        assert_eq!(recent_in(&state).len(), 7);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_lock_left_by_a_dead_process_is_taken_over() {
        let dir = folder("stale");
        let path = dir.join("portal.pchart");
        fs::write(sibling(&path, "lock"), "999999999").unwrap();
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        save(&path, &plan).unwrap();
        assert!(!sibling(&path, "lock").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_live_lock_makes_the_writer_wait_and_then_give_up() {
        let dir = folder("live");
        let path = dir.join("portal.pchart");
        fs::write(sibling(&path, "lock"), "1").unwrap();
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        let err = save(&path, &plan).unwrap_err();
        assert!(err.to_string().ends_with("is being changed by process 1; try again"), "{err}");
        assert!(!path.exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
