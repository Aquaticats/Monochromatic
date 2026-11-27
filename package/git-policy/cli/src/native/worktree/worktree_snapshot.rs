//! What:
//!  The mechanics of rung 1 of the cheap-worktree ladder:
//! a snapshot of the source subvolume taken at a stage name beside
//!  the destination,
//! made into a worktree there (its own `.git` pointer,
//! no stubs of nested worktrees,
//! the source's index
//!  with refreshed stat fields),
//! and swapped with the destination in one atomic step.
//! Why:
//!  The snapshot shares every data block with the source and carries its ignored state at no cost
//!  (`doc/planning/cli-git-btrfs-worktree.md`).
//! It is made at a name Git never hands out,
//! so the destination is
//!  complete or untouched at every instant:
//! `RENAME_EXCHANGE` puts the finished subvolume at the destination and the
//!  empty directory real Git made at the stage name,
//! which is then removed.
//! A source that changed while the snapshot was taken is caught by comparing the snapshot's own files with the index
//!  entries copied from the source;
//! the index step itself is in `worktree_snapshot_index_step.rs`.
//!
//! The stubs a nested worktree leaves,
//! and the swap with its settling,
//! are the sibling files `worktree_snapshot_stubs.rs` and `worktree_snapshot_place.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await takeSnapshot({ source, plan, stage, admin }); // throws a reason; the caller destroys the stage
//! await putInPlace({ stage, destination, mode });
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
};
use crate::diagnostic_log::warn;
use crate::fs_beneath::{
    ObjectId,
    identity_at,
    open_root,
};
use crate::fs_btrfs_ioctl::snapshot_create_at;
use crate::worktree_clean_content::ConversionEnvironment;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_path_failure_message::subject_failure;
pub use crate::worktree_snapshot_index_step::CopyState;
use crate::worktree_snapshot_index_step::{
    Proof,
    install_index,
    read_own_index,
};
/// The swap of the finished snapshot with the destination,
/// and the settling of what it displaced,
/// re-exported so every caller of this rung names them here.
pub use crate::worktree_snapshot_place::{
    PlaceError,
    PlaceSeams,
    put_in_place,
    put_in_place_with,
    settle_displaced,
};
use crate::worktree_snapshot_placeholders::HandOver;
use crate::worktree_snapshot_source::SnapshotSource;
/// The stubs a nested worktree leaves in a snapshot,
/// and the worktree's own `.git` pointer,
/// re-exported so every caller of this rung names them here.
pub(crate) use crate::worktree_snapshot_stubs::{
    remove_stubs,
    replace_git_pointer,
};
use crate::worktree_trash::{
    Pruned,
    prune_snapshot,
};
use crate::worktree_trash_manifest::TrashManifest;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// The tag every line this module logs carries.
const TAG: &str = "worktreeSnapshot";

/// What:
///  A snapshot step's failure carrying the identity of the stage this process made,
/// so the caller can destroy it.
/// Why:
///   Once the stage exists,
/// every later failure must name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcome.mapErr((reason) => ({ reason, stage: made }))
/// ```
fn failing<T>(
    outcome: Result<T, String>,
    made: Option<ObjectId>,
) -> Result<T, SnapshotFailure> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(reason) => Err(SnapshotFailure {
            reason,
            stage: made,
        }),
    };
}

/// What:
///  The identity of what a path's name leads to now,
/// or nothing when its parent cannot be opened or the name cannot be looked up.
/// Why:
///   The stage's identity is what a later removal checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityAt(openRoot(parent(path)), basename(path)) ?? undefined
/// ```
fn identity_of_name(path: &Path) -> Option<ObjectId> {
    let parent: File = open_root(path.parent().unwrap_or(path)).ok()?;
    return identity_at(&parent, path.file_name()?).ok();
}

