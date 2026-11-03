//! The finder (Ctrl+P, pictures 10 and 50): type what you want to do, in your words. It lists the
//! actions and items that match, the chosen one on the raised band of picture 36, and Enter runs it as
//! its key would.

use pchart_draw::bits::{self, Icon};
use pchart_draw::scene::{Fill, Shape};
use pchart_draw::style::ADVANCE;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Clear;

use crate::actions::{self, Action};
use crate::app::App;
use crate::help::{frame, place};
use crate::pictures::Placement;

const EXAMPLES: [&str; 4] = ["move launch", "pdf", "holidays", "undo"];

/// Draws the finder over the screen.
pub fn draw(app: &App, frame_: &mut Frame, area: Rect, placements: &mut Vec<Placement>) {
    let Some(finder) = &app.finder else { return };
    let p = app.palette;
    let (cw, ch) = (f32::from(app.cell.0), f32::from(app.cell.1));
    let u = ch / 21.0;
    let found = actions::find(app, &finder.typed);
    let split = found.iter().position(|(action, _)| *action == Action::Show).unwrap_or(found.len());
    let step = 30.0 * u;
    let head = 36.0 * u;
    let groups = usize::from(split > 0) + usize::from(split < found.len());
    let footer = 70.0 * u;
    let rect = place(area, area.width.saturating_sub(8).min(124), area.height.saturating_sub(5).min(24), Some(3));
    let (w, h) = (f32::from(rect.width) * cw, f32::from(rect.height) * ch);
    let mut s = bits::canvas(w, h, &p);
    frame(&mut s, w, h, u, &p);
    let left = 26.0 * u;
    let size = 16.6 * u;
    let advance = size * ADVANCE;
    bits::runs(&mut s, 20.0 * u, 29.0 * u, size, &[(":: FIND AN ACTION", p.dim, false)]);
    if !finder.typed.trim().is_empty() {
        let count = match found.len() {
            1 => "1 match".to_string(),
            count => format!("{count} matches"),
        };
        bits::runs(&mut s, w - 26.0 * u - count.chars().count() as f32 * advance, 58.0 * u, size, &[(&count, p.dim, false)]);
    }
    let end = bits::runs(&mut s, left, 58.0 * u, size, &[("› ", p.accent, true), (&finder.typed, p.bright, false)]);
    s.add(Shape::Rect { x: end + u, y: 44.0 * u, w: 2.0 * u, h: 17.0 * u, radius: 0.0, fill: Fill::Color(p.bright), stroke: None });
    s.add(Shape::Line { x1: 20.0 * u, y1: 72.0 * u, x2: w - 20.0 * u, y2: 72.0 * u, stroke: pchart_draw::scene::Stroke { color: p.rule, width: u, dash: None } });

    let names: Vec<String> = found.iter().map(|&(action, item)| action.name(app, item)).collect();
    let name_w = names.iter().take(split).map(|name| name.chars().count()).max().unwrap_or(0).max(names.iter().skip(split).map(|name| name.chars().count() + 3).max().unwrap_or(0)).min(28) as f32 * advance;
    let key_x = w - 26.0 * u;
    let what_x = left + 18.0 * u + name_w + 28.0 * u;
    let fit = |text: &str, room: f32| {
        let count = (room / advance).floor().max(0.0) as usize;
        if text.chars().count() <= count { text.to_string() } else { text.chars().take(count.saturating_sub(1)).collect::<String>() + "…" }
    };
    let bottom = h - footer;
    let rows_room = ((bottom - 84.0 * u - groups as f32 * head) / step).floor().max(1.0) as usize;
    let first = finder.chosen.saturating_sub(rows_room.saturating_sub(1));
    let mut y = 84.0 * u;
    for (at, (&(action, item), name)) in found.iter().zip(&names).enumerate() {
        if at == 0 && split > 0 || at == split && split < found.len() {
            y += head - 14.0 * u;
            bits::runs(&mut s, left, y, 15.0 * u, &[(if at < split { "ACTIONS" } else { "ITEMS" }, p.dim, false)]);
            y += 14.0 * u;
        }
        if at < first {
            continue;
        }
        if y + step > bottom + 4.0 * u {
            break;
        }
        let chosen = at == finder.chosen;
        let baseline = y + 20.0 * u;
        let ground = if chosen { p.raised } else { p.page };
        if chosen {
            s.add(Shape::Rect { x: 8.0 * u, y: y + 2.0 * u, w: w - 16.0 * u, h: step - 2.0 * u, radius: 0.0, fill: Fill::Color(p.raised), stroke: None });
            s.add(Shape::Rect { x: 8.0 * u, y: y + 2.0 * u, w: 4.0 * u, h: step - 2.0 * u, radius: 0.0, fill: Fill::Color(p.accent), stroke: None });
        }
        let mut x = left + 18.0 * u;
        if action == Action::Show
            && let Some(index) = item
        {
            let it = &app.plan.items[index];
            let (cx, cy) = (x + 8.0 * u, baseline - 5.0 * u);
            if it.milestone && !it.important {
                let r = 7.0 * u;
                s.add(Shape::Path { points: vec![(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)], closed: true, fill: Fill::Color(p.accent), stroke: None });
            } else {
                let (icon, tone) = if it.milestone { (Icon::Star, p.accent) } else { (Icon::Bar, if app.schedule.chain.contains(&index) { p.ink } else { p.dim }) };
                bits::draw_icon(&mut s, icon, cx, cy, 14.0 * u, tone, ground);
            }
            x += 26.0 * u;
        }
        bits::runs(&mut s, x, baseline, size, &[(&fit(name, what_x - x - 20.0 * u), if chosen { p.bright } else { p.ink }, chosen)]);
        let key = action.key();
        let key_w = key.chars().count() as f32 * advance;
        let what = action.what(app, item, &finder.typed);
        bits::runs(&mut s, what_x, baseline, size, &[(&fit(&what, key_x - key_w - 24.0 * u - what_x), if chosen { p.ink } else { p.dim }, false)]);
        bits::runs(&mut s, key_x - key_w, baseline, size, &[(key, p.ink, true)]);
        y += step;
    }
    if found.is_empty() {
        bits::runs(&mut s, left, 84.0 * u + 30.0 * u, size, &[(&format!("Nothing found for \"{}\"", finder.typed.trim()), p.ink, false)]);
    }
    if h - footer > 84.0 * u + head + step {
        bits::runs(&mut s, left, h - 46.0 * u, size, &[("Type what you want to do, in your words:", p.dim, false)]);
        let mut x = left;
        for example in EXAMPLES {
            x = bits::runs(&mut s, x, h - 20.0 * u, size, &[(&format!("\"{example}\""), p.ink, false)]) + 3.0 * advance;
        }
    }
    frame_.render_widget(Clear, rect);
    placements.push(Placement { rect, svg: s.svg(), paper: false });
}
