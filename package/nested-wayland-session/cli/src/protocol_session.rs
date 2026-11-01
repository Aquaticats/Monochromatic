//! Commands about the keyboard repeat announcement and window closing.

/// Sibling modules of this crate.
use super::Command;

/// Largest repeat rate and delay accepted.
///
/// What:
///      `pub const MAX_REPEAT_RATE: i32 = 1_000; pub const MAX_REPEAT_DELAY_MS: i32 = 10_000;`.
/// Why:
///       `wl_keyboard.repeat_info` carries 32-bit integers;
///  these bounds keep a client's repeat timer sensible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_REPEAT_RATE = 1_000, MAX_REPEAT_DELAY_MS = 10_000;
/// ```
pub const MAX_REPEAT_RATE: i32 = 1_000;

/// Largest repeat delay accepted,
///  in milliseconds.
///
/// What:
///      See `MAX_REPEAT_RATE`.
/// Why:
///       Same bound reasoning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_REPEAT_DELAY_MS = 10_000;
/// ```
pub const MAX_REPEAT_DELAY_MS: i32 = 10_000;

/// Parse `repeat-info [RATE DELAY_MS]`.
///
/// What:
///      `pub(super) fn parse_repeat_info(tokens: &mut SplitWhitespace) -> Result<Command, String>`.
///  No arguments reads the current pair.
///  `RATE` is repeats per second (0 turns repeat off for clients),
///  `DELAY_MS` the wait before the first repeat.
/// Why:
///       Wayland clients generate key repeat themselves from this announcement,
///  so the pair decides how many repeats a held key yields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseRepeatInfo(tokens: string[]): Command { ... }
/// ```
pub(super) fn parse_repeat_info(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let Some(rate_text) = tokens.next() else {
        return Ok(Command::RepeatInfo(None))
    };
    let delay_text = tokens.next().ok_or_else(|| return "repeat-info expects RATE DELAY_MS or nothing".to_string())?;
    if tokens.next().is_some() {
        return Err("repeat-info expects RATE DELAY_MS or nothing".to_string())
    }
    let rate = rate_text.parse::<i32>().map_err(|_| return format!("repeat-info rate is not a whole number: {rate_text}"))?;
    let delay = delay_text.parse::<i32>().map_err(|_| return format!("repeat-info delay is not a whole number: {delay_text}"))?;
    if !(0..=MAX_REPEAT_RATE).contains(&rate) {
        return Err(format!("repeat-info rate must be from 0 to {MAX_REPEAT_RATE} per second, got: {rate}"))
    }
    if !(0..=MAX_REPEAT_DELAY_MS).contains(&delay) {
        return Err(format!("repeat-info delay must be from 0 to {MAX_REPEAT_DELAY_MS} milliseconds, got: {delay}"))
    }
    return Ok(Command::RepeatInfo(Some((rate, delay))))
}

/// Parse `close [INDEX]`.
///
/// What:
///      `pub(super) fn parse_close(tokens: &mut SplitWhitespace) -> Result<Command, String>`.
///  `INDEX` counts from 0 in the `windows` listing.
/// Why:
///       A user closes a window with its close button;
///  the compositor sends the same `xdg_toplevel.close` request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseClose(tokens: string[]): Command { ... }
/// ```
pub(super) fn parse_close(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let index = match tokens.next() {
        None => None,
        Some(text) => Some(text.parse::<usize>().map_err(|_| return format!("close index is not a whole number: {text}"))?),
    };
    if tokens.next().is_some() {
        return Err("close takes at most a window index".to_string())
    }
    return Ok(Command::CloseWindow(index))
}
