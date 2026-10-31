use jiff::civil::date;
use pchart_core::{Holidays, Plan, Schedule, analyse, schedule};
use pchart_draw::scene::{Anchor, Scene, Shape};
use pchart_draw::style::sans_width;
use pchart_draw::{Audience, gantt, milestones, now_next_later, phase_plan, swimlanes};

fn pages() -> Vec<(String, Scene)> {
    let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
    let today = date(2026, 10, 29);
    let mut out = Vec::new();
    for text in [include_str!("../../../samples/customer-portal.pchart"), include_str!("crowded.pchart")] {
        let plan = Plan::parse(text).unwrap();
        let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        let schedule: Schedule = schedule(&plan, &calendars, today).unwrap();
        let checks = analyse(&plan, &calendars, &schedule, today).checks;
        out.push((format!("{} milestones", plan.name), milestones(&plan, &schedule, today, Audience::Team)));
        for audience in [Audience::Team, Audience::Client] {
            out.push((format!("{} gantt for {audience:?}", plan.name), gantt(&plan, &schedule, today, &checks, audience)));
            out.push((format!("{} swimlanes for {audience:?}", plan.name), swimlanes(&plan, &schedule, today, &checks, audience)));
        }
        out.push((format!("{} phases", plan.name), phase_plan(&plan, &schedule, today)));
        out.push((format!("{} now-next-later", plan.name), now_next_later(&plan, &schedule, today)));
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

#[test]
fn names_on_pages_are_whole() {
    for (name, scene) in pages() {
        for mark in &scene.marks {
            if let Shape::Text { text, .. } = &mark.shape {
                assert!(!text.ends_with('…'), "{name}: \"{text}\" is cut");
            }
        }
    }
}

#[test]
fn notes_show_for_the_team_and_not_for_the_client() {
    let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
    let mut plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
    let sign_off = plan.item("design-sign-off").unwrap();
    plan.items[sign_off].important = true;
    plan.items[sign_off].note = Some("Client signs the design in a workshop in New York.".into());
    let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
    let today = jiff::civil::date(2026, 10, 29);
    let schedule: Schedule = schedule(&plan, &calendars, today).unwrap();
    let team = milestones(&plan, &schedule, today, Audience::Team).svg();
    let client = milestones(&plan, &schedule, today, Audience::Client).svg();
    assert!(team.contains("workshop in New York.") && !client.contains("workshop") && team.contains(">important<") && client.contains(">important<"));
}

