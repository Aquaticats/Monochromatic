//! What:
//!  The Git settle of one removal,
//!  called only with the proof that this slot's clone is gone.
//! Why:
//!  The settle removes a registration,
//!  a pointer and a folder the next add reuses,
//!  so no caller may reach it on a guess
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Removal and recovery",
//!  step 5).
//!  The absence is proved here again rather than taken from the caller's own word,
//!  and the live settle itself refuses a registration of another identity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settled = settleTheRegistration(input, plan);
//! ```

use crate::worktree_clone_managed_lock::clone_destruction_proof::{
    CloneDestructionProof,
    TheLiveCloneAnswer,
    prove_the_recorded_clone_is_gone,
};
use crate::worktree_clone_managed_lock::recorded_lock_facts::{
    RecordedClone,
    read_the_recorded_facts,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_inputs::RemovalCompletion;
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::CloneAbsence;
use crate::worktree_clone_removal_proof_and_plan::removal_settle::{
    SettleAnswer,
    SettleRequest,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;

/// What:
///  The Git settle,
///  which runs only with a proof that this slot's clone is gone.
/// Why:
///  The settle removes a registration,
///  a pointer and a folder the next add reuses,
///  so it is the one removal-side step that may not be retried on a guess twice.
///
/// # Throws
/// A pool that cannot prove the absence,
/// or the settle's own refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settled = settleTheRegistration(input, plan);
/// ```
pub(crate) fn settle_the_registration(
    input: &RemovalCompletion<'_>,
    plan: &DestructionPlan,
) -> Result<SettleAnswer, RemovalStop> {
    match crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::the_clone_absence(
        input.probe,
        plan.facts.clone_dataset.as_str(),
        plan.facts.clone_guid,
    )? {
        CloneAbsence::Gone => {}
        CloneAbsence::There {
            name,
            ..
        } => {
            return Err(RemovalStop::TheSettleRefused {
                step: String::from("the clone's absence"),
                said: format!("`{name}` still answers"),
            });
        }
        CloneAbsence::Reused {
            guid,
        } => {
            return Err(RemovalStop::TheSettleRefused {
                step: String::from("the clone's absence"),
                said: format!("the name answers GUID `{}`", guid.file_name()),
            });
        }
    }
    let record =
        read_the_recorded_facts(input.entry.common_dir, input.entry.slot).map_err(|refusal| {
            return RemovalStop::TheSettleRefused {
                step: String::from("the binding's lock facts"),
                said: format!("{refusal:?}"),
            };
        })?;
    let Some(_registration) = record.registration.clone() else {
        return Ok(SettleAnswer {
            wrote_the_lost_lock: false,
            released_the_lock: false,
            removed_the_registration: false,
            removed_the_underlay: false,
            lines: Vec::new(),
        });
    };
    let clone: RecordedClone = RecordedClone {
        dataset: plan.facts.clone_dataset.clone(),
        guid: Some(plan.facts.clone_guid),
    };
    let proof: CloneDestructionProof =
        prove_the_recorded_clone_is_gone(input.entry.slot, &clone, &TheLiveCloneAnswer::Absent)
            .map_err(|refusal| {
                return RemovalStop::TheSettleRefused {
                    step: String::from("the destruction proof"),
                    said: refusal.words(),
                };
            })?;
    return input.settle.settle_the_registration(&SettleRequest {
        common_dir: input.entry.common_dir,
        slot: input.entry.slot,
        slot_path: plan.facts.target.as_path(),
        registration_id: plan.facts.registration_id.as_slice(),
        admin: plan.facts.admin.as_str(),
        slot_identity: record.slot_identity,
        proof: &proof,
    });
}

/// One plan write through the folder that publishes it whole.
/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_settle_step_tests.rs"]
mod tests;
