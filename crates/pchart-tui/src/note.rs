//! The note being written (picture 40), drawn into the roadmap picture over the dimmed chart: what it is
//! about, the text as typed with a caret, and who sees it in exports.

use pchart_draw::bits;
use pchart_draw::scene::{Fill, Scene, Shape, Stroke};
use pchart_draw::style::ADVANCE;

use crate::app::{App, Spot};
use crate::ui::day;

/// `text` broken at spaces into lines of at most `count` characters; a longer word is broken where it must.
pub(crate) fn lines(text: &str, count: usize) -> Vec<String> {
    let mut out = vec![String::new()];
    for word in text.split(' ') {
        let line = out.last_mut().expect("there is always a line");
        if line.is_empty() || line.chars().count() + 1 + word.chars().count() <= count {
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        } else {
            out.push(word.to_string());
        }
        while out.last().is_some_and(|line| line.chars().count() > count) {
            let line = out.pop().expect("checked above");
            let (head, tail): (String, String) = (line.chars().take(count).collect(), line.chars().skip(count).collect());
            out.extend([head, tail]);
        }
    }
    out
}

/// Draws the note being written into `s` at `x`, `y`, `width` × `height` pixels.
pub fn dialog(app: &App, s: &mut Scene, x: f32, y: f32, width: f32, height: f32, unit: f32) {
    let (Some(typed), Some(spot)) = (app.notes.as_ref().and_then(|notes| notes.typed.as_ref()), app.spot()) else { return };
    let (p, u) = (app.palette, unit);
    s.add(Shape::Rect { x, y, w: width, h: height, radius: 0.0, fill: Fill::Color(p.page), stroke: Some(Stroke { color: p.rule, width: u, dash: None }) });
    let corner = 16.0 * u;
    for (cx, cy, dx, dy) in [(x, y, 1.0, 1.0), (x + width, y, -1.0, 1.0), (x, y + height, 1.0, -1.0), (x + width, y + height, -1.0, -1.0)] {
        s.add(Shape::Path { points: vec![(cx + dx * corner, cy), (cx, cy), (cx, cy + dy * corner)], closed: false, fill: Fill::None, stroke: Some(Stroke { color: p.ink, width: 2.0 * u, dash: None }) });
    }
    let (title, about, milestone) = match spot {
        Spot::Item(index) => (app.plan.items[index].name.to_uppercase(), app.plan.items[index].name.clone(), app.plan.items[index].milestone),
        Spot::Place(lane, date) => (format!("{} · {}", app.plan.lanes[lane].name, day(date)).to_uppercase(), day(date), false),
    };
    let left = x + 26.0 * u;
    let room = width - 52.0 * u;
    let fit = |text: &str, size: f32| text.chars().take((room / (size * ADVANCE)).floor().max(1.0) as usize).collect::<String>();
    bits::runs(s, x + 20.0 * u, y + 29.0 * u, 16.6 * u, &[(&fit(&format!(":: NOTE · {title}"), 16.6 * u), p.dim, false)]);
    let ask = format!("What should people know about {about}?");
    bits::runs(s, left, y + 74.0 * u, (24.0 * u).min(room / (ask.chars().count() as f32 * ADVANCE)), &[(&ask, p.bright, true)]);

    let (size, top, rows) = (16.6 * u, y + 100.0 * u, 3);
    s.add(Shape::Rect { x: left, y: top, w: room, h: rows as f32 * 21.0 * u + 10.0 * u, radius: 3.0 * u, fill: Fill::None, stroke: Some(Stroke { color: p.ink, width: 1.2 * u, dash: None }) });
    let count = ((room - 24.0 * u) / (size * ADVANCE)).floor().max(1.0) as usize;
    let all = lines(typed, count);
    let shown = &all[all.len().saturating_sub(rows)..];
    let mut end = (left + 12.0 * u, top + 21.0 * u);
    for (row, line) in shown.iter().enumerate() {
        let baseline = top + 21.0 * u + row as f32 * 21.0 * u;
        end = (bits::runs(s, left + 12.0 * u, baseline, size, &[(line, p.bright, false)]), baseline);
    }
    s.add(Shape::Rect { x: end.0 + u, y: end.1 - 14.0 * u, w: 2.0 * u, h: 17.0 * u, radius: 0.0, fill: Fill::Color(p.bright), stroke: None });
    if milestone && top + (rows as f32 + 2.5) * 21.0 * u < y + height {
        let hint = "Exports for Team and Management show it under the milestone; the client's do not.";
        bits::runs(s, left, top + rows as f32 * 21.0 * u + 42.0 * u, 15.0 * u, &[(&fit(hint, 15.0 * u), p.dim, false)]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_note_breaks_at_spaces_and_a_long_word_where_it_must() {
        assert_eq!(lines("Client signs the design in a workshop", 16), ["Client signs the", "design in a", "workshop"]);
        assert_eq!(lines("abcdefghij", 4), ["abcd", "efgh", "ij"]);
        assert_eq!(lines("", 10), [""]);
    }
}
