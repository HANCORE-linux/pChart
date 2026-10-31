use std::fmt::Write;

use pchart_draw::Palette;
use pchart_draw::page::{CARD, FIXED, GRAY, INK, LATE, LIGHT, LINK, MID, PAPER, SHADES, SOFT, WARN, WARN_LINE};
use pchart_draw::scene::Color;
use resvg::tiny_skia::Pixmap;

/// Sixel with a fixed palette built from the drawing colours, so no picture needs quantising or dithering.
pub struct Encoder {
    colors: Vec<[u8; 3]>,
    lookup: Vec<u16>,
    /// What half transparent pixels are mixed with: the page of the palette.
    ground: [u8; 3],
}

fn rgb(color: Color) -> [u8; 3] {
    [color.0, color.1, color.2]
}

fn mix(a: Color, b: Color, amount: f64) -> [u8; 3] {
    let blend = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * amount).round() as u8;
    [blend(a.0, b.0), blend(a.1, b.1), blend(a.2, b.2)]
}

impl Encoder {
    pub fn new(palette: &Palette) -> Self {
        let colors = palette.colors();
        Encoder::of(&colors, &[(palette.page, &colors), (palette.raised, &colors), (palette.alert_ground, &colors)], palette.page)
    }

    /// For a light export page on the dark screen: its colours, their edges on paper, cards, phases and
    /// tags, and the page and its shadow over the dark ground.
    pub fn paper(palette: &Palette) -> Self {
        let colors = [PAPER, INK, GRAY, LIGHT, MID, SOFT, LINK, CARD, SHADES[0], SHADES[1], SHADES[2], SHADES[3], FIXED, LATE, WARN, WARN_LINE, palette.page, Color(0, 0, 0)];
        let words = [INK, GRAY];
        let mut grounds: Vec<(Color, &[Color])> = vec![(PAPER, &colors[..16]), (CARD, &[INK, GRAY, MID, FIXED, LATE]), (WARN, &[INK]), (palette.page, &[PAPER, Color(0, 0, 0)])];
        grounds.extend(SHADES.iter().map(|&shade| (shade, &words[..])));
        Encoder::of(&colors, &grounds, palette.page)
    }

    /// `colors`, and each ink of `grounds` blended onto its ground in seven steps; half transparent
    /// pixels are mixed with `ground` first.
    fn of(colors: &[Color], grounds: &[(Color, &[Color])], ground: Color) -> Self {
        let mut out: Vec<[u8; 3]> = Vec::new();
        let mut add = |value: [u8; 3]| {
            if !out.contains(&value) {
                out.push(value);
            }
        };
        for &color in colors {
            add(rgb(color));
        }
        for &(under, inks) in grounds {
            for &ink in inks {
                for step in 1..8 {
                    add(mix(under, ink, f64::from(step) / 8.0));
                }
            }
        }
        out.truncate(256);
        Encoder { colors: out, lookup: vec![0; 1 << 18], ground: rgb(ground) }
    }

    fn index(&mut self, r: u8, g: u8, b: u8) -> u8 {
        let cell = (usize::from(r >> 2) << 12) | (usize::from(g >> 2) << 6) | usize::from(b >> 2);
        let known = self.lookup[cell];
        if known != 0 {
            return (known - 1) as u8;
        }
        let (r, g, b) = (i32::from(r & !3) + 2, i32::from(g & !3) + 2, i32::from(b & !3) + 2);
        let mut best = (0usize, i32::MAX);
        for (index, [pr, pg, pb]) in self.colors.iter().enumerate() {
            let (dr, dg, db) = (r - i32::from(*pr), g - i32::from(*pg), b - i32::from(*pb));
            let distance = 3 * dr * dr + 4 * dg * dg + 2 * db * db;
            if distance < best.1 {
                best = (index, distance);
            }
        }
        self.lookup[cell] = best.0 as u16 + 1;
        best.0 as u8
    }

