//! What:
//! `cargo-shadow lean run`:
//! the one pass of the lean-folder tools over this machine,
//! in the plan's order (idle trees,
//! older-compiler sweep,
//! size caps,
//! run records,
//! toolchains,
//! Podman).
//! Why:
//! The owner decided (2026-11-07) that the tools' first run,
//! under their own locks,
//! cleans what exists today,
//! and that a systemd user timer runs them daily.
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

/// Sets,
/// paths and time.
use std::{collections::HashSet, path::{Path, PathBuf}, time::Duration};

/// Imports the names this file uses from `crate::lean_build_trees`.
use crate::lean_build_trees::{Tree, TreeKind, list_trees, profile_dirs_of};
/// Imports the names this file uses from `crate::lean_checkouts`.
use crate::lean_checkouts::list_checkouts;
/// Imports the names this file uses from `crate::lean_guard`.
use crate::lean_guard::GuardEnvironment;
/// Imports the names this file uses from `crate::lean_identity`.
use crate::lean_identity::{Deadline, now_seconds};
/// Imports the names this file uses from `crate::lean_idle_trees`.
use crate::lean_idle_trees::{IdlePolicy, TreeVerdict, collect_idle_trees};
/// Imports the names this file uses from `crate::lean_keyed_folder`.
use crate::lean_keyed_folder::configured_base;
/// Imports the names this file uses from `crate::lean_policy`.
use crate::lean_policy::{GRACE_SECONDS, HOST_PROFILE_CAP_BYTES, IN_TREE_PROFILE_CAP_BYTES, TREE_IDLE_SECONDS, UNIDENTIFIED_TREE_IDLE_SECONDS};
/// Imports the names this file uses from `crate::lean_rustup_toolchains`.
use crate::lean_rustup_toolchains::dated_nightly_name;
/// Imports the names this file uses from `crate::lean_run_discovery`.
use crate::lean_run_discovery::{target_directories, verification_directories};
/// Imports the names this file uses from `crate::lean_run_report`.
use crate::lean_run_report::{Place, gib, measure_places, space_lines};
/// Imports the names this file uses from `crate::lean_run_steps`.
use crate::lean_run_steps::{run_podman, run_records, run_toolchains, sweep_and_cap};

/// The mechanisms of the run,
/// by the name `--only` takes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Mechanism {
    /// C:
    /// idle trees of the host build base.
    Trees,
    /// B and E:
    /// the older-compiler sweep and the size caps of profile directories.
    Units,
    /// D:
    /// verification run records.
    Records,
    /// Dated nightly toolchains no checkout pins.
    Toolchains,
    /// Unused Podman images and volumes.
    Podman,
}

/// What the caller asked for.
#[derive(Clone, Debug)]
pub struct RunOptions {
    /// Report what would be removed and remove nothing.
    pub dry_run: bool,
    /// The mechanisms to run (all when empty).
    pub only: Vec<Mechanism>,
    /// The longest the whole run may take.
    pub deadline: Duration,
    /// The directory the run starts from,
    /// inside a checkout.
    pub start: PathBuf,
}

/// Which mechanisms a run includes.
impl RunOptions {
    /// Whether a mechanism is part of this run.
    pub fn includes(&self, mechanism: Mechanism) -> bool {
        return self.only.is_empty() || self.only.contains(&mechanism);
    }
}

/// What:
/// `fn retired_toolchain_names(trees: &[Tree], pinned: &HashSet<String>) -> HashSet<String>` lists the dated nightlies whose keyed
/// folders exist and that no checkout pins.
/// Why:
/// Retiring a toolchain is removing its folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retiredToolchainNames(trees: Tree[], pinned: Set<string>): Set<string>
/// ```
fn retired_toolchain_names(trees: &[Tree], pinned: &HashSet<String>) -> HashSet<String> {
    let mut retired: HashSet<String> = HashSet::new();
    for tree in trees {
        if let TreeKind::Keyed(name) = &tree.kind
            && dated_nightly_name(name).is_some()
            && !pinned.contains(name)
        {
            retired.insert(name.clone());
        }
    }
    return retired;
}

