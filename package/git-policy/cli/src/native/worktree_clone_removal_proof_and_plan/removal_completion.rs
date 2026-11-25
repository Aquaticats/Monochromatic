//! What:
//!  One removal from its plan to its end:
//!  the origin cleanup record before the first destroy,
//!  the snapshot requests with a fresh check and a fresh holder scan before each,
//!  the names settled by their GUIDs,
//!  the head destroy,
//!  the origin attempt,
//!  the Git settle,
//!  the removal's own records,
//!  and the report.
//! Why:
//!  This is the writing path plan step 16 builds behind the removal's cut:
//!  nothing in the release's command path calls it,
//!  and the step that switches the command over is a later round's
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 16).
//!  The order is the origin cleanup design's fourteen steps,
//!  and every refusal before a destroy leaves the clone unmounted with its records kept
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals").
//!  The checks sit immediately before the requests they guard,
//!  each chunk of a split included,
//!  and before the head,
//!  because a mark set between two checks is the accepted residual and a check that ran earlier
//!  proves nothing about the request it does not guard.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = completeTheRemoval({ entry, reads, unmount, clock, check, probe, ... });
//! console.log(done.lines().join('\n'));
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_removal_proof_and_plan::fresh_holder_scan::{FreshHolderScan, the_holder_lines};
use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{HeadWord, SnapshotOutcome};
use crate::worktree_clone_removal_proof_and_plan::removal_destruction::{planned_origin_of, record_the_answer, refuse_a_marked_or_replaced_origin, the_remaining_snapshots};
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{RemovalClock, RemovalEntry};
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::{
    CloneAbsence, DatasetDestroyDevice, HeadDestroyRequest, HeadDestroyed, amend_the_plan, destroy_the_head, push_a_line, settle_the_recorded_names, the_new_own_snapshots,
};
use crate::worktree_clone_removal_proof_and_plan::removal_records_end::{RecordsGone, remove_the_removal_records};
use crate::worktree_clone_removal_proof_and_plan::removal_origin_step::{attempt_the_origin_cleanup, publish_the_origin_record};
use crate::worktree_clone_removal_proof_and_plan::removal_report::{OriginCleanupDone, TheRemovalDone};
use crate::worktree_clone_removal_proof_and_plan::removal_settle::{RegistrationSettle, SettleAnswer};
use crate::worktree_clone_removal_proof_and_plan::removal_settle_step::settle_the_registration;
use crate::worktree_clone_removal_proof_and_plan::removal_resume::{TheStart, start_the_removal};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::RemovalUnmount;
use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::snapshot_batch_outcome::BatchOutcome;
use crate::worktree_clone_snapshot_batch::snapshot_batch_plan::plan_requests;
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::{RequestResult, SnapshotDestroyDevice, send_request};
use std::path::PathBuf;

/// What:
///  Everything one whole removal drives,
///  as named seams so a control answers each without a pool,
/// a namespace or a repository.
/// Why:
///  The batch reads and writes through one device,
///  the head through another,
///  the holder scan through a third,
///  and the settle through a fourth,
///  so a control can make any one of them refuse and leave the rest real.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Completion = { entry; reads; unmount; clock; check; probe; snapshots; head; holders; settle; budget; now };
/// ```
pub struct RemovalCompletion<'a> {
    /// The repository and slot this removal is about.
    pub entry: &'a RemovalEntry<'a>,
    /// The pool reads the proofs make.
    pub reads: &'a dyn crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads,
    /// The request that takes the clone's mount down.
    pub unmount: &'a dyn RemovalUnmount,
    /// The process facts a new plan or an amendment records.
    pub clock: &'a dyn RemovalClock,
    /// The reader the deferred-origin checks read through.
    pub check: &'a dyn DatasetReader,
    /// The reader the absence proofs and the live intent read through.
    pub probe: &'a dyn DatasetReader,
    /// The device the snapshot requests go to.
    pub snapshots: &'a dyn SnapshotDestroyDevice,
    /// The device the head destroy goes to.
    pub head: &'a dyn DatasetDestroyDevice,
    /// The fresh all-namespace scan each destroy is preceded by.
    pub holders: &'a dyn FreshHolderScan,
    /// The Git settle.
    pub settle: &'a dyn RegistrationSettle,
    /// The bytes one snapshot request may carry.
    pub budget: RequestBudget,
    /// The moment this run's records and attempts carry.
    pub now: &'a str,
}

