use std::time::Duration;

use jiff::civil::Date;
use pchart_core::{Calendar, Plan, Schedule};
use pchart_draw::scene::Color as Ink;
use pchart_draw::{Palette, Settings};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui_image::Image;
use ratatui_image::protocol::Protocol;

use crate::pictures::Cost;

pub const MIN_SIZE: (u16, u16) = (100, 30);
const SIDE: u16 = 46;

pub struct App {
    pub plan: Plan,
    pub calendars: Vec<Calendar>,
    pub today: Date,
    pub path: String,
    pub schedule: Schedule,
    pub selected: usize,
    pub generation: u64,
    pub picture: Option<(u64, Rect, Protocol)>,
    pub cost: Option<(Cost, Duration)>,
    pub protocol: &'static str,
    pub cell: (u16, u16),
    pub message: Option<String>,
    pub palette: Palette,
}

fn color(ink: Ink) -> Color {
    Color::Rgb(ink.0, ink.1, ink.2)
}

impl App {
    pub fn new(plan: Plan, calendars: Vec<Calendar>, today: Date, path: String, schedule: Schedule, protocol: &'static str, cell: (u16, u16)) -> App {
        let selected = plan.item("frontend").or_else(|| schedule.chain.iter().copied().find(|&index| !plan.items[index].milestone)).unwrap_or(0);
        App { plan, calendars, today, path, schedule, selected, generation: 0, picture: None, cost: None, protocol, cell, message: None, palette: Palette::labra() }
    }

    /// Where the chart picture goes on a screen of this size.
    pub fn chart(area: Rect) -> Rect {
        let width = area.width.saturating_sub(SIDE);
        Rect { x: area.x + 1, y: area.y + 8, width: width.saturating_sub(2), height: area.height.saturating_sub(8 + 6) }
    }

    pub fn svg(&self, rect: Rect) -> String {
        let (cw, ch) = self.cell;
        let settings = Settings {
            width: f32::from(rect.width) * f32::from(cw),
            height: f32::from(rect.height) * f32::from(ch),
            unit: f32::from(ch) / 21.0,
            today: self.today,
            selected: Some(self.selected),
            palette: self.palette,
        };
        pchart_draw::roadmap(&self.plan, &self.schedule, &self.calendars, &settings).svg()
    }

    pub fn select(&mut self, step: isize) {
        let count = self.plan.items.len() as isize;
        let position = self.schedule.order.iter().position(|&index| index == self.selected).unwrap_or(0) as isize;
        self.selected = self.schedule.order[(position + step).rem_euclid(count) as usize];
        self.generation += 1;
    }

