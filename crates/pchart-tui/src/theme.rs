//! The colours pChart draws in (picture 44): those of the current Omarchy theme, followed while it
//! changes, or a theme named with --theme from the themes rumb ships. Without Omarchy, Labra.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

use pchart_draw::Palette;
use pchart_draw::scene::Color;

const BUNDLED: &str = include_str!("../themes.json");

pub struct Theme {
    pub palette: Palette,
    /// A theme named with --theme paints its own page, since the terminal shows another one.
    pub ground: bool,
    live: Option<Live>,
}

struct Live {
    colors: PathBuf,
    name: PathBuf,
    stamp: Option<(SystemTime, String, bool)>,
    checked: Instant,
}

impl Theme {
    /// The theme named, else the current Omarchy theme, else Labra.
    pub fn pick(name: Option<&str>) -> Result<Theme, String> {
        match name {
            None => Ok(live().unwrap_or(Theme { palette: Palette::labra(), ground: false, live: None })),
            Some(name) => named(name).map(|palette| Theme { palette, ground: true, live: None }).ok_or_else(|| format!("there is no theme \"{name}\"; the themes are {}", names().join(", "))),
        }
    }

    /// Whether it follows the Omarchy theme.
    pub fn live(&self) -> bool {
        self.live.is_some()
    }

    /// Reads the Omarchy theme again when it changed, at most once a second; true when the palette did.
    /// A theme it cannot read is Labra, as at the start.
    pub fn refresh(&mut self, now: Instant) -> bool {
        let Some(live) = &mut self.live else { return false };
        if now.saturating_duration_since(live.checked) < Duration::from_secs(1) {
            return false;
        }
        live.checked = now;
        let stamp = stamp(live);
        if stamp.is_none() || stamp == live.stamp {
            return false;
        }
        live.stamp = stamp;
        let palette = read_live(live).unwrap_or(Palette::labra());
        if palette == self.palette {
            return false;
        }
        self.palette = palette;
        true
    }
}

/// The themes --theme knows.
pub fn names() -> Vec<String> {
    let mut names: Vec<String> = std::iter::once("labra".to_string()).chain(bundled().into_keys()).collect();
    names.sort();
    names.dedup();
    names
}

fn bundled() -> BTreeMap<String, HashMap<String, String>> {
    serde_json::from_str(BUNDLED).unwrap_or_default()
}

fn named(name: &str) -> Option<Palette> {
    match name {
        "labra" => Some(Palette::labra()),
        _ => bundled().get(name).and_then(Palette::from_theme),
    }
}

fn current() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").filter(|home| !home.is_empty()).map(PathBuf::from)?;
    [".local/state/omarchy/current", ".config/omarchy/current"].into_iter().map(|dir| home.join(dir)).find(|dir| dir.join("theme/colors.toml").is_file())
}

fn live() -> Option<Theme> {
    let current = current()?;
    let mut live = Live { colors: current.join("theme/colors.toml"), name: current.join("theme.name"), stamp: None, checked: Instant::now() };
    live.stamp = stamp(&live);
    let palette = read_live(&live)?;
    Some(Theme { palette, ground: false, live: Some(live) })
}

fn stamp(live: &Live) -> Option<(SystemTime, String, bool)> {
    let modified = std::fs::metadata(&live.colors).and_then(|meta| meta.modified()).ok()?;
    let name = std::fs::read_to_string(&live.name).unwrap_or_default().trim().to_string();
    Some((modified, name, live.colors.with_file_name("light.mode").is_file()))
}

fn read_live(live: &Live) -> Option<Palette> {
    if std::fs::read_to_string(&live.name).is_ok_and(|name| name.trim().eq_ignore_ascii_case("labra")) {
        return Some(Palette::labra());
    }
    let text = std::fs::read_to_string(&live.colors).ok()?;
    Palette::from_theme(&resolve(parse(&text), live.colors.with_file_name("light.mode").is_file()))
}

/// The colours of an Omarchy colors.toml, by name.
fn parse(text: &str) -> HashMap<String, String> {
    let mut colors = HashMap::new();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else { continue };
        let key: String = key.chars().filter(|c| !matches!(c, '"' | '\'' | ' ' | '\t')).collect();
        if key.is_empty() || key.starts_with('#') {
            continue;
        }
        let value = match value.find(['"', '\'']) {
            Some(start) => {
                let rest = &value[start + 1..];
                rest[..rest.find(['"', '\'']).unwrap_or(rest.len())].to_string()
            }
            None => value.trim().to_string(),
        };
        colors.insert(key, value);
    }
    colors
}

