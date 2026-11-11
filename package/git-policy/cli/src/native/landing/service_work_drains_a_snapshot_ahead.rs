//! What:
//!  The service's drain of the snapshots it makes ahead of the next add,
//! as one step of a round.
//! Why:
//!  A snapshot the wrapper makes ahead is the one kind of work there that a person's command is waiting for,
//! and
//!  `BTRFS_IOC_SNAP_CREATE_V2` waits for a btrfs transaction commit of the whole filesystem,
//! so it runs before the upkeep and
//!  every kind that frees space,
//! on a thread of the process's blocking pool,
//! and it settles the words for its own kind.
//! A retryable answer settles nothing:
//! the record keeps its claim,
//! and the claim's lease running
//!  out is what queues it again with one more try,
//! so a volume that was busy or full is asked again rather than
//!  dropped.
//! The step sits in a
//!  file of its own so the drain of the round stays readable and short.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const back = await drainASnapshotAhead(held, workDir, itemsTaken, handlers);
//! ```

use crate::landing::service_work_handlers_replanning_and_acceptance::{WorkHandlers, is_acceptable};
use crate::service_claim_renewal::HeldClaims;
use crate::service_work_storage_awaited::still_held_awaiting;
use crate::work_item::{Claimed, DONE, FAILED};
use std::path::Path;

/// What:
///  Run the snapshots ahead a round claimed,
/// each awaited on the runtime,
/// and settle what each one came to.
/// Why:
///   The claims go back to the set they came from,
/// so nothing here locks it,
/// and a claim that another instance took
///  over is left to that instance.
/// A record this service may not act on is one the round already refused,
/// so every item
///  here is acceptable.
/// A retryable answer settles nothing,
/// because the condition it named can pass:
/// the claim keeps its lease,
/// and the lease running out is what queues the record again with one more try.
/// `None` is a round that cannot hand its claims back,
/// which ends the drain where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function drainASnapshotAhead(held, workDir, claimed, handlers) { const mine = await stillHeld(held, workDir, claimed); for (const item of mine) { const verdict = await handlers.snapshotAheadAwaiting(item.item); held = await settleWithTheWords(held, workDir, item, verdict, 'snapshot ahead'); } return held; }
/// ```
pub(crate) async fn drain_a_snapshot_ahead<H: WorkHandlers + ?Sized>(held: HeldClaims, work_dir: &Path, claimed: Vec<Claimed>, handlers: &H) -> Option<HeldClaims> {
    let (mut back, mine): (HeldClaims, Vec<Claimed>) = match still_held_awaiting(held, work_dir, claimed).await {
        Some((back, mine)) => (back, mine),
        None => return None,
    };
    for item in mine {
        if !is_acceptable(&item.item, item.item.git_dir.as_path()) {
            back = settle(back, work_dir, item, FAILED).await?;
            continue;
        }
        let verdict: crate::service_work::Verdict = handlers.snapshot_ahead_awaiting(&item.item).await;
        let settle_as: Option<&'static str> = match &verdict {
            crate::service_work::Verdict::Done | crate::service_work::Verdict::Skipped(_) => Some(DONE),
            crate::service_work::Verdict::Failed(reason) => {
                crate::service_warn!("the snapshot ahead of {} could not be made: {reason}", item.item.repository.display());
                Some(FAILED)
            }
            // The copy could not be made for a reason that can pass, so nothing is settled: the claim keeps its lease,
            //  and the lease running out queues the record again with one more try.
            crate::service_work::Verdict::Retry(reason) => {
                crate::service_warn!(
                    "the snapshot ahead of {} could not be made yet: {reason}; this is tried again when its claim runs out",
                    item.item.repository.display()
                );
                None
            }
        };
        if let Some(state) = settle_as {
            back = settle(back, work_dir, item, state).await?;
        }
    }
    return Some(back);
}

/// What:
///  Move one settled claim to its state directory,
/// and answer the set it left.
/// Why:
///   Every step of the drain settles a claim the same way,
/// so the words for a state are in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settle = (held, workDir, item, state) => settleAll(held, workDir, [item], state);
/// ```
async fn settle(held: HeldClaims, work_dir: &Path, item: Claimed, state: &'static str) -> Option<HeldClaims> {
    return crate::service_work_storage_awaited::settle_all_awaiting(held, work_dir, vec![item], state).await;
}
