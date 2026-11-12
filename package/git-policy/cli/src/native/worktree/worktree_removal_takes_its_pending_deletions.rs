//! What:
//!  The subvolumes this program left inside a worktree for its commit service to delete,
//!  moved out of that worktree before its removal hands anything to the kernel or to Git,
//!  and the volume that is nobody's but the person's, which the removal refuses.
//! Why:
//!  A removal of a subvolume worktree hands the worktree's own subvolume to the commit service by renaming it
//!  inside the directory it holds (`worktree_remove_hand_over.rs`),
//!  and when that worktree stood inside another one,
//!  the renamed subvolume stands inside the outer worktree.
//! That is the state on 2026-11-11T22:30:57-05:00,
//!  when a nested worktree was removed and the removal of the outer one,
//!  minutes later,
//!  found `cli-git-deleting-000000000001f5f7` below its own root.
//! The destroy of the outer subvolume cannot go through a
//!  tree that holds another subvolume (`ENOTEMPTY`),
//! so the outer removal refused,
//!  and the person was told to delete this program's own leftover with the btrfs tool.
//!  What stands at such a name is not the person's:
//! the removal that renamed it proved the worktree clean and unlocked,
//!  and the record of the delete outlives the command,
//!  so the outer removal moves it out of the tree as well,
//!  with a record of its own,
//!  and goes on.
//! Nothing here deletes a subvolume:
//! a rename is bounded,
//!  and the destroy is the service's work
//! (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
//!  A name is no proof of what stands at it,
//! so an entry that has the shape of this program's name and is no subvolume
//!  is somebody else's and is refused with every other volume below the root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const below = await takeThePendingDeletionsBelow({ commonDir, held, target }); // clear | foreign | unreadable
//! ```

use crate::fs_kind::is_subvolume_root;
use crate::subvolume_remove_walk::{Boundary, find_boundary_below};
use crate::worktree_removal_holds_the_worktree::Held;
use crate::worktree_remove_hand_over::{DELETING_PREFIX, leave_a_subvolume_for_the_service};
use std::path::Path;

/// The number of hexadecimal digits the subvolume id in a leaving name takes.
/// Why:
///   The name is `cli-git-deleting-` and the id in sixteen hexadecimal digits
/// (`deleting_name`),
/// so a name of that shape is one this program made and no other directory is.
const IDENTIFIER_DIGITS: usize = 16;

/// What:
///  What one walk below a worktree's root found once every subvolume this program left for the service is out of
///  the way.
/// Why:
///   Three answers need three next steps:
/// the removal goes on,
/// it refuses while nothing has
///  happened,
/// or the tree proves nothing and the removal is Git's,
/// which is what it was before the proof
///  existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type VolumesBelow = { kind: 'clear' } | { kind: 'foreign'; found: Boundary } | { kind: 'unreadable'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VolumesBelow {
    /// Nothing below the root but what this program left there,
    /// which is now out of the tree.
    Clear,
    /// A subvolume or a mount point this program did not leave.
    Foreign(Boundary),
    /// A tree that cannot be read for what stands below it.
    Unreadable(String),
}

/// What:
///  Whether this program wrote the name `path` has while it left a subvolume for the service to delete.
/// Why:
///   The name is the only thing a walk reads before it decides whether to open an entry,
/// and a name of the
///  shape `cli-git-deleting-<16 lowercase hexadecimal digits>` is one nothing but `deleting_name` makes
///  (`{id:016x}`).
/// An upper-case digit is no part of that shape, and a directory somebody named with one is theirs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mine = /^cli-git-deleting-[0-9a-f]{16}$/.test(name);
/// ```
fn is_a_leaving_name(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| { return name.to_str() }) else { return false };
    let Some(identifier) = name.strip_prefix(DELETING_PREFIX) else { return false };
    return identifier.len() == IDENTIFIER_DIGITS && identifier.bytes().all(|byte| { return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte) });
}

/// What:
///  Move every subvolume this program left below `target` out of that tree,
///  and answer what else stands below it,
///  or the reason a leftover could not be moved.
/// Why:
///   A nested worktree's removal leaves its subvolume inside the outer worktree,
/// and the outer removal
///  cannot destroy a tree that holds it;
/// the leftover is this program's own,
/// so it goes to the service like the
///  worktree's own subvolume does,
/// and the person is never told to delete it by hand
/// (a reported case,
///  2026-11-11T22:31:17-05:00).
///  The walk answers the first boundary it meets,
/// so this looks again after each move:
/// a tree with several of
///  them is a tree whose last one would otherwise stop the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const below = await takeThePendingDeletionsBelow({ commonDir, held, target });
/// ```
///
/// # Errors
///
/// The words for a leftover that cannot be recorded or moved,
/// which is the reason the removal refuses while
///  nothing of the worktree has changed.
pub(crate) fn take_the_pending_deletions_below(common_dir: &Path, held: &Held, target: &Path) -> Result<VolumesBelow, String> {
    loop {
        let found: Boundary = match find_boundary_below(&held.target, target) {
            Ok(None) => return Ok(VolumesBelow::Clear),
            Ok(Some(found)) => found,
            Err(error) => return Ok(VolumesBelow::Unreadable(format!("the tree of {} cannot be walked for a subvolume of its own ({error})", target.display()))),
        };
        // What stands at the name is asked of the entry itself: a directory this program renamed is a subvolume,
        // and anything else at that name is somebody else's.
        if !is_a_leaving_name(found.path.as_path()) || !is_subvolume_root(found.path.as_path()).unwrap_or(false) {
            return Ok(VolumesBelow::Foreign(found));
        }
        leave_a_subvolume_for_the_service(common_dir, found.path.as_path(), held.parent_path.as_path())?;
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_takes_its_pending_deletions_tests.rs"]
mod tests;
