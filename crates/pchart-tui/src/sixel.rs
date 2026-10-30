use std::fmt::Write;

use pchart_draw::Palette;
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
        let mut colors: Vec<[u8; 3]> = Vec::new();
        let mut add = |value: [u8; 3]| {
            if !colors.contains(&value) {
                colors.push(value);
            }
        };
        for color in palette.colors() {
            add(rgb(color));
        }
        for ground in [palette.page, palette.raised, palette.alert_ground] {
            for ink in palette.colors() {
                for step in 1..8 {
                    add(mix(ground, ink, f64::from(step) / 8.0));
                }
            }
        }
        colors.truncate(256);
        Encoder { colors, lookup: vec![0; 1 << 18], ground: rgb(palette.page) }
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
    fn one_band_per_six_rows() {
        let mut pixmap = Pixmap::new(13, 14).unwrap();
        pixmap.fill(resvg::tiny_skia::Color::from_rgba8(4, 7, 4, 255));
        let text = Encoder::new(&Palette::labra()).encode(&pixmap);
        assert!(text.starts_with("\x1bP0;1;0q\"1;1;13;14"));
        assert!(text.ends_with("\x1b\\"));
        assert_eq!(text.matches('-').count(), 2);
    }
}
