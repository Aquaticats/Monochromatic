//! What:
//!  Installation of one validated stage into a newly registered worktree:
//!  preflight of every existing destination entry,
//! batched creation with its durable records,
//!  mode application and rollback of this installation's own paths when it fails.
//! Why:
//!  A destination that already holds an identical entry is accepted,
//!  and a differing entry is never overwritten,
//! so the copy cannot destroy user state.
//!  Recording each batch before and after it runs bounds the work an interrupted run leaves unrecorded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const copied = await installSnapshot({ snapshot, destinationRoot, journalState, priorCreations });
//! ```

use crate::side_by_side_on_the_runtime::run_all_side_by_side_on_the_runtime;
use crate::worktree_copy_compare::{entry_matches, lstat_or_absent};
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause};
use crate::parallel_budget::{HostResources, sliced_workers};
use crate::worktree_copy_stage_leaf_copying::PARALLEL_COPY_MAX_WORKERS;
use crate::worktree_copy_install_entry::{
    InstalledPath, capture_installed_path, create_selected_entry, ensure_parents_remembering,
};
use crate::worktree_copy_manifest::{EntryKind, WorktreeCopyEntry, apply_entry_modes};
use crate::worktree_copy_rollback::rollback_created;
use crate::worktree_copy_selection::{filesystem_path, shown_repository_path};
use crate::json_record::quote;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// What:
///  The number of manifest entries claimed by one durable intent record and settled by one creation record.
/// Why:
///   Bounds both the number of journal appends and the work an interrupted owner can leave unrecorded.
/// Measured with
///  `install_snapshot` and a recorder that appends each batch to a file and syncs it twice,
/// on a tree of 22,717 entries (2026-11-03,
///  five runs each in alternating orders,
/// other sessions running,
/// a debug build):
/// medians 6,542,
/// 2,265,
/// 983,
/// 746,
/// 688 and 663 ms
///  for batches of 64,
/// 256,
/// 1,024,
/// 4,096,
/// 16,384 and one batch,
/// so a batch costs about 16 ms of syncs and 4,096 entries add 12
///  percent to the install and 16,384 add 4 percent.
/// The real journal's append was not run;
/// its cost per batch may differ.
/// Kept at
///  4096 as the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// option A).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INSTALL_BATCH_ENTRIES = 4096;
/// ```
pub const INSTALL_BATCH_ENTRIES: usize = 4096;


/// What:
///  The durable record of an installation in progress.
/// Why:
///   The installation reports every claim before it creates a path and every proven creation after,
///  so the journal slice can persist both without this module knowing the file format.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordIntents({ state, relativePaths }); recordCreations({ state, entries });
/// ```
pub trait InstallRecorder {
    /// Records the absent paths one batch is about to create,
    /// before any of them exists.
    fn record_intents(&mut self, relative_paths: &[Vec<u8>]) -> Result<(), WorktreeCopyError>;

    /// Records the identities of the paths one batch created.
    fn record_creations(&mut self, entries: &[InstalledPath]) -> Result<(), WorktreeCopyError>;
}

/// What:
///  A recorder that writes nothing,
/// for installations whose caller keeps no durable state.
/// Why:
///   Such an installation is not crash-safe;
/// it exists so the installer can be exercised before the journal lands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no journal state
/// ```
#[derive(Debug, Default)]
pub struct NoRecord;

impl InstallRecorder for NoRecord {
    /// Accepts the intent without storing it.
    fn record_intents(&mut self, _relative_paths: &[Vec<u8>]) -> Result<(), WorktreeCopyError> {
        return Ok(());
    }

    /// Accepts the creations without storing them.
    fn record_creations(&mut self, _entries: &[InstalledPath]) -> Result<(), WorktreeCopyError> {
        return Ok(());
    }
}

/// What:
///  The selected paths absent from the destination,
/// after every existing entry has been proven identical.
/// Why:
///   An existing entry that differs stops the installation before any mutation,
/// and an identical one is accepted without being overwritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function preflightDestination({ snapshot, destinationRoot, recorded }): Promise<ReadonlySet<string>>;
/// ```
pub fn preflight_destination(
    stage_root: &Path,
    destination_root: &Path,
    entries: &[WorktreeCopyEntry],
    claimed: &BTreeSet<Vec<u8>>,
) -> Result<BTreeSet<Vec<u8>>, WorktreeCopyError> {
    let mut absent: BTreeSet<Vec<u8>> = BTreeSet::new();
    for entry in entries {
        let destination_path: PathBuf =
            filesystem_path(destination_root, entry.relative_path.as_slice())?;
        // `match` on the existing entry: an absent path is recorded, a claimed directory is accepted, anything else must be identical.
        match lstat_or_absent(destination_path.as_path())? {
            None => {
                absent.insert(entry.relative_path.clone());
            }
            Some(metadata) => {
                let claimed_directory: bool = entry.kind == EntryKind::Directory
                    && metadata.is_dir()
                    && claimed.contains(&entry.relative_path);
                if claimed_directory {
                    continue;
                }
                if !entry_matches(stage_root, destination_root, entry)? {
                    let message: String = format!(
                        "monochromatic-git-policy-cli: ignored-state copy would overwrite differing destination entry {} in {}.",
                        quote(shown_repository_path(entry.relative_path.as_slice()).as_str()),
                        quote(destination_root.to_string_lossy().as_ref())
                    );
                    return Err(failure_with_cause(message.as_str(), None));
                }
            }
        }
    }
    return Ok(absent);
}

