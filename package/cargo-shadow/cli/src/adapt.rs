//! What:
//! Three tiny named conversions that error mappers and time arithmetic reuse across the crate.
//! Why:
//! A named function is passed where a closure that only calls one method used to sit (`.map_err(text_of)`),
//! which keeps the call
//! sites short and satisfies the repository linter's `rust/avoidable-closure` rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // adapt.ts
//! export const textOf = (error: { toString(): string }) => error.toString();
//! ```

/// What:
/// `use std::fmt::Display;` imports `Display` from the standard library so this file can name it unqualified.
/// Why:
/// `Display` is used by `text_of`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Display } from 'std/fmt';
/// ```
use std::fmt::Display;
/// What:
/// `use std::time::Duration;` imports `Duration` from the standard library so this file can name it unqualified.
/// Why:
/// `Duration` is used by `millis_of`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Duration } from 'std/time';
/// ```
use std::time::Duration;

/// What:
/// `use crate::http::RegistryError;` imports `RegistryError` from another module of this crate so this file can name it unqualified.
/// Why:
/// `RegistryError` is used by `message_of`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RegistryError } from 'crate/http';
/// ```
use crate::http::RegistryError;

/// What:
/// `pub fn text_of(error: impl Display) -> String` prints any displayable error into an owned `String`
/// (`impl Display` accepts any type that can be printed with `{}`).
/// Why:
/// Many functions in this crate return `Result<_, String>`,
/// and the errors they receive (I/O,
/// TOML,
/// JSON) all print with `{}`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const textOf = (error: unknown): string => String(error);
/// ```
pub fn text_of(error: impl Display) -> String {
    return error.to_string();
}

/// What:
/// `pub fn message_of(error: RegistryError) -> String` takes the message out of a registry error,
/// consuming the error.
/// Why:
/// The set-aside and update code report registry failures as plain text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const messageOf = (error: RegistryError): string => error.message;
/// ```
pub fn message_of(error: RegistryError) -> String {
    return error.message;
}

/// What:
/// `pub fn millis_of(elapsed: Duration) -> i64` turns a `Duration` into whole milliseconds as `i64` (`as i64` converts the
/// wider `u128`;
/// siblings:
/// `u64`,
/// `f64`).
/// Why:
/// Timestamps in this crate are milliseconds since 1970 as `i64`,
/// the type the release-age arithmetic uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const millisOf = (elapsedMs: number): number => elapsedMs;
/// ```
pub fn millis_of(elapsed: Duration) -> i64 {
    return elapsed.as_millis() as i64;
}

/// What:
/// `pub fn position_of_text(texts: &[String], wanted: &str) -> Option<usize>` returns the index of the first text equal to
/// `wanted`,
/// or `None`.
/// Why:
/// The dump reader looks up each column name in the header row of a table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const positionOfText = (texts: string[], wanted: string) => { const at = texts.indexOf(wanted); return at === -1 ? undefined : at; };
/// ```
pub fn position_of_text(texts: &[String], wanted: &str) -> Option<usize> {
    for (index, text) in texts.iter().enumerate() {
        if text == wanted {
            return Some(index);
        }
    }
    return None;
}

/// What:
/// `pub fn contains_text(texts: &[String], wanted: &str) -> bool` tells whether any text equals `wanted`,
/// without copying
/// `wanted` into a `String`.
/// Why:
/// Version selectors list exact versions,
/// and a locked version is compared against them as plain text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const containsText = (texts: string[], wanted: string) => texts.includes(wanted);
/// ```
pub fn contains_text(texts: &[String], wanted: &str) -> bool {
    return position_of_text(texts, wanted).is_some();
}
