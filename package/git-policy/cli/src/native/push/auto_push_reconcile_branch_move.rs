//! What:
//!  The step that moves the branch to the published merge:
//!  the worktree's files first,
//!  then a compare-and-swap of the branch,
//!  and the window a `git switch` of another command opens between them.
//! Why:
//!  `git merge --ff-only` moves the branch `HEAD` names at the instant it runs,
//!  so a switch that finishes in the window between the check and the merge
//!  moves a branch this command never checked.
//!  Naming the branch removes that window,
//!  and the step is a value so a control can play the switch inside it
//!  (`advance_local_branch_moving`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface BranchMove { moveBranch(git, branchRef, local, mergeOid): Promise<Result<void, string>> }
//! const SHIPPED_MOVE: BranchMove;
//! ```

/// Import the module's tag and the runner's argument builder.
use crate::auto_push_reconcile_basics_and_limits::{
    TAG,
    argv,
};
/// Import the probe's Git runner and its awaited queries.
use crate::auto_push_single_flight::{
    PushGit,
    SingleFlightError,
    run_query_awaiting,
};
/// Import the diagnostic log of the wrapper.
use crate::diagnostic_log::{
    debug,
    warn,
};
/// Import the record of what a Git step said,
/// so the fast-forward's own words reach the service's log.
use crate::inner_git_words::record_streams;
/// Import the future a step's caller awaits,
/// which keeps a task's work out of the synchronous face,
/// and the box a borrowed future is handed back in.
use std::future::Future;
use std::pin::Pin;

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
    ) -> Pin<Box<dyn Future<Output = Result<Result<(), String>, SingleFlightError>> + Send + 'a>>
    {
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
async fn still_on_the_branch(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
) -> Result<bool, SingleFlightError> {
    let checked_out = run_query_awaiting(git, &argv(&["symbolic-ref", "--quiet", "HEAD"])).await?;
    if checked_out.code != Some(0)
        || String::from_utf8_lossy(&checked_out.stdout).trim() != branch_ref
    {
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
    BETWEEN_LOOK_AND_WRITE.with(|slot| {
        *slot.borrow_mut() = Some(Box::new(step));
    });
    return BetweenGuard;
}

/// Ends the control's step when dropped.
#[cfg(test)]
pub(crate) struct BetweenGuard;

#[cfg(test)]
impl Drop for BetweenGuard {
    fn drop(&mut self) {
        BETWEEN_LOOK_AND_WRITE.with(|slot| {
            *slot.borrow_mut() = None;
        });
    }
}

/// Plays the control's step,
/// if one is set.
#[cfg(test)]
fn play_between_look_and_write() {
    // The step borrows nothing of the caller, so it runs here rather than through a reference that would hold the borrow.
    let played: Option<Box<dyn Fn()>> = BETWEEN_LOOK_AND_WRITE.with(|slot| {
        return slot.borrow_mut().take();
    });
    if let Some(step) = played {
        step();
        BETWEEN_LOOK_AND_WRITE.with(|slot| {
            *slot.borrow_mut() = Some(step);
        });
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
async fn tree_of(
    git: &PushGit,
    revision: &str,
) -> Result<Option<String>, SingleFlightError> {
    let asked = run_query_awaiting(
        git,
        &argv(&[
            "rev-parse",
            "--verify",
            format!("{revision}^{{tree}}").as_str(),
        ]),
    )
    .await?;
    if asked.code != Some(0) {
        return Ok(None);
    }
    return Ok(Some(
        String::from_utf8_lossy(&asked.stdout).trim().to_owned(),
    ));
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
pub(crate) fn move_the_named_branch<'a>(
    git: &'a PushGit,
    branch_ref: &'a str,
    local: &'a str,
    merge_oid: &'a str,
) -> Pin<Box<dyn Future<Output = Result<Result<(), String>, SingleFlightError>> + Send + 'a>> {
    return Box::pin(async move {
        // The branch tip must be the merge's ancestor: `read-tree` moves between any two trees, and `update-ref` moves a branch
        //  to any commit, so this is the check `--ff-only` made for the move.
        let ancestry = run_query_awaiting(
            git,
            &argv(&["merge-base", "--is-ancestor", local, merge_oid]),
        )
        .await?;
        match ancestry.code {
            Some(0) => {}
            // Git's own words for the refusal a `merge --ff-only` prints.
            Some(1) => {
                return Ok(Err(String::from(
                    "Git refused the fast-forward: fatal: Not possible to fast-forward, aborting.",
                )));
            }
            other => {
                let how: String = match other {
                    Some(code) => code.to_string(),
                    None => String::from("on a signal"),
                };
                return Ok(Err(format!(
                    "Git refused the fast-forward: merge-base --is-ancestor exited {how}"
                )));
            }
        }
        if !still_on_the_branch(git, branch_ref, local).await? {
            debug(
                TAG,
                "another branch is checked out here, so the fast-forward moves the branch and leaves that checkout as it is",
            );
        } else {
            #[cfg(test)]
            play_between_look_and_write();
            let checked_out =
                run_query_awaiting(git, &argv(&["read-tree", "-u", "-m", local, merge_oid]))
                    .await?;
            if checked_out.code != Some(0) {
                return Ok(Err(refusal_of(&checked_out.stderr)));
            }
            // The fast-forward carries no `--quiet`, so what Git said is kept in the journal (the service's log, or the wrapper's entry).
            record_streams(
                &["read-tree", "-u", "-m", local, merge_oid],
                &checked_out.stdout,
                &checked_out.stderr,
            );
            // The question again, after the write: a `git switch` that landed inside it was given the merge's files, and they go
            //  back before the branch moves below. They go back to the tree this fast-forward started from, never to the checkout's
            //  whole tree: `read-tree -u -m` changes only the paths where the merge and that tree differ, which are exactly the paths
            //  this command wrote, so a staged edit of the person's on any other path is left alone.
            if !still_on_the_branch(git, branch_ref, local).await? {
                let merged: Option<String> = tree_of(git, merge_oid).await?;
                let shown_here: Option<String> = tree_of(git, "HEAD").await?;
                if merged.is_some() && merged == shown_here {
                    // The checkout already shows the merge's tree: nothing of this fast-forward is in it.
                    debug(
                        TAG,
                        "the branch checked out here already holds the merge's tree, so the fast-forward has nothing to put back",
                    );
                } else {
                    let put_back = run_query_awaiting(
                        git,
                        &argv(&["read-tree", "-u", "-m", merge_oid, local]),
                    )
                    .await?;
                    if put_back.code != Some(0) {
                        return Ok(Err(format!(
                            "another command checked out another branch while this fast-forward wrote its files, and putting them back failed: {}",
                            refusal_of(&put_back.stderr)
                        )));
                    }
                    record_streams(
                        &["read-tree", "-u", "-m", merge_oid, local],
                        &put_back.stdout,
                        &put_back.stderr,
                    );
                    warn(
                        TAG,
                        "another command checked out another branch while the fast-forward wrote its files; the merge's own paths were put back to the tree it started from and only the branch moved",
                    );
                }
            }
        }
        let message: String =
            format!("merge: fast-forward {branch_ref} to the published reconcile merge");
        let moved = run_query_awaiting(
            git,
            &argv(&[
                "update-ref",
                "-m",
                message.as_str(),
                branch_ref,
                merge_oid,
                local,
            ]),
        )
        .await?;
        if moved.code != Some(0) {
            return Ok(Err(refusal_of(&moved.stderr)));
        }
        record_streams(
            &["update-ref", branch_ref, merge_oid, local],
            &moved.stdout,
            &moved.stderr,
        );
        return Ok(Ok(()));
    });
}

/// The step the shipped reconcile moves with,
/// named once so a control can play the window with exactly what ships.
pub(crate) const SHIPPED_MOVE: MoveTheNamedBranch = MoveTheNamedBranch;

/// The reason line for a fast-forward Git refused:
/// its first line of words.
pub(crate) fn refusal_of(stderr: &[u8]) -> String {
    let detail = String::from_utf8_lossy(stderr);
    return format!(
        "Git refused the fast-forward: {}",
        detail.lines().next().unwrap_or_default().trim()
    );
}
