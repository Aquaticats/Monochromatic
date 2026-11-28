//! What:
//!  The account's answer to a removal that a holder stopped:
//!  one `mount` request,
//!  its verified answer,
//!  the staged folder's settlement and the plan's one action.
//! Why:
//!  The removal order may not name this folder,
//!  and this folder already reads the order's records,
//!  so the answer is here and the order asks it through
//!  `worktree_clone_removal_proof_and_plan::the_self_cancelling_seam`
//!  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`).
//!  Every write is the one the design gives a cancellation:
//!  the intent first,
//!  because a standing `unmount` request must not be acted on again,
//!  and the plan last,
//!  so a kill between any two steps leaves a state the next command settles
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The cancelling sequence's own states").
//!  The binding is read again here rather than carried across the seam:
//!  every deciding read of this route is repeated under the claim,
//!  and the removal holds that claim while it asks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cancelled = theSelfCancellingAnswer().cancelTheRemoval(stopped);
//! ```

use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::the_cancelled_removal::{
    TheSelfCancellation,
    WhereItStopped,
    the_removal_cancels_itself,
};
use crate::worktree_clone_removal_cancel::the_reversing_request::LiveTheMountRequest;
use crate::worktree_clone_removal_cancel::the_staged_state::LiveTheStagedFolder;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::read_the_recorded_binding;
use crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::{
    TheCancellationFailed,
    TheCancelledRemoval,
    TheRemovalCancelling,
    TheStopPoint,
    TheStoppedRemoval,
};
use crate::worktree_clone_unmount::user_cache_pin::passwd_home_of;
use std::path::PathBuf;

/// What:
///  The release's answer to a removal that stopped at a holder.
/// Why:
///  It holds nothing:
///  everything it writes is decided by the records the removal names,
///  so one value serves every stop point of every removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = new TheSelfCancellingAnswer();
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TheSelfCancellingAnswer;

/// What the design's stop point is called in this folder's own words.
fn where_it_stopped(stop: TheStopPoint) -> WhereItStopped {
    return match stop {
        TheStopPoint::InStaging => WhereItStopped::InStaging,
        TheStopPoint::ByAScanBeforeADelete => WhereItStopped::ByAScanBeforeADelete,
        TheStopPoint::AtTheHeadDestroy => WhereItStopped::AtTheHeadDestroy,
    };
}

impl TheRemovalCancelling for TheSelfCancellingAnswer {
    fn cancel_the_removal(
        &self,
        request: &TheStoppedRemoval<'_>,
    ) -> Result<TheCancelledRemoval, TheCancellationFailed> {
        let recorded = read_the_recorded_binding(request.common_dir, request.slot).map_err(
            |stop| {
                return TheCancellationFailed {
                    sentence: stop.message(),
                };
            },
        )?;
        let home: PathBuf = match passwd_home_of(unsafe { libc::geteuid() }) {
            Ok(home) => home,
            Err(refusal) => {
                return Err(TheCancellationFailed {
                    sentence: format!(
                        "this account's home could not be read ({}), so the staged folder cannot be settled.",
                        refusal.message()
                    ),
                });
            }
        };
        let mount_request = LiveTheMountRequest::open(
            recorded.clone_dataset.as_str(),
            recorded.clone_guid,
        )
        .map_err(|refusal| {
            return TheCancellationFailed {
                sentence: refusal.message(),
            };
        })?;
        let staged: LiveTheStagedFolder<'_> = LiveTheStagedFolder {
            home: home.as_path(),
            common_dir: request.common_dir,
        };
        let done = the_removal_cancels_itself(&TheSelfCancellation {
            common_dir: request.common_dir,
            recorded: &recorded,
            slot_path: recorded.slot_path.as_path(),
            plan: request.plan,
            destroyed: request.destroyed,
            stop: where_it_stopped(request.stop),
            holders: request.holders,
            mount_request: &mount_request,
            staged: &staged,
        })
        .map_err(|refusal: CancelRefusal| {
            return TheCancellationFailed {
                sentence: refusal.message(),
            };
        })?;
        return Ok(TheCancelledRemoval {
            lines: done.lines(),
            code: done.exit_code(),
        });
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_self_cancelling_answer_tests.rs"]
mod tests;
