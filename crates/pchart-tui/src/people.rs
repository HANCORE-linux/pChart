//! The People screen (pictures 64): in a workspace, how much of their time each team and each person
//! has planned, week by week across the projects; under it the work of the chosen one, and a line
//! about the chosen week.

use jiff::civil::{Date, Weekday};
use pchart_core::text::span;
use pchart_draw::bits::{self, Icon};
use pchart_draw::people::{self, Line, Row, Work};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::{App, Crew};
use crate::pictures::Placement;
use crate::ui::{Inks, panel};

/// The people of the workspace as rows, in the order `workspace.toml` names them; with `only`, that
/// one person.
pub(crate) fn rows(crew: &Crew, only: Option<&str>) -> Vec<Row> {
    crew.workspace
        .people
        .iter()
        .filter(|person| only.is_none_or(|name| name == person.name))
        .map(|person| {
            let capacity = crew.workspace.capacity(&person.name);
            let team = [person.team.clone(), person.team.as_deref().and_then(|team| crew.workspace.city(team)).map(str::to_string)].into_iter().flatten().collect::<Vec<_>>().join(" · ");
            let work = pchart_core::people::work(&crew.projects, &person.name)
                .into_iter()
                .map(|(project, item, dates)| {
                    let of = &crew.projects[project];
                    let off = std::iter::successors(Some(dates.start), |day| day.tomorrow().ok()).take_while(|day| *day <= dates.finish).filter(|&day| !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday) && !of.on(item, day)).collect();
                    Work { name: of.plan.items[item].name.clone(), project: of.name.clone(), start: dates.start, finish: dates.finish, share: pchart_core::people::share(&of.plan, item), off }
                })
                .collect();
            Row { name: person.name.clone(), team, work, away: person.away.iter().map(|away| (away.from, away.to)).collect(), capacity }
        })
        .collect()
}

pub(crate) fn monday(mut day: Date) -> Date {
    while day.weekday() != Weekday::Monday {
        day = day.yesterday().expect("a date in range");
    }
    day
}

/// The weeks the rows need: from the Monday of the first work, or of today, to a week after the last.
pub(crate) fn weeks(rows: &[Row], today: Date) -> (Date, Date) {
    let first = rows.iter().flat_map(|row| row.work.iter().map(|work| work.start)).min().unwrap_or(today).min(today);
    let last = rows.iter().flat_map(|row| row.work.iter().map(|work| work.finish)).max().unwrap_or(today).max(today);
    (monday(first), monday(last).checked_add(jiff::Span::new().days(13)).unwrap_or(last))
}

/// The lines of the capacity table: each team in the order its first person comes, with its people.
pub(crate) fn lines(rows: &[Row]) -> Vec<Line> {
    let mut teams: Vec<(String, Vec<usize>)> = Vec::new();
    for (at, row) in rows.iter().enumerate() {
        match teams.iter_mut().find(|(team, _)| *team == row.team) {
            Some((_, members)) => members.push(at),
            None => teams.push((row.team.clone(), vec![at])),
        }
    }
    teams.into_iter().flat_map(|(team, members)| std::iter::once(Line::Team(team, members.clone())).chain(members.into_iter().map(Line::Person))).collect()
}

/// Every week the table can show: from this week to a week after the last work.
pub(crate) fn all_weeks(rows: &[Row], today: Date) -> Vec<Date> {
    let (_, end) = weeks(rows, today);
    std::iter::successors(Some(monday(today)), |day| day.checked_add(jiff::Span::new().days(7)).ok()).take_while(|day| *day <= end).collect()
}

/// The line about the chosen cell: who, the week, how much of their time is planned of what they can
/// do, and the work it is.
fn line(rows: &[Row], line: &Line, monday: Date, p: &pchart_draw::Palette) -> Vec<bits::Part> {
    let friday = (0..4).fold(monday, |day, _| day.tomorrow().expect("a date in range"));
    let week = (Some((Icon::Calendar, p.dim)), format!("W{:02} · {}", monday.iso_week_date().week(), span(monday, friday)), p.ink, false);
    match line {
        Line::Team(name, members) => {
            let (planned, capacity) = people::team_week(rows, members, monday);
            let tone = if planned > capacity { p.warn } else { p.dim };
            let free = if planned > capacity { format!("{} % too much", planned - capacity) } else { format!("{} % free", capacity - planned) };
            vec![(None, name.clone(), p.bright, true), week, (Some((Icon::Clock, tone)), format!("{} % full · {planned} % of {capacity} % planned, {free}", planned * 100 / capacity.max(1)), p.ink, false)]
        }
        Line::Person(at) => {
            let row = &rows[*at];
            let load = row.week(monday);
            let tone = if load > row.capacity { p.warn } else { p.dim };
            let free = if load > row.capacity { format!("{} % too much", load - row.capacity) } else { format!("{} % free", row.capacity - load) };
            let mut parts = vec![(None, row.name.clone(), p.bright, true), week, (Some((Icon::Clock, tone)), format!("{} % full · {load} % of {} % planned, {free}", row.full(monday), row.capacity), p.ink, false)];
            let work: Vec<String> = row.work.iter().filter(|work| work.start <= friday && monday <= work.finish).map(|work| format!("{} {} %", work.name, work.share)).collect();
            if !work.is_empty() {
                parts.push((Some((Icon::Bar, p.muted)), work.join(" · "), p.ink, false));
            }
            parts
        }
    }
}

