//! What:
//!  What a removal that stops at a holder asks of the code that restores the
//!  clone:
//!  the stop point,
//!  the records the removal already proved,
//!  and the report the person reads.
//! Why:
//!  The project's answer of 2026-11-22 makes a holder the scan sees stop the
//!  destruction,
//!  and a removal without `--force` cancels itself there and attaches the clone
//!  again
//!  (`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
//!  "A removal that meets a holder cancels itself").
//!  The reversal needs the mount request,
//!  the staging folder's settlement and the plan's one action,
//!  which are `worktree_clone_removal_cancel`'s,
//!  and that folder already reads this folder's records,
//!  so the removal cannot name it back:
//!  this seam is the one direction the layout rules leave open
//!  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`).
//!  The removal therefore names what it stopped at,
//!  and the answer beside it does the reversing.
//!  A control that is not about a stop point hands no answer at all,
//!  and a stop is then the removal's own,
//!  which is what every control of the order before this round read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cancelled = cancelling?.cancelTheRemoval({ commonDir, slot, plan, destroyed, holders, stop });
//! ```

use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use std::path::Path;

/// What:
///  The two step words the mount handler files beside an unmount a busy mount
///  rolled back.
/// Why:
///  The handler moves the clone back to its slot and files a `cancelled`
///  receipt naming one of these two words,
///  which is how a removal learns that its own unmount is what a holder
///  blocked rather than a step that refused
///  (`worktree_clone_mount_handler/sweep_unmount_answers.rs`,
///  the `UnmountOutcome::Reattached` arm with `cause: ReattachCause::Busy`,
///  which writes `bind-gone` or `bind-present` as the receipt's failed step).
///  They are the producer's own words,
///  copied from that arm,
///  so a rename there is a finding here.
pub const THE_ROLLBACK_STEPS: &[&str] = &["bind-gone", "bind-present"];

/// Whether one receipt's failed step is the handler's rollback of a busy unmount.
pub fn is_the_handlers_rollback(step: &str) -> bool {
    return THE_ROLLBACK_STEPS.contains(&step);
}

/// What:
///  The point a removal stopped at,
///  as far as a removal of this build can reach one.
/// Why:
///  The design names five stop points,
///  and three of them are reachable here:
///  the handler's own rollback of a busy unmount,
///  the fresh scan before a snapshot delete,
///  and the head destroy a holder outlasts
///  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
///  "The stop points",
///  points 6,
///  12 and 15).
///  The design's point 5 exists only while the forced arm's scan stands,
///  and no forced arm ships here,
///  and its point 8 is a scan after the unmount that no step builds yet,
///  so neither is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StopPoint = 'in-staging' | 'by-a-scan-before-a-delete' | 'at-the-head-destroy';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TheStopPoint {
    /// The handler bound the folder,
    /// moved the clone onto it,
    /// and either its scan saw a holder or the unmount refused `EBUSY`,
    /// so it moved the clone back to its slot.
    InStaging,
    /// The fresh all-namespace scan before a snapshot delete saw a holder,
    /// and the earlier chunks are gone.
    ByAScanBeforeADelete,
    /// The head destroy was refused `dataset is busy` with the clone's own
    /// snapshot list empty.
    AtTheHeadDestroy,
}

impl TheStopPoint {
    /// The clause a sentence or a report names this point with.
    pub fn clause(self) -> &'static str {
        return match self {
            TheStopPoint::InStaging => {
                "the clone's unmount was blocked by a holder, so the handler moved the clone back to its slot"
            }
            TheStopPoint::ByAScanBeforeADelete => {
                "the all-namespace scan before a snapshot delete saw a holder"
            }
            TheStopPoint::AtTheHeadDestroy => {
                "the head destroy was refused `dataset is busy` with the clone's own snapshot list empty"
            }
        };
    }
}

/// What:
///  Everything one self-cancellation is asked with.
/// Why:
///  The removal holds the slot's claim and has proved the clone already,
///  so the answer takes no claim and needs the records,
///  the plan and the holders,
///  and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stopped = { commonDir; slot; plan?; destroyed; holders; stop };
/// ```
pub struct TheStoppedRemoval<'a> {
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The slot the removal was removing.
    pub slot: &'a str,
    /// The plan the removal wrote or adopted,
    /// and nothing when the stop came before the plan's first write.
    pub plan: Option<&'a DestructionPlan>,
    /// The snapshots the plan's own lines prove destroyed,
    /// which no cancellation restores.
    pub destroyed: &'a [String],
    /// Every holder the scan or the receipt named,
    /// in its own words.
    pub holders: &'a [String],
    /// Where the removal stopped.
    pub stop: TheStopPoint,
}

/// What:
///  What one self-cancellation left,
///  as the lines a person reads and the status the command ends in.
/// Why:
///  The report is the answer's own,
///  and the removal prints it as it stands rather than rebuilding it:
///  the snapshot names it says were destroyed are facts only the answer holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cancelled = { lines: string[]; code: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheCancelledRemoval {
    /// The report,
    /// one line per element.
    pub lines: Vec<String>,
    /// The status the command ends in,
    /// because the worktree was not removed.
    pub code: i32,
}

/// What:
///  A self-cancellation that could not be made,
///  with the sentence a person reads instead.
/// Why:
///  A request that could not be written,
///  an answer that never came and a plan that could not go
///  leave a state a later command settles,
///  and each says so in its own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Failed = { sentence: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TheCancellationFailed {
    /// The sentence the removal prints in place of a report.
    pub sentence: String,
}

/// What:
///  What a removal that stops at a holder asks the account's answer to do.
/// Why:
///  The removal names the stop and its records,
///  and the answer owns every write:
///  one `mount` request under a fresh nonce,
///  its verified answer,
///  the staged folder's settlement,
///  and the plan's one action.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TheRemovalCancelling { cancelTheRemoval(request: TheStoppedRemoval): Cancelled }
/// ```
pub trait TheRemovalCancelling {
    /// Reverse one removal a holder stopped.
    ///
    /// # Errors
    ///
    /// The request that could not be written,
    /// the answer that never came or named another mount,
    /// or a plan that could not be removed.
    fn cancel_the_removal(
        &self,
        request: &TheStoppedRemoval<'_>,
    ) -> Result<TheCancelledRemoval, TheCancellationFailed>;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_self_cancelling_seam_tests.rs"]
mod tests;
