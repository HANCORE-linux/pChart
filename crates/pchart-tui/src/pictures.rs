use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use image::{DynamicImage, RgbaImage};
use pchart_draw::{Output, Palette};
use ratatui::Frame;
use ratatui::layout::{Rect, Size};
use ratatui_image::picker::{Capability, Picker, ProtocolType};
use ratatui_image::protocol::Protocol;
use ratatui_image::protocol::kitty::Kitty;
use ratatui_image::protocol::sixel::Sixel;
use ratatui_image::Image;
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
}

impl Painter {
    pub fn new(picker: Picker, palette: &Palette) -> Self {
        Painter { picker, output: Output::default(), sixel: Encoder::new(palette) }
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

    /// Paints `svg` for `rect`. A kitty image keeps the `id` of its place, so each new picture replaces
    /// the last one there in place.
    pub fn paint(&mut self, svg: &str, rect: Rect, id: u32) -> Option<(Protocol, Cost)> {
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
            Protocol::Kitty(Kitty::new(image, size, id, self.picker.tmux_detected(), compress).ok()?)
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

/// A picture in a place on the screen.
pub struct Placement {
    pub rect: Rect,
    pub svg: String,
}

struct Job {
    rect: Rect,
    key: u64,
    id: u32,
    svg: String,
}

struct Done {
    rect: Rect,
    key: u64,
    protocol: Option<Protocol>,
}

/// The pictures of the screen, painted off the UI thread. Each place shows its last picture until
/// the new one is ready; while one is being painted, newer ones for the same place replace each
/// other, so a held key never queues up stale pictures.
pub struct Pictures {
    cell: (u16, u16),
    slots: HashMap<Rect, (u64, Protocol)>,
    pending: HashMap<Rect, u64>,
    waiting: HashMap<Rect, Job>,
    ids: HashMap<Rect, u32>,
    free: Vec<u32>,
    next: u32,
    jobs: Sender<Job>,
    done: Receiver<Done>,
}

impl Pictures {
    pub fn start(mut painter: Painter) -> Self {
        let cell = painter.cell();
        let (jobs, inbox) = mpsc::channel::<Job>();
        let (outbox, done) = mpsc::channel();
        thread::spawn(move || {
            for job in inbox {
                let protocol = painter.paint(&job.svg, job.rect, job.id).map(|(protocol, _)| protocol);
                if outbox.send(Done { rect: job.rect, key: job.key, protocol }).is_err() {
                    return;
                }
            }
        });
        Pictures { cell, slots: HashMap::new(), pending: HashMap::new(), waiting: HashMap::new(), ids: HashMap::new(), free: Vec::new(), next: std::process::id() << 8, jobs, done }
    }

    pub fn cell(&self) -> (u16, u16) {
        self.cell
    }

    /// Takes in finished pictures; true when one arrived.
    pub fn receive(&mut self) -> bool {
        let mut any = false;
        while let Ok(done) = self.done.try_recv() {
            any = true;
            self.pending.remove(&done.rect);
            if let Some(protocol) = done.protocol {
                self.slots.insert(done.rect, (done.key, protocol));
            }
            if let Some(job) = self.waiting.remove(&done.rect) {
                self.send(job);
            }
        }
        any
    }

    /// Shows `placements`, asking for the pictures that changed. Returns true while some are still
    /// being painted.
    pub fn draw(&mut self, frame: &mut Frame, placements: Vec<Placement>) -> bool {
        let rects: Vec<Rect> = placements.iter().map(|placement| placement.rect).collect();
        for placement in placements {
            let key = fingerprint(&placement.svg);
            if self.slots.get(&placement.rect).is_some_and(|(shown, _)| *shown == key) || self.pending.get(&placement.rect) == Some(&key) {
                self.waiting.remove(&placement.rect);
                continue;
            }
            let job = Job { rect: placement.rect, key, id: self.id(placement.rect), svg: placement.svg };
            if self.pending.contains_key(&placement.rect) {
                self.waiting.insert(placement.rect, job);
            } else {
                self.send(job);
            }
        }
        for rect in &rects {
            if let Some((_, protocol)) = self.slots.get(rect) {
                frame.render_widget(Image::new(protocol), *rect);
            }
        }
        self.slots.retain(|rect, _| rects.contains(rect));
        self.waiting.retain(|rect, _| rects.contains(rect));
        let (pending, free) = (&self.pending, &mut self.free);
        self.ids.retain(|rect, id| {
            let keep = rects.contains(rect) || pending.contains_key(rect);
            if !keep {
                free.push(*id);
            }
            keep
        });
        rects.iter().any(|rect| self.pending.contains_key(rect) || !self.slots.contains_key(rect))
    }

    fn send(&mut self, job: Job) {
        let (rect, key) = (job.rect, job.key);
        if self.jobs.send(job).is_ok() {
            self.pending.insert(rect, key);
        }
    }

    fn id(&mut self, rect: Rect) -> u32 {
        if let Some(id) = self.ids.get(&rect) {
            return *id;
        }
        let id = self.free.pop().unwrap_or_else(|| {
            self.next += 1;
            self.next
        });
        self.ids.insert(rect, id);
        id
    }
}

fn fingerprint(svg: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    svg.hash(&mut hasher);
    hasher.finish()
}
