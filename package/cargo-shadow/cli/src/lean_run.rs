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
use crate::lean_identity::Deadline;
/// Imports the names this file uses from `crate::lean_run_trees`.
use crate::lean_run_trees::tree_report;
/// Imports the names this file uses from `crate::lean_keyed_folder`.
use crate::lean_keyed_folder::configured_base;
/// Imports the names this file uses from `crate::lean_policy`.
use crate::lean_policy::{GRACE_SECONDS, HOST_PROFILE_CAP_BYTES, IN_TREE_PROFILE_CAP_BYTES};
/// Imports the names this file uses from `crate::lean_progress`.
use crate::lean_progress::{LongPass, Progress, resume_index, state_directory};
/// Imports the names this file uses from `crate::lean_rustup_toolchains`.
use crate::lean_rustup_toolchains::dated_nightly_name;
/// Imports the names this file uses from `crate::lean_run_discovery`.
use crate::lean_run_discovery::{target_directories, verification_directories};
/// Imports the names this file uses from `crate::lean_run_report`.
use crate::lean_run_report::{Place, measure_places, space_lines};
/// Imports the names this file uses from `crate::lean_run_steps`.
use crate::lean_run_steps::{UnitsPass, run_podman, run_records, run_toolchains, sweep_and_cap};

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
pub(crate) fn retired_toolchain_names(trees: &[Tree], pinned: &HashSet<String>) -> HashSet<String> {
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
/// `pub struct MechanismReport` is what one mechanism of a run did.
/// Why:
/// A run needs its lines and,
/// for the two long passes,
/// where the next run continues.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MechanismReport {
    /// The lines the mechanism reports.
    pub lines: Vec<String>,
    /// The last item a pass judged,
    /// when the deadline stopped it before the end of the list it was given;
    /// nothing when the pass finished,
    /// so the next run starts at the front.
    pub resume_after: Option<PathBuf>,
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
/// `fn unit_profiles(base: Option<&Path>, checkouts: &[PathBuf], deadline: &Deadline) -> Option<Vec<(PathBuf, u64)>>` lists every profile
/// directory the sweep and the size caps reach,
/// each with the cap that applies to it,
/// and answers nothing when the deadline passed before the listing finished.
/// Why:
/// Mechanisms B and E read the same directories with the same caps,
/// in the checkout's own `target/` directories and in the host build base,
/// and the list is sorted so a recorded position names the same place on the next run.
/// The listing itself is the longest fixed cost of that pass:
/// measured 2026-11-10,
/// 880 s to walk 68,755 profile directories,
/// so it obeys the deadline like everything else.
/// A listing the deadline cut is not judged at all:
/// a sorted list that is missing its tail is not the list a recorded position indexes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unitProfiles(base: string | undefined, checkouts: string[], deadline: Deadline): [string, number][] | undefined
/// ```
fn unit_profiles(base: Option<&Path>, checkouts: &[PathBuf], deadline: &Deadline) -> Option<Vec<(PathBuf, u64)>> {
    let mut profiles: Vec<(PathBuf, u64)> = Vec::new();
    for checkout in checkouts {
        for target in target_directories(checkout) {
            if deadline.passed() {
                return None;
            }
            for profile in profile_dirs_of(&target, deadline)? {
                profiles.push((profile, IN_TREE_PROFILE_CAP_BYTES));
            }
        }
    }
    if let Some(found) = base {
        for tree in list_trees(found, deadline)? {
            if deadline.passed() {
                return None;
            }
            for profile in profile_dirs_of(&tree.path, deadline)? {
                profiles.push((profile, HOST_PROFILE_CAP_BYTES));
            }
        }
    }
    profiles.sort_by(by_profile);
    profiles.dedup_by(same_profile);
    return Some(profiles);
}

/// What:
/// `fn by_profile(left: &(PathBuf, u64), right: &(PathBuf, u64)) -> std::cmp::Ordering` is the order a recorded position
/// indexes.
/// Why:
/// The units pass continues after a key,
/// so two runs must walk the profile directories in the same order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byProfile = (left: [string, number], right: [string, number]): number => left[0] < right[0] ? -1 : left[0] > right[0] ? 1 : 0;
/// ```
fn by_profile(left: &(PathBuf, u64), right: &(PathBuf, u64)) -> std::cmp::Ordering {
    return left.0.cmp(&right.0);
}

/// What:
/// `fn same_profile(left: &mut (PathBuf, u64), right: &mut (PathBuf, u64)) -> bool` says whether two entries name the same
/// directory.
/// Why:
/// A checkout and a build tree can name one profile directory,
/// and judging it twice would pay for it twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sameProfile = (left: [string, number], right: [string, number]): boolean => left[0] === right[0];
/// ```
fn same_profile(left: &mut (PathBuf, u64), right: &mut (PathBuf, u64)) -> bool {
    return left.0 == right.0;
}

/// What a mechanism is given when the run reaches it.
pub(crate) struct RunContext<'a> {
    /// What the caller asked for.
    pub(crate) options: &'a RunOptions,
    /// The run's deadline.
    pub(crate) deadline: Deadline,
    /// The real `/proc` the guards read.
    pub(crate) environment: &'a GuardEnvironment,
    /// The checkouts the run found.
    pub(crate) checkouts: &'a [PathBuf],
    /// The toolchains a checkout's lock pins.
    pub(crate) pinned: &'a HashSet<String>,
    /// The host build base,
    /// when the Cargo configuration has one.
    pub(crate) base: Option<&'a Path>,
    /// The rustup home the toolchains live in.
    pub(crate) rustup_home: &'a Path,
    /// Where the last run stopped in each long pass.
    pub(crate) progress: &'a Progress,
}

