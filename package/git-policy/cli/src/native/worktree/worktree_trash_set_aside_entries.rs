//! What:
//!  The entries a snapshot's leave-out sets aside:
//! the record of where they go and how many were moved,
//! the checks and sweeps it applies to them,
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
//! and the record the walk keeps of them,
//! kept
//!  in a file of their own so the walk itself stays readable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { namesTheContainer, removeContainersBeneath, inheritedTrashEntries } = setAsideEntries;
//! ```

use crate::fs_beneath::{
    ObjectId,
    open_directory_at,
    plain_components,
};
use crate::fs_beneath_walk_chain::open_directory_beneath;
use crate::the_names_and_the_manifest_of_the_trash::TRASH_PARENT;
use crate::worktree_copy_selection::STAGE_PREFIX;
use crate::worktree_path_failure_message::path_failure;
use crate::worktree_trash_manifest::{
    entry_name,
    is_trash_name,
};
use crate::worktree_trash_remove::remove_tree_beneath;
use crate::worktree_trash_set_aside::Trash;
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::File;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{
    Path,
    PathBuf,
};

/// Where left-out directories go:
/// into one trash directory when there is one,
/// else they are removed in place.
pub(super) struct Aside {
    /// The snapshot,
    /// held open;
    /// every directory moved or removed is reached from it by plain names.
    pub(super) root: File,
    /// The path of the snapshot,
    /// for messages.
    pub(super) stage: PathBuf,
    /// The trash directory,
    /// absent when the leave-out removes in the foreground.
    pub(super) trash: Option<Trash>,
    /// The identity of each directory moved into the trash,
    /// in the order moved.
    pub(super) entries: Vec<ObjectId>,
    /// How many directories were set aside.
    pub(super) count: usize,
}

impl Aside {
    /// The held directory that contains `relative`,
    /// and the entry's name in it.
    fn holder(
        &self,
        relative: &Path,
    ) -> Result<(File, OsString), String> {
        let names: Vec<OsString> = path_failure(
            plain_components(relative),
            relative,
            "is not a plain path inside the snapshot",
        )?;
        let Some((name, parents)) = names.split_last() else {
            return Err(format!("{} names the snapshot itself", relative.display()));
        };
        let parent: File = path_failure(
            open_directory_beneath(&self.root, parents),
            self.stage.join(relative).as_path(),
            "is reached through a link or another mount in the snapshot",
        )?;
        return Ok((parent, name.clone()));
    }

    /// Set one directory aside by renaming it into the trash,
    /// or remove it when there is no trash or when it is another command's stage
    ///  container;
    /// `relative` is its path below the snapshot.
    /// Why:
    ///   A stage container holds another worktree's ignored state,
    /// credentials included,
    /// and it is removed here,
    ///  before the snapshot is installed,
    /// because the trash stands inside the new worktree until the emptying runs and a
    ///  command that runs `git add --all` in that window stages what it holds;
    /// emptying the trash afterwards does not
    ///  take those blobs out of the index.
    /// Everything else still waits for the emptying,
    /// so the add pays only the
    ///  container's own removal.
    pub(super) fn put(
        &mut self,
        relative: &Path,
    ) -> Result<(), String> {
        // A container is removed here and never renamed into the trash: the pattern is None for it either way.
        let direct_container: bool = names_the_container(relative);
        if !direct_container {
            // A directory set aside whole can hold one, because a container stands beside the destination of one add and a
            //  nested worktree is another such place; the sweep removes those before the rename, so a kill before the
            //  emptying cannot leave another command's credentials in the trash.
            let (holder, name): (File, OsString) = self.holder(relative)?;
            remove_containers_beneath(&holder, name.as_os_str(), &self.stage.join(relative))?;
        }
        let (parent, name): (File, OsString) = self.holder(relative)?;
        let shown: PathBuf = self.stage.join(relative);
        let into_trash: Option<&Trash> = match direct_container {
            true => None,
            false => self.trash.as_ref(),
        };
        match into_trash {
            Some(trash) => {
                let to: String = entry_name(self.entries.len());
                let moved: ObjectId =
                    trash.set_aside(&parent, name.as_os_str(), to.as_str(), shown.as_path())?;
                self.entries.push(moved);
            }
            None => {
                path_failure(
                    remove_tree_beneath(
                        shown.parent().unwrap_or(self.stage.as_path()),
                        &parent,
                        name.as_os_str(),
                        None,
                    ),
                    shown.as_path(),
                    "cannot be removed",
                )?;
            }
        }
        self.count += 1;
        return Ok(());
    }
}

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
/// A directory nobody could look inside is never
///  taken for one with nothing to sweep:
/// it is a reason for the caller to keep what holds it where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeContainersBeneath(holder, name, shown) { for (const entry of readdir(holder, name)) { if (entry.name.startsWith(STAGE_PREFIX)) await removeTreeBeneath(...); else if (entry.isDirectory()) recurse; } }
/// ```
pub(super) fn remove_containers_beneath(
    holder: &File,
    name: &OsStr,
    shown: &Path,
) -> Result<(), String> {
    let directory: File = path_failure(
        open_directory_at(holder, name),
        shown,
        "is not a plain directory",
    )?;
    let mut children: Vec<(OsString, Option<bool>)> = Vec::new();
    for listed in path_failure(
        std::fs::read_dir(crate::fs_beneath::handle_path(&directory)),
        shown,
        "cannot be listed",
    )? {
        let entry: std::fs::DirEntry = path_failure(listed, shown, "cannot be listed")?;
        children.push((entry.file_name(), listed_directory_of(&entry)));
    }
    for (child, listed_directory) in children {
        let path: PathBuf = shown.join(&child);
        if names_the_container(Path::new(child.as_os_str())) {
            path_failure(
                remove_tree_beneath(shown, &directory, child.as_os_str(), None),
                path.as_path(),
                "cannot be removed",
            )?;
            continue;
        }
        // The listing already carries whether the entry is a directory, so only a listing that does not say costs a
        //  lookup of its own.
        let is_directory: bool = match listed_directory {
            Some(found) => found,
            None => is_directory_entry_at(&directory, child.as_os_str(), path.as_path())?,
        };
        if !is_directory {
            continue;
        }
        remove_containers_beneath(&directory, child.as_os_str(), path.as_path())?;
    }
    return Ok(());
}

