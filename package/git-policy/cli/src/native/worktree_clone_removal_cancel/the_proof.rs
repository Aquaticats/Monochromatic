//! What:
//!  Which slot,
//!  which clone,
//!  and which records the command proves,
//!  in the design's own order:
//!  the slot's pointer,
//!  the add's binding,
//!  the plan,
//!  the clone's own request,
//!  at least one removal record,
//!  the clone's live identity and origin,
//!  and the snapshots the plan's tuples settle against.
//! Why:
//!  The command acts only on what it can prove,
//!  and a person who typed another slot's name must not be told a removal was reversed
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The proof" section).
//!  The design proves the records first and repeats every deciding read under the claim,
//!  so the slot that is not being removed is refused before any claim is taken,
//!  and no record that changed while the claim was taken is ever acted on.
//!  A pointer to another checkout is the caller's mistake;
//!  a damaged pointer,
//!  a missing or mismatched binding,
//!  a plan of another clone,
//!  a clone whose identity or origin differs,
//!  a request that is no local one,
//!  a plan whose owner still runs,
//!  and a snapshot list that cannot be read are states that prevent the command.
//!  Nothing here writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const guid = proveTheSlotIsARemovals(input);
//! const proved = proveTheSlotsRecords(input);
//! console.log(proved.destroyed);
//! ```

/// What the proof reads,
/// and everything one proof establishes.
#[path = "the_proof_inputs.rs"]
mod the_proof_inputs;
/// The records,
/// the plan,
/// the request and the clone's identity,
/// read and refused.
#[path = "the_proof_reads.rs"]
mod the_proof_reads;

/// What the proof reads and everything it establishes.
pub use self::the_proof_inputs::{
    TheRecordsProof,
    WORKTREES_FOLDER,
    WhatTheRecordsProve,
};
/// The reads the proof makes,
/// each with the refusal it owes.
use self::the_proof_reads::{
    live_identity,
    read_the_plan,
    read_the_records,
    read_the_request,
};
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountIntent;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::the_destroy_outcomes::settle_the_destroy_outcomes;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use std::path::PathBuf;

/// What:
///  Prove the records and the clone's own request,
///  without reading the clone's snapshot inventory.
/// Why:
///  A mistyped slot or a slot that is not being removed must be refused before a claim is
///  created,
///  because the claim is what makes the reads one moment and a caller's mistake must leave it
///  untouched.
///
/// # Errors
/// A damaged pointer,
/// a pointer to another checkout,
/// a binding that is absent or of another slot,
/// a plan that cannot be read or names another clone,
/// a clone that does not answer,
/// a request that is no local one,
/// a slot with no removal record,
/// or a plan whose owner still runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guid = proveTheSlotIsARemovals(input);
/// ```
pub fn prove_the_slot_is_a_removals(
    input: &TheRecordsProof<'_>
) -> Result<DatasetGuid, CancelRefusal> {
    let recorded: RecordedBinding = read_the_records(input)?;
    let plan: Option<DestructionPlan> = read_the_plan(input, &recorded)?;
    let _intent: MountIntent = read_the_request(input, &recorded, plan.is_some())?;
    return Ok(recorded.clone_guid);
}

/// What:
///  Prove the records and the live clone together,
///  and settle what the plan's tuples prove destroyed.
/// Why:
///  The act may follow only this proof:
///  the records say which clone,
///  the pool says the clone is the recorded one and names its own origin,
///  and the inventory settles every tuple,
///  so a clone of another identity and an outcome nobody can prove are both refused before
///  anything moves.
///
/// # Errors
/// Every refusal the records' proof makes,
/// an origin that is not the recorded one,
/// an inventory that cannot be read,
/// or a tuple whose outcome cannot be settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveTheSlotsRecords(input);
/// ```
pub fn prove_the_slots_records(
    input: &TheRecordsProof<'_>
) -> Result<WhatTheRecordsProve, CancelRefusal> {
    let recorded: RecordedBinding = read_the_records(input)?;
    let plan: Option<DestructionPlan> = read_the_plan(input, &recorded)?;
    let intent: MountIntent = read_the_request(input, &recorded, plan.is_some())?;
    let identity = live_identity(input, &recorded)?;
    let live_origin: Option<String> =
        Some(identity.stats.origin()).filter(|origin| return !origin.is_empty());
    if live_origin.as_deref() != Some(recorded.origin_snapshot.as_str()) {
        return Err(CancelRefusal::TheCloneIsNotTheRecordedOne {
            field: "originSnapshot",
            recorded: recorded.origin_snapshot.clone(),
            live: live_origin.unwrap_or_else(|| return String::from("none")),
        });
    }
    let live: Vec<crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject> =
        input
            .reads
            .snapshots_of(recorded.clone_dataset.as_str(), recorded.clone_guid)
            .map_err(|reason| {
                return CancelRefusal::TheInventoryIsUnreadable {
                    clone: recorded.clone_dataset.clone(),
                    reason,
                };
            })?;
    let destroyed: Vec<String> = match &plan {
        Some(standing) => settle_the_destroy_outcomes(standing, live.as_slice())?,
        None => Vec::new(),
    };
    let slot_path: PathBuf = input.checkout.join(WORKTREES_FOLDER).join(input.slot);
    return Ok(WhatTheRecordsProve {
        recorded,
        slot_path,
        plan,
        intent,
        destroyed,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_proof_tests.rs"]
mod tests;
