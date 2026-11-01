//! What:
//! The one error type every toolchain command returns,
//! and the exit code each kind maps to.
//! Why:
//! Commands report three different things (a bad plan or command line,
//! a plan and lock that disagree,
//! and
//!           a failure while doing the work),
//! and `main` needs one exit code per kind without parsing messages.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // error.ts
//! export class ToolchainError extends Error { constructor(readonly kind: 'config' | 'drift' | 'failure', message: string) { super(message); } }
//! ```

/// Imports the text-formatting trait so the error can print itself.
use std::fmt;

/// Imports `USAGE_EXIT_CODE` from `crate::constants`.
use crate::constants::USAGE_EXIT_CODE;

/// What:
/// `pub enum ErrorKind` declares a closed set of three named cases.
/// Rust enums are tagged unions like a TS
///           string-literal union (siblings:
/// a struct holds all its fields at once,
/// an enum holds one case).
/// Why:
/// The kind decides the exit code and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ErrorKind = 'config' | 'drift' | 'failure';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// The command line,
    /// the plan file or the lock file cannot be read as what it should be (exit code 2).
    Config,
    /// The plan and the lock disagree,
    /// or the lock was edited by hand,
    /// or installed tools drifted (exit code 1).
    Drift,
    /// An install,
    /// a download or a lookup failed (exit code 1).
    Failure,
}

/// One failure with its kind and a message that names the affected input and what to do next.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolchainError {
    /// Which exit code this error maps to.
    pub kind: ErrorKind,
    /// The text shown to the person,
    /// already naming the file,
    /// tool or URL involved.
    pub message: String,
}

/// The methods of `ToolchainError`.
impl ToolchainError {
    /// What:
    /// `pub fn config(message: impl Into<String>) -> ToolchainError` builds a `Config` error.
    /// `impl
    ///           Into<String>` accepts anything convertible to an owned `String` (a `&str` literal or a `String`).
    /// Why:
    /// Call sites pass literals and formatted text alike.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const config = (message: string) => new ToolchainError('config', message);
    /// ```
    pub fn config(message: impl Into<String>) -> ToolchainError {
        return ToolchainError { kind: ErrorKind::Config, message: message.into() };
    }

    /// Builds a `Drift` error (plan and lock disagree,
    /// or installed tools drifted).
    pub fn drift(message: impl Into<String>) -> ToolchainError {
        return ToolchainError { kind: ErrorKind::Drift, message: message.into() };
    }

    /// Builds a `Failure` error (an install,
    /// download or lookup failed).
    pub fn failure(message: impl Into<String>) -> ToolchainError {
        return ToolchainError { kind: ErrorKind::Failure, message: message.into() };
    }

    /// What:
    /// The process exit code for this error:
    /// 2 for `Config` (the same code a bad command line uses),
    /// 1 for
    ///           the other two.
    /// Why:
    /// `main` maps errors to exit codes in one place.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// exitCode(): number { return this.kind === 'config' ? 2 : 1; }
    /// ```
    pub fn exit_code(&self) -> i32 {
        if self.kind == ErrorKind::Config {
            return USAGE_EXIT_CODE;
        }
        return 1;
    }
}

/// Implements `fmt::Display` for `ToolchainError`.
impl fmt::Display for ToolchainError {
    /// What:
    /// Prints the message alone,
    /// so `eprintln!("mpm-shadow: {error}")` reads as one sentence.
    /// Why:
    /// The kind is for the exit code,
    /// not for the reader.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return this.message; }
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}", self.message);
    }
}

/// What:
/// `impl std::error::Error for ToolchainError {}` marks the type as a standard error value.
/// Why:
/// Lets callers box it or use `?` in functions returning `Box<dyn Error>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // class ToolchainError extends Error
/// ```
impl std::error::Error for ToolchainError {}
