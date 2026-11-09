//! What:
//!  Remove the old tree of a conversion,
//! but only the directory the journal recorded,
//! pinned by a descriptor from the moment
//!  its identity is checked until its last entry is gone.
//! Why:
//!  A device and inode read from a path say nothing about what the path names a moment later:
//! another process can rename the
//!  recorded directory away and put an unrelated one at the name before a recursive delete resolves it again.
//! The directory
//!  is opened below its parent without following a link,
//! its identity is read from that descriptor,
//! its entries are removed
//!  through the descriptor's own path,
//! and the last step removes the name only while it still names the same object and only
//!  if that object is empty (a directory removal cannot take a tree).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await removeRecordedOldTree({ platform, parent, name, oldRoot, installed }); // 'removed' | 'absent' | { kept: reason } | { differs: reason }
//! ```

use crate::fs_beneath::{ObjectId, handle_path, identity_at, identity_of, open_directory_at, remove_at};
use crate::subvolume_error::MigrateError;
use crate::subvolume_platform::Platform;
use crate::subvolume_remove::remove_tree;
use crate::worktree_convert_compare::first_difference_after;
use crate::worktree_convert_journal::{io_step, is_recorded_object};
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// What a request to remove the old tree did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Removal {
    /// The recorded directory was removed.
    Removed,
    /// Nothing was at the name.
    Absent,
    /// The name holds something else,
    /// or the directory moved;
    /// nothing was deleted,
    /// for this reason.
    Kept(String),
    /// The old tree holds something the installed tree does not hold in the same form,
    /// or something this command does not
    ///  delete;
    /// nothing was deleted or copied,
    /// for this reason.
    Differs(String),
}

/// What:
///  Remove the directory `name` of `parent` if it is the old root,
/// deleting beneath a descriptor on it.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeRecordedOldTree(platform, parent, name, oldRoot, installed);
/// ```
pub fn remove_recorded_old_tree(platform: &dyn Platform, parent: &std::fs::File, name: &OsStr, old: ObjectId, installed: &Path, synced_ms: Option<i64>) -> Result<Removal, MigrateError> {
    return remove_recorded_old_tree_seamed(platform, parent, name, old, installed, synced_ms, &mut do_nothing);
}

/// What:
///  A function that does nothing,
/// for the call that runs between pinning the old tree and listing it.
/// Why:
///   The real conversion has nothing to do at that point;
/// only a control puts work there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const doNothing = () => {};
/// ```
fn do_nothing() {}

/// `remove_recorded_old_tree` with a call right after the directory was pinned and judged to be the old tree,
/// where a control
///  moves it away and puts another directory at the name.
pub fn remove_recorded_old_tree_seamed(platform: &dyn Platform, parent: &std::fs::File, name: &OsStr, old: ObjectId, installed: &Path, synced_ms: Option<i64>, after_pinned: &mut dyn FnMut()) -> Result<Removal, MigrateError> {
    let shown: PathBuf = PathBuf::from(name);
    let pinned: std::fs::File = match open_directory_at(parent, name) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Removal::Absent),
        Err(error) => return Ok(Removal::Kept(format!("{} cannot be opened as a directory below its parent ({error}), so it is not the old tree this conversion recorded", shown.display()))),
    };
    let found = io_step(identity_of(&pinned), "examine", shown.as_path())?;
    if !is_recorded_object(&old, &found) {
        return Ok(Removal::Kept(format!("{} is not the old tree this conversion recorded", shown.display())));
    }
    let through: PathBuf = handle_path(&pinned);
    if platform.is_subvolume_root(through.as_path())? {
        return Ok(Removal::Kept(format!("{} is a subvolume, so it is not the old tree this conversion recorded", shown.display())));
    }
    // Nothing is deleted unless the installed tree holds all of the old one; a difference is reported and settled by a person.
    match first_difference_after(through.as_path(), installed, synced_ms) {
        Ok(None) => {}
        Ok(Some(reason)) => return Ok(Removal::Differs(reason)),
        Err(error) => return Ok(Removal::Differs(format!("the old tree cannot be compared with the installed worktree ({error})"))),
    }
    let metadata: std::fs::Metadata = io_step(pinned.metadata(), "examine", shown.as_path())?;
    if std::os::unix::fs::MetadataExt::mode(&metadata) & 0o700 != 0o700 {
        io_step(pinned.set_permissions(std::fs::Permissions::from_mode(std::os::unix::fs::MetadataExt::mode(&metadata) | 0o700)), "make writable", shown.as_path())?;
    }
    after_pinned();
    let listing = io_step(std::fs::read_dir(&through), "list the old tree", shown.as_path())?;
    for item in listing {
        let entry: std::fs::DirEntry = io_step(item, "list the old tree", shown.as_path())?;
        io_step(remove_tree(entry.path().as_path()), "remove the old tree", entry.path().as_path())?;
    }
    match identity_at(parent, name) {
        Ok(still) if is_recorded_object(&old, &still) => {}
        Ok(_) => return Ok(Removal::Kept(format!("{} no longer names the old tree, which is empty and was left where it is", shown.display()))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Removal::Removed),
        Err(error) => return Err(MigrateError::io("examine", shown.as_path(), &error)),
    }
    io_step(remove_at(parent, name, true), "remove the old tree", shown.as_path())?;
    return Ok(Removal::Removed);
}
