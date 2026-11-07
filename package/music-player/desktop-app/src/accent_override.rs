//! The accent override that the environment variable `MUSIC_PLAYER_ACCENT` selects.
//!
//! What:
//! `accent_override` maps the variable's text (`RRGGBB`,
//! with or without a leading `#`) to the color that
//! replaces the system accent as the theme root (A3,
//! D108).
//! Why:
//! screenshots,
//! renders and verification runs need a deterministic accent;
//! unset follows the system.

/// Imports the formatting traits that render `AccentError` as text.
use std::fmt;

/// Imports the 8-bit sRGB color that a valid accent value becomes.
use crate::oklch::Srgb8;

/// What:
///      `pub const ACCENT_ENV: &str = "MUSIC_PLAYER_ACCENT";`.
/// `&str` is a borrowed string slice;
/// the sibling is the
///  owned `String`.
/// Why:
///       One name is shared by the startup code and the README;
/// a constant needs no allocation,
/// so `&str`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ACCENT_ENV = "MUSIC_PLAYER_ACCENT";
/// ```
pub const ACCENT_ENV: &str = "MUSIC_PLAYER_ACCENT";

/// What:
///      `pub enum AccentError { Malformed(String) }`.
/// Why:
///       A typo must be reported rather than silently treated as the system accent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AccentError extends Error { constructor(readonly value: string) { super(`malformed accent ${value}`); } }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccentError {
    /// The value that was given.
    Malformed(
        /// The variable's text,
        /// as it was given.
        String,
    ),
}

/// Text of an `AccentError` for the log and for the startup message.
impl fmt::Display for AccentError {
    /// What:
    /// Writes the sentence that names the rejected value and the accepted forms.
    /// Why:
    /// The startup message and the log show this text.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Self::Malformed(value) => formatter.write_str(&format!(
                "MUSIC_PLAYER_ACCENT is \"{value}\", which is not a color. Use six hex digits such as 6060C0 or #6060C0, \
                 or leave the variable unset to follow the system accent."
            )),
        };
    }
}

/// Lets `AccentError` travel as a standard error value.
impl std::error::Error for AccentError {}

/// What:
///      `fn hex_channel(digits: &str, text: &str, start: usize) -> Result<u8, AccentError>`.
///  Reads the two hex digits that begin at byte `start` of `digits` as one color channel.
/// Why:
///       The accent has three channels read the same way,
///  and a rejected channel names the whole text that was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hexChannel = (digits: string, text: string, start: number): number => { ...throws Malformed(text) when not hex... };
/// ```
fn hex_channel(digits: &str, text: &str, start: usize) -> Result<u8, AccentError> {
    return match u8::from_str_radix(&digits[start..start + 2], 16) {
        Ok(channel) => Ok(channel),
        Err(error) => {
            tracing::warn!(error = %error, value = text, "the accent override has an unreadable channel");
            Err(AccentError::Malformed(text.to_string()))
        }
    };
}

/// What:
///      `pub fn accent_override(value: Option<&str>) -> Result<Option<Srgb8>, AccentError>`:
/// `None` or an empty
///  text follows the system,
/// six hex digits (optionally after `#`) give that color.
/// Why:
///       The caller sets `Theme.accent-override` from the result;
/// a malformed value is an error naming the text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function accentOverride(value?: string): Srgb8 | undefined {
///   if (value === undefined || value === "") return undefined;
///   const digits = value.startsWith("#") ? value.slice(1) : value;
///   if (!/^[0-9a-fA-F]{6}$/.test(digits)) throw new AccentError(value);
///   return { red: parseInt(digits.slice(0, 2), 16), green: parseInt(digits.slice(2, 4), 16), blue: parseInt(digits.slice(4, 6), 16) };
/// }
/// ```
pub fn accent_override(value: Option<&str>) -> Result<Option<Srgb8>, AccentError> {
    let text: &str = match value {
        None => return Ok(None),
        Some("") => return Ok(None),
        Some(other) => other,
    };
    let digits: &str = text.strip_prefix('#').unwrap_or(text);
    let mut only_hex_digits: bool = true;
    for digit in digits.chars() {
        if !digit.is_ascii_hexdigit() {
            only_hex_digits = false;
        }
    }
    if digits.len() != 6 || !only_hex_digits {
        tracing::warn!(value = text, "the accent override is not six hex digits");
        return Err(AccentError::Malformed(text.to_string()));
    }
    return Ok(Some(Srgb8 {
        red: hex_channel(digits, text, 0)?,
        green: hex_channel(digits, text, 2)?,
        blue: hex_channel(digits, text, 4)?,
    }));
}

/// Unit tests of `accent_override`,
/// in the sidecar file `accent_override_tests.rs`.
/// Role of the module:
/// the accent override that the environment variable `MUSIC_PLAYER_ACCENT` selects.
#[cfg(test)]
#[path = "accent_override_tests.rs"]
mod tests;
