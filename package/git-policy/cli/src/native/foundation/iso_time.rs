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

/// The shape of a stamp:
/// year to millisecond with a `Z`,
/// as `toISOString` writes it.
/// `%3f` is always three fractional digits.
const STAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%S.%3fZ";

/// What:
///  Format milliseconds since the Unix epoch as `toISOString` does for years 0 to 9999.
/// Why:
///   The wrapper only formats the current time;
/// the four-digit year form covers it.
/// The calendar is the published `jiff`
///  crate's (the owner's ruling of 2026-11-07,
/// "Time stamps:
/// jiff");
/// an instant outside its range (before year -9999 or after
///  9999) is clamped to the range's end,
/// where the hand-written arithmetic printed a meaningless date.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Date(milliseconds).toISOString()
/// ```
pub fn format_iso_milliseconds(milliseconds: i64) -> String {
    let instant: jiff::Timestamp = jiff::Timestamp::from_millisecond(milliseconds).unwrap_or(if milliseconds < 0 {
        jiff::Timestamp::MIN
    } else {
        jiff::Timestamp::MAX
    });
    return instant.strftime(STAMP_FORMAT).to_string();
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
    return format_iso_milliseconds(jiff::Timestamp::now().as_millisecond());
}

/// Calendar controls stay out of the release executable.
#[cfg(test)]
#[path = "iso_time_tests.rs"]
mod tests;
