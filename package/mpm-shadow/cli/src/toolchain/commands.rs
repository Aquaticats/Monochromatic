//! What:
//! The command-line surface of the toolchain half:
//! the `clap` description of `install` (alias `i`),
//! `update`,
//! `outdated`,
//! `list` (alias `ls`),
//! `bin`
//!           and the other verbs,
//! named after the pnpm commands they mirror,
//! and the function that runs one of them and turns its result into an exit code.
//! Why:
//! `cli.rs` keeps the older `prune` command and delegates every toolchain verb here,
//! so the toolchain code
//!           and its grammar live together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commands.ts
//! export function subcommands(): Command[]
//! export function dispatch(name: string, matches: ParsedArgs): number | undefined
//! ```

/// Imports the `clap` type for reading a parsed command line and starting a command.
use clap::{ArgMatches, Command};

/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports what this module uses from `super::bin_paths`.
use super::bin_paths::{bin_paths_output, render_bin_paths};
/// Imports `older_lock_warnings` from `super::cargo_installs`.
use super::cargo_installs::older_lock_warnings;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `Host` from `super::context`.
use super::context::Host;
/// Imports `connect_or_explain` from `super::http`.
use super::http::connect_or_explain;
/// Imports what this module uses from `super::lock_cmd`.
use super::lock_cmd::{LockMode, run_lock};
/// Imports `parse_lock` from `super::lock`.
use super::lock::parse_lock;
/// Imports `OsFamily` from `super::model`.
use super::model::OsFamily;
/// Imports `read_plan` from `super::plan`.
use super::plan::read_plan;
/// Imports `ProcessRunner` from `super::runner`.
use super::runner::ProcessRunner;
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope, resolve_scope};
/// Imports `run_status` from `super::status_cmd`.
use super::status_cmd::run_status;
/// Imports what this module uses from `super::sync`.
use super::sync::{FrozenReason, LockPolicy, run_lockfile_only, run_sync_with};
/// Imports the lines `install` prints for a sync.
use super::sync_report_text::{format_sync_report, locked_line, rewritten_line};
/// Imports `is_ci` from `super::ci_detect`.
use super::ci_detect::is_ci;
/// Imports what this module uses from `super::verify_archive`.
use super::verify_archive::run_verify_archive;

/// Imports what this module uses from `super::update`.
use super::update::{format_update_report, run_update_for};

/// Re-exports the verb declarations from `super::verb_grammar`,
/// so `cli.rs` and the tests keep reading the grammar through this module.
pub use super::verb_grammar::{Verb, VERBS};

/// What:
/// The toolchain subcommands for `mpm-shadow --help`,
/// built from the `VERBS` declaration.
/// Why:
/// Each verb is a plain `clap` command;
/// the grammar is data the tests can inspect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function subcommands(): Command[]
/// ```
pub fn subcommands() -> Vec<Command> {
    let mut commands: Vec<Command> = Vec::new();
    for verb in VERBS {
        // `verb.build` is the declared argument builder; calling it through the declaration
        // keeps the parsed name and the argument grammar in one place.
        commands.push((verb.build)(verb));
    }
    return commands;
}

/// Prints a success line to standard output.
fn print_line(line: &str) {
    println!("{line}");
}