/// What:
///  Records the identity of the object this snapshot made,
///  saying so in the log when the record cannot be written.
/// Why:
///  The claim of an add names its stage before the stage exists,
/// so nothing but this identity proves what the wrapper made there,
///  and a kill can come at any moment once the object stands;
/// the record is written as soon as the object exists,
/// and raised again
///  once the wrapper's last write into it is done.
/// A record that cannot be written never fails the add:
/// the add is whole either way,
///  and a stage a kill leaves is then reported instead of removed,
/// which is the safe answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordStageMadeObject({ commonDir: source.commonDir, container: stage, object: made });
/// ```
pub fn record_made_of_the_stage(
    source: &SnapshotSource,
    stage: &Path,
    made: Option<ObjectId>,
) {
    let Some(object) = made else { return };
    let recorded: Result<(), crate::worktree_copy_error::WorktreeCopyError> =
        crate::worktree_copy_stage_made_object::record_made(
            source.common_dir.as_path(),
            stage,
            object,
        );
    if let Err(error) = recorded {
        warn(TAG, format!("monochromatic-git-policy-cli: the object of the stage {} could not be recorded ({}); a stage a kill leaves behind will be reported and not removed", stage.display(), error.0).as_str());
    }
}

/// What:
///  Take the snapshot at `stage` and make it the new worktree there;
/// the caller destroys `stage` on an error.
/// Why:
///   Each step is one the later steps depend on;
/// the first failure ends the rung with its reason.
/// Every add takes its own snapshot of the source's ignored files,
/// never one the commit service made earlier:
/// a copy taken ahead is the source as of an older instant,
/// and the owner answered a request to keep installing one with "Fresh snapshot every add" (2026-11-11),
/// measured at 5.4 s at the median and 26 s at worst against his bound of 60 s for an add.
/// One `BTRFS_IOC_SNAP_CREATE_V2` forces a btrfs transaction commit on the thread that calls it,
/// which is what that bound pays for.
/// The index is installed before the source's `.git` goes,
/// so a copy the source changed under can still read the index it carries.
/// The paths the conversion fence leaves to Git are installed without stat data (`written`),
/// so the index this step publishes cannot vouch for a path whose bytes are not Git's checkout's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await takeSnapshot({ source, plan, stage, admin, pointer, environment, written });
/// ```
pub fn take_snapshot(
    source: &SnapshotSource,
    plan: &AddPlan,
    stage: &Path,
    admin: &File,
    pointer: &[u8],
    environment: &ConversionEnvironment,
    written: &std::collections::HashSet<Vec<u8>>,
) -> Result<Snapshotted, SnapshotFailure> {
    // The record comes first, and names the container, the instant, and the source and transaction the snapshot below must
    // follow, while it holds no object yet: the identity of an object cannot be read before it exists, and a kill between
    // the creation below and the record of the identity left a whole snapshot nothing could prove the wrapper made
    // (`worktree_copy_stage_reaping_proof_and_words.rs`). The binding record this rung wrote a moment ago was flushed, so
    // the journal's own subvolume carries the last committed transaction, and the snapshot made now is later than it.
    let origin: Option<crate::worktree_copy_stage_snapshot_origin::SnapshotOrigin> =
        crate::worktree_copy_stage_snapshot_origin::origin_of_the_source(
            source.root.as_path(),
            source.common_dir.as_path(),
        );
    if let Err(error) = crate::worktree_copy_stage_made_object::record_creating_at(
        source.common_dir.as_path(),
        stage,
        std::time::SystemTime::now(),
        origin.as_ref(),
    ) {
        warn(TAG, format!("monochromatic-git-policy-cli: the creation of the stage {} could not be recorded ({}); a stage a kill leaves behind will be reported and not removed", stage.display(), error.0).as_str());
    }
    let snapshot_ioctl_entered: bool = begin_step("snapshotIoctl");
    let snapshot_ioctl_outcome = snapshot_create_at(source.root.as_path(), stage);
    end_step(snapshot_ioctl_entered);
    if let Err(error) = snapshot_ioctl_outcome {
        return Err(SnapshotFailure {
            reason: format!("the snapshot was refused ({error})"),
            stage: None,
        });
    }
    let made: Option<ObjectId> = identity_of_name(stage);
    // The add's own snapshot needs no earlier proof of the index: this rung walks the copy it just made.
    let proven: Option<Proof> = None;
    // From here the stage exists and this process made it; its identity is what a later removal checks, so it is recorded
    // before the next step can be killed.
    record_made_of_the_stage(source, stage, made);
    failing(
        subject_failure(
            std::fs::set_permissions(stage, std::fs::Permissions::from_mode(0o700)),
            &"the stage",
            "cannot be made private",
        ),
        made,
    )?;
    // The snapshot's own index is read before its `.git` goes: it is the index of the same instant as the files.
    let own_index: Option<Vec<u8>> = if source.dirty.is_some() {
        let read_own_index_entered: bool = begin_step("readOwnIndex");
        let read_own_index_outcome = read_own_index(stage);
        end_step(read_own_index_entered);
        Some(failing(read_own_index_outcome, made)?)
    } else {
        None
    };
    let prune_entered: bool = begin_step("prune");
    let prune_outcome = prune_snapshot(stage, source);
    end_step(prune_entered);
    let pruned: Pruned = failing(prune_outcome, made)?;
    let mut hand_over: HandOver = HandOver::starting(pruned.trash);
    let remove_stubs_entered: bool = begin_step("removeStubs");
    let remove_stubs_outcome = remove_stubs(stage, source, plan, &mut hand_over);
    end_step(remove_stubs_entered);
    failing(remove_stubs_outcome, made)?;
    let install_index_entered: bool = begin_step("installIndex");
    let install_index_outcome = install_index(
        source,
        stage,
        admin,
        own_index.as_deref(),
        environment,
        proven,
        written,
    );
    end_step(install_index_entered);
    let state: CopyState = failing(install_index_outcome, made)?;
    let replace_pointer_entered: bool = begin_step("replacePointer");
    let replace_pointer_outcome = replace_git_pointer(stage, pointer, source, &mut hand_over);
    end_step(replace_pointer_entered);
    failing(replace_pointer_outcome, made)?;
    let Some(stage_id) = made else {
        return Err(SnapshotFailure {
            reason: String::from("the stage cannot be examined"),
            stage: None,
        });
    };
    let trash: Option<TrashManifest> = hand_over.finish();
    // The trash says what it is before anything publishes it: the manifest travels with the trash through the swap, so a
    //  command killed between the swap and its record leaves a trash the next pass can prove and empty, where a trash only a
    //  record describes left an orphan nothing would ever name.
    if let Some(manifest) = trash.as_ref()
        && let Err(reason) = crate::worktree_trash_own_manifest::write(stage, manifest)
    {
        warn(TAG, format!("monochromatic-git-policy-cli: the trash {} cannot say what it is ({reason}), so a kill before its record leaves it for the next pass to name and the person to remove", manifest.name).as_str());
    }
    return Ok(Snapshotted {
        stage: stage_id,
        trash,
        taken_over: pruned.taken_over,
        state,
    });
}

/// What a snapshot that was taken left for the steps after it.
#[derive(Debug, Eq, PartialEq)]
pub struct Snapshotted {
    /// The identity of the stage this process made.
    pub stage: ObjectId,
    /// The trash the leave-out made in the stage,
    /// with the identity of everything it moved there;
    /// the disposal after the swap
    ///  empties exactly that.
    pub trash: Option<TrashManifest>,
    /// How many of the trash's directories came from an earlier add's trash,
    /// which the hand-over names to the person.
    pub taken_over: usize,
    /// What the index step settled about the copy.
    pub state: CopyState,
}

/// Why a snapshot could not be taken,
/// and the identity of the stage it left,
/// if it left one.
#[derive(Debug, Eq, PartialEq)]
pub struct SnapshotFailure {
    /// The reason,
    /// in words.
    pub reason: String,
    /// The identity of the stage this process made before it failed.
    pub stage: Option<ObjectId>,
}

/// Snapshot mechanics controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_tests.rs"]
mod tests;
