//! What:
//!  The time stamps in the names of saved-copy directories, and the entry that carries one.
//! Why:
//!  `discarded_prune.rs` lists, plans and removes; reading a directory name as a time needs only the calendar, so it lives in a file of its own that `discarded_prune.rs` imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const millis = parseStamp("2026-11-02T14-01-07"); // number | undefined
//! ```

use crate::discarded_store::Identity;

/// A copy younger than this is never removed, by either limit. No measurement sets this yet; see `DEFAULT_MAX_AGE_DAYS`.
pub const YOUNG_FLOOR_MILLISECONDS: i64 = 24 * 60 * 60 * 1000;

/// Milliseconds in a day.
pub const DAY_MILLISECONDS: i64 = 24 * 60 * 60 * 1000;

/// The resolution of a directory name: it says the second of the save, and the save may have been made at any millisecond
/// of that second, so an age is counted from the end of it.
pub const STAMP_RESOLUTION_MILLISECONDS: i64 = 1000;

/// Days from 1970-01-01 to a civil date, the inverse of `iso_time::civil_from_days` (Howard Hinnant's `days_from_civil`).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let shifted_year: i64 = if month <= 2 { year - 1 } else { year };
    let era: i64 = shifted_year.div_euclid(400);
    let year_of_era: i64 = shifted_year.rem_euclid(400);
    let shifted_month: i64 = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year: i64 = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era: i64 = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    return era * 146_097 + day_of_era - 719_468;
}

/// The days of a month of the proleptic Gregorian calendar, February's by the leap-year rule.
fn days_in_month(year: i64, month: i64) -> i64 {
    return match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        _ => 28,
    };
}

/// The whole number written in the characters `from` to `to` of a name; zero when they are not a number.
fn number_at(name: &str, from: usize, to: usize) -> i64 {
    return name[from..to].parse::<i64>().unwrap_or(0);
}

/// What:
///  The time a copy directory's name stands for, in milliseconds since the epoch, or nothing when the name is not one the
///  wrapper makes.
/// Why:
///   The name (`2026-11-02T14-01-07`, or the same with `-<number>` for a second save in that second) is the time of the
///  save in UTC; reading it, not the directory's modification time, keeps a copy that someone later edited or touched from
///  looking young, and leaves every directory with another name alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseStamp('2026-11-02T14-01-07-3') // 1793628067000
/// ```
pub fn parse_stamp(name: &str) -> Option<i64> {
    let bytes: &[u8] = name.as_bytes();
    if bytes.len() < 19 || !name.is_ascii() {
        return None;
    }
    let shape: &[u8; 19] = b"dddd-dd-ddTdd-dd-dd";
    for (found, wanted) in bytes[..19].iter().zip(shape) {
        let fits: bool = if *wanted == b'd' { found.is_ascii_digit() } else { found == wanted };
        if !fits {
            return None;
        }
    }
    let suffix: &str = &name[19..];
    if !(suffix.is_empty() || (suffix.starts_with('-') && suffix.len() > 1 && suffix.as_bytes()[1..].iter().all(u8::is_ascii_digit))) {
        return None;
    }
    let (year, month, day): (i64, i64, i64) = (number_at(name, 0, 4), number_at(name, 5, 7), number_at(name, 8, 10));
    let (hour, minute, second): (i64, i64, i64) = (number_at(name, 11, 13), number_at(name, 14, 16), number_at(name, 17, 19));
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    return Some(((days_from_civil(year, month, day) * 24 + hour) * 60 + minute) * 60_000 + second * 1000);
}

/// One copy directory, as listed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stamped {
    /// The directory's name.
    pub name: String,
    /// The time its name stands for.
    pub millis: i64,
    /// The bytes of the files it holds; wide, so that no sum of file lengths can wrap.
    pub bytes: u128,
    /// The directory the listing saw under that name.
    pub identity: Identity,
}
