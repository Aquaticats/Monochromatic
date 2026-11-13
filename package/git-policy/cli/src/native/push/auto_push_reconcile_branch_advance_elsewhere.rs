//! What:
//!  The move of the branch a push sent where the worktree that has it checked out is not this one,
//!  and the entry that decides which of the three moves the branch gets.
//! Why:
//!  A branch is checked out in at most one worktree,
//! and only the worktree that holds it has files to write when it moves;
//!  a branch no live worktree holds moves as a ref (`auto_push_reconcile_branch_advance_without_a_checkout.rs`).
//!  The move here is the one of the worktree this command runs in
//!  (`auto_push_reconcile_branch_advance.rs`, which this file calls):
//! the same two-way index and file update,
//!  the same compare-and-swap of the ref,
//! and the same record that guards a killed move,
//!  run with every Git start made in the worktree that has to move.
//!  A pushed branch that happened to share the command's checkout tip used to be moved over a
//!  merge of another branch;
//! this file is where that is decided by the ref Git says was pushed and by nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const advanced = await advancePushedBranch(git, branchRef, local, mergeOid); // here | elsewhere | nowhere | not
//! ```

use crate::auto_push_single_flight::{
    PushGit, SingleFlightError, run_query_awaiting,
};
use crate::auto_push_dirty_paths::{diff_paths, listed, overlapping_paths, status_paths};
/// The here-path's move, the answers a move gives, and the words of a refusal.
use crate::auto_push_reconcile_branch_advance::{
    Advanced, BRANCH_MOVED, SHIPPED_MOVE, advance_local_branch_awaiting, advice_to_move, fast_forward_recorded,
    status_differs,
};
/// Which worktree holds the branch, whether it is this one, and how to run Git in it.
use crate::auto_push_reconcile_branch_worktree::{git_at, is_this_worktree, live_holder_awaiting};
/// The move of a branch no live worktree holds.
use crate::auto_push_reconcile_branch_advance_without_a_checkout::{NowhereMove, move_the_ref_only_awaiting};
use crate::auto_push_reconcile_basics_and_limits::{TAG, argv};
use crate::diagnostic_log::warn;
use std::path::Path;

/// What:
///  Move the branch and the worktree that has it checked out,
/// where that worktree is not this one,
/// or say why it was not moved.
/// Why:
///   The same checks the current worktree gets apply there:
/// the worktree still has the branch at the tip the push sent,
///  and none of its uncommitted work lies in a file the move writes.
/// The move is the two-way update and the compare-and-swap the current worktree's move is,
/// run in that
///  worktree,
/// so a worktree that its own command checks out another branch in keeps that branch's files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function advanceInAnotherWorktree(there, place, branchRef, local, mergeOid): Promise<Advanced>;
/// ```
async fn advance_in_another_worktree(
    there: &PushGit,
    place: &Path,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Advanced, SingleFlightError> {
    let named: String = format!("the worktree at {}", place.display());
    let where_to: String = format!("in {named}");
    let checked_out = run_query_awaiting(there, &argv(&["symbolic-ref", "--quiet", "HEAD"])).await?;
    if checked_out.code != Some(0) || String::from_utf8_lossy(&checked_out.stdout).trim() != branch_ref {
        return Ok(Advanced::Not {
            reason: format!("{named} checked out another branch while the merge was published"),
            advice: advice_to_move(merge_oid, where_to.as_str()),
        });
    }
    let head = run_query_awaiting(there, &argv(&["rev-parse", "--verify", "HEAD^{commit}"])).await?;
    if head.code != Some(0) || String::from_utf8_lossy(&head.stdout).trim() != local {
        return Ok(Advanced::Not {
            reason: String::from(BRANCH_MOVED),
            advice: advice_to_move(merge_oid, where_to.as_str()),
        });
    }
    let status_arguments = argv(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=no",
    ]);
    let status = run_query_awaiting(there, &status_arguments).await?;
    if status.code != Some(0) {
        return Ok(Advanced::Not {
            reason: format!("the status of {named} could not be read"),
            advice: advice_to_move(merge_oid, where_to.as_str()),
        });
    }
    let dirty = status_paths(&status.stdout);
    let kept: bool = !dirty.is_empty();
    if kept {
        let incoming = run_query_awaiting(
            there,
            &argv(&["diff", "--name-only", "-z", "--no-renames", local, merge_oid]),
        )
        .await?;
        if incoming.code != Some(0) {
            return Ok(Advanced::Not {
                reason: format!("the files the merge changes in {named} could not be read"),
                advice: advice_to_move(merge_oid, where_to.as_str()),
            });
        }
        let overlap = overlapping_paths(&dirty, &diff_paths(&incoming.stdout));
        if !overlap.is_empty() {
            return Ok(Advanced::Not {
                reason: format!(
                    "{named} has uncommitted changes to files the merge changes: {}",
                    listed(&overlap)
                ),
                advice: advice_to_move(merge_oid, where_to.as_str()),
            });
        }
    }
    if let Err(reason) = fast_forward_recorded(there, branch_ref, local, merge_oid, &SHIPPED_MOVE).await? {
        return Ok(Advanced::Not {
            reason,
            advice: advice_to_move(merge_oid, where_to.as_str()),
        });
    }
    if kept {
        // That worktree's uncommitted work must read exactly as it did; the move is done, so a difference is reported.
        let after = run_query_awaiting(there, &status_arguments).await?;
        if status_differs(&status.stdout, &after.stdout) {
            warn(
                TAG,
                &format!("the status of {named} differs after the fast-forward; compare it with the status before the merge"),
            );
        }
    }
    return Ok(Advanced::Elsewhere {
        worktree: place.to_path_buf(),
        kept,
    });
}

