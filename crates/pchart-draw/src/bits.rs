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
