//! What:
//!  The step a removal takes when a holder stops it:
//!  the answer's own report,
//!  or the sentence for an answer that could not put the clone back.
//! Why:
//!  Every stop point asks the same question and prints the same two shapes,
//!  so the call is written once and each point names only where it stopped
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The stop points").
//!  A removal a control runs with no answer wired keeps the stop it had
//!  before this round,
//!  which is why the call answers nothing rather than a stop:
//!  the release's command path always hands one
//!  (`worktree_clone_removal_proof_and_plan::the_self_cancelling_seam`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stopped = cancelledAt({ cancelling, commonDir, slot, plan, holders, stop });
//! if (stopped) throw stopped;
//! ```

use crate::worktree_clone_removal_proof_and_plan::plan_names_and_states::the_names_destroyed;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_inputs::RemovalCompletion;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::{
    TheRemovalCancelling,
    TheStopPoint,
    TheStoppedRemoval,
    is_the_handlers_rollback,
};
use std::path::Path;

/// What:
///  Everything one stop point asks its answer with.
/// Why:
///  The removal holds the slot's claim and has proved the clone already,
///  so the order's own facts,
///  the plan and the holders are all a call site names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stopped = { cancelling?; commonDir; slot; plan?; holders; stop };
/// ```
pub(crate) struct TheStopTheRemovalTakes<'a> {
    /// The account's answer,
    /// and nothing when no answer is wired.
    pub cancelling: Option<&'a dyn TheRemovalCancelling>,
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The slot the removal is removing.
    pub slot: &'a str,
    /// The plan the removal wrote or adopted,
    /// and nothing before its first write.
    pub plan: Option<&'a DestructionPlan>,
    /// Every holder the scan or the receipt named.
    pub holders: &'a [String],
    /// Where the removal stopped.
    pub stop: TheStopPoint,
}

/// What:
///  The stop a removal ends in at one stop point,
///  and nothing when no answer is wired.
/// Why:
///  A cancellation exits with the wrapper's refusal status because the worktree
///  was not removed,
///  and a failed answer says what it left so the next command can settle it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopped = cancelledAt(the);
/// ```
pub(crate) fn cancelled_at(the: &TheStopTheRemovalTakes<'_>) -> Option<RemovalStop> {
    let cancelling: &dyn TheRemovalCancelling = the.cancelling?;
    let destroyed: Vec<String> = the.plan.map(the_names_destroyed).unwrap_or_default();
    let asked: TheStoppedRemoval<'_> = TheStoppedRemoval {
        common_dir: the.common_dir,
        slot: the.slot,
        plan: the.plan,
        destroyed: destroyed.as_slice(),
        holders: the.holders,
        stop: the.stop,
    };
    return Some(match cancelling.cancel_the_removal(&asked) {
        Ok(done) => RemovalStop::TheRemovalCancelledItself {
            said: done.lines.join("\n"),
        },
        Err(failed) => RemovalStop::TheRemovalCouldNotCancelItself {
            stop: the.stop.clause(),
            said: failed.sentence,
        },
    });
}

/// What:
///  The stop a removal ends in at its own unmount step:
///  its cancellation where the handler rolled a busy unmount back,
///  and the step's own refusal everywhere else.
/// Why:
///  A rollback is the one unmount answer that puts the clone at its slot again,
///  so it is the design's second stop point and not a refused step
///  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
///  "The stop points",
///  point 6).
///  The plan is not named here because the unmount request runs before the
///  plan's first write,
///  and an adopted plan a rollback meets is left for the next removal to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw atTheUnmount(input, stop);
/// ```
pub(crate) fn at_the_unmount(
    input: &RemovalCompletion<'_>,
    stop: RemovalStop,
) -> RemovalStop {
    let RemovalStop::UnmountFailed {
        step,
        ..
    } = &stop
    else {
        return stop;
    };
    if !is_the_handlers_rollback(step.as_str()) {
        return stop;
    }
    let the: TheStopTheRemovalTakes<'_> = TheStopTheRemovalTakes {
        cancelling: input.cancelling,
        common_dir: input.entry.common_dir,
        slot: input.entry.slot,
        plan: None,
        holders: &[],
        stop: TheStopPoint::InStaging,
    };
    return match cancelled_at(&the) {
        Some(stopped) => stopped,
        None => stop,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_self_cancelling_step_tests.rs"]
mod tests;
