//! What:
//!  The move of the local branch after the merge was pushed:
//! the fast-forward,
//! the record that guards it,
//! the check that
//!  uncommitted work is not in the way,
//! and the words of a refusal.
//! Why:
//!  The branch follows the published merge only where nothing can be lost;
//! this file holds those checks apart from the
//!  retry loop in `auto_push_reconcile.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const advanced = await advanceLocalBranch(git, branchRef, local, mergeOid); // false | true | refusal text
//! ```

use crate::auto_push_single_flight::{
    PushGit, SingleFlightError, run_query,
};
use crate::auto_push_dirty_paths::{diff_paths, listed, overlapping_paths, status_paths};
use crate::diagnostic_log::{debug, warn};
use std::path::{Path, PathBuf};
use crate::inner_git_words::record_streams;
use crate::iso_time::now_iso;
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent, write_merge_intent};
use crate::merge_isolation_recovery::{MergeRecoveryAction, RecoveryPlace, settle_failed_merge};
use crate::random_id::random_uuid;
use crate::transaction_git::GitContext;
use crate::transaction_owner::current_transaction_owner;
use crate::auto_push_reconcile_basics_and_limits::{TAG, argv};
/// What:
///  Whether the status read after a fast-forward differs from the one read before it.
///  `&[u8]` borrows Git's output.
/// Why:
///   The uncommitted work must read exactly as it did;
/// a difference is the only sign that the move touched it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statusDiffers(before: Uint8Array, after: Uint8Array): boolean;
/// ```
pub(crate) fn status_differs(before: &[u8], after: &[u8]) -> bool {
    return before != after;
}

/// What:
///  The step that brings the checkout and then a named branch to the merge,
/// given as a value.
/// Why:
///   A control must be able to play a `git switch` in the instant between the checks above and the move
/// (`advance_local_branch_moving`),
/// which is the window the move has to survive;
/// the shipped step is `move_the_named_branch`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BranchMove = (git, branchRef, local, mergeOid) => Result<void, string>;
/// ```
pub(crate) type BranchMove<'a> = &'a dyn Fn(&PushGit, &str, &str, &str) -> Result<Result<(), String>, SingleFlightError>;

/// What:
///  Bring this worktree's index and files to the merge,
/// then move the branch this command checked,
/// and only that branch.
/// Why:
///   `git merge --ff-only <merge>` moves the branch `HEAD` names at the instant it runs,
/// so an ordinary `git switch` that finishes in the window between the check of the branch and the merge
///  moves a branch this command never checked.
/// Naming the branch is what removes that window:
/// the two-way update of the index and the worktree,
/// which is what Git's own fast-forward of a checkout does,
/// leaves the worktree holding the merge's files while the branch still holds the old tip,
/// and a `git switch` over that refuses;
/// the branch then moves by compare-and-swap against the tip this command checked,
/// so a branch that moved meanwhile is refused instead of overwritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git.readTreeUpdateIndex(['-u','-m',local,mergeOid]); await git.updateRef(['-m',message,branchRef,mergeOid,local]);
/// ```
pub(crate) fn move_the_named_branch(git: &PushGit, branch_ref: &str, local: &str, merge_oid: &str) -> Result<Result<(), String>, SingleFlightError> {
    // The branch tip must be the merge's ancestor: `read-tree` moves between any two trees, and `update-ref` moves a branch
    //  to any commit, so this is the check `--ff-only` made for the move.
    let ancestry = run_query(git, &argv(&["merge-base", "--is-ancestor", local, merge_oid]))?;
    match ancestry.code {
        Some(0) => {}
        // Git's own words for the refusal a `merge --ff-only` prints.
        Some(1) => return Ok(Err(String::from("Git refused the fast-forward: fatal: Not possible to fast-forward, aborting."))),
        other => {
            let how: String = match other {
                Some(code) => code.to_string(),
                None => String::from("on a signal"),
            };
            return Ok(Err(format!("Git refused the fast-forward: merge-base --is-ancestor exited {how}")));
        }
    }
    let checked_out = run_query(git, &argv(&["read-tree", "-u", "-m", local, merge_oid]))?;
    if checked_out.code != Some(0) {
        return Ok(Err(refusal_of(&checked_out.stderr)));
    }
    // The fast-forward carries no `--quiet`, so what Git said is kept in the journal (the service's log, or the wrapper's entry).
    record_streams(&["read-tree", "-u", "-m", local, merge_oid], &checked_out.stdout, &checked_out.stderr);
    let message: String = format!("merge: fast-forward {branch_ref} to the published reconcile merge");
    let moved = run_query(git, &argv(&["update-ref", "-m", message.as_str(), branch_ref, merge_oid, local]))?;
    if moved.code != Some(0) {
        return Ok(Err(refusal_of(&moved.stderr)));
    }
    record_streams(&["update-ref", branch_ref, merge_oid, local], &moved.stdout, &moved.stderr);
    return Ok(Ok(()));
}

