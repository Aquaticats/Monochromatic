//! Wire text for taskbar progress answers.
//!
//! Every answer stays on one line and uses only the separators the protocol documents,
//!  so a client-chosen URI or record cannot break the response framing.

/// Imports the update record that every formatter here reads.
use crate::launcher_history::LauncherUpdate;

/// Characters that pass through an encoded URI unchanged.
///
/// What:
///      `fn is_uri_safe(byte: u8) -> bool`.
///  Letters, digits,
///  and the marks `-`, `.`, `_`, `~`, `:`, `/`.
/// Why:
///       These cover ordinary `application://` URIs,
///  so the common case reads the same as the app sent it,
///  while spaces, `;`, `=`, and line breaks always become escapes.
fn is_uri_safe(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b':' | b'/')
}

/// Percent-encodes an app URI so it contains no space, `;`, `=`, or line break.
///
/// What:
///      `pub fn encode_uri(text: &str) -> String`.
///  Every byte outside the safe set becomes `%XX` with two uppercase hexadecimal digits.
/// Why:
///       The `launcher` answer is a list of `key=value` fields separated by spaces,
///  and a URI from the client could contain any of those characters.
///
/// @example
/// ```ts
/// encodeUri("application://a b.desktop"); // => "application://a%20b.desktop"
/// ```
pub fn encode_uri(text: &str) -> String {
    // What:     `let mut encoded = String::with_capacity(text.len());` then a byte loop.
    //           Each byte is either copied or written as `%` plus its two hex digits.
    // Why:      Bytes, not characters, are the unit percent-encoding defines,
    //           so multi-byte UTF-8 is escaped byte by byte.
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if is_uri_safe(byte) {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    return encoded
}

/// Formats the `launcher` answer payload for the newest update.
///
/// What:
///      `pub fn format_latest(update: &LauncherUpdate, now_ms: u64) -> String`.
///  Fields appear in the order `uri`, `progress`, `visible`, `count`, `count-visible`,
///  `urgent`, `age-ms`.
/// Why:
///       One line lets a test read every field of the newest update in a single round trip.
pub fn format_latest(update: &LauncherUpdate, now_ms: u64) -> String {
    // What:     `now_ms.saturating_sub(update.at_ms)`. Age of the update, never negative.
    // Why:      The update may be newer than the `now_ms` the caller sampled.
    let age_ms = now_ms.saturating_sub(update.at_ms);
    return format!(
        "uri={} progress={:.4} visible={} count={} count-visible={} urgent={} age-ms={age_ms}",
        encode_uri(&update.app_uri),
        update.progress,
        update.progress_visible,
        update.count,
        update.count_visible,
        update.urgent,
    )
}

/// Formats one record of the `launcher-log` answer.
///
/// What:
///      `pub fn format_record(update: &LauncherUpdate) -> String`.
///  Produces `t=<ms> progress=<value> visible=<bool>`.
/// Why:
///       Progress and its visibility are what a test compares against the values it emitted.
pub fn format_record(update: &LauncherUpdate) -> String {
    return format!(
        "t={} progress={:.4} visible={}",
        update.at_ms, update.progress, update.progress_visible,
    )
}

/// Formats the `launcher-log` answer payload: a count, then the records.
///
/// What:
///      `pub fn format_log(records: &[&LauncherUpdate]) -> String`.
///  Empty input gives `0`.
///  Otherwise the count is followed by one space and the records joined by `; `.
/// Why:
///       The count is the number of records on this line,
///  so the reader knows how many to expect without counting separators.
pub fn format_log(records: &[&LauncherUpdate]) -> String {
    // What:     `if records.is_empty() { return "0".to_string(); }`. Avoid a trailing space.
    // Why:      The documented empty answer is exactly `ok 0`.
    if records.is_empty() {
        return "0".to_string();
    }
    let rendered: Vec<String> = records.iter().map(|record| return format_record(record)).collect();
    return format!("{} {}", records.len(), rendered.join("; "))
}

/// Unit tests for the answer text.
#[cfg(test)]
#[path = "launcher_format_tests.rs"]
mod tests;
