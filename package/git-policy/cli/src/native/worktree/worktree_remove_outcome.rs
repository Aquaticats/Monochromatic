//! What:
//!  How one attempt to remove a worktree ended,
//!  and what each ending needs of the caller.
//! Why:
//!  A command that removes a worktree can end in ways whose next steps differ:
//! Git removes it (nothing changed),
//! the
//!  work is done,
//! the work is done and nothing of it was the removal's to delete,
//! the wrapper refuses,
//! or the files
//!  are gone and the registration is not.
//!  Every caller matches on these,
//! so they are one subject,
//! apart from the removal's own steps,
//! which reached the repository's
//!  length limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type RemoveOutcome = { kind: 'left-to-git' } | { kind: 'removed' } | { kind: 'destroyed-in-place' } | ...;
//! ```

use crate::subvolume_remove_walk::Boundary;

/// What:
///  How one attempt to remove a worktree with a subvolume destroy ended.
/// Why:
///   Four endings need four different next steps:
/// Git removes it (nothing changed),
/// the work is done,
/// the wrapper refuses
///  because something beside the worktree would be left behind by a removal,
/// or the files are gone and the registration
///  is not,
/// which only the person can finish and which must never reach Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoveOutcome = { kind: 'left-to-git' } | { kind: 'removed' } | { kind: 'refused'; report: string } | { kind: 'incomplete'; report: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoveOutcome {
    /// Nothing was changed;
    /// real Git removes the worktree or states its refusal.
    LeftToGit,
    /// The subvolume and the administrative directory are gone.
    Removed,
    /// The worktree and its registration are gone,
    /// and what someone else put inside it stands where it was,
    /// with the directories that lead to it.
    ///
    /// The report is for standard error and says what is left and what frees the path.
    RemovedAround(Vec<Boundary>),
    /// The subvolume of the worktree was destroyed by this command,
    /// because another command took the hand-over first,
    /// and the administrative directory is gone.
    ///
    /// The ending is not `Removed`:
    /// every file of the worktree is gone when the command returns,
    /// and nothing waits for the commit
    ///  service,
    /// which `Removed` promises.
    DestroyedInPlace,
    /// The worktree and its registration are gone,
    /// and the volume the removal went around went away while its pass ran,
    /// so the empty root went with one `rmdir`:
    /// nothing was destroyed in one step and nothing waits for the commit service.
    ///
    /// `Removed` cannot stand for this,
    /// because the command that prints a line for `Removed` promises a delete the commit
    ///  service still has to make,
    /// and on this path there is none.
    RemovedAroundNothingLeft,
    /// The removal is refused and nothing was changed;
    /// the report is for standard error and says what stands in the way and how to clear it.
    Refused(String),
    /// The subvolume was destroyed but the registration could not be removed;
    /// the report is for standard error.
    Incomplete(String),
}