/// Fills in the names a theme leaves out from the ones it has, as rumb does: older themes name only
/// color0 to color15, or bg and fg.
fn resolve(mut c: HashMap<String, String>, light_file: bool) -> HashMap<String, String> {
    c.retain(|_, value| !value.is_empty());
    let alias = |c: &mut HashMap<String, String>, key: &str, from: &str| {
        if !c.contains_key(key)
            && let Some(value) = c.get(from).cloned()
        {
            c.insert(key.to_string(), value);
        }
    };
    for (key, short) in [("background", "bg"), ("foreground", "fg"), ("lighter_background", "lighter_bg"), ("dark_foreground", "dark_fg"), ("light_foreground", "light_fg"), ("bright_foreground", "bright_fg")] {
        alias(&mut c, key, short);
    }
    alias(&mut c, "background", "color0");
    alias(&mut c, "foreground", "color7");
    for (key, from) in [("red", "color1"), ("green", "color2"), ("yellow", "color3"), ("blue", "color4"), ("magenta", "color5"), ("cyan", "color6"), ("bright_red", "color9"), ("magenta", "purple")] {
        alias(&mut c, key, from);
    }
    for (key, chain) in [("light_foreground", &["color7", "foreground"][..]), ("bright_foreground", &["color15", "foreground"]), ("selection", &["selection_background", "color8", "color0", "background"])] {
        for from in chain {
            alias(&mut c, key, from);
        }
    }
    if !c.contains_key("mode") {
        let light = light_file || c.get("background").and_then(|value| Color::parse(value)).is_some_and(|Color(r, g, b)| u32::from(r) + u32::from(g) + u32::from(b) > 382);
        c.insert("mode".to_string(), if light { "light" } else { "dark" }.to_string());
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_theme_that_changes_is_followed_and_one_that_cannot_be_read_is_labra() {
        let dir = std::env::temp_dir().join(format!("pchart-theme-live-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("theme")).unwrap();
        let colors = dir.join("theme/colors.toml");
        std::fs::write(&colors, "background = \"#282828\"\nforeground = \"#d4be98\"\nred = \"#ea6962\"\nyellow = \"#d8a657\"\naccent = \"#7daea3\"\n").unwrap();
        std::fs::write(dir.join("theme.name"), "gruvbox").unwrap();
        let mut live = Live { colors: colors.clone(), name: dir.join("theme.name"), stamp: None, checked: Instant::now() };
        live.stamp = stamp(&live);
        let mut theme = Theme { palette: read_live(&live).unwrap(), ground: false, live: Some(live) };
        assert_eq!(theme.palette.page, Color(0x28, 0x28, 0x28));
        let later = Instant::now() + Duration::from_secs(2);
        assert!(!theme.refresh(later), "nothing changed");
        std::fs::write(&colors, "background = \"nope\"\n").unwrap();
        std::fs::write(dir.join("theme.name"), "broken").unwrap();
        assert!(theme.refresh(later + Duration::from_secs(2)) && theme.palette == Palette::labra());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_theme_by_name_paints_its_page_and_an_unknown_name_lists_the_themes() {
        let harbor = Theme::pick(Some("harbor")).unwrap();
        assert!(harbor.ground && !harbor.live() && harbor.palette.page == Color(0xdf, 0xe4, 0xc4) && harbor.palette.warn.hex() == "#805c4a");
        assert_eq!(Theme::pick(Some("labra")).unwrap().palette, Palette::labra());
        let err = Theme::pick(Some("nope")).err().unwrap();
        assert!(err.starts_with("there is no theme \"nope\"; the themes are ") && err.contains("harbor") && err.contains("labra"));
        assert!(names().len() > 40);
    }

    #[test]
    fn an_old_colors_toml_with_only_numbered_colours_still_gives_a_palette() {
        let colors = resolve(parse("color0 = \"#101010\"\ncolor7 = '#e0e0e0'\ncolor1 = \"#cc3333\" # red\ncolor3=\"#cccc33\"\ncolor4 = \"#3366cc\"\n"), false);
        assert_eq!((colors["background"].as_str(), colors["foreground"].as_str(), colors["red"].as_str(), colors["mode"].as_str()), ("#101010", "#e0e0e0", "#cc3333", "dark"));
        let palette = Palette::from_theme(&colors).unwrap();
        assert!(palette.page == Color(0x10, 0x10, 0x10) && palette.accent != palette.ink && palette.alert != palette.ink);
    }
}
