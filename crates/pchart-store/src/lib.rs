use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use pchart_core::{Error, Plan};

fn failed(path: &Path, err: impl std::fmt::Display) -> Error {
    Error::File(format!("{}: {err}", path.display()))
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
    let _lock = Lock::take(path)?;
    let temporary = sibling(path, &format!("{}.tmp", std::process::id()));
    let written = (|| {
        let mut file = File::create(&temporary)?;
        file.write_all(plan.to_toml().as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if written.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    written.map_err(|err| failed(path, err))
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
