//! What:
//!  The path where the clone still stands:
//!  the origin cleanup record published before the first destroy,
//!  the snapshot requests with a fresh holder scan and the deferred-origin check before each,
//!  the plan amended with the clone's own history,
//!  the head destroy,
//!  the origin attempt,
//!  the Git settle and the removal's own records.
//! Why:
//!  The checks sit immediately before the requests they guard,
//!  each chunk of a split included,
//!  because a mark set between two checks is the accepted residual
//!  and a check that ran earlier proves nothing about the request it does not guard.
//!  A name the answer leaves is the next run's:
//!  the design retries only after identifying the new own snapshot and reproving the plan,
//!  never on an unexplained refusal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! publishTheOriginRecord(input, plan);
//! sendTheRemaining(input, folder, plan, planned, planPath, pool, requests, residual);
//! const head = destroyTheHead({ plan, folder, probe, device });
//! ```

use crate::worktree_clone_removal_proof_and_plan::fresh_holder_scan::the_holder_lines;
use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_names_and_states::{
    the_names_destroyed,
    the_names_not_destroyed,
    the_states_of,
};
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_inputs::RemovalCompletion;
use crate::worktree_clone_removal_proof_and_plan::removal_destruction::{
    planned_origin_of,
    record_the_answer,
    refuse_a_marked_or_replaced_origin,
};
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::{
    HeadDestroyRequest,
    HeadDestroyed,
    destroy_the_head,
};
use crate::worktree_clone_removal_proof_and_plan::removal_origin_step::{
    attempt_the_origin_cleanup,
    publish_the_origin_record,
};
use crate::worktree_clone_removal_proof_and_plan::removal_records_end::{
    RecordsGone,
    remove_the_removal_records,
};
use crate::worktree_clone_removal_proof_and_plan::removal_report::{
    OriginCleanupDone,
    TheRemovalDone,
};
use crate::worktree_clone_removal_proof_and_plan::removal_self_cancelling_step::{
    TheStopTheRemovalTakes,
    cancelled_at,
};
use crate::worktree_clone_removal_proof_and_plan::removal_settle::SettleAnswer;
use crate::worktree_clone_removal_proof_and_plan::removal_settle_step::settle_the_registration;
use crate::worktree_clone_removal_proof_and_plan::removal_slot_claim::ClaimEnd;
use crate::worktree_clone_removal_proof_and_plan::removal_snapshot_settlement::{
    amend_the_plan,
    settle_the_recorded_names,
    the_new_own_snapshots,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_snapshot_batch::snapshot_batch_outcome::BatchOutcome;
use crate::worktree_clone_snapshot_batch::snapshot_batch_plan::plan_requests;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::{
    RequestResult,
    send_request,
};
use std::path::PathBuf;
/// The path where the clone stands:
/// the origin record,
/// the snapshot requests,
/// the head destroy,
/// the origin attempt,
/// the settle and the records.
pub(super) fn the_head_present_path(
    input: &RemovalCompletion<'_>,
    folder: PlanFolder,
    entered: crate::worktree_clone_removal_proof_and_plan::removal_entry::EnteredRemoval,
    lost: NamesToBeLost,
) -> Result<TheRemovalDone, RemovalStop> {
    let planned = planned_origin_of(input.entry.slot, &entered);
    if entered.plan.head == HeadWord::Gone {
        return Err(RemovalStop::ThePlanContradictsThePool {
            head: String::from(HeadWord::Gone.word()),
            clone: entered.plan.facts.clone_dataset.clone(),
        });
    }
    let mut plan: DestructionPlan = entered.plan;
    let plan_path: PathBuf = entered.plan_path;
    let pool: String = entered.pool;
    // The record is published before the first destroy, and the attempt after the head reads it again.
    publish_the_origin_record(input, &plan)?;
    let mut requests: Vec<BatchOutcome> = Vec::new();
    let mut residual: usize = 0;
    // One pass settles what the pool holds, amends the plan with own history it does not name, and
    // sends what remains in one request per chunk. A name the answer leaves is the next run's:
    // the design retries only after identifying the new own snapshot and reproving the plan,
    // never on an unexplained refusal.
    send_the_remaining(
        input,
        &folder,
        &mut plan,
        &planned,
        plan_path.as_path(),
        pool.as_str(),
        &mut requests,
        &mut residual,
    )?;
    let after_the_batch: Vec<
        crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject,
    > = the_live_snapshots(input, &plan)?;
    settle_the_recorded_names(&mut plan, after_the_batch.as_slice())?;
    let added = the_new_own_snapshots(&plan, after_the_batch.as_slice())?;
    let amended: bool = !added.is_empty();
    if amended {
        amend_the_plan(&mut plan, added.as_slice())?;
        plan.facts.run = input.clock.run()?;
    }
    publish_the_plan(&folder, &plan)?;
    if amended {
        // The own snapshot appeared after this run's first enumeration, so every check runs again and
        // the amendment is submitted once. Every name the first request left keeps its line.
        send_the_remaining(
            input,
            &folder,
            &mut plan,
            &planned,
            plan_path.as_path(),
            pool.as_str(),
            &mut requests,
            &mut residual,
        )?;
        let after_the_amendment: Vec<
            crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject,
        > = the_live_snapshots(input, &plan)?;
        settle_the_recorded_names(&mut plan, after_the_amendment.as_slice())?;
        publish_the_plan(&folder, &plan)?;
    }
    let remaining: Vec<String> = the_names_not_destroyed(&plan);
    if !remaining.is_empty() {
        return Err(RemovalStop::TheSnapshotsRemain {
            names: the_states_of(&plan, remaining.as_slice()),
        });
    }
    residual = residual.max(refuse_the_holders(input, &plan)?);
    refuse_a_marked_or_replaced_origin(input.check, &planned, &plan)?;
    let head: HeadDestroyed = destroy_the_head(&mut HeadDestroyRequest {
        plan: &mut plan,
        folder: &folder,
        probe: input.probe,
        device: input.head,
    })?;
    // The last stop point the design names: a holder outlasts the head destroy. The kernel answers `EBUSY` with every
    //  one of the clone's own snapshots already gone, which is a holder and not a name this run left.
    if head.head == HeadWord::Refused {
        let left: Vec<String> = the_names_not_destroyed(&plan);
        if left.is_empty() && head.errno == u32::try_from(libc::EBUSY).ok() {
            let the: TheStopTheRemovalTakes<'_> = TheStopTheRemovalTakes {
                cancelling: input.cancelling,
                common_dir: input.entry.common_dir,
                slot: input.entry.slot,
                plan: Some(&plan),
                holders: &[],
                stop: crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::TheStopPoint::AtTheHeadDestroy,
            };
            if let Some(stopped) = cancelled_at(&the) {
                return Err(stopped);
            }
        }
        return Err(RemovalStop::TheHeadDestroyWasRefused {
            errno: head.errno,
            names: the_states_of(&plan, left.as_slice()),
        });
    }
    let origin: OriginCleanupDone = attempt_the_origin_cleanup(input, &plan)?;
    let settle: SettleAnswer = settle_the_registration(input, &plan)?;
    let records: RecordsGone =
        remove_the_removal_records(input.entry.common_dir, input.entry.slot, &plan)?;
    return Ok(TheRemovalDone {
        slot: String::from(input.entry.slot),
        dataset: plan.facts.clone_dataset.clone(),
        destroyed: the_names_destroyed(&plan),
        bookmarks: lost.bookmarks.clone(),
        remaining: Vec::new(),
        head: head.head,
        holder_reads_unfinished: residual,
        origin,
        settle,
        records,
        claim: ClaimEnd::Held,
        plan_path,
        requests,
    });
}

