use jiff::civil::{Date, date};
use pchart_core::{Holidays, Plan, schedule};
use pchart_draw::scene::{Anchor, Shape};
use pchart_draw::style::ADVANCE;
use pchart_draw::{Cut, Level, Palette, Settings, natural_height, part, roadmap};

/// The left and right of every text on the line of the months, and whether it is "today".
fn top_line(scene: &pchart_draw::Scene) -> Vec<(f32, f32, String)> {
    let texts: Vec<(f32, f32, f32, Anchor, String)> = scene.marks.iter().filter_map(|mark| match &mark.shape {
        Shape::Text { x, y, text, size, anchor, .. } => Some((*x, *y, *size, *anchor, text.clone())),
        _ => None,
    }).collect();
    let Some(&(_, line, ..)) = texts.iter().find(|text| text.4 == "today") else { return Vec::new() };
    texts
        .into_iter()
        .filter(|text| (text.1 - line).abs() < 0.5)
        .map(|(x, _, size, anchor, text)| {
            let w = text.chars().count() as f32 * size * ADVANCE;
            let left = match anchor {
                Anchor::Start => x,
                Anchor::Middle => x - w / 2.0,
                Anchor::End => x - w,
            };
            (left, left + w, text)
        })
        .collect()
}

fn overlaps(scene: &pchart_draw::Scene) -> Vec<String> {
    let line = top_line(scene);
    let Some(today) = line.iter().find(|text| text.2 == "today") else { return Vec::new() };
    line.iter().filter(|text| text.2 != "today" && text.0 < today.1 + 2.0 && today.0 < text.1 + 2.0).map(|text| text.2.clone()).collect()
}

/// "today" never covers the name of a month, on the roadmap or in a part of it, whatever day it is.
#[test]
fn today_never_covers_a_month() {
    let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
    let plan = Plan::parse(include_str!("../../../samples/customer-portal.pchart")).unwrap();
    let calendars: Vec<_> = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
    let mut found = Vec::new();
    let mut day: Date = date(2026, 10, 1);
    while day <= date(2027, 3, 31) {
        let schedule = schedule(&plan, &calendars, day).unwrap();
        for (unit, width) in [(1.0, 1220.0), (0.76, 760.0)] {
            let mut settings = Settings { width, height: 0.0, unit, today: day, selected: None, level: Level::Full, palette: Palette::labra(), cursor: None };
            settings.height = natural_height(&plan, &schedule, &settings);
            for month in overlaps(&roadmap(&plan, &schedule, &calendars, &settings)) {
                found.push(format!("{day} roadmap {width}: {month}"));
            }
            for index in 0..plan.items.len() {
                let Some(cut) = Cut::around(&plan, &schedule, &[index]) else { continue };
                for month in overlaps(&part(&plan, &schedule, &calendars, &Settings { height: 300.0, ..settings }, &cut)) {
                    found.push(format!("{day} part around {} {width}: {month}", plan.items[index].id));
                }
            }
        }
        day = day.tomorrow().unwrap();
    }
    found.dedup();
    assert!(found.is_empty(), "{} overlaps, first: {:#?}", found.len(), &found[..found.len().min(12)]);
}
