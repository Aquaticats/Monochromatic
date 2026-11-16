//! What:
//!  The question the serving loop asks about the worktree whose units are its own:
//! is it still there.
//! Why:
//!   A removal Git ran with the wrapper replaced and a worktree deleted without Git leave the socket unit listening,
//! and the service that serves it is already running,
//! so no start happens for a gone worktree and nothing would ever ask.
//! The answer is the same rule the sweep and `doctor` use (`worktree_is_gone`),
//! so an unmounted device and an instance nothing names keep the service serving.
//! It sits in a file of its own because the serving loop is one of the package's longest files,
//! and this question is the one thing in it that is about removal and not about serving.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (ownWorktreeGoneNow(settings, untilIdle)) stopOwnUnitsAndEnd();
//! ```

use crate::service_server::ServeSettings;
use std::time::Duration;

/// What:
///  Whether this service should stop serving because the worktree whose units are its own is gone.
/// Why:
///   Every wake of a service whose worktree is known asks this:
/// the removal that takes a worktree while its service runs is what leaves a socket listening for a directory that is gone.
/// The answer is the same rule the sweep and `doctor` use (`worktree_is_gone`),
/// so an unmounted device and an instance nothing names keep the service serving.
/// The question is asked before the busy count and before the idle bound:
/// after-commit work that counts itself busy,
/// and a service that has served nothing for hours,
/// both answer it the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (ownWorktreeGoneNow(settings, untilIdle)) stopOwnUnitsAndEnd();
/// ```
pub fn own_worktree_gone_now(settings: &ServeSettings, until_idle: Duration) -> bool {
    if settings.own_git_dir.is_none() || until_idle <= settings.own_worktree_check {
        return false;
    }
    return settings.own_git_dir.as_deref().is_some_and(crate::service_units_of_removed_worktrees::worktree_is_gone);
}
