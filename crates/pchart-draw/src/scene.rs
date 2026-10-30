use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    Color(Color),
    Hatch,
    HatchAlert,
    /// Thin diagonal lines over the background: time an item has to spare.
    Spare,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
    pub dash: Option<(f32, f32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Rect { x: f32, y: f32, w: f32, h: f32, radius: f32, fill: Fill, stroke: Option<Stroke> },
    Line { x1: f32, y1: f32, x2: f32, y2: f32, stroke: Stroke },
    Path { points: Vec<(f32, f32)>, closed: bool, fill: Fill, stroke: Option<Stroke> },
    Circle { x: f32, y: f32, r: f32, fill: Color },
    Text { x: f32, y: f32, text: String, size: f32, color: Color, bold: bool, anchor: Anchor },
}

/// A shape, its opacity, and the plan item it shows, so a click can find the item.
#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    pub shape: Shape,
    pub opacity: f32,
    pub item: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub width: f32,
    pub height: f32,
    pub background: Color,
    pub hatch: (Color, Color),
    pub hatch_alert: (Color, Color),
    pub spare: Color,
    pub font: &'static str,
    pub marks: Vec<Mark>,
}

impl Scene {
    pub fn add(&mut self, shape: Shape) {
        self.marks.push(Mark { shape, opacity: 1.0, item: None });
    }

    pub fn add_for(&mut self, item: usize, opacity: f32, shape: Shape) {
        self.marks.push(Mark { shape, opacity, item: Some(item) });
    }