/// What:
///  Bring the branch the push sent to the published merge,
/// in whichever worktree has it checked out,
/// and never another branch.
/// Why:
///   The branch left behind its own remote makes the next commit rejected again,
/// and the person would have to
///  merge by hand;
/// the merge contains the pushed tip,
/// so the move is a pure fast-forward.
///   Only the branch the push sent is moved:
/// the command's own checkout,
/// the branch it had checked out
///  and every other branch are left exactly where they were,
/// which is what the wrong name of a typed push broke.
///   A branch checked out in another worktree moves there under that worktree's own rules:
/// it follows only while it still
///  has the branch at the pushed tip and none of its uncommitted work lies in a file the move writes.
///   A branch no live worktree holds moves as a ref alone,
/// and that move looks at the listing again on either side of its write,
/// so a worktree that takes the branch inside that window is not left with the older files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function advancePushedBranch(git, branchRef, local, mergeOid): Promise<Advanced>;
/// ```
pub(crate) async fn advance_pushed_branch_awaiting(
    git: &PushGit,
    branch_ref: &str,
    local: &str,
    merge_oid: &str,
) -> Result<Advanced, SingleFlightError> {
    let Some(place) = live_holder_awaiting(git, branch_ref).await? else {
        return match move_the_ref_only_awaiting(git, branch_ref, local, merge_oid).await? {
            NowhereMove::Answered(advanced) => Ok(advanced),
            // A worktree took the branch between the two looks, so its files are written there and by its own move.
            NowhereMove::Held(taken) => {
                let there: PushGit = git_at(git, taken.as_path());
                advance_in_another_worktree(&there, taken.as_path(), branch_ref, local, merge_oid).await
            }
        };
    };
    if is_this_worktree(git, place.as_path()).await? {
        return match advance_local_branch_awaiting(git, branch_ref, local, merge_oid).await? {
            Ok(kept) => Ok(Advanced::Here { kept }),
            Err(reason) => Ok(Advanced::Not {
                reason,
                advice: advice_to_move(merge_oid, "in this worktree"),
            }),
        };
    }
    let there: PushGit = git_at(git, place.as_path());
    return advance_in_another_worktree(&there, place.as_path(), branch_ref, local, merge_oid).await;
}
