//! What:
//! Reads and writes the timestamps the registry gives:
//! RFC 3339 text such as `2026-10-29T12:34:56.123456+00:00` to
//! milliseconds since 1970 and back.
//! Why:
//! The release-age and trust rules compare publish times with "now" and with each other;
//! the catalog has no date
//! library,
//! and the two shapes crates.io uses are small enough to read directly.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // time-text.ts
//! export const parseMillis = (text: string) => Date.parse(text);
//! export const formatIso = (ms: number) => new Date(ms).toISOString();
//! ```

/// Milliseconds in one minute.
pub const MS_PER_MINUTE: i64 = 60_000;

/// Milliseconds in one second.
const MS_PER_SECOND: i64 = 1_000;

/// Seconds in one day.
const SECONDS_PER_DAY: i64 = 86_400;

/// Days from 0000-03-01 to 1970-01-01 in the proleptic Gregorian calendar (the origin of the civil-date arithmetic).
const DAYS_TO_UNIX_EPOCH: i64 = 719_468;

/// Days in one 400-year Gregorian cycle.
const DAYS_PER_CYCLE: i64 = 146_097;

/// What:
/// `fn days_from_civil(year: i64, month: i64, day: i64) -> i64` counts the days from 1970-01-01 to a calendar date.
/// Why:
/// Howard Hinnant's well-known `days_from_civil` arithmetic:
/// no table of month lengths,
/// exact for every Gregorian date.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const daysFromCivil = (y: number, m: number, d: number) => Date.UTC(y, m - 1, d) / 86_400_000;
/// ```
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let shifted_year: i64 = if month <= 2 { year - 1 } else { year };
    let era: i64 = shifted_year.div_euclid(400);
    let year_of_era: i64 = shifted_year - era * 400;
    let month_from_march: i64 = (month + 9) % 12;
    let day_of_year: i64 = (153 * month_from_march + 2) / 5 + day - 1;
    let day_of_era: i64 = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    return era * DAYS_PER_CYCLE + day_of_era - DAYS_TO_UNIX_EPOCH;
}

/// What:
/// `fn civil_from_days(days: i64) -> (i64, i64, i64)` is the inverse of `days_from_civil`:
/// a day count to (year,
/// month,
/// day).
/// Why:
/// Formatting a cutoff time needs the calendar date back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const civilFromDays = (days: number) => { const d = new Date(days * 86_400_000); return [d.getUTCFullYear(), d.getUTCMonth() + 1, d.getUTCDate()]; };
/// ```
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted: i64 = days + DAYS_TO_UNIX_EPOCH;
    let era: i64 = shifted.div_euclid(DAYS_PER_CYCLE);
    let day_of_era: i64 = shifted - era * DAYS_PER_CYCLE;
    let year_of_era: i64 = (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146_096) / 365;
    let day_of_year: i64 = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march: i64 = (5 * day_of_year + 2) / 153;
    let day: i64 = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month: i64 = if month_from_march < 10 { month_from_march + 3 } else { month_from_march - 9 };
    let year: i64 = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };
    return (year, month, day);
}

/// A reader over the timestamp text that moves forward as it takes pieces.
struct Cursor<'a> {
    /// What:
    /// Field `remaining` is the part of the timestamp text not yet read (`&'a str` is a borrowed slice that must live as long as the text it points into).
    /// Why:
    /// Each reading step consumes a piece from the front without copying the text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// remaining: string
    /// ```
    remaining: &'a str,
}

/// What:
/// `impl<'a> Cursor<'a> { ... }` attaches the reading steps to the cursor (`'a` ties the cursor to the text it reads).
/// Why:
/// The timestamp parser is a straight line of named steps instead of repeated slicing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Cursor { digits(n) {} expect(c) {} accept(c) {} }
/// ```
impl<'a> Cursor<'a> {
    /// Takes exactly `length` ASCII digits and returns their number.
    fn digits(&mut self, length: usize) -> Option<i64> {
        if self.remaining.len() < length || !self.remaining.is_char_boundary(length) {
            return None;
        }
        let (head, tail) = self.remaining.split_at(length);
        if head.bytes().take_while(u8::is_ascii_digit).count() != head.len() {
            return None;
        }
        let value: i64 = head.parse().ok()?;
        self.remaining = tail;
        return Some(value);
    }

    /// Takes one expected character.
    fn expect(&mut self, expected: char) -> Option<()> {
        self.remaining = self.remaining.strip_prefix(expected)?;
        return Some(());
    }

