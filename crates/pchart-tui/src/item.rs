//! Adding, renaming and linking items (picture 51): dialogs drawn into the roadmap picture over the
//! dimmed chart, as the date question is, each saying what would follow before Enter changes the plan.

use pchart_draw::bits;
use pchart_draw::scene::{Color, Fill, Scene, Shape, Stroke};
use pchart_draw::Palette;
use pchart_draw::style::ADVANCE;

use crate::app::{App, Form};
use crate::question::follows;
use crate::ui::day;

fn fit(text: &str, size: f32, room: f32) -> String {
    let count = (room / (size * ADVANCE)).floor().max(0.0) as usize;
    if text.chars().count() <= count { text.to_string() } else { text.chars().take(count.saturating_sub(1)).collect::<String>() + "…" }
}

/// The frame, the title and the question of a dialog at `x`, `y`.
#[allow(clippy::too_many_arguments)]
fn head(app: &App, s: &mut Scene, x: f32, y: f32, width: f32, height: f32, u: f32, title: &str, ask: &str) {
    let p = app.palette;
    s.add(Shape::Rect { x, y, w: width, h: height, radius: 0.0, fill: Fill::Color(p.page), stroke: Some(Stroke { color: p.rule, width: u, dash: None }) });
    let corner = 16.0 * u;
    for (cx, cy, dx, dy) in [(x, y, 1.0, 1.0), (x + width, y, -1.0, 1.0), (x, y + height, 1.0, -1.0), (x + width, y + height, -1.0, -1.0)] {
        s.add(Shape::Path { points: vec![(cx + dx * corner, cy), (cx, cy), (cx, cy + dy * corner)], closed: false, fill: Fill::None, stroke: Some(Stroke { color: p.ink, width: 2.0 * u, dash: None }) });
    }
    let room = width - 52.0 * u;
    bits::runs(s, x + 20.0 * u, y + 29.0 * u, 16.6 * u, &[(&fit(&format!(":: {title}"), 16.6 * u, room), p.dim, false)]);
    bits::runs(s, x + 26.0 * u, y + 74.0 * u, (24.0 * u).min(room / (ask.chars().count() as f32 * ADVANCE)), &[(ask, p.bright, true)]);
}

/// The raised band with the pink edge of the line `↑↓` is on.
fn band(s: &mut Scene, x: f32, y: f32, width: f32, height: f32, u: f32, p: &Palette) {
    s.add(Shape::Rect { x: x + 8.0 * u, y, w: width - 16.0 * u, h: height, radius: 0.0, fill: Fill::Color(p.raised), stroke: None });
    s.add(Shape::Rect { x: x + 8.0 * u, y, w: 4.0 * u, h: height, radius: 0.0, fill: Fill::Color(p.accent), stroke: None });
}

/// Text as typed in a box, with the caret when it is the one being typed in.
#[allow(clippy::too_many_arguments)]
fn field(s: &mut Scene, x: f32, y: f32, width: f32, text: &str, caret: bool, u: f32, ink: Color, bright: Color) {
    s.add(Shape::Rect { x, y, w: width, h: 27.0 * u, radius: 3.0 * u, fill: Fill::None, stroke: Some(Stroke { color: ink, width: 1.2 * u, dash: None }) });
    let end = bits::runs(s, x + 10.0 * u, y + 19.0 * u, 16.6 * u, &[(&fit(text, 16.6 * u, width - 24.0 * u), bright, false)]);
    if caret {
        s.add(Shape::Rect { x: end + u, y: y + 5.0 * u, w: 2.0 * u, h: 17.0 * u, radius: 0.0, fill: Fill::Color(bright), stroke: None });
    }
}

