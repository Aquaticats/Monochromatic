//! What:
//! `cargo-shadow lean run`:
//! the one pass of the lean-folder tools over this machine,
//! cheapest mechanism first and the units pass last
//! (run records,
//! toolchains,
//! Podman,
//! idle trees,
//! older-compiler sweep and size caps).
//! Why:
//! The owner decided (2026-11-07) that the tools' first run,
//! under their own locks,
//! cleans what exists today,
//! and that a systemd user unit runs them;
//! since 2026-11-11 that unit is a continuous service,
//! one pass after another with a pause between them.
//! One deadline covers the whole run,
//! so the order is what decides which mechanism a deadline cuts:
//! measured in the first run (2026-11-07T23:52:28-05:00 to 2026-11-08T01:24:39-05:00),
//! records and toolchains took 2.5 minutes together,
//! the Podman pass 20.5 minutes,
//! the idle-trees pass 29 minutes,
//! and the units pass was stopped 40 minutes in after about 700 of several thousand profile directories.
//! In the plan's order that pass came second and starved the Podman pass.
//! The run never starts `cargo`,
//! never starts itself,
//! and obeys one deadline;
//! each mechanism prints one line per removal with its proof on standard error,
//! and the summary on standard output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function runLean(options: RunOptions): number
//! ```
//!
//! The run's parts are one file each:
//! the mechanisms,
//! what the caller asked for and the order (`mechanisms_and_options.rs`),
//! what a mechanism reports and what it is given (`report_and_context.rs`),
//! the dispatch of one mechanism (`mechanism_dispatch.rs`),
//! the profile directories the units pass walks (`unit_profile_listing.rs`),
//! and the dated nightlies no checkout pins (`retired_toolchains.rs`).

/// The set of toolchain names the checkouts pin.
use std::collections::HashSet;
/// The owned path type the recorded positions are keys of.
use std::path::PathBuf;

/// Imports the names this file uses from `crate::lean_checkouts`.
use crate::lean_checkouts::list_checkouts;
/// Imports the names this file uses from `crate::lean_guard`.
use crate::lean_guard::GuardEnvironment;
/// Imports the names this file uses from `crate::lean_identity`.
use crate::lean_identity::Deadline;
/// Imports the names this file uses from `crate::lean_keyed_folder`.
use crate::lean_keyed_folder::configured_base;
/// Imports the names this file uses from `crate::lean_policy`.
use crate::lean_policy::GRACE_SECONDS;
/// Imports the names this file uses from `crate::lean_progress`.
use crate::lean_progress::{
    Progress,
    state_directory,
};
/// Imports the names this file uses from `crate::lean_run_report`.
use crate::lean_run_report::{
    Place,
    measure_places,
    space_lines,
};

/// The dispatch of one mechanism of a run.
mod mechanism_dispatch;
/// The mechanisms a run takes,
/// what the caller asked for and the order they are taken in.
mod mechanisms_and_options;
/// What one mechanism reports and what the run hands it.
mod report_and_context;
/// The dated nightlies whose folders exist and that no checkout pins.
mod retired_toolchains;
/// The profile directories the units pass walks.
mod unit_profile_listing;

/// Re-exports the dispatch of one mechanism.
pub(crate) use self::mechanism_dispatch::run_mechanism;
/// Re-exports the mechanisms,
/// what the caller asked for and the order they are taken in.
pub use self::mechanisms_and_options::{
    Mechanism,
    RUN_ORDER,
    RunOptions,
    long_pass,
    run_order,
};
/// Re-exports what one mechanism of a run reports.
pub use self::report_and_context::MechanismReport;
/// Re-exports what a mechanism is given when the run reaches it.
pub(crate) use self::report_and_context::RunContext;
/// Imports the printing of each mechanism's result as that mechanism ends.
use self::report_and_context::say;
/// Re-exports the dated nightlies no checkout pins.
pub(crate) use self::retired_toolchains::retired_toolchain_names;

