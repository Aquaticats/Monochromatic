//! What:
//!  What a removal does after its plan:
//!  the deferred-origin check before every snapshot request,
//!  the one batch that destroys the clone's own snapshots,
//!  the fresh enumeration of the clone,
//!  the check again before the head destroy,
//!  and the stop that leaves the head destroy to plan step 16.
//! Why:
//!  A marked origin snapshot is destroyed by ZFS itself when its last clone goes,
//!  so the removal checks that the clone and its origin are the objects it planned and
//!  that the origin is not marked,
//!  and refuses otherwise
//!  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`;
//!  `doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "One check,
//! called at every snapshot request and before the head destroy").
//!  The design places one call before every snapshot request,
//!  each chunk of a split included,
//!  and one after the batch's fresh enumeration and before the head request
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery",
//! steps 3 to 6).
//!  The look before the unmount is not here:
//!  it sits inside the lineage proof
//!  (`worktree_clone_removal_proof_and_plan::lineage_proof`).
//!  This release sends no head destroy,
//!  so this file stops after the head check and says so in its value:
//!  everything plan step 16 adds after that point
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 16)
//!  is what makes a removal finish.
//!  A release command reaches this file through
//!  `removal_completion::complete_the_removal`,
//!  whose `destroy_the_removal` is the entry the whole order keeps for a caller
//!  that wants only the destruction half.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const underway = destroyTheRemoval({ entry, reads, unmount, clock, check, device, budget });
//! console.log(underway.lost.text());
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanAttempt,
    PlanSnapshot,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use crate::worktree_clone_removal_proof_and_plan::removal_clock::RemovalClock;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{
    EnteredRemoval,
    RemovalEntry,
    enter_the_removal,
};
use crate::worktree_clone_removal_proof_and_plan::removal_entry_facts::history_of;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_removal_proof_and_plan::removal_staging_folder::RemovalStagingFolder;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::RemovalUnmount;
use crate::worktree_clone_snapshot_batch::request_budget::RequestBudget;
use crate::worktree_clone_snapshot_batch::snapshot_batch_outcome::BatchOutcome;
use crate::worktree_clone_snapshot_batch::snapshot_batch_plan::{
    RequestPlan,
    plan_requests,
};
use crate::worktree_clone_snapshot_batch::snapshot_batch_send::{
    RequestResult,
    SnapshotDestroyDevice,
    send_request,
};
use crate::worktree_clone_snapshot_batch::snapshot_name_outcome::NameOutcome;
use crate::worktree_removal_deferred_origin_check::origin_verdict::{
    Verdict,
    check_planned_origin,
};
use crate::worktree_removal_deferred_origin_check::planned_origin::PlannedOrigin;
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Everything one removal's destruction step reads and drives:
///  the entry's own context and seams,
///  the check's reader,
///  the request's device,
///  and the budget the set is split by.
/// Why:
///  The checks judge the pool through the reader the mount handler already answers,
///  the batch sends through the binding's own request,
///  and a control answers both without opening this machine's device,
///  so each seam is one named input rather than a parameter list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Input = { entry; reads; unmount; clock; check; device; budget };
/// ```
pub struct RemovalDestruction<'a> {
    /// The repository and slot this removal is about.
    pub entry: &'a RemovalEntry<'a>,
    /// The pool reads the entry's proof makes.
    pub reads: &'a dyn RemovalPoolReads,
    /// The request that takes the clone down.
    pub unmount: &'a dyn RemovalUnmount,
    /// The folder that request's handler binds,
    /// which the wrapper makes before it asks.
    pub staging: &'a dyn RemovalStagingFolder,
    /// The process facts the plan's header carries.
    pub clock: &'a dyn RemovalClock,
    /// The reader the deferred-origin checks read through.
    pub check: &'a dyn DatasetReader,
    /// The device one snapshot request is sent to.
    pub device: &'a dyn SnapshotDestroyDevice,
    /// The bytes one request's encoded input may carry.
    pub budget: RequestBudget,
}

