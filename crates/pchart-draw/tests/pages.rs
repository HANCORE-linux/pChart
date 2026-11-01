use jiff::civil::date;
use pchart_core::{Holidays, Plan, Schedule, analyse, schedule};
use pchart_draw::page::{INK, MID};
use pchart_draw::scene::{Anchor, Fill, Scene, Shape};
use pchart_draw::style::sans_width;
use pchart_draw::{Audience, dot, gantt, matrix, milestones, network, now_next_later, phase_plan, structure, swimlanes};

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
        out.push((format!("{} structure", plan.name), structure(&plan, &schedule, today)));
        out.push((format!("{} network", plan.name), network(&plan, &schedule, today)));
        out.push((format!("{} matrix", plan.name), matrix(&plan, &schedule, today)));
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


#[test]
fn the_structure_and_the_network_show_how_the_plan_is_built_without_dates() {
    let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
    let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
    let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
    let today = jiff::civil::date(2026, 10, 29);
    let schedule: Schedule = schedule(&plan, &calendars, today).unwrap();
    let texts = |scene: &Scene| -> Vec<String> { scene.marks.iter().filter_map(|mark| if let Shape::Text { text, .. } = &mark.shape { Some(text.clone()) } else { None }).collect() };
    let built = texts(&structure(&plan, &schedule, today));
    for text in ["Project structure · as of 29 Oct 2026", "1", "Requirements", "1.1 · Product · 10 wd", "4", "Beta & acceptance", "4.4 · Client"] {
        assert!(built.contains(&text.to_string()), "{text}");
    }
    let network = network(&plan, &schedule, today);
    let words = texts(&network);
    for text in ["Network diagram · as of 29 Oct 2026", "Product", "Berlin", "Requirements", "10 wd", "Launch", "decides the date", "waits on"] {
        assert!(words.contains(&text.to_string()), "{text}");
    }
    assert!(built.iter().chain(&words).all(|text| !text.contains("Dates scheduled") && !text.contains("Mar 2027")));
    let top = |name: &str| network.marks.iter().find_map(|mark| match &mark.shape { Shape::Text { text, y, .. } if text == name => Some(*y), _ => None }).unwrap();
    assert!(top("Requirements") < top("UX concept") && top("UX concept") < top("UI design") && top("Feature freeze") < top("Beta test") && top("Beta test") < top("Launch"));
}

#[test]
fn the_matrix_puts_a_dot_below_the_diagonal_where_a_row_waits_on_a_column() {
    let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
    let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
    let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
    let today = date(2026, 10, 29);
    let schedule: Schedule = schedule(&plan, &calendars, today).unwrap();
    let scene = matrix(&plan, &schedule, today);
    let texts: Vec<&str> = scene.marks.iter().filter_map(|mark| if let Shape::Text { text, .. } = &mark.shape { Some(text.as_str()) } else { None }).collect();
    for text in ["Dependency matrix · as of 29 Oct 2026", "1 Requirements", "Requirements", "Launch", "waits on, with time to spare"] {
        assert!(texts.contains(&text), "{text}");
    }
    assert!(texts.iter().all(|text| !text.contains("Dates scheduled") && !text.contains("Mar 2027")));
    let diagonal: Vec<(f32, f32, f32)> = scene.marks.iter().filter_map(|mark| match mark.shape {
        Shape::Rect { x, y, w, fill: Fill::Color(MID), .. } => Some((x, y, w)),
        _ => None,
    }).collect();
    assert_eq!(diagonal.len(), plan.items.len());
    let dots: Vec<(f32, f32, bool)> = scene.marks.iter().filter_map(|mark| match mark.shape {
        Shape::Circle { x, y, fill, .. } if y < scene.height - 120.0 => Some((x, y, fill == INK)),
        _ => None,
    }).collect();
    let links: Vec<(usize, usize)> = plan.items.iter().enumerate().flat_map(|(index, item)| item.after.iter().filter_map(|id| plan.item(id)).map(move |pred| (pred, index))).collect();
    let chain = links.iter().filter(|(pred, index)| schedule.chain.contains(pred) && schedule.chain.contains(index)).count();
    assert_eq!(dots.len(), links.len());
    assert_eq!(dots.iter().filter(|(_, _, ink)| *ink).count(), chain);
    assert!(chain > 0 && dots.iter().any(|(_, _, ink)| !ink));
    for (x, y, _) in dots {
        let &(dx, _, _) = diagonal.iter().find(|(_, dy, w)| y > *dy && y < dy + w).unwrap();
        assert!(x < dx, "a dot on or above the diagonal at {x} {y}");
    }
}

#[test]
fn the_dot_file_has_a_cluster_per_lane_and_an_arrow_per_wait() {
    let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
    let mut plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
    plan.items[0].name = "Say \"hi\" \\ go".into();
    let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
    let schedule: Schedule = schedule(&plan, &calendars, date(2026, 10, 29)).unwrap();
    let text = dot(&plan, &schedule);
    assert!(text.starts_with("digraph \"Customer portal v1\" {\n") && text.ends_with("}\n"), "{text}");
    assert_eq!(text.matches("subgraph \"cluster_").count(), plan.lanes.len());
    let links: usize = plan.items.iter().map(|item| item.after.iter().filter(|id| plan.item(id).is_some()).count()).sum();
    assert_eq!(text.matches(" -> ").count(), links);
    assert!(text.contains(r#"label="Say \"hi\" \\ go\n"#), "{text}");
    assert!(text.contains(r#"label="Product · Berlin""#));
    assert!(text.contains(r#""launch" [shape=diamond"#) && text.contains(r#"xlabel="Launch""#));
    assert!(text.contains(r##"-> "ux-concept" [color="#1f2421", penwidth=2];"##));
}
