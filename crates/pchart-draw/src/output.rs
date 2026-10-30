use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{self, Tree};

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
