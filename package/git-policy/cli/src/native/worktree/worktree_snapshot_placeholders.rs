//! What:
//!  The empty folders a snapshot holds where its source holds a nested subvolume it did not carry,
//! below the folders that hold a
//!  repository's worktrees:
//! found by one listing of each such folder and removed as an empty directory.
//! Why:
//!  A btrfs snapshot leaves out every nested subvolume and shows a stand-in directory in its place
//!  (`doc/troubleshooting/btrfs-snapshot-keeps-an-empty-placeholder-for-a-nested-subvolume.md`),
//! so a worktree made as a
//!  snapshot while another add stages its snapshot beside its destination carries an empty folder nothing names,
//! and a
//!  tool that compares the tree with another copy sees a folder where the source holds a subvolume.
//! The listing is limited to those folders,
//! and the removal to a folder that is inode 2 and empty,
// because every other
//!  entry may be someone's data.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeSnapshotPlaceholders({ stage, source, plan });
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{entry_metadata_at, handle_path, open_directory_beneath, plain_components, remove_at};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_path_failure_message::subject_failure;
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeSnapshotPlaceholders";

/// What:
///  The inode number btrfs gives a stand-in,
/// read from the module that owns the number.
/// Why:
///   The leave-out step needs the same number to leave a stand-in alone,
/// and one constant in
///  two places would drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STAND_IN_INODE = BTRFS_STAND_IN_INODE;
/// ```
const BTRFS_STAND_IN_INODE: u64 = crate::fs_kind::BTRFS_STAND_IN_INODE;

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
fn worktree_folders(nested: &[PathBuf], destination: &Path, source_root: &Path) -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = Vec::new();
    let paths = nested.iter().map(PathBuf::as_path).chain(std::iter::once(destination));
    for path in paths {
        let Some(parent) = path.parent() else { continue };
        if parent.strip_prefix(source_root).is_err() || folders.contains(&parent.to_path_buf()) {
            continue;
        }
        folders.push(parent.to_path_buf());
    }
    return folders;
}

/// What:
///  The names a held folder lists.
/// Why:
///   The placeholders are found by looking at every entry of a folder that holds worktrees,
/// and an entry that cannot be read
///  keeps the rung from finishing quietly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = (await readdir(handlePath(dir))).map((e) => e.name);
/// ```
fn listed_names(directory: &File) -> std::io::Result<Vec<OsString>> {
    let mut names: Vec<OsString> = Vec::new();
    for entry in std::fs::read_dir(handle_path(directory))? {
        names.push(entry?.file_name());
    }
    return Ok(names);
}

/// What:
///  Whether the entry `name` below `directory` is the stand-in btrfs shows for a nested subvolume a snapshot did not
///  carry.
/// Why:
///   The inode number and the directory type are the whole test;
/// an entry that cannot be examined is no stand-in and stays,
/// and
///  the removal itself is what refuses a folder that holds any entry,
/// so a stand-in that gained content is never discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (found) => found.isDirectory() && found.ino === STAND_IN_INODE
/// ```
fn is_stand_in(directory: &File, name: &OsStr) -> bool {
    return match entry_metadata_at(directory, name) {
        Ok(found) => found.is_dir() && found.ino() == BTRFS_STAND_IN_INODE,
        Err(_unreadable) => false,
    };
}

/// What:
///  Remove every stand-in the folder `folder` holds in the snapshot below the held handle of its root,
/// each as an empty directory,
///  and return how many were removed.
/// Why:
///   A folder the snapshot does not carry and a path outside the source are nothing to do;
/// a path that reaches the
///  snapshot through a link or another mount is an error,
/// so no entry is ever removed from the tree such a link points at.
/// The name of a stand-in can be replaced between the look at it and its removal,
/// since another add exchanges its
///  stage name with the empty folder Git made.
/// The empty-directory removal is what keeps every file of that case too:
/// a folder that holds anything is
///  refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeStandInsBelow({ root, sourceRoot, folder }): Promise<number>;
/// ```
fn remove_stand_ins_below(root: &File, source_root: &Path, folder: &Path) -> Result<usize, String> {
    let Ok(relative) = folder.strip_prefix(source_root) else { return Ok(0) };
    let shown: String = folder.display().to_string();
    let names: Vec<OsString> = subject_failure(plain_components(relative), &shown, "is not a plain path inside the source")?;
    let held: File = match open_directory_beneath(root, names.as_slice()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(format!("{shown} is reached through a link or another mount in the snapshot ({error})")),
    };
    let listed: Vec<OsString> = match listed_names(&held) {
        Ok(found) => found,
        Err(error) => return Err(format!("{shown} cannot be listed in the snapshot ({error})")),
    };
    let mut removed: usize = 0;
    for name in listed {
        if !is_stand_in(&held, name.as_os_str()) {
            continue;
        }
        let path: String = format!("{shown}/{}", name.to_string_lossy());
        match remove_at(&held, name.as_os_str(), true) {
            Ok(()) => removed += 1,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{path} is a stand-in folder that cannot be removed from the snapshot as an empty directory ({error})")),
        }
    }
    return Ok(removed);
}

/// What:
///  Remove the stand-ins the snapshot holds below the folders that hold this repository's worktrees and the destination.
/// Why:
///   The registered worktrees and the destination are removed one by one before this runs,
/// so what is left in those folders
///  is what no worktree names:
/// a stage container of an add in flight,
/// which the new worktree must not carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeSnapshotPlaceholders({ root, source, plan });
/// ```
pub(crate) fn remove_snapshot_placeholders(root: &File, source: &SnapshotSource, plan: &AddPlan) -> Result<(), String> {
    let mut removed: usize = 0;
    for folder in worktree_folders(source.nested.as_slice(), plan.destination.as_path(), source.root.as_path()) {
        removed += remove_stand_ins_below(root, source.root.as_path(), folder.as_path())?;
    }
    debug(TAG, format!("{}: {removed} stand-in folders of nested subvolumes removed from the snapshot", plan.destination.display()).as_str());
    return Ok(());
}

/// Placeholder removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_snapshot_placeholders_tests.rs"]
mod tests;
