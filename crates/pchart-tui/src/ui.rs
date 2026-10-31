//! What each screen shows, and what gives way in a small window (see "Small windows" in the plan).

use jiff::civil::{Date, Weekday};
use pchart_core::analysis::{Check, Severity, State};
use pchart_draw::bits::{self, Symbol};
use pchart_draw::scene::Color as Ink;
use pchart_draw::{Level, shown};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::{App, Screen};
use crate::pictures::Placement;

/// The smallest window pChart draws in; below it says how much it needs.
pub const MIN: (u16, u16) = (60, 16);
const SIDE: u16 = 46;
const TILES: u16 = 6;

fn color(ink: Ink) -> Color {
    Color::Rgb(ink.0, ink.1, ink.2)
}

pub(crate) struct Inks {
    pub ink: Style,
    pub bright: Style,
    pub dim: Style,
    pub muted: Style,
    pub rule: Style,
    pub warn: Style,
    pub alert: Style,
    pub accent: Color,
    pub raised: Color,
}

impl Inks {
    fn of(app: &App) -> Inks {
        let p = app.palette;
        let style = |ink: Ink| Style::new().fg(color(ink));
        Inks { ink: style(p.ink), bright: style(p.bright), dim: style(p.dim), muted: style(p.muted), rule: style(p.rule), warn: style(p.warn), alert: style(p.alert), accent: color(p.accent), raised: color(p.raised) }
    }
}

pub(crate) fn bold(style: Style) -> Style {
    style.add_modifier(Modifier::BOLD)
}

pub(crate) fn width(spans: &[Span]) -> u16 {
    spans.iter().map(|span| span.content.chars().count() as u16).sum()
}

pub(crate) fn put(frame: &mut Frame, x: u16, y: u16, room: u16, spans: Vec<Span<'static>>) {
    let area = frame.area();
    if y >= area.height || x >= area.width {
        return;
    }
    let room = room.min(area.width - x);
    frame.render_widget(Paragraph::new(Line::from(spans)), Rect { x, y, width: room, height: 1 });
}

/// `text` cut to `room` characters, ending in "…" when it was cut.
pub(crate) fn cut(text: &str, room: usize) -> String {
    if text.chars().count() <= room {
        return text.to_string();
    }
    text.chars().take(room.saturating_sub(1)).collect::<String>() + "…"
}

/// A panel: a thin frame with its title; the panel with the focus gets brighter corners.
pub(crate) fn panel(frame: &mut Frame, inks: &Inks, rect: Rect, title: &str, focus: bool) {
    if rect.width < 4 || rect.height < 3 {
        return;
    }
    frame.render_widget(Block::bordered().border_style(inks.rule), rect);
    if focus {
        let (x0, y0, x1, y1) = (rect.x, rect.y, rect.x + rect.width - 1, rect.y + rect.height - 1);
        let buffer = frame.buffer_mut();
        for (x, y, symbol) in [(x0, y0, "┌"), (x0 + 1, y0, "─"), (x0, y0 + 1, "│"), (x1, y0, "┐"), (x1 - 1, y0, "─"), (x1, y0 + 1, "│"), (x0, y1, "└"), (x0 + 1, y1, "─"), (x0, y1 - 1, "│"), (x1, y1, "┘"), (x1 - 1, y1, "─"), (x1, y1 - 1, "│")] {
            buffer[(x, y)].set_symbol(symbol).set_style(inks.ink);
        }
    }
    if !title.is_empty() {
        put(frame, rect.x + 2, rect.y + 1, rect.width.saturating_sub(4), vec![Span::styled(format!(":: {title}"), inks.dim)]);
    }
}

/// Draws the screen and returns the pictures it needs.
pub fn draw(app: &App, frame: &mut Frame) -> Vec<Placement> {
    let area = frame.area();
    let inks = Inks::of(app);
    if area.width < MIN.0 || area.height < MIN.1 {
        too_small(frame, &inks, area);
        return Vec::new();
    }
    let mut placements = Vec::new();
    title_bar(app, frame, &inks, area);
    match app.screen {
        Screen::Situation => crate::situation::draw(app, frame, &inks, area, &mut placements),
        Screen::Checks => crate::checks::draw(app, frame, &inks, area, &mut placements),
        Screen::Export => crate::export::draw(app, frame, &inks, area, &mut placements),
        Screen::Roadmap => {
            let top = if area.width >= 120 && area.height >= 44 {
                tiles(app, frame, &inks, area, &mut placements);
                1 + TILES
            } else {
                summary(app, frame, &inks, area);
                2
            };
            roadmap(app, frame, &inks, Rect { x: 0, y: top, width: area.width, height: area.height - 1 - top }, &mut placements);
        }
    }
    keys(app, frame, &inks, area);
    placements
}

fn too_small(frame: &mut Frame, inks: &Inks, area: Rect) {
    let size = |value: u16, need: u16| Span::styled(value.to_string(), bold(if value < need { inks.alert } else { inks.bright }));
    let lines = [
        vec![Span::styled("Terminal size too small:", inks.ink)],
        vec![Span::styled("Width = ", inks.ink), size(area.width, MIN.0), Span::styled(" Height = ", inks.ink), size(area.height, MIN.1)],
        vec![],
        vec![Span::styled("Needed:", inks.ink)],
        vec![Span::styled("Width = ", inks.ink), Span::styled(MIN.0.to_string(), bold(inks.bright)), Span::styled(" Height = ", inks.ink), Span::styled(MIN.1.to_string(), bold(inks.bright))],
    ];
    let top = area.y + (area.height / 2).saturating_sub(2);
    for (y, spans) in (top..area.bottom()).zip(lines) {
        let x = area.x + area.width.saturating_sub(width(&spans)) / 2;
        put(frame, x, y, area.width.saturating_sub(x), spans);
    }
}

