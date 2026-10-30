use jiff::civil::date;
use pchart_core::{Holidays, Plan, Schedule, analyse, schedule};
use pchart_draw::scene::{Anchor, Scene, Shape};
use pchart_draw::style::sans_width;
use pchart_draw::{gantt, milestones, now_next_later, phase_plan, swimlanes};

fn pages() -> Vec<(String, Scene)> {
    let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
    let today = date(2026, 10, 29);
    let mut out = Vec::new();
    for text in [include_str!("../../../samples/customer-portal.pchart"), include_str!("crowded.pchart")] {
        let plan = Plan::parse(text).unwrap();
        let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        let schedule: Schedule = schedule(&plan, &calendars, today).unwrap();
        let checks = analyse(&plan, &calendars, &schedule, today).checks;
        out.push((format!("{} milestones", plan.name), milestones(&plan, &schedule, today)));
        out.push((format!("{} gantt", plan.name), gantt(&plan, &schedule, today, &checks)));
        out.push((format!("{} phases", plan.name), phase_plan(&plan, &schedule, today)));
        out.push((format!("{} now-next-later", plan.name), now_next_later(&plan, &schedule, today)));
        out.push((format!("{} swimlanes", plan.name), swimlanes(&plan, &schedule, today, &checks)));
    }
    out
}

#[test]
fn texts_on_pages_neither_cover_each_other_nor_leave_the_page() {
    for (name, scene) in pages() {
        let texts: Vec<(String, [f32; 4])> = scene
            .marks
            .iter()
            .filter_map(|mark| match &mark.shape {
                Shape::Text { x, y, text, size, bold, anchor, .. } => {
                    let width = sans_width(text, *size, *bold);
                    let left = match anchor {
                        Anchor::Start => *x,
                        Anchor::Middle => x - width / 2.0,
                        Anchor::End => x - width,
                    };
                    Some((text.clone(), [left, y - 0.74 * size, left + width, y + 0.24 * size]))
                }
                _ => None,
            })
            .collect();
        for (index, (a, x)) in texts.iter().enumerate() {
            assert!(x[0] >= 0.0 && x[2] <= scene.width && x[1] >= 0.0 && x[3] <= scene.height, "{name}: \"{a}\" leaves the page");
            for (b, y) in &texts[index + 1..] {
                assert!(x[2] <= y[0] || y[2] <= x[0] || x[3] <= y[1] || y[3] <= x[1], "{name}: \"{a}\" covers \"{b}\"");
            }
        }
    }
}
