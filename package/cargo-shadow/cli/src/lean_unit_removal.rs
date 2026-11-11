//! What:
//! The removal of units and incremental directories from one Cargo profile directory,
//! shared by the sweep of older compilers and the size caps.
//! Why:
//! The protocol is the plan's shared one (`doc/planning/lean-build-dirs.md`,
//! section "Options"):
//! classify,
//! take the profile's build locks without waiting,
//! classify again under them,
//! scan every process,
//! move every path of each unit into one trash directory by atomic rename while the locks are held,
//! release the locks,
//! then delete the trash.
//! All paths of a unit are renamed under the one lock,
//! so no cooperating cargo sees half a unit.
//! Every removal logs its proof.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function applyRemovals(profile: string, planner: Planner, env: GuardEnvironment, now: number): ProfileOutcome
//! ```

/// The streaming log line.
use crate::lean_run_report::emit;
/// Paths.
use std::path::{Path, PathBuf};

/// The proof,
/// the identities,
/// the removal primitives,
/// the reader and the sizes.
use crate::lean_guard::{Guard, GuardEnvironment, Guarded, guard_profile};
/// Imports the names this file uses from `crate::lean_identity`.
use crate::lean_identity::{Deadline, Identity};
/// Imports the names this file uses from `crate::lean_removal`.
use crate::lean_removal::{Item, Refusal, TRASH_DIRECTORY_NAME, claim_into_trash, delete_claimed, empty_trash, item_of, modes_named};
/// Imports the names this file uses from `crate::lean_sizes`.
use crate::lean_sizes::add_tree_bytes;
/// Imports the names this file uses from `crate::lean_unit_reader`.
use crate::lean_unit_reader::{IncrementalDirectory, ProfileRead, ProfileUnits, Unit, read_profile};

/// What a planner decided to remove.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Removals {
    /// Whole units.
    pub units: Vec<Unit>,
    /// Incremental directories.
    pub incremental: Vec<IncrementalDirectory>,
    /// Why,
    /// for the log line.
    pub reason: String,
}

/// Something that decides,
/// from what a profile directory holds,
/// what to remove.
pub trait Planner {
    /// The decision for these units at this time.
    fn plan(&self, units: &ProfileUnits, now: i64) -> Removals;
}

/// What happened to one profile directory.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProfileOutcome {
    /// The directory.
    pub profile: PathBuf,
    /// Why nothing was done,
    /// when nothing was.
    pub skipped: Option<String>,
    /// Units removed,
    /// or in a dry run that would be.
    pub units: usize,
    /// Incremental directories removed or that would be.
    pub incremental: usize,
    /// Disk bytes freed,
    /// or in a dry run what removal would free.
    pub bytes: u64,
    /// Entries a process was using,
    /// kept.
    pub kept_in_use: usize,
    /// Refusals,
    /// with the reason of each.
    pub refusals: Vec<String>,
}

/// What:
/// `fn items_of_unit(unit: &Unit) -> Vec<Item>` is the identity of every path of a unit that still exists.
/// Why:
/// The identity is the one the removal checks again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itemsOfUnit = (unit: Unit): Item[] => unit.paths.flatMap((p) => { try { return [itemOf(p)]; } catch { return []; } });
/// ```
fn items_of_unit(unit: &Unit) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();
    for path in &unit.paths {
        if let Ok(item) = item_of(path) {
            items.push(item);
        }
    }
    return items;
}

/// What:
/// `fn bytes_of_removals(removals: &Removals, deadline: &Deadline) -> u64` measures what a dry run would free.
/// Why:
/// A report says how many bytes a run would free before it frees them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bytesOfRemovals(removals: Removals, deadline: Deadline): number
/// ```
fn bytes_of_removals(removals: &Removals, deadline: &Deadline) -> u64 {
    let mut seen: std::collections::HashSet<Identity> = std::collections::HashSet::new();
    let mut bytes: u64 = 0;
    for unit in &removals.units {
        for path in &unit.paths {
            bytes += add_tree_bytes(path, &mut seen, deadline).bytes;
        }
    }
    for directory in &removals.incremental {
        bytes += add_tree_bytes(&directory.path, &mut seen, deadline).bytes;
    }
    return bytes;
}

