//! What:
//!  Whether a worktree is clean enough to be copied from,
//! or removed without asking Git:
//! no operation in progress,
//! an
//!  index equal to the tree of `HEAD`,
//! every tracked file unchanged by its recorded size,
//! times and inode,
//! and no
//!  untracked file that Git does not ignore.
//! Why:
//!  A snapshot or a clone carries the bytes of the source's files,
//! so any change the source has not committed would
//!  reach the new worktree,
//! and a removal by the subvolume ioctl cannot be undone.
//! The tracked-file checks read the
//!  index in process (no Git process) with the same comparison Git's fast path makes,
//! and refuse every shape they cannot
//!  prove:
//! a split or sparse index,
//! a skip-worktree or assume-valid entry,
//! a submodule,
//! an entry as new as the index
//!  file (Git re-hashes those,
//! so their stat fields prove nothing).
//! The one question the index cannot answer,
//! whether
//!  an untracked file is ignored,
//! goes to one `git ls-files` run.
//! Every refusal returns its reason in words for the log.
//!
//! The reading every check judges,
//! and the tracked-file proof itself,
//! are the sibling files `worktree_clean_state.rs` and `worktree_clean_tracked_proof.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clean = await checkClean({ root, gitDir, commonDir }); // { headCommit, headTree } or throws a reason
//! ```

use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::lazy_runtime::run_on_runtime;
use crate::transaction_git::GitContext;
use crate::worktree_clean_content::ConversionEnvironment;
pub use crate::worktree_clean_stat_verdicts::{
    CleanRefusal,
    CleanRefusalKind,
    StatEntry,
    StatVerdict,
    metadata_differs_from_entry,
    metadata_verdict,
};
/// The answer a clean check makes,
/// and the two readings of `HEAD` and of the index's stat fields it rests on,
/// re-exported so every caller of this check names them here.
pub use crate::worktree_clean_state::{
    CleanSource,
    head_commit,
    stored_stats,
};
use crate::worktree_clean_state::{
    State,
    read_state,
};
use crate::worktree_clean_tracked_proof::{
    a_refused_job_is_a_refusal,
    check_tracked,
    check_tracked_files,
    check_tracked_files_in_a_job,
    entry_a_reset_cannot_undo,
    the_clean_verdict,
};
pub use crate::worktree_clean_tree_walk::{
    ContentTarget,
    StatCursor,
    paths_whose_content_differs,
    stat_fields_beneath,
    tree_differs_from_index,
    tree_differs_from_index_beneath,
    tree_fields_by_content,
};
use crate::worktree_clean_untracked_scan::{
    scan_for_untracked_file,
    scan_for_untracked_file_awaiting,
};
use std::ffi::OsString;
use std::path::Path;

/// What:
///  The checks of `check_clean` that read no untracked file:
/// no operation in progress,
/// an index equal to the tree of `HEAD`,
///  and every tracked file with the size,
/// times and inode its index entry recorded.
/// Why:
///   A caller that copies the whole worktree,
/// untracked files included (the conversion of a worktree into a subvolume),
///  needs a quiet and unmodified tracked state but has no reason to refuse for an untracked file,
/// and needs no Git process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkCleanTracked(root: string, gitDir: string, commonDir: string): CleanSource; // throws the reason
/// ```
pub fn check_clean_tracked(
    root: &Path,
    git_dir: &Path,
    common_dir: &Path,
) -> Result<CleanSource, String> {
    return match check_tracked(root, git_dir, common_dir, None, false) {
        Ok(source) => Ok(source),
        Err(refusal) => Err(refusal.reason),
    };
}

/// What:
///  The cleanliness proof that settles every entry its stat data cannot vouch for by reading the file,
///  and scans for an untracked file as `check_clean` does.
/// Why:
///   A worktree added a moment ago holds files whose whole-second times are the index's,
/// and by Git's own racy
///  rule that proves nothing either way;
/// Git settles such an entry by comparing the file with the commit,
/// and so does
///  this proof,
/// which is the one the conversion path already trusts.
/// The untracked scan is the same one `check_clean` makes,
/// so
///  a tree this proves clean is exactly a tree that check would prove clean could it settle the stat data,
/// and the
///  volumes a removal goes around sit in untracked directories the scan does not read inside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkCleanByContent({ context, root, gitDir, commonDir }): Promise<CleanSource>;
/// ```
pub fn check_clean_by_content(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    common_dir: &Path,
) -> Result<CleanSource, CleanRefusal> {
    let variables: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    let platform: crate::subvolume_platform::LinuxPlatform =
        crate::subvolume_platform::LinuxPlatform::forwarding_to(
            variables.as_slice(),
            Some(context.real_git.as_path()),
        );
    let environment: ConversionEnvironment =
        crate::subvolume_platform::Platform::conversion_environment(&platform);
    return the_clean_verdict((
        check_tracked(root, git_dir, common_dir, Some(&environment), false),
        scan_for_untracked_file((context, root)),
    ));
}

