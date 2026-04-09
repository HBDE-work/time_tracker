use chrono::Datelike;
use chrono::Local;
use chrono::NaiveDate;
use chrono::Weekday;

fn parse_weekday(s: &str) -> Option<Weekday> {
    match s.to_lowercase().as_str() {
        "monday" | "mon" | "montag" | "mo" => Some(Weekday::Mon),
        "tuesday" | "tue" | "dienstag" | "di" => Some(Weekday::Tue),
        "wednesday" | "wed" | "mittwoch" | "mi" => Some(Weekday::Wed),
        "thursday" | "thu" | "donnerstag" | "do" => Some(Weekday::Thu),
        "friday" | "fri" | "freitag" | "fr" => Some(Weekday::Fri),
        "saturday" | "sat" | "samstag" | "sa" => Some(Weekday::Sat),
        "sunday" | "sun" | "sonntag" | "so" => Some(Weekday::Sun),
        _ => None,
    }
}

pub(crate) fn resolve_date(day: Option<String>, week: Option<u32>, year: Option<i32>) -> NaiveDate {
    let now = Local::now().date_naive();

    match day {
        None => now,
        Some(day_str) => {
            let weekday =
                parse_weekday(&day_str).unwrap_or_else(|| panic!("Unknown weekday: '{day_str}'"));

            let y = year.unwrap_or(now.year());
            let w = week.unwrap_or(now.iso_week().week());

            NaiveDate::from_isoywd_opt(y, w, weekday)
                .unwrap_or_else(|| panic!("Invalid date: year={y}, week={w}, day={day_str}"))
        }
    }
}
