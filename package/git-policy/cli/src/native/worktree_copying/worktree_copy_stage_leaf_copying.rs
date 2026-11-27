//! What:
//!  The copying of a staged snapshot's files and links from the source into the stage,
//! one worker per slice of the manifest.
//! Why:
//!  The stage must hold every byte before the final comparison,
//! and a clone or a copy of one file is independent of the
//!  next,
//! so a large manifest is split in slices that run side by side,
//! each stopping when another has failed.
//! This file splits the manifest and gathers what the slices left;
//! what one file,
//! one link and one entry of a slice are made of lives beside it,
//! and the names the rest of the crate reaches are re-exported here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stamps = await copyManifestEntries(sourceRoot, stageRoot, entries); // one stamp per cloned file
//! ```

use crate::parallel_budget::{
    HostResources,
    sliced_workers,
};
use crate::side_by_side_on_the_runtime::run_all_side_by_side_on_the_runtime;
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    raw_failure,
};
use crate::worktree_copy_manifest::{
    EntryKind,
    FileStamp,
    WorktreeCopyEntry,
};
use crate::worktree_copy_selection::filesystem_path;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
// What: the names the rest of the crate reaches through this file, defined in the two files beside it.
// Why: a split moves code, never a caller, so every path outside this folder stays what it was.
pub use crate::worktree_copy_stage_entry_copying::CopiedLeaf;
pub use crate::worktree_copy_stage_entry_copying::copy_one_leaf;
pub use crate::worktree_copy_stage_file_copying::copy_file_exclusive;
pub use crate::worktree_copy_stage_file_copying::copy_file_exclusive_with;
pub use crate::worktree_copy_stage_file_copying::create_private_tree;
pub(crate) use crate::worktree_copy_stage_file_copying::stamp_is_settled;
pub use crate::worktree_copy_stage_file_copying::try_clone;

// What: the four names only a control asks for, so a release build holds no copy of them.
// Why: the controls that pin a clock or read a settled stamp are the only readers outside this folder.
#[cfg(test)]
pub(crate) use crate::worktree_copy_stage_file_copying::ClockPin;
#[cfg(test)]
pub(crate) use crate::worktree_copy_stage_file_copying::RECENT_CHANGE_NANOS;
#[cfg(test)]
pub(crate) use crate::worktree_copy_stage_file_copying::clock_reads;
#[cfg(test)]
pub(crate) use crate::worktree_copy_stage_file_copying::stamp_nanos;

/// What:
///  The entries below which a copy stays on one thread:
/// `PARALLEL_BREAK_EVEN_ENTRIES`.
/// Why:
///   Starting threads costs more than a very small copy.
/// Measured with `copy_manifest_entries` itself forced to 1,
/// 2,
/// 4 and 8
///  workers on reflinked trees of this repository (2026-11-03,
/// five runs each in alternating orders,
/// other sessions running,
/// a
///  debug build):
/// 8 workers took 0.7 ms against 1.1 ms for 30 entries,
/// 1.9 ms against 3.6 ms for 90,
/// 5.9 ms against 13.4 ms for
///  296,
/// 25.4 ms against 91.4 ms for 2,119,
/// 57.3 ms against 202.6 ms for 4,722,
/// and 104.4 ms against 397.6 ms for 9,301,
/// so the
///  old threshold of 4096 kept copies of up to 4,095 entries on one thread for up to 145 ms more.
/// Picked by the coordinator under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// option A).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARALLEL_COPY_MIN_ENTRIES = PARALLEL_BREAK_EVEN_ENTRIES;
/// ```
pub const PARALLEL_COPY_MIN_ENTRIES: usize = crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES;

/// What:
///  The most threads one copy starts.
/// Why:
///   Cloning is a metadata operation bound by the filesystem's locks more than by CPU,
/// and other sessions share this host.
///  Measured with `clone_tree` (`package_repair_tree_clone.rs`,
/// the same `copy_file_exclusive` primitive as the copy stage) on trees of reflinked tracked files of this repository,
/// medians of three runs,
/// 28 threads,
/// other sessions running,
/// 2026-11-03:
/// at 22,718 entries 1,
/// 2,
/// 4,
/// 8 and 16 workers took 1,549,
/// 880,
/// 555,
/// 397 and 415 ms (three runs each,
/// the
///  spread of 16 workers 383 to 444 ms),
/// at 9,301 entries 619,
/// 363,
/// 236,
/// 183 and 194 ms,
/// and at 4,723 entries 330,
/// 190,
/// 124,
/// 99
///  and 106 ms,
/// so more than eight workers were never faster.
/// Kept at eight;
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARALLEL_COPY_MAX_WORKERS = 8;
/// ```
pub(crate) const PARALLEL_COPY_MAX_WORKERS: usize = 8;

