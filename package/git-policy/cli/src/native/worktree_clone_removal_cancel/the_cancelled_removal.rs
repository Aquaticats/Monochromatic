//! What:
//!  What a removal restores when it cancels itself,
//!  and what a forced removal's failure leaves:
//!  the same reversal the command makes,
//!  with the words the stop point owes a person.
//! Why:
//!  The project's answer of 2026-11-22 makes a holder the scan sees stop the destruction,
//!  and the record decides that a cancellation after part of the history is gone attaches the
//!  clone again and reports each snapshot it destroyed
//!  (`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
//!  "A cancellation after part of the history is gone").
//!  The design names five points where a removal stops by cancelling itself and one restored state
//!  for a forced removal's failure
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The stop points" and "What a removal that cancels itself restores"):
//!  the scan before the move,
//!  the blocked unmount in staging,
//!  a scan after the unmount or before a delete,
//!  and the head destroy a holder outlasts.
//!  The removal holds its own claim already,
//!  knows the clone from its own proof and knows the destroyed set from the plan it wrote,
//!  so this module takes no claim and writes no plan field.
//!  Its outcome is a report and the exit status `1`,
//!  because the worktree was not removed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = theRemovalCancelsItself({ commonDir, recorded, slotPath, plan, outcomes, stop, holders, mountRequest, staged });
//! for (const line of done.lines()) console.error(line);
//! process.exit(1);
//! ```

use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::the_cancelling_sequence::{
    TheReversalDone,
    TheReversalRequest,
    reverse_the_removal,
};
use crate::worktree_clone_removal_cancel::the_plan_it_leaves::ThePlansFate;
use crate::worktree_clone_removal_cancel::the_reversing_request::TheMountRequest;
use crate::worktree_clone_removal_cancel::the_staged_state::{
    StagedSettlement,
    TheStagedFolder,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use std::path::Path;

/// The status a removal that cancelled itself exits with,
/// because the worktree was not removed.
pub const CANCELLED_BY_ITSELF: i32 = 1;

/// What:
///  Where a removal stopped.
/// Why:
///  Each point destroyed a different amount of history and leaves the clone in a different place,
///  and the report's first line and its holder sentence depend on which one it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stop = 'before-the-move' | 'in-staging' | 'by-a-scan' | 'before-a-delete' | 'at-the-head-destroy' | 'the-forced-arms-failure';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhereItStopped {
    /// The unmount request's own scan saw a holder before anything moved.
    BeforeTheMove,
    /// The holder kept the staged clone busy and the handler moved it back.
    InStaging,
    /// A scan after the unmount saw a holder,
    /// and nothing was destroyed.
    ByAScanAfterTheUnmount,
    /// A scan before a delete saw a holder,
    /// and the earlier chunks are gone.
    ByAScanBeforeADelete,
    /// The head destroy refused `dataset is busy` with the clone's own snapshot list empty.
    AtTheHeadDestroy,
    /// A forced removal could not do what it promised,
    /// and its failure path restores this same state.
    TheForcedArmsFailure,
}

impl WhereItStopped {
    /// Whether nothing of the clone's history was destroyed at this point.
    ///
    /// Why:
    ///  The report says either that every snapshot survives or which ones do not,
    ///  and the stop point is what decides which sentence is true.
    pub fn destroyed_nothing(self) -> bool {
        return match self {
            WhereItStopped::BeforeTheMove
            | WhereItStopped::InStaging
            | WhereItStopped::ByAScanAfterTheUnmount => true,
            WhereItStopped::ByAScanBeforeADelete
            | WhereItStopped::AtTheHeadDestroy
            | WhereItStopped::TheForcedArmsFailure => false,
        };
    }

    /// The clause the report's first line carries.
    pub fn clause(self) -> &'static str {
        return match self {
            WhereItStopped::BeforeTheMove => "the all-namespace scan before the move saw a holder",
            WhereItStopped::InStaging => {
                "the clone's unmount was blocked by a holder in another session"
            }
            WhereItStopped::ByAScanAfterTheUnmount => {
                "the all-namespace scan after the unmount saw a holder in a namespace this namespace's unmount does not reach"
            }
            WhereItStopped::ByAScanBeforeADelete => {
                "the all-namespace scan before a snapshot delete saw a holder"
            }
            WhereItStopped::AtTheHeadDestroy => {
                "the head destroy was refused `dataset is busy` with the clone's own snapshot list empty"
            }
            WhereItStopped::TheForcedArmsFailure => {
                "it was asked to end the holders other than your protected set and could not"
            }
        };
    }
}

/// What:
///  Everything one self-cancellation drives.
/// Why:
///  The removal holds its claim and knows its own plan,
///  so what it needs is the records,
///  the settled outcomes,
///  the stop point,
///  the holders the scan or the receipt named,
///  and the two seams that move the mount and settle the folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cancellation = { commonDir; recorded; slotPath; plan?; destroyed; stop; holders; mountRequest; staged };
/// ```
pub struct TheSelfCancellation<'a> {
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The identities the removal proved.
    pub recorded: &'a RecordedBinding,
    /// The clone's slot.
    pub slot_path: &'a Path,
    /// The plan the removal wrote or adopted,
    /// where it wrote one.
    pub plan: Option<&'a DestructionPlan>,
    /// The snapshots the plan's tuples prove destroyed.
    pub destroyed: &'a [String],
    /// Where the removal stopped.
    pub stop: WhereItStopped,
    /// Every holder the scan or the receipt named,
    /// in its own words.
    pub holders: &'a [String],
    /// The route's own request writer and wait.
    pub mount_request: &'a dyn TheMountRequest,
    /// The staging folder's settlement.
    pub staged: &'a dyn TheStagedFolder,
}

