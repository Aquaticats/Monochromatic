//! What:
//! The steps of a lean-folder run that sweep profile directories,
//! run records,
//! toolchains and Podman,
//! each returning its summary lines.
//! Why:
//! `lean_run.rs` orders the steps;
//! each step here only wires one mechanism to the real machine
//! (the real `/proc`,
//! the real `rustup`,
//! the real `podman`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function sweepAndCap(profiles: [string, number][], dryRun: boolean, deadline: Deadline, env: GuardEnvironment): string[]
//! ```

/// Sets,
/// paths and child processes.
use std::{collections::HashSet, path::{Path, PathBuf}, process::Command, time::Duration};

/// Imports the names this file uses from `crate::bounded_run`.
use crate::bounded_run::{Bound, RunError, output_within};
/// Imports the names this file uses from `crate::lean_container_references`.
use crate::lean_container_references::collect_references;
/// Imports the names this file uses from `crate::lean_guard`.
use crate::lean_guard::GuardEnvironment;
/// Imports the names this file uses from `crate::lean_identity`.
use crate::lean_identity::{Deadline, now_seconds};
/// Imports the names this file uses from `crate::lean_podman_cleanup`.
use crate::lean_podman_cleanup::clean_podman;
/// Imports the names this file uses from `crate::lean_podman_real`.
use crate::lean_podman_real::RealPodman;
/// Imports the names this file uses from `crate::lean_policy`.
use crate::lean_policy::{GRACE_SECONDS, RECORD_CAP_BYTES, RECORD_PROTECT_SECONDS, RECORD_RETENTION_SECONDS};
/// Imports the names this file uses from `crate::lean_run_report`.
use crate::lean_run_report::gib;
/// Imports the names this file uses from `crate::lean_rustup_toolchains`.
use crate::lean_rustup_toolchains::{ToolchainVerdict, Uninstaller, installed_toolchains, retire_nightlies};
/// Imports the names this file uses from `crate::lean_run_records_glue`.
use crate::lean_run_records_glue::sweep_records_directory;
/// Imports the names this file uses from `crate::lean_size_caps`.
use crate::lean_size_caps::SizeCap;
/// Imports the names this file uses from `crate::lean_sweep_plan`.
use crate::lean_sweep_plan::OlderCompilerSweep;
/// Imports the names this file uses from `crate::lean_unit_removal`.
use crate::lean_unit_removal::{ProfileOutcome, apply_removals};

/// How long one `rustup toolchain uninstall` may take.
const UNINSTALL_LIMIT: Duration = Duration::from_secs(600);

/// The real `rustup`.
struct RealRustup;

/// Uninstalls a toolchain through `rustup`.
impl Uninstaller for RealRustup {
    /// Runs `rustup toolchain uninstall` with a deadline.
    fn uninstall(&self, directory_name: &str) -> Result<(), String> {
        let bound: Bound = Bound { limit: UNINSTALL_LIMIT, what: format!("rustup toolchain uninstall {directory_name}"), rerun: String::from("cargo-shadow lean run") };
        return match output_within(Command::new("rustup").args(["toolchain", "uninstall", directory_name]), None, &bound) {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
            Err(RunError::TimedOut { what, limit, .. }) => Err(format!("{what} did not finish within {} s", limit.as_secs())),
            Err(other) => Err(format!("{other:?}")),
        };
    }
}

/// What:
/// `pub struct UnitsPass` is what one pass over the profile directories did.
/// Why:
/// The count of judged directories is where the next run continues,
/// and the lines are what the run reports.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UnitsPass {
    /// The lines the pass reports.
    pub lines: Vec<String>,
    /// How many of the profile directories it was given were judged before the deadline stopped it.
    pub judged: usize,
}

/// What:
/// `pub fn sweep_and_cap(profiles: &[(PathBuf, u64)], dry_run: bool, deadline: &Deadline, environment: &GuardEnvironment) -> UnitsPass`
/// runs the older-compiler sweep and then the size cap on each profile directory.
/// Why:
/// Mechanisms B and E share the protocol and the directories;
/// B runs first so E sizes what is left,
/// and a deadline that stops the pass says how far it got.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function sweepAndCap(profiles: [string, number][], dryRun: boolean, deadline: Deadline, env: GuardEnvironment): UnitsPass
/// ```
pub fn sweep_and_cap(profiles: &[(PathBuf, u64)], dry_run: bool, deadline: &Deadline, environment: &GuardEnvironment) -> UnitsPass {
    let now: i64 = now_seconds();
    let (mut sweep_units, mut sweep_incremental, mut sweep_bytes): (usize, usize, u64) = (0, 0, 0);
    let (mut cap_units, mut cap_incremental, mut cap_bytes): (usize, usize, u64) = (0, 0, 0);
    let mut notes: Vec<String> = Vec::new();
    let mut busy: usize = 0;
    let mut judged: usize = 0;
    for (profile, cap) in profiles {
        if deadline.passed() {
            break;
        }
        let mut log: Vec<String> = Vec::new();
        let sweep: ProfileOutcome = apply_removals(profile, &OlderCompilerSweep { grace_seconds: GRACE_SECONDS }, environment, now, dry_run, deadline, &mut log);
        let capped: ProfileOutcome = apply_removals(profile, &SizeCap { cap_bytes: *cap, grace_seconds: GRACE_SECONDS, deadline: *deadline }, environment, now, dry_run, deadline, &mut log);
        judged += 1;
        sweep_units += sweep.units;
        sweep_incremental += sweep.incremental;
        sweep_bytes += sweep.bytes;
        cap_units += capped.units;
        cap_incremental += capped.incremental;
        cap_bytes += capped.bytes;
        for outcome in [&sweep, &capped] {
            if let Some(why) = &outcome.skipped {
                if why.starts_with("busy") || why.starts_with("a build is queued") {
                    busy += 1;
                } else if !why.starts_with("unknown layout") {
                    notes.push(format!("  skipped {}: {why}", profile.display()));
                } else {
                    notes.push(format!("  left alone {}: {why}", profile.display()));
                }
            }
            for refusal in &outcome.refusals {
                notes.push(format!("  refused in {}: {refusal}", profile.display()));
            }
        }
    }
    let verb: &str = if dry_run { "would remove" } else { "removed" };
    let mut lines: Vec<String> = vec![
        format!("older-compiler sweep: {judged} of {} profile directories read; {verb} {sweep_units} units and {sweep_incremental} incremental directories ({})", profiles.len(), gib(sweep_bytes)),
        format!("size caps: {verb} {cap_units} units and {cap_incremental} incremental directories ({})", gib(cap_bytes)),
        format!("profile directories with a running or queued build, retried by the next run: {busy}"),
    ];
    lines.extend(notes.into_iter().take(40));
    return UnitsPass { lines, judged };
}