/// What:
///  One whole removal,
///  from the plan that stands or the one this run writes to the report.
/// Why:
///  The report is the removal's only trace beyond its records,
///  and the value keeps what was destroyed beside what remains,
///  so no caller can print one without the other.
///
/// # Throws
/// A refusal at any step,
/// each naming what it left and the way forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = completeTheRemoval(input);
/// ```
pub fn complete_the_removal(input: &RemovalCompletion<'_>) -> Result<TheRemovalDone, RemovalStop> {
    let started: TheStart = start_the_removal(input.entry, input.reads, input.unmount, input.clock, input.probe)?;
    let folder: PlanFolder = PlanFolder::open_or_create(input.entry.common_dir).map_err(|refusal| {
        return RemovalStop::PlanUnreadable { path: plans_folder(input.entry.common_dir), refusal };
    })?;
    return match started {
        TheStart::Gone { plan, plan_path, lost, pool } => the_head_gone_path(input, folder, plan, plan_path, lost, pool),
        TheStart::Present { entered } => {
            let lost: NamesToBeLost = entered.lost.clone();
            the_head_present_path(input, folder, entered, lost)
        }
    };
}

/// The path where the clone stands:
/// the origin record,
/// the snapshot requests,
/// the head destroy,
/// the origin attempt,
/// the settle and the records.
fn the_head_present_path(input: &RemovalCompletion<'_>, folder: PlanFolder, entered: crate::worktree_clone_removal_proof_and_plan::removal_entry::EnteredRemoval, lost: NamesToBeLost) -> Result<TheRemovalDone, RemovalStop> {
    let planned = planned_origin_of(input.entry.slot, &entered);
    if entered.plan.head == HeadWord::Gone {
        return Err(RemovalStop::ThePlanContradictsThePool { head: String::from(HeadWord::Gone.word()), clone: entered.plan.facts.clone_dataset.clone() });
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
    send_the_remaining(input, &folder, &mut plan, &planned, plan_path.as_path(), pool.as_str(), &mut requests, &mut residual)?;
    let after_the_batch: Vec<crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject> = the_live_snapshots(input, &plan)?;
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
        send_the_remaining(input, &folder, &mut plan, &planned, plan_path.as_path(), pool.as_str(), &mut requests, &mut residual)?;
        let after_the_amendment: Vec<crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject> = the_live_snapshots(input, &plan)?;
        settle_the_recorded_names(&mut plan, after_the_amendment.as_slice())?;
        publish_the_plan(&folder, &plan)?;
    }
    let remaining: Vec<String> = the_names_not_destroyed(&plan);
    if !remaining.is_empty() {
        return Err(RemovalStop::TheSnapshotsRemain { names: the_states_of(&plan, remaining.as_slice()) });
    }
    residual = residual.max(refuse_the_holders(input, &plan)?);
    refuse_a_marked_or_replaced_origin(input.check, &planned, &plan)?;
    let head: HeadDestroyed = destroy_the_head(&mut HeadDestroyRequest { plan: &mut plan, folder: &folder, probe: input.probe, device: input.head })?;
    let origin: OriginCleanupDone = attempt_the_origin_cleanup(input, &plan)?;
    let settle: SettleAnswer = settle_the_registration(input, &plan)?;
    let records: RecordsGone = remove_the_removal_records(input.entry.common_dir, input.entry.slot, &plan)?;
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
    let live: Vec<crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject> = the_live_snapshots(input, plan)?;
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
    let requested = plan_requests(the_pool.as_str(), remaining.as_slice(), input.budget).map_err(|refusal| {
        return RemovalStop::TheSnapshotSetIsNotSendable { said: refusal.to_string(), sent: requests.len() };
    })?;
    for chunk in requested.chunks() {
        *residual = (*residual).max(refuse_the_holders(input, plan)?);
        refuse_a_marked_or_replaced_origin(input.check, planned, plan)?;
        let answer: RequestResult = send_request(input.snapshots, the_pool.as_str(), chunk.as_slice()).map_err(|refusal| {
            return RemovalStop::TheSnapshotSetIsNotSendable { said: refusal.to_string(), sent: requests.len() };
        })?;
        let RequestResult::Answered(outcome) = answer else { continue };
        record_the_answer(plan, plan_path, &outcome)?;
        publish_the_plan(folder, plan)?;
        requests.push(outcome);
    }
    return Ok(());
}

