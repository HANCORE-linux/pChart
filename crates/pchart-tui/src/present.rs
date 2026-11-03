//! Present mode (pictures 74): the situation for a meeting, one statement a screen and large, in the
//! order of docs/PLAN.md: where we stand, the chain that decides the date, who waits on whom, what is
//! next, and the roadmap at the Overview level.

use pchart_draw::Level;
use pchart_draw::bits::Icon;
use pchart_draw::situation::{self, Statement};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Span;

use crate::app::{App, Step};
use crate::pictures::Placement;
use crate::ui::{Inks, bold, day, panel, put};

pub const SLIDES: [&str; 5] = ["WHERE DO WE STAND", "THIS CHAIN DECIDES THE DATE", "WHO WAITS ON WHOM", "NEXT", "ROADMAP"];

fn pixels(app: &App, rect: Rect) -> (f32, f32) {
    (f32::from(rect.width) * f32::from(app.cell.0), f32::from(rect.height) * f32::from(app.cell.1))
}

fn unit(app: &App) -> f32 {
    f32::from(app.cell.1) / 21.0
}

/// How much larger than on the situation screen a slide draws, for a picture `rows` cells high there.
fn scale(body: Rect, rows: u16) -> f32 {
    (f32::from(body.height.saturating_sub(4)) / f32::from(rows)).clamp(1.0, 2.0)
}

/// The handoffs slide 3 shows: those on the deciding chain, else all of them.
fn handoffs(app: &App) -> Vec<&pchart_core::analysis::Handoff> {
    let deciding: Vec<_> = app.analysis.handoffs.iter().filter(|handoff| handoff.deciding).collect();
    if deciding.is_empty() { app.analysis.handoffs.iter().collect() } else { deciding }
}

/// The items `↑↓` chooses among on `slide` (pictures 76), in the order the slide shows them.
pub fn entries(app: &App, slide: usize) -> Vec<usize> {
    match slide {
        0 => app.analysis.verdict.as_ref().map(|verdict| verdict.target).into_iter().collect(),
        1 => app.schedule.chain.clone(),
        2 => handoffs(app).iter().map(|handoff| handoff.item).collect(),
        3 => app
            .steps()
            .iter()
            .filter_map(|step| match step {
                Step::Next(at) => match &app.analysis.next[*at] {
                    pchart_core::analysis::Next::StartBy { item, .. } | pchart_core::analysis::Next::Due { item, .. } => Some(*item),
                    pchart_core::analysis::Next::Check(index) => app.analysis.checks[*index].items().first().copied(),
                },
                Step::Report(at) => app.plan.item(&app.reports[*at].1.item),
            })
            .collect(),
        _ => (0..app.plan.items.len()).collect(),
    }
}

/// Space: shows or hides the details of the chosen entry; they need 150 columns, as beside the roadmap.
pub fn toggle_details(app: &mut App, width: u16) {
    app.present_details ^= true;
    app.message = (app.present_details && width < 150).then(|| "The details need 150 columns".into());
}

/// `←→`: the slide `step` slides on, starting at its first entry.
pub fn turn(app: &mut App, step: isize) {
    if let Some(slide) = app.present {
        app.present = Some((slide as isize + step).clamp(0, SLIDES.len() as isize - 1) as usize);
        app.present_at = 0;
    }
}

/// `↑↓`: the entry `step` entries on.
pub fn choose(app: &mut App, step: isize) {
    if let Some(slide) = app.present {
        let count = entries(app, slide).len();
        app.present_at = (app.present_at as isize + step).clamp(0, count.saturating_sub(1) as isize) as usize;
    }
}

