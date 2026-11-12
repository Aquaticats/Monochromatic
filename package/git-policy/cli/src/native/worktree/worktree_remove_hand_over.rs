//! What:
//!  The removal of a worktree's subvolume handed to the commit service,
//! and the service's own delete of it.
//! Why:
//!  A subvolume destroy is one kernel call,
//! but it takes `down_write(&fs_info->subvol_sem)` and joins the running
//!  transaction (`btrfs_delete_subvolume`,
//! `fs/btrfs/inode.c:4801` and `:4848` at `v7.2.15`),
//! so it waits for whatever snapshot another add is committing and for the transaction that is running,
//!  and the cleaner is busy behind it
//! (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
//! A person removing a worktree waits for none of that:
//! the worktree's
//!  directory is renamed out of the way inside its own parent in one `rename(2)`,
//! a durable work item names the renamed
//!  subvolume,
//! and the service deletes it later.
//! The rename is what makes the person's view right at once:
//! the path `git worktree remove` was given no longer exists,
//! and every file of it is still under a name this wrapper chose.
//! Nothing is lost:
//! the worktree was proven clean and unlocked before either step,
//! and the rename is inside the directory the
//!  removal already holds open.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await handOverRemoval({ context, commonDir, parent, parentPath, name, id, target }); // rename, record, wake
//! const verdict = deleteSubvolumeOf(item); // the service's own delete
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::{identity_at, open_root, rename_at};
use crate::fs_btrfs_ioctl::{snapshot_destroy_by_id, snapshot_destroy_in, subvolume_id};
use crate::iso_time::now_iso;
use crate::private_storage::create_private_directory_all;
use crate::service_client::{Activation, ClientConfig, manager_watch_for, wake};
use crate::service_messages::render;
use crate::service_protocol::HEARTBEAT_SILENCE;
use crate::service_provision::provision;
use crate::service_provision_host_and_manager::units_record_dir_of;
use crate::service_route::{Route, route_from_environment};
use crate::service_work::work_directory;
use crate::transaction_git::GitContext;
use crate::wait_ceilings::{ANSWER_CEILING, HAND_OVER_GREETING_WAIT, SHORT_EXTRA_WAIT, WITHDRAW_CEILING, waiting_notice_after};
use crate::work_item::{PENDING, WorkItem, WorkKind, enqueue, state_directory};
use crate::worktree_removal_interrupted_destroy::record_leaving;
use crate::service_work::Verdict;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemoveHandOver";

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
///  Write the item that will delete the renamed subvolume,
/// so that a kill after this point leaves it named.
/// Why:
///   The record is written before the rename,
/// so the two orders a kill can leave are both safe:
/// a record whose path
///  does not exist deletes nothing,
/// and a rename whose record is missing would leave a directory nothing names.
/// Writing
///  it twice is one record:
/// the queue's id is the subvolume's,
/// so a second write finds the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// enqueue(join(workDir(commonDir), 'pending'), removalItem({ commonDir, parentPath, id }))
/// ```
fn record_the_removal(common_dir: &Path, parent_path: &Path, id: u64) -> Result<WorkItem, String> {
    let item: WorkItem = removal_item(common_dir, parent_path, id);
    return match enqueue(work_directory(common_dir).as_path(), &item) {
        Ok(_added) => Ok(item),
        Err(reason) => Err(reason),
    };
}

/// What:
///  Wake the service of the repository whose common directory is `common_dir`,
/// saying nothing of its own,
/// as an add wakes it for its trash.
/// Why:
///   A wake that cannot be made leaves the record for the next command of the repository,
/// which is every commit
///  and every add,
/// so the delete happening later is not a failure of the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wakeService({ context, commonDir });
/// ```
fn wake_the_service(context: &GitContext, common_dir: &Path) -> Result<(), String> {
    let Some(Route { address, provision: host }) = route_from_environment(context.real_git.as_path(), common_dir) else {
        return Err(String::from("no commit service can be started from this session"));
    };
    if let Some(setup) = host.as_ref()
        && let Err(failure) = provision(setup)
    {
        return Err(render(&failure));
    }
    // What: The record is written and the directory renamed before this wake, so the wake waits the hand-over's own greeting
    //       bound and never the commit's start wait.
    // Why:  A person removing a worktree waits for none of what the service does with the record, and every later command of the
    //       repository wakes the same service (`wait_ceilings.rs`, `HAND_OVER_GREETING_WAIT`).
    let config: ClientConfig = ClientConfig {
        address,
        activation: Activation::Managed,
        start_wait: HAND_OVER_GREETING_WAIT,
        heartbeat_silence: HEARTBEAT_SILENCE,
        start_watch: host.as_ref().map(manager_watch_for),
        keep_fresh: None,
        answer_ceiling: ANSWER_CEILING,
        withdraw_ceiling: WITHDRAW_CEILING,
        waiting_notice_after: waiting_notice_after(),
        extra_wait: SHORT_EXTRA_WAIT,
        start_ceiling: HAND_OVER_GREETING_WAIT,
        units_record_dir: host.as_ref().map(units_record_dir_of),
    };
    let mut silent: std::io::Sink = std::io::sink();
    return match wake(&config, common_dir, None, &mut silent) {
        Ok(()) => Ok(()),
        Err(failure) => Err(render(&failure)),
    };
}