fn title_bar(app: &App, frame: &mut Frame, inks: &Inks, area: Rect) {
    let version: String = env!("CARGO_PKG_VERSION").split('.').take(2).collect::<Vec<_>>().join(".");
    let left = vec![Span::styled("pchart", bold(inks.bright)), Span::styled(format!(" {version} · {}", app.plan.name), inks.dim)];
    let mut x = width(&left);
    put(frame, 0, 0, area.width, left);
    x = (x + 5).max(36.min(area.width / 4));
    let warnings = app.analysis.checks.iter().filter(|check| check.severity() != Severity::Hint).count();
    let problems = app.analysis.checks.iter().any(|check| check.severity() == Severity::Problem);
    let full = area.width >= 100;
    for (number, name, screen) in [("1", "Situation", Screen::Situation), ("2", "Roadmap", Screen::Roadmap), ("3", "Checks", Screen::Checks), ("4", "Export", Screen::Export)] {
        let current = app.screen == screen;
        let mut spans = if current {
            let under = |style: Style| style.add_modifier(Modifier::UNDERLINED).underline_color(inks.accent).bg(inks.raised);
            vec![Span::styled(" ", under(inks.ink)), Span::styled(number, under(bold(Style::new().fg(inks.accent)))), Span::styled(format!(" {name} "), under(bold(inks.bright)))]
        } else if full {
            vec![Span::styled(format!(" {number}"), bold(inks.ink)), Span::styled(format!(" {name} "), inks.dim)]
        } else {
            vec![Span::styled(format!(" {number} "), bold(inks.ink))]
        };
        if screen == Screen::Checks && warnings > 0 {
            spans.push(Span::styled(format!("▲{warnings} "), if problems { inks.alert } else { inks.warn }));
        }
        let w = width(&spans);
        put(frame, x, 0, w, spans);
        x += w + 1;
    }
    let right = if area.width >= 135 { format!("{} · saved", home(&app.path)) } else { "saved".to_string() };
    let right = if (right.chars().count() as u16) + x + 2 > area.width { "saved".to_string() } else { right };
    let w = right.chars().count() as u16;
    if x + w < area.width {
        put(frame, area.width - w - 1, 0, w, vec![Span::styled(right, inks.dim)]);
    }
}

/// `path` with the home folder as "~".
pub(crate) fn home(path: &str) -> String {
    match std::env::var("HOME").ok().filter(|home| !home.is_empty() && path.starts_with(home.as_str())) {
        Some(home) => format!("~{}", &path[home.len()..]),
        None => path.to_string(),
    }
}

/// The verdict words and their colour: neutral for on track, yellow for at risk, red for late.
pub(crate) fn state(app: &App, inks: &Inks) -> Option<(&'static str, Style)> {
    app.analysis.verdict.as_ref().map(|verdict| match verdict.state {
        State::Holds => ("on track", inks.bright),
        State::Tight => ("at risk", inks.warn),
        State::DoesNotHold => ("late", inks.alert),
    })
}

pub(crate) fn holidays(app: &App) -> Vec<(usize, Date, String)> {
    let weekday = |day: Date| !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday);
    let mut out = Vec::new();
    for (index, item) in app.plan.items.iter().enumerate().filter(|(_, item)| !item.milestone) {
        let lane = app.plan.lane(&item.lane).unwrap_or(0);
        let slot = &app.schedule.slots[index].planned;
        out.extend(app.calendars[lane].holidays(slot.start, slot.finish).filter(|(day, _)| weekday(*day)).map(|(day, name)| (index, day, name.to_string())));
    }
    out
}

