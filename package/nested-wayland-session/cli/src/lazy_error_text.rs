//! Error text that is built only when a failure happens,
//! passed to a named function instead of to a closure.
//!
//! `anyhow`'s `with_context`,
//! `Option::ok_or_else` and `Result::map_err` take a closure so the text costs nothing on success.
//! `format_args!` gives the same property without a closure:
//! it holds references to its arguments and formats nothing until a function turns it into a `String`.
//! Each function here takes the `format_args!` value (or a literal prefix) and builds the text only on the failing arm.

/// The formatting types the functions take as their message arguments.
use std::fmt::{Arguments, Debug, Display};

/// What:
///      `pub(crate) fn with_message<T, E>(result: Result<T, E>, message: Arguments<'_>) -> anyhow::Result<T>`.
///  Return the success value,
///  or the failure turned into an `anyhow::Error` with `message` added as context.
///  `E: Into<anyhow::Error>` accepts any standard error type and `anyhow::Error` itself.
/// Why:
///       Replaces `.with_context(|| format!(...))`:
///  the message `String` is allocated only on the failing arm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withMessage<T>(run: () => T, message: () => string): T { try { return run(); } catch (e) { throw wrap(e, message()); } }
/// ```
pub fn with_message<T, E: Into<anyhow::Error>>(result: Result<T, E>, message: Arguments<'_>) -> anyhow::Result<T> {
    match result {
        Ok(value) => return Ok(value),
        Err(error) => return Err(error.into().context(message.to_string())),
    }
}

/// What:
///      `pub(crate) fn option_or_error<T>(option: Option<T>, message: Arguments<'_>) -> anyhow::Result<T>`.
///  Return the present value,
///  or a new `anyhow::Error` carrying `message` when it is absent.
/// Why:
///       Replaces `.ok_or_else(|| anyhow!(...))`:
///  the message `String` is allocated only when the value is absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function optionOrError<T>(value: T | null, message: () => string): T { if (value === null) throw new Error(message()); return value; }
/// ```
pub fn option_or_error<T>(option: Option<T>, message: Arguments<'_>) -> anyhow::Result<T> {
    match option {
        Some(value) => return Ok(value),
        None => return Err(anyhow::Error::msg(message.to_string())),
    }
}

/// What:
///      `pub(crate) fn option_or_text<T>(option: Option<T>, message: Arguments<'_>) -> Result<T, String>`.
///  Return the present value,
///  or `message` as an owned `String` error when it is absent.
/// Why:
///       Replaces `.ok_or_else(|| "...".to_string())` and `.ok_or_else(|| format!(...))` in the protocol parsers,
///  whose error type is a `String`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function optionOrText<T>(value: T | null, message: () => string): T { if (value === null) throw message(); return value; }
/// ```
pub fn option_or_text<T>(option: Option<T>, message: Arguments<'_>) -> Result<T, String> {
    match option {
        Some(value) => return Ok(value),
        None => return Err(message.to_string()),
    }
}

/// What:
///      `pub(crate) fn result_or_text<T, E>(result: Result<T, E>, message: Arguments<'_>) -> Result<T, String>`.
///  Return the success value,
///  or `message` as an owned `String` error that replaces the original error.
/// Why:
///       Replaces `.map_err(|_| "...".to_string())` and `.map_err(|_| format!(...))`:
///  the original error carries nothing the message needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resultOrText<T>(run: () => T, message: () => string): T { try { return run(); } catch { throw message(); } }
/// ```
pub fn result_or_text<T, E>(result: Result<T, E>, message: Arguments<'_>) -> Result<T, String> {
    match result {
        Ok(value) => return Ok(value),
        Err(_) => return Err(message.to_string()),
    }
}

/// What:
///      `pub(crate) fn failed_text<T, E: Display>(result: Result<T, E>, what: &str) -> Result<T, String>`.
///  Return the success value,
///  or a `String` error reading `"{what}: {error}"`.
/// Why:
///       Replaces `.map_err(|error| format!("{what}: {error}"))`:
///  the failure's own text follows the fixed description,
///  built only on the failing arm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failedText<T>(run: () => T, what: string): T { try { return run(); } catch (e) { throw `${what}: ${e}`; } }
/// ```
pub fn failed_text<T, E: Display>(result: Result<T, E>, what: &str) -> Result<T, String> {
    match result {
        Ok(value) => return Ok(value),
        Err(error) => return Err(format!("{what}: {error}")),
    }
}

/// What:
///      `pub(crate) fn failed_error<T, E: Display>(result: Result<T, E>, what: &str) -> anyhow::Result<T>`.
///  Return the success value,
///  or an `anyhow::Error` reading `"{what}: {error}"`.
/// Why:
///       Replaces `.map_err(|err| anyhow!("{what}: {err}"))`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failedError<T>(run: () => T, what: string): T { try { return run(); } catch (e) { throw new Error(`${what}: ${e}`); } }
/// ```
pub fn failed_error<T, E: Display>(result: Result<T, E>, what: &str) -> anyhow::Result<T> {
    match result {
        Ok(value) => return Ok(value),
        Err(error) => return Err(anyhow::anyhow!("{what}: {error}")),
    }
}

/// What:
///      `pub(crate) fn failed_debug_error<T, E: Debug>(result: Result<T, E>, what: &str) -> anyhow::Result<T>`.
///  Return the success value,
///  or an `anyhow::Error` reading `"{what}: {error:?}"` (the error's `Debug` text).
/// Why:
///       Replaces `.map_err(|err| anyhow!("{what}: {err:?}"))` for errors that only implement `Debug` usefully.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failedDebugError<T>(run: () => T, what: string): T { try { return run(); } catch (e) { throw new Error(`${what}: ${inspect(e)}`); } }
/// ```
pub fn failed_debug_error<T, E: Debug>(result: Result<T, E>, what: &str) -> anyhow::Result<T> {
    match result {
        Ok(value) => return Ok(value),
        Err(error) => return Err(anyhow::anyhow!("{what}: {error:?}")),
    }
}
