//! Parse the `launcher-log` control request.
//!
//! The other taskbar verbs take no arguments and are routed through the protocol's
//!  `no_arguments` helper, so only the optional record count needs a parser here.

/// Reuse the command representation the executor matches on.
use super::Command;

/// The number of records `launcher-log` lists when no count is given.
const DEFAULT_LOG_COUNT: usize = 64;

/// Parse the arguments of a `launcher-log` request (`launcher-log [N]`).
///
/// What:
///      `pub(super) fn parse_log(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String>`.
///  An optional whole number of at least one; the default is 64.
/// Why:
///       A count of zero would answer with no records and hide a mistyped argument.
pub(super) fn parse_log(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    // What:     `let count = match tokens.next() { Some(text) => ..., None => DEFAULT };`.
    //           Parse the optional count, falling back to the default when absent.
    // Why:      A bare `launcher-log` is the common request.
    let count = match tokens.next() {
        Some(text) => text
            .parse::<usize>()
            .map_err(|_| return "launcher-log count is not a whole number".to_string())?,
        None => DEFAULT_LOG_COUNT,
    };
    if count == 0 {
        return Err("launcher-log count must be at least 1".to_string());
    }
    if tokens.next().is_some() {
        return Err("launcher-log takes at most one count".to_string());
    }
    return Ok(Command::LauncherLog { count });
}