/// What:
/// `fn run_trees(base: &Path, pinned: &HashSet<String>, options: &RunOptions, deadline: &Deadline, environment: &GuardEnvironment) -> Vec<String>`
/// runs mechanism C and returns its summary lines.
/// Why:
/// It reports removed trees by count and size,
/// and every kept or skipped tree by its reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runTrees(base: string, pinned: Set<string>, options: RunOptions, deadline: Deadline, env: GuardEnvironment): string[]
/// ```
fn run_trees(base: &Path, pinned: &HashSet<String>, options: &RunOptions, deadline: &Deadline, environment: &GuardEnvironment) -> Vec<String> {
    let trees: Vec<Tree> = list_trees(base);
    let retired: HashSet<String> = retired_toolchain_names(&trees, pinned);
    let policy: IdlePolicy = IdlePolicy { idle_seconds: TREE_IDLE_SECONDS, unidentified_idle_seconds: UNIDENTIFIED_TREE_IDLE_SECONDS };
    let mut log: Vec<String> = Vec::new();
    let outcomes = collect_idle_trees(&trees, base, &retired, &policy, environment, now_seconds(), options.dry_run, deadline, &mut log);
    for line in &log {
        crate::lean_run_report::log_line(line);
    }
    let (mut removed, mut bytes, mut kept, mut skipped): (usize, u64, usize, usize) = (0, 0, 0, 0);
    let mut skips: Vec<String> = Vec::new();
    for outcome in &outcomes {
        match &outcome.verdict {
            TreeVerdict::Removed { bytes: freed, .. } => {
                removed += 1;
                bytes += freed;
            }
            TreeVerdict::Keep(_) => kept += 1,
            TreeVerdict::Skip(why) => {
                skipped += 1;
                skips.push(format!("  skipped {}: {why}", outcome.tree.display()));
            }
        }
    }
    let verb: &str = if options.dry_run { "would remove" } else { "removed" };
    let mut lines: Vec<String> = vec![format!("idle trees: {} trees read; {verb} {removed} ({} by disk blocks); kept {kept}; skipped {skipped}", trees.len(), gib(bytes))];
    lines.extend(skips.into_iter().take(25));
    return lines;
}

/// What:
/// `fn say(summary: &mut Vec<String>, lines: Vec<String>)` prints each line at once and keeps it.
/// Why:
/// A run of hours shows each mechanism's result when that mechanism ends,
/// not only at the very end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function say(summary: string[], lines: string[]): void
/// ```
fn say(summary: &mut Vec<String>, lines: Vec<String>) {
    for line in lines {
        crate::lean_run_report::summary_line(&line);
        summary.push(line);
    }
}

/// What:
/// `pub fn run_lean(options: &RunOptions) -> i32` runs the selected mechanisms and prints the summary.
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
        eprintln!("cargo-shadow: lean run: {} is not inside a Git checkout, so no checkout was found; start it from the repository", options.start.display());
        return 1;
    }
    let home: PathBuf = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let cargo_home: PathBuf = match std::env::var_os("CARGO_HOME") {
        Some(chosen) => PathBuf::from(chosen),
        None => home.join(".cargo"),
    };
    let rustup_home: PathBuf = match std::env::var_os("RUSTUP_HOME") {
        Some(chosen) => PathBuf::from(chosen),
        None => home.join(".rustup"),
    };
    let base: Option<PathBuf> = configured_base(&cargo_home);
    let mut places: Vec<(String, PathBuf)> = vec![(String::from("/"), PathBuf::from("/")), (String::from("home"), home.clone())];
    if let Some(found) = &base {
        places.push((String::from("host build base"), found.clone()));
    }
    let measured: Vec<Place> = measure_places(&places);
    let mut summary: Vec<String> = Vec::new();
    say(&mut summary, vec![format!("lean run: {} checkouts, {}", checkouts.len(), if options.dry_run { "dry run (nothing is removed)" } else { "removing" })]);
    let pinned: HashSet<String> = crate::lean_rustup_toolchains::pinned_by(&checkouts, &rustup_home.join("settings.toml"));
    if options.includes(Mechanism::Trees) {
        match &base {
            Some(found) => say(&mut summary, run_trees(found, &pinned, options, &deadline, &environment)),
            None => say(&mut summary, vec![String::from("idle trees: no build.build-dir with {workspace-path-hash} in the Cargo configuration, so there is no host build base")]),
        }
    }
    if options.includes(Mechanism::Units) {
        let mut profiles: Vec<(PathBuf, u64)> = Vec::new();
        for checkout in &checkouts {
            for target in target_directories(checkout) {
                for profile in profile_dirs_of(&target) {
                    profiles.push((profile, IN_TREE_PROFILE_CAP_BYTES));
                }
            }
        }
        if let Some(found) = &base {
            for tree in list_trees(found) {
                for profile in profile_dirs_of(&tree.path) {
                    profiles.push((profile, HOST_PROFILE_CAP_BYTES));
                }
            }
        }
        say(&mut summary, sweep_and_cap(&profiles, options.dry_run, &deadline, &environment));
    }
    if options.includes(Mechanism::Records) {
        let mut targets: Vec<PathBuf> = Vec::new();
        for checkout in &checkouts {
            targets.extend(target_directories(checkout));
        }
        say(&mut summary, run_records(&verification_directories(&targets), options.dry_run, &deadline, &environment));
    }
    if options.includes(Mechanism::Toolchains) {
        say(&mut summary, run_toolchains(&rustup_home.join("toolchains"), &pinned, options.dry_run, &deadline, &environment));
    }
    if options.includes(Mechanism::Podman) {
        let main: PathBuf = match crate::lean_checkouts::main_checkout(&options.start) {
            Some(found) => found,
            None => checkouts[0].clone(),
        };
        say(&mut summary, run_podman(&main, options.dry_run, &deadline));
    }
    say(&mut summary, space_lines(&measured));
    if deadline.passed() {
        say(&mut summary, vec![String::from("the run's deadline passed: whatever was not reached waits for the next run")]);
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