/// What:
/// `pub fn long_pass(mechanism: Mechanism) -> Option<LongPass>` is the long pass a mechanism continues,
/// when it is one.
/// Why:
/// Only the idle-trees pass and the units pass are longer than a run's bound,
/// so only they keep a position between runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const longPass = (mechanism: Mechanism): LongPass | undefined => mechanism === Mechanism.Trees ? LongPass.Trees : mechanism === Mechanism.Units ? LongPass.Units : undefined;
/// ```
pub fn long_pass(mechanism: Mechanism) -> Option<LongPass> {
    return match mechanism {
        Mechanism::Trees => Some(LongPass::Trees),
        Mechanism::Units => Some(LongPass::Units),
        Mechanism::Records | Mechanism::Toolchains | Mechanism::Podman => None,
    };
}

/// What:
/// `pub(crate) fn run_mechanism(mechanism: Mechanism, context: &RunContext) -> MechanismReport` runs one mechanism and returns
/// its summary lines and where the next run continues.
/// Why:
/// `run_lean` walks `RUN_ORDER` and each mechanism is one arm here,
/// so the order has one place to change,
/// and the two long passes start after the position the last run recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runMechanism(mechanism: Mechanism, context: RunContext): MechanismReport
/// ```
pub(crate) fn run_mechanism(mechanism: Mechanism, context: &RunContext) -> MechanismReport {
    return match mechanism {
        Mechanism::Trees => match context.base {
            Some(found) => tree_report(found, context.progress.after(LongPass::Trees), context.options, &context.deadline, context.environment, context.pinned),
            None => MechanismReport {
                lines: vec![String::from("idle trees: no build.build-dir with {workspace-path-hash} in the Cargo configuration, so there is no host build base")],
                resume_after: None,
            },
        },
        Mechanism::Units => {
            let listing_began: std::time::Instant = std::time::Instant::now();
            let Some(profiles) = unit_profiles(context.base, context.checkouts, &context.deadline) else {
                return MechanismReport {
                    lines: vec![
                        format!("older-compiler sweep: the deadline passed while listing the profile directories, {} s in, so none was judged", listing_began.elapsed().as_secs()),
                        String::from("  the list is sorted for the recorded position, so a part of it is not the list the next pass continues in"),
                    ],
                    resume_after: context.progress.after(LongPass::Units).map(Path::to_path_buf),
                };
            };
            let listing_seconds: u64 = listing_began.elapsed().as_secs();
            let mut keys: Vec<PathBuf> = Vec::with_capacity(profiles.len());
            for (profile, _cap) in &profiles {
                keys.push(profile.clone());
            }
            let start: usize = resume_index(&keys, context.progress.after(LongPass::Units));
            let chosen: &[(PathBuf, u64)] = &profiles[start..];
            let UnitsPass { mut lines, judged } = sweep_and_cap(chosen, context.options.dry_run, &context.deadline, context.environment);
            let last_judged: Option<&(PathBuf, u64)> = if judged > 0 { Some(&chosen[judged - 1]) } else { None };
            let resume_after: Option<PathBuf> = match (judged < chosen.len(), last_judged) {
                (true, Some(last)) => Some(last.0.clone()),
                (true, None) if start > 0 => Some(profiles[start - 1].0.clone()),
                _ => None,
            };
            lines.insert(1, format!("  listed {} profile directories in {} s", profiles.len(), listing_seconds));
            // The position notes come before the notes of what was skipped or left alone.
            if let (true, Some(last)) = (judged < chosen.len(), last_judged) {
                lines.insert(1, format!("  the deadline passed after {judged} of {} profile directories, at {}; the next run continues after it", chosen.len(), last.0.display()));
            }
            if start > 0 {
                lines.insert(1, format!("  this pass started after {}, the profile directory the last run stopped at", profiles[start - 1].0.display()));
            }
            MechanismReport { lines, resume_after }
        }
        Mechanism::Records => {
            let mut targets: Vec<PathBuf> = Vec::new();
            for checkout in context.checkouts {
                targets.extend(target_directories(checkout));
            }
            MechanismReport { lines: run_records(&verification_directories(&targets), context.options.dry_run, &context.deadline, context.environment), resume_after: None }
        }
        Mechanism::Toolchains => MechanismReport {
            lines: run_toolchains(&context.rustup_home.join("toolchains"), context.pinned, context.options.dry_run, &context.deadline, context.environment),
            resume_after: None,
        },
        Mechanism::Podman => {
            let main: PathBuf = match crate::lean_checkouts::main_checkout(&context.options.start) {
                Some(found) => found,
                None => match context.checkouts.first() {
                    Some(first) => first.clone(),
                    None => {
                        return MechanismReport { lines: vec![String::from("podman: no checkout was found, so nothing was judged")], resume_after: None };
                    }
                },
            };
            MechanismReport { lines: run_podman(&main, context.options.dry_run, &context.deadline), resume_after: None }
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
    let mut progress: Progress = Progress::default();
    let state: Option<PathBuf> = state_directory();
    match &state {
        Some(directory) => {
            let (recorded, notes) = crate::lean_progress::read(directory);
            progress = recorded;
            say(&mut summary, notes);
        }
        None => say(&mut summary, vec![String::from("lean progress: neither XDG_STATE_HOME nor HOME names a state directory, so every pass starts at the front")]),
    }
    let pinned: HashSet<String> = crate::lean_rustup_toolchains::pinned_by(&checkouts, &rustup_home.join("settings.toml"));
    for mechanism in run_order(&options.only) {
        let context: RunContext = RunContext { options, deadline, environment: &environment, checkouts: &checkouts, pinned: &pinned, base: base.as_deref(), rustup_home: &rustup_home, progress: &progress };
        let report: MechanismReport = run_mechanism(mechanism, &context);
        say(&mut summary, report.lines);
        // A dry run removes nothing and reaches nothing new, so it never moves the recorded position.
        if let (Some(directory), Some(pass), false) = (state.as_deref(), long_pass(mechanism), options.dry_run) {
            progress.set(pass, report.resume_after);
            match progress.save(directory) {
                Ok(()) => {}
                Err(reason) => say(&mut summary, vec![format!("lean progress: could not record where this run stopped: {reason}")]),
            }
        }
    }
    say(&mut summary, space_lines(&measured));
    if deadline.passed() {
        let where_it_continues: String = match &state {
            Some(directory) => format!(", and the idle-trees and units passes record where they stopped in {}", directory.join(crate::lean_progress::PROGRESS_FILE_NAME).display()),
            None => String::new(),
        };
        say(&mut summary, vec![format!("the run's deadline passed: what was not reached waits for the next run{where_it_continues}")]);
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