/// What:
/// `pub fn run_lean(options: &RunOptions) -> i32` runs the selected mechanisms in `RUN_ORDER` and prints the summary.
/// Why:
/// The exit code is 0 when the run completed (skips and kept items are not failures),
/// and 1 when no checkout could be found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runLean(options: RunOptions): number
/// ```
pub fn run_lean(options: &RunOptions) -> i32 {
    let deadline: Deadline = Deadline::after(options.deadline);
    let environment: GuardEnvironment = GuardEnvironment::real();
    let checkouts: Vec<PathBuf> = list_checkouts(&options.start);
    if checkouts.is_empty() {
        eprintln!(
            "cargo-shadow: lean run: {} is not inside a Git checkout, so no checkout was found; start it from the repository",
            options.start.display()
        );
        return 1;
    }
    let home: PathBuf = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let cargo_home: PathBuf = match std::env::var_os("CARGO_HOME") {
        Some(chosen) => PathBuf::from(chosen),
        None => home.join(".cargo"),
    };
    let rustup_home: PathBuf = match std::env::var_os("RUSTUP_HOME") {
        Some(chosen) => PathBuf::from(chosen),
        None => home.join(".rustup"),
    };
    let base: Option<PathBuf> = configured_base(&cargo_home);
    let mut places: Vec<(String, PathBuf)> = vec![
        (String::from("/"), PathBuf::from("/")),
        (String::from("home"), home.clone()),
    ];
    if let Some(found) = &base {
        places.push((String::from("host build base"), found.clone()));
    }
    let measured: Vec<Place> = measure_places(&places);
    let mut summary: Vec<String> = Vec::new();
    say(
        &mut summary,
        vec![format!(
            "lean run: {} checkouts, {}",
            checkouts.len(),
            if options.dry_run {
                "dry run (nothing is removed)"
            } else {
                "removing"
            }
        )],
    );
    let mut progress: Progress = Progress::default();
    let state: Option<PathBuf> = state_directory();
    match &state {
        Some(directory) => {
            let (recorded, notes) = crate::lean_progress::read(directory);
            progress = recorded;
            say(&mut summary, notes);
        }
        None => say(
            &mut summary,
            vec![String::from(
                "lean progress: neither XDG_STATE_HOME nor HOME names a state directory, so every pass starts at the front",
            )],
        ),
    }
    let pinned: HashSet<String> =
        crate::lean_rustup_toolchains::pinned_by(&checkouts, &rustup_home.join("settings.toml"));
    for mechanism in run_order(&options.only) {
        let context: RunContext = RunContext {
            options,
            deadline,
            environment: &environment,
            checkouts: &checkouts,
            pinned: &pinned,
            base: base.as_deref(),
            rustup_home: &rustup_home,
            progress: &progress,
        };
        let report: MechanismReport = run_mechanism(mechanism, &context);
        say(&mut summary, report.lines);
        // A dry run removes nothing and reaches nothing new, so it never moves the recorded position.
        if let (Some(directory), Some(pass), false) =
            (state.as_deref(), long_pass(mechanism), options.dry_run)
        {
            progress.set(pass, report.resume_after);
            match progress.save(directory) {
                Ok(()) => {}
                Err(reason) => say(
                    &mut summary,
                    vec![format!(
                        "lean progress: could not record where this run stopped: {reason}"
                    )],
                ),
            }
        }
    }
    say(&mut summary, space_lines(&measured));
    if deadline.passed() {
        let where_it_continues: String = match &state {
            Some(directory) => format!(
                ", and the idle-trees and units passes record where they stopped in {}",
                directory
                    .join(crate::lean_progress::PROGRESS_FILE_NAME)
                    .display()
            ),
            None => String::new(),
        };
        say(
            &mut summary,
            vec![format!(
                "the run's deadline passed: what was not reached waits for the next run{where_it_continues}"
            )],
        );
    }
    return 0;
}

/// What:
/// `pub fn grace_note() -> String` names the grace period in days for the help text.
/// Why:
/// The help says exactly what the run keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const graceNote = (): string => `${GRACE_SECONDS / 86400} days`;
/// ```
pub fn grace_note() -> String {
    return format!("{} days", GRACE_SECONDS / 86_400);
}

/// The tests of the order a run takes its mechanisms in.
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "lean_run_tests.rs"]
mod lean_run_tests;