/// What:
///  What one self-cancellation left,
///  as the report names it.
/// Why:
///  A person reads what was destroyed,
///  that it cannot be restored,
///  where the clone stands,
///  what stayed staged,
///  and what to run next,
///  and every one of those is a fact this value holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Done = { slot; dataset; guid; stop; holders; destroyed; mountId; staged; plans };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheSelfCancellationDone {
    /// The slot that was being removed.
    pub slot: String,
    /// The clone's dataset name.
    pub dataset: String,
    /// The clone's GUID.
    pub guid: String,
    /// Where the removal stopped.
    pub stop: WhereItStopped,
    /// Every holder the scan or the receipt named.
    pub holders: Vec<String>,
    /// Every snapshot the removal destroyed.
    pub destroyed: Vec<String>,
    /// The nonce the reversing request was filed under.
    pub nonce: String,
    /// The mount the handler's answer observed.
    pub mount_id: u64,
    /// What became of the staged state.
    pub staged: StagedSettlement,
    /// What became of the plan.
    pub plans: ThePlansFate,
}

impl TheSelfCancellationDone {
    /// What:
    ///  The report a person reads on standard error.
    /// Why:
    ///  It names the slot,
    ///  the dataset and GUID,
    ///  the operation that was blocked and its reason,
    ///  any holder,
    ///  what was destroyed and that it cannot be restored,
    ///  the mount state,
    ///  and the way forward.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// console.error(done.lines().join('\n'));
    /// ```
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::with_capacity(6);
        lines.push(format!(
            "removal of `{slot}` cancelled itself: {clause}, so {loss} The clone is mounted at its slot.",
            slot = self.slot,
            clause = self.stop.clause(),
            loss = if self.stop.destroyed_nothing() { "nothing was destroyed and every snapshot survives." } else { "snapshots were destroyed and cannot be restored." }
        ));
        lines.push(format!(
            "The request that was blocked is the removal's `unmount`; the removal wrote a fresh `mount` request under nonce `{nonce}` and the handler answered it, observing mount {mount_id}. The clone's dataset is `{dataset}` (GUID `{guid}`).",
            nonce = self.nonce,
            mount_id = self.mount_id,
            dataset = self.dataset,
            guid = self.guid
        ));
        if !self.destroyed.is_empty() {
            let mut names: Vec<String> = Vec::with_capacity(self.destroyed.len());
            for name in &self.destroyed {
                names.push(format!("- `{name}`"));
            }
            lines.push(format!(
                "This removal destroyed {} snapshot(s), which no cancellation restores, and the destruction plan keeps them:\n{}",
                self.destroyed.len(),
                names.join("\n")
            ));
        }
        if !self.holders.is_empty() {
            lines.push(format!(
                "The holder(s) it named:\n{}",
                self.holders.join("\n")
            ));
        }
        match &self.staged {
            StagedSettlement::NoStagedFolder => {}
            StagedSettlement::Removed { .. } | StagedSettlement::ReleasedNoFolder => {
                lines.push(String::from("The removal's staging folder, its record and its claim are gone."));
            }
            StagedSettlement::Retained { step, said } => lines.push(format!(
                "The staging folder, its record and its claim are kept and reported ({step}: {said})."
            )),
        }
        match &self.plans {
            ThePlansFate::NoPlan => {}
            ThePlansFate::Removed {
                path,
            } => lines.push(format!(
                "The destruction plan at {} witnessed nothing and left.",
                path.display()
            )),
            ThePlansFate::Kept {
                path,
                ..
            } => lines.push(format!(
                "The destruction plan at {} stands as the loss record.",
                path.display()
            )),
        }
        lines.push(String::from(
            "Ways forward: end that job and run the removal again, or run `git monochromatic-git-policy-cli cancel-worktree-removal <slot>` to read the state and put the clone back.",
        ));
        return lines;
    }

    /// The status the removal exits with,
    /// because the worktree was not removed.
    pub fn exit_code(&self) -> i32 {
        return CANCELLED_BY_ITSELF;
    }
}

/// What:
///  Cancel one removal that stopped,
///  in the one order the command also takes.
/// Why:
///  The intent write comes first,
///  because a standing `unmount` request must not be acted on again,
///  and a kill at the first step leaves a request for the state the clone is in rather than one
///  that keeps trying to unmount it.
///
/// # Errors
/// The request that could not be written,
/// the answer that never came or named another mount,
/// or a plan that could not be removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = theRemovalCancelsItself(input);
/// ```
pub fn the_removal_cancels_itself(
    input: &TheSelfCancellation<'_>
) -> Result<TheSelfCancellationDone, CancelRefusal> {
    let reversal: TheReversalDone = reverse_the_removal(&TheReversalRequest {
        common_dir: input.common_dir,
        recorded: input.recorded,
        slot_path: input.slot_path,
        plan: input.plan,
        destroyed: input.destroyed,
        mount_request: input.mount_request,
        staged: input.staged,
    })?;
    return Ok(TheSelfCancellationDone {
        slot: input.recorded.slot.clone(),
        dataset: input.recorded.clone_dataset.clone(),
        guid: input.recorded.clone_guid.file_name(),
        stop: input.stop,
        holders: input.holders.to_vec(),
        destroyed: reversal.destroyed.clone(),
        nonce: reversal.nonce,
        mount_id: reversal.mount_id,
        staged: reversal.staged,
        plans: reversal.plans,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_cancelled_removal_tests.rs"]
mod tests;
