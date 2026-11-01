use jiff::civil::date;
use pchart_core::{Holidays, Plan, Schedule, analyse, schedule};
use pchart_draw::{Audience, Level, Output, Palette, Settings, gantt, milestones, natural_height, now_next_later, phase_plan, roadmap, swimlanes};

/// The sample, the crowded plan and the templates, each as it is, from a start date and to a target date.
fn plans() -> Vec<(Plan, Vec<pchart_core::Calendar>)> {
    let holidays = Holidays::parse(include_str!("../../../samples/holidays.tsv"));
    [
        include_str!("../../../samples/customer-portal.pchart"),
        include_str!("crowded.pchart"),
        include_str!("../../pchart-store/templates/new-application.pchart"),
        include_str!("../../pchart-store/templates/website-relaunch.pchart"),
        include_str!("../../pchart-store/templates/product-launch.pchart"),
        include_str!("../../pchart-store/templates/empty.pchart"),
    ]
    .into_iter()
    .flat_map(|text| {
        let plan = Plan::parse(text).unwrap();
        let name = plan.name.clone();
        [plan.new_plan(&name, Some(date(2026, 11, 2)), true), plan.new_plan(&name, Some(date(2027, 3, 1)), false), plan]
    })
    .map(|plan| {
        let calendars = plan.lanes.iter().map(|lane| holidays.calendar(lane.place.as_deref())).collect();
        (plan, calendars)
    })
    .collect()
}

/// No text runs over the edge of its shape, and no name is cut down to "…" alone.
#[test]
fn no_text_runs_over_the_edge_of_its_shape() {
    let today = date(2026, 10, 29);
    let output = Output::default();
    let mut found = Vec::new();
    let sizes = [(1.0, 1220.0, None), (1.0, 1220.0, Some(500.0)), (0.8, 1220.0, Some(640.0)), (1.0, 1220.0, Some(380.0)), (0.76, 760.0, Some(330.0)), (0.76, 560.0, Some(260.0))].into_iter().chain((0..13).map(|step| (1.0, 600.0 + 100.0 * step as f32, Some(630.0))));
    let sizes: Vec<(f32, f32, Option<f32>)> = sizes.collect();
    for (plan, calendars) in plans() {
        let schedule: Schedule = schedule(&plan, &calendars, today).unwrap();
        let checks = analyse(&plan, &calendars, &schedule, today).checks;
        for level in [Level::Overview, Level::Standard, Level::Full] {
            for &(unit, width, height) in &sizes {
                let mut settings = Settings { width: width * unit, height: 0.0, unit, today, selected: None, level, palette: Palette::labra(), cursor: None };
                settings.height = height.unwrap_or_else(|| natural_height(&plan, &schedule, &settings));
                let svg = roadmap(&plan, &schedule, &calendars, &settings).svg();
                for crossing in output.crossings(&svg).unwrap().into_iter().chain(svg.contains(">…</text>").then(|| "a lone \"…\"".to_string())) {
                    found.push(format!("{} roadmap {level:?} {width}x{height:?}: {crossing}", plan.name));
                }
            }
        }
        let mut pages = vec![("milestones", milestones(&plan, &schedule, today, Audience::Team)), ("phases", phase_plan(&plan, &schedule, today)), ("now-next-later", now_next_later(&plan, &schedule, today)), ("structure", pchart_draw::structure(&plan, &schedule, today)), ("network", pchart_draw::network(&plan, &schedule, today)), ("matrix", pchart_draw::matrix(&plan, &schedule, today))];
        for audience in [Audience::Team, Audience::Client] {
            pages.push(("gantt", gantt(&plan, &schedule, today, &checks, audience)));
            pages.push(("swimlanes", swimlanes(&plan, &schedule, today, &checks, audience)));
        }
        for (view, scene) in pages {
            let svg = scene.svg();
            for crossing in output.crossings(&svg).unwrap().into_iter().chain(svg.contains(">…</text>").then(|| "a lone \"…\"".to_string())) {
                found.push(format!("{} {view}: {crossing}", plan.name));
            }
        }
    }
    assert!(found.is_empty(), "{} crossings:\n{}", found.len(), found.join("\n"));
}

#[test]
fn a_name_sticking_out_of_its_arrow_is_found() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="40" viewBox="0 0 200 40"><path d="M10,10L80,10L88,19L80,28L10,28L18,19Z" fill="#555555"/><text x="20" y="24" font-size="13.5" fill="#dddddd">Launch plan</text><rect x="100" y="5" width="90" height="30" fill="#555555"/><text x="110" y="24" font-size="13.5" fill="#dddddd">Launch</text></svg>"##;
    let found = Output::default().crossings(svg).unwrap();
    assert!(found.len() == 1 && found[0].starts_with("\"Launch plan\""), "{found:?}");
}
