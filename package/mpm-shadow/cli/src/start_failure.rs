//! What:
//! The message for a real `mpm` that could not be started.
//! Why:
//! The person sees only this text: it must name the command they ran, the program tried, the stage, the cause in
//! plain words, the remedy, the exit status the shadow returns and what had completed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function describeStartFailure(failure: StartFailure): string
//! ```

/// Imports the owned operating-system text type.
use std::ffi::OsString;
/// Imports the error of a failed start.
use std::io::Error;
/// Imports the borrowed path type.
use std::path::Path;

use crate::start_retry::RetryReport;

/// Everything the message needs to know.
pub struct StartFailure<'a> {
    /// The arguments `mpm` was run with (after the program name).
    pub outer: &'a [OsString],
    /// The program the shadow tried to start as the real `mpm`.
    pub program: &'a Path,
    /// Why the start failed.
    pub error: &'a Error,
    /// What the retrying did before giving up.
    pub retry: RetryReport,
    /// How many shadows sit above this one.
    pub depth: u32,
}

/// The message, several lines, starting with `mpm-shadow:`.
pub fn describe_start_failure(_failure: &StartFailure) -> String {
    return String::new();
}
