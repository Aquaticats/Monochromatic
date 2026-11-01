//! The color scheme override that the environment variable `MUSIC_PLAYER_COLOR_SCHEME` selects.
//!
//! What: `scheme_override` maps the variable's text to the `Theme.scheme-override` value the window reads.
//! Why: verification runs need a fixed light or dark window, and the default follows the system.

use std::fmt;

/// What:
///      `pub const COLOR_SCHEME_ENV: &str = "MUSIC_PLAYER_COLOR_SCHEME";`.
///  The environment variable that selects the color scheme.
/// Why:
///       One name is shared by the startup code and the README.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COLOR_SCHEME_ENV = "MUSIC_PLAYER_COLOR_SCHEME";
/// ```
pub const COLOR_SCHEME_ENV: &str = "MUSIC_PLAYER_COLOR_SCHEME";

/// What:
///      `pub const FOLLOW_SYSTEM: i32 = 0;` and its siblings `LIGHT` and `DARK`.
///  The three values `Theme.scheme-override` takes.
/// Why:
///       The Slint global reads these integers, so the Rust side names them once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FOLLOW_SYSTEM = 0, LIGHT = 1, DARK = 2;
/// ```
pub const FOLLOW_SYSTEM: i32 = 0;
/// Forces the light scheme.
pub const LIGHT: i32 = 1;
/// Forces the dark scheme.
pub const DARK: i32 = 2;

/// What:
///      `pub enum ColorSchemeError { Unknown(String) }`.
///  The variable's text is neither `light`, `dark`, nor empty.
/// Why:
///       A typo must be reported rather than silently treated as the system scheme.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ColorSchemeError extends Error { constructor(readonly value: string) { super(`unknown color scheme ${value}`); } }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorSchemeError {
    /// The value that was given.
    Unknown(String),
}

impl fmt::Display for ColorSchemeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Self::Unknown(value) => formatter.write_str(&format!(
                "{COLOR_SCHEME_ENV} must be light or dark, or unset to follow the system; it was {value:?}."
            )),
        };
    }
}

impl std::error::Error for ColorSchemeError {}

/// What:
///      `pub fn scheme_override(value: Option<&str>) -> Result<i32, ColorSchemeError>`.
///  The override for an absent, empty, `light`, or `dark` value; an error for anything else.
/// Why:
///       Unset means follow the system, and an empty variable (`MUSIC_PLAYER_COLOR_SCHEME=`) counts as unset, as the
///  accent variable already does; each named scheme maps to one fixed value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function schemeOverride(value: string | undefined): number {
///   if (value === undefined || value.trim() === "") return FOLLOW_SYSTEM;
///   if (value === "light") return LIGHT;
///   if (value === "dark") return DARK;
///   throw new ColorSchemeError(value);
/// }
/// ```
pub fn scheme_override(value: Option<&str>) -> Result<i32, ColorSchemeError> {
    return match value {
        None => Ok(FOLLOW_SYSTEM),
        Some(blank) if blank.trim().is_empty() => {
            tracing::debug!("the color scheme variable is empty; following the system");
            Ok(FOLLOW_SYSTEM)
        }
        Some("light") => Ok(LIGHT),
        Some("dark") => Ok(DARK),
        Some(other) => Err(ColorSchemeError::Unknown(other.to_string())),
    };
}

#[cfg(test)]
#[path = "color_scheme_tests.rs"]
mod tests;
