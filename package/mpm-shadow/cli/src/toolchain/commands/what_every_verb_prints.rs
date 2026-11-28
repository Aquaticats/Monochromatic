//! What:
//! The words every toolchain verb prints:
//! a success line,
//! the result of a lock-only run,
//! an error and its exit code,
//! and the warning lines a command prints on standard error when this tree's lock pins an older
//! repository-published tool than the one installed.
//! Why:
//! Every verb prints its failures through one function,
//! so an error looks the same whichever verb met it,
//! and the older-lock warning is `toolchain <verb>: warning: <text>`,
//! the format `install` already uses,
//! because `install --lockfile-only`,
//! `update` and `bin` have no report of their own to carry it
//! and `bin` prints JSON on standard output.
//! A missing or unreadable lock gives no warning:
//! those commands report that themselves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function printLine(line: string): void
//! export function printLockOutcome(verb: string, scope: Scope, outcome: string | Error): number
//! export function report(error: ToolchainError): number
//! export function olderLockWarningLines(verb: string, scope: Scope, family?: OsFamily): string[]
//! function warnOlderLocks(verb: string, scope: Scope): void
//! function startDirectory(): string
//! ```

/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `older_lock_warnings` from `super::super::cargo_installs`.
use super::super::cargo_installs::older_lock_warnings;
/// Imports `Host` from `super::super::context`.
use super::super::context::Host;
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports `parse_lock` from `super::super::lock`.
use super::super::lock::parse_lock;
/// Imports `OsFamily` from `super::super::model`.
use super::super::model::OsFamily;
/// Imports `Scope` from `super::super::scope`.
use super::super::scope::Scope;

/// Prints a success line to standard output.
pub(super) fn print_line(line: &str) {
    crate::notices::report_line(line);
}

/// Prints the result of a `run_lock` call the way every lock-only mode does and returns the exit code.
pub(super) fn print_lock_outcome(
    verb: &str,
    scope: &Scope,
    outcome: Result<String, ToolchainError>,
) -> i32 {
    return match outcome {
        Ok(line) => {
            print_line(&line);
            warn_older_locks(verb, scope);
            0
        }
        Err(error) => report(&error),
    };
}

/// Prints an error to standard error and returns its exit code.
pub(super) fn report(error: &ToolchainError) -> i32 {
    crate::notices::say(&format!("mpm-shadow: {error}"));
    return error.exit_code();
}

/// What:
/// The warning lines a command prints on standard error when this tree's lock pins an older repository-published tool
/// than the one installed:
/// `toolchain <verb>: warning: <text>`,
/// the format `install` already uses.
/// Why:
/// `install --lockfile-only`,
/// `update` and `bin` have no report of their own to carry them,
/// and `bin` prints JSON on standard output.
/// A missing or unreadable lock gives none:
/// those commands report that themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function olderLockWarningLines(verb: string, scope: Scope, family?: OsFamily): string[]
/// ```
pub fn older_lock_warning_lines(
    verb: &str,
    scope: &Scope,
    family: Option<OsFamily>,
) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(&scope.lock_file) else {
        return Vec::new();
    };
    let Ok(parsed) = parse_lock(&text, &scope.lock_file.display().to_string()) else {
        return Vec::new();
    };
    let mut lines: Vec<String> = Vec::new();
    for warning in older_lock_warnings(scope, &parsed.lock.tools, family) {
        lines.push(format!("toolchain {verb}: warning: {warning}"));
    }
    return lines;
}

/// Prints `older_lock_warning_lines` on standard error.
/// The lines warn about a lock this run reads on its way to a result it still prints,
/// so they go through the crate's notice writer:
/// a stream that refuses a warning must not end a command that was going to succeed.
pub(super) fn warn_older_locks(
    verb: &str,
    scope: &Scope,
) {
    for line in older_lock_warning_lines(verb, scope, Host::current().family) {
        crate::notices::say(&line);
    }
}

/// The directory the command was started in (the root search starts here).
pub(super) fn start_directory() -> PathBuf {
    return match std::env::current_dir() {
        Ok(directory) => directory,
        Err(_) => PathBuf::from("."),
    };
}
