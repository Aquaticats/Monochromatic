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
    // `jiff` refuses a month, day, hour, minute or second the calendar does not have (February 30, hour 24, minute 60).
    let civil: jiff::civil::DateTime = jiff::civil::DateTime::new(
        i16::try_from(year).ok()?,
        i8::try_from(month).ok()?,
        i8::try_from(day).ok()?,
        i8::try_from(hour).ok()?,
        i8::try_from(minute).ok()?,
        i8::try_from(second).ok()?,
        0,
    )
    .ok()?;
    let instant: jiff::Zoned = civil.to_zoned(jiff::tz::TimeZone::UTC).ok()?;
    return Some(instant.timestamp().as_millisecond());
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
