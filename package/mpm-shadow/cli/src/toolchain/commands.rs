//! What:     The command-line surface of the toolchain half: the `clap` description of `lock`, `bin-paths` (and the
//!           verbs added with them), and the function that runs one of them and turns its result into an exit code.
//! Why:      `cli.rs` keeps the older `prune` command and delegates every toolchain verb here, so the toolchain code
//!           and its grammar live together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commands.ts
//! export function subcommands(): Command[]
//! export function dispatch(name: string, matches: ParsedArgs): number | undefined
//! ```

/// Imports the `clap` types for describing arguments and reading what was typed.
use clap::{Arg, ArgAction, ArgMatches, Command};

use std::path::PathBuf;

use super::bin_paths::{bin_paths_output, render_bin_paths};
use super::error::ToolchainError;
use super::context::Host;
use super::http::connect_or_explain;
use super::lock_cmd::{LockMode, run_lock};
use super::plan::read_plan;
use super::runner::ProcessRunner;
use super::scope::{EnvMap, Scope, resolve_scope};
use super::status_cmd::run_status;
use super::sync::{format_sync_report, run_sync};
use super::update::{format_update_report, run_update};

/// The `--global` flag every toolchain verb accepts: act on the machine-global plan and lock.
fn global_flag() -> Arg {
    return Arg::new("global")
        .long("global")
        .action(ArgAction::SetTrue)
        .help("Act on the machine-global toolchain (the plan and lock under the per-user config directory)");
}

/// What:     The toolchain subcommands for `mpm-shadow --help`.
/// Why:      Each verb is a plain `clap` command; the grammar is data the tests can inspect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function subcommands(): Command[]
/// ```
pub fn subcommands() -> Vec<Command> {
    let lock: Command = Command::new("lock")
        .about("Rewrite toolchain.lock.json from toolchain.plan.jsonc without the network, keeping every resolved version")
        .arg(
            Arg::new("check")
                .long("check")
                .action(ArgAction::SetTrue)
                .help("Write nothing; exit 1 when the plan and the lock disagree or the lock's digest does not match"),
        )
        .arg(
            Arg::new("adopt")
                .long("adopt")
                .action(ArgAction::SetTrue)
                .conflicts_with("check")
                .help("Accept a lock that has no digest yet (the first conversion of an older lock) and add the digest"),
        )
        .arg(global_flag());
    let bin_paths: Command = Command::new("bin-paths")
        .about("Print the toolchain's program directories and exposed binaries as JSON (read-only, offline)")
        .arg(global_flag());
    let sync: Command = Command::new("sync")
        .about("Install every locked tool that is not at its locked version, then rebuild the exposure directory")
        .arg(global_flag());
    let update: Command = Command::new("update")
        .about("Resolve every tool of the plan and write toolchain.lock.json (looks versions up on the network)")
        .arg(
            Arg::new("check")
                .long("check")
                .action(ArgAction::SetTrue)
                .help("Report what would move without writing the lock or installing anything; exit 1 when something would move"),
        )
        .arg(global_flag());
    let status: Command = Command::new("status")
        .about("Compare the installed tools with the lock (read-only; exits 1 on drift or when the plan and lock disagree)")
        .arg(global_flag());
    return vec![lock, update, sync, status, bin_paths];
}

/// Prints a success line to standard output.
fn print_line(line: &str) {
    println!("{line}");
}

/// Prints an error to standard error and returns its exit code.
fn report(error: &ToolchainError) -> i32 {
    eprintln!("mpm-shadow: {error}");
    return error.exit_code();
}

/// The directory the command was started in (the root search starts here).
fn start_directory() -> PathBuf {
    return std::env::current_dir().unwrap_or_else(|_| return PathBuf::from("."));
}

/// What:     Runs the toolchain verb `name` with the arguments `matches`, or returns `None` when `name` is not one of
///           this module's verbs.
/// Why:      `cli.rs` asks this first and falls back to its own `prune`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function dispatch(name: string, matches: ParsedArgs): number | undefined
/// ```
pub fn dispatch(name: &str, matches: &ArgMatches) -> Option<i32> {
    let env: EnvMap = EnvMap::from_process();
    let global: bool = matches.get_flag("global");
    let scope: Scope = match resolve_scope(global, &start_directory(), &env) {
        Ok(found) => found,
        Err(error) => return Some(report(&error)),
    };
    match name {
        "lock" => {
            let mode: LockMode = if matches.get_flag("check") {
                LockMode::Check
            } else if matches.get_flag("adopt") {
                LockMode::Adopt
            } else {
                LockMode::Write
            };
            return Some(match run_lock(&scope, mode) {
                Ok(line) => {
                    print_line(&line);
                    0
                }
                Err(error) => report(&error),
            });
        }
        "sync" => {
            let http = connect_or_explain();
            return Some(match run_sync(&scope, &env, &ProcessRunner, http.as_ref(), &Host::current()) {
                Ok(sync_report) => {
                    println!(
                        "toolchain sync: platform {}, root {}, exposure {}",
                        Host::current().platform,
                        scope.toolchain_root.display(),
                        scope.exposure_dir.display()
                    );
                    print!("{}", format_sync_report(&sync_report));
                    i32::from(!sync_report.failed.is_empty())
                }
                Err(error) => report(&error),
            });
        }
        "update" => {
            let check: bool = matches.get_flag("check");
            let http = connect_or_explain();
            return Some(match run_update(&scope, &env, &ProcessRunner, http.as_ref(), &Host::current(), check) {
                Ok(update_report) => {
                    print!("{}", format_update_report(&update_report, check));
                    i32::from(update_report.has_failures() || (check && update_report.has_movement()))
                }
                Err(error) => report(&error),
            });
        }
        "status" => {
            return Some(match run_status(&scope, &env, &ProcessRunner, &Host::current()) {
                Ok(status_report) => {
                    print!("{}", status_report.text);
                    i32::from(status_report.drifted)
                }
                Err(error) => report(&error),
            });
        }
        "bin-paths" => {
            // Only the plan is read: the IDE sandbox may run this before any lock exists.
            let outcome = read_plan(&scope.plan_file).and_then(|plan| return bin_paths_output(&plan, &scope, &env));
            return Some(match outcome {
                Ok(document) => {
                    print!("{}", render_bin_paths(&document));
                    0
                }
                Err(error) => report(&error),
            });
        }
        _ => return None,
    }
}
