//! What:
//!  How one item whose kind answers with words rather than a push is settled:
//! a skipped one is done and says why,
//! a failed one is failed and says why,
//! and one that answered `Retry` keeps its claim,
//! whose lease running out queues it again with one more try.
//! Why:
//!  A trash and a preview and a relink and a subvolume delete answer the same four verdicts and leave the person the same two kinds of line,
//! so the drain keeps one account of them,
//! and the file they sit in stays short enough to read whole.
//! A retryable answer is a condition that can pass (a busy volume,
//! a full disk,
//! an I/O error),
//! so it may not become a failed record:
//! nothing scans `failed`,
//! and an id that reached it can never be queued again,
//! which leaves a copy of the source that proved stale on the disk for ever.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleWithTheWords(held, workDir, item, verdict, kind)
//! ```

use crate::service_claim_renewal::HeldClaims;
use crate::service_work_handlers_replanning_and_acceptance::Verdict;
use crate::service_work_storage_awaited::settle_all_awaiting;
use crate::work_item::{Claimed, DONE, FAILED};
use std::path::Path;

/// What:
///  Settle one item whose kind answers with words rather than a push:
/// a skipped one is done and says why,
/// a failed one is failed and says why,
/// and a retryable one is left where it is with the reason said.
/// Why:
///   The kinds differ only in the words the person reads,
/// so the account of them is one function.
/// A retried item is settled by nothing here:
/// its claim keeps its lease,
/// and the lease running out is what returns the record to `pending` with one more try
/// (`work_item_recovery.rs`),
/// which is the retry the answer asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleWithTheWords(held, workDir, item, verdict, kind)
/// ```
pub(crate) async fn settle_with_the_words(held: HeldClaims, work_dir: &Path, item: Claimed, verdict: Verdict, kind: &str) -> Option<HeldClaims> {
    match verdict {
        Verdict::Done => {
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await;
        }
        Verdict::Delivered { said, .. } => {
            // Only a push answers this, and a push is settled by `settle_after_the_push`, which records what it said before
            //  the item moves. A kind that answers in words reaches this only if a handler is given a push's verdict shape,
            //  which this module cannot make: the words are said rather than dropped, and the item is settled as done, because
            //  nothing here may fail a person's work over the shape of an answer.
            crate::service_warn!("a step that answers in words answered as a push did, and its words are said here: {said}");
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await;
        }
        Verdict::Failed(reason) => {
            say_it_failed(kind, &item, reason.as_str());
            return settle_all_awaiting(held, work_dir, vec![item], FAILED).await;
        }
        Verdict::Retry(reason) => {
            say_it_failed(kind, &item, format!("{reason}; this is tried again when its claim runs out").as_str());
            return Some(held);
        }
        Verdict::Skipped(reason) => {
            match kind {
                "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} was left as it is: {reason}", item.item.parent),
                "prewarm" => crate::service_log!("the Cargo build directory of {} was left as it is: {reason}", item.item.repository.display()),
                "relink" => crate::service_log!("the package state of {} was left as it is: {reason}", item.item.repository.display()),
                "subvolume delete" => crate::service_log!("the subvolume {} was left as it is: {reason}", item.item.parent),
                _ => crate::service_log!("the review of {} was skipped: {reason}", item.item.oid),
            }
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await;
        }
    }
}

/// What:
///  Say in the service's log what one item of `kind` could not do,
/// and why.
/// Why:
///   The kinds differ only in what the person reads,
/// and a retried item says the same thing plus what happens
///  next,
/// so one function says it for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayItFailed(kind, item, reason) // the trash's line goes to standard error, the rest to the journal
/// ```
fn say_it_failed(kind: &str, item: &Claimed, reason: &str) {
    match kind {
        "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} could not be emptied: {reason}", item.item.parent),
        "prewarm" => crate::service_warn!("the Cargo build directory of {} could not be filled: {reason}", item.item.repository.display()),
        "relink" => crate::service_warn!("the package state of {} could not be relinked: {reason}", item.item.repository.display()),
        "subvolume delete" => crate::service_warn!("the subvolume {} could not be deleted: {reason}", item.item.parent),
        _ => crate::service_warn!("the review of {} failed: {reason}", item.item.oid),
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_work_settling_with_words_tests.rs"]
mod tests;
