//! What:
//!  How one item whose kind answers with words rather than a push is settled:
//! a skipped one is done and says why,
//! a failed one is failed and says why,
//! one that answered `Retry` keeps its claim,
//! whose lease running out queues it again with one more try,
//! and one that answered `Deferred` is queued again at once,
//! with the tries it had,
//! for the look the caller is told to make.
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
use crate::work_item::{Claimed, DONE, FAILED, PENDING};
use std::path::Path;

/// What:
///  What settling one item did with its claim,
/// for the caller that must look at the directory again.
/// Why:
///  A record left for a later look is queued again here,
/// so nothing but the caller can say when to come back for it,
/// and a caller told nothing would leave it to the next wake a person's command happens to make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Settling = { held: HeldClaims; lookAgain: boolean }
/// ```
pub(crate) struct Settling {
    /// The claims this instance still holds.
    pub(crate) held: HeldClaims,
    /// Whether an item asked for a later look at this directory because its own bound ran out with work left.
    pub(crate) look_again: bool,
}

/// What:
///  Settle one item whose kind answers with words rather than a push:
/// a skipped one is done and says why,
/// a failed one is failed and says why,
/// a retryable one is left where it is with the reason said,
/// and one whose bound ran out with work left is queued again for a later look.
/// Why:
///   The kinds differ only in the words the person reads,
/// so the account of them is one function.
/// A retried item is settled by nothing here:
/// its claim keeps its lease,
/// and the lease running out is what returns the record to `pending` with one more try
/// (`work_item_recovery.rs`),
/// which is the retry the answer asked for.
///  The look that kept it answers that moment as when to look at this directory again
/// (`service_work_drain_pending_claims.rs`),
/// so the record is queued again and run without another request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleWithTheWords(held, workDir, item, verdict, kind)
/// ```
pub(crate) async fn settle_with_the_words(held: HeldClaims, work_dir: &Path, item: Claimed, verdict: Verdict, kind: &str) -> Option<Settling> {
    match verdict {
        Verdict::Done => {
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await.map(|back| { return Settling { held: back, look_again: false }; });
        }
        Verdict::Delivered { said, .. } => {
            // Only a push answers this, and a push is settled by `settle_after_the_push`, which records what it said before
            //  the item moves. A kind that answers in words reaches this only if a handler is given a push's verdict shape,
            //  which this module cannot make: the words are said rather than dropped, and the item is settled as done, because
            //  nothing here may fail a person's work over the shape of an answer.
            crate::service_warn!("a step that answers in words answered as a push did, and its words are said here: {said}");
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await.map(|back| { return Settling { held: back, look_again: false }; });
        }
        Verdict::Failed(reason) => {
            say_it_failed(kind, &item, reason.as_str());
            return settle_all_awaiting(held, work_dir, vec![item], FAILED).await.map(|back| { return Settling { held: back, look_again: false }; });
        }
        // What:
        //  A step that ran out of its own bound with work left is queued again here,
        //  with the tries it had,
        //  and says how far it got.
        // Why:
        //  Nothing of it failed, so it may not spend a retry or reach `failed`,
        //  and its work is finished over several rounds, so it must come back for one.
        //  The claim goes back to `pending` rather than waiting for its lease:
        //  the round it asks for is the next look, not a retry after a pause a failure waits.
        Verdict::Deferred(reason) => {
            say_it_went_as_far_as_its_bound(kind, &item, reason.as_str());
            return settle_all_awaiting(held, work_dir, vec![item], PENDING).await.map(|back| { return Settling { held: back, look_again: true }; });
        }
        Verdict::Retry(reason) => {
            say_it_failed(kind, &item, format!("{reason}; this is tried again when its claim runs out").as_str());
            return Some(Settling { held, look_again: false });
        }
        Verdict::Skipped(reason) => {
            match kind {
                "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} was left as it is: {reason}", item.item.parent),
                "prewarm" => crate::service_log!("the Cargo build directory of {} was left as it is: {reason}", item.item.repository.display()),
                "relink" => crate::service_log!("the package state of {} was left as it is: {reason}", item.item.repository.display()),
                "subvolume delete" => crate::service_log!("the subvolume {} was left as it is: {reason}", item.item.parent),
                _ => crate::service_log!("the review of {} was skipped: {reason}", item.item.oid),
            }
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await.map(|back| { return Settling { held: back, look_again: false }; });
        }
    }
}

/// What:
///  Say in the service's log that a step's own bound ran out with work left of it,
///  and how far it got.
/// Why:
///  A round that removed entries and ran out of time is not a failure and must not read as one,
/// and a person reading the log must see the work going on rather than a fault repeated.
/// The line is written once for each round that went as far as its bound allowed,
///  which is what says the item is not stuck.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayItWentAsFarAsItsBound(kind, item, reason) // one line, at the level of progress
/// ```
fn say_it_went_as_far_as_its_bound(kind: &str, item: &Claimed, reason: &str) {
    match kind {
        "trash" => crate::service_log!("the trash {} went as far as its bound allowed and is taken up again in a later look: {reason}", item.item.parent),
        _ => crate::service_log!(
            "the {} of {} went as far as its bound allowed and is taken up again in a later look: {reason}",
            kind,
            item.item.repository.display()
        ),
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