/// One path of a unit that was claimed into the trash,
/// and what the run says about it.
struct Claimed {
    /// Where it now is.
    target: PathBuf,
    /// What it was when it was judged.
    identity: Identity,
    /// What the removal line says before the deletion adds the mode clause,
    /// held by the first path of a unit;
    /// the other paths of the same unit are covered by that one line.
    line: Option<String>,
}

/// What:
/// `fn claim_all(guarded: &Guarded, removals: &Removals, trash: &Path, outcome: &mut ProfileOutcome, log: &mut Vec<String>) -> Vec<Claimed>`
/// moves every unit and incremental directory of the decision into the trash while the locks are held.
/// Why:
/// A unit with a path a process is using is kept whole.
/// A path that changed identity ends that unit and is reported.
/// The removal's proof line is written once the entry is deleted,
/// so it can say that a mode was changed,
/// as it says which locks were taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function claimAll(guarded: Guarded, removals: Removals, trash: string, outcome: ProfileOutcome, log: string[]): Claimed[]
/// ```
fn claim_all(guarded: &Guarded, removals: &Removals, trash: &Path, outcome: &mut ProfileOutcome, log: &mut Vec<String>) -> Vec<Claimed> {
    let mut claimed: Vec<Claimed> = Vec::new();
    let mut serial: u64 = 0;
    for unit in &removals.units {
        let items: Vec<Item> = items_of_unit(unit);
        let mut blocked: Option<String> = None;
        for item in &items {
            if let Some(used) = guarded.scan.blocks(&item.path) {
                blocked = Some(format!("pid {} uses {} as {}", used.pid, used.path.display(), used.how));
            }
        }
        if let Some(reason) = blocked {
            outcome.kept_in_use += 1;
            emit(log, format!("kept unit {}: {reason}", unit.label));
            continue;
        }
        let mut moved: Vec<(PathBuf, Identity)> = Vec::new();
        let mut failed: bool = false;
        for item in &items {
            serial += 1;
            match claim_into_trash(item, trash, serial) {
                Ok(target) => moved.push((target, item.identity)),
                Err(refusal) => {
                    outcome.refusals.push(format!("unit {}: {refusal}", unit.label));
                    failed = true;
                    break;
                }
            }
        }
        if !failed {
            outcome.units += 1;
            let description: String = format!(
                "removed unit {} ({} paths, compiler {}, last invoked {}): {}; proof: {}",
                unit.label,
                moved.len(),
                unit.compiler,
                unit.invoked,
                removals.reason,
                guarded.proof
            );
            match moved.split_first() {
                Some((first, rest)) => {
                    claimed.push(Claimed { target: first.0.clone(), identity: first.1, line: Some(description) });
                    for (target, identity) in rest {
                        claimed.push(Claimed { target: target.clone(), identity: *identity, line: None });
                    }
                }
                // A unit whose paths are all gone already holds nothing to delete, so its line has no deletion to add to.
                None => emit(log, description),
            }
        }
        // A unit that failed half way leaves what moved in the trash for the next run's sweep to finish,
        // and the refusal above already named it.
    }
    for directory in &removals.incremental {
        if let Some(used) = guarded.scan.blocks(&directory.path) {
            outcome.kept_in_use += 1;
            emit(log, format!("kept incremental directory {}: pid {} uses {} as {}", directory.path.display(), used.pid, used.path.display(), used.how));
            continue;
        }
        serial += 1;
        match item_of(&directory.path) {
            Ok(item) => match claim_into_trash(&item, trash, serial) {
                Ok(target) => {
                    outcome.incremental += 1;
                    claimed.push(Claimed {
                        target,
                        identity: item.identity,
                        line: Some(format!("removed incremental directory {} (modified {}): {}; proof: {}", directory.path.display(), directory.modified, removals.reason, guarded.proof)),
                    });
                }
                Err(refusal) => outcome.refusals.push(format!("incremental {}: {refusal}", directory.path.display())),
            },
            Err(error) => outcome.refusals.push(format!("incremental {}: {error}", directory.path.display())),
        }
    }
    return claimed;
}

