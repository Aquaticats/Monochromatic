//! What:
//! Methods that turn a failure into error text,
//! building the text only when the failure happens,
//! and taking it as a `format_args!` value instead of a closure.
//! Why:
//! `map_err` and `ok_or_else` took a closure so that a success pays nothing for the message.
//! `format_args!` holds references to its arguments and formats nothing until a function turns it into a `String`,
//! so these methods keep that property without a closure.
//! Gotcha:
//! `format_args!` evaluates the expressions of its arguments at the call,
//! so an argument must be a cheap borrow or a `Display` wrapper (such as `path.display()`),
//! never a computation that reads a file or allocates.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // result.mapErr(() => message()) written as a call that takes the message parts, not a callback
//! ```

/// The formatting types the methods take and the bound on the failure.
use std::fmt::{Arguments, Display};

/// What:
/// `ExplainedError` adds `explained` and `explained_with_tail` to a `Result` whose failure can be printed.
/// Why:
/// Most failure texts read `SUBJECT (FAILURE)`,
/// optionally followed by advice,
/// and the subject is the only part that differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Result<T, E> { explained(subject: string): Result<T, string>; explainedWithTail(subject: string, tail: string): Result<T, string>; }
/// ```
pub trait ExplainedError<T> {
    /// What:
    /// `explained(subject)` keeps a success and turns a failure into `"{subject} ({failure})"`.
    /// Why:
    /// The subject names the step that failed,
    /// the parenthesis carries the cause.
    fn explained(self, subject: Arguments<'_>) -> Result<T, String>;

    /// What:
    /// `explained_with_tail(subject, tail)` keeps a success and turns a failure into `"{subject} ({failure}){tail}"`.
    /// Why:
    /// Some failures end with advice that follows the cause.
    fn explained_with_tail(self, subject: Arguments<'_>, tail: &str) -> Result<T, String>;

    /// What:
    /// `described(subject)` keeps a success and turns a failure into `"{subject}: {failure}"`.
    /// Why:
    /// Some failure texts put the cause after a colon instead of in a parenthesis.
    fn described(self, subject: Arguments<'_>) -> Result<T, String>;
}

/// What:
/// The implementation for every `Result` whose failure implements `Display`.
/// Why:
/// The text is built on the `Err` arm only.
impl<T, E: Display> ExplainedError<T> for Result<T, E> {
    /// What:
    /// Builds `"{subject} ({failure})"` on the `Err` arm.
    /// Why:
    /// The common shape of a step failure.
    fn explained(self, subject: Arguments<'_>) -> Result<T, String> {
        match self {
            Ok(value) => return Ok(value),
            Err(error) => return Err(format!("{subject} ({error})")),
        }
    }

    /// What:
    /// Builds `"{subject}: {failure}"` on the `Err` arm.
    /// Why:
    /// A failure text with the cause after a colon.
    fn described(self, subject: Arguments<'_>) -> Result<T, String> {
        match self {
            Ok(value) => return Ok(value),
            Err(error) => return Err(format!("{subject}: {error}")),
        }
    }

    /// What:
    /// Builds `"{subject} ({failure}){tail}"` on the `Err` arm.
    /// Why:
    /// A step failure with advice after the cause.
    fn explained_with_tail(self, subject: Arguments<'_>, tail: &str) -> Result<T, String> {
        match self {
            Ok(value) => return Ok(value),
            Err(error) => return Err(format!("{subject} ({error}){tail}")),
        }
    }
}

/// What:
/// `OrText` adds `or_text` to an `Option`.
/// Why:
/// An absent value is reported as text,
/// built only when the value is absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Option<T> { orText(message: string): Result<T, string>; }
/// ```
pub trait OrText<T> {
    /// What:
    /// `or_text(message)` keeps a present value and turns an absent one into `message` as an owned `String`.
    /// Why:
    /// Replaces `ok_or_else(|| format!(..))`.
    fn or_text(self, message: Arguments<'_>) -> Result<T, String>;
}

/// What:
/// The implementation for every `Option`.
/// Why:
/// The text is built on the `None` arm only.
impl<T> OrText<T> for Option<T> {
    /// What:
    /// Builds the message on the `None` arm.
    /// Why:
    /// A present value costs nothing.
    fn or_text(self, message: Arguments<'_>) -> Result<T, String> {
        match self {
            Some(value) => return Ok(value),
            None => return Err(message.to_string()),
        }
    }
}