/// What:
///  What one removal holds after it has done everything this release does:
///  the plan with the batch's outcomes,
///  the names it printed,
///  the kernel's answer per request,
///  and the clone's own snapshots that are still there.
/// Why:
///  The head destroy is plan step 16's,
///  so the value names what step 16 continues from rather than claiming a removal finished:
///  the clone stands with its own snapshots either gone or named here,
///  and the plan records every answer this run received.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Underway = { plan; planPath; lost; requests; remainingSnapshots };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DestructionUnderway {
    /// The plan as this run last wrote it,
    /// with one line per answer it received.
    pub plan: DestructionPlan,
    /// The file the plan was last published at.
    pub plan_path: PathBuf,
    /// The names this removal printed as lost.
    pub lost: NamesToBeLost,
    /// Every request this run sent,
    /// with the kernel's per-name answer kept whole.
    pub requests: Vec<BatchOutcome>,
    /// The clone's own snapshots the fresh enumeration still found,
    /// each by full name.
    pub remaining_snapshots: Vec<String>,
}

/// What:
///  Enter the removal and then destroy what it planned:
///  check,
///  send a chunk,
///  record its answer,
///  repeat for every chunk,
///  enumerate the clone again,
///  and check once more before the head destroy.
/// Why:
///  The check sits immediately before the request it guards,
///  and the history sentence a refusal prints comes from the plan's own outcomes rather than
///  from which call site refused,
///  so a refusal at the second chunk names what the first one destroyed
///  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
///  "The refusals" and "What a refusal leaves,
/// and what the record keeps").
///  A refusal anywhere here keeps the intent,
/// the plan and the registration,
///  and leaves the clone unmounted,
///  which is what the project decided for the checks
///  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`).
///
/// # Throws
/// The entry's own refusal,
/// a check's refusal,
/// a set the route will not submit,
/// or a plan that cannot hold an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const underway = destroyTheRemoval(input);
/// ```
pub fn destroy_the_removal(
    input: &RemovalDestruction<'_>
) -> Result<DestructionUnderway, RemovalStop> {
    let entered: EnteredRemoval =
        enter_the_removal(
            input.entry,
            input.reads,
            input.unmount,
            input.clock,
            input.staging,
        )?;
    let planned: PlannedOrigin = planned_origin_of(input.entry.slot, &entered);
    let folder: PlanFolder =
        PlanFolder::open_or_create(input.entry.common_dir).map_err(|refusal| {
            return RemovalStop::PlanUnreadable {
                slot: String::from(input.entry.slot),
                path: crate::worktree_clone_binding_record::registration_home::plans_folder(
                    input.entry.common_dir,
                ),
                refusal,
            };
        })?;
    let mut plan: DestructionPlan = entered.plan;
    let requested: RequestPlan = plan_requests(
        entered.pool.as_str(),
        the_snapshots_to_send(&plan).as_slice(),
        input.budget,
    )
    .map_err(|refusal| {
        return RemovalStop::TheSnapshotSetIsNotSendable {
            said: refusal.to_string(),
            sent: 0,
        };
    })?;
    let mut requests: Vec<BatchOutcome> = Vec::new();
    for chunk in requested.chunks() {
        refuse_a_marked_or_replaced_origin(input.check, &planned, &plan)?;
        let answer: RequestResult =
            send_request(input.device, entered.pool.as_str(), chunk.as_slice()).map_err(
                |refusal| {
                    return RemovalStop::TheSnapshotSetIsNotSendable {
                        said: refusal.to_string(),
                        sent: requests.len(),
                    };
                },
            )?;
        let RequestResult::Answered(outcome) = answer else {
            // A planned chunk is never empty, and `send_request` skips an empty set itself, so this arm
            // is the route's own rule rather than a state a caller can reach.
            continue;
        };
        record_the_answer(&mut plan, entered.plan_path.as_path(), &outcome)?;
        folder.publish(&plan).map_err(|refusal| {
            return RemovalStop::ThePlanCouldNotHoldTheAnswer {
                path: entered.plan_path.clone(),
                said: refusal.message(folder.path()),
            };
        })?;
        requests.push(outcome);
    }
    let remaining: Vec<String> = the_remaining_snapshots(input.reads, &plan)?;
    refuse_a_marked_or_replaced_origin(input.check, &planned, &plan)?;
    return Ok(DestructionUnderway {
        plan,
        plan_path: entered.plan_path,
        lost: entered.lost,
        requests,
        remaining_snapshots: remaining,
    });
}

