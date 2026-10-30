use jiff::civil::date;
use pchart_core::{Calendar, Holidays, Plan, schedule};
use pchart_draw::scene::{Anchor, Shape};
use pchart_draw::{Level, Palette, Settings, natural_height, roadmap};

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
fn names(plan: &Plan, calendars: &[Calendar], level: Level, unit: f32, height: Option<f32>) -> Vec<(String, [f32; 4])> {
    let today = date(2026, 10, 29);
    let schedule = schedule(plan, calendars, today).unwrap();
    let mut settings = Settings { width: 1220.0 * unit, height: 0.0, unit, today, selected: Some(0), level, palette: Palette::labra() };
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
            for (unit, height) in [(1.0, None), (1.0, Some(500.0)), (0.8, Some(640.0))] {
                let names = names(&plan, &calendars, level, unit, height);
                for (index, (a, x)) in names.iter().enumerate() {
                    for (b, y) in &names[index + 1..] {
                        assert!(x[2] <= y[0] || y[2] <= x[0] || x[3] <= y[1] || y[3] <= x[1], "{} {level:?} {unit} {height:?}: \"{a}\" covers \"{b}\"", plan.name);
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
        assert_eq!(names(&plan, &calendars, level, 1.0, None).len(), plan.items.len(), "{level:?}");
    }
}
