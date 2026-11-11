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
        Ok(MergeRecoveryAction::IndexHeld | MergeRecoveryAction::DeferredRefused | MergeRecoveryAction::OwnerActive) => refused(
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