/// Draws the open dialog into `s` at `x`, `y`, `width` × `height` pixels.
pub fn dialog(app: &App, s: &mut Scene, x: f32, y: f32, width: f32, height: f32, u: f32) {
    let Some(form) = &app.form else { return };
    let p = app.palette;
    let left = x + 26.0 * u;
    let room = width - 52.0 * u;
    let mut top = y + 100.0 * u;
    match form {
        Form::Add(adding) => {
            let from = &app.plan.items[adding.from].name;
            head(app, s, x, y, width, height, u, &format!("ADD AFTER {}", from.to_uppercase()), "What needs doing?");
            let label_w = 9.0 * 16.6 * u * ADVANCE;
            let step = ((height - 180.0 * u) / 5.0).clamp(30.0 * u, 40.0 * u);
            let lane = &app.plan.lanes[adding.lane];
            let lane_words = [Some(lane.name.as_str()), lane.city.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
            let places = app.places(adding.from);
            for row in 0..5 {
                let chosen = row == adding.at;
                if chosen {
                    band(s, x, top, width, step - 6.0 * u, u, &p);
                }
                let baseline = top + 22.0 * u;
                let label = ["Name", "It is", "Takes", "Lane", "Comes"][row];
                bits::runs(s, left, baseline, 16.6 * u, &[(label, if chosen { p.bright } else { p.dim }, chosen)]);
                let value_x = left + label_w;
                match row {
                    0 => field(s, value_x, top + 3.0 * u, room - label_w, &adding.name, chosen, u, p.ink, p.bright),
                    1 => {
                        let end = bits::chip(s, value_x, baseline, "Work", !adding.milestone, false, u, &p);
                        bits::chip(s, end + 12.0 * u, baseline, "Milestone", adding.milestone, false, u, &p);
                    }
                    2 if adding.milestone => {
                        bits::runs(s, value_x, baseline, 16.6 * u, &[("a point in time, no length", p.muted, false)]);
                    }
                    2 => {
                        let box_w = 110.0 * u;
                        field(s, value_x, top + 3.0 * u, box_w, &adding.length, chosen, u, p.ink, p.bright);
                        let echo = match pchart_core::input::length(&adding.length) {
                            Some(1) => ("→  1 working day".to_string(), p.bright, true),
                            Some(days) => (format!("→  {days} working days"), p.bright, true),
                            None if !app.dated() && adding.length.trim().is_empty() => ("not known yet: it stays empty".to_string(), p.muted, false),
                            None => ("such as 5, 2w or 3 weeks".to_string(), p.muted, false),
                        };
                        bits::runs(s, value_x + box_w + 20.0 * u, baseline, 16.6 * u, &[(&echo.0, echo.1, echo.2)]);
                    }
                    3 => {
                        bits::runs(s, value_x, baseline, 16.6 * u, &[(&fit(&lane_words, 16.6 * u, room - label_w - 40.0 * u), if chosen { p.bright } else { p.ink }, false)]);
                    }
                    _ => {
                        let words = places.get(adding.place).map_or("", |(_, words)| words.as_str());
                        bits::runs(s, value_x, baseline, 16.6 * u, &[(&fit(words, 16.6 * u, room - label_w - 40.0 * u), if chosen { p.bright } else { p.ink }, false)]);
                    }
                }
                if chosen && (row == 1 || row == 3 || row == 4) {
                    bits::runs(s, x + width - 26.0 * u - 2.0 * 16.6 * u * ADVANCE, baseline, 16.6 * u, &[("←→", p.dim, true)]);
                }
                top += step;
            }
        }
        Form::Length { item, typed } => {
            let name = &app.plan.items[*item].name;
            head(app, s, x, y, width, height, u, &format!("LENGTH · {}", name.to_uppercase()), &format!("How long does {name} take?"));
            let box_w = 110.0 * u;
            field(s, left, top, box_w, typed, true, u, p.ink, p.bright);
            let echo = match pchart_core::input::length(typed) {
                Some(1) => ("→  1 working day".to_string(), p.bright, true),
                Some(days) => (format!("→  {days} working days"), p.bright, true),
                None if app.plan.items[*item].duration > 0 => (format!("now {} working days", app.plan.items[*item].duration), p.muted, false),
                None => ("such as 5, 2w or 3 weeks".to_string(), p.muted, false),
            };
            bits::runs(s, left + box_w + 20.0 * u, top + 19.0 * u, 16.6 * u, &[(&echo.0, echo.1, echo.2)]);
            let others = app.plan.without_length().into_iter().filter(|other| other != item).count();
            let words = match others {
                0 => "With it every piece of work has a length, and the dates come.".to_string(),
                1 => "1 other piece of work has no length yet.".to_string(),
                count => format!("{count} other pieces of work have no length yet."),
            };
            bits::runs(s, left, top + 64.0 * u, 15.0 * u, &[(&fit(&words, 15.0 * u, room), p.dim, false)]);
            return;
        }
        Form::Rename { item, typed } => {
            let name = &app.plan.items[*item].name;
            head(app, s, x, y, width, height, u, &format!("RENAME · {}", name.to_uppercase()), &format!("What is {name} called?"));
            field(s, left, top, room, typed, true, u, p.ink, p.bright);
            bits::runs(s, left, top + 64.0 * u, 15.0 * u, &[(&fit("What waits on it, its dates and its notes stay as they are.", 15.0 * u, room), p.dim, false)]);
            return;
        }
        Form::Remove { item } => {
            let name = &app.plan.items[*item].name;
            head(app, s, x, y, width, height, u, &format!("REMOVE · {}", name.to_uppercase()), &format!("Remove {name}?"));
            match app.removal(*item) {
                Ok((_, words)) if words.is_empty() => {
                    bits::runs(s, left, top + 20.0 * u, 16.6 * u, &[("Nothing waits on it.", p.ink, false)]);
                    top += 32.0 * u;
                }
                Ok((_, words)) => {
                    for line in words {
                        bits::runs(s, left, top + 20.0 * u, 16.6 * u, &[(&fit(&line, 16.6 * u, room), p.ink, false)]);
                        top += 28.0 * u;
                    }
                    top += 4.0 * u;
                }
                Err(err) => {
                    for line in crate::note::lines(&err, (room / (16.6 * u * ADVANCE)).floor() as usize) {
                        bits::runs(s, left, top + 20.0 * u, 16.6 * u, &[(&line, p.ink, false)]);
                        top += 28.0 * u;
                    }
                    return;
                }
            }
        }
        Form::Who { item, at, share } => {
            let it = &app.plan.items[*item];
            head(app, s, x, y, width, height, u, &format!("WHO · {}", it.name.to_uppercase()), &format!("Who does {}?", it.name));
            let rows = app.who_rows(*item);
            let span = app.schedule.slots[*item].planned;
            let lane = app.plan.lane(&it.lane).map(|lane| &app.plan.lanes[lane]);
            let team = lane.map_or("", |lane| lane.name.as_str());
            let step = 30.0 * u;
            let label_w = 9.0 * 16.6 * u * ADVANCE;
            bits::runs(s, left, top + 20.0 * u, 16.6 * u, &[("It takes", p.dim, false)]);
            let end = bits::chip(s, left + label_w, top + 20.0 * u, &format!("{share} %"), true, false, u, &p);
            let words = format!("of their time, {} – {}", day(span.start), day(span.finish));
            bits::runs(s, end + 14.0 * u, top + 20.0 * u, 16.6 * u, &[(&fit(&words, 16.6 * u, x + width - 26.0 * u - end - 14.0 * u - 3.0 * 16.6 * u * ADVANCE), p.ink, false)]);
            bits::runs(s, x + width - 26.0 * u - 2.0 * 16.6 * u * ADVANCE, top + 20.0 * u, 16.6 * u, &[("←→", p.dim, true)]);
            top += step + 14.0 * u;
            let name_w = rows.iter().flatten().map(|name| name.chars().count()).max().unwrap_or(6).clamp(6, 22) as f32 * 16.6 * u * ADVANCE;
            let stand = |name: &str| -> (Option<(bits::Icon, Color)>, String) {
                let Some(crew) = &app.crew else { return (None, String::new()) };
                let skip = Some((crew.here, *item));
                match pchart_core::people::stand(&crew.workspace, &crew.projects, name, span.start, span.finish, skip) {
                    pchart_core::people::Stand::Away(away) => (Some((bits::Icon::Warn, p.warn)), format!("away {}", pchart_core::text::span(away.from, away.to))),
                    pchart_core::people::Stand::Taken { most, capacity } if most + share > capacity => (Some((bits::Icon::Warn, p.warn)), format!("would be at {} % of {capacity} %", most + share)),
                    pchart_core::people::Stand::Taken { most, capacity } => (Some((bits::Icon::Check, p.dim)), format!("{} % free then", capacity - most)),
                }
            };
            let in_team = |name: &str| app.crew.as_ref().and_then(|crew| crew.workspace.person(name)).and_then(|person| person.team.as_deref()) == Some(team);
            let mut heading: Option<bool> = None;
            for (row, who) in rows.iter().enumerate() {
                if app.crew.is_some() {
                    let group = who.as_deref().map(in_team);
                    if group.is_some() && group != heading {
                        heading = group;
                        let words = if group == Some(true) { [Some(team), lane.and_then(|lane| lane.city.as_deref())].into_iter().flatten().collect::<Vec<_>>().join(" · ").to_uppercase() } else { "OTHER TEAMS".into() };
                        bits::runs(s, left, top + 20.0 * u, 14.0 * u, &[(&words, p.dim, false)]);
                        top += step;
                    }
                }
                let chosen = row == *at;
                if chosen {
                    band(s, x, top, width, step - 2.0 * u, u, &p);
                }
                let baseline = top + 20.0 * u;
                let mark = it.who == *who;
                if mark {
                    s.add(Shape::Circle { x: left + 7.0 * u, y: baseline - 5.5 * u, r: 5.0 * u, fill: p.ink });
                } else {
                    s.add(Shape::Path { points: (0..=24).map(|k| { let a = k as f32 / 24.0 * std::f32::consts::TAU; (left + 7.0 * u + 6.0 * u * a.cos(), baseline - 5.5 * u + 6.0 * u * a.sin()) }).collect(), closed: true, fill: Fill::None, stroke: Some(Stroke { color: p.muted, width: 1.2 * u, dash: None }) });
                }
                match who {
                    Some(name) => {
                        bits::runs(s, left + 28.0 * u, baseline, 16.6 * u, &[(&fit(name, 16.6 * u, name_w + u), if chosen { p.bright } else { p.ink }, chosen)]);
                        let (icon, words) = stand(name);
                        let at_x = left + 28.0 * u + name_w + 24.0 * u;
                        let mut text_x = at_x;
                        if let Some((icon, tone)) = icon {
                            bits::draw_icon(s, icon, at_x + 8.0 * u, baseline - 5.0 * u, 15.0 * u, tone, if chosen { p.raised } else { p.page });
                            text_x += 24.0 * u;
                        }
                        let other = app.crew.as_ref().and_then(|crew| crew.workspace.person(name)).and_then(|person| person.team.clone()).filter(|other| other != team);
                        let words = match other {
                            Some(other) => format!("{words} · {other}"),
                            None => words,
                        };
                        bits::runs(s, text_x, baseline, 16.6 * u, &[(&fit(&words, 16.6 * u, x + width - 26.0 * u - text_x), p.dim, false)]);
                    }
                    None => {
                        bits::runs(s, left + 28.0 * u, baseline, 16.6 * u, &[("No one named", if chosen { p.bright } else { p.ink }, chosen)]);
                    }
                }
                top += step;
            }
            return;
        }
        Form::WaitsOn { item, on, at } => {
            let it = &app.plan.items[*item];
            head(app, s, x, y, width, height, u, &format!("WAITS ON · {}", it.name.to_uppercase()), &format!("What does {} wait on?", it.name));
            let order = app.waits_order(*item);
            let step = 30.0 * u;
            let rows = ((y + height - top - 110.0 * u) / step).floor().max(1.0) as usize;
            let first = at.saturating_sub(rows.saturating_sub(1));
            let name_w = order.iter().map(|&index| app.plan.items[index].name.chars().count()).max().unwrap_or(0).min(24) as f32 * 16.6 * u * ADVANCE;
            for (row, &index) in order.iter().enumerate().skip(first).take(rows) {
                let other = &app.plan.items[index];
                let chosen = row == *at;
                if chosen {
                    band(s, x, top, width, step - 2.0 * u, u, &p);
                }
                let baseline = top + 20.0 * u;
                s.add(Shape::Rect { x: left, y: baseline - 13.0 * u, w: 15.0 * u, h: 15.0 * u, radius: 3.0 * u, fill: if on[index] { Fill::Color(p.ink) } else { Fill::None }, stroke: (!on[index]).then_some(Stroke { color: p.muted, width: 1.2 * u, dash: None }) });
                bits::runs(s, left + 28.0 * u, baseline, 16.6 * u, &[(&fit(&other.name, 16.6 * u, name_w + u), if chosen { p.bright } else { p.ink }, chosen)]);
                let lane = app.plan.lane(&other.lane).map_or("", |lane| app.plan.lanes[lane].name.as_str());
                let what = format!("{lane} · ends {}", day(app.schedule.slots[index].planned.finish));
                bits::runs(s, left + 28.0 * u + name_w + 24.0 * u, baseline, 16.6 * u, &[(&fit(&what, 16.6 * u, room - name_w - 52.0 * u), p.dim, false)]);
                top += step;
            }
            let waiting: Vec<&str> = app.schedule.order.iter().filter(|&&index| pchart_core::edit::waits_on(&app.plan, index, *item)).map(|&index| app.plan.items[index].name.as_str()).collect();
            if let Some(first) = waiting.first() {
                let words = match waiting.len() {
                    1 => format!("{first} waits on {}, so it is not offered.", it.name),
                    count => format!("{first} and {} more wait on {}, so they are not offered.", count - 1, it.name),
                };
                top += 10.0 * u;
                bits::runs(s, left, top + 20.0 * u, 15.0 * u, &[(&fit(&words, 15.0 * u, room), p.muted, false)]);
                top += step;
            }
        }
    }
    if !app.dated() {
        return;
    }
    top += 18.0 * u;
    s.add(Shape::Line { x1: x + 16.0 * u, y1: top - 10.0 * u, x2: x + width - 16.0 * u, y2: top - 10.0 * u, stroke: Stroke { color: p.rule, width: u, dash: None } });
    if top + 60.0 * u > y + height {
        return;
    }
    match app.form_edits() {
        Ok((edits, _)) if edits.is_empty() => {
            bits::runs(s, left, top + 28.0 * u, 16.6 * u, &[("Nothing changes.", p.muted, false)]);
        }
        Ok((edits, _)) => follows(app, s, &edits, left, top, room, u),
        Err(err) => {
            let words = match err.as_str() {
                "Type a name" => "Type a name to see what follows.".to_string(),
                _ => format!("{err} to see what follows."),
            };
            bits::runs(s, left, top + 28.0 * u, 16.6 * u, &[(&words, p.muted, false)]);
        }
    }
}
