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
use crate::fs_beneath::{open_directory_at, open_root, rename_at};
use crate::fs_btrfs_ioctl::{snapshot_destroy_by_id, snapshot_destroy_in, subvolume_id};
use crate::iso_time::now_iso;
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
/// the queue's id is the subvolume's and the directory that holds it,
/// so a second write finds the first.
/// A directory whose path is not valid UTF-8 is refused here,
/// because a work item names its paths in text:
///  a record of the replacement character names another directory,
/// which no delete could ever find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// enqueue(join(workDir(commonDir), 'pending'), removalItem({ commonDir, parentPath, id }))
/// ```
fn record_the_removal(common_dir: &Path, parent_path: &Path, id: u64) -> Result<WorkItem, String> {
    if parent_path.to_str().is_none() {
        return Err(format!(
            "the directory that holds {} is not named by text, so no work record can name the directory the removal would rename it to ({} is not valid UTF-8)",
            parent_path.join(deleting_name(id)).display(),
            parent_path.display()
        ));
    }
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
/// make sure a record still names it,
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
/// The record is written first so that a kill in this window leaves one of two safe states,
/// and a running service can
///  claim that record before the rename happens:
/// it looks for the hidden path and finds nothing,
/// so this command asks whether
///  a record is still queued for the directory it renamed and deletes it itself when none is.
/// Otherwise the worktree's files
///  would stand under a hidden name no record names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await handOverRemoval({ context, commonDir, parent, parentPath, name, id });
/// ```
pub fn hand_over_removal(context: &GitContext, common_dir: &Path, parent: &File, parent_path: &Path, name: &OsStr, id: u64, target: &Path) -> Result<(), String> {
    return hand_over_with(common_dir, parent, parent_path, name, id, target, &ThroughTheService { context });
}

/// What:
///  Who is told that a deletion record waits for the service.
/// Why:
///   A removal's wake is one function bound to the context the command holds,
/// and a closure carrying that binding is a shape this crate's closure rule refuses
///  (`package/git-policy/cli/doc/closure-lint-suppressions.md`:
/// only the standard library's own callables are suppressed),
/// so the state travels in a value and the wake is a method of it.
pub trait ToldOfHandOver {
    /// What:
    ///  Tell the service of `common_dir` that a record waits.
    /// Why:
    ///   A hand-over is never what makes a removal fail,
    /// so the answer is a reason the removal warns about rather than an error it returns.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// told(commonDir): string | undefined
    /// ```
    fn told(&self, common_dir: &Path) -> Result<(), String>;
}

/// The service of the person's session,
/// woken through the context the removal holds.
pub struct ThroughTheService<'a> {
    /// The context whose repository is the session's.
    pub context: &'a GitContext,
}

impl ToldOfHandOver for ThroughTheService<'_> {
    fn told(&self, common_dir: &Path) -> Result<(), String> {
        return wake_the_service(self.context, common_dir);
    }
}

/// What:
///  `hand_over_removal` with the wake injected,
/// so a control reaches no service and no session bus.
/// Why:
///   The record is written and the directory renamed before the service is told,
/// so a control can assert both without a
///  service,
/// and a wake that fails is never the removal's failure:
/// the record stays for the next command of the
///  repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith({ commonDir, parent, parentPath, name, id, target, told });
/// ```
pub fn hand_over_with(common_dir: &Path, parent: &File, parent_path: &Path, name: &OsStr, id: u64, target: &Path, told: &dyn ToldOfHandOver) -> Result<(), String> {
    let item: WorkItem = record_the_removal(common_dir, parent_path, id)?;
    let deleting: OsString = deleting_name(id);
    if let Err(error) = rename_at(parent, name, parent, deleting.as_os_str(), false) {
        // Nothing was renamed, so the record names a path that does not exist and the service will find nothing to delete.
        return Err(format!("the subvolume of {} cannot be renamed out of the way ({error})", target.display()));
    }
    debug(TAG, format!("the subvolume of {} waits for the service's delete at {}", target.display(), Path::new(&item.parent).display()).as_str());
    if !a_record_is_queued(common_dir, &item) {
        // A service settled the record, or claimed it and looked while the directory still stood at its old name, so nothing names the directory now:
        //  this command deletes it rather than leave the person's files under a name no record names.
        let verdict: Verdict = delete_subvolume_of(&item);
        if verdict != Verdict::Done {
            warn(
                TAG,
                format!(
                    "the record of {} was settled before the removal renamed it, and the renamed directory cannot be deleted ({verdict:?}), so it was left at {}",
                    target.display(),
                    Path::new(&item.parent).display()
                )
                .as_str(),
            );
        }
        return Ok(());
    }
    if let Err(why) = told.told(common_dir) {
        // The record stays, and the next command of this repository wakes the service that reads it.
        warn(TAG, format!("the delete of {} is recorded and not woken ({why})", target.display()).as_str());
    }
    return Ok(());
}