/// What:
///  How many threads a copy of this many entries uses:
/// one below the threshold,
/// else the host's budget (the CPUs the process may
///  use less a margin,
/// and the free memory),
/// capped.
/// Why:
///   The bound follows the CPUs the process is allowed (a container's quota included),
/// never the whole host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries < MIN ? 1 : Math.min(availableParallelism(), MAX)
/// ```
pub fn copy_worker_count(entries: usize) -> usize {
    return copy_worker_count_on(&HostResources::read(), entries);
}

/// `copy_worker_count` with the host's resources chosen by the caller,
/// so a control states a host of any size.
pub fn copy_worker_count_on(
    host: &HostResources,
    entries: usize,
) -> usize {
    if entries < PARALLEL_COPY_MIN_ENTRIES {
        return 1;
    }
    return sliced_workers(entries, PARALLEL_COPY_MAX_WORKERS, host);
}

/// What:
///  Copies the files and links of the manifest entries at `positions` into the stage,
/// recording each cloned file's stamp in a list of the slice's own.
/// Why:
///   Each entry is recreated from the source's no-follow view,
/// so no symbolic link is ever followed;
///  directories exist already,
/// and a parent is created only when the entry's parent changes.
///  The stamps come back in the order of the positions rather than into one shared list,
/// because a job of the blocking pool owns what it reads and writes;
/// the caller places them by the slice each position belongs to,
/// so a slice that stopped early cannot shift another slice's stamps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyLeaves({ sourceRoot, stageRoot, entries, positions, failed }): (FileStamp | undefined)[];
/// ```
fn copy_leaves(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
    positions: &[usize],
    failed: &AtomicBool,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
    ceiling: &CopyCeiling,
) -> Result<SliceCopy, WorktreeCopyError> {
    let mut last_parent: Option<PathBuf> = None;
    let mut stamps: Vec<Option<FileStamp>> = Vec::with_capacity(positions.len());
    let mut gone: Vec<Vec<u8>> = Vec::new();
    for at in positions {
        let Some(entry) = entries.get(*at) else {
            continue;
        };
        if failed.load(Ordering::Relaxed) {
            return Ok(SliceCopy {
                stamps,
                gone,
            });
        }
        // A command that finishes a copy it did not start stops at its ceiling, entry by entry, so no checkout's
        //  size decides how long it waits.
        if ceiling.reached() {
            return Err(ceiling.stopped_error());
        }
        if entry.kind == EntryKind::Directory {
            stamps.push(None);
            continue;
        }
        let stage_path: PathBuf = filesystem_path(stage_root, entry.relative_path.as_slice())?;
        let parent: PathBuf = parent_of(stage_path.as_path());
        if last_parent.as_ref() != Some(&parent) {
            create_private_tree(parent.as_path())?;
            last_parent = Some(parent);
        }
        match copy_one_leaf(source_root, stage_path.as_path(), entry, clone)? {
            CopiedLeaf::Staged(stamp) => stamps.push(stamp),
            // The source no longer holds it, so there is nothing to copy and no reason to fail the add.
            CopiedLeaf::Gone => {
                stamps.push(None);
                gone.push(entry.relative_path.clone());
            }
        }
    }
    return Ok(SliceCopy {
        stamps,
        gone,
    });
}
/// What:
///  What one slice of the manifest left in the stage,
/// and what it found gone.
/// Why:
///   A job of the blocking pool owns what it writes,
/// so its stamps come back in its own list,
/// which the
///  caller places by the slice each position belongs to;
/// an entry the source no longer holds has no stamp and no
///  stage file,
/// so the job names it instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SliceCopy = { stamps: (FileStamp | undefined)[]; gone: Uint8Array[] };
/// ```
struct SliceCopy {
    /// The stamp of every position of the slice,
    /// in position order.
    stamps: Vec<Option<FileStamp>>,
    /// The entries the source no longer held when their turn came,
    /// as the bytes of each name.
    gone: Vec<Vec<u8>>,
}

