//! What:
//!  The removal of a finished or abandoned copy transaction's stage and journal.
//! Why:
//!  A stage can hold directories whose modes forbid their own removal and very many files,
//! so the removal makes the stage
//!  owner-accessible first and removes large subtrees side by side;
//! the journal goes only after the stage is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removePendingJournal(pending); // removes the stage, then the journal and its unfinished writes
//! ```

use crate::blocking_job::BlockingJobError;
use crate::parallel_budget::{HostResources, sliced_workers};
use crate::private_storage::sync_directory;
use crate::side_by_side_on_the_runtime::run_all_side_by_side_on_the_runtime;
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure, raw_failure_of};
use crate::worktree_copy_journal::write_journal;
use crate::worktree_copy_journal_encoding_and_validation::{Journal, PendingJournal, Phase, TEMPORARY_SUFFIX};
use crate::worktree_copy_stage_leaf_copying::PARALLEL_COPY_MAX_WORKERS;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// What:
///  Makes every directory of a stage owner-accessible before its entries are deleted.
/// Why:
///   An installed directory can carry a mode that forbids its own removal;
/// only the stage's own directories are changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await prepareStageRemoval(stageContainer);
/// ```
pub fn prepare_stage_removal(stage_container: &Path) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::PermissionsExt;
    let mut pending: Vec<PathBuf> = vec![stage_container.to_path_buf()];
    while let Some(directory) = pending.pop() {
        // `match` skips a directory already removed and refuses anything else that cannot be inspected.
        let metadata: std::fs::Metadata = match std::fs::symlink_metadata(directory.as_path()) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(raw_failure(&error)),
        };
        if !metadata.is_dir() {
            continue;
        }
        // A directory another command removed at the same moment is already as writable as it needs to be.
        match std::fs::set_permissions(directory.as_path(), std::fs::Permissions::from_mode(0o700)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(raw_failure(&error)),
        }
        // In a walk of one stage no entry can point back,
        // and a directory a link names is never entered: only a directory's own entries
        // are read, so a stage whose directory another command emptied is walked no further.
        let listing = match std::fs::read_dir(directory.as_path()) {
            Ok(listing) => listing,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(raw_failure(&error)),
        };
        for listed in listing {
            let child: std::fs::DirEntry = match listed {
                Ok(child) => child,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(raw_failure(&error)),
            };
            let child_type: std::fs::FileType = match child.file_type() {
                Ok(kind) => kind,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(raw_failure(&error)),
            };
            if child_type.is_dir() {
                pending.push(child.path());
            }
        }
    }
    return Ok(());
}

/// What:
///  Removes a stage container with everything in it,
/// whatever modes the installed directories carry.
/// Why:
///   Completed transactions and abandoned stages are removed the same way,
/// and an absent stage is already removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await prepareStageRemoval(stageContainer); await rm(stageContainer, { recursive: true, force: true });
/// ```
pub fn remove_stage(stage_container: &Path) -> Result<(), WorktreeCopyError> {
    // A stage made by a snapshot is a subvolume; where the mount allows it one call removes the whole tree.
    #[cfg(target_os = "linux")]
    if crate::fs_kind::is_subvolume_root(stage_container).unwrap_or(false) && crate::fs_btrfs_ioctl::subvolume_destroy_at(stage_container).is_ok() {
        return Ok(());
    }
    prepare_stage_removal(stage_container)?;
    return remove_tree_if_present(stage_container);
}

/// What:
///  Removes one completed transaction:
/// its stage,
/// its unfinished temporaries,
/// and its journal.
/// Why:
///   A complete phase is written first,
/// so an interruption during cleanup resumes as cleanup,
/// not as installation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeWorktreeCopyJournal(pending);
/// ```
pub fn remove_pending_journal(pending: &PendingJournal) -> Result<(), WorktreeCopyError> {
    if pending.journal.phase != Phase::Complete {
        let completed: Journal = Journal {
            phase: Phase::Complete,
            ..pending.journal.clone()
        };
        write_journal(pending.path.as_path(), &completed)?;
    }
    remove_stage(pending.journal.stage_container.as_path())?;
    remove_file_if_present(pending.path.as_path())?;
    remove_unfinished_writes(pending.path.as_path())?;
    if let Some(parent) = pending.path.parent() {
        sync_directory(parent).map_err(raw_failure_of)?;
    }
    return Ok(());
}

