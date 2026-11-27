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
//!  This file keeps the two steps of that subject the crate's layout rules tie to the name it already had,
//!  and every name a caller already used.
//!  Everything else of the subject sits in a file of its own beside it,
//!  because this file reached the repository's line ceiling with all of it in it:
//!  the records a removal writes beside a registration
//!  (`worktree_remove_hand_over_records_beside_the_registration.rs`),
//!  the record that says a removal of a worktree began
//!  (`worktree_remove_hand_over_begun_record.rs`),
//!  the record a removal queues and the wake that makes the service read it
//!  (`worktree_remove_hand_over_queue_and_wake.rs`),
//!  the removal the service makes of a whole worktree,
//!  the removal it makes around a volume that is not this removal's own,
//!  and the choice of which destroy a record asks for
//!  (`worktree_remove_hand_over_service_removal.rs`),
//!  the Git and the pool that removal reads
//!  (`worktree_remove_hand_over_service_git_and_pool.rs`),
//!  the notice that says a worktree was kept
//!  (`worktree_remove_hand_over_kept_worktree_notice.rs`),
//!  and the kernel's destroy of one renamed subvolume
//!  (`worktree_remove_hand_over_subvolume_delete.rs`).
//!  What stays here is the hand-over itself and the names every caller already reads by this module.

use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::fs_beneath::rename_at;
use crate::service_work::Verdict;
use crate::transaction_git::GitContext;
use crate::work_item::WorkItem;
use crate::worktree_removal_interrupted_destroy::record_leaving;
use crate::worktree_removal_leaving_that_outlives_its_record::{
    Leaving,
    forget_the_leaving,
    queue_the_delete_again,
    record_a_leaving,
};
// What:     `use crate::worktree_remove_hand_over_queue_and_wake::{...}` imports the
//           queue record, the wake and the item a hand-over writes.
// Why:      The record a hand-over leaves is that file's subject, and this file is
//           the one that makes it, so it names the definitions rather than a re-export.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ThroughTheService, aRecordIsQueued, queueTheRemoval, theRemovalOf } from './worktree_remove_hand_over_queue_and_wake';
// ```
use crate::worktree_remove_hand_over_queue_and_wake::{
    ThroughTheService,
    a_record_is_queued,
    queue_the_removal,
    the_removal_of,
};
use crate::worktree_remove_hand_over_subvolume_delete::destroy_the_subvolume_of;
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::File;
use std::path::Path;

/// What:
///  The name a subvolume this program leaves for the service takes,
///  and the record that names its delete,
///  re-exported where every reader of them already names them.
/// Why:
///   The shape of a leaving name and of its record is one subject
/// (`worktree_removal_leaves_a_subvolume_for_the_service.rs`),
/// and a caller names them by this module,
/// so the subject moved and the names did not.
pub use crate::worktree_removal_leaves_a_subvolume_for_the_service::{
    DELETE_REFERENCE,
    DELETING_PREFIX,
    deleting_name,
    removal_item,
};

/// What:
///  The move that leaves one subvolume for the service,
///  re-exported so every caller keeps naming it by this module.
/// Why:
///   A caller names the move and the record together,
/// and the two moved to the file that owns them
/// (`worktree_removal_leaves_a_subvolume_for_the_service.rs`),
/// so the names stay where the callers already read them.
pub(crate) use crate::worktree_removal_leaves_a_subvolume_for_the_service::leave_a_subvolume_for_the_service;

/// What:
///  The calls one command of the repository makes about the records a removal queues,
///  re-exported where every caller of them already names them.
/// Why:
///   A wake and a queueing that a command makes are read where the command already looks for them,
/// and the file that holds them is one of this subject's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { handOverWorktreeRemoval, requeueRemovalsAnEarlierReleaseRefused, wakeTheService } from './queue-and-wake';
/// ```
pub use crate::worktree_remove_hand_over_queue_and_wake::{
    hand_over_worktree_removal,
    requeue_removals_an_earlier_release_refused,
    wake_the_service,
};

/// What:
///  Who is told that a record waits for the service,
///  re-exported for the control of the leaving record that stands in for the service.
/// Why:
///   A control of another file's records answers the question this trait asks,
/// and the file that defines it is one of this subject's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type { ToldOfHandOver } from './queue-and-wake';
/// ```
pub(crate) use crate::worktree_remove_hand_over_queue_and_wake::ToldOfHandOver;

/// What:
///  The subvolume id a leaving record names,
///  re-exported where the command that queues a leaving already names it.
/// Why:
///   A record's object name is read by the command that queues a delete again,
/// and the reading belongs to the file that destroys the subvolume it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { recordedSubvolume } from './subvolume-delete';
/// ```
pub(crate) use crate::worktree_remove_hand_over_subvolume_delete::recorded_subvolume;

/// What:
///  The names the controls of this subject reach through this file.
/// Why:
///   A control of one of these files sits beside it and reads its names by this module,
/// which is the name every reader already uses,
/// and the names themselves belong to the files that define them.
#[cfg(test)]
pub(crate) use crate::worktree_remove_hand_over_begun_record::{
    BEGUN_FOLDER,
    record_begun,
};
/// What:
///  The reference a record of a whole-worktree removal names,
///  re-exported for the controls that read it.
/// Why:
///   A control writes a record of that kind by hand,
/// and the word it writes is the one the queueing file defines.
#[cfg(test)]
pub(crate) use crate::worktree_remove_hand_over_queue_and_wake::WORKTREE_REMOVAL_REFERENCE;
#[cfg(test)]
pub(crate) use crate::worktree_remove_hand_over_queue_and_wake::{
    not_woken_warning,
    pending_record_path,
    worktree_removal_item,
};
#[cfg(test)]
pub(crate) use crate::worktree_remove_hand_over_records_beside_the_registration::identity_standing_there;
#[cfg(test)]
pub(crate) use crate::worktree_remove_hand_over_service_removal::{
    delete_subvolume_of,
    delete_subvolume_of_with,
};

