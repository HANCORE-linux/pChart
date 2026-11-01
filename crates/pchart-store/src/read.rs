//! How a folder is built, read into a plan without dates (picture 62): Rust crates and their files, and
//! Hyprland, Lua and CSS files that take in others. A file that uses another waits on it.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use pchart_core::edit::waits_on;
use pchart_core::model::{Item, Lane};
use pchart_core::{Error, Plan};

/// A folder read into a plan, and what was found on the way.
#[derive(Debug, Clone)]
pub struct Read {
    pub plan: Plan,
    /// The files read, by kind ("Rust", "Hyprland", "Lua", "CSS").
    pub kinds: BTreeMap<&'static str, usize>,
    /// Uses left out because the plan would wait on itself in a circle, as (file, the file it uses).
    pub circles: Vec<(String, String)>,
}

const SKIP: [&str; 4] = ["target", "node_modules", "build", "dist"];

/// The most files pChart reads: more would not make a chart anyone can read.
pub const MOST_FILES: usize = 1000;

/// The most entries pChart looks at while it counts, so that it stops early on a whole disk.
pub const MOST_ENTRIES: usize = 20_000;

/// Folders pChart does not read: the system, and a whole home.
const SYSTEM: [&str; 19] = ["/", "/bin", "/boot", "/dev", "/efi", "/etc", "/lib", "/lib64", "/mnt", "/media", "/opt", "/proc", "/root", "/run", "/sbin", "/srv", "/sys", "/usr", "/var"];

/// How far reading has come: files and bytes read, of how many.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub files: usize,
    pub of_files: usize,
    pub bytes: u64,
    pub of_bytes: u64,
}

/// Every file under `dir`, without following links, skipping hidden folders and build output; false
/// once more than `MOST_ENTRIES` entries were seen.
fn files(dir: &Path, out: &mut Vec<PathBuf>, seen: &mut usize, counting: &mut dyn FnMut(usize)) -> bool {
    let Ok(entries) = fs::read_dir(dir) else { return true };
    let mut entries: Vec<PathBuf> = entries.filter_map(|entry| entry.ok().map(|entry| entry.path())).collect();
    entries.sort();
    for path in entries {
        *seen += 1;
        if seen.is_multiple_of(500) {
            counting(*seen);
        }
        if *seen > MOST_ENTRIES {
            return false;
        }
        let name = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let Ok(meta) = fs::symlink_metadata(&path) else { continue };
        if name.starts_with('.') || SKIP.contains(&name.as_str()) || meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            if !files(&path, out, seen, counting) {
                return false;
            }
        } else {
            out.push(path);
        }
    }
    true
}

fn kind(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()? {
        "rs" => Some("Rust"),
        "conf" => Some("Hyprland"),
        "lua" => Some("Lua"),
        "css" => Some("CSS"),
        _ => None,
    }
}

/// The crate a Rust file belongs to: the nearest `Cargo.toml` above it with a package name, and the
/// module path of the file inside it ("" for the root).
fn module(path: &Path, root: &Path) -> Option<(String, String)> {
    let mut dir = path.parent()?;
    while dir.starts_with(root) {
        let manifest = dir.join("Cargo.toml");
        if let Ok(text) = fs::read_to_string(&manifest)
            && let Some(name) = package(&text)
        {
            let inside = path.strip_prefix(dir.join("src")).ok()?;
            let mut parts: Vec<String> = inside.with_extension("").iter().map(|part| part.to_string_lossy().into_owned()).collect();
            if matches!(parts.last().map(String::as_str), Some("mod")) || matches!(parts.as_slice(), [one] if one == "lib" || one == "main") {
                parts.pop();
            }
            return Some((name, parts.join("::")));
        }
        dir = dir.parent()?;
    }
    None
}

fn package(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_package = line == "[package]";
        } else if in_package && let Some(value) = line.strip_prefix("name") {
            return Some(value.trim_start().strip_prefix('=')?.trim().trim_matches('"').to_string());
        }
    }
    None
}

