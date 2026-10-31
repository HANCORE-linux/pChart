//! The Checks screen (pictures 22 and 23): the checks by weight, what was checked and found nothing,
//! and the holidays the plan accounts for; for the chosen check the roadmap around it, what it means
//! and what can be done.

use pchart_core::analysis::{Check, Severity};
use pchart_core::text::{self, Run};
use pchart_draw::{Cut, Level, part};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;

use crate::app::App;
use crate::pictures::Placement;
use crate::situation::wrap;
use crate::ui::{Inks, bold, cut, day, holidays, panel, put};

/// The left column next to the roadmap around the chosen check, as in picture 22.
const LEFT: u16 = 74;

pub fn draw(app: &App, frame: &mut Frame, inks: &Inks, area: Rect, placements: &mut Vec<Placement>) {
    let body = Rect { x: 0, y: 1, width: area.width, height: area.height - 2 };
    let title = title(app);
    let days = holidays(app);
    if area.width >= 150 {
        let checks = Rect { width: LEFT, height: if days.is_empty() { body.height } else { body.height * 26 / 48 }, ..body };
        panel(frame, inks, checks, &title, true);
        list(app, frame, inks, checks, false);
        if !days.is_empty() {
            let rest = Rect { y: checks.y + checks.height, height: body.height - checks.height, ..checks };
            panel(frame, inks, rest, &format!("HOLIDAYS ACCOUNTED FOR · {}", days.len()), false);
            holiday_rows(app, frame, inks, rest.x, rest.y + 2, rest.width, rest.y + rest.height - 1);
        }
        chosen(app, frame, inks, Rect { x: LEFT, width: body.width - LEFT, ..body }, placements);
    } else {
        panel(frame, inks, body, &title, true);
        let bottom = body.y + body.height - 1;
        let y = list(app, frame, inks, body, true);
        if !days.is_empty() && y + 2 < bottom {
            put(frame, body.x + 3, y + 1, body.width.saturating_sub(6), vec![Span::styled(format!("HOLIDAYS ACCOUNTED FOR · {}", days.len()), inks.muted)]);
            holiday_rows(app, frame, inks, body.x, y + 2, body.width, bottom);
        }
    }
}

/// "CHECKS · 1 WARNING · 1 HINT".
fn title(app: &App) -> String {
    let mut parts = vec!["CHECKS".to_string()];
    for (severity, one, many) in [(Severity::Problem, "PROBLEM", "PROBLEMS"), (Severity::Warning, "WARNING", "WARNINGS"), (Severity::Hint, "HINT", "HINTS")] {
        match app.analysis.checks.iter().filter(|check| check.severity() == severity).count() {
            0 => {}
            1 => parts.push(format!("1 {one}")),
            count => parts.push(format!("{count} {many}")),
        }
    }
    parts.join(" · ")
}

/// A check in the list: its title, and where it is.
fn words(app: &App, check: &Check) -> (String, String) {
    let name = |index: usize| app.plan.items[index].name.clone();
    let finish = |index: usize| day(app.schedule.slots[index].planned.finish);
    match *check {
        Check::NoBuffer { before, target } => ("No buffer before a fixed date".into(), format!("{} {} → {} {}", name(before), finish(before), name(target), finish(target))),
        Check::PhaseEndsWithoutMilestone { item, .. } => (text::check(&app.plan, check).0.trim_end_matches('.').to_string(), format!("Its last item is {}, {}", name(item), finish(item))),
        Check::LongItem { item } => (text::check(&app.plan, check).0.trim_end_matches('.').to_string(), format!("{} – {}", day(app.schedule.slots[item].planned.start), finish(item))),
        _ => {
            let (title, detail) = text::check(&app.plan, check);
            (title.trim_end_matches('.').to_string(), detail)
        }
    }
}

fn runs(runs: &[Run], inks: &Inks, plain: Style) -> Vec<Span<'static>> {
    runs.iter().map(|(text, strong)| Span::styled(text.clone(), if *strong { bold(inks.bright) } else { plain })).collect()
}

