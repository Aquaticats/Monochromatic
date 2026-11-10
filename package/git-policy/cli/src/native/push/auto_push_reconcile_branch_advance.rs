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
    PushGit, SingleFlightError, run_query_awaiting,
};
use crate::auto_push_reconcile_branch_advance_settling::settle_failed_merge_awaiting;
use crate::auto_push_dirty_paths::{diff_paths, listed, overlapping_paths, status_paths};
use crate::diagnostic_log::{debug, warn};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use crate::inner_git_words::record_streams;
use crate::iso_time::now_iso;
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent, write_merge_intent};
use crate::merge_isolation_recovery::{MergeRecoveryAction, RecoveryPlace};
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
///  The step starts Git,
/// so it hands back a future its caller awaits rather than a value:
/// a start of a task's work is refused from inside the runtime
/// (`merge/merge_second_pass_git_runner.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface BranchMove { moveBranch(git, branchRef, local, mergeOid): Promise<Result<void, string>> }
/// ```
pub(crate) trait BranchMove: Send + Sync {
    /// Bring the checkout and the named branch to the merge,
    /// and answer why not when Git refused it.
    fn move_branch<'a>(
        &'a self,
        git: &'a PushGit,
        branch_ref: &'a str,
        local: &'a str,
        merge_oid: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Result<(), String>, SingleFlightError>> + Send + 'a>>;
}

/// The step the shipped reconcile moves with.
pub(crate) struct MoveTheNamedBranch;

impl BranchMove for MoveTheNamedBranch {
    fn move_branch<'a>(
        &'a self,
        git: &'a PushGit,
        branch_ref: &'a str,
        local: &'a str,
        merge_oid: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Result<(), String>, SingleFlightError>> + Send + 'a>> {
        return move_the_named_branch(git, branch_ref, local, merge_oid);
    }
}

/// What:
///  Whether this worktree still has `branch_ref` checked out at `local`.
/// Why:
///   The index and the files of a worktree belong to no branch:
/// any command of that worktree can check another one out,
/// and
///  that is what the write below must not mistake for its own.
/// The question is asked either side of the write,
/// so the window between them is the only
///  one a switch can still land in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = await git.symbolicRefQuiet('HEAD'); const tip = await git.revParse('HEAD^{commit}'); return head === branchRef && tip === local;
/// ```
async fn still_on_the_branch(git: &PushGit, branch_ref: &str, local: &str) -> Result<bool, SingleFlightError> {
    let checked_out = run_query_awaiting(git, &argv(&["symbolic-ref", "--quiet", "HEAD"])).await?;
    if checked_out.code != Some(0) || String::from_utf8_lossy(&checked_out.stdout).trim() != branch_ref {
        return Ok(false);
    }
    let head = run_query_awaiting(git, &argv(&["rev-parse", "--verify", "HEAD^{commit}"])).await?;
    if head.code != Some(0) {
        return Ok(false);
    }
    return Ok(String::from_utf8_lossy(&head.stdout).trim() == local);
}

