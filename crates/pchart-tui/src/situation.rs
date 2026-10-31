//! The situation screen (pictures 5, 19 and 20): where do we stand, who is waiting on whom, what is
//! burning next, and the chain that decides the date.

use jiff::civil::Date;
use pchart_core::analysis::{Check, Handoff, Next, Severity, State};
use pchart_draw::{bits, situation};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;

use crate::app::App;
use crate::pictures::Placement;
use crate::ui::{Inks, bold, cut, day, panel, put, width};

/// A picture of `rect`, in pixels.
fn pixels(app: &App, rect: Rect) -> (f32, f32) {
    (f32::from(rect.width) * f32::from(app.cell.0), f32::from(rect.height) * f32::from(app.cell.1))
}

fn unit(app: &App) -> f32 {
    f32::from(app.cell.1) / 21.0
}

/// Bold text `size` pixels high for a 21 px cell, as a picture of `rows` rows at `x`, `y`.
#[allow(clippy::too_many_arguments)]
fn big(app: &App, placements: &mut Vec<Placement>, x: u16, y: u16, rows: u16, text: &str, size: f32, ink: pchart_draw::scene::Color, room: u16) -> u16 {
    let font = size * unit(app);
    let cols = ((text.chars().count() as f32 * 0.6 * font) / f32::from(app.cell.0)).ceil() as u16 + 1;
    let rect = Rect { x, y, width: cols.min(room), height: rows };
    let (w, h) = pixels(app, rect);
    placements.push(Placement { rect, svg: bits::big(text, font, ink, w, h, &app.palette).svg(), paper: false });
    rect.width
}

/// A lane in a sentence: "the client" for a lane called Client or Customer, otherwise its name.
fn who(app: &App, lane: usize, first: bool) -> String {
    let name = &app.plan.lanes[lane].name;
    match name.to_lowercase().as_str() {
        "client" | "customer" => format!("{}he {}", if first { "T" } else { "t" }, name.to_lowercase()),
        _ => name.clone(),
    }
}

fn city(app: &App, lane: usize) -> Option<String> {
    app.plan.lanes[lane].city.clone()
}

fn weeks(app: &App, due: Date) -> String {
    let days = app.today.until(due).map_or(0, |span| span.get_days());
    match days / 7 {
        1 => "in 1 week".to_string(),
        weeks if weeks > 0 => format!("in {weeks} weeks"),
        _ => "this week".to_string(),
    }
}

pub fn draw(app: &App, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    if area.width >= 150 && area.height >= 44 {
        let chain_h = 15;
        let lists_y = 12;
        let lists_h = area.height - 1 - chain_h - lists_y;
        verdict(app, frame, inks, Rect { x: 0, y: 1, width: area.width, height: 11 }, placements, true);
        let left = area.width * 86 / 170;
        waiting(app, frame, inks, Rect { x: 0, y: lists_y, width: left, height: lists_h }, true);
        burning(app, frame, inks, Rect { x: left, y: lists_y, width: area.width - left, height: lists_h }, placements, 3, true);
        chain(app, frame, inks, Rect { x: 0, y: lists_y + lists_h, width: area.width, height: chain_h }, placements);
        return;
    }
    let tall = area.height >= 30;
    let verdict_h = if tall { 8 } else { 7 };
    verdict(app, frame, inks, Rect { x: 0, y: 1, width: area.width, height: verdict_h }, placements, false);
    let mut y = 1 + verdict_h;
    if tall {
        let left = 56.min(area.width / 2);
        burning(app, frame, inks, Rect { x: 0, y, width: left, height: 15 }, placements, 3, false);
        chain_list(app, frame, inks, Rect { x: left, y, width: area.width - left, height: 15 });
        y += 15;
    } else {
        burning(app, frame, inks, Rect { x: 0, y, width: area.width, height: 8 }, placements, 2, false);
        y += 8;
    }
    waiting(app, frame, inks, Rect { x: 0, y, width: area.width, height: area.height - 1 - y }, false);
}

