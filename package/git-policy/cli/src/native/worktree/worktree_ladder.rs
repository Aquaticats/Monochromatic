//! What:
//!  The cheap-worktree ladder around one `git worktree add`:
//! what runs before real Git and after it,
//! on Linux;
//! on other
//!  systems the ladder does nothing and Git's own checkout stands.
//! Why:
//!  `doc/planning/cli-git-btrfs-worktree.md` designs the ladder and measures it:
//! a snapshot of a clean subvolume
//!  worktree,
//! then clones of its tracked files,
//! then Git's own checkout.
//! The lifecycle of the copy
//!  (`worktree_copy.rs`) calls this one module,
//! so the rungs stay out of its settlement and recovery code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ladder = beginLadder(plan, context, identity);
//! const git = await runGit(ladder.firstStep());
//! const { git: status, carried } = await ladder.finish(git, ...);
//! ```

#[cfg(target_os = "linux")]
pub use crate::worktree_ladder_run::{Ladder, LadderResult, begin_ladder, begin_ladder_holding};

/// What:
///  The ladder on a system without btrfs subvolumes and snapshots:
/// nothing is prepared and nothing is settled.
/// Why:
///   Git's own checkout stands there,
/// and the lifecycle calls the same four names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ladder = beginLadder(plan, context, identity); // does nothing
/// ```
#[cfg(not(target_os = "linux"))]
mod other {
    use crate::transaction_git::GitContext;
    use crate::worktree_add_plan::AddPlan;
    use crate::worktree_copy::{GitRunner, GitStatus, GitStep};
    use crate::worktree_copy_stage_claim::StageClaim;
    use crate::worktree_identity::WorktreeIdentity;
    use std::io::Write;
    use std::path::PathBuf;

    /// The ladder that does nothing.
    pub struct Ladder;

    /// What the ladder tells the lifecycle after Git.
    pub struct LadderResult {
        /// Git's status for the person.
        pub git: GitStatus,
        /// The destination that received the source's ignored state with the snapshot.
        pub carried: Option<PathBuf>,
    }

    /// Prepare nothing.
    pub fn begin_ladder(_plan: Option<&AddPlan>, _context: &GitContext, _identity: &WorktreeIdentity) -> Ladder {
        return Ladder;
    }

    /// Prepare nothing,
    /// and leave the caller's record to the copy.
    pub fn begin_ladder_holding(_plan: Option<&AddPlan>, _held: &mut Option<StageClaim>, _context: &GitContext, _identity: &WorktreeIdentity) -> Ladder {
        return Ladder;
    }

    impl Ladder {
        /// No destination was made before real Git on this system,
        /// so the lifecycle reserves its own.
        pub fn destination_identity(&self) -> Option<crate::fs_beneath::ObjectId> {
            return None;
        }

        /// The person's arguments.
        pub fn first_step(&self) -> GitStep<'static> {
            return GitStep::Person;
        }

        /// Settle nothing.
        pub fn finish(self, git: GitStatus, __run_git: &mut dyn GitRunner, _context: &GitContext, _out: &mut dyn Write) -> LadderResult {
            return LadderResult { git, carried: None, claim: None };
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub use other::{Ladder, LadderResult, begin_ladder, begin_ladder_holding};