pub fn draw(app: &App, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    let Some(crew) = &app.crew else { return };
    let p = app.shown();
    let rows = rows(crew, None);
    let lines = lines(&rows);
    let weeks = all_weeks(&rows, app.today);
    let (cw, ch) = app.cell;
    let u = f32::from(ch) / 21.0;
    let chosen = (app.person.min(lines.len().saturating_sub(1)), app.week.min(weeks.len().saturating_sub(1)));
    let body = Rect { x: 0, y: 1, width: area.width, height: area.height - 3 };
    let table_h = ((pchart_draw::people::grid_height(&lines, u) / f32::from(ch)).ceil() as u16 + 3).min(body.height.saturating_sub(8)).max(8);
    let table = Rect { height: table_h, ..body };
    let rect = Rect { x: table.x + 1, y: table.y + 2, width: table.width.saturating_sub(2), height: table.height.saturating_sub(3) };
    let (w, h) = (f32::from(rect.width) * f32::from(cw), f32::from(rect.height) * f32::from(ch));
    let x0 = if w >= 1000.0 * u { 240.0 * u } else { 170.0 * u };
    let fit = (((w - x0 - 8.0 * u) / pchart_draw::people::week_width(u)).floor() as usize).max(1);
    let first = chosen.1.saturating_sub(fit - 1).min(weeks.len().saturating_sub(fit));
    let shown = &weeks[first..(first + fit).min(weeks.len())];
    let span_words = match (shown.first(), shown.last()) {
        (Some(a), Some(b)) => format!(" · {} – {}", a.strftime("%d %b"), b.strftime("%d %b %Y")),
        _ => String::new(),
    };
    panel(frame, inks, table, &format!("CAPACITY · {}{span_words}", crew.name).to_uppercase(), true);
    let scene = people::grid(&rows, &lines, shown, app.today, (chosen.0, chosen.1 - first), w, h, u, &p);
    placements.push(Placement { rect, svg: scene.svg(), paper: false });

    let below = Rect { y: table.y + table.height, height: body.height.saturating_sub(table.height), ..body };
    let members: Vec<Row> = match lines.get(chosen.0) {
        Some(Line::Team(_, members)) => members.iter().map(|&at| rows[at].clone()).collect(),
        Some(Line::Person(at)) => vec![rows[*at].clone()],
        None => Vec::new(),
    };
    if below.height >= 6 && !members.is_empty() && !shown.is_empty() {
        let who = match lines.get(chosen.0) {
            Some(Line::Team(name, _)) => name.clone(),
            _ => members[0].name.clone(),
        };
        let strip = Rect { x: below.x + 1, y: below.y + 2, width: below.width.saturating_sub(2), height: below.height.saturating_sub(3) };
        let (sw, sh) = (f32::from(strip.width) * f32::from(cw), f32::from(strip.height) * f32::from(ch));
        let fits = (1..=members.len()).rev().find(|&count| people::work_height(&members[..count], u, shown.len(), sw, shown[0]) <= sh).unwrap_or(1);
        let count = if fits < members.len() { format!(" · {} OF {}", fits, members.len()) } else { String::new() };
        panel(frame, inks, below, &format!("WORK · {who}{count}").to_uppercase(), false);
        let scene = people::work_under(&members[..fits], shown[0], shown.len(), app.today, sw, sh, u, &p);
        placements.push(Placement { rect: strip, svg: scene.svg(), paper: false });
    }
    if let (Some(line_of), Some(&monday)) = (lines.get(chosen.0), weeks.get(chosen.1)) {
        let line_rect = Rect { x: 2, y: area.height - 2, width: area.width.saturating_sub(4), height: 1 };
        let scene = bits::line(&line(&rows, line_of, monday, &p), f32::from(line_rect.width) * f32::from(cw), f32::from(ch), u, &p);
        placements.push(Placement { rect: line_rect, svg: scene.svg(), paper: false });
    }
}
