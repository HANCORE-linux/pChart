use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use image::{DynamicImage, RgbaImage};
use pchart_draw::{Output, Palette};
use ratatui::layout::{Rect, Size};
use ratatui_image::picker::{Capability, Picker, ProtocolType};
use ratatui_image::protocol::Protocol;
use ratatui_image::protocol::kitty::Kitty;
use ratatui_image::protocol::sixel::Sixel;
use resvg::tiny_skia::Pixmap;

use crate::sixel::Encoder;

/// Time spent on one picture, step by step.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cost {
    pub parse: Duration,
    pub raster: Duration,
    pub encode: Duration,
}

/// Turns SVG into a terminal picture: kitty graphics in kitty and Ghostty, sixel in foot.
pub struct Painter {
    picker: Picker,
    output: Output,
    sixel: Encoder,
    id: u32,
}

impl Painter {
    pub fn new(picker: Picker, palette: &Palette) -> Self {
        Painter { picker, output: Output::default(), sixel: Encoder::new(palette), id: std::process::id() << 8 }
    }

    pub fn cell(&self) -> (u16, u16) {
        let font = self.picker.font_size();
        (font.width, font.height)
    }

    pub fn protocol_name(&self) -> &'static str {
        match self.picker.protocol_type() {
            ProtocolType::Kitty => "kitty graphics",
            ProtocolType::Sixel => "sixel",
            ProtocolType::Iterm2 => "iterm2",
            ProtocolType::Halfblocks => "halfblocks",
        }
    }

    /// Paints `svg` for `rect`. The kitty image keeps one id, so each new picture replaces the last in place.
    pub fn paint(&mut self, svg: &str, rect: Rect) -> Option<(Protocol, Cost)> {
        let (cell_w, cell_h) = self.cell();
        let (width, height) = (u32::from(rect.width) * u32::from(cell_w), u32::from(rect.height) * u32::from(cell_h));
        let started = Instant::now();
        let tree = self.output.tree(svg).ok()?;
        let parse = started.elapsed();
        let started = Instant::now();
        let pixmap = self.output.pixmap(&tree, width, height)?;
        let raster = started.elapsed();
        let started = Instant::now();
        let size = Size::new(rect.width, rect.height);
        let protocol = if self.picker.protocol_type() == ProtocolType::Sixel {
            Protocol::Sixel(Sixel { data: sixel(&mut self.sixel, &pixmap, rect), size, is_tmux: false })
        } else {
            let image = DynamicImage::ImageRgba8(rgba(&pixmap)?);
            let compress = self.picker.capabilities().contains(&Capability::KittyCompression);
            Protocol::Kitty(Kitty::new(image, size, self.id, self.picker.tmux_detected(), compress).ok()?)
        };
        Some((protocol, Cost { parse, raster, encode: started.elapsed() }))
    }
}

fn rgba(pixmap: &Pixmap) -> Option<RgbaImage> {
    let pixels = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let color = pixel.demultiply();
            [color.red(), color.green(), color.blue(), color.alpha()]
        })
        .collect();
    RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixels)
}

fn sixel(encoder: &mut Encoder, pixmap: &Pixmap, rect: Rect) -> String {
    let mut data = String::new();
    for _ in 0..rect.height {
        data.push_str(&format!("\x1b[{}X\x1b[1B", rect.width));
    }
    data.push_str(&format!("\x1b[{}A", rect.height));
    data.push_str(&encoder.encode(pixmap));
    data
}

pub struct Job {
    pub generation: u64,
    pub rect: Rect,
    pub svg: String,
}

pub struct Done {
    pub generation: u64,
    pub rect: Rect,
    pub protocol: Option<Protocol>,
    pub cost: Cost,
}

/// Paints off the UI thread. While a picture is being painted, newer requests replace each other,
/// so a held key never queues up stale pictures.
pub struct Worker {
    jobs: Sender<Job>,
    done: Receiver<Done>,
    busy: bool,
    waiting: Option<Job>,
}

impl Worker {
    pub fn start(mut painter: Painter) -> Self {
        let (jobs, inbox) = mpsc::channel::<Job>();
        let (outbox, done) = mpsc::channel();
        thread::spawn(move || {
            for job in inbox {
                let painted = painter.paint(&job.svg, job.rect);
                let (protocol, cost) = painted.map_or((None, Cost::default()), |(protocol, cost)| (Some(protocol), cost));
                if outbox.send(Done { generation: job.generation, rect: job.rect, protocol, cost }).is_err() {
                    return;
                }
            }
        });
        Worker { jobs, done, busy: false, waiting: None }
    }

    pub fn request(&mut self, job: Job) {
        if self.busy {
            self.waiting = Some(job);
        } else if self.jobs.send(job).is_ok() {
            self.busy = true;
        }
    }

    pub fn finished(&mut self) -> Option<Done> {
        let done = self.done.try_recv().ok()?;
        self.busy = false;
        if let Some(job) = self.waiting.take() {
            self.request(job);
        }
        Some(done)
    }
}
