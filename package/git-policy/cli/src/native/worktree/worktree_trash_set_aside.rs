//! What:
//!  The trash directory one snapshot's leave-out made and holds open,
//! the one the stand-in step puts a whole
//!  worktree folder into,
//! the two ways it is made or opened again,
//! and the moves in and out of it.
//! Why:
//!  Both steps set directories aside for the same commit service in the same work item,
//! so the move into the trash
//!  and the manifest it answers with are one operation however many steps use it.
//! A directory may be exchanged for another between the look at it and its move,
//! so only what the move
//!  carried may be deleted later,
//! and the identity is therefore compared across every rename.
//! A folder that is taken whole is checked before the rename that puts it here,
//! and the copy that arrived is looked
//!  at again before the service is told to delete it,
//! because a directory somebody made in between is never deleted with the
//!  stand-ins it replaced.
//! The moves are kept in a file of their own so the leave-out's walk stays readable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { Trash, makeTrash, reopenTrash } = setAside;
//! ```

use super::{TRASH_PARENT, TrashManifest};
use crate::diagnostic_log::debug;
use crate::fs_beneath::{ObjectId, identity_at, identity_of, make_directory_at, open_directory_at, open_directory_beneath, plain_components, rename_at};
use crate::random_id::random_uuid;
use crate::worktree_path_failure_message::{path_failure, subject_failure};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeTrashSetAside";

/// A name for one trash directory:
/// forty hexadecimal digits,
/// which is also the object name the work item carries.
/// One random identifier for the name of a trash directory;
/// the reason is the error.
fn fresh_uuid() -> Result<String, String> {
    return match random_uuid() {
        Ok(made) => Ok(made),
        Err(error) => Err(format!("a name for the trash could not be made: {error:?}")),
    };
}

fn new_trash_name() -> Result<String, String> {
    let first: String = fresh_uuid()?;
    let second: String = fresh_uuid()?;
    let digits: String = format!("{first}{second}").chars().filter(char::is_ascii_hexdigit).collect();
    return Ok(digits.chars().take(40).collect::<String>().to_ascii_lowercase());
}

/// What:
///  The trash directory one snapshot's leave-out made and holds open,
/// and the one the stand-in step puts a whole worktree
///  folder into when the leave-out moved nothing.
/// Why:
///   Both steps set directories aside for the same commit service in the same work item,
/// so the move into the trash
///  and the manifest it answers with are one operation however many steps use it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Trash = { dir: File; name: string; id: ObjectId; parent: File };
/// ```
pub(crate) struct Trash {
    /// The trash directory,
    /// held open so every move names an entry of this object.
    dir: File,
    /// Its name below the trash parent.
    name: String,
    /// Its identity,
    /// taken when it was made.
    id: ObjectId,
    /// The directory that holds it,
    /// for removing it again when nothing is moved into it.
    parent: File,
}

impl Trash {
    /// What:
    ///  Move the entry `name` of `from` into this trash as `to`,
    /// and answer the identity it has there.
    /// Why:
    ///   A directory may be exchanged for another between the look at it and its move,
    /// and only what the move
    ///  carried may be deleted later,
    /// so the identity is compared across the rename.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const id = trash.setAside({ from, name: 'topic', to: 'd3', shown: path }); // throws the reason
    /// ```
    pub(crate) fn set_aside(&self, from: &File, name: &OsStr, to: &str, shown: &Path) -> Result<ObjectId, String> {
        let looked: ObjectId = path_failure(identity_at(from, name), shown, "cannot be examined")?;
        path_failure(rename_at(from, name, &self.dir, OsStr::new(to), false), shown, "cannot be set aside")?;
        let moved: ObjectId = path_failure(identity_at(&self.dir, OsStr::new(to)), shown, "cannot be examined in the trash")?;
        if moved != looked {
            return Err(format!("{} changed while it was set aside ({} instead of {})", shown.display(), moved.encode(), looked.encode()));
        }
        return Ok(moved);
    }

    /// What:
    ///  The name this trash has below the trash parent.
    /// Why:
    ///   The leave-out must not set aside the trash it is filling,
    /// and that entry is found by the name the trash was made with.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trash.name() // 'd3...' or the forty digits it was made with
    /// ```
    pub(crate) fn name(&self) -> &str {
        return self.name.as_str();
    }

