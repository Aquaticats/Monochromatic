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
    Advanced, BRANCH_MOVED, Checkout, Moved, SHIPPED_MOVE, advance_local_branch_awaiting, advice_to_move,
    fast_forward_recorded, status_differs,
};
/// Which worktree holds the branch, whether it is this one, and how to run Git in it.
use crate::auto_push_reconcile_branch_worktree::{git_at, is_this_worktree, live_holders_awaiting};
/// The move of a branch no live worktree holds.
use crate::auto_push_reconcile_branch_advance_without_a_checkout::{NowhereMove, move_the_ref_only_awaiting};
use crate::auto_push_reconcile_basics_and_limits::{TAG, argv};
use crate::diagnostic_log::{debug, warn};
use std::path::{Path, PathBuf};

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
    let checkout: Checkout = match fast_forward_recorded(there, branch_ref, local, merge_oid, &SHIPPED_MOVE).await? {
        Ok(checkout) => checkout,
        Err(reason) => {
            return Ok(Advanced::Not {
                reason,
                advice: advice_to_move(merge_oid, where_to.as_str()),
            });
        }
    };
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
    return Ok(answered_about(place, checkout, kept));
}

/// What:
///  The report's answer for a move made in another worktree,
/// from what became of that worktree's checkout.
/// Why:
///   A switch of that session's can land between the write of its files and the write of the branch,
/// and then the worktree shows the branch it chose rather than the merge:
///  the report says which, and whether anything of the merge is still there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = answeredAbout(place, checkout, kept);
/// ```
fn answered_about(place: &Path, checkout: Checkout, kept: bool) -> Advanced {
    return match checkout {
        Checkout::Followed => Advanced::Elsewhere { worktree: place.to_path_buf(), kept },
        Checkout::WentTo { shows } => Advanced::WorktreeWentTo { worktree: place.to_path_buf(), shows },
        Checkout::Holding { shows, paths, reason } => Advanced::WorktreeHolding {
            worktree: place.to_path_buf(),
            shows,
            paths,
            reason,
        },
    };
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
    let holders: Vec<PathBuf> = live_holders_awaiting(git, branch_ref).await?;
    if holders.is_empty() {
        return match move_the_ref_only_awaiting(git, branch_ref, local, merge_oid).await? {
            NowhereMove::Answered(advanced) => Ok(advanced),
            // Worktrees took the branch between the two looks, so their files are written by their own moves.
            NowhereMove::Held(taken) => {
                let mut answers: Vec<(PathBuf, Advanced)> = Vec::new();
                for place in taken {
                    let there: PushGit = git_at(git, place.as_path());
                    let answer: Advanced = advance_in_another_worktree(&there, place.as_path(), branch_ref, local, merge_oid).await?;
                    answers.push((place, answer));
                }
                return Ok(the_one_answer(answers));
            }
        };
    }
    // What: every worktree that shows the branch is advanced, not only the first one Git lists.
    // Why:  Git allows the same branch in more than one worktree with `--force`, and advancing one of them left the
    // others at the old tip with no merge intent and no branch-move record, so a commit there could record the old
    // tree over the published merge
    //  (a review of commit `986a82bbab`, its finding 2).
    let mut answers: Vec<(PathBuf, Advanced)> = Vec::new();
    for place in holders {
        if is_this_worktree(git, place.as_path()).await? {
            let moved: Advanced = match advance_local_branch_awaiting(git, branch_ref, local, merge_oid).await? {
                Ok(Moved { checkout: Checkout::Followed, kept }) => Advanced::Here { kept },
                Ok(Moved { checkout, kept }) => answered_about(place.as_path(), checkout, kept),
                Err(reason) => Advanced::Not {
                    reason,
                    advice: advice_to_move(merge_oid, "in this worktree"),
                },
            };
            answers.push((place, moved));
            continue;
        }
        let there: PushGit = git_at(git, place.as_path());
        let answer: Advanced = advance_in_another_worktree(&there, place.as_path(), branch_ref, local, merge_oid).await?;
        // What: a worktree that shows the branch but could not follow it leaves a record, so its next command
        // finishes the checkout.
        // Why:  its checkout may be behind the branch by commits the push never sent, and a commit there would
        // record the older tree; the owner's decision is that no checkout showing the pushed branch stays at an
        // older tip without a record saying so
        //  (a review of commit `986a82bbab`, its finding 2).
        if matches!(answer, Advanced::Not { .. } | Advanced::WorktreeHolding { .. }) {
            let _left: bool = leave_a_record_for(&there, branch_ref, local, merge_oid, place.as_path()).await?;
        }
        answers.push((place, answer));
    }
    return Ok(the_one_answer(answers));
}

