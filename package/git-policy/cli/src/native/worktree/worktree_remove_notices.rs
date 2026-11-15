//! What:
//!  The four lines a removal that destroyed a subvolume prints,
//!  and the one place that chooses between them.
//! Why:
//!  Two things about a destroyed subvolume's files are the person's to know:
//!  whether the commit service still has to
//!  delete them,
//!  or whether they are already gone because this command made the destroy,
//!  and whether anything was checked for
//!  changes first.
//! A line that names a queued delete where there is none sends a person looking for a delete
//!  nobody will make,
//! and one that hides a delete that is queued leaves them hunting for files that are still
//!  on disk.
//! The four lines and the choice between them live here,
//!  apart from the removal's own steps,
//!  because they are one subject and the removal's file reached the
//!  repository's length limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notice = destroyNotice({ force, handedOver }); // one of four lines
//! ```

/// What:
///  The line the command prints on standard error whenever a removed worktree was a btrfs subvolume.
/// Why:
///  The subvolume leaves the person's path in one step that cannot be undone,
/// and its files are deleted by the commit
///  service after the command returned,
/// so a person who looks for the files has to be told which of the two happened and
///  why nothing of theirs was lost:
/// what went is what another process wrote or locked in the moments since the last check,
/// which is the same risk
///  `git worktree remove` takes.
pub const DESTROY_NOTICE: &str = "monochromatic-git-policy-cli: removed the btrfs subvolume of the worktree in one step, and the commit service deletes the files after this command returns, in its own round (a later command of the repository wakes it); this cannot be undone, and anything another process wrote or locked in the moments since the last check went with it (the risk of git worktree remove, here faster).";

/// What:
///  The line the command prints on standard error when a removed worktree was a btrfs subvolume and the person forced the removal.
/// Why:
///   `--force` is the one form that removes a worktree holding changes,
/// and the destroy cannot be undone,
/// so the
///  line has to say that nothing was checked for changes before it,
///  and the files leave later than the path does,
/// because the same commit service the unforced line names deletes them after this command returns.
pub const DESTROY_FORCED_NOTICE: &str = "monochromatic-git-policy-cli: removed the btrfs subvolume of the forced removal in one step, without checking it for changes first, as `--force` asks, and the commit service deletes the files after this command returns, in its own round (a later command of the repository wakes it); this cannot be undone, and everything the worktree held went with it, which is the set `git worktree remove --force` deletes (the risk of git worktree remove, here faster).";

/// What:
///  The line the command prints on standard error when it destroyed the subvolume here,
///  because another command had taken the hand-over first.
/// Why:
///  The hand-over's line says the commit service deletes the files after the command returns,
/// and on this
///  path there is nothing for the service to delete:
/// the kernel call ran in this command,
/// so every file of the worktree is
///  gone when the command returns.
/// A person looking for the files is told which of the two happened.
pub const DESTROY_HERE_NOTICE: &str = "monochromatic-git-policy-cli: removed the btrfs subvolume of the worktree in one step, here, and every file of it is gone now, so the commit service has nothing left to delete for it; this cannot be undone, and anything another process wrote or locked in the moments since the last check went with it (the risk of git worktree remove, here faster).";

/// What:
///  The same line for a removal the person forced,
///  which checked nothing for changes first.
/// Why:
///   `--force` is the one form that removes a worktree holding changes,
/// and a destroy made here cannot be undone,
/// so the line
///  says that nothing was checked and that everything the worktree held is gone now.
pub const DESTROY_FORCED_HERE_NOTICE: &str = "monochromatic-git-policy-cli: removed the btrfs subvolume of the forced removal in one step, here, without checking it for changes first, as `--force` asks, and every file of it is gone now, so the commit service has nothing left to delete for it; this cannot be undone, and everything the worktree held went with it, which is the set `git worktree remove --force` deletes (the risk of git worktree remove, here faster).";

/// What:
///  The line the command prints for one removal that destroyed a subvolume,
///  chosen by where the destroy ran and whether the person forced it.
/// Why:
///  A command that prints the wrong one of these four promises a delete that is not queued,
/// or hides one that is;
/// the choice is one place so no caller can word it differently,
/// and a control can read all four.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notice = destroyNotice({ force, where: 'here' | 'handed-over' });
/// ```
pub fn destroy_notice(force: bool, handed_over: bool) -> &'static str {
    return match (handed_over, force) {
        (true, true) => DESTROY_FORCED_NOTICE,
        (true, false) => DESTROY_NOTICE,
        (false, true) => DESTROY_FORCED_HERE_NOTICE,
        (false, false) => DESTROY_HERE_NOTICE,
    };
}