/// What:
///  `check_clean_tracked` that settles an entry the stat data cannot vouch for by comparing the file with the committed
///  blob in process,
/// under the configuration and attribute locations of `environment`.
/// Why:
///   The conversion copies the whole worktree and rewrites the index afterwards,
/// and an entry it could not prove keeps its old
///  stat data,
/// so Git hashes it at the first status;
/// the snapshot rung settles the entries of the copy the same way before it
///  installs the index (`worktree_snapshot.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkCleanTrackedByContent(root: string, gitDir: string, commonDir: string, environment: ConversionEnvironment, ownedIndexLock: boolean): CleanSource; // throws the reason
/// ```
pub fn check_clean_tracked_by_content(
    root: &Path,
    git_dir: &Path,
    common_dir: &Path,
    environment: &ConversionEnvironment,
    owned_index_lock: bool,
) -> Result<CleanSource, String> {
    return match check_tracked(
        root,
        git_dir,
        common_dir,
        Some(environment),
        owned_index_lock,
    ) {
        Ok(source) => Ok(source),
        Err(refusal) => Err(refusal.reason),
    };
}

/// What:
///  Check that the worktree at `root`,
/// whose Git directory is `git_dir` and whose shared directory is `common_dir`,
/// is
///  clean,
/// on the runtime:
/// the untracked scan awaited on this task and the tracked proof a job of the blocking pool beside it.
/// Why:
///   The two read different things and write nothing,
/// so neither waits for the other;
/// the scan's wait is bounded by its step
///  terms and the proof walks every tracked file,
/// which must never run on the task that awaits that bounded wait.
/// A task of the runtime cannot start
///  this child through the synchronous face,
/// so the scan is the awaited one.
/// Where the pool refuses a thread,
/// the proof runs in turn under a place of the
///  process's bound (`blocking_jobs_within_the_budget.rs`,
/// `run_a_job_or_in_turn`),
/// so the answer is the same and only the overlap is lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkCleanAwaiting({ context, root, gitDir, commonDir }): Promise<CleanSource>; // throws the reason
/// ```
pub async fn check_clean_awaiting(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    common_dir: &Path,
) -> Result<CleanSource, CleanRefusal> {
    let (untracked, tracked): (
        Result<Option<String>, CleanRefusal>,
        Result<Result<CleanSource, CleanRefusal>, BlockingJobError>,
    ) = tokio::join!(
        scan_for_untracked_file_awaiting((context, root)),
        run_a_job_or_in_turn(
            (
                root.to_path_buf(),
                git_dir.to_path_buf(),
                common_dir.to_path_buf()
            ),
            check_tracked_files_in_a_job
        ),
    );
    let proof: Result<CleanSource, CleanRefusal> = match tracked {
        Ok(verdict) => verdict,
        Err(refused) => Err(a_refused_job_is_a_refusal(refused)),
    };
    return the_clean_verdict((proof, untracked));
}

/// What:
///  `check_clean_awaiting` for a caller that is not a task of the runtime:
/// it enters the process's runtime once and waits there,
/// and where no runtime can be built the two checks run one
///  after another on this thread.
/// Why:
///   The callers of this face are synchronous (the removal of a worktree and the choice of a snapshot's source),
/// so one
///  of them enters the runtime for them;
/// from inside the runtime it refuses,
/// and `check_clean_awaiting` is the face such a caller uses.
/// The
///  work done without a runtime is the same work,
/// with nothing running beside it,
/// so every verdict is the one the runtime gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkClean(args: { context: GitContext; root: string; gitDir: string; commonDir: string }): CleanSource; // throws a reason
/// ```
pub fn check_clean(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    common_dir: &Path,
) -> Result<CleanSource, CleanRefusal> {
    return match run_on_runtime(check_clean_awaiting(context, root, git_dir, common_dir)) {
        Ok(verdict) => verdict,
        Err(refusal) => {
            crate::diagnostic_log::debug(
                "checkClean",
                format!(
                    "the tracked files and the untracked scan run one after another ({refusal})"
                )
                .as_str(),
            );
            the_clean_verdict((
                check_tracked_files((root, git_dir, common_dir)),
                scan_for_untracked_file((context, root)),
            ))
        }
    };
}

/// What:
///  What `HEAD` names for a worktree that is not clean,
/// and why it is not,
/// once it is known that the only trouble is ordinary
///  changes.
/// Why:
///   The snapshot rung copies such a worktree and restores the copy to its `HEAD`;
/// the copy's own index is installed,
/// so the
///  index bytes here are only as read:
/// they name the tracked paths and prove nothing about the files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DirtySource = { source: CleanSource; why: CleanRefusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirtySource {
    /// What `HEAD` names;
    /// its `index_bytes` are the index as read,
    /// not proven equal to the files.
    pub source: CleanSource,
    /// The first reason the worktree is not clean.
    pub why: CleanRefusal,
}

/// What:
///  For a worktree `check_clean` refused with the ordinary change `why`,
/// what `HEAD` names,
/// or the refusal that a restore
///  cannot undo when the index holds one anywhere.
/// Why:
///   The first refusal hides the rest:
/// a staged edit can come before a submodule or a skip-worktree entry,
/// and only a look at
///  every entry tells a change a reset undoes from a state it does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dirtySource(args: { gitDir: string; commonDir: string; why: CleanRefusal }): DirtySource; // throws a CleanRefusal
/// ```
pub fn dirty_source(
    git_dir: &Path,
    common_dir: &Path,
    why: CleanRefusal,
) -> Result<DirtySource, CleanRefusal> {
    if !why.kind.restorable() {
        return Err(why);
    }
    let state: State = read_state(git_dir, common_dir)?;
    if let Some(refusal) = entry_a_reset_cannot_undo(&state) {
        return Err(refusal);
    }
    let read: Vec<u8> = state.bytes.clone();
    return Ok(DirtySource {
        source: state.into_source(read),
        why,
    });
}

#[cfg(test)]
#[path = "worktree_clean_tests.rs"]
mod tests;
