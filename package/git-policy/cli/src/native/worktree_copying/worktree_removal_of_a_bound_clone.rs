//! What:
//!  The refusal of one `git worktree remove` of a slot a whole managed clone is
//!  bound to,
//!  mounted or not,
//!  with the slot's state printed beside it.
//! Why:
//!  Git's own removal deletes the registration and the slot's underlay folder,
//!  which is the pointer the add kept for `git worktree prune`,
//!  and the clone dataset with the person's checked-out files would then be
//!  reachable by nobody
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  Only the two states that prove a whole worktree refuse:
//!  a pending add's removal is the add's own way forward,
//!  and a slot nothing proves stays Git's.
//!  Nothing here removes,
//!  unmounts or writes:
//!  a refusal says what stands and the one next safe command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = refuseTheRemovalOfABoundClone(facts); // undefined leaves the removal to Git
//! ```

use crate::subvolume_command::REFUSED_EXIT_CODE;
use crate::worktree_clone_slot_state::slot_state_classification::SlotRouteState;
use crate::worktree_copy_forward::{CopyFacts, ForwardedRun};
use crate::worktree_identity::common_directory;
use std::path::Path;

/// What:
///  The refusal of a removal of a slot a whole managed clone is bound to,
///  as the run the command ends in.
/// Why:
///  A command that prints its own report and changes nothing is not a failure
///  of a step,
///  so it ends with the wrapper's refusal status,
///  and Git never runs against the slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ending = refuseTheRemovalOfABoundClone(facts);
/// ```
pub fn refuse_the_removal_of_a_bound_clone(facts: &CopyFacts) -> Option<ForwardedRun> {
    let plan = facts.remove_plan.as_ref()?;
    let common_dir: &Path = common_directory(&facts.identity)?;
    let slot: String = crate::worktree_clone_binding_record::registration_home::slot_of(plan.target.as_path()).ok()?;
    let at = crate::worktree_clone_link_repair::the_slot_at(common_dir, slot.as_str(), plan.target.as_path());
    if !matches!(at.state, SlotRouteState::MountedManagedClone { .. } | SlotRouteState::KnownUnmountedClone { .. }) {
        return None;
    }
    let mut stderr: String = format!(
        "the removal of `{}` is refused: a whole managed clone is bound to this slot, and this release has no removal for a clone (plan steps 14 and 16 build it). Nothing was removed, and Git did not run.\n",
        plan.target.display()
    );
    for line in at.lines() {
        stderr.push_str(line.as_str());
        stderr.push('\n');
    }
    return Some(ForwardedRun::Failed { stderr, code: REFUSED_EXIT_CODE });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_of_a_bound_clone_tests.rs"]
mod tests;