    pub fn svg(&self) -> String {
        let mut out = String::with_capacity(64 * self.marks.len() + 512);
        let (w, h) = (self.width, self.height);
        let _ = write!(out, r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="{}">"#, self.font);
        let _ = write!(out, r#"<rect width="{w}" height="{h}" fill="{}"/>"#, self.background.hex());
        let pattern = |out: &mut String, id: &str, size: f32, (ground, ink): (Color, Color), width: f32| {
            let _ = write!(
                out,
                r#"<pattern id="{id}" width="{size}" height="{size}" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="{size}" height="{size}" fill="{}"/><line x1="0" y1="0" x2="0" y2="{size}" stroke="{}" stroke-width="{width}"/></pattern>"#,
                ground.hex(),
                ink.hex()
            );
        };
        out.push_str("<defs>");
        pattern(&mut out, "hatch", 4.0, self.hatch, 1.6);
        pattern(&mut out, "alert", 5.0, self.hatch_alert, 2.0);
        let _ = write!(
            out,
            r#"<pattern id="spare" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><line x1="0" y1="0" x2="0" y2="6" stroke="{}" stroke-width="1.4" opacity="0.55"/></pattern>"#,
            self.spare.hex()
        );
        out.push_str("</defs>");
        for mark in &self.marks {
            write_mark(&mut out, mark);
        }
        out.push_str("</svg>");
        out
    }

    /// The topmost item drawn at a point.
    pub fn item_at(&self, x: f32, y: f32) -> Option<usize> {
        self.marks.iter().rev().find(|mark| mark.item.is_some() && contains(&mark.shape, x, y)).and_then(|mark| mark.item)
    }
}

fn contains(shape: &Shape, x: f32, y: f32) -> bool {
    match shape {
        Shape::Rect { x: left, y: top, w, h, .. } => x >= *left && x <= left + w && y >= *top && y <= top + h,
        Shape::Path { points, closed: true, .. } => {
            let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for (px, py) in points {
                (min_x, min_y, max_x, max_y) = (min_x.min(*px), min_y.min(*py), max_x.max(*px), max_y.max(*py));
            }
            x >= min_x && x <= max_x && y >= min_y && y <= max_y
        }
        Shape::Text { x: left, y: base, text, size, anchor, .. } => {
            let width = text.chars().count() as f32 * size * 0.6;
            let left = match anchor {
                Anchor::Start => *left,
                Anchor::Middle => left - width / 2.0,
                Anchor::End => left - width,
            };
            x >= left && x <= left + width && y >= base - size && y <= base + size * 0.3
        }
        _ => false,
    }
}

fn fill(fill: Fill) -> String {
    match fill {
        Fill::Color(color) => color.hex(),
        Fill::Hatch => "url(#hatch)".into(),
        Fill::HatchAlert => "url(#alert)".into(),
        Fill::Spare => "url(#spare)".into(),
        Fill::None => "none".into(),
    }
}

fn stroke(out: &mut String, stroke: Option<Stroke>) {
    if let Some(Stroke { color, width, dash }) = stroke {
        let _ = write!(out, r#" stroke="{}" stroke-width="{width}""#, color.hex());
        if let Some((on, off)) = dash {
            let _ = write!(out, r#" stroke-dasharray="{on} {off}""#);
        }
    }
}

fn write_mark(out: &mut String, mark: &Mark) {
    let opacity = if mark.opacity < 1.0 { format!(r#" opacity="{}""#, mark.opacity) } else { String::new() };
    match &mark.shape {
        Shape::Rect { x, y, w, h, radius, fill: f, stroke: s } => {
            let _ = write!(out, r#"<rect x="{x:.1}" y="{y:.1}" width="{:.1}" height="{:.1}""#, w.max(0.0), h.max(0.0));
            if *radius > 0.0 {
                let _ = write!(out, r#" rx="{radius}""#);
            }
            let _ = write!(out, r#" fill="{}""#, fill(*f));
            stroke(out, *s);
            let _ = write!(out, "{opacity}/>");
        }
        Shape::Line { x1, y1, x2, y2, stroke: s } => {
            let _ = write!(out, r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}""#);
            stroke(out, Some(*s));
            let _ = write!(out, "{opacity}/>");
        }
        Shape::Path { points, closed, fill: f, stroke: s } => {
            out.push_str(r#"<path d=""#);
            for (index, (x, y)) in points.iter().enumerate() {
                let _ = write!(out, "{}{x:.1},{y:.1}", if index == 0 { 'M' } else { 'L' });
            }
            if *closed {
                out.push('Z');
            }
            let _ = write!(out, r#"" fill="{}""#, fill(*f));
            stroke(out, *s);
            let _ = write!(out, "{opacity}/>");
        }
        Shape::Circle { x, y, r, fill: color } => {
            let _ = write!(out, r#"<circle cx="{x:.1}" cy="{y:.1}" r="{r}" fill="{}"{opacity}/>"#, color.hex());
        }
        Shape::Text { x, y, text, size, color, bold, anchor } => {
            let anchor = match anchor {
                Anchor::Start => "",
                Anchor::Middle => r#" text-anchor="middle""#,
                Anchor::End => r#" text-anchor="end""#,
            };
            let weight = if *bold { r#" font-weight="700""# } else { "" };
            let _ = write!(out, r#"<text x="{x:.1}" y="{y:.1}" font-size="{size:.1}" fill="{}"{anchor}{weight}{opacity} xml:space="preserve">"#, color.hex());
            for c in text.chars() {
                match c {
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    '&' => out.push_str("&amp;"),
                    _ => out.push(c),
                }
            }
            out.push_str("</text>");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene() -> Scene {
        let ink = Color(1, 2, 3);
        Scene { width: 100.0, height: 50.0, background: ink, hatch: (ink, ink), hatch_alert: (ink, ink), spare: ink, font: "JetBrains Mono", marks: Vec::new() }
    }

    #[test]
    fn text_is_escaped() {
        let mut s = scene();
        s.add(Shape::Text { x: 1.0, y: 2.0, text: "R&D <beta>".into(), size: 10.0, color: Color(0, 0, 0), bold: false, anchor: Anchor::Start });
        assert!(s.svg().contains(">R&amp;D &lt;beta&gt;</text>"));
    }

    #[test]
    fn a_point_finds_the_topmost_item() {
        let mut s = scene();
        let rect = |x| Shape::Rect { x, y: 10.0, w: 20.0, h: 10.0, radius: 0.0, fill: Fill::None, stroke: None };
        s.add_for(3, 1.0, rect(10.0));
        s.add_for(4, 1.0, rect(20.0));
        assert_eq!(s.item_at(15.0, 15.0), Some(3));
        assert_eq!(s.item_at(25.0, 15.0), Some(4));
        assert_eq!(s.item_at(60.0, 15.0), None);
    }
}
