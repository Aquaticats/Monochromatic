//! What:
//!  The tracked-file proof of a clean check:
//! the reason the index differs from the tree of `HEAD`,
//! every tracked file unchanged by the size,
//! times and inode its entry recorded or,
//! where those cannot vouch for it,
//! by its content,
//! the first entry a reset to a commit cannot undo,
//! and the answer two verdicts make.
//! Why:
//!  A caller that copies the whole worktree,
//! untracked files included,
//! needs a quiet and unmodified tracked state and no Git process.
//! A proof that panicked proves nothing,
//! and a pool that refused the work says which state it is in,
//! so either becomes a refusal rather than a claim.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const source = checkTrackedFiles([root, gitDir, commonDir]); // throws the reason
//! const verdict = theCleanVerdict([tracked, untracked]); // throws the first reason
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
};
use crate::blocking_job::BlockingJobError;
use crate::repo_read_index::IndexEntry;
use crate::repo_read_tree::{
    FlatEntry,
    MODE_GITLINK,
};
use crate::worktree_clean_content::ConversionEnvironment;
use crate::worktree_clean_stat_verdicts::{
    ASSUME_VALID,
    CleanRefusal,
    CleanRefusalKind,
    file_changed,
};
use crate::worktree_clean_state::{
    CleanSource,
    State,
    operation_in_progress,
    read_state,
};
use crate::worktree_clean_tree_walk::{
    ContentTarget,
    tree_differs_from_index_by_content,
    tree_refusal,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The reason the index differs from the tree of `HEAD`,
/// or nothing when the two are the same files.
/// Why:
///   A staged change,
/// a staged deletion or an intent-to-add entry is an uncommitted state the copy must not carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexDiffersFromTree(index: IndexEntry[], tree: FlatEntry[]): string | undefined;
/// ```
pub(crate) fn index_differs_from_tree(
    index: &[IndexEntry],
    tree: &[FlatEntry],
) -> Option<CleanRefusal> {
    if index.len() != tree.len() {
        return Some(CleanRefusal::new(
            CleanRefusalKind::StagedChange,
            "the index and the tree of HEAD hold different numbers of files",
        ));
    }
    for (staged, committed) in index.iter().zip(tree) {
        if staged.stage != 0 || staged.extended != 0 {
            return Some(CleanRefusal::new(
                CleanRefusalKind::NotPlain,
                format!(
                    "the index entry {} is unmerged, intent-to-add or skip-worktree",
                    String::from_utf8_lossy(&staged.path)
                ),
            ));
        }
        if staged.mode == MODE_GITLINK {
            return Some(CleanRefusal::new(
                CleanRefusalKind::NotPlain,
                format!("{} is a submodule", String::from_utf8_lossy(&staged.path)),
            ));
        }
        if staged.path != committed.path
            || staged.mode != committed.mode
            || staged.id != committed.id
        {
            return Some(CleanRefusal::new(
                CleanRefusalKind::StagedChange,
                format!(
                    "{} is staged with a change",
                    String::from_utf8_lossy(&staged.path)
                ),
            ));
        }
    }
    return None;
}

/// The checks of `check_clean_tracked`,
/// by stat data alone or,
/// given a `scope`,
/// by content where the stat data is not enough.
pub(crate) fn check_tracked(
    root: &Path,
    git_dir: &Path,
    common_dir: &Path,
    environment: Option<&ConversionEnvironment>,
    owned_index_lock: bool,
) -> Result<CleanSource, CleanRefusal> {
    if let Some(name) = operation_in_progress(git_dir, owned_index_lock) {
        return Err(CleanRefusal::new(
            CleanRefusalKind::OperationInProgress(name),
            format!("{name} exists in {}", git_dir.display()),
        ));
    }
    let read_index_and_tree_entered: bool = begin_step("readIndexAndTree");
    // Both proofs read the repository's own store, as `main`'s reader did: the question is about this worktree's own files.
    let read_index_and_tree_outcome = read_state(git_dir, common_dir);
    end_step(read_index_and_tree_entered);
    let state: State = read_index_and_tree_outcome?;
    let index_against_tree_entered: bool = begin_step("indexAgainstTree");
    let index_against_tree_outcome =
        index_differs_from_tree(state.entries.as_slice(), state.tree.as_slice());
    end_step(index_against_tree_entered);
    if let Some(refusal) = index_against_tree_outcome {
        return Err(refusal);
    }
    let differs: Option<CleanRefusal> = match environment {
        None => {
            let walk_source_entered: bool = begin_step("walkSource");
            let walked: Option<CleanRefusal> =
                tree_refusal(root, state.stats.as_slice(), state.index_seconds);
            end_step(walk_source_entered);
            walked
        }
        Some(found) => tree_differs_from_index_by_content(
            &ContentTarget {
                root,
                git_dir,
                common_dir,
                environment: found,
                format: state.format,
            },
            state.stats.as_slice(),
            state.entries.as_slice(),
            state.index_seconds,
        )
        .map(file_changed),
    };
    if let Some(refusal) = differs {
        return Err(refusal);
    }
    let bytes: Vec<u8> = state.bytes.clone();
    return Ok(state.into_source(bytes));
}

/// What:
///  The first entry of a worktree's index or tree that a reset to a commit cannot be proven to undo,
/// if any:
/// unmerged,
///  intent-to-add and skip-worktree entries,
/// assume-unchanged entries,
/// and submodules in the index or in the tree of `HEAD`.
/// Why:
///   The first refusal of the clean check can be an ordinary change that hides one of these further on;
/// a restore needs all of
///  the index looked at,
/// not the first offender.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryAResetCannotUndo(state: State): CleanRefusal | undefined;
/// ```
pub(crate) fn entry_a_reset_cannot_undo(state: &State) -> Option<CleanRefusal> {
    for staged in &state.entries {
        let shown: String = String::from_utf8_lossy(&staged.path).into_owned();
        if staged.stage != 0 || staged.extended != 0 {
            return Some(CleanRefusal::new(
                CleanRefusalKind::NotPlain,
                format!("the index entry {shown} is unmerged, intent-to-add or skip-worktree"),
            ));
        }
        if staged.mode == MODE_GITLINK {
            return Some(CleanRefusal::new(
                CleanRefusalKind::NotPlain,
                format!("{shown} is a submodule"),
            ));
        }
    }
    for entry in &state.tree {
        if entry.mode == MODE_GITLINK {
            return Some(CleanRefusal::new(
                CleanRefusalKind::NotPlain,
                format!(
                    "{} is a submodule in HEAD",
                    String::from_utf8_lossy(&entry.path)
                ),
            ));
        }
    }
    for entry in &state.stats {
        if entry.flags & ASSUME_VALID != 0 {
            return Some(CleanRefusal::new(
                CleanRefusalKind::NotPlain,
                format!(
                    "{} is marked assume-unchanged",
                    String::from_utf8_lossy(&entry.path)
                ),
            ));
        }
    }
    return None;
}

/// What:
///  The tracked-file proof of `check_clean`,
/// which runs as a job of the blocking pool while the untracked scan is awaited.
/// Why:
///   The proof reads the index,
/// the tree of `HEAD` and a stat of every tracked file,
/// none of which is a question
///  for a task that also awaits a bounded Git child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkTrackedFiles([root, gitDir, commonDir]: [string, string, string]): CleanSource;
/// ```
pub(crate) fn check_tracked_files(
    (root, git_dir, common_dir): (&Path, &Path, &Path)
) -> Result<CleanSource, CleanRefusal> {
    return check_tracked(root, git_dir, common_dir, None, false);
}

/// What:
///  `check_tracked_files` for a job of the blocking pool,
/// which owns the paths it reads.
/// Why:
///   A job of the pool must own everything it reads,
/// and the caller's paths live on a stack that the job outlives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkTrackedFilesInAJob([root, gitDir, commonDir]: [string, string, string]): CleanSource; // throws the reason
/// ```
pub(crate) fn check_tracked_files_in_a_job(
    (root, git_dir, common_dir): (PathBuf, PathBuf, PathBuf)
) -> Result<CleanSource, CleanRefusal> {
    return check_tracked_files((root.as_path(), git_dir.as_path(), common_dir.as_path()));
}

/// What:
///  The answer two verdicts make:
/// the tracked proof's refusal first,
/// then the untracked file's.
/// Why:
///   The two checks read different things and their verdicts were always read in this order,
/// so both faces of the check answer the
///  same state with the same reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theCleanVerdict([tracked, untracked]: [CleanSource | Refusal, string | undefined | Refusal]): CleanSource;
/// ```
pub(crate) fn the_clean_verdict(
    (tracked, untracked): (
        Result<CleanSource, CleanRefusal>,
        Result<Option<String>, CleanRefusal>,
    )
) -> Result<CleanSource, CleanRefusal> {
    let clean: CleanSource = tracked?;
    if let Some(name) = untracked? {
        return Err(CleanRefusal::new(
            CleanRefusalKind::Untracked,
            format!("{name} is untracked and not ignored"),
        ));
    }
    return Ok(clean);
}

/// What:
///  Keep the reason of a tracked proof that could not run as a job of the pool.
/// Why:
///   A proof that panicked proves nothing,
/// and a pool that refused the work says which state it is in;
/// the check reports
///  either as a refusal rather than unwinding a caller or claiming a verdict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aRefusedJobIsARefusal(error: BlockingJobError): Refusal;
/// ```
pub(crate) fn a_refused_job_is_a_refusal(error: BlockingJobError) -> CleanRefusal {
    return match error {
        BlockingJobError::Panicked => {
            CleanRefusal::unreadable("the proof of the tracked files stopped unexpectedly")
        }
        other => {
            CleanRefusal::unreadable(format!("the tracked files could not be proven ({other})"))
        }
    };
}
