use std::collections::HashMap;

use crate::scene::Color;

/// Colours of a drawing. Colour is a signal only: `alert` means too late, `warn` means look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub page: Color,
    pub ink: Color,
    pub bright: Color,
    pub dim: Color,
    pub muted: Color,
    pub raised: Color,
    pub rule: Color,
    pub accent: Color,
    pub warn: Color,
    pub alert: Color,
    pub alert_ground: Color,
}

impl Palette {
    pub const fn labra() -> Self {
        Palette {
            page: Color(0x04, 0x07, 0x04),
            ink: Color(0xd3, 0xd7, 0xb5),
            bright: Color(0xde, 0xe1, 0xc8),
            dim: Color(0x9e, 0xa1, 0x88),
            muted: Color(0x58, 0x5d, 0x58),
            raised: Color(0x1d, 0x20, 0x1d),
            rule: Color(0x2a, 0x2e, 0x2a),
            accent: Color(0xe3, 0x3d, 0x84),
            warn: Color(0xd3, 0xdd, 0x2c),
            alert: Color(0xee, 0x58, 0x5d),
            alert_ground: Color(0x3a, 0x14, 0x16),
        }
    }

    /// The palette under a dialog: every colour two thirds of the way to the page.
    pub fn dimmed(&self) -> Palette {
        let dim = |color: Color| {
            let mix = |from: u8, to: u8| (f32::from(from) + (f32::from(to) - f32::from(from)) * 0.68).round() as u8;
            Color(mix(color.0, self.page.0), mix(color.1, self.page.1), mix(color.2, self.page.2))
        };
        Palette { page: self.page, ink: dim(self.ink), bright: dim(self.bright), dim: dim(self.dim), muted: dim(self.muted), raised: dim(self.raised), rule: dim(self.rule), accent: dim(self.accent), warn: dim(self.warn), alert: dim(self.alert), alert_ground: dim(self.alert_ground) }
    }

    /// The palette of an Omarchy theme from its colours (picture 44): each role keeps the contrast it
    /// has against the page in Labra. Neutrals are mixed between the theme's background and text;
    /// accent, yellow and red start from the theme's own colour and move towards the text until they
    /// read. None without a background and a text colour.
    pub fn from_theme(colors: &HashMap<String, String>) -> Option<Palette> {
        let color = |key: &str| colors.get(key).and_then(|value| Color::parse(value));
        let page = color("background")?;
        let ink = color("light_foreground").or(color("foreground"))?;
        let bright = color("bright_foreground").or(color("foreground")).unwrap_or(ink);
        let neutral = |target: f64| (0..=100).map(|step| page.mix(ink, f64::from(step) / 100.0)).find(|&mixed| mixed.contrast(page) >= target).unwrap_or(ink);
        let readable = |from: Color, target: f64| (0..=40).map(|step| from.mix(ink, f64::from(step) / 40.0)).find(|&mixed| mixed.contrast(page) >= target).unwrap_or(ink);
        let alert = readable(color("red").unwrap_or(ink), 5.97);
        Some(Palette {
            page,
            ink,
            bright,
            dim: neutral(7.62),
            muted: neutral(3.01),
            raised: color("selection").filter(|selection| (1.1..=1.6).contains(&selection.contrast(page))).unwrap_or_else(|| neutral(1.23)),
            rule: neutral(1.47),
            accent: readable(color("accent").or(color("blue")).unwrap_or(ink), 5.08),
            warn: readable(color("yellow").unwrap_or(ink), 4.5),
            alert,
            alert_ground: page.mix(alert, 0.15),
        })
    }

    pub fn colors(&self) -> [Color; 11] {
        [self.page, self.ink, self.bright, self.dim, self.muted, self.raised, self.rule, self.accent, self.warn, self.alert, self.alert_ground]
    }
}