    /// What:
    ///  The folder `name` this trash holds,
    /// when the identity `moved` proves it is the entry that was set aside there;
    ///  nothing when it is not there any more.
    /// Why:
    ///   A folder that is taken whole is checked before the rename that puts it here,
    /// so the copy that arrived is
    ///  looked at again before the service is told to delete it;
    /// a directory somebody made between the check and the rename goes
    ///  with no stand-in,
    /// and only the entry that was set aside is ever looked at.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const held = trash.heldEntry({ name: 'd3', moved: id }); // throws when the entry is another object
    /// ```
    pub(crate) fn held_entry(&self, name: &str, moved: ObjectId) -> Result<Option<File>, String> {
        let looked: ObjectId = match identity_at(&self.dir, OsStr::new(name)) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("{name} cannot be examined in the trash ({error})")),
        };
        if looked != moved {
            return Err(format!("{name} in the trash is not the entry that was set aside there ({} instead of {})", looked.encode(), moved.encode()));
        }
        return match open_directory_at(&self.dir, OsStr::new(name)) {
            Ok(found) => Ok(Some(found)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("{name} in the trash is not a plain directory ({error})")),
        };
    }

    /// What:
    ///  Move the entry `name` this trash holds back to `to` below `from`,
    ///  and prove that what arrived is the entry that was taken.
    /// Why:
    ///   A folder whose entries changed after the check is not deleted with them;
    /// it goes back where it was,
    /// and the
    ///  identity is compared across the rename,
    /// so only the folder that was looked at is put back.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trash.putBack({ name: 'd3', from: parent, to: 'worktrees' }); // throws the reason
    /// ```
    pub(crate) fn put_back(&self, name: &str, from: &File, to: &OsStr, shown: &Path) -> Result<(), String> {
        let looked: ObjectId = path_failure(identity_at(&self.dir, OsStr::new(name)), shown, "cannot be examined in the trash")?;
        path_failure(rename_at(&self.dir, OsStr::new(name), from, to, false), shown, "cannot be put back where it was")?;
        let arrived: ObjectId = path_failure(identity_at(from, to), shown, "cannot be examined")?;
        if arrived != looked {
            return Err(format!("{} changed while it was put back ({} instead of {})", shown.display(), arrived.encode(), looked.encode()));
        }
        return Ok(());
    }

    /// What:
    ///  Move the entry `name` this trash holds into the trash parent as `kept`,
    /// and prove that what arrived is the entry that was taken.
    /// Why:
    ///   A folder that changed after it was checked is not deleted with its stand-ins,
    /// and the trash entry name a
    ///  manifest would name is not its place any more,
    /// so one rename takes it beside the trash,
    /// where no later set-aside
    ///  finds its name taken;
    /// the move stays inside the snapshot because a folder that holds a stand-in of a nested
    ///  subvolume cannot be renamed out of the snapshot at all.
    /// The identity is compared across the rename,
    /// so only the
    ///  folder this call took out is named as the kept one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trash.keepBeside({ name: 'd3', as: '.cli-git-worktree-kept-<digits>', shown }); // throws the reason
    /// ```
    pub(crate) fn keep_beside(&self, name: &str, kept: &str, shown: &Path) -> Result<(), String> {
        let looked: ObjectId = path_failure(identity_at(&self.dir, OsStr::new(name)), shown, "cannot be examined in the trash")?;
        path_failure(rename_at(&self.dir, OsStr::new(name), &self.parent, OsStr::new(kept), false), shown, "cannot be kept beside the trash")?;
        let arrived: ObjectId = path_failure(identity_at(&self.parent, OsStr::new(kept)), shown, "cannot be examined beside the trash")?;
        if arrived != looked {
            return Err(format!("{} changed while it was kept beside the trash ({} instead of {})", shown.display(), arrived.encode(), looked.encode()));
        }
        return Ok(());
    }

    /// What:
    ///  Remove the trash directory itself when nothing was moved into it,
    /// so no empty trash is left in the new worktree.
    /// Why:
    ///   The trash is made before the first directory is set aside,
    /// and a leave-out or a fold that sets aside
    ///  nothing must not leave a folder behind that no record names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trash.discard(); // a refusal only leaves an empty directory
    /// ```
    pub(crate) fn discard(&self) {
        if let Err(error) = crate::fs_beneath::remove_at(&self.parent, OsStr::new(self.name.as_str()), true) {
            debug(TAG, format!("the empty trash {} could not be removed ({error})", self.name).as_str());
        }
    }

    /// What:
    ///  The manifest of this trash with the identities moved into it,
    /// or nothing and no trash when it holds none.
    /// Why:
    ///   One trash is one work item,
    /// so the identity of the trash directory and of everything in it travel together;
    ///  a trash that holds nothing would name nothing to delete.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const manifest = trash.finish(entries); // undefined when entries is empty
    /// ```
    pub(crate) fn finish(self, entries: Vec<ObjectId>) -> Option<TrashManifest> {
        if entries.is_empty() {
            self.discard();
            return None;
        }
        return Some(TrashManifest { name: self.name, trash: self.id, entries });
    }
}