/// The paths a Rust file names: `crate::a::b` and `other_crate::a`, from `use` lines and anywhere in the code.
fn rust_uses(text: &str, own: &str, crates: &BTreeSet<String>) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    let words: Vec<&str> = text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':')).filter(|word| word.contains("::")).collect();
    for word in words {
        let parts: Vec<&str> = word.split("::").filter(|part| !part.is_empty()).collect();
        let Some((first, rest)) = parts.split_first() else { continue };
        if *first == "crate" && rest.is_empty() {
            continue;
        }
        let target = if *first == "crate" { own.to_string() } else { first.replace('_', "-") };
        if crates.contains(&target) {
            out.push((target, rest.iter().map(|part| part.to_string()).collect()));
        }
    }
    for group in text.split("use crate::").skip(1) {
        if let Some(inner) = group.split(';').next().and_then(|line| line.trim().strip_prefix('{')).and_then(|line| line.strip_suffix('}')) {
            for part in top_level(inner) {
                let head = part.split("::{").next().unwrap_or(part);
                let path: Vec<String> = head.split("::").map(|part| part.trim().to_string()).filter(|part| !part.is_empty() && part != "self").collect();
                out.push((own.to_string(), path));
            }
        }
    }
    out
}

/// The parts of `a, b::{c, d}, e` split at the commas outside braces.
fn top_level(text: &str) -> Vec<&str> {
    let (mut parts, mut depth, mut from) = (Vec::new(), 0, 0);
    for (at, c) in text.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(text[from..at].trim());
                from = at + 1;
            }
            _ => {}
        }
    }
    parts.push(text[from..].trim());
    parts.into_iter().filter(|part| !part.is_empty()).collect()
}

fn quoted_after<'a>(line: &'a str, start: &str) -> Option<&'a str> {
    let rest = &line[line.find(start)? + start.len()..];
    let quote = rest.find(['"', '\''])?;
    let rest = &rest[quote + 1..];
    Some(&rest[..rest.find(['"', '\''])?])
}