/// What:
///  Creates one batch's absent entries in manifest order,
/// recording the claim before and the identities after.
/// Why:
///   The claim is durable before the first path exists,
/// so an interruption leaves a record of every path that might be partial.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installBatch({ snapshot, destinationRoot, batch, absent, created, journalState }): Promise<void>;
/// ```
fn install_batch(
    stage_root: &Path,
    destination_root: &Path,
    batch: &[WorktreeCopyEntry],
    absent: &BTreeSet<Vec<u8>>,
    created: &mut Vec<InstalledPath>,
    recorder: &mut dyn InstallRecorder,
    known: &mut std::collections::HashSet<Vec<u8>>,
) -> Result<(), WorktreeCopyError> {
    let mut claims: Vec<Vec<u8>> = Vec::new();
    for entry in batch {
        if absent.contains(&entry.relative_path) {
            claims.push(entry.relative_path.clone());
        }
    }
    recorder.record_intents(claims.as_slice())?;
    let batch_start: usize = created.len();
    // Directories and the scaffolding above every entry first, in manifest order, so each leaf finds its parent.
    let mut leaves: Vec<usize> = Vec::new();
    for (at, entry) in batch.iter().enumerate() {
        ensure_parents_remembering(destination_root, entry, created, known)?;
        if entry.kind != EntryKind::Directory {
            leaves.push(at);
            continue;
        }
        let destination_path: PathBuf =
            filesystem_path(destination_root, entry.relative_path.as_slice())?;
        // A path that appeared since preflight is never created over; the next settle step decides what it is.
        if lstat_or_absent(destination_path.as_path())?.is_none() {
            create_selected_entry(stage_root, destination_root, entry)?;
            created.push(capture_installed_path(
                destination_path.as_path(),
                entry.relative_path.as_slice(),
                true,
            )?);
        }
        known.insert(entry.relative_path.clone());
    }
    install_leaves(stage_root, destination_root, batch, leaves.as_slice(), created)?;
    recorder.record_creations(&created[batch_start..])?;
    return Ok(());
}

/// What:
///  The leaves below which a batch's files and links are created by one worker:
/// `PARALLEL_BREAK_EVEN_ENTRIES`.
/// Why:
///   A large batch spends its time in independent link calls;
/// a very small one is faster without the thread start.
/// Measured with
///  `install_snapshot` itself forced to 1,
/// 2,
/// 4 and 8 workers on reflinked trees of this repository (2026-11-03,
/// five runs each in
///  alternating orders,
/// other sessions running,
/// a debug build):
/// 8 workers took 1.3 ms against 1.6 ms for 30 entries,
/// 8.9 ms
///  against 14.8 ms for 296,
/// 49.2 ms against 107.7 ms for 2,119,
/// and 244.9 ms against 500.2 ms for 9,301,
/// so the old threshold of
///  1024 left batches of up to 1,023 leaves on one thread for up to 50 ms more.
/// Picked by the coordinator under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// option A).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARALLEL_INSTALL_MIN_LEAVES = PARALLEL_BREAK_EVEN_ENTRIES;
/// ```
const PARALLEL_INSTALL_MIN_LEAVES: usize = crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES;

/// What:
///  Install one staged entry unless something is already at its destination.
/// Why:
///   A path that appeared since the preflight is never created over;
/// the next settle step decides what it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (exists(destination)) return undefined; createSelectedEntry(...); return captureInstalledPath(destination);
/// ```
fn install_one(stage_root: &Path, destination_root: &Path, entry: &WorktreeCopyEntry) -> Result<Option<InstalledPath>, WorktreeCopyError> {
    let destination_path: PathBuf = filesystem_path(destination_root, entry.relative_path.as_slice())?;
    if lstat_or_absent(destination_path.as_path())?.is_some() {
        return Ok(None);
    }
    create_selected_entry(stage_root, destination_root, entry)?;
    return capture_installed_path(destination_path.as_path(), entry.relative_path.as_slice(), true).map(Some);
}

