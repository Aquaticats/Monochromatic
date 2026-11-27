//! What:
//!  The empty folders a snapshot holds where its source holds a nested subvolume it did not carry,
//! below the folders that hold a
//!  repository's worktrees:
//! found by one listing of each such folder,
//! and taken off the add's waiting path,
//! because a folder that holds
//!  nothing but such stand-ins goes into the snapshot's trash behind one rename and a fresh empty folder takes its
//!  place.
//! Why:
//!  A btrfs snapshot leaves out every nested subvolume and shows a stand-in directory in its place
//!  (`doc/troubleshooting/btrfs-snapshot-keeps-an-empty-placeholder-for-a-nested-subvolume.md`),
//! so a worktree made as a
//!  snapshot while another add stages its snapshot beside its destination carries an empty folder nothing names,
//! and a
//!  tool that compares the tree with another copy sees a folder where the source holds a subvolume.
//! Each stand-in removed inside
//!  the add is one btrfs metadata change,
//! and every add waits on the transaction the others commit,
//! so a repository with many
//!  worktrees made every add wait on every other one:
//! 33 of 34 sampled adds sat in `wait_current_trans` inside `unlinkat` on 2026-11-10
//!  (`doc/planning/cli-git-worktree-add-stand-ins-off-the-waiting-path.md`).
//! The folder goes into the same trash the last release gave the
//!  commit service,
//! so the stand-ins are deleted after the add returns,
//! one worktree's trashes at a time and at the lowest I/O
//!  priority (`worktree_trash_hand_over.rs`).
//! The listing is limited to the folders that hold worktrees,
//! and a folder is taken only
//!  when every entry of it is a stand-in that is inode 2 and empty,
//! because every other entry may be someone's data:
//!  such a folder stays where it is,
//! and only the stand-ins in it are removed one by one.
//! A folder that is taken whole is looked at again in the trash,
//! because a directory somebody made between the check and the
//!  rename is never deleted with the stand-ins it replaced:
//! such a folder goes back where it was,
//! and when a directory took
//!  that name meanwhile,
//! it is kept under a name of this command's own beside the trash of the new worktree,
//! and the warning names the
//!  path it will have there,
//! so nothing the add deletes afterwards can take it (`never discard user data`).
//!
//! The look at one entry of such a folder and the trash a whole folder goes into
//! are the sibling files `worktree_snapshot_placeholders_entries.rs` and
//! `worktree_snapshot_placeholders_hand_over.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeSnapshotPlaceholders({ root, source, plan, trash }); // one rename per worktree folder that holds only stand-ins
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{
    open_directory_at,
    plain_components,
    remove_at,
};
use crate::worktree_command_facts::AddPlan;
use crate::worktree_path_failure_message::subject_failure;
/// One entry of a folder that holds worktrees,
/// and the two moments a control may change the snapshot at,
/// re-exported so every caller of this sweep names them here.
pub(crate) use crate::worktree_snapshot_placeholders_entries::{
    BTRFS_STAND_IN_INODE,
    FoldSeams,
    do_nothing,
    is_stand_in,
    listed_names,
};
/// The trash a whole worktree folder goes into,
/// re-exported so every caller of this sweep names it here.
pub(crate) use crate::worktree_snapshot_placeholders_hand_over::HandOver;
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::OsString;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::{
    Path,
    PathBuf,
};

/// Import the tag this module's diagnostics carry.
use crate::worktree::worktree_snapshot_placeholders_hand_over::TAG;

/// What:
///  The folders the worktrees of this repository are made in:
/// the parent of each registered worktree inside the source and the parent
///  of the destination,
/// once each and only below the source.
/// Why:
///   A stage container of an add in flight is named beside its destination,
/// so it lies in one of these folders;
/// a folder
///  outside the source is in no snapshot of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeFolders({ nested, destination, sourceRoot }): string[];
/// ```
fn worktree_folders(
    nested: &[PathBuf],
    destination: &Path,
    source_root: &Path,
) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = Vec::new();
    let paths = nested
        .iter()
        .map(PathBuf::as_path)
        .chain(std::iter::once(destination));
    for path in paths {
        let Some(parent) = path.parent() else {
            continue;
        };
        if parent.strip_prefix(source_root).is_err() || folders.contains(&parent.to_path_buf()) {
            continue;
        }
        folders.push(parent.to_path_buf());
    }
    return folders;
}

/// What:
///  The directory `names` lead to below the held snapshot,
/// or nothing when the snapshot does not carry it or when a
///  stand-in of a nested subvolume stands on the way.
/// Why:
///   A stand-in holds nothing and belongs to no folder a caller could name,
/// so nothing is below it;
/// every other refusal stays
///  an error,
/// so a link or another mount is named and never walked through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openBelowStandIns({ root, names, shown }): File | undefined; // throws on a link or a mount
/// ```
pub(crate) fn open_below_stand_ins(
    root: &File,
    names: &[OsString],
    shown: &str,
) -> Result<Option<File>, String> {
    let mut current: File = subject_failure(root.try_clone(), &shown, "cannot be held")?;
    for name in names {
        let next: File = match open_directory_at(&current, name.as_os_str()) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "{shown} is reached through a link or another mount in the snapshot ({error})"
                ));
            }
        };
        let found: std::fs::Metadata =
            subject_failure(next.metadata(), &shown, "cannot be examined")?;
        if found.is_dir() && found.ino() == BTRFS_STAND_IN_INODE {
            return Ok(None);
        }
        current = next;
    }
    return Ok(Some(current));
}

