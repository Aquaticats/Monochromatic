//! What:
//!  UTC timestamps in the exact form of JavaScript's `Date.prototype.toISOString`:
//!       `YYYY-MM-DDTHH:MM:SS.mmmZ`.
//! Why:
//!  Transaction owner records carry `createdAt` and capture records `invokedAt` in this
//!      form;
//!  recovery orders dead transactions and the reservation grants turns by comparing
//!      these strings,
//!  so both wrappers must write the same shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! new Date().toISOString()
//! ```

/// Milliseconds in one day.
const MILLISECONDS_PER_DAY: i64 = 86_400_000;

/// What:
///  The civil date of a day count since 1970-01-01,
///  as (year,
///  month,
///  day).
///       `(i64, u32, u32)` is a tuple of three numbers.
/// Why:
///   Howard Hinnant's `civil_from_days`,
///  exact for the proleptic Gregorian calendar
///       JavaScript dates use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const d = new Date(days * 86400000); [d.getUTCFullYear(), d.getUTCMonth() + 1, d.getUTCDate()]
/// ```
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted: i64 = days + 719_468;
    let era: i64 = shifted.div_euclid(146_097);
    let day_of_era: i64 = shifted.rem_euclid(146_097);
    let year_of_era: i64 =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year: i64 = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index: i64 = (5 * day_of_year + 2) / 153;
    let day: i64 = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month: i64 = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year: i64 = year_of_era + era * 400 + i64::from(month <= 2);
    // Both fit: month is 1 to 12 and day 1 to 31.
    return (year, month as u32, day as u32);
}

/// What:
///  Format milliseconds since the Unix epoch as `toISOString` does for years 0 to 9999.
/// Why:
///   The wrapper only formats the current time;
///  the four-digit year form covers it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Date(milliseconds).toISOString()
/// ```
pub fn format_iso_milliseconds(milliseconds: i64) -> String {
    let days: i64 = milliseconds.div_euclid(MILLISECONDS_PER_DAY);
    let within: i64 = milliseconds.rem_euclid(MILLISECONDS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    let hours: i64 = within / 3_600_000;
    let minutes: i64 = within / 60_000 % 60;
    let seconds: i64 = within / 1000 % 60;
    let millis: i64 = within % 1000;
    return format!(
        "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}.{millis:03}Z"
    );
}

/// What:
///  The current time as `toISOString` text.
/// Why:
///   Owner and capture records record when the invocation started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Date().toISOString()
/// ```
pub fn now_iso() -> String {
    // A clock before 1970 reads as the epoch; nothing orders by such a time.
    let elapsed: std::time::Duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let milliseconds: i64 = i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX);
    return format_iso_milliseconds(milliseconds);
}

/// Calendar controls stay out of the release executable.
#[cfg(test)]
#[path = "iso_time_tests.rs"]
mod tests;
