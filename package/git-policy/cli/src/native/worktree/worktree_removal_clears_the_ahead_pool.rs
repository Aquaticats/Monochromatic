//! What:
//!  The snapshots the wrapper made ahead of an add,
//! taken out of the administrative directory of a worktree that is being removed,
//! before anything irreversible happens to that worktree.
//! Why:
//!  A worktree that was the source of one `git worktree add` holds
//! `<admin>/monochromatic-git-policy-cli/ahead`,
//!  a pool whose members are subvolumes of that worktree (`worktree_snapshot_ahead.rs`).
//! A removal destroys the
//!  worktree's own subvolume in one step and empties the administrative directory after it,
//! and that emptying refuses
//!  a nested subvolume,
//! because a tree that holds one holds data that is not part of the tree being removed
//!  (`subvolume_remove_walk.rs`).
//! So the pool was the one thing that made a finished removal end as "the files are
//!  gone and the registration is not",
//! with the whole pool left to `git worktree prune` (a Sonnet finding,
//! F1),
//! and the sweep after
//!  a merge reported that state as a worktree left in place.
//! Nothing else takes the pool:
//! the sweep of the repository scratch directory cannot reach a folder inside a worktree's
//!  administrative directory,
//! and a fill names only a worktree that still exists.
//! The pool is the wrapper's own cache of the worktree being
//!  removed,
//! exactly as the shadow pool of a removed worktree is (`worktree_removal_clears_the_pool.rs`),
//! and an add that
//!  finds no pool makes its own copy,
//! so nothing an add needs is lost with it.
//! The pool is taken out of its own name with one
//!  rename before anything of it is removed:
//! a fill claims a member by renaming it out of that name,
//! so a member is either
//!  taken before the rename,
//! and then it is not below the tree the deletion was given,
//! or the pool's name is gone and the fill
//!  makes its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! clearTheSnapshotsAhead(admin); // throws the reason nothing was removed
//! ```

use crate::fs_btrfs_ioctl::subvolume_destroy_at;
use crate::fs_kind::is_subvolume_root;
use crate::private_storage::remove_tree;
use crate::worktree_removal_clears_the_pool::{REMOVING_PREFIX, RemoveAPool, take_the_pool_out_of_its_name};
use crate::worktree_snapshot_ahead::pool_folder;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// What:
///  The one call that removes one member of the pool,
/// behind a stand-in.
/// Why:
///   A member is a subvolume,
/// which no plain directory of a control is,
/// and a control has to see what a member that cannot go
///  comes to without a btrfs filesystem underneath it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DestroyASnapshot = (member: string) => void; // throws the reason it did not
/// ```
pub(crate) type DestroyASnapshot<'a> = dyn Fn(&Path) -> std::io::Result<()> + 'a;

/// What:
///  Remove every snapshot the wrapper made ahead of an add for the worktree whose administrative directory is `admin`,
/// and
///  answer the reason when one cannot go.
/// Why:
///   The removal is a refusal when this fails,
/// so it must run while the worktree is still there,
/// and the
///  answer is the words for that refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearTheSnapshotsAhead(admin); // throws the reason
/// ```
pub(crate) fn clear_the_snapshots_ahead(admin: &Path) -> Result<(), String> {
    let taken: Option<PathBuf> = take_the_pool_out_of_its_name(pool_folder(admin).as_path(), REMOVING_PREFIX)?;
    let Some(moved) = taken else { return Ok(()) };
    return clear_the_snapshots_ahead_with(moved.as_path(), &destroy_a_snapshot_ahead, &remove_tree);
}

/// What:
///  Destroy one member of the pool,
/// and remove one that is no subvolume at all.
/// Why:
///   Every member the service makes is a subvolume,
/// and only the wrapper writes here,
/// so an entry that is not one is a name
///  nothing else reads and it goes as a plain tree;
/// a member another process claimed first is gone,
/// which is the same
///  end state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// destroyASnapshotAhead(member); // throws the reason
/// ```
fn destroy_a_snapshot_ahead(member: &Path) -> std::io::Result<()> {
    if !is_subvolume_root(member).unwrap_or(false) {
        return remove_tree(member);
    }
    return match subvolume_destroy_at(member) {
        Ok(()) => Ok(()),
        Err(error) => Err(std::io::Error::from_raw_os_error(error.errno)),
    };
}

/// What:
///  Remove every member of the pool `taken`,
/// then the folder itself,
/// with the two calls a control replaces.
/// Why:
///   The deletion is given a name no fill reads,
/// so a member that appears afterwards is made in a pool of its own,
/// and
///  one that cannot go is the reason the whole removal refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearTheSnapshotsAheadWith(taken, destroy, remove); // throws the reason
/// ```
pub(crate) fn clear_the_snapshots_ahead_with(taken: &Path, destroy: &DestroyASnapshot<'_>, remove: &RemoveAPool<'_>) -> Result<(), String> {
    let listing = match std::fs::read_dir(taken) {
        Ok(listing) => listing,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("the folder {taken} of the snapshots the wrapper made ahead of an add cannot be read ({error})", taken = taken.display())),
    };
    for found in listing {
        let member: PathBuf = match found {
            Ok(entry) => entry.path(),
            Err(error) => return Err(format!("the folder {taken} of the snapshots the wrapper made ahead of an add cannot be read ({error})", taken = taken.display())),
        };
        match destroy(member.as_path()) {
            Ok(()) => {}
            // Another process claimed the member between the listing and the destroy: it is not in the pool any more.
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "the snapshot {member} the wrapper made ahead of an add cannot be removed ({error})",
                    member = member.display()
                ));
            }
        }
    }
    return match remove(taken) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("the folder {taken} of the snapshots the wrapper made ahead of an add cannot be removed ({error})", taken = taken.display())),
    };
}

/// Controls of the clear stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_clears_the_ahead_pool_tests.rs"]
mod tests;
