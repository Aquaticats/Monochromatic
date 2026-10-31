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

/// Imports the `clap` types for describing arguments and reading what was typed.
use clap::{Arg, ArgAction, ArgMatches, Command};

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
use super::sync::{FrozenReason, LockPolicy, format_sync_report, locked_line, rewritten_line, run_lockfile_only, run_sync_with};
/// Imports `is_ci` from `super::ci_detect`.
use super::ci_detect::is_ci;
/// Imports what this module uses from `super::verify_archive`.
use super::verify_archive::run_verify_archive;

/// Imports what this module uses from `super::update`.
use super::update::{format_update_report, run_update_for};

/// The `--global` flag every toolchain verb accepts:
/// act on the machine-global plan and lock.
fn global_flag() -> Arg {
    return Arg::new("global")
        .long("global")
        .short('g')
        .action(ArgAction::SetTrue)
        .help("Act on the machine-global toolchain (the plan and lock under the per-user config directory)");
}

/// What:
/// The toolchain subcommands for `mpm-shadow --help`.
/// Why:
/// Each verb is a plain `clap` command;
/// the grammar is data the tests can inspect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function subcommands(): Command[]
/// ```
pub fn subcommands() -> Vec<Command> {
    let install: Command = Command::new("install")
        .visible_alias("i")
        .about("Install every locked tool that is not at its locked version, then rebuild the exposure directory; a plan entry the lock lacks is resolved and locked alone first, unless frozen")
        .arg(
            Arg::new("frozen-lockfile")
                .long("frozen-lockfile")
                .action(ArgAction::SetTrue)
                .conflicts_with("no-frozen-lockfile")
                .help("Never resolve: refuse a plan that is ahead of the lock and name `mpm-shadow update <ids>`; with --lockfile-only, only check (the default in CI when a lock exists, like pnpm)"),
        )
        .arg(
            Arg::new("no-frozen-lockfile")
                .long("no-frozen-lockfile")
                .action(ArgAction::SetTrue)
                .help("Resolve a plan that is ahead of the lock even in CI"),
        )
        .arg(
            Arg::new("lockfile-only")
                .long("lockfile-only")
                .action(ArgAction::SetTrue)
                .help("Only bring toolchain.lock.json up to date; install nothing"),
        )
        .arg(
            Arg::new("offline")
                .long("offline")
                .action(ArgAction::SetTrue)
                .requires("lockfile-only")
                .help("Use no network: rewrite the lock's declaration fields from the plan, keeping every resolved version; refuse an entry that needs resolving"),
        )
        .arg(global_flag());
    let bin: Command = Command::new("bin")
        .about("Print the directory the toolchain's executables are linked into (read-only, offline)")
        .arg(
            Arg::new("json")
                .long("json")
                .action(ArgAction::SetTrue)
                .help("Print the program directories and exposed binaries as JSON instead (for the IDE launcher)"),
        )
        .arg(global_flag());
    let update: Command = Command::new("update")
        .about("Resolve the plan's tools (all, or only the named ones) and write toolchain.lock.json (looks versions up on the network)")
        .arg(tool_names_arg())
        .arg(global_flag());
    let outdated: Command = Command::new("outdated")
        .about("Report which tools `update` would move, writing and installing nothing; exits 1 when any tool is outdated, like pnpm outdated")
        .arg(tool_names_arg())
        .arg(global_flag());
    let verify_archive: Command = Command::new("verify-archive")
        .about("Check a downloaded archive's SHA-256 against the lock's row for a tool on a platform (the bootstrap check)")
        .arg(Arg::new("tool").required(true).help("The tool id in the lock, for example cargo:monochromatic-mpm-shadow"))
        .arg(Arg::new("file").required(true).help("The downloaded file"))
        .arg(
            Arg::new("platform")
                .long("platform")
                .help("The lock's platform id (default: this machine's, for example linux-x64)"),
        )
        .arg(global_flag());
    let list: Command = Command::new("list")
        .visible_alias("ls")
        .about("Compare the installed tools with the lock (read-only; exits 1 on drift or when the plan and lock disagree)")
        .arg(global_flag());
    return vec![install, update, outdated, list, verify_archive, bin];
}

/// The tool-id patterns `update` and `outdated` take.
fn tool_names_arg() -> Arg {
    return Arg::new("tool")
        .num_args(0..)
        .value_name("TOOL")
        .help("Tool ids of the plan, `*` matches any run of characters and a leading `!` excludes (default: every tool); the other rows keep their versions");
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
