//! Dates typed loosely, as the date question takes them: "4 jan", "jan 4", "04 Jan 2027", "2027-01-04",
//! "friday", "next friday", "W1", "3w", "10d", "today", "tomorrow". A date without a year is the next
//! one from `today` on.

use jiff::civil::{Date, Weekday};

use crate::calendar::next;

const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
const DAYS: [(&str, Weekday); 7] = [("mon", Weekday::Monday), ("tue", Weekday::Tuesday), ("wed", Weekday::Wednesday), ("thu", Weekday::Thursday), ("fri", Weekday::Friday), ("sat", Weekday::Saturday), ("sun", Weekday::Sunday)];

fn month(word: &str) -> Option<i8> {
    (word.len() >= 3).then(|| MONTHS.iter().position(|month| word.starts_with(month))).flatten().map(|index| index as i8 + 1)
}

fn weekday(word: &str) -> Option<Weekday> {
    (word.len() >= 3).then(|| DAYS.iter().find(|(name, _)| word.starts_with(name))).flatten().map(|(_, day)| *day)
}

fn days_from(today: Date, count: i64) -> Date {
    (0..count).fold(today, |day, _| next(day))
}

/// The date `text` names, counted from `today`; None when it names none.
pub fn date(text: &str, today: Date) -> Option<Date> {
    let text = text.trim().to_lowercase();
    let words: Vec<&str> = text.split(|c: char| c.is_whitespace() || c == ',').filter(|word| !word.is_empty()).collect();
    match words.as_slice() {
        [] => None,
        ["today"] => Some(today),
        ["tomorrow"] => Some(next(today)),
        [iso] if iso.len() == 10 && iso.chars().nth(4) == Some('-') => iso.parse::<Date>().ok(),
        [week] if week.starts_with('w') && week.len() <= 3 && week[1..].chars().all(|c| c.is_ascii_digit()) => {
            let number: i8 = week[1..].parse().ok()?;
            [today.year(), today.year() + 1].into_iter().filter_map(|year| jiff::civil::ISOWeekDate::new(year, number, Weekday::Monday).ok()).map(|week| week.date()).find(|day| *day >= today)
        }
        [span] if span.trim_start_matches('+').len() >= 2 && (span.ends_with('w') || span.ends_with('d')) => {
            let count: i64 = span.trim_start_matches('+')[..span.trim_start_matches('+').len() - 1].parse().ok()?;
            Some(days_from(today, if span.ends_with('w') { count * 7 } else { count }))
        }
        ["next", day] | [day] if weekday(day).is_some() => {
            let want = weekday(day)?;
            (1..=7).map(|count| days_from(today, count)).find(|day| day.weekday() == want)
        }
        [a, b, rest @ ..] if rest.len() <= 1 => {
            let (number, name) = if a.chars().all(|c| c.is_ascii_digit()) { (*a, *b) } else { (*b, *a) };
            let day: i8 = number.parse().ok()?;
            let month = month(name)?;
            match rest {
                [year] => Date::new(year.parse().ok()?, month, day).ok(),
                _ => [today.year(), today.year() + 1].into_iter().filter_map(|year| Date::new(year, month, day).ok()).find(|date| *date >= today),
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date as d;

    #[test]
    fn loose_dates_are_understood_from_today() {
        let today = d(2026, 10, 29);
        for (text, want) in [
            ("4 jan", d(2027, 1, 4)),
            ("Jan 4", d(2027, 1, 4)),
            ("04 Jan 2027", d(2027, 1, 4)),
            ("2027-01-04", d(2027, 1, 4)),
            ("30 oct", d(2026, 10, 30)),
            ("friday", d(2026, 10, 30)),
            ("next monday", d(2026, 11, 2)),
            ("W1", d(2027, 1, 4)),
            ("w45", d(2026, 11, 2)),
            ("3w", d(2026, 11, 19)),
            ("10d", d(2026, 11, 8)),
            ("today", today),
            ("tomorrow", d(2026, 10, 30)),
        ] {
            assert_eq!(date(text, today), Some(want), "{text}");
        }
        for text in ["", "soon", "31 feb", "4", "jan"] {
            assert_eq!(date(text, today), None, "{text}");
        }
    }
}