/// What:
///  One pass over one removal's remaining names:
///  settle them by the live inventory,
///  amend the plan with the clone's own history it does not name,
///  and send one request per chunk with a fresh holder scan and the deferred-origin check before
///  each.
/// Why:
///  The checks sit immediately before the requests they guard,
///  and the plan is published after each answer,
///  so a kill inside a request leaves the record of every answer that preceded it.
///
/// # Throws
/// A settlement,
/// a scan or a check that refused,
/// or a plan write that refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sendTheRemaining(input, folder, plan, planned, planPath, requests, residual);
/// ```
fn send_the_remaining(
    input: &RemovalCompletion<'_>,
    folder: &PlanFolder,
    plan: &mut DestructionPlan,
    planned: &crate::worktree_removal_deferred_origin_check::planned_origin::PlannedOrigin,
    plan_path: &std::path::Path,
    pool: &str,
    requests: &mut Vec<BatchOutcome>,
    residual: &mut usize,
) -> Result<(), RemovalStop> {
    let live: Vec<crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject> =
        the_live_snapshots(input, plan)?;
    settle_the_recorded_names(plan, live.as_slice())?;
    let added = the_new_own_snapshots(plan, live.as_slice())?;
    if !added.is_empty() {
        amend_the_plan(plan, added.as_slice())?;
        plan.facts.run = input.clock.run()?;
    }
    publish_the_plan(folder, plan)?;
    let remaining: Vec<String> = the_names_not_destroyed(plan);
    if remaining.is_empty() {
        return Ok(());
    }
    let the_pool: String = String::from(pool);
    let requested = plan_requests(the_pool.as_str(), remaining.as_slice(), input.budget).map_err(
        |refusal| {
            return RemovalStop::TheSnapshotSetIsNotSendable {
                said: refusal.to_string(),
                sent: requests.len(),
            };
        },
    )?;
    for chunk in requested.chunks() {
        *residual = (*residual).max(refuse_the_holders(input, plan)?);
        refuse_a_marked_or_replaced_origin(input.check, planned, plan)?;
        let answer: RequestResult =
            send_request(input.snapshots, the_pool.as_str(), chunk.as_slice()).map_err(
                |refusal| {
                    return RemovalStop::TheSnapshotSetIsNotSendable {
                        said: refusal.to_string(),
                        sent: requests.len(),
                    };
                },
            )?;
        let RequestResult::Answered(outcome) = answer else {
            continue;
        };
        record_the_answer(plan, plan_path, &outcome)?;
        publish_the_plan(folder, plan)?;
        requests.push(outcome);
    }
    return Ok(());
}

