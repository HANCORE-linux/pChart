use jiff::civil::date;
use pchart_core::{Calendar, Holidays, Plan, schedule};
use pchart_draw::scene::{Anchor, Shape};
use pchart_draw::roadmap::shown;
use pchart_draw::{Cut, Level, Palette, Settings, natural_height, part, roadmap};

fn plans() -> Vec<(Plan, Vec<Calendar>)> {
    let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
    [include_str!("../../../samples/customer-portal.pchart"), include_str!("crowded.pchart")]
        .into_iter()
        .map(|text| {
            let plan = Plan::parse(text).unwrap();
            let calendars = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
            (plan, calendars)
        })
        .collect()
}

/// Where the names of items are drawn, as left, top, right and bottom.
fn names(plan: &Plan, calendars: &[Calendar], level: Level, unit: f32, size: (f32, Option<f32>)) -> Vec<(String, [f32; 4])> {
    let today = date(2026, 10, 29);
    let schedule = schedule(plan, calendars, today).unwrap();
    let (width, height) = size;
    let mut settings = Settings { width: width * unit, height: 0.0, unit, today, selected: Some(0), level, palette: Palette::labra(), cursor: None };
    settings.height = height.unwrap_or_else(|| natural_height(plan, &schedule, &settings));
    roadmap(plan, &schedule, calendars, &settings)
        .marks
        .into_iter()
        .filter(|mark| mark.item.is_some())
        .filter_map(|mark| match mark.shape {
            Shape::Text { x, y, text, size, anchor, .. } => {
                let width = text.chars().count() as f32 * size * 0.6;
                let left = match anchor {
                    Anchor::Start => x,
                    Anchor::Middle => x - width / 2.0,
                    Anchor::End => x - width,
                };
                Some((text, [left, y - 0.78 * size, left + width, y + 0.24 * size]))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn names_never_cover_each_other() {
    for (plan, calendars) in plans() {
        for level in [Level::Overview, Level::Standard, Level::Full] {
            for (unit, size) in [(1.0, (1220.0, None)), (1.0, (1220.0, Some(500.0))), (0.8, (1220.0, Some(640.0))), (1.0, (1220.0, Some(380.0))), (0.76, (760.0, Some(330.0))), (0.76, (560.0, Some(260.0)))] {
                let names = names(&plan, &calendars, level, unit, size);
                for (index, (a, x)) in names.iter().enumerate() {
                    for (b, y) in &names[index + 1..] {
                        assert!(x[2] <= y[0] || y[2] <= x[0] || x[3] <= y[1] || y[3] <= x[1], "{} {level:?} {unit} {size:?}: \"{a}\" covers \"{b}\"", plan.name);
                    }
                }
            }
        }
    }
}

#[test]
fn the_sample_names_every_item_at_its_natural_size() {
    let (plan, calendars) = plans().remove(0);
    for level in [Level::Standard, Level::Full] {
        assert_eq!(names(&plan, &calendars, level, 1.0, (1220.0, None)).len(), plan.items.len(), "{level:?}");
    }
}

#[test]
fn a_small_picture_shows_less_detail_and_keeps_the_fixed_target_in_view() {
    let (plan, calendars) = plans().remove(0);
    let today = date(2026, 10, 29);
    let schedule = schedule(&plan, &calendars, today).unwrap();
    let frontend = plan.item("frontend").unwrap();
    let client = plan.lane("client").unwrap();
    let at = |height: f32| shown(&plan, &schedule, &Settings { width: 1220.0, height, unit: 1.0, today, selected: Some(frontend), level: Level::Full, palette: Palette::labra(), cursor: None });
    assert_eq!(at(756.0), (Level::Full, (0..plan.lanes.len()).collect()));
    assert_eq!(at(560.0), (Level::Standard, (0..plan.lanes.len()).collect()));
    let (level, lanes) = at(330.0);
    assert_eq!(level, Level::Standard);
    assert!(lanes.len() < plan.lanes.len() && lanes.contains(&plan.lane("frontend").unwrap()) && lanes.contains(&client), "{lanes:?}");
}

#[test]
fn names_in_the_roadmap_around_a_check_stay_apart_and_inside() {
    let today = date(2026, 10, 29);
    for (plan, calendars) in plans() {
        let schedule = schedule(&plan, &calendars, today).unwrap();
        let checks = pchart_core::analyse(&plan, &calendars, &schedule, today).checks;
        let around: Vec<(String, Vec<usize>)> = checks.iter().map(|check| (check.rule().to_string(), check.items())).chain((0..plan.items.len()).map(|index| (plan.items[index].id.clone(), vec![index]))).collect();
        for (what, items) in around {
            let cut = Cut::around(&plan, &schedule, &items).unwrap();
            for (width, height) in [(658.0, 416.0), (900.0, 300.0)] {
                let settings = Settings { width, height, unit: 0.76, today, selected: None, level: Level::Full, palette: Palette::labra(), cursor: None };
                let names: Vec<(String, [f32; 4])> = part(&plan, &schedule, &calendars, &settings, &cut)
                    .marks
                    .into_iter()
                    .filter(|mark| mark.item.is_some())
                    .filter_map(|mark| match mark.shape {
                        Shape::Text { x, y, text, size, anchor, .. } => {
                            let w = text.chars().count() as f32 * size * 0.6;
                            let left = match anchor {
                                Anchor::Start => x,
                                Anchor::Middle => x - w / 2.0,
                                Anchor::End => x - w,
                            };
                            Some((text, [left, y - 0.78 * size, left + w, y + 0.24 * size]))
                        }
                        Shape::Rect { x, w, .. } => {
                            assert!(x >= 0.0 && x + w <= width + 0.5, "{}: a bar leaves the picture", plan.name);
                            None
                        }
                        _ => None,
                    })
                    .collect();
                for (index, (a, x)) in names.iter().enumerate() {
                    assert!(x[0] >= 0.0 && x[2] <= width, "{} {what}: \"{a}\" leaves the picture", plan.name);
                    for (b, y) in &names[index + 1..] {
                        assert!(x[2] <= y[0] || y[2] <= x[0] || x[3] <= y[1] || y[3] <= x[1], "{} {what}: \"{a}\" covers \"{b}\"", plan.name);
                    }
                }
            }
        }
    }
}
