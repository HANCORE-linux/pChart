//! Small pictures for the terminal screens: text larger than a cell, and the symbols of a legend.

use crate::scene::{Anchor, Color, Fill, Mark, Scene, Shape, Stroke};
use crate::style::{FONT, Palette};

fn scene(width: f32, height: f32, palette: &Palette) -> Scene {
    Scene { width, height, background: None, hatch: (palette.raised, palette.muted), hatch_alert: (palette.alert_ground, palette.alert), spare: palette.dim, font: FONT, marks: Vec::new() }
}

/// `text` in bold at `size` pixels, its baseline near the bottom of a `width` × `height` picture.
pub fn big(text: &str, size: f32, color: Color, width: f32, height: f32, palette: &Palette) -> Scene {
    let mut s = scene(width, height, palette);
    s.add(Shape::Text { x: 0.0, y: height * 0.84, text: text.to_string(), size, color, bold: true, anchor: Anchor::Start });
    s
}

/// What a symbol in a legend shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Chain,
    Spare,
    SpareHatched,
    Milestone,
    Fixed,
    Holiday,
    Today,
}

/// A legend symbol in a `width` × `height` picture whose lower half is the row of its label; `unit`
/// is one pixel of a 21 px tall cell.
pub fn symbol(symbol: Symbol, width: f32, height: f32, unit: f32, palette: &Palette) -> Scene {
    let p = palette;
    let mut s = scene(width, height, p);
    let (u, y) = (unit, height * 0.75);
    let rect = |x: f32, w: f32, fill: Fill, stroke: Option<Stroke>| Shape::Rect { x, y: y - 5.0 * u, w, h: 10.0 * u, radius: 2.0 * u, fill, stroke };
    let diamond = |x: f32, y: f32, r: f32, color: Color| Shape::Path { points: vec![(x, y - r), (x + r, y), (x, y + r), (x - r, y)], closed: true, fill: Fill::Color(color), stroke: None };
    match symbol {
        Symbol::Chain => s.marks.push(Mark { shape: rect(0.0, 22.0 * u, Fill::Color(p.ink), None), opacity: 0.95, item: None }),
        Symbol::Spare => s.marks.push(Mark { shape: rect(0.0, 22.0 * u, Fill::Color(p.dim), None), opacity: 0.5, item: None }),
        Symbol::SpareHatched => {
            s.marks.push(Mark { shape: rect(0.0, 12.0 * u, Fill::Color(p.dim), None), opacity: 0.5, item: None });
            s.add(rect(12.0 * u, 14.0 * u, Fill::Spare, Some(Stroke { color: p.muted, width: u, dash: None })));
        }
        Symbol::Milestone => s.add(diamond(10.0 * u, y, 6.0 * u, p.ink)),
        Symbol::Fixed => {
            s.add(diamond(10.0 * u, y + 2.0 * u, 6.0 * u, p.bright));
            s.add(Shape::Line { x1: 10.0 * u, y1: y - 6.0 * u, x2: 10.0 * u, y2: y - 16.0 * u, stroke: Stroke { color: p.bright, width: 1.6 * u, dash: None } });
            s.add(Shape::Circle { x: 10.0 * u, y: y - 19.0 * u, r: 4.0 * u, fill: p.bright });
        }
        Symbol::Holiday => s.add(Shape::Rect { x: 4.0 * u, y: y - 9.0 * u, w: 12.0 * u, h: 18.0 * u, radius: 0.0, fill: Fill::Hatch, stroke: None }),
        Symbol::Today => s.add(Shape::Line { x1: 10.0 * u, y1: y - 9.0 * u, x2: 10.0 * u, y2: y + 9.0 * u, stroke: Stroke { color: p.ink, width: 1.5 * u, dash: None } }),
    }
    s
}

/// A page of an export as the preview shows it: scaled to fit a `width` × `height` picture, centred,
/// over a soft shadow. `unit` is one pixel of a 21 px tall cell.
pub fn sheet(page: &Scene, width: f32, height: f32, unit: f32) -> String {
    let (dx, dy) = (6.0 * unit, 8.0 * unit);
    let scale = ((width - dx) / page.width).min((height - dy) / page.height);
    let (w, h) = (page.width * scale, page.height * scale);
    let (x, y) = ((width - dx - w) / 2.0, (height - dy - h) / 2.0);
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}"><rect x="{}" y="{}" width="{w}" height="{h}" fill="#000000" opacity="0.5"/>{}</svg>"##,
        x + dx,
        y + dy,
        page.nested(x, y, w, h)
    )
}

/// Symbols that stand for words. They are drawn, not taken from a font, so every terminal shows them
/// the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Check,
    Clock,
    Cross,
    Play,
    Warn,
    Info,
    Person,
    Calendar,
    Folder,
    /// A short bar, as the legend draws an item.
    Bar,
}

fn ring(x: f32, y: f32, r: f32) -> Vec<(f32, f32)> {
    (0..40).map(|step| step as f32 / 40.0 * std::f32::consts::TAU).map(|angle| (x + r * angle.cos(), y + r * angle.sin())).collect()
}