/// What:
///  Whether a listed entry is a directory,
/// not a link to one,
/// as far as the listing said,
/// or nothing when the listing did not say.
/// Why:
///   A walk reads every directory's entries anyway,
/// and the type a listing hands back is the entry's own;
/// re-examining the
///  name would be one lookup per entry of every directory the leave-out sets aside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listedDirectoryOf = (entry) => entry.fileType().map((kind) => kind.isDirectory());
/// ```
fn listed_directory_of(entry: &std::fs::DirEntry) -> Option<bool> {
    return entry.file_type().ok().map(is_dir_kind);
}

/// What:
///  Whether the entry `name` of `directory` is a directory,
/// not a link to one,
/// and the reason when nobody could look at it.
/// Why:
///   Only a directory is walked,
/// so a link named like a directory is never followed out of the snapshot.
/// The answer comes from the held handle,
/// so no path is resolved twice.
/// A subtree the sweep could not examine is not a subtree with nothing in it:
/// an entry that
///  cannot be looked at is a reason to keep what holds it where it is,
/// and the caller names it.
/// An entry that
///  is gone by now is neither,
/// because there is nothing left to sweep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isDirectoryEntryAt = (directory, name, shown) => { let found; try { found = lstatSync(join(handlePath(directory), name)); } catch (error) { if (error.code === 'ENOENT') return false; throw `${shown} cannot be examined (${error})`; } return found.isDirectory(); };
/// ```
fn is_directory_entry_at(
    directory: &File,
    name: &OsStr,
    shown: &Path,
) -> Result<bool, String> {
    let found: std::fs::Metadata = match crate::fs_beneath::entry_metadata_at(directory, name) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return path_failure::<bool, std::io::Error>(Err(error), shown, "cannot be examined");
        }
    };
    return Ok(is_directory_metadata(found));
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
    let Some(name) = relative.file_name() else {
        return false;
    };
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
    return trash.name();
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
/// Only a directory named the
///  way the wrapper names a trash is one:
/// the parent is kept out of every commit,
/// so a person may keep their own
///  directories there,
/// and taking those for an earlier add's trash moved them into this add's trash,
/// where the
///  service deleted them.
/// The parent is reached through the held handle of the snapshot and listed once,
/// so a name
///  that is not text is carried by its bytes.
///   A trash is recognised by the name the wrapper gives one,
/// and what this add takes over is identified again at the move,
/// so the
///  emptying deletes only the directory it moved;
/// the manifest inside a trash is what lets a record-less one be emptied
///  outside a copy (`worktree_trash_own_manifest.rs`),
/// and a copy re-creates the identities it names,
/// so it decides
///  nothing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdir(join(stage, TRASH_PARENT)).filter((name) => isTrashName(name) && name !== trash?.name && provesTheWrapperMadeIt(stage, name)).map((name) => join(TRASH_PARENT, name));
/// ```
pub(super) fn inherited_trash_entries(
    stage: &Path,
    aside: &Aside,
) -> Result<Vec<PathBuf>, String> {
    let parent: PathBuf = stage.join(TRASH_PARENT);
    let mut found: Vec<PathBuf> = Vec::new();
    let listed = match std::fs::read_dir(parent.as_path()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(found),
        Err(error) => return Err(format!("{} cannot be listed ({error})", parent.display())),
    };
    let own: Option<&str> = aside.trash.as_ref().map(trash_name);
    for listed_entry in listed {
        let entry: std::fs::DirEntry =
            path_failure(listed_entry, parent.as_path(), "cannot be listed")?;
        let name: OsString = entry.file_name();
        if own == name.to_str() || !is_directory_entry(&entry) {
            continue;
        }
        // A name that is not the wrapper's shape of a trash is the person's own directory, and the snapshot carries it into the new
        //  worktree as it is.
        if !name.to_str().is_some_and(is_trash_name) {
            continue;
        }
        found.push(Path::new(TRASH_PARENT).join(name));
    }
    return Ok(found);
}
