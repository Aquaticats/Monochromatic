//! What:
//!  The one value the wrapper's forwarded removal path needs from this folder:
//!  the account's answer to a removal that a holder stopped.
//! Why:
//!  The answer lives in `worktree_clone_removal_cancel`,
//!  and the forwarded command path lives in `worktree_copying`,
//!  which may not name that folder:
//!  the crate's layout allow list tolerates the uses it has today and may only
//!  shrink,
//!  so a new use by the forwarded path is a finding no line may record
//!  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`).
//!  This folder is one the forwarded path already uses and one that may name
//!  the cancellation,
//!  so the value is handed over here and the removal order takes it as a seam
//!  (`worktree_clone_removal_proof_and_plan::the_self_cancelling_seam`).
//!  It decides nothing:
//!  the stop points and the reversal belong to the order and to the cancel
//!  folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cancelling = theRemovalsCancelling();
//! ```

use crate::worktree_clone_removal_cancel::the_self_cancelling_answer::TheSelfCancellingAnswer;
use crate::worktree_clone_removal_proof_and_plan::the_self_cancelling_seam::TheRemovalCancelling;

/// The one answer,
/// made once so a removal's stop points share the value.
static THE_ANSWER: TheSelfCancellingAnswer = TheSelfCancellingAnswer;

/// What:
///  The account's answer to a removal that a holder stopped.
/// Why:
///  The removal order asks it at each stop point it reaches,
///  and it owns every write that puts the clone back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cancelling = theRemovalsCancelling();
/// ```
pub fn the_removals_cancelling() -> &'static dyn TheRemovalCancelling {
    return &THE_ANSWER;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_removals_cancelling_tests.rs"]
mod tests;
