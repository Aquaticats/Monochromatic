//! Parse the runtime `accent-color` control verb.

/// Imports `Command` from `super`.
use super::Command;

/// Imports `AccentColor` from `crate::accent_color`.
use crate::accent_color::AccentColor;

/// Parse the tokens after `accent-color` into the typed command.
///
/// What:
///      `pub(super) fn parse(tokens) -> Result<Command, String>`.
///  Exactly one value,
///  the same six hexadecimal digits `--accent-color` accepts.
/// Why:
///       A second token or an embedded newline must fail whole instead of changing the color.
pub(super) fn parse(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let Some(text) = tokens.next() else {
        return Err("accent-color requires a color as six hexadecimal digits (RRGGBB)".to_string());
    };
    if tokens.next().is_some() {
        return Err("accent-color takes exactly one color: RRGGBB".to_string());
    }
    return match AccentColor::parse(text) {
        Ok(color) => Ok(Command::AccentColor(color)),
        Err(message) => Err(format!("accent-color {message}")),
    };
}