/// What:
///  Git's fast-forward to the published merge,
/// with an intent record written first and settled after,
/// so a branch that moved in
///  the instant between Git's checkout and its ref update,
/// or a service that died there,
/// leaves the checkout as it was or put
///  back to the moved head,
/// never half merged.
/// Why:
///   The owner's rule "Auto merging should never fail."
/// (`SPEC.md`,
/// "Merges and commits never fail"):
/// the person's own merge and
///  the reconcile can both move the same branch,
/// and the loser's checkout must not stay beside a branch that does not hold it.
///  The same record and settling serve the person's merge (`merge_isolation_recovery.rs`),
/// and a record whose owner died is
///  settled by the next wrapper command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = await fastForwardRecorded({ git, branchRef, local, mergeOid }); // undefined when it moved
/// ```
pub(crate) fn fast_forward_recorded(git: &PushGit, branch_ref: &str, local: &str, merge_oid: &str, move_branch: BranchMove<'_>) -> Result<Result<(), String>, SingleFlightError> {
    let git_dir: Option<PathBuf> = {
        let named = run_query(git, &argv(&["rev-parse", "--absolute-git-dir"]))?;
        if named.code == Some(0) { Some(PathBuf::from(String::from_utf8_lossy(&named.stdout).trim())) } else { None }
    };
    let mut recorded: Option<(PathBuf, MergeIntent)> = None;
    if let Some(dir) = git_dir.as_deref() {
        recorded = record_merge_intent(dir, git, branch_ref, local, merge_oid);
    }
    let moved: Result<(), String> = move_branch(git, branch_ref, local, merge_oid)?;
    let Some((record, intent)) = recorded else {
        // Without a record the fast-forward runs as it always did, and Git's refusal is the reason.
        return Ok(moved);
    };
    if moved.is_ok() {
        if let Err(error) = remove_merge_intent(record.as_path()) {
            debug(TAG, &format!("the merge record could not be removed: {error}"));
        }
        return Ok(Ok(()));
    }
    let refusal: String = moved.err().unwrap_or_default();
    let place: RecoveryPlace = RecoveryPlace {
        git: GitContext { real_git: git.real_git.clone(), overlay: git.environment.clone(), global_prefix: Vec::new(), common_dir: git_dir.clone() },
        cwd: git.cwd.clone(),
        common_dir: git_dir.unwrap_or_default(),
    };
    return Ok(match settle_failed_merge(&place, record.as_path(), &intent) {
        Ok(MergeRecoveryAction::Completed) => Ok(()),
        Ok(_) => Err(refusal),
        Err(error) => Err(format!("{refusal} ({})", error.0)),
    });
}

/// The step the shipped reconcile moves with,
/// named once so a control can play the window with exactly what ships.
pub(crate) const SHIPPED_MOVE: BranchMove<'static> = &move_the_named_branch;

/// The reason line for a fast-forward Git refused:
/// its first line of words.
pub(crate) fn refusal_of(stderr: &[u8]) -> String {
    let detail = String::from_utf8_lossy(stderr);
    return format!("Git refused the fast-forward: {}", detail.lines().next().unwrap_or_default().trim());
}

