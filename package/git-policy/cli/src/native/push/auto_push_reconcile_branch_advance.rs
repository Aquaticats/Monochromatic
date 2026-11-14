//! What:
//!  The move of the branch a push sent after its merge was published:
//!  which worktree has it checked out,
//! the fast-forward there,
//! the record that guards it,
//! the check that
//!  uncommitted work is not in the way,
//!  and the words of a refusal.
//! Why:
//!  The branch follows the published merge only where nothing can be lost;
//! this file holds those checks apart from the
//!  retry loop in `auto_push_reconcile.rs`,
//! and it moves the branch the push sent and no other:
//! the command's own checkout,
//!  another branch a worktree happens to have, and every branch nobody pushed stay exactly where they were.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const advanced = await advancePushedBranch(git, branchRef, local, mergeOid); // here | elsewhere | nowhere | not
//! ```

use crate::auto_push_single_flight::{
    PushGit, SingleFlightError, run_query_awaiting,
};
use crate::auto_push_dirty_paths::{diff_paths, listed, overlapping_paths, status_paths};
use crate::diagnostic_log::{debug, warn};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use crate::inner_git_words::record_streams;
use crate::iso_time::now_iso;
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent, write_merge_intent};
use crate::merge_isolation_recovery::{MergeRecoveryAction, RecoveryPlace, settle_failed_merge_awaiting};
use crate::random_id::random_uuid;
use crate::transaction_git::GitContext;
use crate::transaction_owner::current_transaction_owner;
use crate::auto_push_reconcile_basics_and_limits::{TAG, argv};
/// What a move's checkout came to, and the step that makes a checkout agree with its own branch.
pub(crate) use crate::auto_push_reconcile_branch_checkout::Checkout;
use crate::auto_push_reconcile_branch_checkout::{bring_the_checkout_to_its_branch_awaiting, the_checkout_after_the_move_awaiting};

