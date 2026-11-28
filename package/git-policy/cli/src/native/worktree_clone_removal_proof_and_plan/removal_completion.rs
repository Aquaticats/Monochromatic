//! What:
//!  One removal's whole order:
//!  the slot's per-slot claim at the order's first step,
//!  the two paths a removal starts from,
//!  and the claim released as the order's last step.
//! Why:
//!  This is the writing path plan step 16 builds behind the removal's cut:
//!  nothing in the release's command path calls it,
//!  and the step that switches the command over is a later round's
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 16).
//!  The order is the origin cleanup design's fourteen steps,
//!  and its first and last are the claim:
//!  taken before anything reads a record,
//!  and released after the plan is proved gone,
//!  so an add or a second removal of the same slot meets it rather than interleaving
//!  with this one's writes
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals").
//!  Every refusal before a destroy leaves the clone unmounted with its records kept
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals").
//!  What the order drives is `removal_completion_inputs.rs`,
//!  the names it asks about are `plan_names_and_states.rs`,
//!  and the two paths are `removal_completion_head_present.rs`
//!  and `removal_completion_head_gone.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = completeTheRemoval({ entry, reads, unmount, clock, check, probe, ... });
//! console.log(done.lines().join('\n'));
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_head_gone::the_head_gone_path;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_head_present::the_head_present_path;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_inputs::RemovalCompletion;
use crate::worktree_clone_removal_proof_and_plan::removal_report::TheRemovalDone;
use crate::worktree_clone_removal_proof_and_plan::removal_resume::{
    TheStart,
    start_the_removal,
};
use crate::worktree_clone_removal_proof_and_plan::removal_self_cancelling_step::at_the_unmount;
use crate::worktree_clone_removal_proof_and_plan::removal_slot_claim::{
    RemovalClaim,
    release_at_the_last_step,
    take_the_removals_claim,
};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;

/// What:
///  One whole removal,
///  from the slot's claim to the report and the claim's release.
/// Why:
///  The report is the removal's only trace beyond its records,
///  and the value keeps what was destroyed beside what remains,
///  so no caller can print one without the other.
///
/// # Throws
/// A refusal at any step,
/// each naming what it left and the way forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = completeTheRemoval(input);
/// ```
pub fn complete_the_removal(input: &RemovalCompletion<'_>) -> Result<TheRemovalDone, RemovalStop> {
    // Step 1: the claim before any read, so an add or a second removal of this slot meets it here rather than
    // interleaving with this run's records.
    let mut claim: RemovalClaim =
        take_the_removals_claim(input.entry.common_dir, input.entry.slot)?;
    let started: TheStart = start_the_removal(
        input.entry,
        input.reads,
        input.unmount,
        input.clock,
        input.probe,
    )
    .map_err(|stop| return at_the_unmount(input, stop))?;
    let folder: PlanFolder =
        PlanFolder::open_or_create(input.entry.common_dir).map_err(|refusal| {
            return RemovalStop::PlanUnreadable {
                path: plans_folder(input.entry.common_dir),
                refusal,
            };
        })?;
    let mut done: TheRemovalDone = match started {
        TheStart::Gone {
            plan,
            plan_path,
            lost,
            pool,
        } => the_head_gone_path(input, folder, plan, plan_path, lost, pool)?,
        TheStart::Present {
            entered,
        } => {
            let lost: NamesToBeLost = entered.lost.clone();
            the_head_present_path(input, folder, entered, lost)?
        }
    };
    // Step 14: the release is last, after the plan is proved gone, so no command sees a free slot whose removal's
    // last witness still stands. Every way out before this line gives the claim back through its own drop.
    done.claim = release_at_the_last_step(&mut claim);
    return Ok(done);
}

/// One live object's kind used by the report,
/// so a bookmark and a snapshot never read alike.
pub const BOOKMARK_KIND: EntryKind = EntryKind::Bookmark;

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_completion_tests.rs"]
mod tests;