/// Import the tag every log line of this subject carries.
use crate::worktree::worktree_remove_hand_over_begun_record::TAG;

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
/// await handOverRemoval({ context, commonDir, parent, parentPath, name, id, target, registration });
/// ```
pub fn hand_over_removal(
    context: &GitContext,
    common_dir: &Path,
    parent: &File,
    parent_path: &Path,
    name: &OsStr,
    id: u64,
    target: &Path,
    registration: &OsStr,
) -> Result<(), String> {
    return hand_over_with(
        common_dir,
        parent,
        parent_path,
        name,
        id,
        target,
        registration,
        &ThroughTheService {
            context,
        },
    );
}

/// What:
///  `hand_over_removal` with the wake injected,
/// so a control reaches no service and no session bus.
/// Why:
///   The leaving record is written first,
///  the delete's own queue record second,
///  and the directory renamed third,
///  so a kill in the moments between leaves a state the next command finishes:
///  the leaving record is settled by nothing,
///  so it names the place the directory takes before it takes it,
///  and a control can assert that without a service.
///   A wake that fails is never the removal's failure:
/// the record stays for the next command of the
///  repository.
///   A delete the queue settled before the rename is deleted here,
///  and when the kernel refuses that destroy,
///  the delete is queued once more for a round to run and the leaving record names the directory until a command proves it
///  gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith({ commonDir, parent, parentPath, name, id, target, registration, told });
/// ```
pub fn hand_over_with(
    common_dir: &Path,
    parent: &File,
    parent_path: &Path,
    name: &OsStr,
    id: u64,
    target: &Path,
    registration: &OsStr,
    told: &dyn ToldOfHandOver,
) -> Result<(), String> {
    let item: WorkItem = the_removal_of(common_dir, parent_path, id)?;
    // The leaving is the one record of these three that nothing settles, so it is written first: a command killed after the
    // rename, and a delete the queue settles before the rename, both leave a place a later command finds
    // (`worktree_removal_leavings_a_command_queues_again.rs`).
    let leaving: Leaving = record_a_leaving(common_dir, &item)?;
    if let Err(reason) = queue_the_removal(common_dir, &item) {
        // Nothing has moved: the leaving record would name a directory the rename never makes.
        forget_the_leaving(common_dir, item.id().as_str());
        return Err(reason);
    }
    let deleting: OsString = deleting_name(id);
    // Where the files will stand is written before the rename, so a command killed between the two leaves a record whose
    // worktree path is gone and whose place is named, which is a state the next command finishes the registration by
    // (a review finding, F4).
    record_leaving(
        common_dir,
        registration,
        parent_path.join(deleting.as_os_str()).as_path(),
    );
    if let Err(error) = rename_at(parent, name, parent, deleting.as_os_str(), false) {
        // Nothing was renamed, so the record names a path that does not exist and the service will find nothing to delete; the
        // leaving names the same place and goes with it.
        forget_the_leaving(common_dir, item.id().as_str());
        return Err(format!(
            "the subvolume of {} cannot be renamed out of the way ({error})",
            target.display()
        ));
    }
    debug(
        TAG,
        format!(
            "the subvolume of {} waits for the service's delete at {}",
            target.display(),
            Path::new(&item.parent).display()
        )
        .as_str(),
    );
    if !a_record_is_queued(common_dir, &item) {
        // A service settled the record, or claimed it and looked while the directory still stood at its old name, so nothing in
        //  the queue names the directory now:
        //  this command deletes it rather than leave the person's files under a name no record names.
        let verdict: Verdict = destroy_the_subvolume_of(&item);
        if verdict == Verdict::Done {
            // Nothing stands where the leaving named, so its record is finished with it.
            forget_the_leaving(common_dir, item.id().as_str());
            return Ok(());
        }
        warn(
            TAG,
            format!(
                "the record of {} was settled before the removal renamed it, and the renamed directory cannot be deleted ({verdict:?}), so it was left at {}",
                target.display(),
                Path::new(&item.parent).display()
            )
            .as_str(),
        );
        // The destroy here could not take it, for a reason that may last, so the delete goes back where a round finds it: the
        // leaving record above names the directory until a command proves it gone.
        if let Err(reason) = queue_the_delete_again(common_dir, &leaving) {
            warn(TAG, format!("the delete of {} cannot be queued again ({reason}); the leaving record still names it", target.display()).as_str());
        }
        return Ok(());
    }
    if let Err(why) = told.told(common_dir) {
        // The record stays, and the next commit of this repository wakes the service that reads it.
        warn(
            TAG,
            format!(
                "the delete of {} is recorded and not woken ({why})",
                target.display()
            )
            .as_str(),
        );
    }
    return Ok(());
}

/// Controls of the hand-over and of the service's delete stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_hand_over_tests.rs"]
mod tests;
