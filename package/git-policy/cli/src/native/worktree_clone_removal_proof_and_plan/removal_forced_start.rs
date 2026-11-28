//! What:
//!  Where a forced removal starts when a destruction plan it may not read stands.
//! Why:
//!  The owner answered on 2026-11-28 that one force goes through a plan that
//!  cannot be read:
//!  "it proves the clone by the record's GUID against the pool,
//!  destroys the clone and its snapshots from the pool's own list,
//!  deletes the unreadable plan file"
//!  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`).
//!  The step that destroys the clone and its snapshots is the plan's own once a
//!  plan exists,
//!  so this start enters the removal with the file named for removal and the act
//!  written down for the report,
//!  and the live proof inside the entry reads the pool's own list.
//!  A pool that does not answer the name the record holds is refused:
//!  nothing then proves what the plan's clone was,
//!  and destroying nothing while deleting the only record of an earlier run is no
//!  act any answer authorizes.
//!  This sits beside the resume rather than inside it,
//!  so the reads that decide
//!  every other start stay one file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const started = theForcedStartPastAPlan(request, recorded, planPath);
//! ```

use crate::worktree_clone_removal_proof_and_plan::removal_entry::{
    EnteringTheRemoval,
    enter_the_removal_with_force,
};
use crate::worktree_clone_removal_proof_and_plan::removal_forced_acts::forced_past_a_plan;
use crate::worktree_clone_removal_proof_and_plan::removal_start_inputs::{
    TheStart,
    TheStartingRemoval,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::Path;

/// What:
///  Where a forced removal starts when a destruction plan it may not read stands.
/// Why:
///  The owner answered on 2026-11-28 that one force goes through a plan that
///  cannot be read:
///  "it proves the clone by the record's GUID against the pool,
///  destroys the clone and its snapshots from the pool's own list,
///  deletes the unreadable plan file"
///  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`).
///  The step that destroys the clone and its snapshots is the plan's own once a
///  plan exists,
///  so this start enters the removal with the file named for removal and the act
///  written down for the report,
///  and the live proof inside the entry reads the pool's own list.
///  A pool that does not answer the name the record holds is refused here:
///  nothing then proves what the plan's clone was,
///  and destroying nothing while deleting the only record of an earlier run is
///  no act any answer authorizes.
///
/// # Throws
/// The plan's file,
/// the pool that did not answer,
/// or the entry's own refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const started = theForcedStartPastAPlan(request, recorded, planPath);
/// ```
pub(crate) fn the_forced_start_past_a_plan(
    request: &TheStartingRemoval<'_>,
    recorded: &crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding,
    plan_path: &Path,
) -> Result<TheStart, RemovalStop> {
    if let Err(reason) = request
        .reads
        .identity(recorded.clone_dataset.as_str())
        .map(|_clone| {})
    {
        return Err(RemovalStop::ThePlanIsUnreadableAndTheCloneIsGone {
            slot: String::from(request.entry.slot),
            path: plan_path.to_path_buf(),
            reason,
        });
    }
    return Ok(TheStart::Present {
        entered: enter_the_removal_with_force(&EnteringTheRemoval {
            entry: request.entry,
            reads: request.reads,
            unmount: request.unmount,
            clock: request.clock,
            staging: request.staging,
            force: request.force,
            forced_before: forced_past_a_plan(plan_path),
        })?,
    });
}