fn tiles(app: &App, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    let (cw, ch) = app.cell;
    let quarter = area.width / 4;
    let big = ch as f32 * 46.0 / 21.0;
    let p = app.palette;
    let target = app.schedule.target;
    let first = app.schedule.chain.first().copied();
    let spare = app.schedule.spare();
    let warnings = app.analysis.checks.iter().filter(|check| check.severity() == Severity::Warning).count();
    let problems = app.analysis.checks.iter().filter(|check| check.severity() == Severity::Problem).count();
    let tone = state(app, inks).map_or(p.bright, |(word, _)| match word {
        "at risk" => p.warn,
        "late" => p.alert,
        _ => p.bright,
    });
    let end = app.schedule.slots.iter().enumerate().max_by_key(|(_, slot)| slot.planned.finish).map(|(index, slot)| (index, slot.planned.finish));
    let day = |date: Date| (date.strftime("%d %b").to_string(), date.strftime("%a · %Y").to_string());
    let tiles: [(String, String, Ink, String, Style, String); 4] = [
        match target {
            Some(index) => {
                let item = &app.plan.items[index];
                let date = app.schedule.slots[index].planned.finish;
                let kind = if item.on.is_some() { "TARGET · FIXED" } else { "TARGET · DEADLINE" };
                (kind.into(), day(date).0, p.bright, day(date).1, inks.dim, format!("{} · scheduled backwards", item.name))
            }
            None => match end {
                Some((index, date)) => ("END".into(), day(date).0, p.bright, day(date).1, inks.dim, format!("{} · scheduled forwards", app.plan.items[index].name)),
                None => ("END".into(), String::new(), p.bright, String::new(), inks.dim, String::new()),
            },
        },
        match first {
            Some(index) => {
                let date = app.schedule.slots[index].latest.start;
                ("LATEST START".into(), day(date).0, p.bright, day(date).1, inks.dim, app.plan.items[index].name.clone())
            }
            None => ("LATEST START".into(), String::new(), p.bright, String::new(), inks.dim, String::new()),
        },
        match spare {
            Some(spare) => ("TO SPARE".into(), spare.to_string(), tone, "working days".into(), inks.dim, "until the latest start".into()),
            None => ("TO SPARE".into(), String::new(), p.bright, String::new(), inks.dim, String::new()),
        },
        {
            let count = holidays(app).len();
            let foot = if count == 1 { "1 holiday accounted for".to_string() } else { format!("{count} holidays accounted for") };
            if problems > 0 {
                ("CHECKS".into(), problems.to_string(), p.alert, if problems == 1 { "problem".into() } else { "problems".into() }, inks.alert, foot)
            } else if warnings > 0 {
                ("CHECKS".into(), warnings.to_string(), p.warn, if warnings == 1 { "warning".into() } else { "warnings".into() }, inks.warn, foot)
            } else {
                ("CHECKS".into(), "0".into(), p.bright, "warnings".into(), inks.dim, foot)
            }
        },
    ];
    for (index, (title, value, value_ink, side, side_style, foot)) in tiles.into_iter().enumerate() {
        let x = index as u16 * quarter;
        let w = if index == 3 { area.width - x } else { quarter };
        panel(frame, inks, Rect { x, y: 1, width: w, height: TILES }, &title, false);
        let mut side_x = x + 2;
        if !value.is_empty() {
            let cols = ((value.chars().count() as f32 * 0.6 * big) / cw as f32).ceil() as u16 + 1;
            let rect = Rect { x: x + 2, y: 3, width: cols.min(w.saturating_sub(4)), height: 2 };
            let (px_w, px_h) = (f32::from(rect.width) * f32::from(cw), f32::from(rect.height) * f32::from(ch));
            placements.push(Placement { rect, svg: bits::big(&value, big, value_ink, px_w, px_h, &app.palette).svg(), paper: false });
            side_x = rect.x + rect.width + 2;
        }
        put(frame, side_x, 4, (x + w).saturating_sub(side_x + 2), vec![Span::styled(side, side_style)]);
        put(frame, x + 2, 5, w.saturating_sub(4), vec![Span::styled(cut(&foot, w.saturating_sub(4) as usize), inks.dim)]);
    }
}

/// One line instead of the tiles: the target, the verdict, the latest start, the checks.
fn summary(app: &App, frame: &mut Frame, inks: &Inks, area: Rect) {
    let room = area.width.saturating_sub(2);
    let mut groups: Vec<Vec<Span<'static>>> = Vec::new();
    let long = |date: Date| date.strftime("%a %d %b %Y").to_string();
    if let Some(index) = app.schedule.target {
        let item = &app.plan.items[index];
        let mut group = vec![Span::styled(format!("{} ", item.name), inks.ink), Span::styled(long(app.schedule.slots[index].planned.finish), bold(inks.bright))];
        if item.on.is_some() {
            group.push(Span::styled(" · fixed", inks.dim));
        }
        groups.push(group);
    }
    if let (Some((word, style)), Some(spare)) = (state(app, inks), app.schedule.spare()) {
        groups.push(vec![Span::styled(word, bold(style)), Span::styled(" · ", inks.dim), Span::styled(spare.to_string(), bold(inks.bright)), Span::styled(" wd to spare", inks.dim)]);
    }
    if let Some(&first) = app.schedule.chain.first()
        && app.schedule.target.is_some()
    {
        groups.push(vec![Span::styled("start by ", inks.dim), Span::styled(app.schedule.slots[first].latest.start.strftime("%a %d %b").to_string(), bold(inks.bright))]);
    }
    let warnings = app.analysis.checks.iter().filter(|check| check.severity() != Severity::Hint).count();
    if warnings > 0 {
        let problems = app.analysis.checks.iter().any(|check| check.severity() == Severity::Problem);
        let noun = if problems { if warnings == 1 { "problem" } else { "problems" } } else if warnings == 1 { "warning" } else { "warnings" };
        groups.push(vec![Span::styled(format!("▲ {warnings} {noun}"), if problems { inks.alert } else { inks.warn })]);
    }
    let join = |groups: &[Vec<Span<'static>>]| {
        let mut line = Vec::new();
        for (index, group) in groups.iter().enumerate() {
            if index > 0 {
                line.push(Span::raw("   "));
            }
            line.extend(group.iter().cloned());
        }
        line
    };
    while groups.len() > 1 && width(&join(&groups)) > room {
        groups.pop();
    }
    put(frame, 1, 1, room, join(&groups));
}

fn keys(app: &App, frame: &mut Frame, inks: &Inks, area: Rect) {
    let item = if app.level == Level::Overview { "lane" } else { "item" };
    let hints: &[(&str, &str)] = match app.screen {
        Screen::Situation => &[("↑↓", "choose"), ("Enter", "show in roadmap"), ("q", "quit")],
        Screen::Roadmap => &[("↑↓", item), ("d", "detail"), ("1", "situation"), ("q", "quit")],
        Screen::Checks => &[("↑↓", "check"), ("Enter", "show in roadmap"), ("1", "situation"), ("q", "quit")],
        Screen::Export => &[("↑↓", "option"), ("←→", "change"), ("Space", "on/off"), ("Enter", "export"), ("Esc", "back")],
    };
    let mut spans = Vec::new();
    for &(key, label) in hints {
        let next = [Span::styled(key, bold(inks.ink)), Span::styled(format!(" {label}"), inks.dim), Span::raw("    ")];
        if width(&spans) + width(&next) > area.width.saturating_sub(1) {
            break;
        }
        spans.extend(next);
    }
    if let Some(message) = &app.message {
        spans.push(Span::styled(message.clone(), inks.warn));
    }
    put(frame, 1, area.height - 1, area.width - 1, spans);
}

