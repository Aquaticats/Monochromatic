//! What:
//!  The removal of a clean linked worktree that is a btrfs subvolume,
//! by one subvolume destroy call and the
//!  deletion of its administrative directory,
//! in place of Git's file-by-file removal.
//! Why:
//!  Git unlinks every file of the worktree (0.7 s without ignored state,
//! 3.6 s with a large ignored tree in the
//!  measurements of `doc/planning/cli-git-btrfs-worktree.md`),
//! and the subvolume call takes milliseconds.
//! The call
//!  cannot be undone,
//! so the wrapper takes it only when the same conditions hold that make Git remove without
//!  `--force`:
//! the worktree is registered in the repository the command runs in,
//! under the name Git would pick,
//! is not
//!  locked,
//! is clean (`worktree_clean.rs`),
//! and holds no submodule.
//! The mount must also allow the call to an
//!  unprivileged user (`user_subvol_rm_allowed`).
//! Every other case,
//! and every call that the kernel refuses,
//! is left to
//!  Git,
//! which then words the refusal itself.
//!
//!  The check and the destroy are not one step,
//!  as Git's check and its deletion are not.
//!  Three things keep the
//!  window small and its worst case no worse than Git's:
//!  the directories are held open from before the proof of
//!  cleanliness to the destroy and the destroy names the subvolume by id (or by name below the held parent),
//!  the lock,
//!  the identity of the worktree directory and of the registration are looked at again immediately before the destroy,
//!  and the registration is emptied through its held descriptor,
//!  never by a path that could name another directory by
//!  then.
//!  What remains is a write or a lock made by another process in the milliseconds between the last look and the
//!  destroy;
//!  Git has the same window for its whole deletion,
//!  which takes seconds (`SPEC.md`).
//!
//!  The trait its checks are answered through,
//!  and the kernel and Git behind it,
//!  are in `worktree_remove_operations.rs`;
//!  the ordered steps of one removal are in
//!  `worktree_remove_when_every_condition_holds.rs`,
//!  and this file re-exports what both of them give.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await removeSubvolumeWorktree(request); // 'left-to-git': run git worktree remove
//! ```

/// What:
///  The operations of one removal that touch the kernel or start Git,
///  behind one trait,
///  and the kernel and Git as they are,
///  re-exported where every caller already names them.
/// Why:
///   The trait and its one real implementation are one subject
///  (`worktree_remove_operations.rs`),
/// and the removal's own file reached the repository's length limit with them in,
/// so the subject moved and the names did not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { LinuxOperations, type RemoveOperations } from './worktree_remove_operations';
/// ```
pub use crate::worktree_remove_operations::{
    LinuxOperations,
    RemoveOperations,
};

/// What:
///  The ordered steps of one removal,
///  which answer how the attempt ended,
///  re-exported where every caller already names it.
/// Why:
///   Those steps are one subject
/// (`worktree_remove_when_every_condition_holds.rs`),
/// and the removal's own file reached the repository's length limit with them in,
/// so the subject moved and the name did not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { removeWith } from './worktree_remove_when_every_condition_holds';
/// ```
pub use crate::worktree_remove_when_every_condition_holds::remove_with;

/// What:
///  The directories one removal holds open,
///  with the identities it proved,
///  re-exported where every caller already names them.
/// Why:
///   The hold is one subject (`worktree_removal_holds_the_worktree.rs`),
/// and every file of a removal already names it here,
/// so the path does not change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type { Held } from './worktree_removal_holds_the_worktree';
/// ```
pub use crate::worktree_removal_holds_the_worktree::Held;

/// The four lines a destroy leaves,
///  and the one place that chooses between them,
///  re-exported where every caller already names them.
/// Why:
///   The words of a destroy are one subject
/// (`worktree_remove_notices.rs`),
/// and the removal's own file reached the repository's length limit with them in it,
/// so the subject
///  moved and the names did not.
pub use crate::worktree_remove_notices::{
    DESTROY_FORCED_HERE_NOTICE,
    DESTROY_FORCED_NOTICE,
    DESTROY_HERE_NOTICE,
    DESTROY_NOTICE,
    destroy_notice,
};

/// How one attempt to remove a worktree ended,
///  re-exported where every caller already names it.
/// Why:
///   The four endings are one subject (`worktree_remove_outcome.rs`),
/// and the removal's own file reached the repository's
///  length limit with them in it,
/// so the subject moved and the name did not.
pub use crate::worktree_remove_outcome::RemoveOutcome;

/// What one removal is asked to do and in which repository,
/// and the question its caller asks again before anything is destroyed,
/// re-exported where every caller already names them.
/// Why:
///   The request is one subject (`worktree_remove_request_and_last_look.rs`),
/// and the removal's own file reached the repository's
///  length limit with it in it,
/// so the subject moved and the names did not.
pub use crate::worktree_remove_request_and_last_look::{
    LastLook,
    RemoveRequest,
};

/// What:
///  Remove the worktree if every condition holds,
/// with the kernel and Git as they are.
/// Why:
///   See `remove_with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeSubvolumeWorktree(request: RemoveRequest): Promise<RemoveOutcome>;
/// ```
pub fn remove_subvolume_worktree(request: &RemoveRequest<'_>) -> RemoveOutcome {
    return remove_with(&LinuxOperations, request);
}

/// Subvolume removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_tests.rs"]
mod tests;

/// Controls that swap objects between the steps of a removal run on a stand-in for the kernel.
#[cfg(test)]
#[path = "worktree_remove_guard_tests.rs"]
mod guard_tests;