pub const FONT: &str = "JetBrains Mono";
pub const REGULAR: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");
pub const BOLD: &[u8] = include_bytes!("../assets/JetBrainsMono-Bold.ttf");
/// The font of export pages.
pub const SANS: &str = "Noto Sans";
pub const SANS_REGULAR: &[u8] = include_bytes!("../assets/NotoSans-Regular.ttf");
pub const SANS_BOLD: &[u8] = include_bytes!("../assets/NotoSans-Bold.ttf");
/// Width of `text` in the bundled Noto Sans at `size`, from the advances of its glyphs.
pub fn sans_width(text: &str, size: f32, bold: bool) -> f32 {
    static FACES: std::sync::OnceLock<[ttf_parser::Face<'static>; 2]> = std::sync::OnceLock::new();
    let faces = FACES.get_or_init(|| [SANS_REGULAR, SANS_BOLD].map(|data| ttf_parser::Face::parse(data, 0).expect("the bundled font reads")));
    let face = &faces[usize::from(bold)];
    let units: u32 = text.chars().map(|c| face.glyph_index(c).and_then(|glyph| face.glyph_hor_advance(glyph)).map_or(u32::from(face.units_per_em()) / 2, u32::from)).sum();
    units as f32 * size / f32::from(face.units_per_em())
}

/// Advance of one character in JetBrains Mono, in font sizes.
pub const ADVANCE: f32 = 0.6;

/// The lines and text size of a phase name in its arrow, `room` wide and `height` high: the whole name
/// on one line, as large as `most` and as small as `least`; else on two lines, broken at a space; else
/// the plan's short name the same way; else its first three letters in capitals ("REQ"), or the first
/// letters of its words ("B&A"); else the name cut with "…" at `least`. `width` measures text.
pub fn phase_label(name: &str, short: Option<&str>, room: f32, height: f32, most: f32, least: f32, width: impl Fn(&str, f32) -> f32) -> (Vec<String>, f32) {
    let one = |text: &str| most.min(most * room / width(text, most).max(0.01));
    let two = |text: &str| {
        let words: Vec<&str> = text.split(' ').collect();
        (1..words.len())
            .map(|at| {
                let (a, b) = (words[..at].join(" "), words[at..].join(" "));
                (one(&a).min(one(&b)).min(height / 2.6), vec![a, b])
            })
            .max_by(|x, y| x.0.total_cmp(&y.0))
    };
    let words: Vec<&str> = name.split_whitespace().collect();
    let letters = if words.len() > 1 {
        words.iter().filter_map(|word| if *word == "&" || word.eq_ignore_ascii_case("and") { Some('&') } else { word.chars().find(|c| c.is_alphanumeric()) }).collect::<String>()
    } else {
        name.chars().filter(|c| c.is_alphanumeric()).take(3).collect()
    }
    .to_uppercase();
    for text in std::iter::once(name).chain(short).chain(Some(letters.as_str()).filter(|letters| !letters.is_empty())) {
        if one(text) >= least {
            return (vec![text.to_string()], one(text));
        }
        if let Some((size, lines)) = two(text).filter(|(size, _)| *size >= least) {
            return (lines, size);
        }
    }
    let chars: Vec<char> = name.chars().collect();
    let keep = (0..chars.len()).rev().find(|&keep| width(&(chars[..keep].iter().collect::<String>() + "…"), least) <= room).unwrap_or(0);
    (vec![chars[..keep].iter().collect::<String>().trim_end().to_string() + "…"], least)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harbor_gives_the_colours_of_picture_44() {
        let colors: HashMap<String, String> = [("background", "#dfe4c4"), ("foreground", "#1c2d28"), ("light_foreground", "#253631"), ("bright_foreground", "#1c2d28"), ("accent", "#5e81ac"), ("selection", "#c8d4a8"), ("red", "#b14752"), ("yellow", "#dc8164")]
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();
        let hex = |color: Color| color.hex();
        let p = Palette::from_theme(&colors).unwrap();
        assert_eq!(
            p.colors().map(hex),
            ["#dfe4c4", "#253631", "#1c2d28", "#36463e", "#778372", "#c8d4a8", "#b6bea4", "#445f75", "#805c4a", "#794045", "#d0cbb1"].map(String::from)
        );
        assert!(Palette::from_theme(&HashMap::new()).is_none());
    }

    #[test]
    fn a_phase_name_shrinks_then_breaks_then_takes_its_short_name() {
        let width = |text: &str, size: f32| sans_width(text, size, false);
        let (lines, size) = phase_label("Requirements", None, 68.7, 52.0, 17.0, 8.0, width);
        assert_eq!(lines, ["Requirements"]);
        assert!((8.0..10.5).contains(&size));
        let (lines, _) = phase_label("Beta and acceptance testing", None, 80.0, 52.0, 17.0, 8.0, width);
        assert_eq!(lines, ["Beta and", "acceptance testing"]);
        assert_eq!(phase_label("Requirements", Some("Reqs"), 30.0, 52.0, 17.0, 8.0, width).0, ["Reqs"]);
        assert_eq!(phase_label("Requirements", None, 30.0, 52.0, 17.0, 8.0, width).0, ["REQ"]);
        assert_eq!(phase_label("Requirements", None, 12.0, 52.0, 17.0, 8.0, width).0, ["R…"]);
        assert_eq!(phase_label("Beta & acceptance", None, 30.0, 52.0, 17.0, 8.0, width).0, ["B&A"]);
        assert_eq!(phase_label("Discovery and research", None, 30.0, 18.0, 12.0, 10.0, width).0, ["D&R"]);
    }
}