/// Where the parts of the roadmap screen go in `body`.
pub struct Layout {
    pub panel: Rect,
    pub chart: Rect,
    legend: Option<u16>,
    chain: Option<u16>,
    bar: Option<u16>,
    side: Option<Rect>,
}

/// Where the roadmap picture goes on a screen of this size.
pub fn chart(area: Rect) -> Rect {
    let top = if area.width >= 120 && area.height >= 44 { 1 + TILES } else { 2 };
    layout(area, Rect { x: 0, y: top, width: area.width, height: area.height - 1 - top }).chart
}

pub fn layout(area: Rect, body: Rect) -> Layout {
    let side = (area.width >= 150).then(|| Rect { x: body.width - SIDE, y: body.y, width: SIDE, height: body.height });
    let panel = Rect { x: body.x, y: body.y, width: body.width - side.map_or(0, |side| side.width), height: body.height };
    let roomy = area.height >= 44;
    let mut bottom = panel.y + panel.height - 1;
    let bar = side.is_none().then(|| {
        bottom -= 3;
        bottom + 1
    });
    if roomy {
        bottom -= 1;
    }
    let chain = (area.height >= 30).then(|| {
        bottom -= 1;
        bottom
    });
    if roomy && chain.is_some() {
        bottom -= 1;
    }
    let legend = (area.width >= 110 && area.height >= 36).then(|| {
        bottom -= 1;
        bottom
    });
    let top = panel.y + 2;
    let end = if legend.is_some() { bottom - 1 } else { bottom };
    let chart = Rect { x: panel.x + 1, y: top, width: panel.width.saturating_sub(2), height: end.saturating_sub(top) };
    Layout { panel, chart, legend, chain, bar, side }
}

