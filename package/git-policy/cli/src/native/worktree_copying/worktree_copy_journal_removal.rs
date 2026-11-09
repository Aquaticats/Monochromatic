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

use crate::parallel_budget::{HostResources, sliced_workers};
use crate::private_storage::sync_directory;
use crate::side_by_side_threads::run_all_side_by_side;
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure, raw_failure_of};
use crate::worktree_copy_journal::write_journal;
use crate::worktree_copy_journal_encoding_and_validation::{Journal, PendingJournal, Phase, TEMPORARY_SUFFIX};
use crate::worktree_copy_stage_leaf_copying::PARALLEL_COPY_MAX_WORKERS;
use std::path::{Path, PathBuf};

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
        std::fs::set_permissions(directory.as_path(), std::fs::Permissions::from_mode(0o700))
            .map_err(raw_failure_of)?;
        for listed in
            std::fs::read_dir(directory.as_path()).map_err(raw_failure_of)?
        {
            let child: std::fs::DirEntry = listed.map_err(raw_failure_of)?;
            let child_type: std::fs::FileType = child
                .file_type()
                .map_err(raw_failure_of)?;
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let i = cursor.next(); subtrees[i] !== undefined; i = cursor.next()) await rm(subtrees[i], { recursive: true });
/// ```
fn remove_subtrees_until_none_is_left((cursor, subtrees): (&std::sync::atomic::AtomicUsize, &[PathBuf])) -> std::io::Result<()> {
    loop {
        let index: usize = cursor.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let Some(subtree) = subtrees.get(index) else {
            return Ok(());
        };
        std::fs::remove_dir_all(subtree.as_path())?;
    }
}

/// What:
///  Removes a directory tree,
/// sharing the subtrees below a split depth among a bounded set of threads.
/// Why:
///   Unlinking a staged copy entry by entry took longer than staging it on a tree of hundreds of thousands of entries;
///  the unlinks are independent,
/// and the thread count follows the host's budget (the CPUs the process may use less a margin,
/// and the free
///  memory),
/// capped at eight.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeTreeSplit(path); // rm -r, with the subtrees removed concurrently
/// ```
pub fn remove_tree_split(path: &Path) -> std::io::Result<()> {
    let mut shallow: Vec<PathBuf> = vec![path.to_path_buf()];
    let mut level: Vec<PathBuf> = vec![path.to_path_buf()];
    let mut subtrees: Vec<PathBuf> = Vec::new();
    for depth in 0..REMOVAL_SPLIT_DEPTH {
        let mut next: Vec<PathBuf> = Vec::new();
        for directory in &level {
            for listed in std::fs::read_dir(directory)? {
                let entry: std::fs::DirEntry = listed?;
                let child: PathBuf = entry.path();
                if !entry.file_type()?.is_dir() {
                    std::fs::remove_file(child.as_path())?;
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
    let workers: usize = sliced_workers(usize::MAX, PARALLEL_COPY_MAX_WORKERS, &HostResources::read());
    let cursor: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let mut worker_states: Vec<(&std::sync::atomic::AtomicUsize, &[PathBuf])> = Vec::new();
    for _worker in 0..workers {
        worker_states.push((&cursor, subtrees.as_slice()));
    }
    let mut outcomes: Vec<std::io::Result<()>> = Vec::new();
    for joined in run_all_side_by_side(worker_states, remove_subtrees_until_none_is_left) {
        outcomes.push(match joined {
            Ok(outcome) => outcome,
            Err(_panic) => Err(std::io::Error::other("a removal worker panicked")),
        });
    }
    for outcome in outcomes {
        outcome?;
    }
    for directory in shallow.iter().rev() {
        std::fs::remove_dir(directory.as_path())?;
    }
    return Ok(());
}

/// What:
///  Removes a tree if it exists,
/// sharing its subtrees among threads.
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