/// What:
/// `pub fn apply_removals(profile: &Path, planner: &dyn Planner, environment: &GuardEnvironment, now: i64, dry_run: bool, deadline: &Deadline, log: &mut Vec<String>) -> ProfileOutcome`
/// runs the shared protocol on one profile directory.
/// Why:
/// It is the single path by which units leave a profile directory,
/// for the sweep of older compilers and for the size caps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function applyRemovals(profile: string, planner: Planner, env: GuardEnvironment, now: number, dryRun: boolean, deadline: Deadline, log: string[]): ProfileOutcome
/// ```
pub fn apply_removals(
    profile: &Path,
    planner: &dyn Planner,
    environment: &GuardEnvironment,
    now: i64,
    dry_run: bool,
    deadline: &Deadline,
    log: &mut Vec<String>,
) -> ProfileOutcome {
    let mut outcome: ProfileOutcome = ProfileOutcome { profile: profile.to_path_buf(), ..ProfileOutcome::default() };
    let first: Removals = match read_profile(profile) {
        ProfileRead::Known(units) => planner.plan(&units, now),
        ProfileRead::Unknown(reason) => {
            outcome.skipped = Some(format!("unknown layout: {reason}"));
            return outcome;
        }
    };
    if first.units.is_empty() && first.incremental.is_empty() {
        return outcome;
    }
    if dry_run {
        outcome.units = first.units.len();
        outcome.incremental = first.incremental.len();
        outcome.bytes = bytes_of_removals(&first, deadline);
        return outcome;
    }
    let guarded: Guarded = match guard_profile(profile, environment) {
        Guard::Ready(ready) => ready,
        Guard::Skip(reason) => {
            outcome.skipped = Some(reason);
            return outcome;
        }
    };
    // Classified again under the locks: what was judged before them may have changed.
    let second: Removals = match read_profile(profile) {
        ProfileRead::Known(units) => planner.plan(&units, now),
        ProfileRead::Unknown(reason) => {
            outcome.skipped = Some(format!("unknown layout under the locks: {reason}"));
            return outcome;
        }
    };
    let trash: PathBuf = profile.join(TRASH_DIRECTORY_NAME);
    let sweep = empty_trash(&trash);
    if !sweep.modes_added.is_empty() {
        emit(log, format!("finished an earlier run's deletion in {} ({} disk bytes){};", trash.display(), sweep.bytes, modes_named(&sweep.modes_added)));
    }
    outcome.refusals.extend(sweep.failures);
    let claimed: Vec<Claimed> = claim_all(&guarded, &second, &trash, &mut outcome, log);
    drop(guarded);
    for entry in claimed {
        match delete_claimed(&entry.target, entry.identity) {
            Ok(deleted) => {
                outcome.bytes += deleted.bytes;
                if let Some(line) = entry.line {
                    emit(log, format!("{line}{}", modes_named(&deleted.modes_added)));
                }
            }
            Err(Refusal::PartlyRemoved { deleted, .. }) => {
                // The unit is half gone: say so, and let the summary name what stayed and why.
                outcome.bytes += deleted.bytes;
                let modes: String = modes_named(&deleted.modes_added);
                match entry.line {
                    Some(line) => emit(log, format!("{line}{modes}")),
                    None => emit(log, format!("partly removed {}{modes}", entry.target.display())),
                }
                outcome.refusals.push(format!(
                    "{} was only partly removed: {} disk bytes are gone and {} part(s) stay, each because: {}",
                    entry.target.display(),
                    deleted.bytes,
                    deleted.remaining.len(),
                    deleted.remaining.join("; ")
                ));
            }
            Err(refusal) => outcome.refusals.push(refusal.to_string()),
        }
    }
    let _ = std::fs::remove_dir(&trash);
    return outcome;
}
