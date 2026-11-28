//! What:
//!  The three values one removal's entry derives from what it read:
//!  the loss a standing plan records,
//!  the plan fields the first write carries,
//!  and the answer to a plan that names another clone.
//! Why:
//!  The entry's order is long and its own doc says what the order is,
//!  so the derivations it makes from a plan live beside it rather than inside
//!  it.
//!  Each one is a fact about records and the pool and no step of the order,
//!  so moving them changes no decision:
//!  the entry still decides when each is asked,
//!  and `removal_entry.rs` asks all three.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = factsOf(recorded, proven, proof, registration, clock);
//! const history = historyOf(plan);
//! refuseAPlanOfAnotherClone(plan, recorded);
//! ```

use crate::subvolume::fs_beneath::ObjectId;
use crate::worktree_clone_removal_proof_and_plan::lineage_proof::{
    ProvenRemoval,
    prove_the_live_lineage,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanFacts,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use crate::worktree_clone_removal_proof_and_plan::removal_clock::RemovalClock;
use crate::worktree_clone_removal_proof_and_plan::removal_force::RemovalForce;
use crate::worktree_clone_removal_proof_and_plan::removal_forced_acts::{
    ForcedActs,
    forced_past_a_proof,
};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::UnmountProof;
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;

/// What:
///  The loss a standing plan already records.
/// Why:
///  A refusal's sentence has to say what this clone has already lost,
///  and only the plan knows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const history = RemovalHistory.ofDestroyed(plan.destroyedNames());
/// ```
pub(crate) fn history_of(plan: Option<&DestructionPlan>) -> RemovalHistory {
    let Some(found) = plan else {
        return RemovalHistory::NothingDestroyed;
    };
    let mut names: Vec<String> = Vec::new();
    for line in &found.attempts {
        if line.outcome == SnapshotOutcome::Destroyed && !names.contains(&line.snapshot.name) {
            names.push(line.snapshot.name.clone());
        }
    }
    return RemovalHistory::of_destroyed(names);
}

/// Whether one standing plan is about this clone and origin.
///
/// # Throws
/// The field in which the plan and the add's record disagree.
pub(crate) fn refuse_a_plan_of_another_clone(
    plan: &DestructionPlan,
    recorded: &RecordedBinding,
) -> Result<(), RemovalStop> {
    let pairs: [(&'static str, String, String); 6] = [
        (
            "cloneDataset",
            plan.facts.clone_dataset.clone(),
            recorded.clone_dataset.clone(),
        ),
        (
            "cloneGuid",
            plan.facts.clone_guid.file_name(),
            recorded.clone_guid.file_name(),
        ),
        (
            "originDataset",
            plan.facts.origin_dataset.clone(),
            recorded.origin_dataset.clone(),
        ),
        (
            "originDatasetGuid",
            plan.facts.origin_dataset_guid.file_name(),
            recorded.origin_dataset_guid.file_name(),
        ),
        (
            "originSnapshot",
            plan.facts.origin_snapshot.clone(),
            recorded.origin_snapshot.clone(),
        ),
        (
            "originSnapshotGuid",
            plan.facts.origin_snapshot_guid.file_name(),
            recorded.origin_snapshot_guid.file_name(),
        ),
    ];
    for (field, plan_value, recorded_value) in pairs {
        if plan_value != recorded_value {
            return Err(RemovalStop::IdentityDiffers {
                object: String::from("the standing destruction plan"),
                field,
                recorded: recorded_value,
                live: plan_value,
            });
        }
    }
    return Ok(());
}

/// What:
///  The plan's own fields,
///  from the identities the proof matched,
///  the answer it verified and the add's record.
/// Why:
///  The plan copies the identities rather than referring to them,
///  because it must stand alone for everything it destroys.
///
/// # Throws
/// A clock fact this process cannot give,
/// which is the only field that is not a record's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = factsOf(recorded, proven, proof, registration, clock);
/// ```
pub(crate) fn facts_of(
    recorded: &RecordedBinding,
    proven: &ProvenRemoval,
    proof: &UnmountProof,
    registration: ObjectId,
    clock: &dyn RemovalClock,
) -> Result<PlanFacts, RemovalStop> {
    return Ok(PlanFacts {
        run: clock.run()?,
        owner: clock.owner()?,
        started: clock.started(),
        clone_dataset: recorded.clone_dataset.clone(),
        clone_guid: proven.identities.clone.guid,
        clone_createtxg: proven.identities.clone.createtxg,
        origin_dataset: recorded.origin_dataset.clone(),
        origin_dataset_guid: proven.identities.origin_dataset.guid,
        origin_snapshot: recorded.origin_snapshot.clone(),
        origin_snapshot_guid: proven.identities.origin_snapshot.guid,
        origin_snapshot_createtxg: proven.identities.origin_snapshot.createtxg,
        registration_id: recorded.registration_id.clone(),
        admin: registration.encode(),
        target: recorded.slot_path.clone(),
        unmount_nonce: proof.nonce.clone(),
        unmount_mount_id: proof.mount_id,
    });
}

/// What:
///  What the proof's force is asked over:
///  the slot's name,
///  the pool,
///  the count,
///  the identity refusal and the add's record.
/// Why:
///  The refusal names the slot and the record's GUID,
///  and two forces replace the proof with the pool's own answer,
///  so those are the values the question needs and nothing of the entry's own
///  order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { slot, reads, force, recorded, stop };
/// ```
pub(crate) struct TheProofsForce<'a> {
    /// The slot whose clone the proof is about.
    pub(super) slot: &'a str,
    /// The pool the live proof reads.
    pub(super) reads: &'a dyn RemovalPoolReads,
    /// How many forces the command carried.
    pub(super) force: RemovalForce,
    /// The add's record,
    /// which pairs with the pool it answers.
    pub(super) recorded: &'a RecordedBinding,
    /// The refusal the proof made.
    pub(super) stop: RemovalStop,
}

/// What:
///  The proof a removal goes on with when the pool did not prove the clone.
/// Why:
///  Two forces pass a clone the wrapper cannot prove,
///  and the owner answered on 2026-11-28 that such a removal
///  "destroys the dataset the slot's record names even when the pool's answer
///  does not match the record"
///  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`).
///  One force or none refuses,
///  and the sentence names two forces as the way through.
///  A refusal that is not an identity a force passes is unchanged:
///  a pool that does not answer,
///  a listing that could not be read,
///  and the transferred history or the origin-side object a proof of a clone
///  that stands does refuse.
///
/// # Throws
/// The stop the proof made,
/// or the stop that names the two forces a person would need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proven = theProofWithForce(request, recorded, stop);
/// ```
pub(crate) fn the_proof_with_force(
    request: &TheProofsForce<'_>,
    forced: &mut ForcedActs,
) -> Result<ProvenRemoval, RemovalStop> {
    let recorded: &RecordedBinding = request.recorded;
    let stop: RemovalStop = request.stop.clone();
    if !a_forced_removal_passes(&stop) {
        return Err(stop);
    }
    if !request.force.is_twice() {
        return Err(RemovalStop::TheCloneCannotBeProved {
            slot: String::from(request.slot),
            cause: the_disagreement_in(&stop),
            recorded_guid: recorded.clone_guid.file_name(),
            answered_guid: the_pools_answer_in(&stop),
        });
    }
    let proven: ProvenRemoval = prove_the_live_lineage(request.reads, recorded)?;
    // Both GUIDs are kept, because the report is the only trace of a destroy nothing proved.
    let acts: ForcedActs = forced_past_a_proof(
        recorded.clone_dataset.as_str(),
        recorded.clone_guid.file_name().as_str(),
        proven.identities.clone.guid.file_name().as_str(),
    );
    forced.lines.extend(acts.lines);
    forced.recorded_guid = acts.recorded_guid;
    forced.answered_guid = acts.answered_guid;
    return Ok(proven);
}

/// What:
///  Whether a force passes the refusal a proof made.
/// Why:
///  Only a proof that found another object than the record names is what two
///  forces go past,
///  and the record's risk is exactly that case:
///  a GUID that differs or a name that was reused.
///  A pool that cannot answer is no such state,
///  and nothing about this route's other refusals is decided here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const passes = (stop) => stop.kind === 'identity-differs' || stop.kind === 'origin-is-not-the-recorded-one';
/// ```
fn a_forced_removal_passes(stop: &RemovalStop) -> bool {
    return matches!(
        stop,
        RemovalStop::IdentityDiffers { .. } | RemovalStop::OriginIsNotTheRecordedOne { .. }
    );
}

/// What:
///  The disagreement one proof made,
///  as the clause the refusal that names two forces prints.
/// Why:
///  The refusal's own sentence is long and promises what nothing was done to,
///  and a clause beside "the clone cannot be proved" has to say which two values
///  disagreed rather than repeat that promise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cause = theDisagreementIn(stop);
/// ```
fn the_disagreement_in(stop: &RemovalStop) -> String {
    return match stop {
        RemovalStop::IdentityDiffers {
            object,
            field,
            recorded,
            live,
        } => format!(
            "the pool answers `{live}` for `{object}`'s `{field}`, where the add recorded `{recorded}`"
        ),
        RemovalStop::OriginIsNotTheRecordedOne {
            clone,
            recorded,
            live,
        } => format!(
            "the clone `{clone}` has origin `{}`, where the add recorded `{recorded}`",
            live.as_deref().unwrap_or("none")
        ),
        other => String::from(other.message()),
    };
}

/// The GUID the pool answered in an identity refusal,
/// when the refusal compared the clone's own GUID.
fn the_pools_answer_in(stop: &RemovalStop) -> Option<String> {
    return match stop {
        RemovalStop::IdentityDiffers {
            field: "cloneGuid",
            live,
            ..
        } => Some(live.clone()),
        _ => None,
    };
}
