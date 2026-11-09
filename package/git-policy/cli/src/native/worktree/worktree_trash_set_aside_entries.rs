//! What:
//!  The entries a snapshot's leave-out sets aside,
//! and the checks and sweeps it applies to them:
//! whether a name is another
//!  command's stage container,
//! the sweep that removes those from a directory moved whole,
//! whether an entry is a directory at
//!  all,
//! and which trash in the trash parent is this add's own.
//! Why:
//!  The leave-out moves other checkouts and caches whole,
//! and a container stands beside the destination of one add,
//!  so it can sit inside such a directory:
//! the sweep is what finds it there.
//! A container holds another worktree's
//!  ignored state,
//! credentials included,
//! and the trash stands inside the new worktree until the emptying runs,
//! so it is
//!  removed while the leave-out runs rather than moved.
//! These are the checks the walk asks about one entry at a time,
//! kept
//!  in a file of their own so the walk itself stays readable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { namesTheContainer, removeContainersBeneath, inheritedTrashEntries } = setAsideEntries;
//! ```

use super::{Aside, TRASH_PARENT, Trash};
use crate::fs_beneath::open_directory_at;
use crate::worktree_copy_selection::STAGE_PREFIX;
use crate::worktree_path_failure_message::path_failure;
use crate::worktree_trash_remove::remove_tree_beneath;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt as _;
use std::fs::File;
use std::path::{Path, PathBuf};

/// What:
///  Remove every stage container that lies anywhere below the directory `name` of `holder`,
/// and say which one could
///  not be removed when one cannot.
/// Why:
///   The leave-out moves a nested worktree or another checkout as one directory,
/// and a container inside it is
///  not a name this walk ever hands to the setting-aside step:
/// the sweep is what finds it,
/// so no trash and
///  no new worktree ever holds another command's credentials.
/// The walk goes through held handles,
/// opens nothing
///  through a link,
/// and looks inside every directory it meets,
/// `.git` included,
/// because the directory it was
///  handed is itself another checkout and a container can stand beside that marker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeContainersBeneath(holder, name, shown) { for (const entry of readdir(holder, name)) { if (entry.name.startsWith(STAGE_PREFIX)) await removeTreeBeneath(...); else if (entry.isDirectory()) recurse; } }
/// ```
pub(super) fn remove_containers_beneath(holder: &File, name: &OsStr, shown: &Path) -> Result<(), String> {
    let directory: File = path_failure(open_directory_at(holder, name), shown, "is not a plain directory")?;
    let mut children: Vec<OsString> = Vec::new();
    for listed in path_failure(std::fs::read_dir(crate::fs_beneath::handle_path(&directory)), shown, "cannot be listed")? {
        let entry: std::fs::DirEntry = path_failure(listed, shown, "cannot be listed")?;
        children.push(entry.file_name());
    }
    for child in children {
        let path: PathBuf = shown.join(&child);
        if names_the_container(Path::new(child.as_os_str())) {
            path_failure(remove_tree_beneath(shown, &directory, child.as_os_str(), None), path.as_path(), "cannot be removed")?;
            continue;
        }
        if !is_directory_entry_at(&directory, child.as_os_str()) {
            continue;
        }
        remove_containers_beneath(&directory, child.as_os_str(), path.as_path())?;
    }
    return Ok(());
}

/// What:
///  Whether the entry `name` of `directory` is a directory,
/// not a link to one.
/// Why:
///   Only a directory is walked,
/// so a link named like a directory is never followed out of the snapshot.
/// The answer comes from the held handle,
/// so no path is resolved twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lstatSync(join(handlePath(directory), name)).isDirectory()
/// ```
fn is_directory_entry_at(directory: &File, name: &OsStr) -> bool {
    return crate::fs_beneath::entry_metadata_at(directory, name).is_ok_and(is_directory_metadata);
}

/// Whether a metadata is a directory's.
fn is_directory_metadata(found: std::fs::Metadata) -> bool {
    return found.is_dir();
}

/// What:
///  Whether a path below the snapshot names another command's stage container,
/// by its last component of the
///  reserved prefix.
/// Why:
///   The walk calls the setting-aside step with the container itself,
/// so the last component is what says whether what it
///  holds is another command's ignored state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.basename(relative).startsWith(STAGE_PREFIX)
/// ```
pub(super) fn names_the_container(relative: &Path) -> bool {
    let Some(name) = relative.file_name() else { return false };
    return name.as_bytes().starts_with(STAGE_PREFIX.as_bytes());
}

/// What:
///  The name of the trash this add made,
/// for the caller that has to tell its own from an earlier add's.
/// Why:
///   The own name is the one entry of the trash parent that this add must not move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trashName = (trash: Trash) => trash.name;
/// ```
fn trash_name(trash: &Trash) -> &str {
    return trash.name.as_str();
}

/// What:
///  Whether a listed entry is a directory.
/// Why:
///   Only directories are trash entries;
/// a file the person put there is none of this add's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isDirectoryEntry = (entry: DirEntry) => entry.fileType().isDirectory();
/// ```
fn is_directory_entry(entry: &std::fs::DirEntry) -> bool {
    return entry.file_type().is_ok_and(is_dir_kind);
}

/// Whether a file type is a directory's.
fn is_dir_kind(kind: std::fs::FileType) -> bool {
    return kind.is_dir();
}

/// What:
///  The directories an earlier add left in the trash parent of the snapshot,
/// as paths below the snapshot,
///  except this add's own trash.
/// Why:
///   A kill before the emptying leaves them there,
/// and the walk skips the trash parent so that this add's own trash is
///  not moved into itself;
/// naming them from the parent's listing is what tells the two apart.
/// The parent is reached through the
///  held handle of the snapshot and listed once,
/// so a name that is not text is carried by its bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdir(join(stage, TRASH_PARENT)).filter((name) => name !== trash?.name).map((name) => join(TRASH_PARENT, name));
/// ```
pub(super) fn inherited_trash_entries(stage: &Path, aside: &Aside) -> Result<Vec<PathBuf>, String> {
    let parent: PathBuf = stage.join(TRASH_PARENT);
    let mut found: Vec<PathBuf> = Vec::new();
    let listed = match std::fs::read_dir(parent.as_path()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(found),
        Err(error) => return Err(format!("{} cannot be listed ({error})", parent.display())),
    };
    let own: Option<&str> = aside.trash.as_ref().map(trash_name);
    for listed_entry in listed {
        let entry: std::fs::DirEntry = path_failure(listed_entry, parent.as_path(), "cannot be listed")?;
        let name: OsString = entry.file_name();
        if own == name.to_str() || !is_directory_entry(&entry) {
            continue;
        }
        found.push(Path::new(TRASH_PARENT).join(name));
    }
    return Ok(found);
}
