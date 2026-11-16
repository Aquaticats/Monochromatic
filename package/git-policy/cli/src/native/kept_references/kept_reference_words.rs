//! What:
//!  The words a span of time is printed in:
//!  the age of one kept reference,
//!  and a limit the release ships or a run sets.
//! Why:
//!  The listing and the log line both print an age and a limit,
//!  and a person compares the two,
//!  so one file says how each reads,
//!  and neither of those files has to use the other.
//!  A limit reads in the largest unit that divides it
//!  (`30 days`,
//!  `1 hour`,
//!  `1 second`),
//!  and an age in days and hours,
//!  because that is the comparison a person makes with a limit of days.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const spanWords = (span) => days(span) || hours(span) || minutes(span) || seconds(span);
//! ```

use std::time::Duration;

/// What:
///  The noun for a count,
///  singular for one and plural for everything else.
/// Why:
///   A line that reads "1 minutes" reads as a mistake,
///  and a person's age comparison is the one figure here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const counted = (count, noun) => `${noun}${count === 1 ? '' : 's'}`;
/// ```
pub fn counted(count: u128, noun: &str) -> String {
    return if count == 1 { String::from(noun) } else { format!("{noun}s") };
}

/// What:
///  Milliseconds as the days and hours a person reads,
///  as in "6 days 2 hours".
/// Why:
///  The exact age is beside it as a timestamp;
///  this is the figure a person compares with the limit,
///  and it never rounds a reference past the limit it is judged by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ageWords = (ms) => `${Math.floor(ms / 86400000)} days ${Math.floor(ms / 3600000) % 24} hours`;
/// ```
pub fn age_words(milliseconds: u128) -> String {
    let days: u128 = milliseconds / (24 * 60 * 60 * 1000);
    let hours: u128 = (milliseconds / (60 * 60 * 1000)) % 24;
    let minutes: u128 = (milliseconds / (60 * 1000)) % 60;
    if days > 0 {
        return format!("{days} {} {hours} {}", counted(days, "day"), counted(hours, "hour"));
    }
    if hours > 0 {
        return format!("{hours} {} {minutes} {}", counted(hours, "hour"), counted(minutes, "minute"));
    }
    return format!("{minutes} {}", counted(minutes, "minute"));
}

/// What:
///  A span of time as the limit it is reads in words,
///  in the largest unit that divides it.
/// Why:
///   A control sets the age limit to a second and the release ships thirty days,
///  and "the 0-day age limit" says nothing a person can compare with,
///  while "1 second" says what the run was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spanWords = (span) => days(span) || hours(span) || minutes(span) || seconds(span);
/// ```
pub fn span_words(span: Duration) -> String {
    let seconds: u128 = u128::from(span.as_secs());
    let days: u128 = seconds / (24 * 60 * 60);
    let hours: u128 = (seconds / (60 * 60)) % 24;
    let minutes: u128 = (seconds / 60) % 60;
    if days > 0 {
        return format!("{days} {}", counted(days, "day"));
    }
    if hours > 0 {
        return format!("{hours} {}", counted(hours, "hour"));
    }
    if minutes > 0 {
        return format!("{minutes} {}", counted(minutes, "minute"));
    }
    return format!("{seconds} {}", counted(seconds, "second"));
}

/// Controls for the words stay out of the release executable.
#[cfg(test)]
#[path = "kept_reference_words_tests.rs"]
mod tests;