/// The sentence under the headline, with its numbers in bold.
fn sentence(app: &App, inks: &Inks) -> Vec<Span<'static>> {
    let Some(verdict) = &app.analysis.verdict else { return Vec::new() };
    let first = app.plan.items[verdict.first].name.clone();
    let wd = |count: i32| if count.abs() == 1 { format!("{count} working day") } else { format!("{count} working days") };
    match verdict.state {
        State::Holds => vec![Span::styled(format!("{} to spare", wd(verdict.spare)), bold(inks.bright)), Span::styled(format!(", with {first} starting by "), inks.ink), Span::styled(day(verdict.latest_start), bold(inks.bright)), Span::styled(".", inks.ink)],
        State::Tight => vec![Span::styled(format!("Only {} to spare", wd(verdict.spare)), bold(inks.warn)), Span::styled(format!(", with {first} starting by "), inks.ink), Span::styled(day(verdict.latest_start), bold(inks.bright)), Span::styled(".", inks.ink)],
        State::DoesNotHold => vec![Span::styled(format!("{} late", wd(-verdict.spare)), bold(inks.alert)), Span::styled(format!(", even with {first} starting today."), inks.ink)],
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

fn verdict(app: &App, frame: &mut Frame, inks: &Inks, rect: Rect, placements: &mut Vec<Placement>, large: bool) {
    let title = format!("WHERE DO WE STAND · {}", app.today.strftime("%a %d %b %Y")).to_uppercase();
    panel(frame, inks, rect, &title, false);
    let p = app.palette;
    let right = if large { 44 } else { 0 };
    let room = rect.width.saturating_sub(4 + right + 4);
    let Some(verdict) = &app.analysis.verdict else {
        let end = app.schedule.slots.iter().map(|slot| slot.planned.finish).max().unwrap_or(app.today);
        put(frame, rect.x + 2, rect.y + 3, room, vec![Span::styled(format!("{} has no fixed target date; it ends {}.", app.plan.name, day(end)), bold(inks.bright))]);
        return;
    };
    let (headline, _) = pchart_core::text::verdict(&app.plan, verdict);
    let tone = match verdict.state {
        State::Holds => p.bright,
        State::Tight => p.warn,
        State::DoesNotHold => p.alert,
    };
    big(app, placements, rect.x + 2, rect.y + 2, 2, &headline, 34.0, tone, room);
    let state = match verdict.state {
        State::Holds => "holds",
        State::Tight => "tight",
        State::DoesNotHold => "does not hold",
    };
    if large {
        let x = rect.x + rect.width - right;
        for row in rect.y + 2..rect.y + rect.height - 1 {
            put(frame, x - 3, row, 1, vec![Span::styled("│", inks.rule)]);
        }
        let number = verdict.spare.to_string();
        let w = big(app, placements, x, rect.y + 2, 3, &number, 64.0, tone, right - 12);
        put(frame, x + w + 1, rect.y + 3, 14, vec![Span::styled("working days", inks.dim)]);
        put(frame, x + w + 1, rect.y + 4, 14, vec![Span::styled(if verdict.spare < 0 { "late" } else { "to spare" }, inks.dim)]);
        let mut at = x;
        for (name, cols) in [("does not hold", 15), ("tight", 10), ("holds", 15)] {
            let rect = Rect { x: at, y: rect.y + 6, width: cols, height: 1 };
            let (w, h) = pixels(app, rect);
            placements.push(Placement { rect, svg: situation::segment(name == state, w, h, unit(app), &app.palette).svg(), paper: false });
            put(frame, at, rect.y + 1, cols, vec![Span::styled(name, if name == state { bold(inks.bright) } else { inks.muted })]);
            at += cols + 1;
        }
        put(frame, x, rect.y + 8, right - 2, vec![Span::styled("tight means less than 5 wd to spare", inks.muted)]);
        put(frame, rect.x + 2, rect.y + 5, room, sentence(app, inks));
        return;
    }
    let mut at = rect.x + 2;
    for (name, cols) in [("does not hold", 2), ("tight", 2), ("holds", 3)] {
        let bar = Rect { x: at, y: rect.y + 4, width: cols, height: 1 };
        let (w, h) = pixels(app, bar);
        placements.push(Placement { rect: bar, svg: situation::segment(name == state, w, h, unit(app), &app.palette).svg(), paper: false });
        at += cols + 1;
        put(frame, at, rect.y + 4, name.chars().count() as u16, vec![Span::styled(name, if name == state { bold(inks.bright) } else { inks.muted })]);
        at += name.chars().count() as u16 + 1;
    }
    let first_row = if rect.height >= 8 { rect.y + 6 } else { rect.y + 5 };
    for (offset, line) in wrap(sentence(app, inks), rect.width.saturating_sub(4)).into_iter().enumerate() {
        let row = first_row + offset as u16;
        if row + 1 >= rect.y + rect.height {
            break;
        }
        put(frame, rect.x + 2, row, rect.width.saturating_sub(4), line);
    }
}

/// The lines of a statement in "What is burning next": its title, then what it is about.
fn statement(app: &App, next: &Next) -> (Option<Severity>, String, Vec<String>) {
    let lane_of = |index: usize| app.plan.lane(&app.plan.items[index].lane).unwrap_or(0);
    let place = |index: usize| {
        let lane = lane_of(index);
        match city(app, lane) {
            Some(city) => format!("{} in {city}", app.plan.lanes[lane].name),
            None => app.plan.lanes[lane].name.clone(),
        }
    };
    let from_today = |days: i32| match days {
        0 => "today".to_string(),
        1 => "1 working day from today".to_string(),
        days => format!("{days} working days from today"),
    };
    match next {
        Next::StartBy { item, date, in_wd } => {
            let mut lines = vec![format!("{} · {}", from_today(*in_wd), place(*item))];
            if app.schedule.chain.first() == Some(item) {
                lines.push("First step of the chain that decides the date".into());
            }
            (None, format!("Start {} by {}", app.plan.items[*item].name, day(*date)), lines)
        }
        Next::Due { item, date, in_wd } => (None, format!("{} is due {}", app.plan.items[*item].name, day(*date)), vec![format!("{} · {}", from_today(*in_wd), place(*item))]),
        Next::Check(index) => {
            let check = &app.analysis.checks[*index];
            match *check {
                Check::NoBuffer { before, target } => {
                    let name = &app.plan.items[target].name;
                    let lower = crate::ui::common(name);
                    let date = |index: usize| day(app.schedule.slots[index].planned.finish);
                    (Some(check.severity()), format!("No buffer before the fixed {lower}"), vec![format!("{} {}, then {name} {}", app.plan.items[before].name, date(before), date(target)), "Nothing sits between them. Add a buffer?".into()])
                }
                _ => {
                    let (title, detail) = pchart_core::text::check(&app.plan, check);
                    (Some(check.severity()), title.trim_end_matches('.').to_string(), vec![detail])
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn burning(app: &App, frame: &mut Frame, inks: &Inks, rect: Rect, placements: &mut Vec<Placement>, lines: u16, large: bool) {
    panel(frame, inks, rect, "WHAT IS BURNING NEXT", true);
    let text_x = rect.x + 8;
    let room = rect.width.saturating_sub(10);
    let bottom = rect.y + rect.height - 1;
    let mut y = rect.y + 2;
    for (position, next) in app.analysis.next.iter().enumerate() {
        if y + lines > bottom {
            break;
        }
        let (severity, title, detail) = statement(app, next);
        if position == app.statement {
            let band = " ".repeat(rect.width.saturating_sub(2) as usize);
            for row in y..y + lines {
                put(frame, rect.x + 1, row, rect.width.saturating_sub(2), vec![Span::styled(band.clone(), Style::new().bg(inks.raised))]);
            }
            put(frame, rect.x + 2, y, 1, vec![Span::styled("›", bold(Style::new().fg(inks.accent)).bg(inks.raised))]);
        }
        let ground = |style: Style| if position == app.statement { style.bg(inks.raised) } else { style };
        let (numeral, mark) = match severity {
            Some(Severity::Problem) => (app.palette.alert, Some(inks.alert)),
            Some(Severity::Warning) => (app.palette.warn, Some(inks.warn)),
            _ => (app.palette.bright, None),
        };
        big(app, placements, rect.x + 4, y, 2.min(lines), &(position + 1).to_string(), 34.0, numeral, 3);
        let mut head = Vec::new();
        if let Some(mark) = mark {
            head.push(Span::styled("▲ ", ground(bold(mark))));
        }
        head.push(Span::styled(cut(&title, room as usize - width(&head) as usize), ground(bold(inks.bright))));
        put(frame, text_x, y, room, head);
        for (offset, line) in detail.into_iter().take(lines as usize - 1).enumerate() {
            let style = if offset == 0 { inks.dim } else { inks.muted };
            put(frame, text_x, y + 1 + offset as u16, room, vec![Span::styled(cut(&line, room as usize), ground(style))]);
        }
        y += lines + u16::from(lines == 3);
    }
    let horizon = app.plan.items.iter().enumerate().filter(|(index, item)| !item.done && app.schedule.slots[*index].planned.finish >= app.today).map(|(index, _)| app.schedule.slots[index].planned.finish).min();
    if app.analysis.next.is_empty() {
        put(frame, rect.x + 3, y, rect.width.saturating_sub(4), vec![Span::styled("Nothing is due.", inks.muted)]);
    } else if let Some(horizon) = horizon
        && y < bottom
    {
        put(frame, rect.x + 3, y, rect.width.saturating_sub(4), vec![Span::styled(format!("Nothing else is due before {}.", day(horizon)), inks.muted)]);
    }
    if large && rect.height >= 21 {
        put(frame, rect.x + 3, rect.y + 13, 20, vec![Span::styled("NEXT 6 WEEKS", inks.muted)]);
        let strip = Rect { x: rect.x + 4, y: rect.y + 14, width: rect.width.saturating_sub(8), height: 6 };
        let (w, h) = pixels(app, strip);
        placements.push(Placement { rect: strip, svg: situation::weeks(&app.plan, &app.schedule, app.today, w, h, unit(app), &app.palette).svg(), paper: false });
    }
}

fn handoff_line(app: &App, inks: &Inks, handoff: &Handoff, cities: bool) -> (Vec<Span<'static>>, Vec<Span<'static>>) {
    let place = |lane: usize| if cities { city(app, lane).map(|city| format!(" in {city}")).unwrap_or_default() } else { String::new() };
    let first = vec![
        Span::styled(who(app, handoff.waiting, true), bold(inks.bright)),
        Span::styled(place(handoff.waiting), inks.muted),
        Span::styled(" waits on ", inks.dim),
        Span::styled(who(app, handoff.on, false), bold(inks.bright)),
        Span::styled(place(handoff.on), inks.muted),
    ];
    let second = vec![Span::styled(app.plan.items[handoff.item].name.clone(), inks.ink), Span::styled(" due ", inks.dim), Span::styled(day(handoff.due), inks.bright)];
    (first, second)
}

fn waiting(app: &App, frame: &mut Frame, inks: &Inks, rect: Rect, large: bool) {
    panel(frame, inks, rect, "WHO IS WAITING ON WHOM", false);
    let deciding: Vec<&Handoff> = app.analysis.handoffs.iter().filter(|handoff| handoff.deciding).collect();
    let rest = app.analysis.handoffs.len() - deciding.len();
    let (x, room) = (rect.x + 2, rect.width.saturating_sub(4));
    let bottom = rect.y + rect.height - 1;
    if large {
        put(frame, x, rect.y + 2, room, vec![Span::styled("On the chain that decides the date", inks.muted)]);
        let mut y = rect.y + 4;
        let mut shown = 0;
        for handoff in &deciding {
            if y + 2 > bottom.saturating_sub(2) {
                break;
            }
            let (first, second) = handoff_line(app, inks, handoff, true);
            put(frame, x, y, 1, vec![Span::styled("▎", inks.ink)]);
            put(frame, x, y + 1, 1, vec![Span::styled("▎", inks.ink)]);
            put(frame, x + 2, y, room - 2, first);
            put(frame, x + 2, y + 1, room - 2, second);
            let when = weeks(app, handoff.due);
            put(frame, x + room - when.chars().count() as u16, y + 1, when.chars().count() as u16, vec![Span::styled(when, inks.muted)]);
            y += 3;
            shown += 1;
        }
        let more = deciding.len() - shown;
        let text = match (more, rest) {
            (0, 0) => String::new(),
            (0, rest) => format!("{rest} more handoffs with time to spare"),
            (more, rest) => format!("{more} more on the chain · {rest} with time to spare"),
        };
        if !text.is_empty() {
            put(frame, x, bottom - 2, room, vec![Span::styled("▸ ", bold(inks.dim)), Span::styled(text, inks.dim)]);
        }
        return;
    }
    let mut y = rect.y + 2;
    let mut lines = bottom.saturating_sub(y);
    if lines as usize >= deciding.len() + 2 {
        put(frame, x, y, room, vec![Span::styled("On the chain that decides the date", inks.muted)]);
        y += 1;
        lines -= 1;
    }
    let shown = deciding.len().min(lines.saturating_sub(1) as usize);
    for handoff in deciding.iter().take(shown) {
        let (mut first, second) = handoff_line(app, inks, handoff, false);
        first.push(Span::styled(": ", inks.dim));
        first.extend(second);
        let when = weeks(app, handoff.due);
        put(frame, x, y, 1, vec![Span::styled("▎", inks.ink)]);
        put(frame, x + 2, y, room.saturating_sub(when.chars().count() as u16 + 3), first);
        put(frame, x + room - when.chars().count() as u16, y, when.chars().count() as u16, vec![Span::styled(when, inks.muted)]);
        y += 1;
    }
    if (lines as usize).saturating_sub(1 + shown) >= 1 {
        y += 1;
    }
    let more = deciding.len() - shown;
    let text = match (more, rest) {
        (0, 0) => String::new(),
        (0, rest) => format!("+ {rest} more with time to spare"),
        (more, rest) => format!("+ {more} more on the chain · {rest} with time to spare"),
    };
    if !text.is_empty() && y < bottom {
        put(frame, x, y, room, vec![Span::styled(text, inks.dim)]);
    }
}

/// "Everything else has time to spare: Data model 11 wd · …", cut with "+N more".
fn others(app: &App, inks: &Inks, room: u16, gap: &str) -> Vec<Span<'static>> {
    let others: Vec<usize> = (0..app.plan.items.len()).filter(|index| !app.schedule.chain.contains(index) && !app.plan.items[*index].done).collect();
    let mut spans = vec![Span::styled(format!("Everything else has time to spare:{gap}"), inks.muted)];
    if others.is_empty() {
        return Vec::new();
    }
    for (position, &index) in others.iter().enumerate() {
        let mut piece = Vec::new();
        if position > 0 {
            piece.push(Span::styled("  ·  ", inks.muted));
        }
        let spare = app.schedule.slots[index].spare;
        piece.push(Span::styled(app.plan.items[index].name.clone(), inks.ink));
        piece.push(Span::styled(format!(" {spare} wd"), if spare < 0 { inks.alert } else { inks.dim }));
        let left = others.len() - position - 1;
        let tail = if left > 0 { format!("  +{left} more") } else { String::new() };
        if width(&spans) + width(&piece) + tail.chars().count() as u16 <= room {
            spans.extend(piece);
            continue;
        }
        spans.push(Span::styled(format!("  +{} more", others.len() - position), inks.muted));
        break;
    }
    spans
}

fn chain(app: &App, frame: &mut Frame, inks: &Inks, rect: Rect, placements: &mut Vec<Placement>) {
    let count = app.schedule.chain.len();
    let picture = Rect { x: rect.x + 1, y: rect.y + 2, width: rect.width.saturating_sub(2), height: 9 };
    let (w, h) = pixels(app, picture);
    let step = if count > 1 { (w - 180.0 * unit(app)) / (count - 1) as f32 / f32::from(app.cell.0) } else { f32::MAX };
    if step < 14.0 {
        chain_list(app, frame, inks, rect);
        return;
    }
    panel(frame, inks, rect, "THIS CHAIN DECIDES THE DATE", false);
    placements.push(Placement { rect: picture, svg: situation::chain(&app.plan, &app.schedule, w, h, unit(app), &app.palette).svg(), paper: false });
    put(frame, rect.x + 2, rect.y + 12, rect.width.saturating_sub(4), others(app, inks, rect.width.saturating_sub(4), "  "));
}

/// The chain as a list, one step per row, with the lanes beside it.
fn chain_list(app: &App, frame: &mut Frame, inks: &Inks, rect: Rect) {
    panel(frame, inks, rect, "THIS CHAIN DECIDES THE DATE", false);
    let (name_x, date_x, lane_x) = (rect.x + 4, rect.x + 20, rect.x + 41);
    let bottom = rect.y + rect.height - 1;
    let mark = app.analysis.next.iter().find_map(|next| match next {
        Next::StartBy { item, .. } | Next::Due { item, .. } => Some(*item),
        Next::Check(_) => None,
    });
    let chain = &app.schedule.chain;
    let rows = (bottom.saturating_sub(rect.y + 3)) as usize;
    let mut y = rect.y + 2;
    for &index in chain.iter().take(rows) {
        let item = &app.plan.items[index];
        let slot = &app.schedule.slots[index].planned;
        let on = Some(index) == mark;
        if on {
            put(frame, rect.x + 2, y, 1, vec![Span::styled("›", bold(inks.ink))]);
        }
        put(frame, name_x, y, 15, vec![Span::styled(cut(&item.name, 15), if on { bold(inks.bright) } else { inks.ink })]);
        let date = if item.on.is_some() {
            format!("{} · fixed", day(slot.finish))
        } else if item.milestone {
            day(slot.finish)
        } else {
            format!("{} – {}", slot.start.strftime("%d %b"), slot.finish.strftime("%d %b"))
        };
        put(frame, date_x, y, 20, vec![Span::styled(date, if item.on.is_some() { inks.bright } else { inks.ink })]);
        y += 1;
    }
    let shown = chain.len().min(rows);
    let mut first = 0;
    while first < shown && lane_x + 4 < rect.x + rect.width {
        let lane = app.plan.lane(&app.plan.items[chain[first]].lane).unwrap_or(0);
        let mut last = first;
        while last + 1 < shown && app.plan.lane(&app.plan.items[chain[last + 1]].lane) == Some(lane) {
            last += 1;
        }
        for row in first..=last {
            let symbol = match (row == first, row == last) {
                (true, true) => "[",
                (true, false) => "┌",
                (false, true) => "└",
                _ => "│",
            };
            put(frame, lane_x - 2, rect.y + 2 + row as u16, 1, vec![Span::styled(symbol, inks.muted)]);
        }
        let label = [Some(app.plan.lanes[lane].name.as_str()), app.plan.lanes[lane].city.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
        let room = (rect.x + rect.width).saturating_sub(lane_x + 2);
        put(frame, lane_x, rect.y + 2 + first as u16, room, vec![Span::styled(cut(&label, room as usize), inks.dim)]);
        first = last + 1;
    }
    if y + 1 < bottom {
        put(frame, rect.x + 2, y + 1, rect.width.saturating_sub(4), others(app, inks, rect.width.saturating_sub(4), " "));
    }
}
