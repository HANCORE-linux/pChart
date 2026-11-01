use jiff::civil::Date;
use pchart_core::{Plan, Schedule};

use crate::page::{Axis, FIXED, GRAY, HEIGHT, INK, LATE, MID, Page, SHADES, X0, X1, wrap};
use crate::scene::{Anchor, Color, Fill, Scene, Shape};
use crate::style::{arrow_label, sans_width};
use crate::views::Audience;

/// The milestone chart for export: phases on a time line and every milestone with its date, on a
/// light page for slides and print.
pub fn milestones(plan: &Plan, schedule: &Schedule, today: Date, audience: Audience) -> Scene {
    let mut page = Page::new(plan, schedule, today, "Milestone chart", HEIGHT);
    let axis = Axis::new(schedule, today, 13, X0, X1);
    let ty = 190.0;
    page.months(&axis, ty - 4.0, ty - 22.0, HEIGHT - 90.0, 15.0);
    page.line(X0, ty + 6.0, X1, ty + 6.0, MID, 1.5);

    for (index, ((name, from, to), phase)) in schedule.phases(plan).into_iter().zip(&plan.phases).enumerate() {
        chevron(&mut page, axis.x(from), axis.xe(to), ty + 34.0, 52.0, &name, phase.short.as_deref(), SHADES[index % SHADES.len()]);
    }

    let my = ty + 214.0;
    page.line(X0, my, X1, my, MID, 2.0);
    let xt = axis.x(today) + axis.px / 2.0;
    page.scene.add(Shape::Path { points: vec![(xt, ty + 14.0), (xt - 6.0, ty + 5.0), (xt + 6.0, ty + 5.0)], closed: true, fill: Fill::Color(GRAY), stroke: None });
    page.text(xt, ty + 30.0, "today", 12.5, GRAY, false, Anchor::Middle, None);

    let mut order: Vec<usize> = (0..plan.items.len()).filter(|&index| plan.items[index].milestone).collect();
    order.sort_by_key(|&index| (Some(index) != schedule.target, schedule.slots[index].planned.finish));
    let mut taken: Vec<[f32; 4]> = Vec::new();
    for index in order {
        let item = &plan.items[index];
        let slot = &schedule.slots[index];
        let xm = axis.milestone(plan, schedule, index);
        let late = slot.spare < 0;
        let target = Some(index) == schedule.target;
        let (r, name_size, color) = if target { (13.0, 19.0, if late { LATE } else { FIXED }) } else { (10.0, 17.0, if late { LATE } else { INK }) };
        page.milestone(xm, my, r, late, false, false, item.important, Some(index));
        let date = format!("{}{}", slot.planned.finish.strftime("%d %b %Y"), if item.on.is_some() { " · fixed" } else { "" });
        let w = sans_width(&item.name, name_size, true).max(sans_width(&date, 15.0, false));
        let cx = if target { xm + 4.0 } else { xm };
        let note = item.note.as_deref().filter(|_| audience.notes()).map_or_else(Vec::new, |note| wrap(note, 13.5, false, w.max(200.0)));
        let w = note.iter().map(|line| sans_width(line, 13.5, false)).fold(w, f32::max);
        let extra = 18.0 * note.len() as f32 + if note.is_empty() { 0.0 } else { 4.0 };
        let below = [cx - w / 2.0, my + 40.0 - name_size, cx + w / 2.0, my + 66.0 + extra];
        let above = [cx - w / 2.0, my - 46.0 - name_size - extra, cx + w / 2.0, my - 20.0];
        let fits = |area: &[f32; 4]| area[0] >= 8.0 && area[2] <= page.scene.width - 8.0 && !taken.iter().any(|other| area[0] < other[2] && other[0] < area[2] && area[1] < other[3] && other[1] < area[3]);
        let Some((area, name_y)) = [(below, my + 40.0), (above, my - 46.0)].into_iter().find(|(area, _)| fits(area)) else { continue };
        let name_y = if target { name_y + 6.0 } else { name_y };
        taken.push([area[0] - 6.0, area[1], area[2] + 6.0, area[3]]);
        page.text(cx, name_y, &item.name, name_size, color, true, Anchor::Middle, Some(index));
        page.text(cx, name_y + 22.0, &date, 15.0, if target || late { color } else { GRAY }, false, Anchor::Middle, Some(index));
        let first = if area[1] < my { name_y - name_size - 4.0 - 18.0 * (note.len() as f32 - 1.0) } else { name_y + 46.0 };
        for (line, text) in note.iter().enumerate() {
            page.text(cx, first + 18.0 * line as f32, text, 13.5, GRAY, false, Anchor::Middle, Some(index));
        }
    }
    page.finish()
}

#[allow(clippy::too_many_arguments)]
fn chevron(page: &mut Page, x0: f32, x1: f32, y: f32, h: f32, name: &str, short: Option<&str>, fill: Color) {
    let tip = page.chevron(x0, x1, y, h, fill, 14.0);
    let Some((lines, size, middle)) = arrow_label(name, short, x0, x1, h, tip, 17.0, 9.0) else { return };
    let pitch = size * 1.2;
    let first = y + h / 2.0 + size * 0.36 - pitch * (lines.len() - 1) as f32 / 2.0;
    for (index, line) in lines.iter().enumerate() {
        page.text(middle, first + pitch * index as f32, line, size, INK, false, Anchor::Middle, None);
    }
}