/// What:
///  Remove every stand-in of the held folder `held` as an empty directory,
/// and answer how many were removed.
/// Why:
///   A folder that holds one entry of someone else's is never moved whole,
/// so the stand-ins are the only thing here that
///  may go,
/// and the empty-directory removal keeps every file of that case too:
/// a folder that holds anything is refused.
fn remove_stand_ins(
    held: &File,
    shown: &str,
    listed: &[OsString],
) -> Result<usize, String> {
    let mut removed: usize = 0;
    for name in listed {
        if !is_stand_in(held, name.as_os_str()) {
            continue;
        }
        let path: String = format!("{shown}/{}", name.to_string_lossy());
        match remove_at(held, name.as_os_str(), true) {
            Ok(()) => removed += 1,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "{path} is a stand-in folder that cannot be removed from the snapshot as an empty directory ({error})"
                ));
            }
        }
    }
    return Ok(removed);
}

/// What:
///  Take the stand-ins a snapshot holds below the folder `folder` off the add's waiting path,
/// and answer how many
///  there were.
/// Why:
///   A folder the snapshot does not carry and a path outside the source are nothing to do;
/// a path that reaches the
///  snapshot through a link or another mount is an error,
/// so no entry is ever removed from the tree such a link points at.
/// The folder itself is taken when every entry of it is an empty stand-in,
/// and its stand-ins are removed one by one when
///  somebody else's entry sits beside them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function standInsBelow({ root, source, folder, handOver }): number;
/// ```
fn stand_ins_below(
    root: &File,
    source: &SnapshotSource,
    folder: &Path,
    hand_over: &mut HandOver,
    seams: &mut FoldSeams<'_>,
) -> Result<usize, String> {
    let Ok(relative) = folder.strip_prefix(source.root.as_path()) else {
        return Ok(0);
    };
    let shown: String = folder.display().to_string();
    let names: Vec<OsString> = subject_failure(
        plain_components(relative),
        &shown,
        "is not a plain path inside the source",
    )?;
    let Some((name, parents)) = names.split_last() else {
        // The source root is the snapshot itself, so no fresh folder can take its place here.
        let held: File = subject_failure(root.try_clone(), &shown, "cannot be held")?;
        let listed: Vec<OsString> = match listed_names(&held) {
            Ok(found) => found,
            Err(error) => {
                return Err(format!(
                    "{shown} cannot be listed in the snapshot ({error})"
                ));
            }
        };
        return remove_stand_ins(&held, &shown, listed.as_slice());
    };
    let Some(parent) = open_below_stand_ins(root, parents, &shown)? else {
        return Ok(0);
    };
    let held: File = match open_directory_at(&parent, name.as_os_str()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => {
            return Err(format!(
                "{shown} is reached through a link or another mount in the snapshot ({error})"
            ));
        }
    };
    let listed: Vec<OsString> = match listed_names(&held) {
        Ok(found) => found,
        Err(error) => {
            return Err(format!(
                "{shown} cannot be listed in the snapshot ({error})"
            ));
        }
    };
    if let Some(count) = hand_over.take_folder(
        root,
        source,
        &parent,
        name.as_os_str(),
        &held,
        listed.as_slice(),
        folder,
        seams,
    )? {
        return Ok(count);
    }
    return remove_stand_ins(&held, &shown, listed.as_slice());
}

/// What:
///  Remove the stand-ins the snapshot holds below the folders that hold this repository's worktrees and the destination,
/// and add every folder taken whole to the trash the add hands to the service.
/// Why:
///   The registered worktrees and the destination are removed one by one before this runs,
/// so what is left in those folders
///  is what no worktree names:
/// a stage container of an add in flight,
/// which the new worktree must not carry.
/// The trash the folders join is the leave-out's,
/// or one made here when the leave-out set nothing aside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeSnapshotPlaceholders({ root, source, plan, handOver });
/// ```
pub(crate) fn remove_snapshot_placeholders(
    root: &File,
    source: &SnapshotSource,
    plan: &AddPlan,
    hand_over: &mut HandOver,
) -> Result<(), String> {
    return remove_snapshot_placeholders_with(
        root,
        source,
        plan,
        hand_over,
        &mut FoldSeams {
            before_take: &mut do_nothing,
            before_put_back: &mut do_nothing,
        },
    );
}

/// What:
///  The same run of the step with the moments at which a control may change the snapshot given.
/// Why:
///   A control has no way to write into the snapshot between the check of a folder and its rename;
/// the seam is that moment,
///  and the production run puts nothing there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeSnapshotPlaceholdersWith({ root, source, plan, handOver, seams });
/// ```
pub(crate) fn remove_snapshot_placeholders_with(
    root: &File,
    source: &SnapshotSource,
    plan: &AddPlan,
    hand_over: &mut HandOver,
    seams: &mut FoldSeams<'_>,
) -> Result<(), String> {
    let mut removed: usize = 0;
    let folders_before: usize = hand_over.taken.len();
    // A folder this sweep takes whole is kept in the trash parent when it cannot go back to the name it came from,
    //  and the warning names the path it has once the stage is the new worktree.
    hand_over.name_the_new_worktree(plan.destination.as_path());
    for folder in worktree_folders(
        source.nested.as_slice(),
        plan.destination.as_path(),
        source.root.as_path(),
    ) {
        removed += stand_ins_below(root, source, folder.as_path(), hand_over, seams)?;
    }
    let took: usize = hand_over.taken.len() - folders_before;
    debug(TAG, format!("{}: {removed} stand-in folders of nested subvolumes removed from the snapshot, {took} worktree folders taken whole into the trash", plan.destination.display()).as_str());
    return Ok(());
}

/// Placeholder removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_placeholders_tests.rs"]
mod tests;
