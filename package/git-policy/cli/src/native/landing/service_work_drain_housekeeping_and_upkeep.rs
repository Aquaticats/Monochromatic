//! What:
//!  The housekeeping one round of the drain may start,
//! the upkeep it runs on the items it holds,
//! and what it leaves for a later round.
//! Why:
//!  A round spends a bounded slice on housekeeping and the person's pushes come first
//! (`housekeeping_of_one_round.rs`),
//! so the three steps that ask, run and defer that housekeeping stand together,
//!  apart from the drain that orders the kinds of work
//! (`service_work_drain_pending_claims.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slice = HousekeepingOfOneRound.startingNow(bound);
//! held = await upkeepAwaiting(held, workDir, handlers, upkeep, slice, started, leftBehind);
//! await leaveForALaterRound(held, gitDir, workDir, leftBehind, slice);
//! ```

use super::service_work_handlers_replanning_and_acceptance::{
    Verdict,
    WorkHandlers,
};
use crate::housekeeping_of_one_round::HousekeepingOfOneRound;
use crate::service_claim_renewal::HeldClaims;
use crate::service_work_storage_awaited::{
    settle_all_awaiting,
    still_held_awaiting,
};
use crate::work_item::{
    Claimed,
    DONE,
    PENDING,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Instant;

/// What:
///  The identity of a claimed item,
/// for the one line that names the housekeeping a round left for a later one.
/// Why:
///  A named function keeps the line that builds the list to the words that describe it,
/// and the drain reads it once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const idOf = (item: Claimed) => item.id;
/// ```
fn id_of_claimed(item: &Claimed) -> String {
    return item.id.clone();
}

/// What:
///  Whether this round may start the housekeeping item it is holding:
/// its slice still has room,
/// or no housekeeping of this round has started yet.
/// Why:
///   The bound is about how much housekeeping a round starts,
/// so the first item of a round always starts.
/// A round that could spend its whole slice on the listing,
/// the settling of the claims it refused and the steps of the kinds before this one would otherwise leave every item for a
///  later round that would spend its slice the same way,
/// so a repository's trash would be left to a round that never comes and the loop of later rounds would spin.
pub(super) fn may_start_housekeeping(
    slice: &HousekeepingOfOneRound,
    started: &mut bool,
) -> bool {
    if *started && slice.spent() {
        return false;
    }
    *started = true;
    return true;
}

/// What:
///  Runs the upkeep of the items this round still holds,
/// until its slice is spent or none is left,
/// and settles each as it ends.
/// Why:
///   The person's push is the backup and the upkeep is what keeps the repository cheap to read,
/// so a round that has spent its housekeeping stops starting upkeep here and the rest waits for a later round,
///  named by `leave_housekeeping_for_a_later_round`.
/// The claims travel in and out of every storage job,
/// so the caller keeps its one owner of them;
/// no answer means a job the system refused a thread for took the set,
///  and the round is over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// held = await upkeepAwaiting(held, workDir, handlers, upkeep, slice, started, leftBehind) // undefined ends the round
/// ```
pub(super) async fn upkeep_the_held_items_awaiting<H: WorkHandlers + Sync + ?Sized>(
    mut held: HeldClaims,
    work_dir: &Path,
    handlers: &H,
    upkeep: Vec<Claimed>,
    slice: &HousekeepingOfOneRound,
    housekeeping_has_started: &mut bool,
    left_for_a_later_round: &mut Vec<Claimed>,
) -> Option<HeldClaims> {
    let mut upkept: Vec<PathBuf> = Vec::new();
    let still_here: Vec<Claimed> = match still_held_awaiting(held, work_dir, upkeep).await {
        Some((back, mine)) => {
            held = back;
            mine
        }
        None => return None,
    };
    for item in still_here {
        // The person's push is the backup and the upkeep is what keeps the repository cheap to read, so a round that has
        //  spent its housekeeping ends here and the rest waits for a later round, named below.
        if !may_start_housekeeping(slice, housekeeping_has_started) {
            left_for_a_later_round.push(item);
            continue;
        }
        if !upkept.contains(&item.item.repository) {
            upkept.push(item.item.repository.clone());
            let upkeep_began: Instant = Instant::now();
            match handlers.maintenance_awaiting(&item.item).await {
                Verdict::Failed(reason) | Verdict::Retry(reason) => {
                    crate::service_warn!("automatic maintenance did not run: {reason}")
                }
                // Only a push answers `Delivered`, and the upkeep of this kind keeps no record; what it said would be a line about
                //  work that was not a push, so it is said in the journal beside the line that the upkeep ran.
                Verdict::Delivered {
                    said,
                    ..
                } => crate::service_log!(
                    "automatic maintenance ran maintenance_ms={} and said: {said}",
                    upkeep_began.elapsed().as_millis()
                ),
                Verdict::Done | Verdict::Skipped(_) => crate::service_log!(
                    "automatic maintenance ran maintenance_ms={}",
                    upkeep_began.elapsed().as_millis()
                ),
            };
        }
        match settle_all_awaiting(held, work_dir, vec![item], DONE).await {
            Some(back) => held = back,
            None => return None,
        }
    }
    return Some(held);
}

/// What:
///  Puts the housekeeping this round did not start back to `pending` for a later round,
/// and says once in the journal what it left where.
/// Why:
///  A person reading the journal sees where their trash went,
/// and the service says when it looks again.
/// A set the system refused a thread for cannot be handed back,
/// and the records it held keep their lease.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await leaveForALaterRound(held, gitDir, workDir, leftBehind, slice)
/// ```
pub(super) async fn leave_housekeeping_for_a_later_round(
    held: HeldClaims,
    git_dir: &Path,
    work_dir: &Path,
    left_for_a_later_round: &mut Vec<Claimed>,
    slice: &HousekeepingOfOneRound,
) {
    let names: Vec<String> = left_for_a_later_round.iter().map(id_of_claimed).collect();
    crate::service_log!(
        "the round leaves {} housekeeping items of {} for a later round, because it spent {} ms of its {} ms on housekeeping: {}",
        names.len(),
        git_dir.display(),
        slice.spent_ms(),
        slice.bound().as_millis(),
        names.join(", ")
    );
    // A set the system refused a thread for cannot be handed back, and the records it held keep their lease.
    let _settled: Option<HeldClaims> = settle_all_awaiting(
        held,
        work_dir,
        std::mem::take(left_for_a_later_round),
        PENDING,
    )
    .await;
}
