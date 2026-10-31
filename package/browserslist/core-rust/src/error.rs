//! What:
//!  The one error type every operation of this crate returns.
//! Why:
//!  Callers match on a single enum instead of juggling one error type per module,
//!  and the "not implemented" family stays distinguishable from data and network failures.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class BrowserslistError extends Error { kind: 'not-implemented' | 'browserslist' | 'data' | 'lock' | 'network' | 'io' }
//! ```

use std::fmt;

/// What:
///  Every failure this crate reports.
///  An `enum` is a type that is exactly one of several named variants,
///  like a TypeScript discriminated union.
/// Why:
///  The variants carry only the facts a diagnostic needs,
///  and `Display` turns each into one message that names the input at fault.
/// Gotcha:
///  `NotImplemented` is a deliberate refusal required by the user's decision,
///  not a bug report: its message always starts with `not implemented`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BrowserslistError =
///   | { kind: 'not-implemented'; subject: string }
///   | { kind: 'browserslist'; message: string }
///   | { kind: 'data'; file: string; offset?: number; message: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserslistError {
    /// A query form or config feature outside the supported subset;
    /// `subject` names it (and the line when there is one).
    NotImplemented { subject: String },
    /// An error npm browserslist itself raises for a supported form,
    /// worded exactly as browserslist words it.
    Browserslist { message: String },
    /// A data file (or the lock) that does not parse;
    /// `offset` is the byte offset in `file` when the failure has one.
    Data { file: String, offset: Option<usize>, message: String },
    /// A pinned download whose digest does not match the lock.
    Integrity { subject: String, expected: String, actual: String },
    /// An HTTP request that failed or answered with something unusable.
    Network { url: String, message: String },
    /// A file system operation that failed.
    Io { path: String, message: String },
}

/// What:
///  Build the common "data file is malformed at a byte offset" error.
/// Why:
///  Parsers report where they gave up with one call instead of spelling out the struct each time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dataError(file: string, offset: number, message: string): BrowserslistError;
/// ```
pub fn data_error(file: &str, offset: usize, message: &str) -> BrowserslistError {
    return BrowserslistError::Data {
        file: file.to_string(),
        offset: Some(offset),
        message: message.to_string(),
    };
}

/// What:
///  Build a "not implemented" error for a named subject.
/// Why:
///  Every refusal in the crate goes through one constructor,
///  so the message prefix the user's decision requires cannot drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notImplemented(subject: string): BrowserslistError;
/// ```
pub fn not_implemented(subject: &str) -> BrowserslistError {
    return BrowserslistError::NotImplemented { subject: subject.to_string() };
}

/// What:
///  `impl fmt::Display` teaches Rust how to print the error as text,
///  the way a TypeScript `Error` has a `message`.
/// Why:
///  CLI output and test assertions read these messages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// get message(): string { /* switch on kind */ }
/// ```
impl fmt::Display for BrowserslistError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented { subject } => {
                return write!(formatter, "not implemented: {subject}");
            }
            Self::Browserslist { message } => {
                return write!(formatter, "{message}");
            }
            Self::Data { file, offset: Some(offset), message } => {
                return write!(formatter, "{file}: {message} (byte offset {offset})");
            }
            Self::Data { file, offset: None, message } => {
                return write!(formatter, "{file}: {message}");
            }
            Self::Integrity { subject, expected, actual } => {
                return write!(
                    formatter,
                    "{subject}: downloaded bytes hash to {actual}, but the lock pins {expected}"
                );
            }
            Self::Network { url, message } => {
                return write!(formatter, "request to {url} failed: {message}");
            }
            Self::Io { path, message } => {
                return write!(formatter, "{path}: {message}");
            }
        }
    }
}

/// What:
///  Marks the type as a standard Rust error so `?` and `Box<dyn Error>` accept it.
/// Why:
///  The update binary and the engine propagate it with `?`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BrowserslistError extends Error {}
/// ```
impl std::error::Error for BrowserslistError {}

/// What:
///  Shorthand for `Result<T, BrowserslistError>`.
/// Why:
///  Every function in the crate returns this, so the signature stays short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome<T> = T; // failures are thrown as BrowserslistError
/// ```
pub type Outcome<T> = Result<T, BrowserslistError>;

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
