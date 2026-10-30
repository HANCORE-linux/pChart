use std::collections::{BTreeMap, HashMap};

use jiff::civil::{Date, Weekday};

#[derive(Debug, Clone, Default)]
pub struct Calendar {
    off: BTreeMap<Date, String>,
}

impl Calendar {
    pub fn new(off: BTreeMap<Date, String>) -> Self {
        Calendar { off }
    }

    pub fn is_working(&self, day: Date) -> bool {
        !matches!(day.weekday(), Weekday::Saturday | Weekday::Sunday) && !self.off.contains_key(&day)
    }

    pub fn holiday(&self, day: Date) -> Option<&str> {
        self.off.get(&day).map(String::as_str)
    }

    pub fn holidays(&self, from: Date, to: Date) -> impl Iterator<Item = (Date, &str)> {
        self.off.range(from..=to).map(|(day, name)| (*day, name.as_str()))
    }

    pub fn on_or_after(&self, mut day: Date) -> Date {
        while !self.is_working(day) {
            day = next(day);
        }
        day
    }

    pub fn on_or_before(&self, mut day: Date) -> Date {
        while !self.is_working(day) {
            day = previous(day);
        }
        day
    }

    /// The last working day of `days` working days that begin on `start`.
    pub fn finish(&self, start: Date, days: u32) -> Date {
        let mut day = self.on_or_after(start);
        for _ in 1..days {
            day = self.on_or_after(next(day));
        }
        day
    }

    /// The first working day of `days` working days that end on `finish`.
    pub fn start(&self, finish: Date, days: u32) -> Date {
        let mut day = self.on_or_before(finish);
        for _ in 1..days {
            day = self.on_or_before(previous(day));
        }
        day
    }

    /// Working days from `from` up to, not including, `to`; negative when `to` comes first.
    pub fn between(&self, from: Date, to: Date) -> i32 {
        let (mut day, end, sign) = if from <= to { (from, to, 1) } else { (to, from, -1) };
        let mut count = 0;
        while day < end {
            if self.is_working(day) {
                count += 1;
            }
            day = next(day);
        }
        sign * count
    }
}

pub fn next(day: Date) -> Date {
    day.tomorrow().expect("a date inside the supported range")
}

pub fn previous(day: Date) -> Date {
    day.yesterday().expect("a date inside the supported range")
}

/// Holidays by region: a country code (`DE`), or a subdivision (`DE-BE`) that differs from its
/// country. In a subdivision, an empty name marks a national holiday it does not keep.
#[derive(Debug, Clone, Default)]
pub struct Holidays {
    regions: HashMap<String, BTreeMap<Date, String>>,
}

impl Holidays {
    /// Reads lines of `REGION<TAB>YYYY-MM-DD<TAB>Name`.
    pub fn parse(text: &str) -> Holidays {
        let mut regions: HashMap<String, BTreeMap<Date, String>> = HashMap::new();
        for line in text.lines() {
            let mut fields = line.splitn(3, '\t');
            let (Some(region), Some(day), Some(name)) = (fields.next(), fields.next(), fields.next()) else {
                continue;
            };
            let Ok(day) = day.parse::<Date>() else { continue };
            regions.entry(region.to_string()).or_default().insert(day, name.to_string());
        }
        Holidays { regions }
    }

    pub fn knows(&self, place: &str) -> bool {
        self.regions.contains_key(place) || place.split_once('-').is_some_and(|(country, _)| self.regions.contains_key(country))
    }

    pub fn calendar(&self, place: Option<&str>) -> Calendar {
        let mut off = BTreeMap::new();
        let Some(place) = place else { return Calendar::new(off) };
        let country = place.split_once('-').map_or(place, |(country, _)| country);
        if let Some(days) = self.regions.get(country) {
            off.extend(days.iter().map(|(day, name)| (*day, name.clone())));
        }
        if country != place
            && let Some(days) = self.regions.get(place)
        {
            for (day, name) in days {
                if name.is_empty() {
                    off.remove(day);
                } else {
                    off.insert(*day, name.clone());
                }
            }
        }
        Calendar::new(off)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn berlin() -> Calendar {
        Holidays::parse("DE\t2026-12-25\tChristmas Day\nDE\t2026-12-26\tSecond Day of Christmas\nDE-BE\t2027-03-08\tInternational Women's Day\n").calendar(Some("DE-BE"))
    }

    #[test]
    fn working_days_skip_weekends_and_holidays() {
        let cal = berlin();
        assert_eq!(cal.finish(date(2026, 12, 23), 3), date(2026, 12, 28));
        assert_eq!(cal.start(date(2026, 12, 28), 3), date(2026, 12, 23));
        assert_eq!(cal.between(date(2026, 12, 23), date(2026, 12, 28)), 2);
        assert_eq!(cal.between(date(2026, 12, 28), date(2026, 12, 23)), -2);
        assert!(!cal.is_working(date(2027, 3, 8)));
    }

    #[test]
    fn a_subdivision_can_skip_a_national_holiday() {
        let holidays = Holidays::parse("US\t2048-10-12\tColumbus Day\nUS-AR\t2048-10-12\t\n");
        assert_eq!(holidays.calendar(Some("US")).holiday(date(2048, 10, 12)), Some("Columbus Day"));
        assert_eq!(holidays.calendar(Some("US-AR")).holiday(date(2048, 10, 12)), None);
        assert_eq!(holidays.calendar(Some("US-NY")).holiday(date(2048, 10, 12)), Some("Columbus Day"));
    }

    #[test]
    fn a_subdivision_adds_to_its_country() {
        let holidays = Holidays::parse("DE\t2026-12-25\tChristmas Day\nDE-BE\t2027-03-08\tInternational Women's Day\n");
        assert!(holidays.knows("DE-BY"));
        assert!(!holidays.knows("FR"));
        let bavaria = holidays.calendar(Some("DE-BY"));
        assert_eq!(bavaria.holiday(date(2026, 12, 25)), Some("Christmas Day"));
        assert_eq!(bavaria.holiday(date(2027, 3, 8)), None);
    }
}
