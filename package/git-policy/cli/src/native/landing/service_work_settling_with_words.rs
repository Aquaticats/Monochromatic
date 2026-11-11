//! What:
//!  How one item whose kind answers with words rather than a push is settled:
//! a skipped one is done and says why,
//! a failed one is failed and says why,
//! and a retryable one stays queued until its attempts run out.
//! Why:
//!  A trash and a preview and a relink and a subvolume delete answer the same four verdicts and leave the person the same two kinds of line,
//! so the drain keeps one account of them,
//! and the file they sit in stays short enough to read whole.
//! A retryable verdict is the one that must not be settled where a failure goes:
//! later rounds claim only
//!  `pending`,
//! and recovery reads `claimed` and `recovering`,
//! never `failed`,
//! so a record settled as failed is never tried
//!  again.
//! `tries` bounds the retries:
//! a call the kernel refuses every time would otherwise keep a round busy for as
//!  long as the service lives,
//! and the line the person reads names the directory that was left.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleWithTheWords(held, workDir, item, verdict, kind)
//! ```

use crate::private_storage::replace_private_file_unsynced;
use crate::service_claim_renewal::HeldClaims;
use crate::service_work_handlers_replanning_and_acceptance::{MAX_TRIES, Verdict};
use crate::service_work_storage_awaited::settle_all_awaiting;
use crate::work_item::{CLAIMED, Claimed, DONE, FAILED, PENDING, WorkItem, state_directory};
use std::path::Path;

/// What:
///  Settle one item whose kind answers with words rather than a push:
/// a skipped one is done and says why,
/// a failed one is failed and says why,
/// and a retryable one is queued again while its attempts last.
/// Why:
///   The kinds differ only in the words the person reads,
/// so the account of them is one function.
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
        Verdict::Failed(reason) => {
            say_the_failure(kind, &item, reason.as_str());
            return settle_all_awaiting(held, work_dir, vec![item], FAILED).await;
        }
        Verdict::Retry(reason) => {
            return settle_a_retry(held, work_dir, item, kind, reason.as_str()).await;
        }
        Verdict::Skipped(reason) => {
            say_the_skip(kind, &item, reason.as_str());
            return settle_all_awaiting(held, work_dir, vec![item], DONE).await;
        }
    }
}

/// What:
///  Queue a retryable item again,
/// and give it up with the words of a failure once `MAX_TRIES` attempts have been made.
/// Why:
///   A verdict that asks for another try is a promise a later round keeps,
/// so the record must stay where a later round looks;
/// and nothing
///  the kernel refuses every time may keep retrying while the service lives,
/// so the count of attempts already in the record decides when it
///  stops.
/// The count is written into the held record before the record moves,
/// so a queue that cannot be written costs the retry and never an
///  unbounded one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// attempt = item.tries + 1; if (attempt >= MAX_TRIES) { sayFailure(reason); settle(FAILED); } else if (countAttempt()) settle(PENDING); else { sayFailure(reason); settle(FAILED); }
/// ```
async fn settle_a_retry(held: HeldClaims, work_dir: &Path, item: Claimed, kind: &str, reason: &str) -> Option<HeldClaims> {
    let attempt: u32 = item.item.tries.saturating_add(1);
    if attempt >= MAX_TRIES {
        crate::service_warn!("the {kind} {} did not go through in {MAX_TRIES} attempts, so it is not tried again", item.item.parent);
        say_the_failure(kind, &item, reason);
        return settle_all_awaiting(held, work_dir, vec![item], FAILED).await;
    }
    match count_an_attempt(work_dir, &item) {
        Ok(()) => {
            say_a_retry(kind, &item, reason);
            return settle_all_awaiting(held, work_dir, vec![item], PENDING).await;
        }
        Err(why) => {
            // A retry that cannot be counted would be an unbounded one, so the item is given up on with both reasons.
            crate::service_warn!("the attempt at {} cannot be counted ({why}), so it is not tried again", item.item.id());
            say_the_failure(kind, &item, reason);
            return settle_all_awaiting(held, work_dir, vec![item], FAILED).await;
        }
    }
}

/// What:
///  Count one attempt in the held record.
/// Why:
///   `tries` is the count the queue already keeps of the attempts an item cost,
/// and it travels with the record,
/// so the next
///  round reads the count this one wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(join(claimed, item.name), { ...item.item, tries: item.item.tries + 1 })
/// ```
///
/// # Errors
///
/// The reason the held record could not be rewritten.
fn count_an_attempt(work_dir: &Path, item: &Claimed) -> Result<(), String> {
    let next: WorkItem = WorkItem { tries: item.item.tries.saturating_add(1), ..item.item.clone() };
    let path: std::path::PathBuf = state_directory(work_dir, CLAIMED).join(item.name.as_str());
    return match replace_private_file_unsynced(path.as_path(), next.encode().as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("the record {} cannot be rewritten ({error})", path.display())),
    };
}

/// What:
///  The line a kind leaves when its work failed and is not tried again.
/// Why:
///   The words of each kind name what the person still has and where it stands,
/// so one function says them wherever a failure is settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayTheFailure(kind, item, reason)
/// ```
fn say_the_failure(kind: &str, item: &Claimed, reason: &str) {
    match kind {
        "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} could not be emptied: {reason}", item.item.parent),
        "prewarm" => crate::service_warn!("the Cargo build directory of {} could not be filled: {reason}", item.item.repository.display()),
        "relink" => crate::service_warn!("the package state of {} could not be relinked: {reason}", item.item.repository.display()),
        "subvolume delete" => crate::service_warn!("the subvolume {} could not be deleted: {reason}", item.item.parent),
        _ => crate::service_warn!("the review of {} failed: {reason}", item.item.oid),
    }
}

/// What:
///  The line a kind leaves when its work is not this item's to do.
/// Why:
///   A skip is a finished item with a reason,
/// and the words of each kind name the object they are about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayTheSkip(kind, item, reason)
/// ```
fn say_the_skip(kind: &str, item: &Claimed, reason: &str) {
    match kind {
        "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} was left as it is: {reason}", item.item.parent),
        "prewarm" => crate::service_log!("the Cargo build directory of {} was left as it is: {reason}", item.item.repository.display()),
        "relink" => crate::service_log!("the package state of {} was left as it is: {reason}", item.item.repository.display()),
        "subvolume delete" => crate::service_log!("the subvolume {} was left as it is: {reason}", item.item.parent),
        _ => crate::service_log!("the review of {} was skipped: {reason}", item.item.oid),
    }
}

/// What:
///  The line a kind leaves when its work may pass and is tried again.
/// Why:
///   A retry leaves the person with work that is not done yet,
/// so the line says so and names the object it is about;
/// the words of the
///  attempt that gives up are the failure's own (`say_the_failure`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sayARetry(kind, item, reason)
/// ```
fn say_a_retry(kind: &str, item: &Claimed, reason: &str) {
    match kind {
        "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} could not be emptied and is tried again: {reason}", item.item.parent),
        "prewarm" => crate::service_log!("the Cargo build directory of {} could not be filled and is tried again: {reason}", item.item.repository.display()),
        "relink" => crate::service_log!("the package state of {} could not be relinked and is tried again: {reason}", item.item.repository.display()),
        "subvolume delete" => crate::service_warn!("the subvolume {} could not be deleted and is tried again: {reason}", item.item.parent),
        _ => crate::service_log!("the review of {} was skipped and is tried again: {reason}", item.item.oid),
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_work_settling_with_words_tests.rs"]
mod tests;
