//! What:
//!  The cheap-worktree ladder around one `git worktree add`: the steps the wrapper takes before real Git runs and after
//!  it returns, so that the new worktree is a btrfs subvolume and, where a rung applies, is made from a snapshot or from
//!  clones instead of Git's checkout.
//! Why:
//!  `doc/planning/cli-git-btrfs-worktree.md` designs the ladder and measures it: a snapshot of a clean subvolume
//!  worktree, then clones of its tracked files, then Git's own checkout. Each rung that cannot run falls to the next
//!  without a word to the person and leaves one diagnostic line for the log. This module is the one place the lifecycle
//!  of the copy (`worktree_copy.rs`) calls, so the rungs stay out of that module's settlement and recovery code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ladder = beginLadder(plan);
//! const git = await runGit();
//! await ladder.finish(git);
//! ```

use super::worktree_add_plan::AddPlan;
#[cfg(target_os = "linux")]
use super::worktree_subvolume::{CreatedSubvolume, prepare_subvolume_destination, settle_subvolume_destination};

/// What:
///  What the ladder has done before Git ran and must settle after it.
/// Why:
///   The destination subvolume it created is its to remove when Git leaves it empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ladder = { subvolume?: CreatedSubvolume };
/// ```
#[derive(Debug)]
pub struct Ladder {
    /// The empty subvolume made at the destination, when one was.
    #[cfg(target_os = "linux")]
    subvolume: Option<CreatedSubvolume>,
}

/// What:
///  Start the ladder for one invocation: make the destination an empty subvolume when the plan names one and its parent
///  is on btrfs.
/// Why:
///   Real Git then fills a subvolume, so the next `git worktree add` can snapshot it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function beginLadder(plan: AddPlan | undefined): Ladder;
/// ```
pub fn begin_ladder(plan: Option<&AddPlan>) -> Ladder {
    #[cfg(target_os = "linux")]
    {
        return Ladder { subvolume: plan.and_then(prepare_subvolume_destination) };
    }
    // Only btrfs has subvolumes and snapshots; other systems keep Git's own checkout.
    #[cfg(not(target_os = "linux"))]
    {
        let _ = plan;
        return Ladder {};
    }
}

impl Ladder {
    /// What:
    ///  Settle what the ladder prepared, after real Git returned whatever its status was.
    /// Why:
    ///   An empty destination Git did not fill is removed, so a failed run leaves nothing behind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await ladder.finish();
    /// ```
    pub fn finish(self) {
        #[cfg(target_os = "linux")]
        if let Some(created) = self.subvolume.as_ref() {
            settle_subvolume_destination(created);
        }
    }
}

/// Ladder controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_tests.rs"]
mod tests;