/// What:
///  Hand the subvolume of a proven,
/// unlocked worktree to the service:
/// record the item,
/// rename the subvolume out of the
///  person's way,
/// and wake the service.
/// Why:
///  The rename is the whole of what the person's command does to the files:
/// one call inside the parent directory the removal
///  already holds open,
/// so nothing is deleted on their path and nothing is waited for.
/// Every step before it is the
///  removal's own proof that the worktree is clean and unlocked.
/// A step that fails puts the directory back and answers the
///  reason,
/// and the caller then destroys it in place as before,
/// so a hand-over is never what makes a removal fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverRemoval({ context, commonDir, parent, parentPath, name, id });
/// ```
pub fn hand_over_removal(context: &GitContext, common_dir: &Path, parent: &File, parent_path: &Path, name: &OsStr, id: u64, target: &Path, registration: &OsStr) -> Result<(), String> {
    return hand_over_with(context, common_dir, parent, parent_path, name, id, target, registration, &wake_the_service);
}

/// What:
///  `hand_over_removal` with the wake injected,
/// so a control reaches no service and no session bus.
/// Why:
///   The record is written and the directory renamed before the service is told,
/// and the removal's own record names the place the
///  directory takes before it takes it,
/// so a kill in the moments between leaves a state the next command finishes,
/// so a control can assert both without a
///  service,
/// and a wake that fails is never the removal's failure:
/// the record stays for the next command of the
///  repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith({ context, commonDir, parent, parentPath, name, id, target, told });
/// ```
pub fn hand_over_with(context: &GitContext, common_dir: &Path, parent: &File, parent_path: &Path, name: &OsStr, id: u64, target: &Path, registration: &OsStr, told: &dyn Fn(&GitContext, &Path) -> Result<(), String>) -> Result<(), String> {
    let item: WorkItem = record_the_removal(common_dir, parent_path, id)?;
    let deleting: OsString = deleting_name(id);
    // Where the files will stand is written before the rename, so a command killed between the two leaves a record whose
    // worktree path is gone and whose place is named, which is a state the next command finishes the registration by
    // (a review finding, F4).
    record_leaving(common_dir, registration, parent_path.join(deleting.as_os_str()).as_path());
    if let Err(error) = rename_at(parent, name, parent, deleting.as_os_str(), false) {
        // Nothing was renamed, so the record names a path that does not exist and the service will find nothing to delete.
        return Err(format!("the subvolume of {} cannot be renamed out of the way ({error})", target.display()));
    }
    debug(TAG, format!("the subvolume of {} waits for the service's delete at {}", target.display(), Path::new(&item.parent).display()).as_str());
    if let Err(why) = told(context, common_dir) {
        // The record stays, and the next command of this repository wakes the service that reads it.
        warn(TAG, format!("the delete of {} is recorded and not woken ({why})", target.display()).as_str());
    }
    return Ok(());
}

/// What:
///  Whether metadata the filesystem answered describes a directory.
/// Why:
///  The answer is asked of what was read once,
/// so a path replaced between the two calls cannot make the proof about one object and the delete about another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isADirectory = (found) => found.isDirectory();
/// ```
fn is_a_directory(found: std::fs::Metadata) -> bool {
    return found.is_dir();
}

/// What:
///  Whether a path is a directory this process may remove as a subvolume.
/// Why:
///  A record names a path,
/// and a path can be replaced by a link or by nothing;
/// only a directory that stands exactly there is deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lstat(path).isDirectory()
/// ```
fn is_directory(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok_and(is_a_directory);
}