    /// Lengthens or shortens the selected item by working days.
    pub fn stretch(&mut self, days: i32) {
        let item = &self.plan.items[self.selected];
        if item.milestone {
            self.message = Some(format!("{} is a milestone; it has no duration", item.name));
            return;
        }
        let mut plan = self.plan.clone();
        plan.items[self.selected].duration = (item.duration as i32 + days).max(1) as u32;
        match pchart_core::schedule(&plan, &self.calendars, self.today) {
            Ok(schedule) => {
                self.plan = plan;
                self.schedule = schedule;
                self.message = None;
                self.generation += 1;
            }
            Err(err) => self.message = Some(err.to_string()),
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let p = self.palette;
        let (ink, bright, dim, muted, rule) = (color(p.ink), color(p.bright), color(p.dim), color(p.muted), color(p.rule));
        let bold = Modifier::BOLD;
        if area.width < MIN_SIZE.0 || area.height < MIN_SIZE.1 {
            let text = format!("pChart needs at least {}×{} cells; this terminal has {}×{}.", MIN_SIZE.0, MIN_SIZE.1, area.width, area.height);
            frame.render_widget(Paragraph::new(text).style(Style::new().fg(ink)), area);
            return;
        }
        let panel = |frame: &mut Frame, rect: Rect, title: &str| {
            frame.render_widget(Block::bordered().border_style(Style::new().fg(rule)), rect);
            frame.render_widget(Paragraph::new(Span::styled(format!(":: {title}"), Style::new().fg(dim))), Rect { x: rect.x + 2, y: rect.y + 1, width: rect.width.saturating_sub(4), height: 1 });
        };
        let line = |frame: &mut Frame, x: u16, y: u16, width: u16, spans: Vec<Span<'static>>| {
            frame.render_widget(Paragraph::new(Line::from(spans)), Rect { x, y, width, height: 1 });
        };
        let long = |date: Date| date.strftime("%a %d %b").to_string();

        let title = format!(" 0.0 · {} · Swimlane roadmap", self.plan.name);
        line(frame, 0, 0, area.width, vec![Span::styled("pchart", Style::new().fg(bright).add_modifier(bold)), Span::styled(title, Style::new().fg(dim))]);
        let right = format!("{} · spike, not saved ", self.path);
        line(frame, area.width.saturating_sub(right.chars().count() as u16), 0, right.chars().count() as u16, vec![Span::styled(right, Style::new().fg(dim))]);

        let quarter = area.width / 4;
        let tiles = [(0, quarter), (quarter, quarter), (2 * quarter, quarter), (3 * quarter, area.width - 3 * quarter)];
        let target = self.schedule.target.map(|index| (&self.plan.items[index], self.schedule.slots[index].planned.finish));
        let first = self.schedule.chain.first().map(|&index| (&self.plan.items[index], &self.schedule.slots[index]));
        let mut tile = |index: usize, title: &str, big: String, big_color: Color, side: String, foot: String| {
            let (x, width) = tiles[index];
            let rect = Rect { x, y: 1, width, height: 6 };
            panel(frame, rect, title);
            line(frame, x + 2, 3, width - 4, vec![Span::styled(big, Style::new().fg(big_color).add_modifier(bold)), Span::styled(format!("  {side}"), Style::new().fg(dim))]);
            line(frame, x + 2, 5, width - 4, vec![Span::styled(foot, Style::new().fg(dim))]);
        };
        match target {
            Some((item, day)) => tile(0, "TARGET · FIXED", day.strftime("%d %b").to_string(), bright, day.strftime("%a · %Y").to_string(), item.name.clone()),
            None => tile(0, "END", String::new(), bright, String::new(), String::new()),
        }
        if let Some((item, slot)) = first {
            let day = slot.latest.start;
            tile(1, "LATEST START", day.strftime("%d %b").to_string(), bright, day.strftime("%a · %Y").to_string(), item.name.clone());
        }
        if let Some(spare) = self.schedule.spare() {
            let tone = if spare < 0 { color(p.alert) } else { bright };
            tile(2, "TO SPARE", spare.to_string(), tone, "working days".into(), "before the target".into());
        }
        match self.cost {
            Some((cost, total)) => {
                let ms = |d: Duration| d.as_secs_f64() * 1000.0;
                tile(3, "LAST PICTURE", format!("{:.0} ms", ms(total)), bright, self.protocol.into(), format!("text {:.0} · raster {:.0} · encode {:.0} ms", ms(cost.parse), ms(cost.raster), ms(cost.encode)));
            }
            None => tile(3, "LAST PICTURE", String::new(), bright, self.protocol.into(), String::new()),
        }

        let chart_panel = Rect { x: 0, y: 7, width: area.width - SIDE, height: area.height - 8 };
        let first_day = self.schedule.slots.iter().map(|slot| slot.planned.start).min().unwrap_or(self.today).min(self.today);
        let last_day = self.schedule.slots.iter().map(|slot| slot.planned.finish).max().unwrap_or(self.today);
        panel(frame, chart_panel, &format!("SWIMLANE ROADMAP · {} – {}", first_day.strftime("%b %Y"), last_day.strftime("%b %Y")).to_uppercase());
        let chart = App::chart(area);
        if let Some((_, rect, protocol)) = &self.picture
            && *rect == App::picture_rect(area)
        {
            frame.render_widget(Image::new(protocol), *rect);
        }

        let legend_y = chart.y + chart.height + 1;
        line(frame, 2, legend_y, chart.width, vec![
            Span::styled("▬ ", Style::new().fg(ink)), Span::styled("decides the date   ", Style::new().fg(dim)),
            Span::styled("▬ ", Style::new().fg(muted)), Span::styled("has time to spare   ", Style::new().fg(dim)),
            Span::styled("◆ ", Style::new().fg(ink)), Span::styled("milestone   ", Style::new().fg(dim)),
            Span::styled("▨ ", Style::new().fg(muted)), Span::styled("holiday in this lane   ", Style::new().fg(dim)),
            Span::styled("│ ", Style::new().fg(ink)), Span::styled("today", Style::new().fg(dim)),
        ]);
        let mut chain = vec![Span::styled("decides the date  ", Style::new().fg(muted))];
        for (position, &index) in self.schedule.chain.iter().enumerate() {
            if position > 0 {
                chain.push(Span::styled(" → ", Style::new().fg(muted)));
            }
            let style = if index == self.selected { Style::new().fg(bright).add_modifier(bold) } else { Style::new().fg(ink) };
            chain.push(Span::styled(self.plan.items[index].name.clone(), style));
        }
        line(frame, 2, legend_y + 2, chart.width, chain);

        let side = Rect { x: area.width - SIDE, y: 7, width: SIDE, height: 19.min(area.height - 8) };
        panel(frame, side, "SELECTION");
        let item = &self.plan.items[self.selected];
        let slot = &self.schedule.slots[self.selected];
        let lane = &self.plan.lanes[self.plan.lane(&item.lane).unwrap_or(0)];
        let (x, w) = (side.x + 2, SIDE - 4);
        line(frame, x, 9, w, vec![Span::styled(item.name.clone(), Style::new().fg(bright).add_modifier(bold))]);
        let place = [Some(lane.name.as_str()), lane.city.as_deref(), lane.place.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
        line(frame, x, 10, w, vec![Span::styled(format!("Lane {place}"), Style::new().fg(muted))]);
        let names = |ids: Vec<&str>| if ids.is_empty() { "—".to_string() } else { ids.join(", ") };
        let waits = names(item.after.iter().filter_map(|id| self.plan.item(id)).map(|index| self.plan.items[index].name.as_str()).collect());
        let then = names(self.plan.items.iter().filter(|other| other.after.contains(&item.id)).map(|other| other.name.as_str()).collect());
        let decides = self.schedule.chain.contains(&self.selected);
        let rows: Vec<(&str, String, Color)> = vec![
            ("Duration", if item.milestone { "milestone".into() } else { format!("{} working days", item.duration) }, ink),
            ("Waits on", waits, ink),
            ("Then", then, ink),
            ("Planned", format!("{} – {}", long(slot.planned.start), long(slot.planned.finish)), ink),
            ("Latest", format!("{} – {}", long(slot.latest.start), long(slot.latest.finish)), dim),
            ("To spare", format!("{} wd{}", slot.spare, if decides { " · decides the date" } else { "" }), if slot.spare < 0 { color(p.alert) } else { ink }),
            ("Date", if item.on.is_some() { "fixed".into() } else { "calculated".into() }, dim),
        ];
        for (row, (key, value, tone)) in rows.into_iter().enumerate() {
            line(frame, x, 12 + row as u16, w, vec![Span::styled(format!("{key:<12}"), Style::new().fg(muted)), Span::styled(value, Style::new().fg(tone))]);
        }
        let calendar = &self.calendars[self.plan.lane(&item.lane).unwrap_or(0)];
        let weekday = |day: Date| !matches!(day.weekday(), jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday);
        let holidays: Vec<String> = calendar.holidays(slot.planned.start, slot.planned.finish).filter(|(day, _)| weekday(*day)).map(|(day, name)| format!("{} {name}", day.strftime("%d %b"))).collect();
        line(frame, x, 20, 12, vec![Span::styled("Holidays", Style::new().fg(muted))]);
        for (row, text) in holidays.iter().take(5).enumerate() {
            line(frame, x + 12, 20 + row as u16, w - 12, vec![Span::styled(text.clone(), Style::new().fg(ink))]);
        }

        let picture_panel = Rect { x: area.width - SIDE, y: 26, width: SIDE, height: area.height.saturating_sub(27) };
        if picture_panel.height >= 6 {
            panel(frame, picture_panel, "PICTURE");
            let (cw, ch) = self.cell;
            let info = [
                ("Protocol", self.protocol.to_string()),
                ("Cell", format!("{cw}×{ch} px")),
                ("Picture", format!("{}×{} cells", chart.width, chart.height.saturating_sub(1))),
                ("", format!("{}×{} px", u32::from(chart.width) * u32::from(cw), u32::from(chart.height.saturating_sub(1)) * u32::from(ch))),
            ];
            for (row, (key, value)) in info.into_iter().enumerate() {
                line(frame, x, 28 + row as u16, w, vec![Span::styled(format!("{key:<12}"), Style::new().fg(muted)), Span::styled(value, Style::new().fg(ink))]);
            }
        }

        let mut keys = vec![];
        for (key, label) in [("↑↓", "item"), ("←→", "duration"), ("q", "quit")] {
            keys.push(Span::styled(key, Style::new().fg(ink).add_modifier(bold)));
            keys.push(Span::styled(format!(" {label}    "), Style::new().fg(dim)));
        }
        if let Some(message) = &self.message {
            keys.push(Span::styled(message.clone(), Style::new().fg(color(p.warn))));
        }
        line(frame, 1, area.height - 1, area.width - 1, keys);
    }

    /// The rect the chart picture fills, one row below the panel title.
    pub fn picture_rect(area: Rect) -> Rect {
        let chart = App::chart(area);
        Rect { y: chart.y + 1, height: chart.height.saturating_sub(1), ..chart }
    }
}
