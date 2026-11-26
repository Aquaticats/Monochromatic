//! What:
//!  The one plan action a cancellation takes:
//!  remove a plan whose every name is still `planned` and whose head is `present`,
//!  and remove no other.
//! Why:
//!  The plan design states one rule for a cancellation in both of its sections:
//!  the removal that cancels itself and the cancel command remove a plan that witnessed nothing
//!  and never one that records a destroyed snapshot
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "A cancellation,
//! and the plan it leaves").
//!  The plan is the loss record when anything went,
//!  and it lives until that clone's next removal reads it,
//!  carries the loss forward and ends it,
//!  so this module unlinks nothing that holds a line a person may need.
//!  The plan's own reader is what refuses a plan this build could not read,
//!  and the name is the clone's GUID,
//!  so a plan beside another clone's name is refused rather than removed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fate = thePlanACancellationLeaves(commonDir, recorded, plan, destroyed); // { kind: 'no-plan' } | { kind: 'removed' } | { kind: 'kept' }
//! ```

use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{HeadWord, SnapshotOutcome};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use std::path::{Path, PathBuf};

/// What:
///  What became of the plan a removal left.
/// Why:
///  The report names the plan's file when it stands,
///  so a person can read the loss record until the clone's next removal ends it,
///  and names nothing when the plan witnessed nothing and left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fate = { kind: 'no-plan' } | { kind: 'removed'; path } | { kind: 'kept'; path };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThePlansFate {
    /// No plan stood for this clone.
    NoPlan,
    /// The plan witnessed nothing and left.
    Removed {
        /// The file that left.
        path: PathBuf,
    },
    /// The plan stands,
    /// as the removal wrote it.
    Kept {
        /// The plan's file.
        path: PathBuf,
    },
}

impl ThePlansFate {
    /// Whether a plan still stands,
    /// which the report and `doctor` name.
    pub fn stands(&self) -> bool {
        return matches!(self, ThePlansFate::Kept { .. });
    }
}

/// What:
///  Take the one plan action a cancellation may take.
/// Why:
///  The unlink runs only in the state that witnessed nothing,
///  and only after the plan is proved to name this clone,
///  so a kept plan is the loss record and never a stale record a later run reads as its own.
///
/// # Errors
/// A plan of another clone,
/// or a plan that could not be removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thePlanACancellationLeaves(commonDir, recorded, plan, destroyed);
/// ```
pub fn the_plan_a_cancellation_leaves(
    common_dir: &Path,
    recorded: &RecordedBinding,
    plan: Option<&DestructionPlan>,
    destroyed: &[String],
) -> Result<ThePlansFate, CancelRefusal> {
    let Some(standing) = plan else {
        return Ok(ThePlansFate::NoPlan);
    };
    let path: PathBuf = plans_folder(common_dir).join(recorded.clone_guid.file_name());
    if standing.facts.clone_guid != recorded.clone_guid || standing.facts.clone_dataset != recorded.clone_dataset {
        return Err(CancelRefusal::ThePlanIsOfAnotherClone {
            path,
            plan_guid: standing.facts.clone_guid.file_name(),
            recorded_guid: recorded.clone_guid.file_name(),
        });
    }
    let witnessed_nothing: bool = destroyed.is_empty() && standing.head == HeadWord::Present && every_line_is_planned(standing);
    if !witnessed_nothing {
        return Ok(ThePlansFate::Kept { path });
    }
    return match std::fs::remove_file(path.as_path()) {
        Ok(()) => Ok(ThePlansFate::Removed { path }),
        Err(error) => Err(CancelRefusal::ThePlanCouldNotGo { path, said: error.to_string() }),
    };
}

/// Whether every line the plan holds is still `planned`,
/// which is the state a cancellation may remove.
fn every_line_is_planned(plan: &DestructionPlan) -> bool {
    for line in &plan.attempts {
        match plan.current_state(line.snapshot.name.as_str()) {
            Some(held) if held.outcome == SnapshotOutcome::Planned => {}
            _ => return false,
        }
    }
    return true;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_plan_it_leaves_tests.rs"]
mod tests;
