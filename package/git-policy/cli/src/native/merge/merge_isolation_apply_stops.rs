//! What:
//!  What a step of an isolated merge that stopped means for the command's ending,
//!  and what a clear that met newer bytes does about them.
//! Why:
//!  A step that failed on a branch that moved is made again on the moved head,
//! one that failed on the index lock is made again
//!  once the lock is gone,
//! and one that failed after the merge began is settled from the record;
//!  a clear that met a file
//!  newer than its capture puts the person's own layers back and makes the merge again on what the file holds
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
//! const ending = settleAFailedStep({ place, record, intent, reference, detail, lockTaken, code, gitStderr });
//! ```

use crate::diagnostics::{EngineFailureCode, render_engine_failure_with_hint};
use crate::merge_isolation_apply_findings::failure_text;
use crate::merge_isolation_report::not_applied_finding;
use crate::merge_isolation_carry_apply::CarryFailure;
use crate::merge_isolation_journal::MergeIntent;
use crate::merge_isolation_recovery::{MergeRecoveryAction, RecoveryPlace, settle_failed_merge};
use crate::merge_isolation_git_reads::Session;
use crate::merge_isolation_run_types::{Bail, IsolatedMerge, refused};
use std::path::Path;

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
        Ok(
            MergeRecoveryAction::IndexHeld
            | MergeRecoveryAction::DeferredRefused
            | MergeRecoveryAction::OwnerActive
            | MergeRecoveryAction::PartlyInstalled,
        ) => refused(
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleAFailedStep({ place, record, intent, reference, detail, lockTaken, code, gitStderr }): Bail;
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
) -> Bail {
    return match settle_failed_merge(place, record, intent, &mut std::io::stderr()) {
        Ok(MergeRecoveryAction::BranchMoved) => Bail::Again(format!("{reference} was merged on a branch that moved meanwhile")),
        Ok(MergeRecoveryAction::Discarded) if lock_taken => Bail::Again(String::from("another program took the index lock first")),
        Ok(MergeRecoveryAction::Completed) => Bail::Done(IsolatedMerge::Finished {
            code: 0,
            stdout: String::new(),
            stderr: format!(
                "monochromatic-git-policy-cli: git merge {reference}: Git reported an error ({detail}) after it had started, and the \
                 wrapper completed the merge.\n"
            ),
        }),
        Ok(_) => refused(code, format!("{}{}", String::from_utf8_lossy(git_stderr), not_applied_finding(reference, detail))),
        Err(error) => refused(2, render_engine_failure_with_hint(0, EngineFailureCode::ContentUnavailable, error.error.0.as_str(), None)),
    };
}

/// What:
///  Put back a merge whose checkout step the wrapper ended,
/// and refuse it with the reason and the state.
/// Why:
///  The step that writes the merge into the checkout may have to download the content it writes,
/// and a server that
///  accepts the request and never answers held it and its `git-lfs` child for good,
/// with `index.lock` held and the worktree half
///  written (measured at 607 seconds,
/// `F13` of the alpha.12 audit).
/// The step is ended at the bound of a bulk step,
/// and what it began must not stay:
/// every path the merge changes is put back to what the branch holds now,
/// the person's own work stays where the merge never
///  wrote it and comes back from the record's copy where it did,
/// the index lock the ended Git left is cleared on the proof recovery acts on,
/// and the record goes.
///  A step that was ended after Git moved the branch is not put back:
/// the merge landed,
/// and its record is settled as any such merge is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function putBackAMergeWhoseCheckoutDidNotFinish({ place, record, intent, reference, newOid, mover }): Bail; // settle or refuse
/// ```
pub(crate) fn put_back_a_merge_whose_checkout_did_not_finish(
    place: &RecoveryPlace,
    record: &Path,
    intent: &MergeIntent,
    reference: &str,
    new_oid: &str,
    mover: &Session,
) -> Bail {
    let target: &str = intent.target_ref.as_deref().unwrap_or("HEAD");
    // The branch is read before anything is put back: a step ended in the instant after Git moved it is a merge that
    //  landed, and putting its paths back would leave a branch naming the merge with a checkout that does not.
    let head: String = match mover.run(&["rev-parse", "--verify", "--quiet", target]) {
        Ok(found) if found.succeeded() => String::from_utf8_lossy(&found.stdout).trim().to_owned(),
        _ => String::new(),
    };
    if head == new_oid {
        return settle_a_failed_step(place, record, intent, reference, "the checkout step was ended after the branch moved", false, 2, b"");
    }
    if head.is_empty() {
        // The branch cannot be read, so nothing of the merge can be put back against it; the record stays and a command
        //  that writes to the checkout is stopped until the branch can be read (`startup_gate.rs`).
        return refused(
            2,
            render_engine_failure_with_hint(
                0,
                EngineFailureCode::ContentUnavailable,
                format!(
                    "monochromatic-git-policy-cli: git merge {reference}: the step that writes the merge into the checkout made no \
                     progress and was ended, and {target} could not be read to put the paths it changed back. Nothing of the merge moved \
                     the branch. Check the branch with `git log -1`, then run any git command again."
                )
                .as_str(),
                None,
            ),
        );
    }
    let settled = crate::lazy_runtime::run_on_runtime(crate::merge_isolation_moved::settle_moved_branch_awaiting(
        place,
        record,
        intent,
        head.as_str(),
        false,
        &mut std::io::stderr(),
    ));
    return match settled {
        // The checkout is the branch's again and the record is gone: the merge was not made, and nothing is left of it.
        Ok(_) => refused(
            2,
            format!(
                "monochromatic-git-policy-cli: git merge {reference}: the step that writes the merge into the checkout showed no sign \
                 of work for its whole window and was ended (or ran to its bound), and Git had not moved the branch, so the merge was \
                 not made. Every path it had begun to write is back at {head} and your own work in them is where it was, the record and \
                 the copy it kept of that work are gone, and no lock or program of the step is left. A `git-lfs` download from a server \
                 that accepts the request and never answers is the usual cause: check that the LFS server answers, then run the merge \
                 again.\n"
            ),
        ),
        Err(refusal) => refused(2, render_engine_failure_with_hint(0, EngineFailureCode::ContentUnavailable, format!("{refusal}").as_str(), None)),
    };
}