/// What:
///  The identities this removal planned,
///  as the plan wrote them.
/// Why:
///  The check compares the live pool with what this run proved,
///  and the plan is that record,
///  so a check that read the add's binding instead would judge against a tuple a lineage
///  change may have invalidated since
///  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
/// veto V2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const planned = plannedOriginOf(slot, entered);
/// ```
pub(super) fn planned_origin_of(
    slot: &str,
    entered: &EnteredRemoval,
) -> PlannedOrigin {
    let facts = &entered.plan.facts;
    return PlannedOrigin {
        slot: String::from(slot),
        clone_name: facts.clone_dataset.clone(),
        clone_guid: facts.clone_guid,
        origin_dataset_name: facts.origin_dataset.clone(),
        origin_dataset_guid: facts.origin_dataset_guid,
        origin_snapshot_name: facts.origin_snapshot.clone(),
        origin_snapshot_guid: facts.origin_snapshot_guid,
        origin_snapshot_createtxg: facts.origin_snapshot_createtxg,
    };
}

/// What:
///  Every snapshot this run submits:
///  the whole verified set the plan's first write recorded.
/// Why:
///  A plan whose snapshot lines are all `planned` is the only one this entry writes,
///  because it refuses a plan that already stands,
///  so this set is the plan's own snapshot list and no name here was settled before.
///  A resumed removal that adopts a kept plan is plan step 16's,
///  and it is what decides which of a kept plan's names are sent again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = plan.namedSnapshots().map(s => s.name);
/// ```
pub(super) fn the_snapshots_to_send(plan: &DestructionPlan) -> Vec<String> {
    return plan
        .named_snapshots()
        .iter()
        .map(|snapshot| return snapshot.name.clone())
        .collect();
}

/// What:
///  Name the state a check refused in,
///  with the loss the plan records so far.
/// Why:
///  A refusal at either check keeps the removal pending with its record,
///  leaves the clone unmounted,
///  and prints the plan's own history,
///  so the sentence is true at a later chunk and on a resumed removal as well as at the first
///  request
///  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`,
///  "What a refusal at the first check leaves").
///
/// # Throws
/// The check's cause,
/// with the object it named and the sentence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = checkPlannedOrigin(check, planned);
/// if (verdict.kind === 'refused') throw stop(verdict.cause, historyOf(plan));
/// ```
pub(super) fn refuse_a_marked_or_replaced_origin(
    check: &dyn DatasetReader,
    planned: &PlannedOrigin,
    plan: &DestructionPlan,
) -> Result<(), RemovalStop> {
    let history: RemovalHistory = history_of(Some(plan));
    return match check_planned_origin(check, planned) {
        Verdict::Proceed => Ok(()),
        Verdict::Refused(refusal) => Err(RemovalStop::OriginCheckRefused {
            refusal: Box::new(refusal),
            history,
        }),
    };
}

