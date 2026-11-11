//! What:
//!  What a work item's verdict does to its records,
//! to the retry plan,
//! and to the person.
//! Why:
//! A drain of work claims items,
//! runs each kind with its awaited handler,
//! and then has two ways to settle what answered:
//! a push carries records of its own (`settle_after_the_push`),
//! and everything else answers in words (`settle_with_the_words`).
//! Both are the drain's own steps and hold no state of their own,
//! so they have a file beside it and the drain's loop stays readable;
//! the drain hands each one the claim set,
//! which travels into the storage job and back (`service_work_storage_awaited.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleAfterThePush({ held, workDir, gitDir, members, verdict, pushingSince, replanning });
//! await settleWithTheWords({ held, workDir, item, verdict, kind });
//! ```

use super::service_work_handlers_replanning_and_acceptance::*;
use crate::service_claim_renewal::HeldClaims;
use crate::service_work_storage_awaited::{
    clear_push_failures_before_awaiting, record_push_failure_awaiting, record_push_skip_awaiting, settle_all_awaiting,
};
use crate::push_failure_notice::PushOutcome;
use crate::work_item::{Claimed, DONE, FAILED};
use std::path::Path;

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
                crate::service_error!("the failed push of {} could not be recorded for the person: {reason}", latest.item.oid);
            }
            return settle_all_awaiting(held, work_dir, settling, FAILED).await;
        }
        Verdict::Failed(text) => {
            // A failure the service gives up on is no longer waiting for a new try.
            if let Some(latest) = newest {
                replanning.forget(git_dir, latest.item.branch_ref.as_str());
            }
            // The record goes before the items settle, so an item in `failed` always has its record.
            if let Some(latest) = newest
                && let Err(reason) = record_push_failure_awaiting(
                    git_dir,
                    latest.item.branch_ref.as_str(),
                    latest.item.oid.as_str(),
                    PushOutcome::Refused,
                    text.as_str(),
                )
                .await
            {
                crate::service_error!("the failed push of {} could not be recorded for the person: {reason}", latest.item.oid);
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
                            crate::service_error!("the skipped push of {} could not be recorded for the person: {reason}", latest.item.oid);
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

/// What:
///  Settle one item whose kind answers with words rather than a push:
/// a skipped one is done and says why,
/// a failed one is failed and says why.
/// Why:
///   A trash and a review answer the same four verdicts and leave the person the same two kinds of line,
/// so the drain
///  keeps one account of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleWithTheWords(held, workDir, item, verdict, kind)
/// ```
pub(super) async fn settle_with_the_words(held: HeldClaims, work_dir: &Path, item: Claimed, verdict: Verdict, kind: &str) -> Option<HeldClaims> {
    match verdict {
        Verdict::Done => {
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await;
        }
        Verdict::Failed(reason) | Verdict::Retry(reason) => {
            match kind {
                "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} could not be emptied: {reason}", item.item.parent),
                "prewarm" => crate::service_warn!("the Cargo build directory of {} could not be filled: {reason}", item.item.repository.display()),
                _ => crate::service_warn!("the review of {} failed: {reason}", item.item.oid),
            }
            return settle_all_awaiting(held, work_dir, vec![item], FAILED).await;
        }
        Verdict::Skipped(reason) => {
            match kind {
                "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} was left as it is: {reason}", item.item.parent),
                "prewarm" => crate::service_log!("the Cargo build directory of {} was left as it is: {reason}", item.item.repository.display()),
                _ => crate::service_log!("the review of {} was skipped: {reason}", item.item.oid),
            }
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await;
        }
    }
}