    pub fn encode(&mut self, pixmap: &Pixmap) -> String {
        let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
        let mut indices = vec![0u8; width * height];
        let mut clear = vec![false; width * height];
        for ((slot, clear), pixel) in indices.iter_mut().zip(clear.iter_mut()).zip(pixmap.pixels()) {
            if pixel.alpha() < 24 {
                *clear = true;
                continue;
            }
            let rest = 255 - u16::from(pixel.alpha());
            let over = |value: u8, ground: u8| (u16::from(value) + (u16::from(ground) * rest + 127) / 255).min(255) as u8;
            *slot = self.index(over(pixel.red(), self.ground[0]), over(pixel.green(), self.ground[1]), over(pixel.blue(), self.ground[2]));
        }
        let mut used = [false; 256];
        for (index, clear) in indices.iter().zip(&clear) {
            if !clear {
                used[usize::from(*index)] = true;
            }
        }
        let mut out = String::with_capacity(width * height / 2);
        let _ = write!(out, "\x1bP0;1;0q\"1;1;{width};{height}");
        for (index, [r, g, b]) in self.colors.iter().enumerate() {
            if used[index] {
                let percent = |value: u8| (u32::from(value) * 100 + 127) / 255;
                let _ = write!(out, "#{index};2;{};{};{}", percent(*r), percent(*g), percent(*b));
            }
        }
        let mut bits = vec![0u8; 256 * width];
        let mut present: Vec<usize> = Vec::with_capacity(64);
        let mut seen = [false; 256];
        let mut spans = [(0usize, 0usize); 256];
        for band in (0..height).step_by(6) {
            if band > 0 {
                out.push('-');
            }
            for color in present.drain(..) {
                seen[color] = false;
            }
            for row in 0..6.min(height - band) {
                let line = &indices[(band + row) * width..(band + row + 1) * width];
                let holes = &clear[(band + row) * width..(band + row + 1) * width];
                for (x, index) in line.iter().enumerate() {
                    if holes[x] {
                        continue;
                    }
                    let color = usize::from(*index);
                    if seen[color] {
                        let span = &mut spans[color];
                        span.0 = span.0.min(x);
                        span.1 = span.1.max(x);
                    } else {
                        seen[color] = true;
                        present.push(color);
                        spans[color] = (x, x);
                    }
                    bits[color * width + x] |= 1 << row;
                }
            }
            for (number, color) in present.iter().enumerate() {
                if number > 0 {
                    out.push('$');
                }
                let _ = write!(out, "#{color}");
                let (first, last) = spans[*color];
                let row = &mut bits[color * width..(color + 1) * width];
                run(&mut out, first, 0);
                let mut x = first;
                while x <= last {
                    let value = row[x];
                    let mut count = 1;
                    while x + count <= last && row[x + count] == value {
                        count += 1;
                    }
                    run(&mut out, count, value);
                    x += count;
                }
                row[first..=last].fill(0);
            }
        }
        out.push_str("\x1b\\");
        out
    }
}

fn run(out: &mut String, count: usize, value: u8) {
    let symbol = char::from(63 + value);
    if count > 3 {
        let _ = write!(out, "!{count}{symbol}");
    } else {
        for _ in 0..count {
            out.push(symbol);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_drawing_colours_are_in_the_palette() {
        let palette = Palette::labra();
        let mut encoder = Encoder::new(&palette);
        assert!(encoder.colors.len() <= 256);
        for color in palette.colors() {
            let index = encoder.index(color.0, color.1, color.2);
            let [r, g, b] = encoder.colors[usize::from(index)];
            assert!(r.abs_diff(color.0) <= 3 && g.abs_diff(color.1) <= 3 && b.abs_diff(color.2) <= 3, "{color:?}");
        }
    }

    #[test]
    fn a_light_page_keeps_its_colours() {
        let mut encoder = Encoder::paper(&Palette::labra());
        assert!(encoder.colors.len() < 256, "{}", encoder.colors.len());
        for color in [PAPER, INK, GRAY, LIGHT, MID, SOFT, LINK, CARD, FIXED, LATE, WARN, WARN_LINE].into_iter().chain(SHADES) {
            let index = encoder.index(color.0, color.1, color.2);
            let [r, g, b] = encoder.colors[usize::from(index)];
            assert!(r.abs_diff(color.0) <= 3 && g.abs_diff(color.1) <= 3 && b.abs_diff(color.2) <= 3, "{color:?}");
        }
    }

    #[test]
    fn one_band_per_six_rows() {
        let mut pixmap = Pixmap::new(13, 14).unwrap();
        pixmap.fill(resvg::tiny_skia::Color::from_rgba8(4, 7, 4, 255));
        let text = Encoder::new(&Palette::labra()).encode(&pixmap);
        assert!(text.starts_with("\x1bP0;1;0q\"1;1;13;14"));
        assert!(text.ends_with("\x1b\\"));
        assert_eq!(text.matches('-').count(), 2);
    }
}