    /// Takes one expected character if it is next;
    /// says whether it was.
    fn accept(&mut self, expected: char) -> bool {
        return self.expect(expected).is_some();
    }
}

/// What:
/// `pub fn parse_rfc3339_millis(text: &str) -> Option<i64>` reads `YYYY-MM-DDTHH:MM:SS[.fraction](Z|+HH:MM|-HH:MM)`.
/// Why:
/// Both crates.io shapes (`2026-10-29T12:00:00Z` from the sparse index and `2026-10-29T12:00:00.123456+00:00` from the API)
/// are covered;
/// anything else is `None`,
/// which callers treat as a missing time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseRfc3339Millis(text: string): number | undefined { const ms = Date.parse(text); return Number.isNaN(ms) ? undefined : ms; }
/// ```
pub fn parse_rfc3339_millis(text: &str) -> Option<i64> {
    let mut cursor: Cursor = Cursor { remaining: text };
    let year: i64 = cursor.digits(4)?;
    cursor.expect('-')?;
    let month: i64 = cursor.digits(2)?;
    cursor.expect('-')?;
    let day: i64 = cursor.digits(2)?;
    if !cursor.accept('T') {
        cursor.expect(' ')?;
    }
    let hour: i64 = cursor.digits(2)?;
    cursor.expect(':')?;
    let minute: i64 = cursor.digits(2)?;
    cursor.expect(':')?;
    let second: i64 = cursor.digits(2)?;
    let mut millis: i64 = 0;
    if cursor.accept('.') {
        let digit_count: usize = cursor.remaining.bytes().take_while(u8::is_ascii_digit).count();
        if digit_count == 0 {
            return None;
        }
        let (digits, tail) = cursor.remaining.split_at(digit_count);
        let mut padded: String = digits.chars().take(3).collect();
        while padded.len() < 3 {
            padded.push('0');
        }
        millis = padded.parse().ok()?;
        cursor.remaining = tail;
    }
    let mut offset_minutes: i64 = 0;
    if !cursor.accept('Z') && !cursor.accept('z') {
        let sign: i64 = if cursor.accept('+') {
            1
        } else if cursor.accept('-') {
            -1
        } else {
            return None;
        };
        let offset_hour: i64 = cursor.digits(2)?;
        cursor.expect(':')?;
        let offset_minute: i64 = cursor.digits(2)?;
        offset_minutes = sign * (offset_hour * 60 + offset_minute);
    }
    if !cursor.remaining.is_empty() {
        return None;
    }
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let days: i64 = days_from_civil(year, month, day);
    let seconds: i64 = days * SECONDS_PER_DAY + hour * 3600 + minute * 60 + second - offset_minutes * 60;
    return Some(seconds * MS_PER_SECOND + millis);
}

/// What:
/// `pub fn format_iso_millis(millis: i64) -> String` writes UTC time as `YYYY-MM-DDTHH:MM:SS.mmmZ`,
/// like JavaScript's
/// `toISOString`.
/// Why:
/// Messages name the moment a young release matures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const formatIsoMillis = (ms: number) => new Date(ms).toISOString();
/// ```
pub fn format_iso_millis(millis: i64) -> String {
    let total_seconds: i64 = millis.div_euclid(MS_PER_SECOND);
    let fraction: i64 = millis.rem_euclid(MS_PER_SECOND);
    let days: i64 = total_seconds.div_euclid(SECONDS_PER_DAY);
    let seconds_of_day: i64 = total_seconds.rem_euclid(SECONDS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    return format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{fraction:03}Z",
        seconds_of_day / 3600,
        (seconds_of_day % 3600) / 60,
        seconds_of_day % 60
    );
}

/// What:
/// `pub fn publish_time_cutoff(now_millis: i64, minutes: u64) -> String` is cargo's `--publish-time` value:
/// now minus the
/// release age,
/// written `YYYY-MM-DDTHH:MM:SSZ`.
/// Why:
/// `cargo -Zunstable-options generate-lockfile --publish-time` takes exactly that shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const publishTimeCutoff = ({ now, minutes }) => `${new Date(now - minutes * 60_000).toISOString().slice(0, 19)}Z`;
/// ```
pub fn publish_time_cutoff(now_millis: i64, minutes: u64) -> String {
    let cutoff: i64 = now_millis - (minutes as i64) * MS_PER_MINUTE;
    let text: String = format_iso_millis(cutoff);
    return format!("{}Z", &text[..19]);
}