/// What:
///  What one copy of the manifest left in the stage and what it found gone.
/// Why:
///   A stamp belongs to the entry it was taken for,
/// and an entry the source no longer holds has no stamp
///  and no stage file;
/// the caller has to know both to compare the stage with the source and to install
///  only what exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopiedManifest = { stamps: (FileStamp | undefined)[]; gone: Uint8Array[] };
/// ```
pub struct CopiedManifest {
    /// The stamp of each entry,
    /// in manifest order;
    /// `None` when the bytes were copied rather than cloned,
    /// and for every entry the source no longer held.
    pub stamps: Vec<Option<FileStamp>>,
    /// The manifest entries the source no longer held when their turn came,
    /// in manifest order,
    /// as the bytes of each name.
    pub gone: Vec<Vec<u8>>,
}

/// What:
///  One slice of the manifest as the state of one job:
/// the entries,
/// the positions of this slice,
/// the two roots,
/// the flag that stops the other workers,
/// and the clone request.
/// Why:
///   A job of the blocking pool owns what it reads,
/// so the manifest the caller lent is shared once here
///  and each job takes its own share of it with the positions it copies,
/// which no job has to copy an entry for.
/// The budget runner takes a function and the state it reads,
/// never a closure,
/// so the slice travels as values the job owns and
///  may be taken again where no thread can start;
/// the clone request travels the same way,
/// so a control can place a write between one entry's copy and the next one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ASliceToCopy = [entries: readonly WorktreeCopyEntry[], source: string, stage: string, positions: number[], failed: boolean, clone: CloneRequest];
/// ```
type ASliceToCopy = (
    Arc<[WorktreeCopyEntry]>,
    PathBuf,
    PathBuf,
    Vec<usize>,
    Arc<AtomicBool>,
    fn(&std::fs::File, &std::fs::File) -> bool,
    Arc<CopyCeiling>,
);

/// What:
///  Copy one slice of leaf entries on a worker,
/// and tell the other workers to stop when it failed.
/// Why:
///   The workers share the flag only to end early;
/// the first failure is each worker's own result,
/// and the caller reports the
///  first in slice order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copySliceAndFlagFailure([entries, sourceRoot, stageRoot, positions, failed]) { const outcome = copyLeaves(...); if (outcome.isErr()) failed.set(true); return outcome; }
/// ```
fn copy_slice_and_flag_failure(
    (entries, source_root, stage_root, positions, failed, clone, ceiling): ASliceToCopy
) -> Result<SliceCopy, WorktreeCopyError> {
    let outcome: Result<SliceCopy, WorktreeCopyError> = copy_leaves(
        source_root.as_path(),
        stage_root.as_path(),
        entries.as_ref(),
        positions.as_slice(),
        failed.as_ref(),
        clone,
        ceiling.as_ref(),
    );
    if outcome.is_err() {
        failed.store(true, Ordering::Relaxed);
    }
    return outcome;
}

/// What:
///  Copies one manifest's entries into the stage:
/// directories first,
/// then files and links in parallel slices.
/// Why:
///   The manifest is parent first,
/// so every directory exists before a worker needs it;
/// the slices are contiguous so
///  neighbouring files share a directory,
/// and the first failure stops the other workers.
/// Every slice is a job of the process's blocking pool,
/// so the copy never holds a worker of the runtime that polls tasks.
/// The callers of this face are synchronous (the staging of one created worktree's ignored state),
/// so it enters the process's runtime once and waits there.
/// The answers are this branch's:
/// a slice names the entries the source no longer held,
/// and a failure of one entry
///  carries its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyManifestEntries({ sourceRoot, stageRoot, entries }): CopiedManifest;
/// ```
pub fn copy_manifest_entries(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
) -> Result<CopiedManifest, WorktreeCopyError> {
    return copy_manifest_entries_with(
        source_root,
        stage_root,
        entries,
        try_clone,
        &CopyCeiling::unbounded(),
    );
}

