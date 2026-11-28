//! What:
//! `mpm-shadow update` and `mpm-shadow outdated`:
//! the lock is resolved and written for every tool or for the ones named,
//! or the same report is printed with nothing written.
//! Why:
//! `pnpm outdated` reports and exits 1 when anything would move,
//! and `pnpm update` writes,
//! so both print one report and the exit code says which question was asked.
//! A lookups failure is a failure in either verb,
//! never a report line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function updateAndOutdatedVerbs(name: string, matches: ParsedArgs, scope: Scope, env: EnvMap): number
//! ```

/// Imports the `clap` type for reading a parsed command line.
use clap::ArgMatches;

/// Imports `Host` from `super::super::context`.
use super::super::context::Host;
/// Imports `connect_or_explain` from `super::super::http`.
use super::super::http::connect_or_explain;
/// Imports `ProcessRunner` from `super::super::runner`.
use super::super::runner::ProcessRunner;
/// Imports what this module uses from `super::super::scope`.
use super::super::scope::{
    EnvMap,
    Scope,
};
/// Imports what this module uses from `super::super::update`.
use super::super::update::{
    format_update_report,
    run_update_for,
};
/// Imports what this module uses from `super::what_every_verb_prints`.
use super::what_every_verb_prints::{
    report,
    warn_older_locks,
};

/// What:
/// Runs `update` or `outdated` (whichever `name` is) with the arguments `matches` and returns the exit code.
/// Why:
/// Only these two verbs differ in whether anything is written,
/// so one function takes the name and asks the one question that depends on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function updateAndOutdatedVerbs(name: string, matches: ParsedArgs, scope: Scope, env: EnvMap): number
/// ```
pub(super) fn update_verbs(
    name: &str,
    matches: &ArgMatches,
    scope: &Scope,
    env: &EnvMap,
) -> i32 {
    // pnpm outdated reports and exits 1 when anything is outdated; pnpm update writes.
    let check: bool = name == "outdated";
    let names: Vec<String> = match matches.get_many::<String>("tool") {
        Some(given) => given.cloned().collect(),
        None => Vec::new(),
    };
    let http = connect_or_explain();
    return match run_update_for(
        scope,
        env,
        &ProcessRunner,
        http.as_ref(),
        &Host::current(),
        check,
        &names,
    ) {
        Ok(update_report) => {
            crate::notices::report(&format_update_report(&update_report, check));
            warn_older_locks(name, scope);
            i32::from(update_report.has_failures() || (check && update_report.has_movement()))
        }
        Err(error) => report(&error),
    };
}
