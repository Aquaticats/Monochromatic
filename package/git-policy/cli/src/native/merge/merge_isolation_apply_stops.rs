//! What:
//!  What a step of an isolated merge that stopped means for the command's ending,
//!  and what a clear that met newer bytes does about them.
//! Why:
//!  A step that failed on a branch that moved is made again on the moved head,
//! one that failed on the index lock is made again
//!  once the lock is gone,
//! and one that failed after the merge began is settled from the record;
//!  a clear that met a file
//!  newer than its capture puts the person's own layers back and makes the merge again on what the file holds,
//!  and a merge whose record stayed because the lock that stopped Git's update stopped the putting back of the
//!  person's own bytes is saved on a side ref and exits 0
//!  (`SPEC.md`,
//! "Merges and commits never fail";
//! the owner's "never discard user data",
//! "Commits should never fail.",
//! and "Use as few locks as possible.").
//!  The writes these endings judge are `merge_isolation_apply.rs`,
//! which is at its code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = theEndOfAFailedStep({ session, plan, intent, record, reference, detail, lockTaken, code, gitStderr });
//! ```

use crate::diagnostics::{EngineFailureCode, render_engine_failure_with_hint};
use crate::merge_isolation_apply_findings::{
    BytesAfterAFailedMerge, WhyTheMergeIsSaved, a_merge_in_the_branch_whose_carry_waits, a_merge_partly_installed_behind_the_lock,
    a_merge_the_branch_moved_under, failure_text,
};
use crate::merge_isolation_apply_save::save_for_a_later_finish;
use crate::merge_isolation_report::not_applied_finding;
use crate::merge_isolation_carry_apply::CarryFailure;
use crate::merge_isolation_git_reads::Session;
use crate::merge_isolation_journal::MergeIntent;
use crate::merge_isolation_recovery::{MergeRecoveryAction, RecoveryPlace, settle_failed_merge};
use crate::merge_isolation_recovery_steps::IndexHeldAt;
use crate::merge_isolation_run_types::{Bail, IsolatedMerge, Plan, refused};
use std::path::{Path, PathBuf};

/// What:
///  Put the person's own work back and make the merge again on the bytes their files hold now.
/// Why:
///   The owner's rule is that a save is never discarded and a merge is never refused for one;
/// the clear stops at the first path
///  it did not judge,
/// so the record goes once the layers are back,
/// and the round starts again on what the person has now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aClearThatMetNewerBytes({ place, record, intent, reference, failure }): Bail; // settle, then Bail.Again
/// ```
pub(crate) fn a_clear_that_met_newer_bytes(place: &RecoveryPlace, record: &Path, intent: &MergeIntent, reference: &str, failure: &CarryFailure) -> Bail {
    let detail: String = failure_text(failure);
    return match settle_failed_merge(place, record, intent, &mut std::io::stderr()) {
        // Nothing of the merge was applied, so the record and the capture are gone and the round is made again.
        Ok(MergeRecoveryAction::Discarded | MergeRecoveryAction::BranchMoved | MergeRecoveryAction::SavedBranchMoved) => Bail::Again(detail),
        // The lock stays held or the record waits, so the next command finishes this very merge and nothing of the person's went.
        Ok(MergeRecoveryAction::IndexHeld(_) | MergeRecoveryAction::DeferredRefused | MergeRecoveryAction::OwnerActive) => refused(
            2,
            format!(
                "monochromatic-git-policy-cli: git merge {reference}: {detail}; the merge was recorded and nothing of yours was \
                 removed. Run any git command again to finish it.\n"
            ),
        ),
        // The merge is in the branch, which another command may have made it; the person's work is back and nothing is left.
        Ok(MergeRecoveryAction::AlreadyLanded | MergeRecoveryAction::Completed) => Bail::Done(IsolatedMerge::Finished {
            code: 0,
            stdout: String::new(),
            stderr: format!("monochromatic-git-policy-cli: git merge {reference}: the merge is in the branch.\n"),
        }),
        Err(error) => refused(2, render_engine_failure_with_hint(0, EngineFailureCode::ContentUnavailable, error.error.0.as_str(), None)),
    };
}

/// What:
///  What a failed step's detail means for the command's ending.
/// Why:
///   A step that failed on a branch that moved is made again on the moved head,
/// one that failed on the index lock is
///  made again once the lock is gone,
/// and one that failed after the merge began is settled from the record.
///  A merge whose record stayed because the lock that stopped Git's update stopped the putting back of the person's
///  own bytes too is saved on a side ref and exits 0,
/// because nothing the machine or another program does ends a
///  merge with a nonzero status (`SPEC.md`,
/// "Merges and commits never fail").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StepEnding = { kind: 'stop'; bail: Bail } | { kind: 'saved-for-a-later-finish'; why: WhyTheMergeIsSaved; at: IndexHeldAt };
/// ```
pub(crate) enum StepEnding {
    /// The command stops with this result,
    /// which is a refusal,
    /// a forward to native Git or a finished merge of its own.
    Stop(Bail),
    /// Nothing more of the merge could be written now and the record stayed,
    /// in the state the settle read.
    SavedForALaterFinish {
        /// The lock that stopped the settle,
        /// as the line that names it words it.
        why: WhyTheMergeIsSaved,
        /// What the checkout and the branch held when it stopped.
        at: IndexHeldAt,
    },
}