/// What:
///  Creates one slice of files and links,
/// stopping at the first failure,
/// and returns what it created with that failure.
/// Why:
///   Rollback must know every path that exists,
/// including those created before a failure,
/// so the slice never discards
///  its record when it stops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installSlice([stageRoot, destinationRoot, slice]): Promise<{ created: InstalledPath[]; failure?: WorktreeCopyError }>;
/// ```
fn install_slice(
    (stage_root, destination_root, slice): (&Path, &Path, &[&WorktreeCopyEntry]),
) -> (Vec<InstalledPath>, Option<WorktreeCopyError>) {
    let mut made: Vec<InstalledPath> = Vec::new();
    for entry in slice {
        let outcome: Result<Option<InstalledPath>, WorktreeCopyError> = install_one(stage_root, destination_root, entry);
        match outcome {
            Ok(Some(installed)) => made.push(installed),
            Ok(None) => {}
            Err(error) => return (made, Some(error)),
        }
    }
    return (made, None);
}

/// What:
///  One slice of a batch's leaves as the state of one job:
/// the batch,
/// the positions of this slice in it,
/// and the two roots.
/// Why:
///   A job of the blocking pool owns what it reads,
/// so the batch travels as one share of it and each job takes its own positions,
///  and a job of the pool is a function and its state,
/// never a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ASliceToInstall = [entries: readonly WorktreeCopyEntry[], stage: string, destination: string, positions: number[]];
/// ```
type ASliceToInstall = (Arc<[WorktreeCopyEntry]>, PathBuf, PathBuf, Vec<usize>);

/// What:
///  Creates the leaves of one slice,
/// chosen from the shared batch by their positions.
/// Why:
///   The positions are what a job of the pool can own without a copy of the manifest,
/// and the slice keeps the batch's own order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const installASliceOf = ([entries, stage, destination, positions]) => installSlice([stage, destination, positions.map((at) => entries[at])]);
/// ```
fn install_a_slice_of((entries, stage_root, destination_root, positions): ASliceToInstall) -> (Vec<InstalledPath>, Option<WorktreeCopyError>) {
    let mut slice: Vec<&WorktreeCopyEntry> = Vec::with_capacity(positions.len());
    for at in positions {
        let Some(entry) = entries.get(at) else {
            continue;
        };
        slice.push(entry);
    }
    return install_slice((stage_root.as_path(), destination_root.as_path(), slice.as_slice()));
}

