use std::fs;
use std::path::{Path, PathBuf};

use pchart_store::read::{MOST_FILES, Progress};

fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pchart-read-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn read(dir: &Path) -> Result<pchart_store::Read, pchart_core::Error> {
    pchart_store::read(dir, "Test", &mut |_| {}, &mut |_: Progress| {})
}

fn waits(read: &pchart_store::Read, item: &str, on: &str) -> bool {
    let plan = &read.plan;
    let find = |name: &str| plan.items.iter().find(|other| other.name == name).unwrap();
    find(item).after.contains(&find(on).id)
}

#[test]
fn rust_files_wait_on_the_files_they_use_and_a_circle_is_left_out_and_named() {
    let dir = folder("rust");
    write(&dir.join("base/Cargo.toml"), "[package]\nname = \"base\"\n");
    write(&dir.join("base/src/lib.rs"), "pub mod model;\npub mod plan;\n");
    write(&dir.join("base/src/model.rs"), "use crate::plan::Plan;\n");
    write(&dir.join("base/src/plan.rs"), "use crate::{model::{self, Model}, };\n");
    write(&dir.join("base/tests/it.rs"), "use base::model;\n");
    write(&dir.join("app/Cargo.toml"), "[package]\nname = \"app\"\n[dependencies]\nbase = { path = \"../base\" }\n");
    write(&dir.join("app/src/main.rs"), "fn main() { base::model::go(); }\n");
    let read = read(&dir).unwrap();
    assert_eq!(read.plan.items.len(), 4, "the test in tests/ is not part of the crate's source");
    assert_eq!(read.plan.lanes.iter().map(|lane| lane.name.as_str()).collect::<Vec<_>>(), ["base", "app"], "what others build on comes first");
    assert!(waits(&read, "app", "model"));
    assert_eq!(read.circles.len(), 1);
    assert!(waits(&read, "model", "plan") != waits(&read, "plan", "model"));
    assert!(!waits(&read, "plan", "base") && !waits(&read, "model", "base"), "a use group is not a use of the crate root");
    assert_eq!(read.kinds.get("Rust"), Some(&4));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn hyprland_lua_and_css_files_wait_on_what_they_take_in() {
    let dir = folder("config");
    write(&dir.join("hyprland.conf"), "source = ./bindings.conf\nsource = ~/somewhere/else.conf\n");
    write(&dir.join("bindings.conf"), "bind = SUPER, Q, killactive\n");
    write(&dir.join("init.lua"), "local theme = require(\"theme.colors\")\n");
    write(&dir.join("theme/colors.lua"), "return {}\n");
    write(&dir.join("style.css"), "@import \"base.css\";\n@import url(\"https://example.org/x.css\");\n");
    write(&dir.join("base.css"), "body {}\n");
    let read = read(&dir).unwrap();
    assert!(waits(&read, "hyprland.conf", "bindings.conf") && waits(&read, "init.lua", "colors.lua") && waits(&read, "style.css", "base.css"));
    assert_eq!(read.plan.items.iter().map(|item| item.after.len()).sum::<usize>(), 3);
    assert!(read.plan.without_length().len() == read.plan.items.len());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn it_does_not_read_the_system_a_whole_home_or_too_much_and_does_not_follow_links() {
    for system in ["/", "/usr", "/etc", "/proc"] {
        assert!(read(Path::new(system)).unwrap_err().to_string().contains("not the system"), "{system}");
    }
    let dir = folder("big");
    write(&dir.join("Cargo.toml"), "[package]\nname = \"big\"\n");
    for number in 0..=MOST_FILES {
        write(&dir.join(format!("src/m{number}.rs")), "");
    }
    assert!(read(&dir).unwrap_err().to_string().contains(&format!("reads at most {MOST_FILES}")));
    let dir = folder("links");
    write(&dir.join("Cargo.toml"), "[package]\nname = \"links\"\n");
    write(&dir.join("src/lib.rs"), "use crate::a;\n");
    write(&dir.join("src/a.rs"), "");
    std::os::unix::fs::symlink("..", dir.join("src/up")).unwrap();
    std::os::unix::fs::symlink("/", dir.join("src/root")).unwrap();
    let mut steps = Vec::new();
    let read = pchart_store::read(&dir, "Links", &mut |_| {}, &mut |progress| steps.push(progress)).unwrap();
    assert_eq!(read.plan.items.len(), 2);
    assert_eq!(steps.last().map(|step| (step.files, step.of_files, step.bytes == step.of_bytes)), Some((2, 2, true)));
    fs::remove_dir_all(dir).unwrap();
    fs::remove_dir_all(folder("big")).unwrap();
}