/// What:
/// `pub fn run_records(directories: &[PathBuf], dry_run: bool, deadline: &Deadline, environment: &GuardEnvironment) -> Vec<String>`
/// applies the run-record retention to each verification directory.
/// Why:
/// Mechanism D.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runRecords(directories: string[], dryRun: boolean, deadline: Deadline, env: GuardEnvironment): string[]
/// ```
pub fn run_records(directories: &[PathBuf], dry_run: bool, deadline: &Deadline, environment: &GuardEnvironment) -> Vec<String> {
    let (mut removed, mut bytes, mut kept): (usize, u64, usize) = (0, 0, 0);
    for directory in directories {
        if deadline.passed() {
            break;
        }
        let outcome = sweep_records_directory(directory, dry_run, deadline, environment, RECORD_RETENTION_SECONDS, RECORD_CAP_BYTES, RECORD_PROTECT_SECONDS);
        removed += outcome.removed;
        bytes += outcome.bytes;
        kept += outcome.kept_in_use;
    }
    let verb: &str = if dry_run { "would remove" } else { "removed" };
    return vec![format!("run records: {} verification directories read; {verb} {removed} records ({}); kept in use {kept}", directories.len(), gib(bytes))];
}

/// What:
/// `pub fn run_toolchains(toolchains: &Path, pinned: &HashSet<String>, dry_run: bool, deadline: &Deadline, environment: &GuardEnvironment) -> Vec<String>`
/// retires the dated nightlies nothing holds.
/// Why:
/// The owner's answer "Rust toolchains (Recommended)".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runToolchains(toolchains: string, pinned: Set<string>, dryRun: boolean, deadline: Deadline, env: GuardEnvironment): string[]
/// ```
pub fn run_toolchains(toolchains: &Path, pinned: &HashSet<String>, dry_run: bool, deadline: &Deadline, environment: &GuardEnvironment) -> Vec<String> {
    let installed = installed_toolchains(toolchains);
    let mut log: Vec<String> = Vec::new();
    let verdicts = retire_nightlies(&installed, pinned, &environment.proc_root, environment.own_pid, GRACE_SECONDS, now_seconds(), dry_run, &RealRustup, deadline, &mut log);
    let verb: &str = if dry_run { "would retire" } else { "retired" };
    let mut lines: Vec<String> = Vec::new();
    let mut retired: usize = 0;
    let mut bytes: u64 = 0;
    for (name, verdict) in &verdicts {
        match verdict {
            ToolchainVerdict::Retired { bytes: freed } => {
                retired += 1;
                bytes += freed;
            }
            ToolchainVerdict::Keep(why) => lines.push(format!("  kept {name}: {why}")),
            ToolchainVerdict::Skip(why) => lines.push(format!("  skipped {name}: {why}")),
        }
    }
    lines.insert(0, format!("toolchains: {} dated nightlies judged; {verb} {retired} ({})", verdicts.len(), gib(bytes)));
    return lines;
}

/// What:
/// `pub fn run_podman(main_checkout: &Path, dry_run: bool, deadline: &Deadline) -> Vec<String>` removes unused images and volumes.
/// Why:
/// The owner's answer "Podman images,
/// volumes (Recommended)":
/// as the user,
/// never as root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runPodman(mainCheckout: string, dryRun: boolean, deadline: Deadline): string[]
/// ```
pub fn run_podman(main_checkout: &Path, dry_run: bool, deadline: &Deadline) -> Vec<String> {
    let Some(references) = collect_references(main_checkout, deadline) else {
        return vec![String::from("podman: the deadline passed while reading the sources for named tags, so nothing was judged")];
    };
    let mut log: Vec<String> = Vec::new();
    let outcome = clean_podman(&RealPodman, &references, GRACE_SECONDS, now_seconds(), dry_run, &mut log);
    let verb: &str = if dry_run { "would remove" } else { "removed" };
    let mut lines: Vec<String> = vec![format!(
        "podman: {verb} {} images and {} volumes; {} images were gone with another one of this run; kept {} images and {} volumes",
        outcome.images.len(),
        outcome.volumes.len(),
        outcome.gone_with_another.len(),
        outcome.kept_images.len(),
        outcome.kept_volumes.len()
    )];
    for failure in &outcome.failures {
        lines.push(format!("  podman failure: {failure}"));
    }
    return lines;
}
