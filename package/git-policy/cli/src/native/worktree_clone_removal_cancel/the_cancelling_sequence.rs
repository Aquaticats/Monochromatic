//! What:
//!  The order both actors share:
//!  one `mount` request under a fresh nonce,
//!  its verified answer,
//!  the staging folder's settlement on that answer,
//!  and then the plan's one action.
//! Why:
//!  The design says the intent write is the first act after the block,
//!  because a standing `unmount` request must not be acted on again
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The arm blocked by a scan" and "The cancelling sequence's own states").
//!  The staging folder is settled on the answer that put the clone back,
//!  and the plan is the last thing touched,
//!  so a kill between any two steps leaves a state the next removal or command settles:
//!  a `mount` request asking for the state the clone is in,
//!  a folder whose own claim and record still prove it,
//!  and a plan that is still the loss record.
//!  The removal holds its claim already and the command takes it before this runs,
//!  so nothing here takes or releases a claim.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = reverseTheRemoval({ commonDir, recorded, slotPath, plan, outcomes, mountRequest, staged });
//! ```

use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::the_destroy_outcomes::TheDestroyOutcomes;
use crate::worktree_clone_removal_cancel::the_plan_it_leaves::{ThePlansFate, the_plan_a_cancellation_leaves};
use crate::worktree_clone_removal_cancel::the_reversing_request::{
    ExpectedMount, MountProof, TheMountRequest, the_reversing_intent, verify_the_mount_answer,
};
use crate::worktree_clone_removal_cancel::the_staged_state::{StagedSettlement, TheStagedFolder};
use std::path::Path;

/// What:
///  Everything one reversal drives.
/// Why:
///  The two actors differ in who is present and in what they print,
///  and not in this order,
///  so the records,
///  the settled outcomes,
///  the request and the staged state travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reversal = { commonDir; recorded; slotPath; plan?; outcomes; mountRequest; staged };
/// ```
pub struct TheReversalRequest<'a> {
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The identities the records prove.
    pub recorded: &'a RecordedBinding,
    /// The clone's slot.
    pub slot_path: &'a Path,
    /// The plan that stands,
    /// where one does.
    pub plan: Option<&'a DestructionPlan>,
    /// The destroyed set the plan's tuples settled to.
    pub outcomes: &'a TheDestroyOutcomes,
    /// The route's own request writer and wait.
    pub mount_request: &'a dyn TheMountRequest,
    /// The staging folder's settlement.
    pub staged: &'a dyn TheStagedFolder,
}

/// What:
///  What one reversal left.
/// Why:
///  The report names the request's nonce,
///  the mount the answer observed,
///  what became of the staged state and of the plan,
///  and the destroyed set that cannot come back,
///  so one value carries every fact a person acts on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Done = { nonce; mount; staged; plans; destroyed };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheReversalDone {
    /// The nonce the reversing request was filed under,
    /// which the report prints so a receipt is named.
    pub nonce: String,
    /// The mount the handler's answer observed.
    pub mount_id: u64,
    /// What became of the staged state.
    pub staged: StagedSettlement,
    /// What became of the plan.
    pub plans: ThePlansFate,
    /// Every snapshot the removal destroyed,
    /// which no cancellation restores.
    pub destroyed: Vec<String>,
}

/// What:
///  Reverse one removal a block stopped.
/// Why:
///  Every step runs in the design's order,
///  and a refusal at any of them leaves what the steps before it wrote,
///  which is a state the next command or the next removal adopts rather than resets.
///
/// # Errors
/// The request that could not be written,
/// the answer that never came or named another mount,
/// or a plan that could not be removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reverseTheRemoval(input);
/// ```
pub fn reverse_the_removal(input: &TheReversalRequest<'_>) -> Result<TheReversalDone, CancelRefusal> {
    let intent = the_reversing_intent()?;
    let answer = input.mount_request.request_the_mount(&intent)?;
    let expected: ExpectedMount = ExpectedMount {
        clone_guid: input.recorded.clone_guid,
        clone_dataset: input.recorded.clone_dataset.clone(),
        slot_path: input.slot_path.to_path_buf(),
        nonce: intent.nonce.clone(),
    };
    let proof: MountProof = verify_the_mount_answer(&answer, &expected)?;
    let clone_guid: DatasetGuid = input.recorded.clone_guid;
    let staged: StagedSettlement = input.staged.settle(clone_guid);
    let plans: ThePlansFate = the_plan_a_cancellation_leaves(input.common_dir, input.recorded, input.plan, input.outcomes)?;
    return Ok(TheReversalDone {
        nonce: intent.nonce.as_str().to_string(),
        mount_id: proof.mount.mount_id,
        staged,
        plans,
        destroyed: input.outcomes.destroyed.clone(),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_cancelling_sequence_tests.rs"]
mod tests;