/// What:
///  Fast-forward the pushed branch and this worktree to the published merge,
/// or the reason that was not done.
///  The success value says whether uncommitted work was left in place.
/// Why:
///   A branch left behind the remote makes the next commit rejected again,
/// and the person at the keyboard would have
///  to merge by hand;
/// the merge contains the branch tip,
/// so the move is a pure fast-forward.
///   It is attempted only where nothing can be lost or surprised:
/// the worktree has the branch checked out at the tip
///  that was pushed,
/// and no uncommitted change (tracked file,
/// staged entry,
/// or either name of a staged rename)
///  lies in a file the move writes.
///   Another session's work in progress therefore never holds the branch behind the remote,
/// while a move that
///  would write over such a file is left for the person.
///   The status read takes no optional lock,
/// and `git merge --ff-only` refuses,
/// without changing anything,
/// a move that
///  would overwrite an untracked file or a change made after the status read.
///   Every refusal is a reason for the report,
/// never a failure:
/// the merge is already published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function advanceLocalBranch({ git, branchRef, local, mergeOid }): Promise<{ reason?: string; keptChanges?: boolean }>;
/// ```
pub(crate) fn advance_local_branch(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Result<bool, String>, SingleFlightError> {
    return advance_local_branch_moving(git, branch_ref, local, merge_oid, SHIPPED_MOVE);
}

/// `advance_local_branch` with the step that does the move given,
/// so a control can play a `git switch` in the window between the checks and it.
pub(crate) fn advance_local_branch_moving(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
    move_branch: BranchMove<'_>,
) -> Result<Result<bool, String>, SingleFlightError> {
    let checked_out = run_query(git, &argv(&["symbolic-ref", "--quiet", "HEAD"]))?;
    if checked_out.code != Some(0) {
        return Ok(Err(String::from("HEAD is detached")));
    }
    if String::from_utf8_lossy(&checked_out.stdout).trim() != branch_ref {
        return Ok(Err(String::from(
            "this worktree has another branch checked out",
        )));
    }
    let head = run_query(git, &argv(&["rev-parse", "--verify", "HEAD^{commit}"]))?;
    if String::from_utf8_lossy(&head.stdout).trim() != local {
        return Ok(Err(String::from(
            "the branch moved after the commit landed",
        )));
    }
    let status_arguments = argv(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=no",
    ]);
    let status = run_query(git, &status_arguments)?;
    if status.code != Some(0) {
        return Ok(Err(String::from("the worktree status could not be read")));
    }
    let dirty = status_paths(&status.stdout);
    let kept: bool = !dirty.is_empty();
    if kept {
        let incoming = run_query(
            git,
            &argv(&["diff", "--name-only", "-z", "--no-renames", local, merge_oid]),
        )?;
        if incoming.code != Some(0) {
            return Ok(Err(String::from(
                "the files the merge changes could not be read",
            )));
        }
        let overlap = overlapping_paths(&dirty, &diff_paths(&incoming.stdout));
        if !overlap.is_empty() {
            return Ok(Err(format!(
                "the worktree has uncommitted changes to files the merge changes: {}",
                listed(&overlap)
            )));
        }
    }
    if let Err(reason) = fast_forward_recorded(git, branch_ref, local, merge_oid, move_branch)? {
        return Ok(Err(reason));
    }
    if kept {
        // The uncommitted work must read exactly as it did; the move is already done, so a difference is reported.
        let after = run_query(git, &status_arguments)?;
        if status_differs(&status.stdout, &after.stdout) {
            warn(
                TAG,
                "the worktree status differs after the fast-forward; compare it with the status before the commit",
            );
        }
    }
    return Ok(Ok(kept));
}

/// Write the record that says a fast-forward of the branch is about to happen,
/// or nothing when the record cannot be made.
pub(crate) fn record_merge_intent(dir: &Path, git: &PushGit, branch_ref: &str, local: &str, merge_oid: &str) -> Option<(PathBuf, MergeIntent)> {
    let owner = current_transaction_owner(random_uuid().ok()?.as_str(), now_iso().as_str()).ok()?;
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: git.cwd.to_string_lossy().into_owned(),
        target_ref: Some(branch_ref.to_owned()),
        old_oid: local.to_owned(),
        new_oid: merge_oid.to_owned(),
    };
    let record: PathBuf = write_merge_intent(dir, &intent).ok()?;
    return Some((record, intent));
}
