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
//! The pool is taken out of its own name with one rename before anything of it is removed:
//! a commit claims a member by renaming it out
//!  of that name,
//! and a deletion goes on unlinking through the directory it already opened,
//! so a claim that came
//!  after the deletion started lost the ref store the commit had just written there
//! (`doc/troubleshooting/remove-dir-all-empties-a-directory-renamed-out-of-its-tree.md`).
//! A claim that comes after the rename finds no pool and makes
//!  its own shadow,
//! and one that came before it took its member out of the tree the deletion was given.
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

/// The prefix of the name a pool takes while it is being removed.
/// Why:
///   A pool is read by its worktree's key and by nothing else,
/// so a name no key has takes the tree out of every reader's way
///  with one rename,
/// while a removal that outlives a write keeps naming the tree it was given.
pub(crate) const REMOVING_PREFIX: &str = "removing-";

/// What:
///  The deletion a pool removal runs,
/// over the name the pool now has.
/// Why:
///   The pool is deleted under a name nothing else reads,
/// and what a commit claiming a member at the moment the
///  deletion starts comes to is what a control has to be able to see,
/// so the deletion is the one step of this removal a
///  control replaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoveAPool = (taken: string) => void; // throws the reason
/// ```
pub(crate) type RemoveAPool<'a> = dyn Fn(&Path) -> std::io::Result<()> + 'a;

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
    clear_the_pool_of_a_removed_worktree_with(common_dir, git_dir, &remove_tree);
}

/// What:
///  The same removal with the deletion of the tree given,
/// which a control replaces to see what a commit claiming a member at that instant comes to.
/// Why:
///   A claim is one rename of a member out of the pool,
/// and the deletion below would go on unlinking through the
///  member's own directory handle;
/// a claim that happens exactly then loses the ref store a commit just wrote,
/// and
///  its read back fails,
/// so the commit reports a failure real Git did not have.
/// The deletion is therefore never given the name a
///  claim reads:
/// the pool is taken out of that name first,
/// and a claim that comes after it finds no pool at
///  all and makes its own shadow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearThePoolOfARemovedWorktreeWith({ commonDir, gitDir, remove });
/// ```
pub(crate) fn clear_the_pool_of_a_removed_worktree_with(common_dir: &Path, git_dir: &Path, remove: &RemoveAPool<'_>) {
    if git_dir == common_dir {
        debug(TAG, format!("{} is the main checkout, whose pool no removal takes", git_dir.display()).as_str());
        return;
    }
    if !worktree_is_gone(git_dir) {
        debug(TAG, format!("{} is still there, so its shadow pool stays", git_dir.display()).as_str());
        return;
    }
    let pool: PathBuf = pool_root_for(common_dir, git_dir);
    // The path is the wrapper's own or the removal takes nothing:
    // the tree this finds is renamed out of its name and deleted,
    // so a link there would delete whatever tree it names.
    if !crate::shadow_pool::the_pool_path_is_the_wrappers_own(pool.as_path()) {
        debug(
            TAG,
            format!("{} is not a folder this wrapper's pool can be, so nothing of it is removed", pool.display()).as_str(),
        );
        return;
    }
    let taken: PathBuf = match take_the_pool_out_of_its_name(pool.as_path(), REMOVING_PREFIX) {
        Ok(Some(taken)) => taken,
        // The pool is not there any more, so nothing of it is left to remove.
        Ok(None) => return,
        Err(reason) => {
            warn(TAG, format!("{reason}; the shadow pool {} stays and keeps its disk", pool.display()).as_str());
            return;
        }
    };
    match remove(taken.as_path()) {
        Ok(()) => debug(TAG, format!("removed the shadow pool {}", taken.display()).as_str()),
        Err(error) => warn(TAG, format!("the shadow pool {} of the removed worktree {} could not be removed ({error}); it stays and keeps its disk", taken.display(), git_dir.display()).as_str()),
    }
}

/// What:
///  Move the pool `pool` to a name that starts with `prefix`,
/// which no reader of that pool has,
/// and answer that name,
/// or nothing when the pool is not there.
/// Why:
///   One rename is what makes the deletion safe against a claim:
/// a claim renames a member out of the pool by the
///  name the pool had,
/// so a member is either taken before the rename,
/// and then it is not below the tree the deletion
///  was given,
/// or the pool's name is gone and the claim finds nothing to take and makes its own shadow.
/// The reason is
///  the error for a pool that cannot be taken out of its name.
///  Both pools of this repository are taken out of their names this way before anything of them is removed,
/// so the
///  rename and the words for it live here once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = takeThePoolOutOfItsName(pool, 'removing-'); // string | undefined
/// ```
pub(crate) fn take_the_pool_out_of_its_name(pool: &Path, prefix: &str) -> Result<Option<PathBuf>, String> {
    let Some(folder) = pool.parent() else { return Err(format!("{} names no folder", pool.display())) };
    // The name is the folder's and the tree's: a tree with no name of its own was never a pool.
    if pool.file_name().is_none() {
        return Err(format!("{} names no pool", pool.display()));
    }
    let taken: PathBuf = folder.join(format!("{prefix}{}", crate::random_id::random_uuid().unwrap_or_default()));
    return match std::fs::rename(pool, taken.as_path()) {
        Ok(()) => Ok(Some(taken)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("the pool {} cannot be taken out of its name ({error})", pool.display())),
    };
}

/// Pool removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_clears_the_pool_tests.rs"]
mod tests;
