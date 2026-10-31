//! What each screen shows, and what gives way in a small window (see "Small windows" in the plan).

use jiff::civil::{Date, Weekday};
use pchart_core::analysis::{Severity, State};
use pchart_draw::bits::{self, Icon, MixedPart, Part};
use pchart_draw::scene::Color as Ink;
use pchart_draw::{Level, Palette, shown};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::app::{App, Screen, Spot};
use crate::pictures::Placement;

/// The smallest window pChart draws in; below it says how much it needs.
pub const MIN: (u16, u16) = (60, 16);
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
    fn of(p: Palette) -> Inks {
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
    let inks = Inks::of(app.palette);
    if app.ground {
        frame.buffer_mut().set_style(area, Style::new().bg(color(app.palette.page)).fg(color(app.palette.ink)));
    }
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
            let palette = if app.question.is_some() || writing(app) { app.palette.dimmed() } else { app.palette };
            let under = Inks::of(palette);
            let top = if area.width >= 120 && area.height >= 44 {
                tiles(app, &palette, frame, &under, area, &mut placements);
                1 + TILES
            } else {
                summary(app, frame, &under, area);
                2
            };
            roadmap(app, &palette, frame, &under, Rect { x: 0, y: top, width: area.width, height: area.height - 1 - top }, &mut placements);
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
    let modes: Vec<String> = match (app.screen, &app.question, &app.before) {
        (Screen::Roadmap, None, None) if writing(app) => vec!["NOTE · nothing changed yet".into(), "NOTE".into()],
        (Screen::Roadmap, None, None) if app.notes.is_some() => vec!["ADD NOTES".into()],
        (Screen::Roadmap, Some(_), _) if area.width >= 100 => vec!["CHANGE A DATE · nothing changed yet".into()],
        (Screen::Roadmap, Some(_), _) => vec!["CHANGE A DATE".into()],
        (Screen::Roadmap, None, Some(_)) => {
            let changes = app.changes();
            match changes.len() {
                0 => vec!["WHAT-IF · nothing changed yet".into(), "WHAT-IF".into()],
                1 => vec![format!("WHAT-IF · {} · not saved", changes[0]), "WHAT-IF · not saved".into(), "WHAT-IF".into()],
                count => vec![format!("WHAT-IF · {count} changes · not saved"), "WHAT-IF · not saved".into(), "WHAT-IF".into()],
            }
        }
        _ => Vec::new(),
    };
    if !modes.is_empty() {
        if let Some(mode) = modes.into_iter().find(|mode| x + mode.chars().count() as u16 + 2 < area.width) {
            let w = mode.chars().count() as u16;
            put(frame, area.width - w - 1, 0, w, vec![Span::styled(mode, inks.warn)]);
        }
        return;
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

fn tiles(app: &App, palette: &Palette, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    let (cw, ch) = app.cell;
    let quarter = area.width / 4;
    let big = ch as f32 * 46.0 / 21.0;
    let p = *palette;
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
                let (side, side_style) = match state(app, inks).filter(|_| app.before.is_some()) {
                    Some((word, style)) => (word.to_string(), style),
                    None => (day(date).1, inks.dim),
                };
                (kind.into(), day(date).0, p.bright, side, side_style, format!("{} · scheduled backwards", item.name))
            }
            None => match end {
                Some((index, date)) => ("END".into(), day(date).0, p.bright, day(date).1, inks.dim, format!("{} · scheduled forwards", app.plan.items[index].name)),
                None => ("END".into(), String::new(), p.bright, String::new(), inks.dim, String::new()),
            },
        },
        match first {
            Some(index) => {
                let date = app.schedule.slots[index].latest.start;
                let name = app.plan.items[index].name.clone();
                match app.before {
                    Some(_) if date < app.today => ("LATEST START".into(), day(date).0, p.alert, day(date).1, inks.dim, name),
                    Some(_) if date == app.today => ("LATEST START".into(), day(date).0, p.warn, "today".into(), inks.warn, name),
                    _ => ("LATEST START".into(), day(date).0, p.bright, day(date).1, inks.dim, name),
                }
            }
            None => ("LATEST START".into(), String::new(), p.bright, String::new(), inks.dim, String::new()),
        },
        match spare {
            Some(spare) => {
                let foot = app.before.as_ref().and_then(|before| before.schedule.spare()).map_or("until the latest start".to_string(), |was| format!("was {was}"));
                ("TO SPARE".into(), spare.to_string(), tone, "working days".into(), inks.dim, foot)
            }
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
            placements.push(Placement { rect, svg: bits::big(&value, big, value_ink, px_w, px_h, palette).svg(), paper: false });
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
    let notes = |spot: Option<Spot>| {
        let milestone = matches!(spot, Some(Spot::Item(index)) if app.plan.items[index].milestone);
        let enter = match (spot, app.note_here()) {
            (_, Some(_)) => "change the note",
            (Some(Spot::Place(..)), None) => "write a note here",
            _ => "write a note",
        };
        let mut hints = vec![("←→", "day"), ("Shift ←→", "week"), ("↑↓", "lane"), ("Enter", enter)];
        if milestone {
            hints.push(("i", "important"));
        }
        hints.push(("Esc", "done"));
        hints
    };
    let hints: Vec<(&str, &str)> = match app.screen {
        Screen::Situation => vec![("↑↓", "choose"), ("Enter", "show in roadmap"), ("q", "quit")],
        Screen::Roadmap if app.question.is_some() => vec![("↑↓", "choose"), ("Enter", "apply"), ("Tab", "try as what-if"), ("Esc", "keep as it is")],
        Screen::Roadmap if writing(app) => vec![("Enter", "keep the note"), ("Esc", "keep as it is")],
        Screen::Roadmap if app.notes.is_some() => notes(app.spot()),
        Screen::Roadmap if app.before.is_some() => vec![("Enter", "keep"), ("Esc", "discard"), ("←→", "keep changing"), ("↑↓", item)],
        Screen::Roadmap => vec![("↑↓", item), ("←→", "length"), ("Enter", "change a date"), ("n", "add notes"), ("u", "undo"), ("d", "detail"), ("1", "situation"), ("q", "quit")],
        Screen::Checks if app.check < app.checks().len() => vec![("↑↓", "check"), ("Enter", "show in roadmap"), ("1", "situation"), ("q", "quit")],
        Screen::Checks => vec![("↑↓", "check"), ("1", "situation"), ("q", "quit")],
        Screen::Export => vec![("↑↓", "option"), ("←→", "change"), ("Space", "on/off"), ("Enter", "export"), ("Esc", "back")],
    };
    let mut spans = Vec::new();
    for (key, label) in hints {
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

/// Where the parts of the roadmap screen go in `body` (picture 35): the chart in its frame, and under
/// it one line about the selection.
pub struct Layout {
    pub panel: Rect,
    pub chart: Rect,
    line: u16,
}

/// Where the roadmap picture goes on a screen of this size.
pub fn chart(area: Rect) -> Rect {
    let top = if area.width >= 120 && area.height >= 44 { 1 + TILES } else { 2 };
    layout(area, Rect { x: 0, y: top, width: area.width, height: area.height - 1 - top }).chart
}

pub fn layout(area: Rect, body: Rect) -> Layout {
    let gap = u16::from(area.height >= 30);
    let line = body.y + body.height - 1 - gap;
    let panel = Rect { x: body.x, y: body.y, width: body.width, height: line - body.y };
    let chart = Rect { x: panel.x + 1, y: panel.y + 2, width: panel.width.saturating_sub(2), height: panel.height.saturating_sub(3) };
    Layout { panel, chart, line }
}

fn roadmap(app: &App, palette: &Palette, frame: &mut Frame, inks: &Inks, body: Rect, placements: &mut Vec<Placement>) {
    let area = frame.area();
    let l = layout(area, body);
    let mut settings = app.settings(l.chart);
    settings.palette = *palette;
    let (level, lanes) = shown(&app.plan, &app.schedule, &settings);
    let first = app.schedule.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(app.today);
    let last = app.schedule.slots.iter().map(|slot| slot.planned.finish).max().unwrap_or(app.today);
    let span = format!("{} – {}", first.strftime("%b %Y"), last.strftime("%b %Y")).to_uppercase();
    let switch_w = 34;
    let room = l.panel.width.saturating_sub(4 + switch_w) as usize;
    let mode = if app.before.is_some() { "WHAT-IF · " } else { "" };
    let title = if lanes.len() < app.plan.lanes.len() {
        format!("{mode}ROADMAP · LANES {}–{} OF {}", lanes.first().map_or(0, |lane| lane + 1), lanes.last().map_or(0, |lane| lane + 1), app.plan.lanes.len())
    } else if format!(":: {mode}SWIMLANE ROADMAP · {span}").chars().count() <= room {
        format!("{mode}SWIMLANE ROADMAP · {span}")
    } else if format!(":: {mode}ROADMAP · {span}").chars().count() <= room {
        format!("{mode}ROADMAP · {span}")
    } else {
        format!("{mode}ROADMAP")
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

    let (cw, ch) = app.cell;
    if l.chart.width > 0 && l.chart.height > 0 {
        let mut scene = match &app.before {
            Some(before) => pchart_draw::what_if(&app.plan, &app.schedule, &app.calendars, &settings, &before.schedule),
            None => pchart_draw::roadmap(&app.plan, &app.schedule, &app.calendars, &settings),
        };
        scene.background = None;
        if app.question.is_some() {
            let w = l.chart.width.saturating_sub(4).min(94);
            let h = l.chart.height.min(24);
            let x = (l.chart.width - w) / 2;
            crate::question::dialog(app, &mut scene, f32::from(x) * f32::from(cw), 0.0, f32::from(w) * f32::from(cw), f32::from(h) * f32::from(ch), f32::from(ch) / 21.0);
        }
        if writing(app) {
            let w = l.chart.width.saturating_sub(4).min(94);
            let h = l.chart.height.min(if matches!(app.spot(), Some(Spot::Item(index)) if app.plan.items[index].milestone) { 14 } else { 12 });
            let x = (l.chart.width - w) / 2;
            crate::note::dialog(app, &mut scene, f32::from(x) * f32::from(cw), f32::from(ch) * 3.0_f32.min(f32::from(l.chart.height - h)), f32::from(w) * f32::from(cw), f32::from(h) * f32::from(ch), f32::from(ch) / 21.0);
        }
        placements.push(Placement { rect: l.chart, svg: scene.svg(), paper: false });
    }
    let rect = Rect { x: l.panel.x + 2, y: l.line, width: l.panel.width.saturating_sub(4), height: 1 };
    let (w, h, u) = (f32::from(rect.width) * f32::from(cw), f32::from(ch), f32::from(ch) / 21.0);
    let line = match app.spot() {
        _ if app.before.is_some() => bits::line_of_runs(&trial(app, palette), w, h, u, palette),
        Some(Spot::Place(lane, date)) => bits::line(&place(app, lane, date, palette), w, h, u, palette),
        _ => bits::line(&selection(app, level, palette), w, h, u, palette),
    };
    placements.push(Placement { rect, svg: line.svg(), paper: false });
}

/// The line under a what-if (picture 38): the selected item and how its length or its date changed,
/// then what the tried plan means for the target and when the work has to start.
fn trial(app: &App, palette: &Palette) -> Vec<MixedPart> {
    let p = *palette;
    let item = &app.plan.items[app.selected];
    let when = |schedule: &pchart_core::Schedule| {
        let span = schedule.slots[app.selected].planned;
        if item.milestone { span.finish } else { span.start }
    };
    let mut parts = vec![(None, vec![(item.name.clone(), p.bright, true)])];
    let before = app.before.as_ref();
    let length = before.map(|before| before.plan.items[app.selected].duration).filter(|&old| old != item.duration);
    let moved = before.map(|before| when(&before.schedule)).filter(|&old| old != when(&app.schedule));
    if let Some(old) = length {
        parts.push((Some((Icon::Clock, p.dim)), vec![(format!("{old} → "), p.dim, false), (format!("{} wd", item.duration), p.bright, true)]));
    } else if let Some(old) = moved {
        parts.push((Some((Icon::Calendar, p.dim)), vec![(format!("{} → ", day(old)), p.dim, false), (day(when(&app.schedule)), p.bright, true)]));
    } else if item.milestone {
        parts.push((Some((Icon::Calendar, p.dim)), vec![(day(when(&app.schedule)), p.ink, false)]));
    } else {
        parts.push((Some((Icon::Clock, p.dim)), vec![(format!("{} wd", item.duration), p.ink, false)]));
    }
    if let Some(verdict) = &app.analysis.verdict {
        let (icon, tone) = match verdict.state {
            State::Holds => (Icon::Check, p.bright),
            State::Tight => (Icon::Clock, p.warn),
            State::DoesNotHold => (Icon::Cross, p.alert),
        };
        let mut runs = vec![(format!("{} is ", app.plan.items[verdict.target].name), p.ink, false)];
        if verdict.state == State::DoesNotHold {
            runs.push((format!("{} wd late", -verdict.spare), p.alert, true));
        } else {
            runs.push((pchart_draw::situation::state_words(verdict.state).to_string(), tone, true));
            runs.push((format!(" · {} wd to spare", verdict.spare), p.ink, false));
        }
        if let Some(&first) = app.schedule.chain.first().filter(|_| verdict.state != State::DoesNotHold) {
            let (name, start) = (&app.plan.items[first].name, app.schedule.slots[first].latest.start);
            let when = match start.cmp(&app.today) {
                std::cmp::Ordering::Equal => format!("{name} must start today"),
                std::cmp::Ordering::Greater => format!("{name} must start by {}", day(start)),
                std::cmp::Ordering::Less => format!("{name} should have started {}", day(start)),
            };
            runs.push((format!(" · {when}"), p.ink, false));
        }
        parts.push((Some((icon, tone)), runs));
    }
    parts
}

/// The line about the selection: its name, its team, its dates and length, and whether it decides the
/// date or what it has to spare; at the Overview level the same for its lane.
fn selection(app: &App, level: Level, palette: &Palette) -> Vec<Part> {
    let p = *palette;
    let lane = &app.plan.lanes[app.plan.lane(&app.plan.items[app.selected].lane).unwrap_or(0)];
    let team = [Some(lane.name.as_str()), lane.city.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
    let decides = |yes: bool| if yes { (Some((Icon::Bar, p.ink)), "decides the date".to_string(), p.ink, false) } else { (Some((Icon::Bar, p.muted)), "time to spare".to_string(), p.ink, false) };
    if level == Level::Overview {
        let (name, folded, from, to, _, on_chain) = folded_lane(app);
        return vec![(None, name, p.bright, true), (None, format!("Lane · {folded}"), p.muted, false), (Some((Icon::Calendar, p.dim)), format!("{} – {}", day(from), day(to)), p.ink, false), decides(on_chain)];
    }
    let item = &app.plan.items[app.selected];
    let slot = &app.schedule.slots[app.selected];
    let dates = if item.milestone { day(slot.planned.finish) } else { format!("{} – {}", day(slot.planned.start), day(slot.planned.finish)) };
    let star = item.important.then_some((Icon::Star, p.ink));
    let mut parts = vec![(star, item.name.clone(), p.bright, true)];
    if !item.milestone {
        parts.push((Some((Icon::Person, p.dim)), team, p.dim, false));
    }
    parts.push((Some((Icon::Calendar, p.dim)), dates, p.ink, false));
    let note = item.note.clone().map(|note| (Some((Icon::Note, p.dim)), note, p.ink, false));
    if app.notes.is_some() {
        return parts.into_iter().chain(note).collect();
    }
    if !item.milestone {
        parts.push((Some((Icon::Clock, p.dim)), format!("{} wd", item.duration), p.ink, false));
    }
    if slot.spare < 0 {
        parts.push((Some((Icon::Bar, p.alert)), format!("{} wd late", -slot.spare), p.alert, false));
    } else if app.schedule.chain.contains(&app.selected) {
        parts.push(decides(true));
    } else {
        parts.push((Some((Icon::Bar, p.muted)), format!("{} wd to spare", slot.spare), p.ink, false));
    }
    parts.into_iter().chain(note).collect()
}

/// The line under the notes cursor on an empty place (picture 43): the lane and its city, the day and
/// its week, and the note there.
fn place(app: &App, lane: usize, date: Date, palette: &Palette) -> Vec<Part> {
    let p = *palette;
    let lane_of = &app.plan.lanes[lane];
    let team = [Some(lane_of.name.as_str()), lane_of.city.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
    let mut parts = vec![(Some((Icon::Person, p.dim)), team, p.bright, true), (Some((Icon::Calendar, p.dim)), format!("{} · W{:02}", date.strftime("%a %d %b %Y"), date.iso_week_date().week()), p.ink, false)];
    parts.extend(app.note_here().map(|note| (Some((Icon::Note, p.dim)), note.to_string(), p.ink, false)));
    parts
}

/// A note is being written.
fn writing(app: &App) -> bool {
    app.notes.as_ref().is_some_and(|notes| notes.typed.is_some())
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
        let svg = |placements: &[Placement]| placements.iter().map(|placement| placement.svg.as_str()).collect::<String>();
        let (rows, placements) = screen(&app, 170, 50);
        let all = svg(&placements);
        assert!(has(&rows, ":: CHECKS · 1 WARNING · 1 HINT") && has(&rows, ":: IN THE ROADMAP · 18 JAN – 12 MAR 2027"));
        for text in ["1 thing to look at", "No buffer before a fixed date", "Acceptance Fri 26 Feb  →  Launch Mon 01 Mar", "Requirements phase ends without a milestone", "6 other checks found nothing", "5 holidays accounted for", "If Acceptance slips by a day, Launch is late.", "Add a 5-day buffer before Launch:", "still on track, with "] {
            assert!(all.contains(text), "{text}");
        }
        assert!(!all.contains("Its last item is Requirements") && placements.len() == 7);
        let (rows, placements) = screen(&app, 80, 24);
        assert!(!has(&rows, "IN THE ROADMAP") && svg(&placements).contains("Add a 5-day buffer before Launch:"));
        app.choose_check(1);
        let (rows, placements) = screen(&app, 170, 50);
        assert!(has(&rows, ":: IN THE ROADMAP · 12 OCT – 04 DEC 2026") && svg(&placements).contains("Add a milestone after Requirements.") && svg(&placements).contains("Its last item is Requirements, Fri 27 Nov"));
        app.show_check_in_roadmap();
        assert_eq!((app.screen, app.plan.items[app.selected].id.as_str()), (Screen::Roadmap, "requirements"));
    }

    #[test]
    fn the_export_screen_writes_what_its_preview_shows() {
        let dir = std::env::temp_dir().join(format!("pchart-export-{}", std::process::id()));
        let mut app = app();
        app.path = dir.join("portal.pchart").display().to_string();
        app.screen = Screen::Export;
        let chosen = |svg: &str, text: &str| svg.contains(&format!("font-weight=\"700\" xml:space=\"preserve\">{text}</text>"));
        let (rows, placements) = screen(&app, 170, 50);
        let options = &placements[0].svg;
        assert!(has(&rows, ":: EXPORT") && has(&rows, ":: PREVIEW · EXACTLY WHAT THE FILE LOOKS LIKE") && placements.len() == 2);
        assert!(chosen(options, "Milestone chart") && chosen(options, "Team") && chosen(options, "PDF") && !chosen(options, "PNG") && options.contains("Markdown"));
        assert!(options.contains("portal-milestones.pdf") && !options.contains("16:9"));
        app.export_change(-1);
        app.export_part(1);
        app.export_change(1);
        app.export_change(1);
        let (_, placements) = screen(&app, 170, 50);
        let options = &placements[0].svg;
        assert!(chosen(options, "Gantt chart") && chosen(options, "Client") && options.contains("hides what items have to spare,") && options.contains("warnings"));
        app.export_part(1);
        app.export_change(1);
        app.export_change(1);
        app.export_toggle();
        app.export_change(1);
        app.export_toggle();
        let names: Vec<String> = crate::export::files(&app).iter().map(|file| file.file_name().unwrap().to_string_lossy().into_owned()).collect();
        assert_eq!(names, ["portal-gantt.pdf", "portal-gantt.svg", "portal-gantt.png", "portal.md", "portal-milestones.svg"]);
        let (rows, placements) = screen(&app, 80, 24);
        assert!(placements.len() == 1 && placements[0].svg.contains("portal-gantt.pdf, .svg, .png, portal.md") && placements[0].svg.contains("The preview needs 110 columns.") && !has(&rows, "PREVIEW"));

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
        let line = |placements: &[Placement]| placements.last().map(|placement| placement.svg.clone()).unwrap_or_default();
        let (rows, placements) = screen(&app, 242, 61);
        assert!(rows[2].contains(":: TARGET · FIXED") && has(&rows, ":: SWIMLANE ROADMAP") && !has(&rows, ":: SELECTION") && !has(&rows, ":: CHECKS · 1 WARNING"));
        assert!(line(&placements).contains("Frontend · Bangalore") && line(&placements).contains("decides the date") && placements.len() == 6);
        let (rows, placements) = screen(&app, 120, 36);
        assert!(rows[1].contains("on track · 12 wd to spare") && line(&placements).contains("Mon 21 Dec – Wed 27 Jan"));
        let (rows, _) = screen(&app, 80, 24);
        assert!(rows.iter().any(|row| row.contains("ROADMAP · LANES") && row.contains("OF 7")));
        let (rows, placements) = screen(&app, 50, 14);
        assert!(placements.is_empty() && rows.iter().any(|row| row.contains("Terminal size too small:")) && rows.iter().any(|row| row.contains("Width = 60 Height = 16")));
    }

    #[test]
    fn a_lane_of_milestones_stands_for_its_milestones() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        app.level = Level::Overview;
        app.selected = app.plan.item("acceptance").unwrap();
        let (_, placements) = screen(&app, 170, 50);
        let line = &placements.last().unwrap().svg;
        assert!(line.contains("Lane · 3 milestones") && line.contains("Fri 18 Dec – Mon 01 Mar") && line.contains("decides the date") && !line.contains("0 items"));
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

    #[test]
    fn the_date_question_changes_a_start_and_u_takes_it_back() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let frontend = app.plan.item("frontend").unwrap();
        app.selected = frontend;
        app.ask();
        assert!(app.answer().is_none() && app.question.as_ref().is_some_and(|question| question.choice == 0 && question.typed.is_empty()));
        let question = app.question.as_mut().unwrap();
        question.choice = 2;
        question.typed = "4 jan".into();
        let date = jiff::civil::date(2027, 1, 4);
        assert_eq!(app.answer(), Some(pchart_core::Edit::Fix { item: "frontend".into(), kind: pchart_core::Fix::NoEarlierThan, date }));
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, "CHANGE A DATE · nothing changed yet") && has(&rows, "keep as it is"));
        for text in ["When does Frontend start?", "After Design sign-off", "not before this date, later if the plan needs it", "Mon 04 Jan 2027", "Launch would be ", "9 working days late.", " moves to "] {
            assert!(svg.contains(text), "{text}");
        }

        app.confirm();
        assert!(app.question.is_none() && app.unsaved && app.history.len() == 1);
        assert_eq!(app.plan.items[frontend].no_earlier_than, Some(date));
        assert_eq!(app.analysis.verdict.as_ref().map(|verdict| (verdict.state, verdict.spare)), Some((pchart_core::analysis::State::DoesNotHold, -9)));
        app.ask();
        let question = app.question.as_mut().unwrap();
        assert!(question.choice == 2 && question.typed == "04 Jan 2027");
        question.choice = 0;
        assert_eq!(app.answer(), Some(pchart_core::Edit::Unfix { item: "frontend".into() }));
        app.question.as_mut().unwrap().choice = 1;
        app.question.as_mut().unwrap().typed = "soon".into();
        app.confirm();
        assert!(app.question.is_some() && app.history.len() == 1 && app.message.as_deref() == Some("Type a date, such as 4 jan or next friday"));
        app.question = None;

        app.unsaved = false;
        app.undo();
        assert!(app.unsaved && app.history.is_empty() && app.plan.items[frontend].no_earlier_than.is_none() && app.message.as_deref() == Some("Undone: Frontend start"));
        assert_eq!(app.analysis.verdict.as_ref().map(|verdict| verdict.state), Some(pchart_core::analysis::State::Holds));
        app.undo();
        assert_eq!(app.message.as_deref(), Some("Nothing to undo"));
    }

    #[test]
    fn a_what_if_writes_nothing_until_enter_keeps_it() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let frontend = app.plan.item("frontend").unwrap();
        app.selected = frontend;
        let (_, placements) = screen(&app, 170, 50);
        let dashes = |placements: &[Placement]| placements.iter().map(|placement| placement.svg.matches("stroke-dasharray").count()).sum::<usize>();
        let before = dashes(&placements);
        for _ in 0..12 {
            app.try_length(1);
        }
        assert!(app.before.is_some() && !app.unsaved && app.history.is_empty() && app.changes() == ["Frontend +12 wd"]);
        assert_eq!(app.analysis.verdict.as_ref().map(|verdict| (verdict.state, verdict.spare)), Some((pchart_core::analysis::State::Tight, 0)));
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        for text in ["WHAT-IF · Frontend +12 wd · not saved", ":: WHAT-IF · SWIMLANE ROADMAP", "today", "was 12", "Esc discard", "←→ keep changing"] {
            assert!(has(&rows, text), "{text}");
        }
        for text in ["25 → ", "37 wd", "Launch is ", "at risk", " · 0 wd to spare", " · Requirements must start today"] {
            assert!(svg.contains(text), "{text}");
        }
        assert!(dashes(&placements) > before + 5);

        app.discard();
        assert!(app.before.is_none() && app.plan.items[frontend].duration == 25 && app.analysis.verdict.as_ref().is_some_and(|verdict| verdict.spare == 12));
        app.try_length(1);
        app.try_length(1);
        app.keep();
        assert!(app.before.is_none() && app.unsaved && app.plan.items[frontend].duration == 27 && app.history.len() == 1);
        app.undo();
        assert!(app.plan.items[frontend].duration == 25 && app.message.as_deref() == Some("Undone: Frontend +2 wd"));
        app.unsaved = false;

        app.ask();
        let question = app.question.as_mut().unwrap();
        question.choice = 2;
        question.typed = "4 jan".into();
        app.try_answer();
        assert!(app.question.is_none() && app.before.is_some() && !app.unsaved && app.changes() == ["Frontend start"]);
        let svg: String = screen(&app, 170, 50).1.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(svg.contains("Mon 21 Dec → ") && svg.contains("Mon 04 Jan") && svg.contains("9 wd late") && !svg.contains("must start"));
        app.discard();
        app.selected = app.plan.item("launch").unwrap();
        app.try_length(1);
        assert!(app.before.is_none() && app.message.as_deref() == Some("Launch is a milestone; it has no duration"));
    }

    #[test]
    fn notes_go_on_items_and_places_and_a_milestone_takes_a_star() {
        let mut app = app();
        app.screen = Screen::Roadmap;
        let (frontend, sign_off) = (app.plan.item("frontend").unwrap(), app.plan.item("design-sign-off").unwrap());
        app.selected = frontend;
        app.add_notes();
        assert_eq!(app.spot(), Some(Spot::Item(frontend)));
        app.cursor_lane(2);
        let qa = app.plan.lane("qa").unwrap();
        assert_eq!(app.spot(), Some(Spot::Place(qa, jiff::civil::date(2026, 12, 21))));
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, "ADD NOTES") && has(&rows, "Enter write a note here") && !has(&rows, "i important"));
        assert!(svg.contains("Test/QA · Bangalore") && svg.contains("Mon 21 Dec 2026 · W52"));
        app.write();
        app.notes.as_mut().unwrap().typed = Some("Meet QA".into());
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, "NOTE · nothing changed yet") && has(&rows, "Enter keep the note") && svg.contains("What should people know about Mon 21 Dec?") && svg.contains("Meet QA"));
        app.keep_note();
        assert!(app.plan.notes.len() == 1 && app.plan.notes[0].text == "Meet QA" && app.unsaved && app.notes.as_ref().is_some_and(|notes| notes.typed.is_none()));

        let svg = |app: &App| screen(app, 170, 50).1.iter().map(|placement| placement.svg.clone()).collect::<String>();
        assert!(svg(&app).contains(">Meet QA<"));
        app.cursor_lane(1);
        app.move_cursor(-3);
        assert_eq!(app.spot(), Some(Spot::Item(sign_off)));
        assert!(has(&screen(&app, 170, 50).0, "i important"));
        app.toggle_important();
        app.write();
        app.notes.as_mut().unwrap().typed = Some("Client signs the design in a workshop in New York.".into());
        app.keep_note();
        let item = &app.plan.items[sign_off];
        assert!(item.important && item.note.as_deref() == Some("Client signs the design in a workshop in New York.") && app.history.len() == 3);

        assert_eq!(svg(&app).matches("Client signs the design").count(), 2);
        app.notes = None;
        assert!(svg(&app).matches("Client signs the design").count() == 1 && !svg(&app).contains(">Meet QA<"));
        let (rows, placements) = screen(&app, 170, 50);
        let line = &placements.last().unwrap().svg;
        assert!(has(&rows, "n add notes") && line.contains("Design sign-off") && line.contains("Client signs the design in a workshop in New York."));
        app.undo();
        assert!(app.plan.items[sign_off].note.is_none() && app.message.as_deref() == Some("Undone: note on Design sign-off"));
    }

    #[test]
    fn the_quiet_lines_open_the_checks_that_found_nothing_and_the_holidays() {
        let mut app = app();
        app.screen = Screen::Checks;
        app.choose_check(2);
        let (rows, placements) = screen(&app, 170, 50);
        let svg: String = placements.iter().map(|placement| placement.svg.as_str()).collect();
        assert!(has(&rows, ":: CHECKED · NOTHING FOUND") && !has(&rows, "Enter show in roadmap"));
        for text in ["Every fixed date holds", "No item is longer than six weeks", "Two items at once by one person: the plan names no one"] {
            assert!(svg.contains(text), "{text}");
        }
        app.choose_check(1);
        let rows = screen(&app, 170, 50).0;
        assert!(has(&rows, ":: HOLIDAYS ACCOUNTED FOR · 5") && has(&rows, "Christmas Day") && has(&rows, "DE-BE") && has(&rows, "Republic Day"));
        assert!(has(&screen(&app, 80, 24).0, "Republic Day"));
        app.show_check_in_roadmap();
        assert_eq!(app.screen, Screen::Checks);
        app.choose_check(1);
        assert_eq!(app.check, 0);
    }
}
