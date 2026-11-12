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
    clear_push_failures_before_awaiting, record_push_failure_awaiting, record_push_report_awaiting, record_push_skip_awaiting, settle_all_awaiting,
};
use crate::push_failure_notice::PushOutcome;
use crate::work_item::{Claimed, DONE, FAILED};
use std::path::Path;

/// What:
///  What one push's verdict does to its records and to the retry plan.
/// Why:
///   The verdicts differ only in what they leave for the person and in what the plan keeps,
/// so they are one function
///  and the drain's loop stays readable.
/// A record goes before its items settle:
/// a service that dies between the
///  two pushes again,
/// and nothing is lost.
///  A record the storage refused never leaves its items settled as if it existed:
/// a push that landed keeps the words nowhere else, so its item is failed and the words are said in the journal,
/// while a refusal or a skip the person was not told about leaves the claims to their lease,
/// and the next try asks
///  the server again and records it then.
///  A push a plan carries may settle either way:
/// that plan says the same words on its next try.
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
        Verdict::Delivered { commit, said } => {
            // The push landed, so an earlier failure of its branch is no longer true and no retry plan is left for it. What it
            //  said is a record the next command shows, and that record goes before the items settle: an item in `done` always has
            //  its record, so a storage that refused the words leaves the item failed rather than done, and the words are said in
            //  the journal where the step that was refused them says them. Nothing makes an accepted push say them again.
            let recorded: bool = match newest {
                Some(latest) if !said.is_empty() => {
                    record_push_report_awaiting(git_dir, latest.item.branch_ref.as_str(), commit.as_str(), said.as_str()).await.is_ok()
                }
                _ => true,
            };
            if let Some(latest) = newest {
                clear_push_failures_before_awaiting(git_dir, latest.item.branch_ref.as_str(), pushing_since).await;
                replanning.forget(git_dir, latest.item.branch_ref.as_str());
            }
            return settle_all_awaiting(held, work_dir, settling, if recorded { DONE } else { FAILED }).await;
        }
        Verdict::Retry(text) => {
            // Without handlers that retry, a retryable push is a failed one. A push that can pass is not known to
            //  have reached the remote, so its record says that instead of naming what was refused, and it says
            //  whether a plan will try it again.
            let (said, outcome, carried): (String, PushOutcome, bool) = match (newest, replanning.first) {
                (Some(latest), Some(first)) => {
                    let planned: PlannedRetry = planned_retry(git_dir, replanning, first, &latest.item, 1, text.as_str());
                    // The framing promises a retry only when the plan the words speak of exists.
                    let outcome: PushOutcome = if planned.kept { PushOutcome::Unfinished } else { PushOutcome::UnfinishedWithNoRetry };
                    (planned.words, outcome, planned.kept)
                }
                _ => (text, PushOutcome::UnfinishedWithNoRetry, false),
            };
            let recorded: bool = match newest {
                Some(latest) => {
                    record_push_failure_awaiting(git_dir, latest.item.branch_ref.as_str(), latest.item.oid.as_str(), outcome, said.as_str()).await.is_ok()
                }
                None => true,
            };
            // A plan that carries the push says these words again on its next try, so the items may settle. Without one the claims
            //  are left to their lease: the push is tried again by itself, and its words are recorded then.
            if !recorded && !carried {
                return Some(held);
            }
            return settle_all_awaiting(held, work_dir, settling, FAILED).await;
        }
        Verdict::Failed(text) => {
            // A failure the service gives up on is no longer waiting for a new try.
            if let Some(latest) = newest {
                replanning.forget(git_dir, latest.item.branch_ref.as_str());
            }
            // The record goes before the items settle, so an item in `failed` always has its record. A record the storage refused
            //  leaves the claims to their lease, so the push is tried again by itself and the server's refusal is recorded then:
            //  settling as failed would be the end of it, with nothing said and nothing that could say it.
            let recorded: bool = match newest {
                Some(latest) => {
                    record_push_failure_awaiting(git_dir, latest.item.branch_ref.as_str(), latest.item.oid.as_str(), PushOutcome::Refused, text.as_str())
                        .await
                        .is_ok()
                }
                None => true,
            };
            if !recorded {
                return Some(held);
            }
            return settle_all_awaiting(held, work_dir, settling, FAILED).await;
        }
        Verdict::Skipped(text) => {
            // Nothing was pushed, so an earlier failure of the branch is still true and stays. A skip with words leaves
            //  its own record before the items settle; one that could not be recorded leaves the claims to their lease, so the
            //  skip is worked out again and recorded then, because settling it either way would leave the person no reason for a
            //  backup that does not exist.
            let recorded: bool = match newest {
                Some(latest) if !text.is_empty() => record_push_skip_awaiting(git_dir, latest.item.branch_ref.as_str(), latest.item.oid.as_str(), text.as_str()).await.is_ok(),
                _ => true,
            };
            if !recorded {
                return Some(held);
            }
            return settle_all_awaiting(held, work_dir, settling, DONE).await;
        }
    }
}
