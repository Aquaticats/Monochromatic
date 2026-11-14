//! What:
//!  One subvolume this program made,
//!  moved out of a tree that is being removed and recorded for the commit service to
//!  delete,
//!  so the person's command never waits for a kernel destroy.
//! Why:
//!   The btrfs destroy takes the filesystem's write lock against every snapshot in flight
//! (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`),
//! so a
//!  removal that made it waited for another add's snapshot to commit.
//! A removal therefore leaves every subvolume it owns to the
//!  service:
//! the worktree's own (`worktree_remove_hand_over.rs`),
//! a pool of snapshots ahead
//!  (`worktree_removal_clears_the_ahead_pool.rs`),
//! and a pending deletion this program wrote earlier
//!  (`worktree_removal_takes_its_pending_deletions.rs`).
//! Each one moves with one `rename(2)` on the filesystem it stands on,
//! and two records are written before the move:
//!  the work item of its delete,
//!  and the leaving record that outlives every queue record for the same place.
//! A command killed between the two writes and the move leaves a record naming a place that does not exist yet
//!  (which deletes nothing) and a leaving a later command moves or names
//!  (`worktree_removal_leaving_that_outlives_its_record.rs`).
//! A record that cannot be written leaves the subvolume where it is and
//!  answers the reason,
//! so the caller refuses while nothing has moved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await leaveASubvolumeForTheService({ commonDir, from, into }); // its new path
//! ```

use crate::diagnostic_log::debug;
use crate::fs_btrfs_ioctl::subvolume_id;
use crate::private_storage::create_private_directory_all;
use crate::service_work::work_directory;
use crate::work_item::{WorkItem, WorkKind, enqueue};
use crate::worktree_removal_leaving_that_outlives_its_record::{forget_the_leaving, record_a_leaving};
use crate::iso_time::now_iso;
use std::ffi::OsString;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalLeavesASubvolume";

/// What a renamed subvolume's name starts with while the service deletes it:
/// a name no person gives a worktree,
/// so it is never mistaken for one,
/// and one the next commands report when it stays.
pub const DELETING_PREFIX: &str = "cli-git-deleting-";

/// The reference an item of this kind names,
/// which is no branch:
/// an item carries a reference-shaped field so that every reader of a record keeps one shape,
/// and no push is ever made for this kind.
pub const DELETE_REFERENCE: &str = "refs/worktree/cli-git-service/subvolume-delete";


/// What:
///  The name a removed worktree's subvolume takes while it waits for the service's delete.
/// Why:
///   The subvolume id names one subvolume for good,
/// while a name names whatever is there when the call runs,
/// so a name built
///  from the id can never stand for another person's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deletingName = (id: bigint) => `cli-git-deleting-${id.toString(16).padStart(16, '0')}`;
/// ```
pub fn deleting_name(id: u64) -> OsString {
    return OsString::from(format!("{DELETING_PREFIX}{id:016x}"));
}

/// What:
///  The item that names one renamed subvolume for the service.
/// Why:
///   The record is the only thing that outlives the command,
/// so it names the exact path the rename made and the subvolume id it
///  holds:
/// a delete acts on that path alone,
/// and a path that is gone is nothing to delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removalItem = ({ commonDir, parentPath, id, deleting }) => ({ kind: 'subvolume-delete', repository: parentPath, gitDir: commonDir, ref: DELETE_REFERENCE, oid: hex40(id), parent: join(parentPath, deleting), created: now, tries: 0 });
/// ```
pub fn removal_item(common_dir: &Path, parent_path: &Path, id: u64) -> WorkItem {
    let deleting: PathBuf = parent_path.join(deleting_name(id));
    return WorkItem {
        kind: WorkKind::SubvolumeDelete,
        repository: parent_path.to_path_buf(),
        git_dir: common_dir.to_path_buf(),
        branch_ref: String::from(DELETE_REFERENCE),
        oid: format!("{id:040x}"),
        parent: deleting.display().to_string(),
        created: now_iso(),
        tries: 0,
    };
}


/// What:
///  Leave one subvolume this program made for the commit service to delete,
///  by writing the record that names the place it will stand,
///  moving it out of the tree that is being removed,
///  and answering that place.
/// Why:
///   The removal's own subvolume is not the only one it leaves:
/// a pool of snapshots ahead of an add and a
///  pending deletion this program wrote earlier are subvolumes inside the tree as well,
/// and the kernel refuses
///  to destroy a tree that holds one.
/// Moving one out is one `rename(2)` on the filesystem it stands on,
///  so nothing waits for the kernel's write lock the way a destroy does
/// (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`),
/// and the record written before the move names
///  what the service deletes later
/// (a review finding,
/// F5).
///  The folder it is moved into is on the same filesystem by construction:
/// a pool sits in the repository's Git
///  directory,
/// and a pending deletion sits in the tree its own removal renamed it out of.
/// A record that
///  cannot be written leaves the subvolume where it is and answers the reason,
/// so the caller refuses while
///  nothing has moved.
///  Two records are written before the move,
/// and they are not the same work:
/// the work item is settled by the first round of the service that looks at it,
/// and a round that looked before the move settles a delete of a path that is not there yet,
/// while the leaving record is settled by nothing and names the directory until a command proves it gone
///  (`worktree_removal_leaving_that_outlives_its_record.rs`,
/// `worktree_removal_leavings_a_command_queues_again.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = await leaveASubvolumeForTheService({ commonDir, from, into }); // its new path
/// ```
pub(crate) fn leave_a_subvolume_for_the_service(common_dir: &Path, from: &Path, into: &Path) -> Result<PathBuf, String> {
    // The entry was read with `symlink_metadata` by the caller, and the paths a removal builds start at a held directory
    // (`/proc/self/fd/<n>`), which `open_root` refuses as reached through a link. A link that takes the entry's place after
    // that read is no harm here: the rename moves the link and never its target, and the service deletes a path only while
    // a directory stands there.
    let held: File = match File::open(from) {
        Ok(held) => held,
        Err(error) => return Err(format!("the subvolume {} cannot be opened ({error})", from.display())),
    };
    let id: u64 = match subvolume_id(&held, from) {
        Ok(found) => found,
        Err(error) => return Err(format!("the subvolume {} cannot be identified ({error})", from.display())),
    };
    if let Err(error) = create_private_directory_all(into) {
        return Err(format!("the folder {} cannot be made ({error})", into.display()));
    }
    // Both records go before the move, so a command killed between them leaves a record whose path is not there yet
    // (which deletes nothing) and a leaving record a later command finishes the move by
    // (`worktree_removal_leavings_a_command_queues_again.rs`).
    let item: WorkItem = removal_item(common_dir, into, id);
    if let Err(reason) = record_a_leaving(common_dir, &item) {
        return Err(format!("the delete of {} cannot be recorded: {reason}", from.display()));
    }
    if let Err(reason) = enqueue(work_directory(common_dir).as_path(), &item) {
        // Nothing has moved: the leaving record names a place the move never makes.
        forget_the_leaving(common_dir, item.id().as_str());
        return Err(format!("the delete of {} cannot be recorded: {reason}", from.display()));
    }
    let left: PathBuf = PathBuf::from(item.parent.as_str());
    if let Err(error) = std::fs::rename(from, left.as_path()) {
        forget_the_leaving(common_dir, item.id().as_str());
        return Err(format!("the subvolume {} cannot be moved to {} ({error})", from.display(), left.display()));
    }
    debug(TAG, format!("the subvolume {} waits for the service's delete at {}", from.display(), left.display()).as_str());
    return Ok(left);
}