/// What:
///  Creates the files and links of one batch,
/// in parallel slices when the batch is large,
/// every slice a job of the process's blocking pool.
/// Why:
///   Their parents exist already,
/// so the creations are independent;
/// every created path is recorded for rollback
///  before the first failure is returned.
/// A creation waits on storage,
/// so it runs on the blocking pool and never on a worker of the runtime that polls tasks,
/// and every slice takes a place of the process's one bound on blocking jobs before its work runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installLeaves({ stageRoot, destinationRoot, entries, leaves, created }): Promise<void>;
/// ```
fn install_leaves(
    stage_root: &Path,
    destination_root: &Path,
    entries: &[WorktreeCopyEntry],
    leaves: &[usize],
    created: &mut Vec<InstalledPath>,
) -> Result<(), WorktreeCopyError> {
    return match crate::lazy_runtime::run_on_runtime(install_leaves_awaiting(stage_root, destination_root, entries, leaves)) {
        Ok((made, stopped)) => {
            // Rollback must know every path that exists, including those created before a failure.
            created.extend(made);
            match stopped {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
        // Nothing more was created, and the reason the runtime could not run the slices is why.
        Err(refusal) => Err(failure_with_cause(
            format!("monochromatic-git-policy-cli: the files and links of one batch were not installed ({refusal}).").as_str(),
            None,
        )),
    };
}

/// What:
///  `install_leaves` on the process's runtime:
/// every slice of the batch's leaves is one job of the blocking pool,
/// and every path created is returned in slice order with the first failure,
/// if any.
/// Why:
///   A caller that is itself a task of the runtime cannot enter it,
/// so it awaits this face instead;
/// the created paths come back even when a slice failed,
/// because rollback must know them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installLeavesAwaiting({ stageRoot, destinationRoot, entries, leaves }): Promise<[InstalledPath[], WorktreeCopyError?]>;
/// ```
async fn install_leaves_awaiting(
    stage_root: &Path,
    destination_root: &Path,
    entries: &[WorktreeCopyEntry],
    leaves: &[usize],
) -> (Vec<InstalledPath>, Option<WorktreeCopyError>) {
    let workers: usize = if leaves.len() < PARALLEL_INSTALL_MIN_LEAVES {
        1
    } else {
        sliced_workers(leaves.len(), PARALLEL_COPY_MAX_WORKERS, &HostResources::read())
    };
    let slice_length: usize = leaves.len().div_ceil(workers).max(1);
    let shared: Arc<[WorktreeCopyEntry]> = Arc::from(entries);
    let mut states: Vec<ASliceToInstall> = Vec::new();
    for slice in leaves.chunks(slice_length) {
        states.push((
            Arc::clone(&shared),
            stage_root.to_path_buf(),
            destination_root.to_path_buf(),
            slice.to_vec(),
        ));
    }
    let mut results: Vec<(Vec<InstalledPath>, Option<WorktreeCopyError>)> = Vec::new();
    for answer in run_all_side_by_side_on_the_runtime(states, install_a_slice_of).await {
        results.push(match answer {
            Ok(done) => done,
            Err(crate::blocking_job::BlockingJobError::Panicked) => (Vec::new(), Some(failure_with_cause("monochromatic-git-policy-cli: an install worker panicked.", None))),
            Err(refused) => (Vec::new(), Some(failure_with_cause(format!("monochromatic-git-policy-cli: an install worker did not run ({refused}).").as_str(), None))),
        });
    }
    let mut failure: Option<WorktreeCopyError> = None;
    let mut made: Vec<InstalledPath> = Vec::new();
    for (created, stopped) in results {
        made.extend(created);
        failure = failure.or(stopped);
    }
    return (made, failure);
}

/// What:
///  Installs a whole stage into the destination,
/// or rolls back its own paths and reports what it kept.
/// Why:
///   A failed installation ends with the destination as it was before,
/// apart from paths the rollback could not prove were its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installSnapshot({ snapshot, destinationRoot, journalState, priorCreations }): Promise<number>;
/// ```
pub fn install_snapshot(
    stage_root: &Path,
    destination_root: &Path,
    entries: &[WorktreeCopyEntry],
    claimed: &BTreeSet<Vec<u8>>,
    prior: &[InstalledPath],
    recorder: &mut dyn InstallRecorder,
) -> Result<usize, WorktreeCopyError> {
    let mut created: Vec<InstalledPath> = prior.to_vec();
    match install_all(
        stage_root,
        destination_root,
        entries,
        claimed,
        &mut created,
        recorder,
    ) {
        Ok(copied) => return Ok(copied),
        Err(error) => {
            let retained: Vec<String> =
                rollback_created(stage_root, destination_root, entries, created.as_slice())?;
            let suffix: String = if retained.is_empty() {
                String::new()
            } else {
                let names: Vec<String> = retained.iter().map(String::as_str).map(quote).collect();
                format!(" Rollback retained: {}.", names.join(", "))
            };
            let message: String = format!(
                "monochromatic-git-policy-cli: ignored-state installation failed for {}.{suffix}",
                quote(destination_root.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error.0)));
        }
    }
}

/// What:
///  The installation steps that run before rollback can apply:
/// preflight,
/// batches,
/// and modes.
/// Why:
///   Separating them lets one rollback cover every failure after the first mutation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const absent = await preflightDestination(...); for (const batch of installBatches(...)) await installBatch(...); await applyEntryModes(...);
/// ```
fn install_all(
    stage_root: &Path,
    destination_root: &Path,
    entries: &[WorktreeCopyEntry],
    claimed: &BTreeSet<Vec<u8>>,
    created: &mut Vec<InstalledPath>,
    recorder: &mut dyn InstallRecorder,
) -> Result<usize, WorktreeCopyError> {
    let started: std::time::Instant = std::time::Instant::now();
    let absent: BTreeSet<Vec<u8>> =
        preflight_destination(stage_root, destination_root, entries, claimed)?;
    let preflighted: std::time::Duration = started.elapsed();
    let mut known: std::collections::HashSet<Vec<u8>> = std::collections::HashSet::new();
    for batch in entries.chunks(INSTALL_BATCH_ENTRIES) {
        install_batch(
            stage_root,
            destination_root,
            batch,
            &absent,
            created,
            recorder,
            &mut known,
        )?;
    }
    let created_after: std::time::Duration = started.elapsed();
    apply_entry_modes(destination_root, entries)?;
    crate::diagnostic_log::debug(
        "worktreeCopyInstall",
        format!(
            "installed {} entries: preflight {} ms, create and record {} ms, modes {} ms",
            entries.len(),
            preflighted.as_millis(),
            (created_after - preflighted).as_millis(),
            (started.elapsed() - created_after).as_millis()
        )
        .as_str(),
    );
    let mut selected: usize = 0;
    for installed in created.iter() {
        if installed.selected {
            selected += 1;
        }
    }
    return Ok(selected);
}

/// Installation controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_install_tests.rs"]
mod tests;
