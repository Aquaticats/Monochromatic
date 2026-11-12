//! What:
//!  The drain's step that deletes the subvolumes a removal renamed out of the person's way:
//! every item of that kind still held runs its handler,
//! and the verdict it answers is settled with the words the person reads.
//! Why:
//!  A removal records the path and returns,
//! so the one kernel call that frees the worktree happens here,
//! with no person waiting for it (`worktree_remove_hand_over.rs`).
//! The step sits in a file of its own
//!  because the drain that calls it is one kind of work away from its own line budget
//! (`service_work_drain_pending_claims.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await deleteSubvolumesAwaiting(held, workDir, deletions, handlers)
//! ```

use crate::service_claim_renewal::HeldClaims;
use crate::service_work_handlers_replanning_and_acceptance::{Verdict, WorkHandlers};
use crate::service_work_settling_with_words::settle_with_the_words;
use crate::service_work_storage_awaited::still_held_awaiting;
use crate::work_item::Claimed;
use std::path::Path;

/// What:
///  Run every still-held deletion through the handler and settle it with its verdict.
/// Why:
///   A claim another instance took over is not this round's work:
/// it is left out and the set is handed back,
/// so the caller settles what is left and carries the set on.
/// `None` means the set went with a job the system refused a thread for,
/// which ends the round rather than the set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function deleteSubvolumesAwaiting(held, workDir, deletions, handlers): Promise<HeldClaims | null>
/// ```
pub(crate) async fn delete_subvolumes_awaiting<H: WorkHandlers + ?Sized>(
    mut held: HeldClaims,
    work_dir: &Path,
    deletions: Vec<Claimed>,
    handlers: &H,
) -> Option<HeldClaims> {
    let deleted: Vec<Claimed> = match still_held_awaiting(held, work_dir, deletions).await {
        Some((back, mine)) => {
            held = back;
            mine
        }
        None => return None,
    };
    for item in deleted {
        let verdict: Verdict = handlers.subvolume_delete_awaiting(&item.item).await;
        match settle_with_the_words(held, work_dir, item, verdict, "subvolume delete").await {
            Some(back) => held = back,
            None => return None,
        }
    }
    return Some(held);
}