fn roadmap(app: &App, frame: &mut Frame, inks: &Inks, body: Rect, placements: &mut Vec<Placement>) {
    let area = frame.area();
    let l = layout(area, body);
    let settings = app.settings(l.chart);
    let (level, lanes) = shown(&app.plan, &app.schedule, &settings);
    let first = app.schedule.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(app.today);
    let last = app.schedule.slots.iter().map(|slot| slot.planned.finish).max().unwrap_or(app.today);
    let span = format!("{} – {}", first.strftime("%b %Y"), last.strftime("%b %Y")).to_uppercase();
    let switch_w = 34;
    let room = l.panel.width.saturating_sub(4 + switch_w) as usize;
    let title = if lanes.len() < app.plan.lanes.len() {
        format!("ROADMAP · LANES {}–{} OF {}", lanes.first().map_or(0, |lane| lane + 1), lanes.last().map_or(0, |lane| lane + 1), app.plan.lanes.len())
    } else if format!(":: SWIMLANE ROADMAP · {span}").chars().count() <= room {
        format!("SWIMLANE ROADMAP · {span}")
    } else if format!(":: ROADMAP · {span}").chars().count() <= room {
        format!("ROADMAP · {span}")
    } else {
        "ROADMAP".to_string()
    };
    panel(frame, inks, l.panel, &title, true);

    let mut switch = vec![Span::styled("Detail ", inks.muted)];
    for (name, option) in [("Overview", Level::Overview), ("Standard", Level::Standard), ("Full", Level::Full)] {
        let style = if option == level {
            bold(inks.bright).bg(inks.raised)
        } else if option == app.level {
            inks.dim.add_modifier(Modifier::UNDERLINED)
        } else {
            inks.dim
        };
        switch.push(Span::styled(format!(" {name} "), style));
    }
    let w = width(&switch);
    if l.panel.width > w + 4 + 12 {
        put(frame, l.panel.x + l.panel.width - w - 2, l.panel.y + 1, w, switch);
    }

    if l.chart.width > 0 && l.chart.height > 0 {
        let mut scene = pchart_draw::roadmap(&app.plan, &app.schedule, &app.calendars, &settings);
        scene.background = None;
        placements.push(Placement { rect: l.chart, svg: scene.svg(), paper: false });
    }
    if let Some(row) = l.legend {
        legend(app, frame, inks, level, l.panel.x + 2, row, l.panel.width.saturating_sub(4), placements);
    }
    if let Some(row) = l.chain {
        put(frame, l.panel.x + 2, row, l.panel.width.saturating_sub(4), chain(app, inks, l.panel.width.saturating_sub(4)));
    }
    if let Some(row) = l.bar {
        let rule = "─".repeat(l.panel.width.saturating_sub(2) as usize);
        put(frame, l.panel.x + 1, row - 1, l.panel.width.saturating_sub(2), vec![Span::styled(rule, inks.rule)]);
        selection_bar(app, frame, inks, l.panel.x + 2, row, l.panel.width.saturating_sub(4), level);
    }
    if let Some(side) = l.side {
        let checks = side.height >= 38;
        let selection = Rect { height: if checks { 19 } else { side.height }, ..side };
        selection_panel(app, frame, inks, selection, level);
        if checks {
            checks_panel(app, frame, inks, Rect { y: side.y + 19, height: side.height - 19, ..side });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn legend(app: &App, frame: &mut Frame, inks: &Inks, level: Level, x: u16, row: u16, room: u16, placements: &mut Vec<Placement>) {
    let (cw, ch) = app.cell;
    let unit = f32::from(ch) / 21.0;
    let mut entries = vec![(Some(Symbol::Chain), "decides the date")];
    match level {
        Level::Full => entries.extend([(Some(Symbol::SpareHatched), "time to spare"), (Some(Symbol::Milestone), "milestone"), (Some(Symbol::Fixed), "fixed date"), (Some(Symbol::Holiday), "holiday (lane)"), (Some(Symbol::Today), "today")]),
        Level::Standard => entries.extend([(Some(Symbol::Spare), "time to spare"), (Some(Symbol::Milestone), "milestone"), (Some(Symbol::Fixed), "fixed date"), (Some(Symbol::Today), "today"), (None, "warning")]),
        Level::Overview => entries.extend([(Some(Symbol::Spare), "time to spare"), (Some(Symbol::Milestone), "milestone"), (Some(Symbol::Fixed), "fixed date"), (Some(Symbol::Today), "today")]),
    }
    let mut at = x;
    for (symbol, label) in entries {
        let w = 3 + label.chars().count() as u16;
        if at + w > x + room {
            break;
        }
        match symbol {
            Some(symbol) => {
                let rect = Rect { x: at, y: row - 1, width: 3, height: 2 };
                let svg = bits::symbol(symbol, f32::from(rect.width) * f32::from(cw), f32::from(rect.height) * f32::from(ch), unit, &app.palette).svg();
                placements.push(Placement { rect, svg, paper: false });
            }
            None => put(frame, at + 1, row, 1, vec![Span::styled("▲", bold(inks.warn))]),
        }
        put(frame, at + 3, row, label.chars().count() as u16, vec![Span::styled(label, inks.dim)]);
        at += w + 3;
    }
}

/// "decides the date  Requirements → … → Launch", cut around the selection when it is too wide.
fn chain(app: &App, inks: &Inks, room: u16) -> Vec<Span<'static>> {
    let chain = &app.schedule.chain;
    let lead = "decides the date  ";
    let spans = |keep: &[bool]| {
        let mut out = vec![Span::styled(lead, inks.muted)];
        let mut gap = false;
        for (position, &index) in chain.iter().enumerate() {
            if !keep[position] {
                gap = true;
                continue;
            }
            if out.len() > 1 {
                out.push(Span::styled(if gap { " → … → " } else { " → " }, inks.muted));
            } else if gap {
                out.push(Span::styled("… → ", inks.muted));
            }
            gap = false;
            let style = if index == app.selected { bold(inks.bright) } else { inks.ink };
            out.push(Span::styled(app.plan.items[index].name.clone(), style));
        }
        if gap {
            out.push(Span::styled(" → …", inks.muted));
        }
        out
    };
    let count = chain.len();
    let at = chain.iter().position(|&index| index == app.selected);
    let all = vec![true; count];
    if width(&spans(&all)) <= room || count == 0 {
        return spans(&all);
    }
    for reach in (0..count).rev() {
        let keep: Vec<bool> = (0..count).map(|position| position == 0 || position + 1 == count || at.is_some_and(|at| position.abs_diff(at) <= reach)).collect();
        if width(&spans(&keep)) <= room {
            return spans(&keep);
        }
    }
    let keep: Vec<bool> = (0..count).map(|position| Some(position) == at).collect();
    spans(&keep)
}

pub(crate) fn lane_label(app: &App, index: usize) -> String {
    let lane = &app.plan.lanes[app.plan.lane(&app.plan.items[index].lane).unwrap_or(0)];
    [Some(lane.name.as_str()), lane.city.as_deref(), lane.place.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ")
}

/// Two rows under the chart instead of the side column: the selected item, its dates and what it has to spare.
#[allow(clippy::too_many_arguments)]
fn selection_bar(app: &App, frame: &mut Frame, inks: &Inks, x: u16, row: u16, room: u16, level: Level) {
    let item = &app.plan.items[app.selected];
    let slot = &app.schedule.slots[app.selected];
    if level == Level::Overview {
        let (name, folded, from, to, ..) = folded_lane(app);
        put(frame, x, row, room, vec![Span::styled(name, bold(inks.bright)), Span::styled(format!("  Lane · {folded}"), inks.muted)]);
        put(frame, x, row + 1, room, vec![Span::styled(format!("{} – {}", day(from), day(to)), inks.ink)]);
        return;
    }
    put(frame, x, row, room, vec![Span::styled(item.name.clone(), bold(inks.bright)), Span::styled(format!("  Lane {}", lane_label(app, app.selected)), inks.muted)]);
    let dates = if item.milestone { day(slot.planned.finish) } else { format!("{} – {} · {} wd", day(slot.planned.start), day(slot.planned.finish), item.duration) };
    let decides = app.schedule.chain.contains(&app.selected);
    let spare = Span::styled(format!("{} wd to spare", slot.spare), if slot.spare < 0 { inks.alert } else { inks.ink });
    let mut line = vec![Span::styled(dates, inks.ink), Span::raw("   "), spare];
    if decides {
        line.push(Span::styled(" · decides the date", inks.ink));
    }
    put(frame, x, row + 1, room, line);
}

pub(crate) fn day(date: Date) -> String {
    date.strftime("%a %d %b").to_string()
}

/// The selected item's lane at the Overview level: its name, what it folds, its span, the items it
/// stands for (its milestones when it has nothing else), and whether one of them decides the date.
fn folded_lane(app: &App) -> (String, String, Date, Date, Vec<usize>, bool) {
    let lane = app.plan.lane(&app.plan.items[app.selected].lane).unwrap_or(0);
    let mine: Vec<usize> = (0..app.plan.items.len()).filter(|&index| app.plan.lane(&app.plan.items[index].lane) == Some(lane)).collect();
    let tasks: Vec<usize> = mine.iter().copied().filter(|&index| !app.plan.items[index].milestone).collect();
    let folded = match (tasks.len(), mine.len()) {
        (0, 1) => "1 milestone".to_string(),
        (0, count) => format!("{count} milestones"),
        (1, _) => "1 item folded".to_string(),
        (count, _) => format!("{count} items folded"),
    };
    let shown = if tasks.is_empty() { mine } else { tasks };
    let from = shown.iter().map(|&index| app.schedule.slots[index].planned.start).min().unwrap_or(app.today);
    let to = shown.iter().map(|&index| app.schedule.slots[index].planned.finish).max().unwrap_or(app.today);
    let decides = shown.iter().any(|index| app.schedule.chain.contains(index));
    (app.plan.lanes[lane].name.clone(), folded, from, to, shown, decides)
}

fn selection_panel(app: &App, frame: &mut Frame, inks: &Inks, rect: Rect, level: Level) {
    panel(frame, inks, rect, "SELECTION", false);
    let (x, w) = (rect.x + 2, rect.width.saturating_sub(4));
    let value_x = x + 12;
    let row = |frame: &mut Frame, y: u16, key: &str, value: String, style: Style| {
        if y + 1 < rect.y + rect.height {
            put(frame, x, y, 12, vec![Span::styled(key.to_string(), inks.muted)]);
            put(frame, value_x, y, w.saturating_sub(12), vec![Span::styled(cut(&value, w.saturating_sub(12) as usize), style)]);
        }
    };
    if level == Level::Overview {
        let (name, folded, from, to, items, decides) = folded_lane(app);
        let lane = &app.plan.lanes[app.plan.lane(&app.plan.items[app.selected].lane).unwrap_or(0)];
        put(frame, x, rect.y + 3, w, vec![Span::styled(name, bold(inks.bright))]);
        put(frame, x, rect.y + 4, w, vec![Span::styled(format!("Lane · {folded}"), inks.muted)]);
        row(frame, rect.y + 6, "From", day(from), inks.ink);
        row(frame, rect.y + 7, "To", day(to), inks.ink);
        let place = [lane.city.as_deref(), lane.place.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
        row(frame, rect.y + 8, "Place", place, inks.ink);
        let items: Vec<&str> = items.iter().map(|&index| app.plan.items[index].name.as_str()).collect();
        row(frame, rect.y + 10, "Items", items.join(", "), inks.ink);
        row(frame, rect.y + 11, "Status", if decides { "decides the date".into() } else { "time to spare".into() }, inks.ink);
        return;
    }
    let item = &app.plan.items[app.selected];
    let slot = &app.schedule.slots[app.selected];
    put(frame, x, rect.y + 3, w, vec![Span::styled(cut(&item.name, w as usize), bold(inks.bright))]);
    put(frame, x, rect.y + 4, w, vec![Span::styled(cut(&format!("Lane {}", lane_label(app, app.selected)), w as usize), inks.muted)]);
    let names = |ids: Vec<&str>| if ids.is_empty() { "—".to_string() } else { ids.join(", ") };
    let waits = names(item.after.iter().filter_map(|id| app.plan.item(id)).map(|index| app.plan.items[index].name.as_str()).collect());
    let then = names(app.plan.items.iter().filter(|other| other.after.contains(&item.id)).map(|other| other.name.as_str()).collect());
    let decides = app.schedule.chain.contains(&app.selected);
    let span = |from: Date, to: Date| if item.milestone { day(to) } else { format!("{} – {}", day(from), day(to)) };
    row(frame, rect.y + 6, "Duration", if item.milestone { "milestone".into() } else { format!("{} working days", item.duration) }, inks.ink);
    row(frame, rect.y + 7, "Waits on", waits, inks.ink);
    row(frame, rect.y + 8, "Then", then, inks.ink);
    row(frame, rect.y + 10, "Planned", span(slot.planned.start, slot.planned.finish), inks.ink);
    row(frame, rect.y + 11, "Earliest", span(slot.earliest.start, slot.earliest.finish), inks.dim);
    row(frame, rect.y + 12, "To spare", format!("{} wd{}", slot.spare, if decides { " · decides the date" } else { "" }), if slot.spare < 0 { inks.alert } else { inks.ink });
    row(frame, rect.y + 13, "Date", if item.on.is_some() { "fixed".into() } else { "calculated".into() }, inks.dim);
    let lane = app.plan.lane(&item.lane).unwrap_or(0);
    let weekday = |day: Date| !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday);
    let days: Vec<String> = app.calendars[lane].holidays(slot.planned.start, slot.planned.finish).filter(|(day, _)| weekday(*day)).map(|(day, name)| format!("{} {name}", day.strftime("%d %b"))).collect();
    if !days.is_empty() && rect.y + 16 < rect.y + rect.height {
        put(frame, x, rect.y + 15, 12, vec![Span::styled("Holidays", inks.muted)]);
        for (offset, text) in days.into_iter().enumerate() {
            let y = rect.y + 15 + offset as u16;
            if y + 1 >= rect.y + rect.height {
                break;
            }
            put(frame, value_x, y, w.saturating_sub(12), vec![Span::styled(cut(&text, w.saturating_sub(12) as usize), inks.ink)]);
        }
    }
}

/// The checks next to the roadmap, in the words of picture 8, and the holidays the plan accounts for.
fn checks_panel(app: &App, frame: &mut Frame, inks: &Inks, rect: Rect) {
    let shown: Vec<&Check> = app.analysis.checks.iter().filter(|check| check.severity() != Severity::Hint).collect();
    let problems = shown.iter().filter(|check| check.severity() == Severity::Problem).count();
    let title = match (problems, shown.len()) {
        (0, 0) => "CHECKS".to_string(),
        (0, 1) => "CHECKS · 1 WARNING".to_string(),
        (0, count) => format!("CHECKS · {count} WARNINGS"),
        (problems, _) => format!("CHECKS · {problems} {}", if problems == 1 { "PROBLEM" } else { "PROBLEMS" }),
    };
    panel(frame, inks, rect, &title, false);
    let (x, w) = (rect.x + 2, rect.width.saturating_sub(4));
    let bottom = rect.y + rect.height - 1;
    let mut y = rect.y + 2;
    for check in shown {
        if y + 3 > bottom {
            break;
        }
        let mark = if check.severity() == Severity::Problem { inks.alert } else { inks.warn };
        let (title, detail) = check_words(app, check);
        put(frame, x, y, 2, vec![Span::styled("▲", bold(mark))]);
        put(frame, x + 2, y, w.saturating_sub(2), vec![Span::styled(cut(&title, w.saturating_sub(2) as usize), inks.bright)]);
        y += 1;
        for line in detail {
            put(frame, x + 2, y, w.saturating_sub(2), vec![Span::styled(cut(&line, w.saturating_sub(2) as usize), inks.dim)]);
            y += 1;
        }
    }
    let days = holidays(app);
    if days.is_empty() || y + 3 > bottom {
        return;
    }
    y += 1;
    put(frame, x, y, w, vec![Span::styled("Holidays accounted for", inks.muted)]);
    for (index, date, name) in days {
        y += 1;
        if y >= bottom {
            break;
        }
        let item = &app.plan.items[index];
        let lane = &app.plan.lanes[app.plan.lane(&item.lane).unwrap_or(0)];
        let code = lane.place.clone().unwrap_or_default();
        put(frame, x, y, 11, vec![Span::styled(cut(&item.name, 11), inks.ink)]);
        put(frame, x + 12, y, 7, vec![Span::styled(date.strftime("%d %b").to_string(), inks.dim)]);
        let name_room = w.saturating_sub(20 + code.chars().count() as u16 + 1) as usize;
        put(frame, x + 19, y, name_room as u16, vec![Span::styled(cut(&name, name_room), inks.dim)]);
        put(frame, x + w - code.chars().count() as u16, y, code.chars().count() as u16, vec![Span::styled(code, inks.muted)]);
    }
}

/// A name inside a sentence: "Launch" becomes "launch", "SAP cutover" stays as it is.
pub(crate) fn common(name: &str) -> String {
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(first), Some(second)) if first.is_uppercase() && second.is_lowercase() => first.to_lowercase().chain(name.chars().skip(1)).collect(),
        _ => name.to_string(),
    }
}

/// A check as picture 8 says it: a general title, then what it is about.
fn check_words(app: &App, check: &Check) -> (String, Vec<String>) {
    let name = |index: usize| app.plan.items[index].name.clone();
    let date = |index: usize| day(app.schedule.slots[index].planned.finish);
    match *check {
        Check::NoBuffer { before, target } => {
            let target_name = name(target);
            let lower = common(&target_name);
            ("No buffer before a fixed date".into(), vec![format!("{} {} → {} {}", name(before), date(before), target_name, date(target)), format!("Add a buffer block before {lower}?")])
        }
        _ => {
            let (title, detail) = pchart_core::text::check(&app.plan, check);
            (title, vec![detail])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pchart_core::{Holidays, Plan};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn app() -> App {
        let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
        let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
        let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        let today = jiff::civil::date(2026, 10, 29);
        let schedule = pchart_core::schedule(&plan, &calendars, today).unwrap();
        App::new(plan, calendars, today, "portal.pchart".into(), schedule, (7, 16))
    }

    fn screen(app: &App, width: u16, height: u16) -> (Vec<String>, Vec<Placement>) {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let mut placements = Vec::new();
        terminal.draw(|frame| placements = draw(app, frame)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        ((0..height).map(|y| (0..width).map(|x| buffer[(x, y)].symbol().to_string()).collect()).collect(), placements)
    }

    fn has(rows: &[String], text: &str) -> bool {
        rows.iter().any(|row| row.contains(text))
    }

    #[test]
    fn the_checks_screen_lists_by_weight_and_says_what_a_check_means() {
        let mut app = app();
        app.screen = Screen::Checks;
        let (rows, placements) = screen(&app, 170, 50);
        assert!(has(&rows, ":: CHECKS · 1 WARNING · 1 HINT") && has(&rows, "PROBLEMS   none") && has(&rows, "› ▲ No buffer before a fixed date"));
        assert!(has(&rows, "Acceptance Fri 26 Feb → Launch Mon 01 Mar") && has(&rows, "Its last item is Requirements, Fri 27 Nov"));
        assert!(has(&rows, "Every fixed date holds · deadlines kept ·") && has(&rows, "Two items of one person at once: the plan names no one"));
        assert!(has(&rows, ":: HOLIDAYS ACCOUNTED FOR · 5") && has(&rows, "Frontend      Tue 26 Jan  Republic Day"));
        assert!(has(&rows, ":: IN THE ROADMAP · 18 JAN – 12 MAR 2027") && has(&rows, "starting by Mon 09 Nov.") && placements.len() == 1);
        let (rows, placements) = screen(&app, 80, 24);
        assert!(has(&rows, "If Acceptance slips by a day, Launch is late.") && has(&rows, "A 5 wd buffer before it leaves 7 wd to spare."));
        assert!(placements.is_empty() && !has(&rows, "IN THE ROADMAP"));
        app.choose_check(1);
        let (rows, _) = screen(&app, 170, 50);
        assert!(has(&rows, ":: IN THE ROADMAP · 12 OCT – 04 DEC 2026") && has(&rows, "Add a milestone after Requirements"));
        app.show_check_in_roadmap();
        assert_eq!((app.screen, app.plan.items[app.selected].id.as_str()), (Screen::Roadmap, "requirements"));
    }

    #[test]
    fn the_export_screen_writes_what_its_preview_shows() {
        let dir = std::env::temp_dir().join(format!("pchart-export-{}", std::process::id()));
        let mut app = app();
        app.path = dir.join("portal.pchart").display().to_string();
        app.screen = Screen::Export;
        let (rows, placements) = screen(&app, 170, 50);
        assert!(has(&rows, "› Milestone chart") && has(&rows, "› Team") && has(&rows, "[x] PDF   [x] SVG   [ ] PNG   [ ] Markdown"));
        assert!(has(&rows, "Page       16:9 · slide") && has(&rows, "Files      portal-milestones.pdf") && placements.len() == 1);
        app.export_change(-1);
        app.export_part(1);
        app.export_change(1);
        app.export_change(1);
        let (rows, _) = screen(&app, 170, 50);
        assert!(has(&rows, "› Gantt chart") && has(&rows, "› Client") && has(&rows, "hides  what items have to spare, warnings"));
        app.export_part(1);
        app.export_change(1);
        app.export_change(1);
        app.export_toggle();
        app.export_change(1);
        app.export_toggle();
        let names: Vec<String> = crate::export::files(&app).iter().map(|file| file.file_name().unwrap().to_string_lossy().into_owned()).collect();
        assert_eq!(names, ["portal-gantt.pdf", "portal-gantt.svg", "portal-gantt.png", "portal.md", "portal-milestones.svg"]);
        let (rows, placements) = screen(&app, 80, 24);
        assert!(has(&rows, "Files      portal-gantt.pdf, .svg, .png, portal.md,") && has(&rows, "The preview needs 110 columns.") && placements.is_empty());

        let written = crate::export::write(&crate::export::job(&app)).unwrap();
        assert_eq!(written, crate::export::files(&app));
        let read = |name: &str| std::fs::read(dir.join("export").join(name)).unwrap();
        assert!(read("portal-gantt.pdf").starts_with(b"%PDF") && read("portal-gantt.png").starts_with(b"\x89PNG"));
        let svg = String::from_utf8(read("portal-gantt.svg")).unwrap();
        assert!(svg.contains("Gantt chart") && !svg.contains("To spare") && !svg.contains("no buffer"));
        let md = String::from_utf8(read("portal.md")).unwrap();
        assert!(md.contains("**Launch on Mon 01 Mar is on track.**\n") && !md.contains("to spare, with") && md.contains("![Milestone chart](portal-milestones.svg)"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn small_windows_give_way_in_the_approved_order() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let (rows, placements) = screen(&app, 242, 61);
        assert!(rows[2].contains(":: TARGET · FIXED") && rows.iter().any(|row| row.contains(":: SELECTION")) && rows.iter().any(|row| row.contains(":: CHECKS · 1 WARNING")));
        assert!(rows.iter().any(|row| row.contains("decides the date  Requirements → UX concept")));
        assert!(placements.len() > 5);
        let (rows, _) = screen(&app, 120, 36);
        assert!(rows[1].contains("on track · 12 wd to spare") && !rows.iter().any(|row| row.contains(":: SELECTION")));
        assert!(rows.iter().any(|row| row.contains("Frontend  Lane Frontend · Bangalore · IN-KA")));
        let (rows, _) = screen(&app, 80, 24);
        assert!(rows.iter().any(|row| row.contains("ROADMAP · LANES") && row.contains("OF 7")));
        assert!(!rows.iter().any(|row| row.contains("decides the date  Requirements")));
        let (rows, placements) = screen(&app, 50, 14);
        assert!(placements.is_empty() && rows.iter().any(|row| row.contains("Terminal size too small:")) && rows.iter().any(|row| row.contains("Width = 60 Height = 16")));
    }

    #[test]
    fn a_lane_of_milestones_stands_for_its_milestones() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        app.level = Level::Overview;
        app.selected = app.plan.item("acceptance").unwrap();
        let (rows, _) = screen(&app, 170, 50);
        assert!(has(&rows, "Lane · 3 milestones") && has(&rows, "Fri 18 Dec") && has(&rows, "Mon 01 Mar"));
        assert!(has(&rows, "Items       Design sign-off, Acceptance, …") && has(&rows, "Status      decides the date") && !has(&rows, "0 items"));
    }

    #[test]
    fn enter_on_a_statement_shows_its_item_in_the_roadmap() {
        let mut app = app();
        app.choose(1);
        app.show_in_roadmap();
        assert_eq!((app.screen, app.plan.items[app.selected].id.as_str()), (Screen::Roadmap, "acceptance"));
        app.screen = Screen::Situation;
        app.choose(1);
        app.show_in_roadmap();
        assert_eq!(app.plan.items[app.selected].id, "requirements");
    }

    #[test]
    fn the_situation_says_where_we_stand_what_is_next_and_what_decides_the_date() {
        let app = app();
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, ":: WHERE DO WE STAND") && has(&rows, ":: NEXT") && has(&rows, ":: THIS CHAIN DECIDES THE DATE") && placements.len() == 4);
        for text in ["Launch on Mon 01 Mar is ", "on track.", "working days to spare", "Start Requirements by Mon 16 Nov", "in 12 working days", "Product · Berlin", "Add a buffer before Launch", "Acceptance Fri 26 Feb  →  Launch Mon 01 Mar"] {
            assert!(svg.contains(text), "{text}");
        }
        assert!(!has(&rows, "WHO IS WAITING") && !svg.contains("NEXT 6 WEEKS") && !svg.contains("does not hold"));
        let (rows, placements) = screen(&app, 80, 24);
        assert!(has(&rows, ":: NEXT") && !has(&rows, "THIS CHAIN DECIDES THE DATE") && placements.len() == 3);
    }
}