pub fn draw(app: &App, frame: &mut Frame, inks: &Inks, area: Rect, slide: usize, placements: &mut Vec<Placement>) {
    let slide = slide.min(SLIDES.len() - 1);
    let counter = format!("{} / {}", slide + 1, SLIDES.len());
    put(frame, 0, 0, area.width, vec![Span::styled(app.plan.name.clone(), bold(inks.bright)), Span::styled(format!(" · {}", day(app.today)), inks.dim)]);
    put(frame, area.width.saturating_sub(counter.chars().count() as u16 + 1), 0, area.width, vec![Span::styled(counter, inks.dim)]);
    let entries = entries(app, slide);
    let at = app.present_at.min(entries.len().saturating_sub(1));
    let side = if app.present_details && !entries.is_empty() && area.width >= 150 { (area.width * 26 / 100).clamp(40, 56) } else { 0 };
    let body = Rect { x: 0, y: 1, width: area.width - side, height: area.height.saturating_sub(2) };
    if side > 0 {
        crate::ui::details(app, entries[at], &app.shown(), frame, inks, Rect { x: body.width, width: side, ..body }, placements);
    }
    panel(frame, inks, body, SLIDES[slide], true);
    let inner = Rect { x: body.x + 2, y: body.y + 2, width: body.width.saturating_sub(4), height: body.height.saturating_sub(3) };
    let p = app.shown();
    let u = unit(app);
    let big = |rows: u16| scale(inner, rows);
    let middle = |height: u16| Rect { y: inner.y + inner.height.saturating_sub(height) / 2, height: height.min(inner.height), ..inner };
    match slide {
        0 => match &app.analysis.verdict {
            Some(verdict) => {
                let k = big(8);
                let rect = middle((8.0 * k).round() as u16);
                let (w, h) = pixels(app, rect);
                placements.push(Placement { rect, svg: situation::verdict(&app.plan, verdict, w, h, u * k, &p).svg(), paper: false });
            }
            None => {
                let end = app.schedule.slots.iter().map(|slot| slot.planned.finish).max().unwrap_or(app.today);
                statements(app, inner, &[Statement { icon: Icon::Calendar, tone: p.ink, title: format!("{} has no fixed target date; it ends {}.", app.plan.name, day(end)), strong: true, line: Vec::new() }], None, placements);
            }
        },
        1 => {
            let steps = app.schedule.chain.len().max(2) as f32 - 1.0;
            let (w, _) = pixels(app, inner);
            let roomy = w / (u * 180.0 + steps * 14.0 * f32::from(app.cell.0));
            let k = big(9).min(roomy.max(1.0));
            let rect = middle((9.0 * k).round() as u16);
            let (w, h) = pixels(app, rect);
            placements.push(Placement { rect, svg: situation::chosen_chain(&app.plan, &app.schedule, Some(at), w, h, u * k, &p).svg(), paper: false });
        }
        2 => {
            let said: Vec<Statement> = handoffs(app)
                .iter()
                .map(|handoff| Statement {
                    icon: Icon::Person,
                    tone: p.ink,
                    title: format!("{} waits on {}", pchart_core::text::lane(&app.plan, handoff.waiting), pchart_core::text::lane(&app.plan, handoff.on)),
                    strong: handoff.deciding,
                    line: vec![(Some(Icon::Calendar), format!("{} due {}", app.plan.items[handoff.item].name, day(handoff.due)))],
                })
                .collect();
            let more = app.analysis.handoffs.len() - said.len();
            if said.is_empty() {
                put(frame, inner.x + 4, inner.y + 1, inner.width, vec![Span::styled("No team waits on another.", inks.dim)]);
            }
            let rest = said.len() - statements(app, inner, &said, Some(at), placements) + more;
            if rest > 0 {
                put(frame, inner.x + 4, inner.y + inner.height.saturating_sub(1), inner.width, vec![Span::styled(format!("and {rest} more"), inks.dim)]);
            }
        }
        3 => {
            let said: Vec<Statement> = app
                .steps()
                .iter()
                .map(|step| match step {
                    Step::Next(at) => crate::situation::statement(app, &app.analysis.next[*at]),
                    Step::Report(at) => {
                        let report = &app.reports[*at].1;
                        let (icon, tone) = crate::reports::symbol(app, report);
                        Statement { icon, tone, title: crate::reports::title(app, report), strong: false, line: vec![(Some(Icon::Note), format!("reported {}", crate::reports::place(report)))] }
                    }
                })
                .collect();
            if said.is_empty() {
                put(frame, inner.x + 4, inner.y + 1, inner.width, vec![Span::styled("Nothing is due.", inks.dim)]);
            }
            statements(app, inner, &said, Some(at), placements);
        }
        _ => {
            let (w, h) = pixels(app, inner);
            let settings = pchart_draw::Settings { width: w, height: h, unit: u * 1.25, today: app.today, selected: entries.get(at).copied(), level: Level::Overview, palette: p, cursor: None };
            placements.push(Placement { rect: inner, svg: pchart_draw::roadmap(&app.plan, &app.schedule, &app.calendars, &settings).svg(), paper: false });
        }
    }
}

/// `said` one under the other as large as the room lets them be, at most four, the one at `chosen` on
/// a raised band; how many it showed.
fn statements(app: &App, inner: Rect, said: &[Statement], chosen: Option<usize>, placements: &mut Vec<Placement>) -> usize {
    let count = said.len().clamp(1, 4) as u16;
    let (w, _) = pixels(app, inner);
    let longest = said.iter().take(4).map(|statement| statement.title.chars().count() as f32 * if statement.strong { 24.0 } else { 20.0 } * pchart_draw::style::ADVANCE + 118.0).fold(0.0, f32::max);
    let k = (f32::from(inner.height.saturating_sub(2)) / (5.0 * f32::from(count))).min(w / (unit(app) * longest.max(1.0))).clamp(1.0, 2.0);
    let (rows, gap) = ((4.0 * k).round() as u16, (5.0 * k).round() as u16);
    let fit = usize::from((inner.height.saturating_sub(1) + gap - rows) / gap).min(said.len());
    let first = chosen.map_or(0, |chosen| chosen.saturating_sub(fit.saturating_sub(1)));
    let top = inner.y + (inner.height.saturating_sub(gap * fit as u16)) / 2;
    for (row, (at, statement)) in said.iter().enumerate().skip(first).take(fit).enumerate() {
        let rect = Rect { x: inner.x, y: top + gap * row as u16, width: inner.width, height: rows };
        let (w, h) = pixels(app, rect);
        placements.push(Placement { rect, svg: situation::statement(statement, chosen == Some(at), w, h, unit(app) * k, &app.shown()).svg(), paper: false });
    }
    fit
}