// The step a control plays in the instant between the look at the checkout and the write of its files, as another
// command's `git switch` finishing there; nothing outside a control.
#[cfg(test)]
thread_local! {
    /// The step of the control running on this thread.
    static BETWEEN_LOOK_AND_WRITE: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// Plays `step` in the instant between the look at the checkout and the write,
/// until the guard drops.
#[cfg(test)]
pub(crate) fn between_look_and_write_in_a_control(step: impl Fn() + 'static) -> BetweenGuard {
    BETWEEN_LOOK_AND_WRITE.with(|slot| { *slot.borrow_mut() = Some(Box::new(step)); });
    return BetweenGuard;
}

/// Ends the control's step when dropped.
#[cfg(test)]
pub(crate) struct BetweenGuard;

#[cfg(test)]
impl Drop for BetweenGuard {
    fn drop(&mut self) {
        BETWEEN_LOOK_AND_WRITE.with(|slot| { *slot.borrow_mut() = None; });
    }
}

/// Plays the control's step,
/// if one is set.
#[cfg(test)]
fn play_between_look_and_write() {
    // The step borrows nothing of the caller, so it runs here rather than through a reference that would hold the borrow.
    let played: Option<Box<dyn Fn()>> = BETWEEN_LOOK_AND_WRITE.with(|slot| { return slot.borrow_mut().take(); });
    if let Some(step) = played {
        step();
        BETWEEN_LOOK_AND_WRITE.with(|slot| { *slot.borrow_mut() = Some(step); });
    }
}

/// What:
///  The tree `revision` names,
/// or nothing when it names none.
/// Why:
///   Putting the merge's files back must name the tree of the commit the checkout shows,
/// not a branch tip that
///  may hold the merge already,
/// and a revision that names no commit is said rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = await git.revParseQuiet(`${revision}^{tree}`); return tree === undefined ? undefined : tree.trim();
/// ```
async fn tree_of(git: &PushGit, revision: &str) -> Result<Option<String>, SingleFlightError> {
    let asked = run_query_awaiting(git, &argv(&["rev-parse", "--verify", format!("{revision}^{{tree}}").as_str()])).await?;
    if asked.code != Some(0) {
        return Ok(None);
    }
    return Ok(Some(String::from_utf8_lossy(&asked.stdout).trim().to_owned()));
}

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
///   The checkout is written only while this worktree still shows the branch that was checked:
/// a `git switch` that
///  finished first leaves another branch's files alone and the branch below moves without it,
/// and one that finished inside the
///  write gets its files put back,
/// because the commit a worktree shows is not this command's to rewrite.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git.readTreeUpdateIndex(['-u','-m',local,mergeOid]); await git.updateRef(['-m',message,branchRef,mergeOid,local]);
/// ```
pub(crate) fn move_the_named_branch<'a>(git: &'a PushGit, branch_ref: &'a str, local: &'a str, merge_oid: &'a str) -> Pin<Box<dyn Future<Output = Result<Result<(), String>, SingleFlightError>> + Send + 'a>> {
    return Box::pin(async move {
        // The branch tip must be the merge's ancestor: `read-tree` moves between any two trees, and `update-ref` moves a branch
        //  to any commit, so this is the check `--ff-only` made for the move.
        let ancestry = run_query_awaiting(git, &argv(&["merge-base", "--is-ancestor", local, merge_oid])).await?;
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
        if !still_on_the_branch(git, branch_ref, local).await? {
            debug(TAG, "another branch is checked out here, so the fast-forward moves the branch and leaves that checkout as it is");
        } else {
            #[cfg(test)]
            play_between_look_and_write();
            let checked_out = run_query_awaiting(git, &argv(&["read-tree", "-u", "-m", local, merge_oid])).await?;
            if checked_out.code != Some(0) {
                return Ok(Err(refusal_of(&checked_out.stderr)));
            }
            // The fast-forward carries no `--quiet`, so what Git said is kept in the journal (the service's log, or the wrapper's entry).
            record_streams(&["read-tree", "-u", "-m", local, merge_oid], &checked_out.stdout, &checked_out.stderr);
            // The question again, after the write: a `git switch` that landed inside it was given the merge's files, and they go
            //  back before the branch moves below. They go back to the tree this fast-forward started from, never to the checkout's
            //  whole tree: `read-tree -u -m` changes only the paths where the merge and that tree differ, which are exactly the paths
            //  this command wrote, so a staged edit of the person's on any other path is left alone.
            if !still_on_the_branch(git, branch_ref, local).await? {
                let merged: Option<String> = tree_of(git, merge_oid).await?;
                let shown_here: Option<String> = tree_of(git, "HEAD").await?;
                if merged.is_some() && merged == shown_here {
                    // The checkout already shows the merge's tree: nothing of this fast-forward is in it.
                    debug(TAG, "the branch checked out here already holds the merge's tree, so the fast-forward has nothing to put back");
                } else {
                    let put_back = run_query_awaiting(git, &argv(&["read-tree", "-u", "-m", merge_oid, local])).await?;
                    if put_back.code != Some(0) {
                        return Ok(Err(format!(
                            "another command checked out another branch while this fast-forward wrote its files, and putting them back failed: {}",
                            refusal_of(&put_back.stderr)
                        )));
                    }
                    record_streams(&["read-tree", "-u", "-m", merge_oid, local], &put_back.stdout, &put_back.stderr);
                    warn(
                        TAG,
                        "another command checked out another branch while the fast-forward wrote its files; the merge's own paths were put back to the tree it started from and only the branch moved",
                    );
                }
            }
        }
        let message: String = format!("merge: fast-forward {branch_ref} to the published reconcile merge");
        let moved = run_query_awaiting(git, &argv(&["update-ref", "-m", message.as_str(), branch_ref, merge_oid, local])).await?;
        if moved.code != Some(0) {
            return Ok(Err(refusal_of(&moved.stderr)));
        }
        record_streams(&["update-ref", branch_ref, merge_oid, local], &moved.stdout, &moved.stderr);
        return Ok(Ok(()));
    });
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
pub(crate) async fn fast_forward_recorded(git: &PushGit, branch_ref: &str, local: &str, merge_oid: &str, move_branch: &dyn BranchMove) -> Result<Result<(), String>, SingleFlightError> {
    let git_dir: Option<PathBuf> = {
        let named = run_query_awaiting(git, &argv(&["rev-parse", "--absolute-git-dir"])).await?;
        // Git prints the directory's bytes and ends them with one line break that belongs to no path, so the record is
        //  written in the worktree a path that is not text names and not in a directory made from its text.
        let printed: &[u8] = named.stdout.strip_suffix(b"\n".as_slice()).unwrap_or(named.stdout.as_slice());
        if named.code == Some(0) { Some(PathBuf::from(crate::path_bytes::os_from_path_bytes(printed))) } else { None }
    };
    let mut recorded: Option<(PathBuf, MergeIntent)> = None;
    if let Some(dir) = git_dir.as_deref() {
        recorded = record_merge_intent(dir, git, branch_ref, local, merge_oid);
    }
    let moved: Result<(), String> = move_branch.move_branch(git, branch_ref, local, merge_oid).await?;
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
        common_dir: git_dir.clone().unwrap_or_default(),
        git_dir: git_dir.unwrap_or_default(),
    };
    return Ok(match settle_failed_merge_awaiting(place, record, intent).await {
        Ok(MergeRecoveryAction::Completed) => Ok(()),
        Ok(_) => Err(refusal),
        Err(error) => Err(format!("{refusal} ({})", error.error.0)),
    });
}

/// The step the shipped reconcile moves with,
/// named once so a control can play the window with exactly what ships.
pub(crate) const SHIPPED_MOVE: MoveTheNamedBranch = MoveTheNamedBranch;

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
    return match crate::lazy_runtime::run_on_runtime(advance_local_branch_awaiting(git, branch_ref, local, merge_oid)) {
        Ok(moved) => moved,
        Err(refusal) => Err(SingleFlightError::Start(std::io::Error::other(refusal.to_string()))),
    };
}

