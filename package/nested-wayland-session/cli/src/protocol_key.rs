//! The `key` command:
//!  a chord,
//!  and press,
//!  release,
//!  tap,
//!  or hold for a duration.

/// Sibling modules of this crate.
use super::{Command, KeyAction};

/// Longest `hold` the protocol accepts,
///  in milliseconds.
///
/// What:
///      `pub const MAX_HOLD_MS: u64 = 600_000;`.
/// Why:
///       One control connection is served at a time,
///  so an absurd hold would block every later command for hours.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_HOLD_MS = 600_000;
/// ```
pub const MAX_HOLD_MS: u64 = 600_000;

/// Parse the arguments of a `key` command.
///
/// What:
///      `pub(super) fn parse(tokens: &mut SplitWhitespace) -> Result<Command, String>`.
///  `key SPEC [press|release|tap|hold MS]`;
///  `SPEC` is a chord such as `ctrl+shift+f10`,
///  a name,
///  or `code:<n>`.
///  The action defaults to `tap`.
/// Why:
///       Press,
///  release,
///  and tap model a finger on one key;
///  hold keeps the key down for a time so the client's own key repeat runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseKey(tokens: string[]): Command { ... }
/// ```
pub(super) fn parse(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let name = tokens.next().ok_or_else(|| return "key requires a name".to_string())?.to_string();
    let action = match tokens.next() {
        Some("press") => KeyAction::Press,
        Some("release") => KeyAction::Release,
        Some("tap") | None => KeyAction::Tap,
        Some("hold") => KeyAction::Hold { millis: parse_hold(tokens.next())? },
        Some(other) => return Err(format!("unknown key action: {other}; use press, release, tap, or hold MS")),
    };
    if tokens.next().is_some() {
        return Err("key takes a name and at most one action".to_string())
    }
    return Ok(Command::Key { name, action })
}

/// Parse the duration after `hold`.
///
/// What:
///      `fn parse_hold(token: Option<&str>) -> Result<u64, String>`.
///  Whole milliseconds from 1 to `MAX_HOLD_MS`.
/// Why:
///       A hold of zero is a tap,
///  and a missing duration is a typo that must not silently tap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseHold(token?: string): number { ... }
/// ```
fn parse_hold(token: Option<&str>) -> Result<u64, String> {
    let text = token.ok_or_else(|| return "key hold requires a duration in milliseconds".to_string())?;
    let millis = text
        .parse::<u64>()
        .map_err(|_| return format!("key hold duration is not a whole number of milliseconds: {text}"))?;
    if millis == 0 || millis > MAX_HOLD_MS {
        return Err(format!("key hold duration must be from 1 to {MAX_HOLD_MS} milliseconds, got: {millis}"))
    }
    return Ok(millis)
}
