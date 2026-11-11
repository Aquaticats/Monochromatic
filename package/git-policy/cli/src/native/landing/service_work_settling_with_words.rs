//! What:
//!  How one item whose kind answers with words rather than a push is settled:
//! a skipped one is done and says why,
//! a failed one is failed and says why.
//! Why:
//!  A trash and a preview and a relink and a subvolume delete answer the same four verdicts and leave the person the same two kinds of line,
//! so the drain keeps one account of them,
//! and the file they sit in stays short enough to read whole.
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
/// a failed one is failed and says why.
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
        Verdict::Failed(reason) | Verdict::Retry(reason) => {
            match kind {
                "trash" => eprintln!("monochromatic-git-policy-cli-service: the trash {} could not be emptied: {reason}", item.item.parent),
                "prewarm" => crate::service_warn!("the Cargo build directory of {} could not be filled: {reason}", item.item.repository.display()),
                "relink" => crate::service_warn!("the package state of {} could not be relinked: {reason}", item.item.repository.display()),
                "subvolume delete" => crate::service_warn!("the subvolume {} could not be deleted: {reason}", item.item.parent),
                _ => crate::service_warn!("the review of {} failed: {reason}", item.item.oid),
            }
            return settle_all_awaiting(held, work_dir, vec![item], FAILED).await;
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

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_work_settling_with_words_tests.rs"]
mod tests;