/// What:
///  The same fast-forward,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   Every question the checks ask and every write the move makes is a start of Git,
/// and the synchronous face enters the runtime for each of them and is refused from there.
/// The checks,
/// the window and the record are the synchronous face's,
/// so the two cannot decide differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moved = await advanceLocalBranchOnTheRuntime({ git, branchRef, local, mergeOid });
/// ```
pub(crate) async fn advance_local_branch_awaiting(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Result<bool, String>, SingleFlightError> {
    return advance_local_branch_moving(git, branch_ref, local, merge_oid, &SHIPPED_MOVE).await;
}

/// `advance_local_branch` with the step that does the move given,
/// so a control can play a `git switch` in the window between the checks and it.
pub(crate) async fn advance_local_branch_moving(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
    move_branch: &dyn BranchMove,
) -> Result<Result<bool, String>, SingleFlightError> {
    let checked_out = run_query_awaiting(git, &argv(&["symbolic-ref", "--quiet", "HEAD"])).await?;
    if checked_out.code != Some(0) {
        return Ok(Err(String::from("HEAD is detached")));
    }
    if String::from_utf8_lossy(&checked_out.stdout).trim() != branch_ref {
        return Ok(Err(String::from(
            "this worktree has another branch checked out",
        )));
    }
    let head = run_query_awaiting(git, &argv(&["rev-parse", "--verify", "HEAD^{commit}"])).await?;
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
    let status = run_query_awaiting(git, &status_arguments).await?;
    if status.code != Some(0) {
        return Ok(Err(String::from("the worktree status could not be read")));
    }
    let dirty = status_paths(&status.stdout);
    let kept: bool = !dirty.is_empty();
    if kept {
        let incoming = run_query_awaiting(
            git,
            &argv(&["diff", "--name-only", "-z", "--no-renames", local, merge_oid]),
        )
        .await?;
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
    if let Err(reason) = fast_forward_recorded(git, branch_ref, local, merge_oid, move_branch).await? {
        return Ok(Err(reason));
    }
    if kept {
        // The uncommitted work must read exactly as it did; the move is already done, so a difference is reported.
        let after = run_query_awaiting(git, &status_arguments).await?;
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
        repository_root: git.cwd.to_path_buf(),
        target_ref: Some(branch_ref.to_owned()),
        old_oid: local.to_owned(),
        new_oid: merge_oid.to_owned(),
        side_ref: None,
    };
    let record: PathBuf = write_merge_intent(dir, &intent).ok()?;
    return Some((record, intent));
}