/// What:
///  The service's own delete of one handed-over subvolume,
/// with the words for what became of it.
/// Why:
///  A path that is gone is nothing to delete:
/// either the rename never happened and the removal destroyed the
///  subvolume in place,
/// or a service already deleted it,
/// so the item is done.
/// A path that is not a directory
///  is not this item's to remove.
/// The destroy names the subvolume by id when the kernel allows it,
/// so a replaced
///  ancestor cannot redirect it,
/// and by name below the opened parent otherwise.
/// A mount that does not allow an unprivileged
///  destroy is a failure the person must hear,
/// because the files stay;
/// anything else may pass and is tried again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deleteSubvolumeOf(item): Verdict
/// ```
pub fn delete_subvolume_of(item: &WorkItem) -> Verdict {
    let path: PathBuf = PathBuf::from(item.parent.as_str());
    if !path.is_absolute() {
        return Verdict::Failed(format!("the record names no absolute path to delete: {}", item.parent));
    }
    let Some(parent_path) = path.parent() else {
        return Verdict::Failed(format!("the record names no directory to delete below: {}", item.parent));
    };
    let Some(name) = path.file_name() else {
        return Verdict::Failed(format!("the record names no directory to delete: {}", item.parent));
    };
    if !is_directory(path.as_path()) {
        debug(TAG, format!("{} is gone already; nothing to delete", path.display()).as_str());
        return Verdict::Done;
    }
    let parent: File = match open_root(parent_path) {
        Ok(opened) => opened,
        Err(error) => return Verdict::Retry(format!("the directory {} cannot be opened ({error})", parent_path.display())),
    };
    // The name still has to lead to the object that was looked at, or the delete would act on whatever took its place.
    match identity_at(&parent, name) {
        Ok(_held) => {}
        Err(error) => return Verdict::Retry(format!("{} cannot be looked up ({error})", path.display())),
    }
    match subvolume_id(&parent, path.as_path()) {
        Ok(id) => match snapshot_destroy_by_id(&parent, id, path.as_path()) {
            Ok(()) => return Verdict::Done,
            Err(error) if error.errno == libc::EPERM => {
                return Verdict::Failed(format!(
                    "{} is still there: this mount does not allow an unprivileged subvolume destroy ({error}), so remove the directory by hand, and remount it with user_subvol_rm_allowed to let the wrapper do it",
                    path.display()
                ));
            }
            Err(error) if by_id_unavailable(error.errno) => {
                debug(TAG, format!("the destroy of {} by id is not available ({error}); destroying it by name", path.display()).as_str());
            }
            Err(error) => return Verdict::Retry(format!("the subvolume of {} cannot be destroyed ({error})", path.display())),
        },
        Err(error) if by_id_unavailable(error.errno) => {
            debug(TAG, format!("the id of the subvolume {} is not available ({error}); destroying it by name", path.display()).as_str());
        }
        Err(error) if error.errno == libc::ENOENT => return Verdict::Done,
        Err(error) => return Verdict::Retry(format!("the subvolume of {} cannot be identified ({error})", path.display())),
    }
    return match snapshot_destroy_in(&parent, name, parent_path) {
        Ok(()) => Verdict::Done,
        Err(error) if error.errno == libc::ENOENT => Verdict::Done,
        Err(error) if error.errno == libc::EPERM => Verdict::Failed(format!(
            "{} is still there: this mount does not allow an unprivileged subvolume destroy ({error}), so remove the directory by hand",
            path.display()
        )),
        Err(error) => Verdict::Retry(format!("the subvolume of {} cannot be destroyed ({error})", path.display())),
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
    // The record goes before the move, so a command killed between the two leaves a record whose path is not there yet
    // (which deletes nothing) and never a directory nothing names.
    let item: WorkItem = removal_item(common_dir, into, id);
    if let Err(reason) = enqueue(work_directory(common_dir).as_path(), &item) {
        return Err(format!("the delete of {} cannot be recorded: {reason}", from.display()));
    }
    let left: PathBuf = PathBuf::from(item.parent.as_str());
    if let Err(error) = std::fs::rename(from, left.as_path()) {
        return Err(format!("the subvolume {} cannot be moved to {} ({error})", from.display(), left.display()));
    }
    debug(TAG, format!("the subvolume {} waits for the service's delete at {}", from.display(), left.display()).as_str());
    return Ok(left);
}

/// Whether an errno from the destroy by id means this kernel lacks the call,
/// so that the destroy by name below the held parent is the way left.
fn by_id_unavailable(errno: i32) -> bool {
    return errno == libc::ENOTTY || errno == libc::EOPNOTSUPP || errno == libc::EINVAL;
}

/// The path of a pending record,
/// for the words that name it.
pub fn pending_record_path(common_dir: &Path, inventory: &WorkItem) -> PathBuf {
    return state_directory(work_directory(common_dir).as_path(), PENDING).join(inventory.id());
}

/// The bytes of a path,
/// for a control that checks the record names the same place twice.
pub fn path_bytes(path: &Path) -> Vec<u8> {
    return path.as_os_str().as_bytes().to_vec();
}

/// Controls of the hand-over and of the service's delete stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_hand_over_tests.rs"]
mod tests;