/// What:
///  Leave a branch-move record for a worktree whose checkout could not follow the branch,
/// naming the tip that worktree's HEAD has as the move's old tip.
/// Why:
///   The record is what a later command in that worktree reads, and its old tip is that worktree's own HEAD:
///  the generic sweep then brings its checkout to the branch and ends the record on its proof.
///  A worktree that already holds the merge needs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const head = await there(['rev-parse','--verify','HEAD^{commit}']); if (contains(merge, head)) return false; return recordBranchMove(commonDir, branchRef, head, merge) !== undefined;
/// ```
async fn leave_a_record_for(there: &PushGit, branch_ref: &str, local: &str, merge_oid: &str, place: &Path) -> Result<bool, SingleFlightError> {
    // What: a record only when this worktree's checkout still holds what the move wrote.
    // Why:  a worktree that already holds the merge's tree on those paths needs nothing, and a record for it would
    // be finished by the next command there with nothing to write
    //  (a review of commit `986a82bbab`, its finding 2).
    let holds: bool = match crate::auto_push_branch_move_repairs::holds_what_a_move_wrote(there, local, merge_oid).await {
        Ok(holds) => holds,
        // A worktree whose questions fail leaves nothing: the report names it, and a record keyed on a failed read
        // would be answered wrongly later.
        Err(_) => return Ok(false),
    };
    if !holds {
        return Ok(false);
    }
    let common_dir: PathBuf = crate::auto_push_branch_move_record::common_git_dir_awaiting(there).await?;
    // What: the record names the tip the push sent as its old tip, which is the tip this worktree's checkout holds.
    // Why:  the generic sweep then brings that checkout to the merge and ends the record on its proof.
    let left: bool = crate::auto_push_branch_move_record::record_branch_move(common_dir.as_path(), branch_ref, local, merge_oid).is_some();
    if left {
        debug(TAG, &format!("the worktree at {} did not follow the branch, so a record for its next command was left", place.display()));
    }
    return Ok(left);
}

/// What:
///  The one answer the report words, out of every worktree's answer.
/// Why:
///   The report prints one line and names one place, so a failure of any worktree is the answer
/// (it is what a person must act on), and otherwise the worktree this command is in, or the first followed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failed = answers.filter(([, answer]) => answer.kind === 'not' || answer.kind === 'worktree-holding');
/// return failed.length ? { kind: 'not', reason: failed.map(describe).join('; '), advice } : (answers.find(([, a]) => a.kind === 'here') ?? answers[0])[1];
/// ```
fn the_one_answer(answers: Vec<(PathBuf, Advanced)>) -> Advanced {
    let mut failed: Vec<String> = Vec::new();
    let mut advices: Vec<String> = Vec::new();
    for (place, answer) in answers.iter() {
        match answer {
            Advanced::Not { reason, advice } => {
                failed.push(format!("the worktree at {}: {reason}", place.display()));
                advices.push(advice.clone());
            }
            Advanced::WorktreeHolding { paths, reason, .. } => {
                failed.push(format!("the worktree at {} still differs on {}: {reason}", place.display(), listed(paths)))
            }
            _ => {}
        }
    }
    if failed.len() == 1 {
        // One worktree failed, and its own words are what the report has always printed for it.
        if let Some((_, Advanced::Not { reason, advice })) = answers.iter().find(|(_, answer)| return matches!(answer, Advanced::Not { .. })) {
            return Advanced::Not {
                reason: reason.clone(),
                advice: advice.clone(),
            };
        }
        return Advanced::Not {
            reason: failed.join("; "),
            advice: String::from("in the worktree named here, commit what is left or check it out again, then push again"),
        };
    }
    if !failed.is_empty() {
        return Advanced::Not {
            reason: failed.join("; "),
            advice: if advices.is_empty() {
                String::from("in each worktree named here, commit what is left or check it out again, then push again")
            } else {
                advices.join("; ")
            },
        };
    }
    let mut followed: Option<Advanced> = None;
    for (_, answer) in answers {
        if matches!(answer, Advanced::Here { .. }) {
            return answer;
        }
        if followed.is_none() {
            followed = Some(answer);
        }
    }
    return followed.unwrap_or(Advanced::Nowhere);
}