/// What:
///  Whether a record for `item` is still queued for a consumer to run.
/// Why:
///   A record a service already settled names no directory,
/// and neither does a claimed one:
/// the claim is taken before the consumer looks,
/// so a consumer that looked while
///  the directory still stood at its old name found nothing to delete
/// and settles the record without a delete.
/// A record still
///  queued is work a consumer has yet to begin,
/// so only that one leaves the directory to the service.
/// A queued record is asked for rather than listed,
///  which costs one lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exists(pending/<id>)
/// ```
fn a_record_is_queued(common_dir: &Path, item: &WorkItem) -> bool {
    return pending_record_path(common_dir, item).is_file();
}

/// What:
///  The subvolume id an item's object name holds,
/// or nothing when the name is not one.
/// Why:
///   The id names one subvolume for good,
/// while a name names whatever is there when the call runs,
/// so the destroy must
///  name the id the record holds and never the name alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = (oid) => /^[0-9a-fA-F]{40}$/.test(oid) ? BigInt('0x' + oid) : undefined;
/// ```
fn recorded_subvolume(oid: &str) -> Option<u64> {
    if oid.len() != 40 || !oid.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    return u64::from_str_radix(oid, 16).ok();
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
/// A path that is not a directory,
/// or that
///  holds another subvolume than the record's id,
/// is not this item's to remove:
/// a replacement is left as it is and named in the
///  failure,
/// because the record promised the person who removed the worktree that one subvolume goes and nothing else
///  (`SPEC.md`,
/// "Cheap worktree creation and removal on btrfs,
/// XFS and overlayfs").
/// The destroy names the subvolume by the id the record holds when the kernel
///  allows it,
/// so a replaced ancestor cannot redirect it,
/// and by name below the opened parent otherwise,
/// once the id
///  that stands there is proven to be the record's.
/// A mount that does not allow an unprivileged destroy is a failure the person must
///  hear,
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
    let Some(recorded): Option<u64> = recorded_subvolume(item.oid.as_str()) else {
        return Verdict::Failed(format!(
            "the record of {} names no subvolume id ({:?}), so nothing proves which subvolume stands there and it was left as it is",
            path.display(),
            item.oid
        ));
    };
    let parent: File = match open_root(parent_path) {
        Ok(opened) => opened,
        // The directory that held it is gone, so nothing stands at the name below it.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Verdict::Done,
        Err(error) => return Verdict::Retry(format!("the directory {} cannot be opened ({error})", parent_path.display())),
    };
    // The name still has to lead to the directory the record names, or the delete would act on whatever took its place.
    let standing: File = match open_directory_at(&parent, name) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound || error.kind() == std::io::ErrorKind::NotADirectory => {
            debug(TAG, format!("{} is gone already; nothing to delete", path.display()).as_str());
            return Verdict::Done;
        }
        Err(error) => return Verdict::Retry(format!("{} cannot be opened ({error})", path.display())),
    };
    match subvolume_id(&standing, path.as_path()) {
        Ok(id) if id == recorded => {}
        Ok(id) => {
            return Verdict::Failed(format!(
                "{} holds the subvolume {id}, and the record names the subvolume {recorded}, so it was left as it is; remove it by hand if it is yours",
                path.display()
            ));
        }
        Err(error) if error.errno == libc::ENOENT => return Verdict::Done,
        Err(error) if by_id_unavailable(error.errno) => {
            return Verdict::Failed(format!(
                "{} cannot be identified as the subvolume {recorded} on this kernel ({error}), so it was left as it is",
                path.display()
            ));
        }
        Err(error) => return Verdict::Retry(format!("the subvolume of {} cannot be identified ({error})", path.display())),
    }
    match snapshot_destroy_by_id(&parent, recorded, path.as_path()) {
        Ok(()) => return Verdict::Done,
        Err(error) if error.errno == libc::EPERM => {
            return Verdict::Failed(format!(
                "{} is still there: this mount does not allow an unprivileged subvolume destroy ({error}), so remove the directory by hand, and remount it with user_subvol_rm_allowed to let the wrapper do it",
                path.display()
            ));
        }
        Err(error) if error.errno == libc::ENOENT => return Verdict::Done,
        Err(error) if by_id_unavailable(error.errno) => {
            debug(TAG, format!("the destroy of {} by id is not available ({error}); destroying it by name", path.display()).as_str());
        }
        Err(error) => return Verdict::Retry(format!("the subvolume of {} cannot be destroyed ({error})", path.display())),
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