/// What:
///  `copy_manifest_entries` with the clone request the caller supplies,
/// so a control can make a source entry change or
///  vanish at a stated moment.
/// Why:
///   The landing and the worktree copy both reach the same primitive,
/// and only a supplied clone lets a
///  control place a write between one entry's copy and the next one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyManifestEntriesWith({ sourceRoot, stageRoot, entries, clone }): CopiedManifest;
/// ```
pub fn copy_manifest_entries_with(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
    ceiling: &CopyCeiling,
) -> Result<CopiedManifest, WorktreeCopyError> {
    return match crate::lazy_runtime::run_on_runtime(copy_manifest_entries_awaiting(
        source_root,
        stage_root,
        Arc::from(entries),
        clone,
        Arc::new(*ceiling),
    )) {
        Ok(copied) => copied,
        // Nothing was copied, and the reason the runtime could not run the workers is why.
        Err(refusal) => Err(raw_failure(&std::io::Error::other(format!(
            "the staged copy could not be made ({refusal})"
        )))),
    };
}

/// What:
///  `copy_manifest_entries` on the process's runtime:
/// every slice of leaves is one job of the blocking pool.
/// Why:
///   A caller that is itself a task of the runtime cannot enter it,
/// so it awaits this face instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function copyManifestEntries({ sourceRoot, stageRoot, entries, clone }): Promise<CopiedManifest>;
/// ```
async fn copy_manifest_entries_awaiting(
    source_root: &Path,
    stage_root: &Path,
    entries: Arc<[WorktreeCopyEntry]>,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
    ceiling: Arc<CopyCeiling>,
) -> Result<CopiedManifest, WorktreeCopyError> {
    for entry in entries.iter() {
        if entry.kind != EntryKind::Directory {
            continue;
        }
        if ceiling.reached() {
            return Err(ceiling.stopped_error());
        }
        let stage_path: PathBuf = filesystem_path(stage_root, entry.relative_path.as_slice())?;
        create_private_tree(stage_path.as_path())?;
    }
    let mut stamps: Vec<Option<FileStamp>> = vec![None; entries.len()];
    let workers: usize = copy_worker_count(entries.len());
    let failed: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let slice_length: usize = entries.len().div_ceil(workers.max(1)).max(1);
    let mut states: Vec<ASliceToCopy> = Vec::new();
    let mut slices: Vec<std::ops::Range<usize>> = Vec::new();
    let mut at: usize = 0;
    for chunk in entries.chunks(slice_length) {
        let range: std::ops::Range<usize> = at..at + chunk.len();
        states.push((
            Arc::clone(&entries),
            source_root.to_path_buf(),
            stage_root.to_path_buf(),
            range.clone().collect(),
            Arc::clone(&failed),
            clone,
            Arc::clone(&ceiling),
        ));
        slices.push(range);
        at += chunk.len();
    }
    let mut outcomes: Vec<Result<SliceCopy, WorktreeCopyError>> = Vec::new();
    for answer in run_all_side_by_side_on_the_runtime(states, copy_slice_and_flag_failure).await {
        outcomes.push(match answer {
            Ok(outcome) => outcome,
            Err(crate::blocking_job::BlockingJobError::Panicked) => Err(raw_failure(
                &std::io::Error::other("a copy worker panicked"),
            )),
            Err(refused) => Err(raw_failure(&std::io::Error::other(format!(
                "a copy worker did not run ({refused})"
            )))),
        });
    }
    let mut gone: Vec<Vec<u8>> = Vec::new();
    for (range, outcome) in slices.into_iter().zip(outcomes) {
        let copied: SliceCopy = outcome?;
        // A slice writes the stamps of its own positions, so a slice that stopped early cannot shift another's.
        for (slot, stamp) in stamps[range].iter_mut().zip(copied.stamps) {
            *slot = stamp;
        }
        gone.extend(copied.gone);
    }
    return Ok(CopiedManifest {
        stamps,
        gone,
    });
}

/// What:
///  The parent directory of a native path,
/// which always exists for a validated path under a root.
/// Why:
///   Selected nested files can need parents the selection did not name,
/// so each file creates its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dirname(path)
/// ```
fn parent_of(path: &Path) -> PathBuf {
    return match path.parent() {
        Some(parent) => parent.to_path_buf(),
        None => path.to_path_buf(),
    };
}
