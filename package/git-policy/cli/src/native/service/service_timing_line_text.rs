//! What:
//!  The prefix of every line the service writes,
//!  the wall clock in whole milliseconds,
//!  and the text of one line:
//!  prefix,
//!  words,
//!  and the time of the event at the end.
//! Why:
//!   The journal stamps a line when it receives it,
//!   and under load that can be seconds after the event,
//!   so the line states its own time as `at_ms=<epoch milliseconds>`,
//!   at the end so that the prefix and the first word stay where readers look for them.
//!  Every other file here builds its line out of these two pieces,
//!  and no other file holds them,
//!  so the format of a service line is decided in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! `${LOG_PREFIX}${line} at_ms=${Date.now()}\n`
//! ```
/// The prefix of every line the service writes,
/// which `journalctl` shows after the unit name.
pub const LOG_PREFIX: &str = "monochromatic-git-policy-cli-service: ";

/// The wall clock in whole milliseconds since the Unix epoch,
/// the time a log line states for its own event.
pub fn wall_millis() -> u128 {
    return match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis(),
        Err(_before_the_epoch) => 0,
    };
}

/// What:
///  The text of one log line:
/// the prefix,
/// the line,
/// the time of the event as `at_ms=<epoch milliseconds>` at the end,
/// and the newline.
/// Why:
///   The journal stamps a line when it receives it,
/// and under load that can be seconds after the event;
/// the line states its own time,
/// at the end so that the prefix and the first word stay where readers look for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${LOG_PREFIX}${line} at_ms=${Date.now()}\n`
/// ```
pub fn stamped_line(
    line: &str,
    at_ms: u128,
) -> String {
    return format!("{LOG_PREFIX}{line} at_ms={at_ms}\n");
}
