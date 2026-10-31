//! The situation screen (pictures 32 and 35): the verdict, the steps of "Next", and the chain that
//! decides the date, each a picture in a frame with its title.

use pchart_core::analysis::{Check, Next, Severity};
use pchart_draw::bits::Icon;
use pchart_draw::situation::{self, Statement};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Span;

use crate::app::App;
use crate::pictures::Placement;
use crate::ui::{Inks, bold, day, panel, put};

/// A picture of `rect`, in pixels.
fn pixels(app: &App, rect: Rect) -> (f32, f32) {
    (f32::from(rect.width) * f32::from(app.cell.0), f32::from(rect.height) * f32::from(app.cell.1))
}

fn unit(app: &App) -> f32 {
    f32::from(app.cell.1) / 21.0
}

/// The columns that `px` pixels of a 21 px tall cell take on this terminal.
fn columns(app: &App, px: f32) -> u16 {
    (px * unit(app) / f32::from(app.cell.0)).round() as u16
}

/// A lane with its city: "Product · Berlin".
fn team(app: &App, item: usize) -> String {
    let lane = &app.plan.lanes[app.plan.lane(&app.plan.items[item].lane).unwrap_or(0)];
    [Some(lane.name.as_str()), lane.city.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ")
}

/// A statement of "Next" in the words and symbols of picture 32.
fn statement(app: &App, next: &Next) -> Statement {
    let p = app.palette;
    let name = |item: usize| app.plan.items[item].name.clone();
    let when = |in_wd: i32| match in_wd {
        0 => "today".to_string(),
        1 => "in 1 working day".to_string(),
        count => format!("in {count} working days"),
    };
    match next {
        Next::StartBy { item, date, in_wd } => Statement { icon: Icon::Play, tone: p.ink, title: format!("Start {} by {}", name(*item), day(*date)), strong: true, line: vec![(Some(Icon::Clock), when(*in_wd)), (Some(Icon::Person), team(app, *item))] },
        Next::Due { item, date, in_wd } => Statement { icon: Icon::Play, tone: p.ink, title: format!("{} is due {}", name(*item), day(*date)), strong: true, line: vec![(Some(Icon::Clock), when(*in_wd)), (Some(Icon::Person), team(app, *item))] },
        Next::Check(index) => {
            let check = &app.analysis.checks[*index];
            let (icon, tone) = if check.severity() == Severity::Problem { (Icon::Cross, p.alert) } else { (Icon::Warn, p.warn) };
            let finish = |item: usize| day(app.schedule.slots[item].planned.finish);
            let (title, line) = match *check {
                Check::NoBuffer { before, target } => (format!("Add a buffer before {}", name(target)), format!("{} {}  →  {} {}", name(before), finish(before), name(target), finish(target))),
                _ => {
                    let (title, detail) = pchart_core::text::check(&app.plan, check);
                    (title.trim_end_matches('.').to_string(), detail)
                }
            };
            Statement { icon, tone, title, strong: false, line: vec![(None, line)] }
        }
    }
}

pub fn draw(app: &App, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    let x = columns(app, 60.0);
    let text_x = x + columns(app, 74.0);
    let w = area.width.saturating_sub(x + 2);
    let bottom = area.height - 1;
    let tall = area.height >= 30;
    let hero = if tall { 8 } else { 6 };
    let stand = Rect { x: 0, y: 1, width: area.width, height: hero + 3 };
    panel(frame, inks, stand, "WHERE DO WE STAND", false);
    match &app.analysis.verdict {
        Some(verdict) => {
            let rect = Rect { x, y: stand.y + 2, width: w, height: hero };
            let (pw, ph) = pixels(app, rect);
            placements.push(Placement { rect, svg: situation::verdict(&app.plan, verdict, pw, ph, unit(app), &app.palette).svg(), paper: false });
        }
        None => {
            let end = app.schedule.slots.iter().map(|slot| slot.planned.finish).max().unwrap_or(app.today);
            put(frame, text_x, stand.y + 3, w, vec![Span::styled(format!("{} has no fixed target date; it ends {}.", app.plan.name, day(end)), bold(inks.bright))]);
        }
    }

    let chain = Rect { x: 1, y: 0, width: area.width.saturating_sub(2), height: 9 };
    let (chain_w, _) = pixels(app, chain);
    let steps = app.schedule.chain.len().max(2);
    let roomy = (chain_w - 180.0 * unit(app)) / (steps - 1) as f32 / f32::from(app.cell.0) >= 14.0;
    let top = stand.y + stand.height;
    let fit = usize::from(bottom.saturating_sub(top + 3) / 5);
    let shown = app.analysis.next.len().min(fit).max(1);
    let next = Rect { x: 0, y: top, width: area.width, height: (3 + 5 * shown as u16).min(bottom - top) };
    let decides = Rect { x: 0, y: next.y + next.height, width: area.width, height: chain.height + 4 };
    let with_chain = roomy && decides.y + decides.height <= bottom;

    panel(frame, inks, next, "NEXT", true);
    let mut y = next.y + 2;
    if app.analysis.next.is_empty() {
        put(frame, text_x, y, w, vec![Span::styled("Nothing is due.", inks.dim)]);
    }
    let band = columns(app, 20.0).max(1);
    let band_w = area.width.saturating_sub(band + columns(app, 80.0));
    for (index, step) in app.analysis.next.iter().enumerate().take(shown) {
        if y + 4 > next.y + next.height {
            break;
        }
        let rect = Rect { x: band, y, width: band_w, height: 4 };
        let (pw, ph) = pixels(app, rect);
        placements.push(Placement { rect, svg: situation::statement(&statement(app, step), index == app.statement, pw, ph, unit(app), &app.palette).svg(), paper: false });
        y += 5;
    }

    if with_chain {
        panel(frame, inks, decides, "THIS CHAIN DECIDES THE DATE", false);
        let rect = Rect { y: decides.y + 2, ..chain };
        let (pw, ph) = pixels(app, rect);
        placements.push(Placement { rect, svg: situation::chain(&app.plan, &app.schedule, pw, ph, unit(app), &app.palette).svg(), paper: false });
    }
}

/// Words wrapped to `room` columns, keeping the style of each span.
pub(crate) fn wrap(spans: Vec<Span<'static>>, room: u16) -> Vec<Vec<Span<'static>>> {
    let mut lines: Vec<Vec<Span<'static>>> = vec![Vec::new()];
    let mut used = 0u16;
    for span in spans {
        let style = span.style;
        let words: Vec<String> = span.content.split_inclusive(' ').map(str::to_string).collect();
        for word in words {
            let w = word.trim_end().chars().count() as u16;
            if used + w > room && used > 0 {
                lines.push(Vec::new());
                used = 0;
            }
            used += word.chars().count() as u16;
            lines.last_mut().expect("a line").push(Span::styled(word, style));
        }
    }
    lines
}
