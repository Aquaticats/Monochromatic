//! What:
//!  What a drained push's verdict does to its records,
//! to the retry plan,
//! and to the person.
//! Why:
//! A drain of work claims items,
//! runs each kind with its awaited handler,
//! and settles what answered:
//! a push carries records of its own (`settle_after_the_push`),
//! and every other kind answers in words (`service_work_settling_with_words.rs`).
//! The push's settlement is the drain's own step and holds no state of its own,
//! so it has a file beside the drain and the loop stays readable;
//! the drain hands it the claim set,
//! which travels into the storage job and back (`service_work_storage_awaited.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleAfterThePush({ held, workDir, gitDir, members, verdict, pushingSince, replanning });
//! ```

use super::service_work_handlers_replanning_and_acceptance::*;
use crate::service_claim_renewal::HeldClaims;
use crate::service_work_storage_awaited::{
    clear_push_failures_before_awaiting, record_push_failure_awaiting, record_push_failure_within_awaiting, record_push_skip_awaiting, settle_all_awaiting,
};
use crate::push_failure_notice::PushOutcome;
use crate::wait_ceilings::AFTER_COMMIT_STATE_CEILING;
use crate::work_item::{Claimed, DONE, FAILED, WorkItem};
use std::path::Path;
use std::time::Duration;

/// What:
///  Write the person's record of a push that did not land,
/// and drop the branch's try only once that record is there.
/// Why:
///   Nothing scans the records of `failed`,
/// so the record is the person's only word about a commit that was not backed up;
/// a try dropped before it exists leaves that commit unpushed with neither a retry nor a word,
/// waiting for a command the person may never type
/// (finding 3 of the review of `c02f49638e`).
/// The record's own step is a job of the pool with a ceiling
/// (`service_work_storage_awaited.rs`),
/// so it can give no answer while every place of the process's bound is taken or the write outlasts its ceiling:
/// the try is then kept (`Replanning::keep_the_try`),
/// and the next round tries the push and writes the record again until the person has it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await theRecordThenTheForget(gitDir, item, attempt, outcome, said, replanning)
/// ```
pub(super) async fn the_record_then_the_forget(git_dir: &Path, item: &WorkItem, attempt: u32, outcome: PushOutcome, said: &str, replanning: &mut Replanning) {
    return the_record_then_the_forget_within(AFTER_COMMIT_STATE_CEILING, git_dir, item, attempt, outcome, said, replanning).await;
}

/// `the_record_then_the_forget` with the ceiling of the record's step given,
/// so a control reaches that step's bound in a moment (`service_work_drains_awaiting_tests.rs`).
pub(super) async fn the_record_then_the_forget_within(
    ceiling: Duration,
    git_dir: &Path,
    item: &WorkItem,
    attempt: u32,
    outcome: PushOutcome,
    said: &str,
    replanning: &mut Replanning,
) {
    let branch: &str = item.branch_ref.as_str();
    match record_push_failure_within_awaiting(ceiling, git_dir, branch, item.oid.as_str(), outcome, said).await {
        Ok(()) => replanning.forget(git_dir, branch),
        Err(reason) => {
            crate::service_error!(
                "the failed push of {} {} could not be recorded for the person, so its try is kept: {reason}",
                item.oid,
                crate::service_timing::in_the_repository_at(item.repository.as_path())
            );
            replanning.keep_the_try(git_dir, item, attempt);
        }
    }
    return;
}

/// What:
///  What one push's verdict does to its records and to the retry plan.
/// Why:
///   The four verdicts differ only in what they leave for the person and in what the plan keeps,
/// so they are one function
///  and the drain's loop stays readable.
/// A record goes before its items settle:
/// a service that dies between the
///  two pushes again,
/// and nothing is lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleAfterThePush(held, workDir, gitDir, members, verdict, pushingSince, replanning)
/// ```
pub(super) async fn settle_after_the_push(
    held: HeldClaims,
    work_dir: &Path,
    git_dir: &Path,
    members: &[Claimed],
    verdict: Verdict,
    pushing_since: std::time::SystemTime,
    replanning: &mut Replanning,
) -> Option<HeldClaims> {
    let newest: Option<&Claimed> = members.last();
    let settling: Vec<Claimed> = members.to_vec();
    match verdict {
        Verdict::Done => {
            // A push of the branch went through, so an earlier failure to push it is no longer true.
            if let Some(latest) = newest {
                clear_push_failures_before_awaiting(git_dir, latest.item.branch_ref.as_str(), pushing_since).await;
                replanning.forget(git_dir, latest.item.branch_ref.as_str());
            }
            return settle_all_awaiting(held, work_dir, settling, DONE).await;
        }
        Verdict::Retry(text) => {
            // Without handlers that retry, a retryable push is a failed one. A push that can pass is not known to
            //  have reached the remote, so its record says that instead of naming what was refused, and it says
            //  whether a plan will try it again.
            let (said, outcome): (String, PushOutcome) = match (newest, replanning.first) {
                (Some(latest), Some(first)) => {
                    let planned: PlannedRetry = planned_retry(git_dir, replanning, first, &latest.item, 1, text.as_str());
                    // The framing promises a retry only when the plan the words speak of exists.
                    let outcome: PushOutcome = if planned.kept { PushOutcome::Unfinished } else { PushOutcome::UnfinishedWithNoRetry };
                    (planned.words, outcome)
                }
                _ => (text, PushOutcome::UnfinishedWithNoRetry),
            };
            if let Some(latest) = newest
                && let Err(reason) =
                    record_push_failure_awaiting(git_dir, latest.item.branch_ref.as_str(), latest.item.oid.as_str(), outcome, said.as_str()).await
            {
                crate::service_error!(
                    "the failed push of {} {} could not be recorded for the person: {reason}",
                    latest.item.oid,
                    crate::service_timing::in_the_repository_at(latest.item.repository.as_path())
                );
            }
            return settle_all_awaiting(held, work_dir, settling, FAILED).await;
        }
        Verdict::Failed(text) => {
            // The record goes before the items settle, so an item in `failed` always has its record, and the branch's try
            //  goes only once that record exists.
            if let Some(latest) = newest {
                // A claimed item's first planned try is number 1, as the retry of its own failure is.
                the_record_then_the_forget(git_dir, &latest.item, 1, PushOutcome::Refused, text.as_str(), replanning).await;
            }
            return settle_all_awaiting(held, work_dir, settling, FAILED).await;
        }
        Verdict::Skipped(text) => {
            // Nothing was pushed, so an earlier failure of the branch is still true and stays. A skip with words leaves
            //  its own record before the items settle; one that could not be recorded settles as failed, so no item
            //  is done without the person having been told.
            let recorded: bool = match newest {
                Some(latest) if !text.is_empty() => {
                    match record_push_skip_awaiting(git_dir, latest.item.branch_ref.as_str(), latest.item.oid.as_str(), text.as_str()).await {
                        Ok(()) => true,
                        Err(reason) => {
                            crate::service_error!(
                                "the skipped push of {} {} could not be recorded for the person: {reason}",
                                latest.item.oid,
                                crate::service_timing::in_the_repository_at(latest.item.repository.as_path())
                            );
                            false
                        }
                    }
                }
                _ => true,
            };
            return settle_all_awaiting(held, work_dir, settling, if recorded { DONE } else { FAILED }).await;
        }
    }
}