/// What:
///  How many directory levels below a removed stage are emptied by one thread before the rest is split among workers.
/// Why:
///   A stage holds a few very large trees (`node_modules`,
/// `target`);
/// three levels below the payload give enough
///  independent subtrees to share without walking the whole stage on one thread first.
///  Measured with `remove_tree_split` itself at depths 1 to 6 on reflinked copies of a tree of 22,717 entries of this repository
///  (2026-11-03,
/// five runs each in alternating orders,
/// other sessions running,
/// a debug build):
/// medians 365,
/// 281,
/// 304,
/// 325,
/// 437 and
///  512 ms;
/// depths 2 and 3 overlap (251 to 305 ms and 253 to 309 ms),
/// and depths 1,
/// 4,
/// 5 and 6 were slower.
/// Kept at 3 as the coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// option A).
/// The tree has no `node_modules` or `target`,
/// so a stage with a few very large subtrees was not measured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REMOVAL_SPLIT_DEPTH = 3;
/// ```
const REMOVAL_SPLIT_DEPTH: usize = 3;

/// What:
///  One removal worker:
/// take the next subtree from the shared cursor and remove it,
/// until none is left or a removal fails.
/// Why:
///   Workers share only the cursor,
/// so each subtree has exactly one owner and no lock is taken.
/// The worker is a job of the blocking pool,
/// which must own what it reads,
/// so the cursor and the subtrees travel as shares of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let i = cursor.next(); subtrees[i] !== undefined; i = cursor.next()) await rm(subtrees[i], { recursive: true });
/// ```
fn remove_subtrees_until_none_is_left((cursor, subtrees): (Arc<std::sync::atomic::AtomicUsize>, Arc<[PathBuf]>)) -> std::io::Result<()> {
    loop {
        let index: usize = cursor.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let Some(subtree) = subtrees.get(index) else {
            return Ok(());
        };
        // A subtree another command removed at the same moment is already gone, which is what this removal wanted.
        match std::fs::remove_dir_all(subtree.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
}

/// What:
///  Remove the subtrees of one removal with as many workers as the host leaves,
/// each worker a job of the blocking pool,
/// and give one outcome per worker.
/// Why:
///   The unlinks are independent,
/// so they run side by side,
/// and every worker takes a place of the process's one bound
/// on blocking jobs before its work runs
/// (`side_by_side_on_the_runtime.rs`).
/// The person waits here while the pool works,
/// never on a worker of the runtime that polls tasks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcomes = await Promise.all(workers.map(() => runBlockingJobWithinTheBudget(() => removeSubtrees(cursor, subtrees))));
/// ```
async fn remove_subtrees_awaiting(
    (cursor, subtrees): (Arc<std::sync::atomic::AtomicUsize>, Arc<[PathBuf]>),
    workers: usize,
) -> Vec<std::io::Result<()>> {
    let mut states: Vec<(Arc<std::sync::atomic::AtomicUsize>, Arc<[PathBuf]>)> = Vec::new();
    for _worker in 0..workers {
        states.push((Arc::clone(&cursor), Arc::clone(&subtrees)));
    }
    let mut outcomes: Vec<std::io::Result<()>> = Vec::new();
    for answer in run_all_side_by_side_on_the_runtime(states, remove_subtrees_until_none_is_left).await {
        outcomes.push(match answer {
            Ok(outcome) => outcome,
            Err(crate::blocking_job::BlockingJobError::Panicked) => Err(std::io::Error::other("a removal worker panicked")),
            Err(refused) => Err(std::io::Error::other(format!("a removal worker did not run ({refused})"))),
        });
    }
    return outcomes;
}

/// What:
///  Removes a directory tree,
/// sharing the subtrees below a split depth among a bounded set of workers of the blocking pool.
/// Why:
///   Unlinking a staged copy entry by entry took longer than staging it on a tree of hundreds of thousands of entries;
///  the unlinks are independent,
/// and the worker count follows the host's budget (the CPUs the process may use less a margin,
/// and the free
///  memory),
/// capped at eight.
/// The walk to the split depth and the removal of what is left above it run on the calling thread,
/// which is this call's own;
/// the subtrees below it are removed by jobs of the pool.
/// The callers of this face are synchronous (the recovery of an interrupted copy and the reaping of
/// abandoned stages),
///  so it enters the process's runtime once and waits there;
/// where no runtime can be built the removal reports it,
/// and `remove_tree_if_present` falls back to one plain recursive removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeTreeSplit(path); // rm -r, with the subtrees removed concurrently
/// ```
pub fn remove_tree_split(path: &Path) -> std::io::Result<()> {
    return match crate::lazy_runtime::run_on_runtime(remove_tree_split_awaiting(path)) {
        Ok(outcome) => outcome,
        // Nothing was removed, and the reason the runtime could not run the workers is why.
        Err(refusal) => Err(std::io::Error::other(format!("the subtrees below {} were not removed side by side ({refusal})", path.display()))),
    };
}

/// What:
///  `remove_tree_split` on the process's runtime:
/// the split walk here and every subtree below the split depth removed by a job of the blocking pool.
/// Why:
///   A caller that is itself a task of the runtime cannot enter it,
///  so it awaits this face instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeTreeSplit(path: string): Promise<void>;
/// ```
async fn remove_tree_split_awaiting(path: &Path) -> std::io::Result<()> {
    let (subtrees, shallow): (Vec<PathBuf>, Vec<PathBuf>) = walk_to_the_split_depth_on_the_pool(path).await?;
    let workers: usize = sliced_workers(usize::MAX, PARALLEL_COPY_MAX_WORKERS, &HostResources::read());
    let cursor: Arc<std::sync::atomic::AtomicUsize> = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    for outcome in remove_subtrees_awaiting((cursor, Arc::from(subtrees)), workers).await {
        outcome?;
    }
    // The directories above the split depth hold nothing now, so this is a handful of known removals rather than a
    //  walk: it stays on the caller, where the sweep that kills the copy at each `rmdir` still traces the wrapper's own
    //  thread (`binary/binary_fault_trace_tests.rs`, `copy_flow`).
    return remove_shallow_directories(shallow.as_slice());
}

/// What:
///  `walk_to_the_split_depth` as a job of the blocking pool,
/// run on this task where the system refuses a thread.
/// Why:
///   A walk of a stage can read an unbounded number of directories,
/// and a disk,
/// a lock or a mount holds it,
///  so it belongs off the worker that carries the call.
/// A thread is only a way to work sooner (`foundation/scoped_start.rs`):
/// where the system refuses one,
///  the walk runs in turn,
/// as the removal it serves did before the pool was asked,
/// and the control that runs it with no
///  thread to start still removes the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function walkToTheSplitDepthOnThePool(path: string): [string[], string[]]
/// ```
async fn walk_to_the_split_depth_on_the_pool(path: &Path) -> std::io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let walking: PathBuf = path.to_path_buf();
    return match crate::blocking_job::run_blocking_job(move || { return walk_to_the_split_depth(walking.as_path()); }).await {
        Ok(lists) => lists,
        Err(BlockingJobError::NoThreadToStart | BlockingJobError::MayStillRunAfterItsThreadWasRefused) => {
            return walk_to_the_split_depth(path);
        }
        Err(other) => Err(std::io::Error::other(format!(
            "the walk to the split depth below {} did not run ({other})",
            path.display()
        ))),
    };
}

/// What:
///  The entries of the tree at `path` down to the split depth:
/// the subtrees at that depth,
/// and the directories above it that
///  are removed once those subtrees are gone.
/// Why:
///   This is one job of the blocking pool rather than a walk of the task that called the removal,
/// because a stage holds an
///  unbounded number of directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walkToTheSplitDepth(path: string): [string[], string[]]
/// ```
fn walk_to_the_split_depth(path: &Path) -> std::io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut shallow: Vec<PathBuf> = vec![path.to_path_buf()];
    let mut level: Vec<PathBuf> = vec![path.to_path_buf()];
    let mut subtrees: Vec<PathBuf> = Vec::new();
    for depth in 0..REMOVAL_SPLIT_DEPTH {
        let mut next: Vec<PathBuf> = Vec::new();
        for directory in &level {
            // A directory another command removed at the same moment holds nothing to remove here.
            let listing = match std::fs::read_dir(directory) {
                Ok(listing) => listing,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            for listed in listing {
                let entry: std::fs::DirEntry = match listed {
                    Ok(entry) => entry,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(error) => return Err(error),
                };
                let child: PathBuf = entry.path();
                let kind: std::fs::FileType = match entry.file_type() {
                    Ok(kind) => kind,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(error) => return Err(error),
                };
                if !kind.is_dir() {
                    // The entry may be gone already: a removal that finds nothing is the removal that was wanted.
                    match std::fs::remove_file(child.as_path()) {
                        Ok(()) => {}
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error),
                    }
                } else if depth + 1 == REMOVAL_SPLIT_DEPTH {
                    subtrees.push(child);
                } else {
                    shallow.push(child.clone());
                    next.push(child);
                }
            }
        }
        level = next;
    }
    return Ok((subtrees, shallow));
}

