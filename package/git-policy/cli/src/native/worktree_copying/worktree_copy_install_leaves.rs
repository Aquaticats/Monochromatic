//! What:
//!  One batch's files and links created in slices:
//! the entries below which one worker is faster,
//! one entry created when nothing is already at its destination,
//! the state one job of the blocking pool owns,
//! and the slices that run side by side.
//! Why:
//!   The creations are independent,
//! so a large batch is split into slices of the blocking pool,
//! and a very small one is faster without the thread start.
//! Every slice takes a place of the process's one bound on blocking jobs before its work runs,
//! and no creation ever waits on a worker of the runtime that polls tasks.
//! It lives beside the installation because that file was over the line limit before this change
//! (`worktree_copy_install.rs`,
//! which re-exports nothing of it:
//! the batch that runs them is the only reader).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await installLeaves({ stageRoot, destinationRoot, entries, leaves, created });
//! ```

use crate::parallel_budget::{
    HostResources,
    sliced_workers,
};
use crate::side_by_side_on_the_runtime::run_all_side_by_side_on_the_runtime;
use crate::worktree_copy_compare::lstat_or_absent;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    failure_with_cause,
};
use crate::worktree_copy_install_entry::{
    InstalledPath,
    capture_installed_path,
    create_selected_entry,
};
use crate::worktree_copy_manifest::WorktreeCopyEntry;
use crate::worktree_copy_selection::filesystem_path;
use crate::worktree_copy_stage_leaf_copying::PARALLEL_COPY_MAX_WORKERS;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;
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
pub(crate) const PARALLEL_INSTALL_MIN_LEAVES: usize =
    crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES;

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
fn install_one(
    stage_root: &Path,
    destination_root: &Path,
    entry: &WorktreeCopyEntry,
) -> Result<Option<InstalledPath>, WorktreeCopyError> {
    let destination_path: PathBuf =
        filesystem_path(destination_root, entry.relative_path.as_slice())?;
    if lstat_or_absent(destination_path.as_path())?.is_some() {
        return Ok(None);
    }
    create_selected_entry(stage_root, destination_root, entry)?;
    return capture_installed_path(
        destination_path.as_path(),
        entry.relative_path.as_slice(),
        true,
    )
    .map(Some);
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
    (stage_root, destination_root, slice): (&Path, &Path, &[&WorktreeCopyEntry])
) -> (Vec<InstalledPath>, Option<WorktreeCopyError>) {
    let mut made: Vec<InstalledPath> = Vec::new();
    for entry in slice {
        let outcome: Result<Option<InstalledPath>, WorktreeCopyError> =
            install_one(stage_root, destination_root, entry);
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
fn install_a_slice_of(
    (entries, stage_root, destination_root, positions): ASliceToInstall
) -> (Vec<InstalledPath>, Option<WorktreeCopyError>) {
    let mut slice: Vec<&WorktreeCopyEntry> = Vec::with_capacity(positions.len());
    for at in positions {
        let Some(entry) = entries.get(at) else {
            continue;
        };
        slice.push(entry);
    }
    return install_slice((
        stage_root.as_path(),
        destination_root.as_path(),
        slice.as_slice(),
    ));
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
pub(crate) fn install_leaves(
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
        sliced_workers(
            leaves.len(),
            PARALLEL_COPY_MAX_WORKERS,
            &HostResources::read(),
        )
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
            Err(crate::blocking_job::BlockingJobError::Panicked) => (
                Vec::new(),
                Some(failure_with_cause(
                    "monochromatic-git-policy-cli: an install worker panicked.",
                    None,
                )),
            ),
            Err(refused) => (
                Vec::new(),
                Some(failure_with_cause(
                    format!(
                        "monochromatic-git-policy-cli: an install worker did not run ({refused})."
                    )
                    .as_str(),
                    None,
                )),
            ),
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