/// What:
///  Write one request's answer onto every name it submitted,
///  as the next line of that name's history.
/// Why:
///  The plan's merge rule reads a name's highest attempt,
///  so the answer is one line above the `planned` line the first write made,
///  and a name's own tuple travels with it rather than a name the answer carried.
///  An answer that names an object the plan does not hold stops the removal rather than being
///  dropped,
///  because a record that lost an answer would say less than the pool does.
///
/// # Throws
/// An answer naming a snapshot the plan does not hold,
/// an errno this record cannot carry,
/// or a line the plan's own merge rule refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const submitted of outcome.names()) plan.recordLine(line(submitted));
/// ```
pub(super) fn record_the_answer(
    plan: &mut DestructionPlan,
    path: &Path,
    outcome: &BatchOutcome,
) -> Result<(), RemovalStop> {
    let named: Vec<PlanSnapshot> = plan.named_snapshots();
    for submitted in outcome.names() {
        let Some(snapshot) = named.iter().find(|held| return held.name == submitted.name) else {
            return Err(RemovalStop::ThePlanCouldNotHoldTheAnswer {
                path: path.to_path_buf(),
                said: format!(
                    "the kernel answered about `{}`, and the plan holds no snapshot of that name",
                    submitted.name
                ),
            });
        };
        let line: PlanAttempt = attempt_line(
            snapshot,
            next_attempt(plan, snapshot.name.as_str()),
            &submitted.outcome,
            path,
        )?;
        plan.record_line(line).map_err(|refusal| {
            return RemovalStop::ThePlanCouldNotHoldTheAnswer {
                path: path.to_path_buf(),
                said: refusal.message(path),
            };
        })?;
    }
    return Ok(());
}

/// The attempt number a name's next line takes:
/// one above the highest this plan already holds for it.
pub(super) fn next_attempt(
    plan: &DestructionPlan,
    name: &str,
) -> u64 {
    return plan
        .current_state(name)
        .map_or(1, |held| return held.attempt + 1);
}

/// What:
///  One line for one name's answer,
///  in the record's own vocabulary and with the kernel's errno kept.
/// Why:
///  The five words mean what the request did,
///  and an errno is a number the record holds as an unsigned one,
///  so a number it cannot hold stops the removal rather than being wrapped into another.
///
/// # Throws
/// An errno outside the record's range,
/// which the kernel does not answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = { snapshot, attempt, outcome: word, errno };
/// ```
pub(super) fn attempt_line(
    snapshot: &PlanSnapshot,
    attempt: u64,
    outcome: &NameOutcome,
    path: &Path,
) -> Result<PlanAttempt, RemovalStop> {
    let (word, errno): (SnapshotOutcome, Option<u32>) = match outcome {
        NameOutcome::Destroyed => (SnapshotOutcome::Destroyed, None),
        NameOutcome::Intact => (SnapshotOutcome::Intact, None),
        NameOutcome::Unknown {
            ..
        } => (SnapshotOutcome::Unknown, None),
        NameOutcome::Refused {
            errno,
        } => match u32::try_from(*errno) {
            Ok(held) => (SnapshotOutcome::Refused, Some(held)),
            Err(_) => {
                return Err(RemovalStop::ThePlanCouldNotHoldTheAnswer {
                    path: path.to_path_buf(),
                    said: format!(
                        "the kernel answered errno {errno} for `{}`, which a record's line cannot carry",
                        snapshot.name
                    ),
                });
            }
        },
    };
    return Ok(PlanAttempt {
        snapshot: snapshot.clone(),
        attempt,
        outcome: word,
        errno,
    });
}

/// What:
///  The clone's own snapshots as a fresh enumeration finds them,
///  each by full name.
/// Why:
///  The design enumerates the clone again after the batch and before the head destroy,
///  and this is the read that says whether the head could go at all
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Removal and recovery",
/// step 6).
///  A listing this route could not read is a refusal rather than a short history.
///
/// # Throws
/// The listing that failed,
/// which makes the clone's own history unreadable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshots = reads.snapshotsOf(plan.facts.cloneDataset, plan.facts.cloneGuid);
/// ```
pub(super) fn the_remaining_snapshots(
    reads: &dyn RemovalPoolReads,
    plan: &DestructionPlan,
) -> Result<Vec<String>, RemovalStop> {
    let objects = reads
        .snapshots_of(plan.facts.clone_dataset.as_str(), plan.facts.clone_guid)
        .map_err(|reason| {
            return RemovalStop::InventoryUnreadable {
                clone: plan.facts.clone_dataset.clone(),
                reason,
            };
        })?;
    let mut names: Vec<String> = objects
        .iter()
        .map(|object| return object.full_name())
        .collect();
    names.sort();
    return Ok(names);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_destruction_tests.rs"]
mod tests;
