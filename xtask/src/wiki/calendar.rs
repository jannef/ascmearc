//! The calendar: the months, and the element whose season each is in.

use ascmearc_core::content::Content;
use ascmearc_core::time::GameTime;
use std::fmt::Write as _;

pub(super) fn months(content: &Content) -> String {
    let mut out = String::from("| Month | Name | Season of |\n|---:|---|---|\n");
    for month in 1..=12 {
        let time = GameTime::from_calendar(1, month, 1, 0, 0).expect("a valid date");
        // The elements whose season holds the month (`[flavour] seasons`, #1732).
        let elements: Vec<&str> = (content.horoscope().map(|h| h.elements.as_slice()))
            .unwrap_or_default()
            .iter()
            .filter(|e| {
                (content.flavour())
                    .is_some_and(|f| f.seasons.get(&e.id).is_some_and(|s| s.holds(month)))
            })
            .map(|e| e.name.as_str())
            .collect();
        let season = if elements.is_empty() {
            "—".to_string()
        } else {
            elements.join(", ")
        };
        let _ = writeln!(
            out,
            "| {month} | {} | {season} |",
            content.calendar().month_name(time)
        );
    }
    out
}