/// Every live snapshot of the clone,
/// which is the inventory the absence rule and the amendment read.
fn the_live_snapshots(
    input: &RemovalCompletion<'_>,
    plan: &DestructionPlan,
) -> Result<
    Vec<crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject>,
    RemovalStop,
> {
    return input
        .reads
        .snapshots_of(plan.facts.clone_dataset.as_str(), plan.facts.clone_guid)
        .map_err(|reason| {
            return RemovalStop::InventoryUnreadable {
                clone: plan.facts.clone_dataset.clone(),
                reason,
            };
        });
}

pub(super) fn publish_the_plan(
    folder: &PlanFolder,
    plan: &DestructionPlan,
) -> Result<(), RemovalStop> {
    return match folder.publish(plan) {
        Ok(_path) => Ok(()),
        Err(refusal) => Err(RemovalStop::ThePlanCouldNotHoldTheAnswer {
            path: folder.path().join(plan.facts.clone_guid.file_name()),
            said: refusal.message(folder.path()),
        }),
    };
}

/// What:
///  One fresh all-namespace scan before one destroy,
///  which refuses the holders it sees and reports the reads it could not finish.
/// Why:
///  A holder the scan sees blocks destruction even when the slot is clear,
///  and an incomplete read is the residual the destroy-anyway decision accepts,
///  so the two answers travel apart.
///
/// # Throws
/// Holders the scan named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unfinished = refuseTheHolders(input, plan);
/// ```
fn refuse_the_holders(
    input: &RemovalCompletion<'_>,
    plan: &DestructionPlan,
) -> Result<usize, RemovalStop> {
    let scan = input
        .holders
        .holders_of(plan.facts.clone_dataset.as_str(), plan.facts.clone_guid)?;
    let lines: Vec<String> = the_holder_lines(&scan);
    if !lines.is_empty() {
        // The scan before a delete is the design's own stop point: a holder the scan sees stops the destruction and a
        //  removal without a forced arm puts the clone back rather than going on.
        let the: TheStopTheRemovalTakes<'_> = TheStopTheRemovalTakes {
            cancelling: input.cancelling,
            common_dir: input.entry.common_dir,
            slot: input.entry.slot,
            plan: Some(plan),
            holders: lines.as_slice(),
            stop: crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::TheStopPoint::ByAScanBeforeADelete,
        };
        if let Some(stopped) = cancelled_at(&the) {
            return Err(stopped);
        }
        return Err(RemovalStop::HoldersSeeTheClone {
            holders: lines,
            incomplete: scan.incomplete.len(),
        });
    }
    return Ok(scan.incomplete.len());
}
