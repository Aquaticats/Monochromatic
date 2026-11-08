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
//! and that a systemd user timer runs them daily.
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

/// The mechanisms in the order a run takes them:
/// cheapest first,
/// the units pass last.
/// Why the order is this and not the plan's:
/// one deadline covers the whole run,
/// and the measured cost of the first run (2026-11-07,
/// `doc/handover/cargo-shadow-lean-build-folders.md`,
/// section "First run") is
/// 2.5 minutes for records and toolchains together,
/// 20.5 minutes for Podman,
/// 29 minutes for the idle trees,
/// and hours for the units pass,
/// which reached about 700 of several thousand profile directories in 40 minutes.
/// A pass that a deadline cuts off waits for the next run;
/// a pass put last therefore starves only itself.
pub const RUN_ORDER: [Mechanism; 5] = [Mechanism::Records, Mechanism::Toolchains, Mechanism::Podman, Mechanism::Trees, Mechanism::Units];

/// What:
/// `pub fn run_order(requested: &[Mechanism]) -> Vec<Mechanism>` is the mechanisms of a run in the order it takes them.
/// Why:
/// The deadline cuts what it reaches first,
/// so the order is decided here once and the run walks it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runOrder(requested: Mechanism[]): Mechanism[]
/// ```
pub fn run_order(requested: &[Mechanism]) -> Vec<Mechanism> {
    let mut order: Vec<Mechanism> = Vec::new();
    for mechanism in RUN_ORDER {
        if requested.is_empty() || requested.contains(&mechanism) {
            order.push(mechanism);
        }
    }
    return order;
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
/// `fn unit_profiles(base: Option<&Path>, checkouts: &[PathBuf]) -> Vec<(PathBuf, u64)>` lists every profile directory the sweep and the size caps reach,
/// each with the cap that applies to it.
/// Why:
/// Mechanisms B and E read the same directories with the same caps,
/// in the checkout's own `target/` directories and in the host build base.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unitProfiles(base: string | undefined, checkouts: string[]): [string, number][]
/// ```
fn unit_profiles(base: Option<&Path>, checkouts: &[PathBuf]) -> Vec<(PathBuf, u64)> {
    let mut profiles: Vec<(PathBuf, u64)> = Vec::new();
    for checkout in checkouts {
        for target in target_directories(checkout) {
            for profile in profile_dirs_of(&target) {
                profiles.push((profile, IN_TREE_PROFILE_CAP_BYTES));
            }
        }
    }
    if let Some(found) = base {
        for tree in list_trees(found) {
            for profile in profile_dirs_of(&tree.path) {
                profiles.push((profile, HOST_PROFILE_CAP_BYTES));
            }
        }
    }
    return profiles;
}

/// What a mechanism is given when the run reaches it.
struct RunContext<'a> {
    /// What the caller asked for.
    options: &'a RunOptions,
    /// The run's deadline.
    deadline: Deadline,
    /// The real `/proc` the guards read.
    environment: &'a GuardEnvironment,
    /// The checkouts the run found.
    checkouts: &'a [PathBuf],
    /// The toolchains a checkout's lock pins.
    pinned: &'a HashSet<String>,
    /// The host build base,
    /// when the Cargo configuration has one.
    base: Option<&'a Path>,
    /// The rustup home the toolchains live in.
    rustup_home: &'a Path,
}

/// What:
/// `fn run_mechanism(mechanism: Mechanism, context: &RunContext) -> Vec<String>` runs one mechanism and returns its summary lines.
/// Why:
/// `run_lean` walks `RUN_ORDER` and each mechanism is one arm here,
/// so the order has one place to change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runMechanism(mechanism: Mechanism, context: RunContext): string[]
/// ```
fn run_mechanism(mechanism: Mechanism, context: &RunContext) -> Vec<String> {
    return match mechanism {
        Mechanism::Trees => match context.base {
            Some(found) => run_trees(found, context.pinned, context.options, &context.deadline, context.environment),
            None => vec![String::from("idle trees: no build.build-dir with {workspace-path-hash} in the Cargo configuration, so there is no host build base")],
        },
        Mechanism::Units => sweep_and_cap(&unit_profiles(context.base, context.checkouts), context.options.dry_run, &context.deadline, context.environment),
        Mechanism::Records => {
            let mut targets: Vec<PathBuf> = Vec::new();
            for checkout in context.checkouts {
                targets.extend(target_directories(checkout));
            }
            run_records(&verification_directories(&targets), context.options.dry_run, &context.deadline, context.environment)
        }
        Mechanism::Toolchains => run_toolchains(&context.rustup_home.join("toolchains"), context.pinned, context.options.dry_run, &context.deadline, context.environment),
        Mechanism::Podman => {
            let main: PathBuf = match crate::lean_checkouts::main_checkout(&context.options.start) {
                Some(found) => found,
                None => match context.checkouts.first() {
                    Some(first) => first.clone(),
                    None => return vec![String::from("podman: no checkout was found, so nothing was judged")],
                },
            };
            run_podman(&main, context.options.dry_run, &context.deadline)
        }
    };
}

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
    let context: RunContext = RunContext { options, deadline, environment: &environment, checkouts: &checkouts, pinned: &pinned, base: base.as_deref(), rustup_home: &rustup_home };
    for mechanism in run_order(&options.only) {
        say(&mut summary, run_mechanism(mechanism, &context));
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
