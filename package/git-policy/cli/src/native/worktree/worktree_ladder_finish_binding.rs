//! What:
//!  What one add holds once real Git returned:
//! the binding of the worktree to the directories Git made for it,
//! and the claim on the stage name,
//! which is released unless a rung took it.
//! Why:
//!  Every write the ladder makes goes to the bound worktree and is checked against the binding,
//! so a worktree that was moved or replaced meanwhile is never written,
//! and a run against a replacement is never taken for the add's.
//! A binding that cannot be proven ends the ladder for this add and leaves what Git made as it is.
//! A claim a rung took travels on to the copy,
//! so the record of an add never disappears between the end of its checkout and the copy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bound = bind({ claim, source, plan }); // throws a reason
//! releaseClaim({ claim, swapped }); // a failure is logged, never thrown
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
};
use crate::diagnostic_log::warn;
use crate::worktree_add_binding::{
    Bound,
    bind_registration,
};
use crate::worktree_command_facts::AddPlan;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_stage_claim::StageClaim;
use crate::worktree_snapshot_source::SnapshotSource;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadder";

/// What:
///  Bind the add to the directories Git made,
/// and write that binding to the add's record.
/// Why:
///   Every write below goes to the bound worktree and is checked against the binding.
/// A binding that cannot be proven ends
///  the ladder for this add,
/// leaving what Git made as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = await bind(claim, source, plan); // throws a reason
/// ```
pub(crate) fn bind(
    claim: &StageClaim,
    source: &SnapshotSource,
    plan: &AddPlan,
) -> Result<Bound, String> {
    let bind_registration_entered: bool = begin_step("bindRegistration");
    let bind_registration_outcome =
        bind_registration(source.common_dir.as_path(), plan.destination.as_path());
    end_step(bind_registration_entered);
    let bound: Bound = bind_registration_outcome?;
    let record_binding_entered: bool = begin_step("recordBinding");
    let record_binding_outcome = claim.record_binding(&bound.binding);
    end_step(record_binding_entered);
    if let Err(error) = record_binding_outcome {
        warn(TAG, format!("the binding of the add could not be recorded ({}); an interruption will be reported by path only", error.0).as_str());
    }
    return Ok(bound);
}

/// Release the claim on a stage name,
/// saying so when that fails;
/// flushing nothing when a subvolume was swapped into place.
pub(crate) fn release_claim(
    claim: StageClaim,
    swapped: bool,
) {
    let release_claim_entered: bool = begin_step("releaseClaim");
    let released: Result<(), WorktreeCopyError> = if swapped {
        claim.release_unsynced()
    } else {
        claim.release()
    };
    end_step(release_claim_entered);
    if let Err(error) = released {
        warn(
            TAG,
            format!("the stage claim could not be released ({})", error.0).as_str(),
        );
    }
}