/// The checks by weight and what was checked and found nothing; with `unfold`, the chosen check also
/// says in two lines what it means and what can be done. Returns the row after the list.
fn list(app: &App, frame: &mut Frame, inks: &Inks, rect: Rect, unfold: bool) -> u16 {
    let (x, w) = (rect.x, rect.width);
    let room = w.saturating_sub(9);
    let order = app.checks();
    let chosen = order.get(app.check).copied();
    let mut lines: Vec<(u16, Vec<Span<'static>>, bool)> = Vec::new();
    for (severity, heading) in [(Severity::Problem, "PROBLEMS"), (Severity::Warning, "WARNINGS"), (Severity::Hint, "HINTS")] {
        let mine: Vec<usize> = order.iter().copied().filter(|&index| app.analysis.checks[index].severity() == severity).collect();
        if !lines.is_empty() {
            lines.push((0, Vec::new(), false));
        }
        if mine.is_empty() {
            lines.push((3, vec![Span::styled(format!("{heading:<11}"), inks.muted), Span::styled("none", inks.dim)], false));
            continue;
        }
        lines.push((3, vec![Span::styled(heading, inks.muted)], false));
        for index in mine {
            let check = &app.analysis.checks[index];
            let on = Some(index) == chosen;
            let (title, place) = words(app, check);
            let mark = match severity {
                Severity::Problem => Span::styled("▲", bold(inks.alert)),
                Severity::Warning => Span::styled("▲", bold(inks.warn)),
                Severity::Hint => Span::styled("·", bold(inks.muted)),
            };
            lines.push((4, vec![mark, Span::raw(" "), Span::styled(cut(&title, room as usize), if on { bold(inks.bright) } else { inks.ink })], on));
            lines.push((6, vec![Span::styled(cut(&place, room as usize), inks.dim)], on));
            if on && unfold {
                let advice = text::advice(&app.plan, &app.calendars, &app.schedule, app.today, check);
                for short in &advice.short {
                    lines.extend(wrap(runs(short, inks, inks.ink), room).into_iter().map(|line| (6, line, true)));
                }
            }
        }
    }
    lines.push((0, Vec::new(), false));
    lines.push((3, vec![Span::styled("CHECKED, NOTHING FOUND", inks.muted)], false));
    for line in fine(app, w.saturating_sub(8) as usize) {
        lines.push((4, vec![Span::styled(line, inks.dim)], false));
    }
    if app.plan.items.iter().all(|item| item.who.is_none()) {
        lines.push((4, vec![Span::styled(cut("Two items of one person at once: the plan names no one", w.saturating_sub(7) as usize), inks.muted)], false));
    }

    let top = rect.y + 2;
    let bottom = rect.y + rect.height - 1;
    let rows = bottom.saturating_sub(top) as usize;
    let first = lines.iter().position(|line| line.2).unwrap_or(0);
    let last = lines.iter().rposition(|line| line.2).unwrap_or(0);
    let skip = if last >= rows { (last + 1 - rows).min(first) } else { 0 };
    let mut y = top;
    let mut marked = false;
    for (indent, spans, on) in lines.into_iter().skip(skip).take(rows) {
        if on {
            frame.buffer_mut().set_style(Rect { x: x + 1, y, width: w.saturating_sub(2), height: 1 }, Style::new().bg(inks.raised));
            if !marked {
                put(frame, x + 2, y, 1, vec![Span::styled("›", bold(Style::new().fg(inks.accent)))]);
                marked = true;
            }
        }
        put(frame, x + indent, y, w.saturating_sub(indent + 2), spans);
        y += 1;
    }
    y
}

/// The rules that ran and found nothing, in plain words, wrapped to `room` with " ·" at the end of a line.
fn fine(app: &App, room: usize) -> Vec<String> {
    let found = |rule: &str| app.analysis.checks.iter().any(|check| check.rule() == rule);
    let who = app.plan.items.iter().any(|item| item.who.is_some());
    let words: Vec<&str> = [
        ("target-too-late", "every fixed date holds"),
        ("deadline-missed", "deadlines kept"),
        ("fixed-before-predecessor", "fixed dates come after their work"),
        ("no-buffer-before-fixed-date", "a buffer before every fixed date"),
        ("fixed-date-on-day-off", "no fixed date on a day off"),
        ("overdue", "nothing overdue"),
        ("overlap", "nobody on two items at once"),
        ("long-item", "no item longer than six weeks"),
        ("phase-without-milestone", "every phase ends with a milestone"),
    ]
    .into_iter()
    .filter(|&(rule, _)| !found(rule) && (rule != "overlap" || who))
    .map(|(_, words)| words)
    .collect();
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for (index, words) in words.into_iter().enumerate() {
        let words = if index == 0 { words[..1].to_uppercase() + &words[1..] } else { words.to_string() };
        if line.is_empty() {
            line = words;
        } else if line.chars().count() + 3 + words.chars().count() > room {
            lines.push(line + " ·");
            line = words;
        } else {
            line = format!("{line} · {words}");
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn holiday_rows(app: &App, frame: &mut Frame, inks: &Inks, x: u16, mut y: u16, w: u16, bottom: u16) {
    for (index, date, name) in holidays(app) {
        if y >= bottom {
            break;
        }
        let item = &app.plan.items[index];
        let code = app.plan.lane(&item.lane).and_then(|lane| app.plan.lanes[lane].place.clone()).unwrap_or_default();
        let code_w = code.chars().count() as u16;
        put(frame, x + 3, y, 13, vec![Span::styled(cut(&item.name, 12), inks.ink)]);
        put(frame, x + 17, y, 11, vec![Span::styled(day(date), inks.dim)]);
        let room = w.saturating_sub(29 + 4 + code_w);
        put(frame, x + 29, y, room, vec![Span::styled(cut(&name, room as usize), inks.dim)]);
        put(frame, (x + w).saturating_sub(3 + code_w), y, code_w, vec![Span::styled(code, inks.muted)]);
        y += 1;
    }
}

/// The chosen check: the roadmap around it, what it means and what can be done.
fn chosen(app: &App, frame: &mut Frame, inks: &Inks, rect: Rect, placements: &mut Vec<Placement>) {
    let Some(&index) = app.checks().get(app.check) else {
        panel(frame, inks, rect, "", false);
        return;
    };
    let check = &app.analysis.checks[index];
    let Some(cut) = Cut::around(&app.plan, &app.schedule, &check.items()) else { return };
    let span = if cut.start.year() == cut.end.year() {
        format!("{} – {}", cut.start.strftime("%d %b"), cut.end.strftime("%d %b %Y"))
    } else {
        format!("{} – {}", cut.start.strftime("%d %b %Y"), cut.end.strftime("%d %b %Y"))
    };
    panel(frame, inks, rect, &format!("IN THE ROADMAP · {}", span.to_uppercase()), false);

    let (x, w) = (rect.x + 3, rect.width.saturating_sub(6));
    let advice = text::advice(&app.plan, &app.calendars, &app.schedule, app.today, check);
    let mut text: Vec<Vec<Span<'static>>> = vec![vec![Span::styled("WHAT IT MEANS", inks.muted)]];
    for line in &advice.means {
        text.extend(wrap(runs(line, inks, inks.ink), w));
    }
    text.push(Vec::new());
    text.push(vec![Span::styled("WHAT YOU CAN DO", inks.muted)]);
    for line in &advice.can_do {
        text.extend(wrap(runs(line, inks, inks.ink), w));
    }

    let bottom = rect.y + rect.height - 1;
    let picture = Rect { x: rect.x + 1, y: rect.y + 2, width: rect.width.saturating_sub(2), height: (rect.height * 26 / 48).min(bottom.saturating_sub(rect.y + 3 + text.len() as u16)) };
    if picture.height >= 8 && picture.width >= 30 {
        let mut settings = app.settings(picture);
        settings.selected = None;
        settings.level = Level::Full;
        let mut scene = part(&app.plan, &app.schedule, &app.calendars, &settings, &cut);
        scene.background = None;
        placements.push(Placement { rect: picture, svg: scene.svg(), paper: false });
    }
    for (y, line) in (picture.y + picture.height + 1..bottom).zip(text) {
        put(frame, x, y, w, line);
    }
}
