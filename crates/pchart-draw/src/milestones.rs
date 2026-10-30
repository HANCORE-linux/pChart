use jiff::civil::Date;
use pchart_core::{Plan, Schedule};

use crate::page::{Axis, FIXED, GRAY, HEIGHT, INK, LATE, MID, Page, SHADES, X0, X1};
use crate::scene::{Anchor, Color, Fill, Scene, Shape};
use crate::style::sans_width;

/// The milestone chart for export: phases on a time line and every milestone with its date, on a
/// light page for slides and print.
pub fn milestones(plan: &Plan, schedule: &Schedule, today: Date) -> Scene {
    let mut page = Page::new(plan, schedule, today, "Milestone chart", HEIGHT);
    let axis = Axis::new(schedule, today, 13, X0, X1);
    let ty = 190.0;
    page.months(&axis, ty - 4.0, ty - 22.0, HEIGHT - 90.0, 15.0);
    page.line(X0, ty + 6.0, X1, ty + 6.0, MID, 1.5);

    for (index, (name, from, to)) in schedule.phases(plan).into_iter().enumerate() {
        chevron(&mut page, axis.x(from), axis.xe(to), ty + 34.0, 52.0, &name, SHADES[index % SHADES.len()]);
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
        page.diamond(xm, my, r, color, None, Some(index));
        let date = format!("{}{}", slot.planned.finish.strftime("%d %b %Y"), if item.on.is_some() { " · fixed" } else { "" });
        let w = sans_width(&item.name, name_size, true).max(sans_width(&date, 15.0, false));
        let cx = if target { xm + 4.0 } else { xm };
        let below = [cx - w / 2.0, my + 40.0 - name_size, cx + w / 2.0, my + 66.0];
        let above = [cx - w / 2.0, my - 46.0 - name_size, cx + w / 2.0, my - 20.0];
        let fits = |area: &[f32; 4]| area[0] >= 8.0 && area[2] <= page.scene.width - 8.0 && !taken.iter().any(|other| area[0] < other[2] && other[0] < area[2] && area[1] < other[3] && other[1] < area[3]);
        let Some((area, name_y)) = [(below, my + 40.0), (above, my - 46.0)].into_iter().find(|(area, _)| fits(area)) else { continue };
        let name_y = if target { name_y + 6.0 } else { name_y };
        taken.push([area[0] - 6.0, area[1], area[2] + 6.0, area[3]]);
        page.text(cx, name_y, &item.name, name_size, color, true, Anchor::Middle, Some(index));
        page.text(cx, name_y + 22.0, &date, 15.0, if target || late { color } else { GRAY }, false, Anchor::Middle, Some(index));
    }
    page.finish()
}

fn chevron(page: &mut Page, x0: f32, x1: f32, y: f32, h: f32, label: &str, fill: Color) {
    let tip = page.chevron(x0, x1, y, h, fill, 14.0);
    let room = x1 - x0 - tip - 8.0;
    let mut size = 17.0;
    let mut label = label.to_string();
    if sans_width(&label, size, false) > room {
        size = (size * room / sans_width(&label, size, false)).max(10.5);
    }
    while sans_width(&label, size, false) > room && label.chars().count() > 3 {
        let keep = label.chars().count() - 2;
        label = label.chars().take(keep).collect::<String>() + "…";
    }
    page.text((x0 + x1) / 2.0 + tip / 2.0, y + h / 2.0 + size * 0.36, &label, size, INK, false, Anchor::Middle, None);
}