/// Reads `dir` into a plan without dates called `name`. `counting` hears how many entries were seen
/// while pChart counts, `reading` how far reading has come.
pub fn read(dir: &Path, name: &str, counting: &mut dyn FnMut(usize), reading: &mut dyn FnMut(Progress)) -> Result<Read, Error> {
    let root = fs::canonicalize(dir).map_err(|err| Error::File(format!("{}: {err}", dir.display())))?;
    let home = std::env::var_os("HOME").and_then(|home| fs::canonicalize(home).ok());
    if SYSTEM.iter().any(|system| root == Path::new(system)) || home.as_deref() == Some(root.as_path()) {
        return Err(Error::File(format!("{}: pChart reads one project or one config folder, not the system or a whole home. Name a folder inside it", root.display())));
    }
    let mut all = Vec::new();
    if !files(&root, &mut all, &mut 0, counting) {
        return Err(Error::File(format!("{}: more than {MOST_ENTRIES} files and folders, more than one project. Name a part of it, such as its src folder", dir.display())));
    }
    let found: Vec<(PathBuf, &'static str)> = all.into_iter().filter_map(|path| kind(&path).map(|kind| (path, kind))).filter(|(path, kind)| *kind != "Rust" || module(path, &root).is_some()).collect();
    if found.len() > MOST_FILES {
        return Err(Error::File(format!("{}: {} files pChart can read, and it reads at most {MOST_FILES}, more than a chart can show. Name a part of it, such as its src folder", dir.display(), found.len())));
    }
    if found.is_empty() {
        return Err(Error::File(format!("{}: no Rust, Hyprland, Lua or CSS files to read", dir.display())));
    }
    let crates: BTreeSet<String> = found.iter().filter(|(_, kind)| *kind == "Rust").filter_map(|(path, _)| module(path, &root).map(|(name, _)| name)).collect();
    let mut plan = Plan { format: pchart_core::model::FORMAT, name: name.to_string(), start: None, phases: Vec::new(), lanes: Vec::new(), items: Vec::new(), notes: Vec::new() };
    let mut kinds = BTreeMap::new();
    let mut keys: Vec<(String, String)> = Vec::new();
    let mut ids: BTreeMap<PathBuf, usize> = BTreeMap::new();
    for (path, kind) in &found {
        let relative = path.strip_prefix(&root).unwrap_or(path);
        let (lane, label, key) = match module(path, &root).filter(|_| *kind == "Rust") {
            Some((krate, module)) => (krate.clone(), if module.is_empty() { krate.clone() } else { module.clone() }, (krate, module)),
            None => {
                let lane = relative.parent().filter(|parent| !parent.as_os_str().is_empty()).map_or_else(|| name.to_string(), |parent| parent.to_string_lossy().into_owned());
                let file = relative.to_string_lossy().into_owned();
                (lane, relative.file_name().map_or(file.clone(), |name| name.to_string_lossy().into_owned()), (String::new(), file))
            }
        };
        if plan.lane(&slug(&lane)).is_none() {
            plan.lanes.push(Lane { id: slug(&lane), name: lane.clone(), place: None, city: None });
        }
        let id = plan.new_id(&relative.to_string_lossy());
        ids.insert(path.clone(), plan.items.len());
        keys.push(key);
        plan.items.push(Item { id, lane: slug(&lane), name: label, ..Item::default() });
        *kinds.entry(*kind).or_insert(0) += 1;
    }
    let find_module = |krate: &str, path: &[String]| -> Option<usize> {
        (0..=path.len()).rev().find_map(|length| keys.iter().position(|(other, module)| other == krate && *module == path[..length].join("::")))
    };
    let mut wanted: BTreeSet<(usize, usize)> = BTreeSet::new();
    let sizes: Vec<u64> = found.iter().map(|(path, _)| fs::symlink_metadata(path).map_or(0, |meta| meta.len())).collect();
    let mut progress = Progress { files: 0, of_files: found.len(), bytes: 0, of_bytes: sizes.iter().sum() };
    for ((path, kind), size) in found.iter().zip(&sizes) {
        progress.files += 1;
        progress.bytes += size;
        reading(progress);
        let Ok(text) = fs::read_to_string(path) else { continue };
        let index = ids[path];
        let dir_of = path.parent().unwrap_or(&root);
        match *kind {
            "Rust" => {
                let own = keys[index].0.clone();
                for (krate, module) in rust_uses(&text, &own, &crates) {
                    if let Some(other) = find_module(&krate, &module) {
                        wanted.insert((index, other));
                    }
                }
            }
            "Hyprland" => {
                for line in text.lines().map(str::trim).filter(|line| line.starts_with("source")) {
                    let Some(target) = line.split_once('=').map(|(_, value)| value.trim()) else { continue };
                    let target = target.strip_prefix("./").unwrap_or(target);
                    let target = match (target.strip_prefix("~/"), std::env::var_os("HOME")) {
                        (Some(rest), Some(home)) => fs::canonicalize(Path::new(&home).join(rest)).unwrap_or_else(|_| Path::new(&home).join(rest)),
                        _ => dir_of.join(target),
                    };
                    if let Some(&other) = ids.get(&target) {
                        wanted.insert((index, other));
                    }
                }
            }
            "Lua" => {
                for line in text.lines() {
                    let Some(module) = quoted_after(line, "require") else { continue };
                    let base = module.replace('.', "/");
                    for candidate in [root.join(format!("{base}.lua")), root.join(&base).join("init.lua"), dir_of.join(format!("{base}.lua"))] {
                        if let Some(&other) = ids.get(&candidate) {
                            wanted.insert((index, other));
                        }
                    }
                }
            }
            _ => {
                for line in text.lines().filter(|line| line.trim_start().starts_with("@import")) {
                    let Some(target) = quoted_after(line, "@import") else { continue };
                    if let Some(&other) = ids.get(&dir_of.join(target)) {
                        wanted.insert((index, other));
                    }
                }
            }
        }
    }
    let mut circles = Vec::new();
    for (index, other) in wanted.into_iter().filter(|(index, other)| index != other) {
        if waits_on(&plan, other, index) {
            circles.push((plan.items[index].name.clone(), plan.items[other].name.clone()));
        } else {
            let id = plan.items[other].id.clone();
            plan.items[index].after.push(id);
        }
    }
    order_lanes(&mut plan);
    let plan = Plan::parse(&plan.to_toml())?;
    Ok(Read { plan, kinds, circles })
}

/// The lanes in the order they are built on: a lane whose items others wait on comes before them;
/// lanes that wait on each other keep the order of their names.
fn order_lanes(plan: &mut Plan) {
    let lane_of = |id: &str| plan.item(id).map(|index| plan.items[index].lane.clone());
    let mut needs: BTreeMap<String, BTreeSet<String>> = plan.lanes.iter().map(|lane| (lane.id.clone(), BTreeSet::new())).collect();
    for item in &plan.items {
        for other in item.after.iter().filter_map(|id| lane_of(id)).filter(|lane| *lane != item.lane) {
            needs.entry(item.lane.clone()).or_default().insert(other);
        }
    }
    let mut order: Vec<String> = Vec::new();
    while order.len() < plan.lanes.len() {
        let ready = needs.iter().find(|(lane, wants)| !order.contains(lane) && wants.iter().all(|want| order.contains(want))).map(|(lane, _)| lane.clone());
        let next = ready.unwrap_or_else(|| needs.keys().find(|lane| !order.contains(lane)).cloned().expect("a lane is left"));
        order.push(next);
    }
    plan.lanes.sort_by_key(|lane| order.iter().position(|id| *id == lane.id));
}

fn slug(text: &str) -> String {
    pchart_core::model::slug(text).unwrap_or_else(|| "files".into())
}
