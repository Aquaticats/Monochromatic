//! What:
//! `mpm-shadow list` (alias `ls`):
//! the status report of what the lock names against what is installed,
//! and the exit code that says whether anything drifted.
//! Why:
//! pnpm's `list` is the verb,
//! and the pnpm page leaves exit codes open,
//! so `list` keeps exiting 1 on drift,
//! which is what scripts reading it depend on.
//! It never resolves and never writes,
//! and the old copies it names are not drift.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function listVerb(scope: Scope, env: EnvMap): number
//! ```

/// Imports `Host` from `super::super::context`.
use super::super::context::Host;
/// Imports `ProcessRunner` from `super::super::runner`.
use super::super::runner::ProcessRunner;
/// Imports what this module uses from `super::super::scope`.
use super::super::scope::{
    EnvMap,
    Scope,
};
/// Imports `run_status` from `super::super::status_cmd`.
use super::super::status_cmd::run_status;
/// Imports what this module uses from `super::what_every_verb_prints`.
use super::what_every_verb_prints::report;

/// What:
/// Runs `list` in `scope` and returns the exit code:
/// 1 when anything drifted.
/// Why:
/// The report is printed as the observer wrote it,
/// so its wording stays the one every command shares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function listVerb(scope: Scope, env: EnvMap): number
/// ```
pub(super) fn list_verb(
    scope: &Scope,
    env: &EnvMap,
) -> i32 {
    return match run_status(scope, env, &ProcessRunner, &Host::current()) {
        Ok(status_report) => {
            print!("{}", status_report.text);
            i32::from(status_report.drifted)
        }
        Err(error) => report(&error),
    };
}