/// What:
///  What a move of the branch came to.
/// Why:
///   The caller words one report line,
/// and it needs both facts a move settles:
/// what became of the checkout it wrote,
///  and whether the person's uncommitted work was left exactly as it was.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Moved {
    /// What became of the checkout the move wrote.
    pub(crate) checkout: Checkout,
    /// Whether uncommitted work of the person's was left as it was.
    pub(crate) kept: bool,
}
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
    /// answer why not when Git refused it,
    /// and answer what became of the checkout when it did not.
    fn move_branch<'a>(
        &'a self,
        git: &'a PushGit,
        branch_ref: &'a str,
        local: &'a str,
        merge_oid: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Result<Checkout, String>, SingleFlightError>> + Send + 'a>>;
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
    ) -> Pin<Box<dyn Future<Output = Result<Result<Checkout, String>, SingleFlightError>> + Send + 'a>> {
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
///  Bring this worktree's index and files to the merge,
/// then move the branch this command checked,
/// and only that branch,
/// and answer what became of the checkout.
/// Why:
///   `git merge --ff-only <merge>` moves the branch `HEAD` names at the instant it runs,
/// so an ordinary `git switch` that finishes in the window between the check of the branch and the merge
///  moves a branch this command never checked.
/// Naming the branch is what removes that window:
/// the two-way update of the index and the worktree,
/// which is what Git's own fast-forward of a checkout does,
/// leaves the worktree holding the merge's files while the branch still holds the old tip,
/// and the branch moves by compare-and-swap against the tip this command checked,
/// so a branch that moved meanwhile is refused instead of overwritten.
///   The checkout is written only while this worktree still shows the branch that was checked,
/// and a switch that landed inside the write has the merge's paths put back onto the branch it shows,
///  which is what `auto_push_reconcile_branch_checkout.rs` asks and writes,
/// because the commit a worktree shows is not this command's to rewrite.
///   A switch can also land in the instant between the last look at the checkout and the write of the reference,
/// and a switch onto a branch whose tree matches the old tip on the merge's paths does not refuse:
/// it carries the merge's content over as a staged change.
/// The comment here said such a switch refuses, and it does not
///  (a review of commit `b0c133f42b`, its finding).
/// So the checkout is looked at once more after the reference moved,
/// and the tree of the branch it shows is written over the paths this move wrote:
///  the merge's for the pushed branch, and that branch's own content for any other.
/// The answer says which of those it found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git.readTreeUpdateIndex(['-u','-m',local,mergeOid]); await git.updateRef(['-m',message,branchRef,mergeOid,local]); await theCheckoutAfterTheMove();
/// ```
pub(crate) fn move_the_named_branch<'a>(git: &'a PushGit, branch_ref: &'a str, local: &'a str, merge_oid: &'a str) -> Pin<Box<dyn Future<Output = Result<Result<Checkout, String>, SingleFlightError>> + Send + 'a>> {
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
            // What: a switch that landed inside the write is given the merge's files, and they go back onto the branch it shows.
            // Why: the reference has not moved yet, so a refusal here leaves the branch where it was and the checkout as the switch
            //      left it, and the report says the branch is not at the merge.
            if !still_on_the_branch(git, branch_ref, local).await? {
                match bring_the_checkout_to_its_branch_awaiting(git, local, merge_oid).await? {
                    Ok(None) => {}
                    Ok(Some(paths)) => {
                        return Ok(Err(format!(
                            "another command checked out another branch while this fast-forward wrote its files, and the merge's changes to {} are still in that checkout and were left as they are",
                            listed(&paths)
                        )));
                    }
                    Err(reason) => return Ok(Err(reason)),
                }
            }
        }
        #[cfg(test)]
        crate::auto_push_reconcile_branch_checkout::play_before_the_branch_moves();
        let message: String = format!("merge: fast-forward {branch_ref} to the published reconcile merge");
        let moved = run_query_awaiting(git, &argv(&["update-ref", "-m", message.as_str(), branch_ref, merge_oid, local])).await?;
        if moved.code != Some(0) {
            return Ok(Err(refusal_of(&moved.stderr)));
        }
        record_streams(&["update-ref", branch_ref, merge_oid, local], &moved.stdout, &moved.stderr);
        // What: the checkout is looked at once more, now that the branch is at the merge, and made to agree with its own branch.
        // Why: a switch can land in the instant between the last look and this write, and it carries the merge's content onto the
        //      branch it goes to instead of refusing (a review of commit `b0c133f42b`, its finding). No lock is taken: the
        //      reference's compare-and-swap has decided who moves the branch, and Git's index lock serialises the write below.
        return Ok(Ok(the_checkout_after_the_move_awaiting(git, branch_ref, local, merge_oid).await?));
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
pub(crate) async fn fast_forward_recorded(git: &PushGit, branch_ref: &str, local: &str, merge_oid: &str, move_branch: &dyn BranchMove) -> Result<Result<Checkout, String>, SingleFlightError> {
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
    let moved: Result<Checkout, String> = move_branch.move_branch(git, branch_ref, local, merge_oid).await?;
    let Some((record, intent)) = recorded else {
        // Without a record the fast-forward runs as it always did, and Git's refusal is the reason.
        return Ok(moved);
    };
    if let Ok(checkout) = &moved {
        if let Err(error) = remove_merge_intent(record.as_path()) {
            debug(TAG, &format!("the merge record could not be removed: {error}"));
        }
        return Ok(Ok(checkout.clone()));
    }
    let refusal: String = moved.err().unwrap_or_default();
    let place: RecoveryPlace = RecoveryPlace {
        git: GitContext { real_git: git.real_git.clone(), overlay: git.environment.clone(), global_prefix: Vec::new(), common_dir: git_dir.clone() },
        cwd: git.cwd.clone(),
        common_dir: git_dir.clone().unwrap_or_default(),
        git_dir: git_dir.unwrap_or_default(),
    };
    // The settle reads Git, so it is the awaited face: a task can start no child through the synchronous one,
    //  which registers its step's pipes by entering the runtime.
    return Ok(match settle_failed_merge_awaiting(&place, record.as_path(), &intent, &mut std::io::stderr()).await {
        // The recovery finished the merge it was recorded for, and it installs the merge's tree together with the person's
        //  work, so the checkout shows the branch it is on.
        Ok(MergeRecoveryAction::Completed) => Ok(Checkout::Followed),
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
/// Why a control still calls it:
///   The loop fast-forwards through the awaited face (`advance_local_branch_awaiting`),
/// and the control that compares the two faces asks both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function advanceLocalBranch({ git, branchRef, local, mergeOid }): Promise<{ reason?: string; keptChanges?: boolean }>;
/// ```
#[cfg(test)]
pub(crate) fn advance_local_branch(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Result<Moved, String>, SingleFlightError> {
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
) -> Result<Result<Moved, String>, SingleFlightError> {
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
) -> Result<Result<Moved, String>, SingleFlightError> {
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
        return Ok(Err(String::from(BRANCH_MOVED)));
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
    let checkout: Checkout = match fast_forward_recorded(git, branch_ref, local, merge_oid, move_branch).await? {
        Ok(checkout) => checkout,
        Err(reason) => return Ok(Err(reason)),
    };
    if kept && checkout == Checkout::Followed {
        // The uncommitted work must read exactly as it did; the move is already done, so a difference is reported.
        let after = run_query_awaiting(git, &status_arguments).await?;
        if status_differs(&status.stdout, &after.stdout) {
            warn(
                TAG,
                "the worktree status differs after the fast-forward; compare it with the status before the commit",
            );
        }
    }
    return Ok(Ok(Moved { checkout, kept }));
}

/// What:
///  Where the published merge left the branch the push sent,
/// and what a person may have to do about it.
/// Why:
///  The report says which of these happened,
/// so a person reading one line learns where their branch is now;
///  a reason and an advice travel together,
/// because a refusal that names no way forward is a dead end.
///  Every move of the branch answers with this,
/// whether it ran in the worktree this command is in,
///  in another one, or in none,
/// so the loop words each of them the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Advanced = { kind: 'here'; kept: boolean } | { kind: 'elsewhere'; worktree: string; kept: boolean } | { kind: 'nowhere' } | { kind: 'not'; reason: string; advice: string };
/// ```
pub(crate) enum Advanced {
    /// The branch and this worktree, which had it checked out, moved to the merge;
    /// `kept` says uncommitted work was left exactly as it was.
    Here {
        /// Whether uncommitted work was left in place.
        kept: bool,
    },
    /// The branch and the worktree at this path, which had it checked out, moved to the merge.
    Elsewhere {
        /// The worktree that had the branch checked out.
        worktree: PathBuf,
        /// Whether that worktree's uncommitted work was left in place.
        kept: bool,
    },
    /// No worktree had the branch checked out, so only the branch moved and no file was written.
    Nowhere,
    /// The branch was not moved,
    /// and the reason and the way forward are here.
    Not {
        /// Why the branch is not at the merge.
        reason: String,
        /// What moves it there once nothing is in the way.
        advice: String,
    },
    /// The branch is at the merge, and the worktree that had it checked out checked another branch out while the move ran,
    /// so that worktree shows that branch and holds nothing of the merge.
    WorktreeWentTo {
        /// The worktree that had the branch checked out.
        worktree: PathBuf,
        /// The branch, or the detached commit, that worktree shows instead.
        shows: Option<String>,
    },
    /// The same, and something of the merge is still in that worktree.
    WorktreeHolding {
        /// The worktree that had the branch checked out.
        worktree: PathBuf,
        /// The branch, or the detached commit, that worktree shows.
        shows: Option<String>,
        /// The paths of the merge that are still there,
        /// empty when the checkout could not be read.
        paths: Vec<Vec<u8>>,
        /// Why they are still there.
        reason: String,
    },
}

/// What:
///  How a person moves a branch that the reconcile left where it was.
/// Why:
///   The merge is published,
/// so the branch follows it by a fast-forward of the merge commit itself,
/// which needs no remote-tracking ref and cannot carry anything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const advice = (mergeOid, where) => `move it with \`git merge --ff-only ${mergeOid}\` ${where} once nothing is in the way`;
/// ```
pub(crate) fn advice_to_move(merge_oid: &str, where_to: &str) -> String {
    return format!("move it with `git merge --ff-only {merge_oid}` {where_to} once nothing is in the way");
}

/// The words a reader of the report gets for one branch the push did not move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const branchMoved = 'the branch moved since the push was made';
/// ```
pub(crate) const BRANCH_MOVED: &str = "the branch moved since the push was made";

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
        carried: None,
    };
    let record: PathBuf = write_merge_intent(dir, &intent).ok()?;
    return Some((record, intent));
}