/// Prints the result of a `run_lock` call the way every lock-only mode does and returns the exit code.
fn print_lock_outcome(verb: &str, scope: &Scope, outcome: Result<String, ToolchainError>) -> i32 {
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
fn report(error: &ToolchainError) -> i32 {
    eprintln!("mpm-shadow: {error}");
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
pub fn older_lock_warning_lines(verb: &str, scope: &Scope, family: Option<OsFamily>) -> Vec<String> {
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
fn warn_older_locks(verb: &str, scope: &Scope) {
    for line in older_lock_warning_lines(verb, scope, Host::current().family) {
        eprintln!("{line}");
    }
}

/// The directory the command was started in (the root search starts here).
fn start_directory() -> PathBuf {
    return match std::env::current_dir() {
        Ok(directory) => directory,
        Err(_) => PathBuf::from("."),
    };
}

/// What:
/// Runs the toolchain verb `name` with the arguments `matches`,
/// or returns `None` when `name` is not one of
///           this module's verbs.
/// Why:
/// `cli.rs` asks this first and falls back to its own `prune`.
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
        "install" => {
            let frozen_flag: bool = matches.get_flag("frozen-lockfile");
            let lockfile_only: bool = matches.get_flag("lockfile-only");
            let offline: bool = matches.get_flag("offline");
            // pnpm: frozen when asked, or by default in CI; `--no-frozen-lockfile` turns it off.
            let policy: LockPolicy = if frozen_flag {
                LockPolicy::Frozen(FrozenReason::Flag)
            } else if matches.get_flag("no-frozen-lockfile") || !is_ci(&env) {
                LockPolicy::Resolve
            } else {
                LockPolicy::Frozen(FrozenReason::Ci)
            };
            if lockfile_only {
                // A frozen lock check writes nothing and needs no network.
                if matches!(policy, LockPolicy::Frozen(_)) && scope.lock_file.exists() {
                    return Some(print_lock_outcome("install", &scope, run_lock(&scope, LockMode::Check)));
                }
                if offline {
                    return Some(print_lock_outcome("install", &scope, run_lock(&scope, LockMode::Write)));
                }
                let http = connect_or_explain();
                return Some(match run_lockfile_only(&scope, &env, &ProcessRunner, http.as_ref(), &Host::current(), policy) {
                    Ok(prepared) => {
                        for (id, version) in &prepared.resolved {
                            print!("{}", locked_line(id, version));
                        }
                        for description in &prepared.rewritten {
                            print!("{}", rewritten_line(description));
                        }
                        if prepared.resolved.is_empty() && prepared.rewritten.is_empty() {
                            println!("toolchain lock already current: {}", scope.lock_file.display());
                        } else {
                            println!("toolchain install: wrote {}; nothing was installed", scope.lock_file.display());
                        }
                        warn_older_locks("install", &scope);
                        0
                    }
                    Err(error) => report(&error),
                });
            }
            let http = connect_or_explain();
            return Some(match run_sync_with(&scope, &env, &ProcessRunner, http.as_ref(), &Host::current(), policy) {
                Ok(sync_report) => {
                    println!(
                        "toolchain install: platform {}, root {}, exposure {}",
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
        "update" | "outdated" => {
            // pnpm outdated reports and exits 1 when anything is outdated; pnpm update writes.
            let check: bool = name == "outdated";
            let names: Vec<String> = match matches.get_many::<String>("tool") {
                Some(given) => given.cloned().collect(),
                None => Vec::new(),
            };
            let http = connect_or_explain();
            return Some(match run_update_for(&scope, &env, &ProcessRunner, http.as_ref(), &Host::current(), check, &names) {
                Ok(update_report) => {
                    print!("{}", format_update_report(&update_report, check));
                    warn_older_locks(name, &scope);
                    i32::from(update_report.has_failures() || (check && update_report.has_movement()))
                }
                Err(error) => report(&error),
            });
        }
        "verify-archive" => {
            let tool: &str = matches.get_one::<String>("tool").map(String::as_str).unwrap_or("");
            let file: &str = matches.get_one::<String>("file").map(String::as_str).unwrap_or("");
            let platform: String = match matches.get_one::<String>("platform") {
                Some(given) => given.clone(),
                None => Host::current().platform,
            };
            return Some(match run_verify_archive(&scope, &platform, tool, std::path::Path::new(file)) {
                Ok(line) => {
                    print_line(&line);
                    0
                }
                Err(error) => report(&error),
            });
        }
        "list" => {
            return Some(match run_status(&scope, &env, &ProcessRunner, &Host::current()) {
                Ok(status_report) => {
                    print!("{}", status_report.text);
                    i32::from(status_report.drifted)
                }
                Err(error) => report(&error),
            });
        }
        "bin" => {
            // Plain output is what `pnpm bin` prints: the directory the executables are linked into.
            if !matches.get_flag("json") {
                println!("{}", scope.exposure_dir.display());
                warn_older_locks("bin", &scope);
                return Some(0);
            }
            // Only the plan is read: the IDE sandbox may run this before any lock exists.
            let outcome = match read_plan(&scope.plan_file) {
                Ok(plan) => bin_paths_output(&plan, &scope, &env),
                Err(error) => Err(error),
            };
            return Some(match outcome {
                Ok(document) => {
                    print!("{}", render_bin_paths(&document));
                    warn_older_locks("bin", &scope);
                    0
                }
                Err(error) => report(&error),
            });
        }
        _ => return None,
    }
}