/// What:
///  Make the trash directory below the trash parent in the snapshot,
/// one plain name at a time;
/// the reason is the error.
/// Why:
///   The leave-out makes it for what it sets aside,
/// and the stand-in step makes it when the leave-out set nothing aside
///  and a worktree folder holds stand-ins to take away.
///
pub(crate) fn make_trash(root: &File) -> Result<Trash, String> {
    let names: Vec<OsString> = match plain_components(Path::new(TRASH_PARENT)) {
        Ok(found) => found,
        Err(error) => return Err(format!("the trash parent is not a plain path ({error})")),
    };
    let mut parent: File = subject_failure(root.try_clone(), &"the snapshot", "cannot be held")?;
    for name in &names {
        subject_failure(make_directory_at(&parent, name.as_os_str(), 0o755), &name.to_string_lossy(), "cannot be made in the snapshot")?;
        parent = subject_failure(open_directory_at(&parent, name.as_os_str()), &name.to_string_lossy(), "is not a plain directory in the snapshot")?;
    }
    let name: String = new_trash_name()?;
    let trash_subject: String = format!("the trash {name}");
    if !subject_failure(make_directory_at(&parent, OsStr::new(name.as_str()), 0o700), &trash_subject, "cannot be made")? {
        return Err(format!("the trash {name} already exists"));
    }
    let dir: File = subject_failure(open_directory_at(&parent, OsStr::new(name.as_str())), &trash_subject, "cannot be opened")?;
    let id: ObjectId = subject_failure(identity_of(&dir), &trash_subject, "cannot be examined")?;
    return Ok(Trash { dir, name, id, parent });
}

/// What:
///  The trash a manifest names,
/// opened again below the held snapshot,
/// after checking that the directory is the one
///  the manifest recorded;
/// the reason is the error.
/// Why:
///   A later step of the same add puts a whole worktree folder in that trash,
/// and one record names one trash,
///  so the stand-ins must join the trash the leave-out made rather than make a second one;
/// only the identity the
///  manifest carries proves the directory is still the one the leave-out made.
///
pub(crate) fn reopen_trash(root: &File, manifest: &TrashManifest) -> Result<Trash, String> {
    let names: Vec<OsString> = subject_failure(plain_components(Path::new(TRASH_PARENT)), &TRASH_PARENT, "is not a plain path")?;
    let parent: File = subject_failure(open_directory_beneath(root, names.as_slice()), &TRASH_PARENT, "cannot be opened in the snapshot")?;
    let shown: String = Path::new(TRASH_PARENT).join(manifest.name.as_str()).display().to_string();
    let dir: File = subject_failure(open_directory_at(&parent, OsStr::new(manifest.name.as_str())), &shown, "is not a plain directory in the snapshot")?;
    let found: ObjectId = subject_failure(identity_of(&dir), &shown, "cannot be examined")?;
    if found != manifest.trash {
        return Err(format!("{shown} is not the trash the leave-out made ({} instead of {}), so nothing is put in it", found.encode(), manifest.trash.encode()));
    }
    return Ok(Trash { dir, name: manifest.name.clone(), id: found, parent });
}
