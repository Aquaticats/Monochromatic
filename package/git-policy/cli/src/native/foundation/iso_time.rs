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

/// The last millisecond this process wrote a stamp for,
/// so the next stamp is never the same one.
static LAST_MILLISECOND: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(i64::MIN);

/// What:
///  A `toISOString` stamp this process has not written before:
/// the current millisecond,
/// or the one after the last stamp it wrote.
/// Why:
///   A stamp can be part of what names a record
/// (`package_repair_prewarm.rs`),
///   and two records asked for inside one millisecond would otherwise be named alike,
/// so a queue that answers a name it already knows as known would drop the second one.
///   The stamp is a moment this process asks for and never one before it,
/// and it is at most one millisecond past the clock for each stamp the clock cannot tell apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let last = -Infinity; const nowISO = () => { last = Math.max(Date.now(), last + 1); return new Date(last).toISOString(); };
/// ```
pub fn now_iso_never_the_same_twice() -> String {
    /// The atomic order of a reading that only has to be a value some thread wrote.
    use std::sync::atomic::Ordering;
    let mut last: i64 = LAST_MILLISECOND.load(Ordering::Relaxed);
    loop {
        let now: i64 = jiff::Timestamp::now().as_millisecond();
        let chosen: i64 = if now > last { now } else { last.saturating_add(1) };
        match LAST_MILLISECOND.compare_exchange_weak(last, chosen, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_written) => return format_iso_milliseconds(chosen),
            // Another thread wrote its own stamp first, so this one asks again from that stamp.
            Err(current) => last = current,
        }
    }
}

/// Calendar controls stay out of the release executable.
#[cfg(test)]
#[path = "iso_time_tests.rs"]
mod tests;
