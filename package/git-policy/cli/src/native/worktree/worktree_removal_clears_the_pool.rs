//! What:
//!  Taking the shadow pool of a worktree with the removal of that worktree:
//! the folder the resident service filled for it,
//! and
//!  the proof that the worktree is gone before anything of it is removed.
//! Why:
//!  A member of the pool names its worktree's root in its config,
//! so the pool of a worktree is that worktree's own
//! (`shadow_pool.rs`),
//!  and it sits in the Git common directory,
//! which a removal does not delete.
//! Nothing else looks at it:
//! the sweep of the repository scratch directory cannot reach a folder beside the scratch directory,
//! and a fill
//!  names only a worktree that still exists.
//! A pool left behind therefore holds up to `POOL_LIMIT` finished
//!  trees,
//! with the links of every one,
//! for a worktree no command can name again,
//! so a repository that makes and removes worktrees
//!  accumulates them without bound.
//! The proof that the worktree is gone is the one the stop of its units uses:
//! the administrative directory
//!  is missing while the place that held it is there,
//! so a removal Git refused,
//! or a worktree on a drive that is not
//!  mounted,
//! keeps its pool.
//! A pool that cannot be removed is logged and never fails the removal the person asked for.
//! A fill that is still writing into the pool when it goes fails and logs its reason,
//! and a commit that finds an empty pool makes its own creation,
//! so nothing a commit needs is lost with it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! clearThePoolOfARemovedWorktree({ commonDir, gitDir }); // never throws
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::private_storage::remove_tree;
use crate::service_units_of_removed_worktrees::worktree_is_gone;
use crate::shadow_pool::pool_root_for;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalClearsThePool";

/// What:
///  Remove the shadow pool of a worktree whose removal has ended,
/// when the administrative directory of that worktree is certainly gone.
/// Why:
///   The name of a worktree's pool is its administrative directory's,
/// so the pool is this command's to take exactly when that directory is gone;
/// a worktree that is still there,
/// or one whose repository is on a device that is not mounted,
/// keeps its pool.
/// The main checkout's pool is never taken:
/// no removal removes the main checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearThePoolOfARemovedWorktree({ commonDir, gitDir });
/// ```
pub fn clear_the_pool_of_a_removed_worktree(common_dir: &Path, git_dir: &Path) {
    if git_dir == common_dir {
        debug(TAG, format!("{} is the main checkout, whose pool no removal takes", git_dir.display()).as_str());
        return;
    }
    if !worktree_is_gone(git_dir) {
        debug(TAG, format!("{} is still there, so its shadow pool stays", git_dir.display()).as_str());
        return;
    }
    let pool: PathBuf = pool_root_for(common_dir, git_dir);
    match remove_tree(pool.as_path()) {
        Ok(()) => debug(TAG, format!("removed the shadow pool {}", pool.display()).as_str()),
        Err(error) => warn(TAG, format!("the shadow pool {} of the removed worktree {} could not be removed ({error}); it stays and keeps its disk", pool.display(), git_dir.display()).as_str()),
    }
}

/// Pool removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_clears_the_pool_tests.rs"]
mod tests;