/// What:
///  Removes the directories above the split depth,
/// deepest first,
/// treating an absent one as removed.
/// Why:
///   The subtrees were removed by their own jobs,
/// so what is left above them is empty;
/// this is the second
///  job of the split removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeShallowDirectories(shallow: string[]): void
/// ```
fn remove_shallow_directories(shallow: &[PathBuf]) -> std::io::Result<()> {
    for directory in shallow.iter().rev() {
        match std::fs::remove_dir(directory.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    return Ok(());
}

/// What:
///  Removes a tree if it exists,
/// sharing its subtrees among workers of the blocking pool.
/// Why:
///   An absent tree is the normal answer for a retried removal;
/// a failed split removal falls back to the plain recursive
///  removal,
/// which reports what remains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { recursive: true, force: true });
/// ```
pub fn remove_tree_if_present(path: &Path) -> Result<(), WorktreeCopyError> {
    // `match` accepts absence and reports every other removal failure.
    // A split removal that fails is tried again as one plain removal.
    let removed: std::io::Result<()> = match remove_tree_split(path) {
        Ok(()) => Ok(()),
        Err(_split_failure) => std::fs::remove_dir_all(path),
    };
    match removed {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// What:
///  Removes one file,
/// treating an absent file as removed.
/// Why:
///   The same retry reasoning as for trees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { force: true });
/// ```
pub fn remove_file_if_present(path: &Path) -> Result<(), WorktreeCopyError> {
    // `match` accepts absence and reports every other removal failure.
    match std::fs::remove_file(path) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// What:
///  Removes the temporary writes an interrupted write of one journal left beside it.
/// Why:
///   Only temporaries named after this journal are removed,
/// so another journal's write is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeUnfinishedJournalWrites(path)
/// ```
pub fn remove_unfinished_writes(path: &Path) -> Result<(), WorktreeCopyError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    let prefix: String = format!(
        "{}.",
        path.file_name().unwrap_or_default().to_string_lossy()
    );
    for listed in std::fs::read_dir(parent).map_err(raw_failure_of)? {
        let entry: std::fs::DirEntry = listed.map_err(raw_failure_of)?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(prefix.as_str()) && name.ends_with(TEMPORARY_SUFFIX) {
            remove_file_if_present(entry.path().as_path())?;
        }
    }
    return Ok(());
}

/// Removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_journal_removal_tests.rs"]
mod tests;
