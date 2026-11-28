//! What:
//! `mpm-shadow prune --toolchain` as a person runs it:
//! the scope the working directory (or `--global`) selects,
//! the locks named with `--lock`,
//! and one line per path the run deleted or would delete.
//! Why:
//! `cli.rs` hands the parsed arguments here,
//! so the command's wiring sits beside the rule it runs and prints what the rule decided.
//! A caller imports from `prune_cargo` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function runPruneCommand(matches: ParsedArgs): number;
//! ```

/// Imports the directory type an extra lock is held in.
use std::path::PathBuf;

/// Imports `ArgMatches` from `clap`.
use clap::ArgMatches;

/// Imports the runner the cleanup starts git through.
use super::super::runner::ProcessRunner;
/// Imports what this file uses from `super::super::scope`.
use super::super::scope::{
    EnvMap,
    Scope,
    resolve_scope,
};

/// Imports the cleanup this command runs.
use super::the_prune_itself::prune_toolchain;
/// What:
/// `mpm-shadow prune --toolchain [--global] [--dry-run] [--lock <file>]...`:
/// runs `prune_toolchain` for the scope the working directory (or `--global`) selects and prints the result.
/// Why:
/// `cli.rs` hands the parsed arguments here,
/// so the command's wiring sits beside the rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runPruneCommand(matches: ParsedArgs): number
/// ```
pub fn run_prune_command(matches: &ArgMatches) -> i32 {
    let env: EnvMap = EnvMap::from_process();
    let start: PathBuf = match std::env::current_dir() {
        Ok(directory) => directory,
        Err(_) => PathBuf::from("."),
    };
    let scope: Scope = match resolve_scope(matches.get_flag("global"), &start, &env) {
        Ok(found) => found,
        Err(error) => {
            crate::notices::say(&format!("mpm-shadow: {error}"));
            return error.exit_code();
        }
    };
    let mut extras: Vec<PathBuf> = Vec::new();
    if let Some(values) = matches.get_many::<String>("lock") {
        for value in values {
            extras.push(PathBuf::from(value));
        }
    }
    let dry_run: bool = matches.get_flag("dry-run");
    return match prune_toolchain(&scope, &ProcessRunner, &extras, dry_run) {
        Ok(removed) => {
            let verb: &str = if dry_run { "would delete" } else { "deleted" };
            for path in &removed {
                crate::notices::report_line(&format!("{verb} {}", path.display()));
            }
            if removed.is_empty() {
                crate::notices::report_line(&format!(
                    "nothing to delete under {}: every cargo version there is named by a known lock, \
                     and no release archive of this repository is older than its newest finished version",
                    scope.toolchain_root.display()
                ));
            }
            0
        }
        Err(error) => {
            crate::notices::say(&format!("mpm-shadow: {error}"));
            error.exit_code()
        }
    };
}
