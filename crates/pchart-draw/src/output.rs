use resvg::tiny_skia::{Pixmap, Point, Transform};
use resvg::usvg::{self, Node, Paint, Tree};

use crate::style::{BOLD, FONT, REGULAR, SANS_BOLD, SANS_REGULAR};

/// Turns scene SVG into pictures and files, with the bundled fonts only, so every machine draws the same.
pub struct Output {
    options: usvg::Options<'static>,
}

impl Default for Output {
    fn default() -> Self {
        let mut options = usvg::Options { font_family: FONT.into(), ..usvg::Options::default() };
        let fonts = options.fontdb_mut();
        fonts.load_font_data(REGULAR.to_vec());
        fonts.load_font_data(BOLD.to_vec());
        fonts.load_font_data(SANS_REGULAR.to_vec());
        fonts.load_font_data(SANS_BOLD.to_vec());
        Output { options }
    }
}

impl Output {
    pub fn tree(&self, svg: &str) -> Result<Tree, String> {
        Tree::from_str(svg, &self.options).map_err(|err| err.to_string())
    }

    /// Texts that run over the edge of a filled shape, such as a name sticking out of its arrow or its
    /// box, measured on the glyphs as they are drawn: a text that touches a shape lies wholly inside it,
    /// with air around each glyph of a sixteenth of its height, and half the shape's edge line more. Shapes under the opaque ground a text
    /// stands on, such as the chart under a dialog, are hidden and do not count, and an opaque panel
    /// drawn later and larger than the text covers it, so only the part left in view counts.
    pub fn crossings(&self, svg: &str) -> Result<Vec<String>, String> {
        enum Drawn {
            Shape(Vec<Point>, bool, f32),
            Text(String, Vec<Point>),
        }
        fn points(path: &usvg::Path) -> Vec<Point> {
            let mut out: Vec<Point> = path.data().points().to_vec();
            path.abs_transform().map_points(&mut out);
            out
        }
        fn glyphs(group: &usvg::Group, out: &mut Vec<Point>) {
            for node in group.children() {
                match node {
                    Node::Group(group) => glyphs(group, out),
                    Node::Path(path) => out.extend(points(path)),
                    _ => {}
                }
            }
        }
        fn walk(group: &usvg::Group, opacity: f32, out: &mut Vec<Drawn>) {
            let opacity = opacity * group.opacity().get();
            for node in group.children() {
                match node {
                    Node::Group(group) => walk(group, opacity, out),
                    Node::Path(path) => {
                        if let Some(fill) = path.fill().filter(|fill| matches!(fill.paint(), Paint::Color(_)) && fill.opacity().get() > 0.0) {
                            let t = path.abs_transform();
                            let edge = path.stroke().map_or(0.0, |stroke| stroke.width().get() / 2.0 * (t.sx * t.sy - t.kx * t.ky).abs().sqrt());
                            out.push(Drawn::Shape(points(path), opacity * fill.opacity().get() >= 1.0, edge));
                        }
                    }
                    Node::Text(text) => {
                        let mut points = Vec::new();
                        glyphs(text.flattened(), &mut points);
                        text.abs_transform().map_points(&mut points);
                        out.push(Drawn::Text(text.chunks().iter().map(|chunk| chunk.text()).collect(), points));
                    }
                    _ => {}
                }
            }
        }
        let tree = self.tree(svg)?;
        let mut drawn = Vec::new();
        walk(tree.root(), 1.0, &mut drawn);
        let inside = |shape: &[Point], x: f32, y: f32| {
            let mut within = false;
            for (index, a) in shape.iter().enumerate() {
                let b = shape[(index + 1) % shape.len()];
                if (a.y > y) != (b.y > y) && x < a.x + (y - a.y) * (b.x - a.x) / (b.y - a.y) {
                    within = !within;
                }
            }
            within
        };
        let bounds = |points: &[Point]| points.iter().fold((f32::MAX, f32::MAX, f32::MIN, f32::MIN), |(x0, y0, x1, y1), point| (x0.min(point.x), y0.min(point.y), x1.max(point.x), y1.max(point.y)));
        let mut out = Vec::new();
        for (at, item) in drawn.iter().enumerate() {
            let Drawn::Text(text, glyphs) = item else { continue };
            let (left, top, right, bottom) = bounds(glyphs);
            let panels: Vec<&[Point]> = drawn[at + 1..]
                .iter()
                .filter_map(|item| match item {
                    Drawn::Shape(shape, true, _) => Some(shape.as_slice()),
                    _ => None,
                })
                .filter(|shape| {
                    let (x0, y0, x1, y1) = bounds(shape);
                    x1 - x0 > right - left && y1 - y0 > bottom - top
                })
                .collect();
            let shown: Vec<Point> = glyphs.iter().copied().filter(|point| !panels.iter().any(|panel| inside(panel, point.x, point.y))).collect();
            if shown.is_empty() {
                continue;
            }
            let air = (bottom - top) / 16.0;
            let ground = (0..at).rev().find(|&index| matches!(&drawn[index], Drawn::Shape(shape, true, _) if shown.iter().all(|point| inside(shape, point.x, point.y)))).unwrap_or(0);
            for item in &drawn[ground..] {
                let Drawn::Shape(shape, _, edge) = item else { continue };
                if panels.contains(&shape.as_slice()) {
                    continue;
                }
                let air = air + edge;
                if shown.iter().any(|point| inside(shape, point.x, point.y)) && !shown.iter().all(|point| [(0.0, 0.0), (air, 0.0), (-air, 0.0), (0.0, air), (0.0, -air)].iter().all(|(dx, dy)| inside(shape, point.x + dx, point.y + dy))) {
                    let (x0, y0, x1, y1) = bounds(shape);
                    out.push(format!("\"{text}\" at {left:.0},{top:.0} – {right:.0},{bottom:.0} runs over the edge of the shape at {x0:.0},{y0:.0} – {x1:.0},{y1:.0}"));
                }
            }
        }
        Ok(out)
    }

    /// Draws the tree into a picture of `width` × `height` pixels.
    pub fn pixmap(&self, tree: &Tree, width: u32, height: u32) -> Option<Pixmap> {
        let mut pixmap = Pixmap::new(width, height)?;
        let size = tree.size();
        resvg::render(tree, Transform::from_scale(width as f32 / size.width(), height as f32 / size.height()), &mut pixmap.as_mut());
        Some(pixmap)
    }

    pub fn png(&self, tree: &Tree, scale: f32) -> Result<Vec<u8>, String> {
        let size = tree.size();
        let pixmap = self.pixmap(tree, (size.width() * scale).round() as u32, (size.height() * scale).round() as u32).ok_or("the picture has no size")?;
        pixmap.encode_png().map_err(|err| err.to_string())
    }

    /// One PDF page the size of the tree, in points.
    pub fn pdf(&self, tree: &Tree) -> Result<Vec<u8>, String> {
        use krilla_svg::{SurfaceExt, SvgSettings};
        let size = tree.size();
        let mut document = krilla::Document::new();
        let settings = krilla::page::PageSettings::from_wh(size.width(), size.height()).ok_or("the page has no size")?;
        let mut page = document.start_page_with(settings);
        let mut surface = page.surface();
        let page_size = krilla::geom::Size::from_wh(size.width(), size.height()).ok_or("the page has no size")?;
        surface.draw_svg(tree, page_size, SvgSettings::default()).ok_or("the drawing could not be written as PDF")?;
        surface.finish();
        page.finish();
        document.finish().map_err(|err| format!("{err:?}"))
    }
}