/// What:
///  The command's ending for one failed step of a merge,
/// with the save it takes instead of a refusal when a lock kept the person's own bytes out of the checkout.
/// Why:
///   The save writes a side ref and a record,
/// which is the writing half's own work (`merge_isolation_apply_save.rs`),
/// and the
///  decision is the settle's,
/// so the two meet here once rather than at every step that can fail.
///  The bound of the wait that ends here is `MERGE_LOCK_CEILING`,
/// one minute,
/// whose measurement and reason are
///  `wait_ceilings.rs`;
/// the round that a cleared lock allows is bounded by `MAX_ROUNDS`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = theEndOfAFailedStep({ session, plan, intent, record, reference, detail, lockTaken, code, gitStderr });
/// ```
pub(crate) fn the_end_of_a_failed_step(
    session: &Session,
    plan: &Plan<'_>,
    intent: &MergeIntent,
    record: &Path,
    reference: &str,
    detail: &str,
    lock_taken: bool,
    the_paths_were_cleared: bool,
    code: i32,
    git_stderr: &[u8],
) -> Result<IsolatedMerge, Bail> {
    let place: RecoveryPlace = RecoveryPlace::settling(&session.context, plan.git_dir, plan.root);
    let ending: StepEnding = settle_a_failed_step(&place, record, intent, reference, detail, lock_taken, code, git_stderr);
    return match ending {
        StepEnding::Stop(bail) => Err(bail),
        StepEnding::SavedForALaterFinish { why, at } => the_end_in_the_state_read(session, plan, intent, record, reference, why, at, the_paths_were_cleared),
    };
}

/// What:
///  The ending a merge takes in the state the settle read when a lock kept it from finishing:
///  saved when the branch has not moved and nothing of the merge is in the checkout,
///  and otherwise a line that states what stands and what the next command does with it.
/// Why:
///   `IndexHeld` says the index could not be written,
/// not which of several states the settle stopped in:
/// a branch that moved under the merge cannot take it as it was saved,
/// a merge already in the branch has only its carry left,
/// and a partly installed merge is already in the checkout.
///  One line for all of them was false in two of the three,
/// so each state ends in its own words,
/// read from the settle instead of assumed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = (at) => at.branchMoved ? moved : at.landed ? landed : at.entries ? partly : save(...);
/// ```
fn the_end_in_the_state_read(
    session: &Session,
    plan: &Plan<'_>,
    intent: &MergeIntent,
    record: &Path,
    reference: &str,
    why: WhyTheMergeIsSaved,
    at: IndexHeldAt,
    the_paths_were_cleared: bool,
) -> Result<IsolatedMerge, Bail> {
    let lock: PathBuf = plan.git_dir.join("index.lock");
    let mut kept: Option<&str> = None;
    if let Some(capture) = plan.carried.as_ref() {
        kept = capture.reference.as_deref();
    }
    let bytes: BytesAfterAFailedMerge<'_> = BytesAfterAFailedMerge { the_paths_were_cleared, kept_on: kept };
    // A landed merge is also a branch built on, so it is answered first: only its carry is left.
    if at.landed {
        return Ok(IsolatedMerge::Finished {
            code: 0,
            stdout: String::new(),
            stderr: a_merge_in_the_branch_whose_carry_waits(reference, lock.as_path(), &bytes),
        });
    }
    if at.branch_moved {
        return Ok(IsolatedMerge::Finished { code: 0, stdout: String::new(), stderr: a_merge_the_branch_moved_under(reference, lock.as_path(), &bytes) });
    }
    if at.the_merges_entries_in_the_checkout {
        return Ok(IsolatedMerge::Finished {
            code: 0,
            stdout: String::new(),
            stderr: a_merge_partly_installed_behind_the_lock(reference, lock.as_path(), &bytes),
        });
    }
    return save_for_a_later_finish(session, plan, intent, record, reference, why, the_paths_were_cleared);
}

/// What:
///  What a failed step's detail means for the command's ending.
/// Why:
///   A step that failed on a branch that moved is made again on the moved head,
/// one that failed on the index lock is
///  made again once the lock is gone,
/// and one that failed after the merge began is settled from the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleAFailedStep({ place, record, intent, reference, detail, lockTaken, code, gitStderr }): StepEnding;
/// ```
pub(crate) fn settle_a_failed_step(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
    reference: &str,
    detail: &str,
    lock_taken: bool,
    code: i32,
    git_stderr: &[u8],
) -> StepEnding {
    return match settle_failed_merge(place, record, intent, &mut std::io::stderr()) {
        Ok(MergeRecoveryAction::BranchMoved) => StepEnding::Stop(Bail::Again(format!("{reference} was merged on a branch that moved meanwhile"))),
        Ok(MergeRecoveryAction::Discarded) if lock_taken => StepEnding::Stop(Bail::Again(String::from("another program took the index lock first"))),
        Ok(MergeRecoveryAction::Completed) => StepEnding::Stop(Bail::Done(IsolatedMerge::Finished {
            code: 0,
            stdout: String::new(),
            stderr: format!(
                "monochromatic-git-policy-cli: git merge {reference}: Git reported an error ({detail}) after it had started, and the \
                 wrapper completed the merge.\n"
            ),
        })),
        // The record stayed, and the settle's putting back was stopped by the same lock that stopped Git's update, so the
        //  merge is saved on a side ref rather than refused. Which of the two instants it was decides what the line may say
        //  about the checkout: a lock taken after the clear left the branch's content where the person's bytes were.
        Ok(MergeRecoveryAction::IndexHeld(at)) => {
            StepEnding::SavedForALaterFinish { why: WhyTheMergeIsSaved::TheLockStoppedTheSettle, at }
        }
        Ok(_) => StepEnding::Stop(refused(code, format!("{}{}", String::from_utf8_lossy(git_stderr), not_applied_finding(reference, detail)))),
        Err(error) => StepEnding::Stop(refused(2, render_engine_failure_with_hint(0, EngineFailureCode::ContentUnavailable, error.error.0.as_str(), None))),
    };
}