/// The head-gone path:
/// the absent clone proved again,
/// every name settled by that proof,
/// the origin attempt,
/// the settle,
/// and the records.
fn the_head_gone_path(input: &RemovalCompletion<'_>, folder: PlanFolder, standing: DestructionPlan, plan_path: PathBuf, lost: NamesToBeLost, _pool: String) -> Result<TheRemovalDone, RemovalStop> {
    let mut plan: DestructionPlan = standing;
    match crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::the_clone_absence(input.probe, plan.facts.clone_dataset.as_str(), plan.facts.clone_guid)? {
        CloneAbsence::Gone => {}
        CloneAbsence::There { name, guid } => {
            return Err(RemovalStop::TheCloneNameWasReused { name, recorded_guid: plan.facts.clone_guid.file_name(), live_guid: guid.file_name() });
        }
        CloneAbsence::Reused { guid } => {
            return Err(RemovalStop::TheCloneNameWasReused { name: plan.facts.clone_dataset.clone(), recorded_guid: plan.facts.clone_guid.file_name(), live_guid: guid.file_name() });
        }
    }
    for snapshot in plan.named_snapshots() {
        if plan.current_state(snapshot.name.as_str()).map(|held| return held.outcome == SnapshotOutcome::Destroyed).unwrap_or(false) {
            continue;
        }
        push_a_line(&mut plan, &snapshot, SnapshotOutcome::Destroyed, None)?;
    }
    plan.head = HeadWord::Gone;
    publish_the_plan(&folder, &plan)?;
    let origin: OriginCleanupDone = attempt_the_origin_cleanup(input, &plan)?;
    let settle: SettleAnswer = settle_the_registration(input, &plan)?;
    let records: RecordsGone = remove_the_removal_records(input.entry.common_dir, input.entry.slot, &plan)?;
    return Ok(TheRemovalDone {
        slot: String::from(input.entry.slot),
        dataset: plan.facts.clone_dataset.clone(),
        destroyed: the_names_destroyed(&plan),
        bookmarks: lost.bookmarks.clone(),
        remaining: Vec::new(),
        head: HeadWord::Gone,
        holder_reads_unfinished: 0,
        origin,
        settle,
        records,
        plan_path,
        requests: Vec::new(),
    });
}

/// Every live snapshot of the clone,
/// which is the inventory the absence rule and the amendment read.
fn the_live_snapshots(input: &RemovalCompletion<'_>, plan: &DestructionPlan) -> Result<Vec<crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject>, RemovalStop> {
    return input.reads.snapshots_of(plan.facts.clone_dataset.as_str(), plan.facts.clone_guid).map_err(|reason| {
        return RemovalStop::InventoryUnreadable { clone: plan.facts.clone_dataset.clone(), reason };
    });
}

/// Every name of the plan whose current state is not `destroyed`,
/// sorted.
pub fn the_names_not_destroyed(plan: &DestructionPlan) -> Vec<String> {
    let mut names: Vec<String> = plan
        .named_snapshots()
        .iter()
        .filter(|snapshot| return plan.current_state(snapshot.name.as_str()).map(|held| return held.outcome != SnapshotOutcome::Destroyed).unwrap_or(true))
        .map(|snapshot| return snapshot.name.clone())
        .collect();
    names.sort();
    return names;
}

/// Every name of the plan whose current state is `destroyed`.
fn the_names_destroyed(plan: &DestructionPlan) -> Vec<String> {
    return plan
        .named_snapshots()
        .iter()
        .filter(|snapshot| return plan.current_state(snapshot.name.as_str()).map(|held| return held.outcome == SnapshotOutcome::Destroyed).unwrap_or(false))
        .map(|snapshot| return snapshot.name.clone())
        .collect();
}

/// The names with the state their history ends on,
/// which is what a stop prints.
pub fn the_states_of(plan: &DestructionPlan, names: &[String]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for name in names {
        let state: &str = plan.current_state(name.as_str()).map_or("planned", |held| return held.outcome.word());
        lines.push(format!("{name} ({state})"));
    }
    return lines;
}

/// What:
///  The snapshot the removal's own enumeration still finds,
///  as the batch's fresh enumeration did,
///  which the report and the stop read.
/// Why:
///  The design enumerates again after the batch and before the head,
///  and this is that read's own list,
///  which differs from the plan's names by a snapshot that appeared after the first write.
///
/// # Throws
/// The listing that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remaining = theRemainingSnapshots(reads, plan);
/// ```
pub fn the_remaining(input: &RemovalCompletion<'_>, plan: &DestructionPlan) -> Result<Vec<String>, RemovalStop> {
    return the_remaining_snapshots(input.reads, plan);
}

fn publish_the_plan(folder: &PlanFolder, plan: &DestructionPlan) -> Result<(), RemovalStop> {
    return match folder.publish(plan) {
        Ok(_path) => Ok(()),
        Err(refusal) => Err(RemovalStop::ThePlanCouldNotHoldTheAnswer { path: folder.path().join(plan.facts.clone_guid.file_name()), said: refusal.message(folder.path()) }),
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
fn refuse_the_holders(input: &RemovalCompletion<'_>, plan: &DestructionPlan) -> Result<usize, RemovalStop> {
    let scan = input.holders.holders_of(plan.facts.clone_dataset.as_str(), plan.facts.clone_guid)?;
    let lines: Vec<String> = the_holder_lines(&scan);
    if !lines.is_empty() {
        return Err(RemovalStop::HoldersSeeTheClone { holders: lines, incomplete: scan.incomplete.len() });
    }
    return Ok(scan.incomplete.len());
}

/// One live object's kind used by the report,
/// so a bookmark and a snapshot never read alike.
pub const BOOKMARK_KIND: EntryKind = EntryKind::Bookmark;

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_completion_tests.rs"]
mod tests;
