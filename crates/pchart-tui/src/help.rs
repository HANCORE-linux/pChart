//! The help of each screen (`?`, picture 50), over the dimmed screen: what the screen answers, what can
//! be done on it with which key, and what its symbols mean.

use pchart_draw::bits::{self, Icon};
use pchart_draw::scene::{Fill, Mark, Scene, Shape, Stroke};
use pchart_draw::style::ADVANCE;
use pchart_draw::Palette;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Clear;

use crate::app::{App, Screen};
use crate::pictures::Placement;

/// A symbol of the legend, drawn as the screen draws it.
#[derive(Clone, Copy)]
enum Sign {
    Chain,
    Spare,
    Late,
    Milestone,
    Important,
    Fixed,
    Today,
    Target,
    Phase,
    Holiday,
    Icon(Icon, Tone),
    Chip(bool),
}

#[derive(Clone, Copy)]
enum Tone {
    Bright,
    Ink,
    Dim,
    Warn,
    Alert,
}

struct Help {
    title: &'static str,
    lead: &'static str,
    keys: Vec<(&'static str, &'static str)>,
    legend_title: &'static str,
    legend: Vec<(Sign, &'static str)>,
}

fn help(screen: Screen) -> Help {
    let everywhere = [("u", "Undo the last change"), ("Ctrl+P", "Find any action by typing"), ("1–4", "Situation, Roadmap, Checks, Export"), ("q", "Quit; every change is saved already")];
    match screen {
        Screen::Roadmap => Help {
            title: "HELP · ROADMAP",
            lead: "When each piece of work happens, and which ones decide the date.",
            keys: [("↑↓", "Choose an item"), ("←→", "Try it longer or shorter, on trial"), ("Enter", "Change when it starts"), ("a", "Add an item after it"), ("r", "Rename it"), ("w", "Choose what it waits on"), ("x", "Remove it"), ("Space", "Show or hide the details"), ("n", "Add notes; i marks a milestone important"), ("d", "Overview, Standard or Full")].into_iter().chain(everywhere).collect(),
            legend_title: "WHAT THE PICTURE SHOWS",
            legend: vec![
                (Sign::Chain, "decides the date"),
                (Sign::Spare, "has time to spare"),
                (Sign::Late, "late"),
                (Sign::Milestone, "a milestone"),
                (Sign::Important, "an important milestone"),
                (Sign::Fixed, "a fixed date"),
                (Sign::Today, "today"),
                (Sign::Target, "the target date"),
                (Sign::Phase, "a phase"),
                (Sign::Icon(Icon::Warn, Tone::Warn), "something to look at"),
                (Sign::Holiday, "a holiday or day off, in Full"),
            ],
        },
        Screen::Situation => Help {
            title: "HELP · SITUATION",
            lead: "Where the plan stands, what comes next, and which chain decides the date.",
            keys: [("↑↓", "Choose a step"), ("Enter", "Show it in the roadmap")].into_iter().chain(everywhere).collect(),
            legend_title: "WHAT THE SYMBOLS SAY",
            legend: vec![
                (Sign::Icon(Icon::Check, Tone::Bright), "on track"),
                (Sign::Icon(Icon::Clock, Tone::Warn), "at risk: under 5 working days to spare"),
                (Sign::Icon(Icon::Cross, Tone::Alert), "late"),
                (Sign::Icon(Icon::Clock, Tone::Dim), "when something is due"),
                (Sign::Icon(Icon::Person, Tone::Dim), "the team that does it"),
                (Sign::Icon(Icon::Warn, Tone::Warn), "something to look at"),
                (Sign::Milestone, "a milestone in the chain"),
            ],
        },
        Screen::Checks => Help {
            title: "HELP · CHECKS",
            lead: "What to look at in the plan, what it means, and what you can do about it.",
            keys: [("↑↓", "Choose a check"), ("Enter", "Show it in the roadmap")].into_iter().chain(everywhere).collect(),
            legend_title: "WHAT THE SYMBOLS SAY",
            legend: vec![
                (Sign::Icon(Icon::Cross, Tone::Alert), "a problem: the plan cannot hold"),
                (Sign::Icon(Icon::Warn, Tone::Warn), "a warning: look at it"),
                (Sign::Icon(Icon::Info, Tone::Dim), "a hint"),
                (Sign::Icon(Icon::Check, Tone::Dim), "checked, nothing found"),
                (Sign::Icon(Icon::Calendar, Tone::Dim), "the holidays the plan accounts for"),
                (Sign::Icon(Icon::Play, Tone::Ink), "what you can do"),
            ],
        },
        Screen::Export => Help {
            title: "HELP · EXPORT",
            lead: "A page for others: choose a view and who reads it. The preview is the file.",
            keys: [("↑↓", "View, audience or formats"), ("←→", "Change it"), ("Space", "Switch a format on or off"), ("Enter", "Write the files"), ("Esc", "Back to the situation")].into_iter().chain(everywhere).collect(),
            legend_title: "WHAT THE SYMBOLS SAY",
            legend: vec![(Sign::Chip(true), "chosen, or on"), (Sign::Chip(false), "off"), (Sign::Icon(Icon::Folder, Tone::Dim), "where the files go"), (Sign::Icon(Icon::Check, Tone::Bright), "written")],
        },
    }
}

/// The frame of the help and the finder (as the note of picture 40): the page, a thin rule and brighter
/// corners.
pub(crate) fn frame(s: &mut Scene, width: f32, height: f32, u: f32, p: &Palette) {
    s.add(Shape::Rect { x: 0.5 * u, y: 0.5 * u, w: width - u, h: height - u, radius: 0.0, fill: Fill::Color(p.page), stroke: Some(Stroke { color: p.rule, width: u, dash: None }) });
    let corner = 16.0 * u;
    let (x0, y0, x1, y1) = (u, u, width - u, height - u);
    for (cx, cy, dx, dy) in [(x0, y0, 1.0, 1.0), (x1, y0, -1.0, 1.0), (x0, y1, 1.0, -1.0), (x1, y1, -1.0, -1.0)] {
        s.add(Shape::Path { points: vec![(cx + dx * corner, cy), (cx, cy), (cx, cy + dy * corner)], closed: false, fill: Fill::None, stroke: Some(Stroke { color: p.ink, width: 2.0 * u, dash: None }) });
    }
}

/// The place of an overlay `width` × `height` cells, centred in the screen between the title and the
/// key bar.
pub(crate) fn place(area: Rect, width: u16, height: u16, top: Option<u16>) -> Rect {
    let width = width.min(area.width.saturating_sub(2));
    let height = height.min(area.height.saturating_sub(2));
    let y = top.unwrap_or((area.height.saturating_sub(height)) / 2).max(1).min(area.height.saturating_sub(1 + height));
    Rect { x: (area.width - width) / 2, y, width, height }
}

fn tone(p: &Palette, tone: Tone) -> pchart_draw::scene::Color {
    match tone {
        Tone::Bright => p.bright,
        Tone::Ink => p.ink,
        Tone::Dim => p.dim,
        Tone::Warn => p.warn,
        Tone::Alert => p.alert,
    }
}

fn sign(s: &mut Scene, sign: Sign, x: f32, y: f32, u: f32, p: &Palette) {
    let bar = |s: &mut Scene, color, opacity| s.marks.push(Mark { shape: Shape::Rect { x, y: y - 6.0 * u, w: 30.0 * u, h: 12.0 * u, radius: 3.0 * u, fill: Fill::Color(color), stroke: None }, opacity, item: None });
    let diamond = |r: f32, cx: f32, cy: f32| vec![(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)];
    let cx = x + 15.0 * u;
    match sign {
        Sign::Chain => bar(s, p.ink, 0.95),
        Sign::Spare => bar(s, p.dim, 0.5),
        Sign::Late => bar(s, p.alert, 0.92),
        Sign::Milestone => s.add(Shape::Path { points: diamond(7.0 * u, cx, y), closed: true, fill: Fill::Color(p.accent), stroke: None }),
        Sign::Important => s.add(Shape::Path { points: bits::star(cx, y, 10.0 * u), closed: true, fill: Fill::Color(p.accent), stroke: None }),
        Sign::Fixed => {
            s.add(Shape::Path { points: diamond(8.0 * u, cx, y + 3.0 * u), closed: true, fill: Fill::Color(p.accent), stroke: None });
            s.add(Shape::Line { x1: cx, y1: y - 7.0 * u, x2: cx, y2: y - 14.0 * u, stroke: Stroke { color: p.accent, width: 1.6 * u, dash: None } });
            s.add(Shape::Circle { x: cx, y: y - 17.0 * u, r: 3.5 * u, fill: p.accent });
        }
        Sign::Today => s.add(Shape::Line { x1: cx, y1: y - 11.0 * u, x2: cx, y2: y + 11.0 * u, stroke: Stroke { color: p.ink, width: 1.5 * u, dash: None } }),
        Sign::Target => s.add(Shape::Line { x1: cx, y1: y - 11.0 * u, x2: cx, y2: y + 11.0 * u, stroke: Stroke { color: p.bright, width: u, dash: Some((4.0 * u, 4.0 * u)) } }),
        Sign::Phase => {
            let (x0, x1, h, tip) = (x, x + 32.0 * u, 14.0 * u, 6.0 * u);
            let top = y - h / 2.0;
            s.add(Shape::Path { points: vec![(x0, top), (x1 - tip, top), (x1, y), (x1 - tip, top + h), (x0, top + h), (x0 + tip, y)], closed: true, fill: Fill::Color(p.raised), stroke: Some(Stroke { color: p.muted, width: u, dash: None }) });
        }
        Sign::Holiday => s.add(Shape::Rect { x: cx - 6.0 * u, y: y - 11.0 * u, w: 12.0 * u, h: 22.0 * u, radius: 0.0, fill: Fill::Hatch, stroke: None }),
        Sign::Icon(icon, t) => bits::draw_icon(s, icon, cx, y, 17.0 * u, tone(p, t), p.page),
        Sign::Chip(on) => {
            s.add(Shape::Rect { x, y: y - 9.0 * u, w: 30.0 * u, h: 18.0 * u, radius: 4.0 * u, fill: if on { Fill::Color(p.ink) } else { Fill::None }, stroke: (!on).then_some(Stroke { color: p.muted, width: 1.2 * u, dash: None }) });
        }
    }
}

/// Draws the help of this screen over it.
pub fn draw(app: &App, frame: &mut Frame, area: Rect, placements: &mut Vec<Placement>) {
    let h = help(app.screen);
    let p = app.palette;
    let (cw, ch) = (f32::from(app.cell.0), f32::from(app.cell.1));
    let u = ch / 21.0;
    let width = area.width.saturating_sub(8).min(132);
    let two = width >= 100;
    let step = 32.0 * u;
    let key_w = 9.0 * 16.6 * u * ADVANCE;
    let rows = if two { h.keys.len().max(h.legend.len()) } else { h.keys.len() + 2 + h.legend.len() };
    let wanted = 150.0 * u + rows as f32 * step + 12.0 * u;
    let rect = place(area, width, (wanted / ch).ceil() as u16, None);
    let (w, hpx) = (f32::from(rect.width) * cw, f32::from(rect.height) * ch);
    let mut s = bits::canvas(w, hpx, &p);
    self::frame(&mut s, w, hpx, u, &p);
    let left = 26.0 * u;
    let room = w - 52.0 * u;
    bits::runs(&mut s, 20.0 * u, 29.0 * u, 16.6 * u, &[(&format!(":: {}", h.title), p.dim, false)]);
    let size = (24.0 * u).min(room / (h.lead.chars().count() as f32 * ADVANCE));
    bits::runs(&mut s, left, 74.0 * u, size, &[(h.lead, p.bright, true)]);

    let column = if two { (room - 40.0 * u) / 2.0 } else { room };
    let fit = |text: &str, size: f32, room: f32| {
        let count = (room / (size * ADVANCE)).floor() as usize;
        if text.chars().count() <= count { text.to_string() } else { text.chars().take(count.saturating_sub(1)).collect::<String>() + "…" }
    };
    let top = 122.0 * u;
    bits::runs(&mut s, left, top, 15.0 * u, &[("WHAT YOU CAN DO", p.dim, false)]);
    let mut y = top + 34.0 * u;
    for (key, words) in &h.keys {
        if y > hpx - 14.0 * u {
            break;
        }
        bits::runs(&mut s, left, y, 16.6 * u, &[(key, p.bright, true)]);
        bits::runs(&mut s, left + key_w, y, 16.6 * u, &[(&fit(words, 16.6 * u, column - key_w), p.ink, false)]);
        y += step;
    }
    let (x, mut y) = if two { (left + column + 40.0 * u, top) } else { (left, y + 18.0 * u) };
    if y + 34.0 * u <= hpx - 14.0 * u {
        bits::runs(&mut s, x, y, 15.0 * u, &[(h.legend_title, p.dim, false)]);
    }
    y += 34.0 * u;
    for (mark, words) in &h.legend {
        if y > hpx - 14.0 * u {
            break;
        }
        sign(&mut s, *mark, x, y - 5.0 * u, u, &p);
        bits::runs(&mut s, x + 48.0 * u, y, 16.6 * u, &[(&fit(words, 16.6 * u, column - 48.0 * u), p.ink, false)]);
        y += step;
    }
    frame.render_widget(Clear, rect);
    placements.push(Placement { rect, svg: s.svg(), paper: false });
}