/// `icon` centred at `x`, `y`, `size` pixels across, in `color`; `ground` is what shows through it.
pub fn draw_icon(s: &mut Scene, icon: Icon, x: f32, y: f32, size: f32, color: Color, ground: Color) {
    let r = size / 2.0;
    let line = |width: f32| Some(Stroke { color, width, dash: None });
    let open = |points: Vec<(f32, f32)>, width: f32| Shape::Path { points, closed: false, fill: Fill::None, stroke: line(width) };
    let circle = |s: &mut Scene, width: f32| s.add(Shape::Path { points: ring(x, y, r - width / 2.0), closed: true, fill: Fill::None, stroke: line(width) });
    match icon {
        Icon::Check => {
            circle(s, r / 8.0);
            s.add(open(vec![(x - r * 0.45, y + r * 0.02), (x - r * 0.1, y + r * 0.36), (x + r * 0.5, y - r * 0.32)], r / 7.0));
        }
        Icon::Clock => {
            circle(s, (r / 8.0).max(1.4));
            s.add(open(vec![(x, y - r * 0.6), (x, y), (x + r * 0.45, y)], (r / 7.0).max(1.4)));
        }
        Icon::Cross => {
            circle(s, r / 8.0);
            let k = r * 0.38;
            s.add(open(vec![(x - k, y - k), (x + k, y + k)], r / 7.0));
            s.add(open(vec![(x + k, y - k), (x - k, y + k)], r / 7.0));
        }
        Icon::Play => s.add(Shape::Path { points: vec![(x - r * 0.5, y - r * 0.7), (x + r * 0.7, y), (x - r * 0.5, y + r * 0.7)], closed: true, fill: Fill::Color(color), stroke: None }),
        Icon::Warn => {
            s.add(Shape::Path { points: vec![(x, y - r), (x + r * 1.1, y + r * 0.85), (x - r * 1.1, y + r * 0.85)], closed: true, fill: Fill::Color(color), stroke: None });
            s.add(Shape::Rect { x: x - r * 0.1, y: y - r * 0.4, w: r * 0.2, h: r * 0.7, radius: r * 0.05, fill: Fill::Color(ground), stroke: None });
            s.add(Shape::Circle { x, y: y + r * 0.55, r: r * 0.12, fill: ground });
        }
        Icon::Info => {
            circle(s, (r / 8.0).max(1.4));
            s.add(Shape::Rect { x: x - r * 0.1, y: y - r * 0.15, w: r * 0.2, h: r * 0.75, radius: 0.0, fill: Fill::Color(color), stroke: None });
            s.add(Shape::Circle { x, y: y - r * 0.5, r: r * 0.14, fill: color });
        }
        Icon::Person => {
            s.add(Shape::Circle { x, y: y - r * 0.45, r: r * 0.38, fill: color });
            let body: Vec<(f32, f32)> = (0..=16).map(|step| step as f32 / 16.0 * std::f32::consts::PI).map(|angle| (x - r * 0.75 * angle.cos(), y + r - r * 0.9 * angle.sin())).collect();
            s.add(Shape::Path { points: body, closed: true, fill: Fill::Color(color), stroke: None });
        }
        Icon::Calendar => {
            let width = (r / 8.0).max(1.4);
            s.add(Shape::Rect { x: x - r, y: y - r * 0.8, w: 2.0 * r, h: 1.8 * r, radius: r * 0.15, fill: Fill::None, stroke: line(width) });
            s.add(open(vec![(x - r, y - r * 0.25), (x + r, y - r * 0.25)], width));
        }
        Icon::Bar => s.add(Shape::Rect { x: x - r * 1.5, y: y - r * 0.55, w: r * 3.0, h: r * 1.1, radius: r * 0.25, fill: Fill::Color(color), stroke: None }),
        Icon::Folder => s.add(Shape::Path { points: vec![(x - r, y - r * 0.6), (x - r * 0.2, y - r * 0.6), (x, y - r * 0.3), (x + r, y - r * 0.3), (x + r, y + r * 0.7), (x - r, y + r * 0.7)], closed: true, fill: Fill::None, stroke: line((r / 8.0).max(1.4)) }),
    }
}

/// Runs of text one after the other on one baseline: `(text, color, bold)` at `size` pixels in the
/// monospaced font, starting at `x`. Returns where the last run ends.
pub fn runs(s: &mut Scene, x: f32, y: f32, size: f32, parts: &[(&str, Color, bool)]) -> f32 {
    let mut at = x;
    for &(text, color, bold) in parts {
        s.add(Shape::Text { x: at, y, text: text.to_string(), size, color, bold, anchor: Anchor::Start });
        at += text.chars().count() as f32 * size * crate::style::ADVANCE;
    }
    at
}

/// A part of a line: a symbol and its colour, then text, its colour and whether it is bold.
pub type Part = (Option<(Icon, Color)>, String, Color, bool);

/// A line of symbols and words, one part after the other with room between them, as the line about
/// the selection under the roadmap. What does not fit is cut with "…".
pub fn line(parts: &[Part], width: f32, height: f32, unit: f32, palette: &Palette) -> Scene {
    let u = unit;
    let mut s = scene(width, height, palette);
    let size = 16.6 * u;
    let y = height * 0.72;
    let mut x = 2.0 * u;
    for (index, (icon, text, color, bold)) in parts.iter().enumerate() {
        if index > 0 {
            x += 30.0 * u;
        }
        if let Some((icon, tone)) = icon {
            let (centre, advance) = if *icon == Icon::Bar { (12.0 * u, 34.0 * u) } else { (8.0 * u, 24.0 * u) };
            draw_icon(&mut s, *icon, x + centre, y - 5.0 * u, 15.0 * u, *tone, palette.page);
            x += advance;
        }
        let room = ((width - x) / (size * crate::style::ADVANCE)).floor() as usize;
        if room == 0 {
            break;
        }
        let text = if text.chars().count() <= room { text.clone() } else { text.chars().take(room.saturating_sub(1)).collect::<String>() + "…" };
        x = runs(&mut s, x, y, size, &[(&text, *color, *bold)]);
    }
    s
}
