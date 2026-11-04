//! What:
//!  Remove the old tree of a conversion, but only the directory the journal recorded, pinned by a descriptor from the moment
//!  its identity is checked until its last entry is gone.
//! Why:
//!  A device and inode read from a path say nothing about what the path names a moment later: another process can rename the
//!  recorded directory away and put an unrelated one at the name before a recursive delete resolves it again. The directory
//!  is opened below its parent without following a link, its identity is read from that descriptor, its entries are removed
//!  through the descriptor's own path, and the last step removes the name only while it still names the same object and only
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
use crate::worktree_convert_compare::first_difference;
use crate::worktree_convert_journal::is_recorded_object;
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
    /// The name holds something else, or the directory moved; nothing was deleted, for this reason.
    Kept(String),
    /// The old tree holds something the installed tree does not hold in the same form, or something this command does not
    ///  delete; nothing was deleted or copied, for this reason.
    Differs(String),
}

/// What:
///  Remove the directory `name` of `parent` if it is the old root, deleting beneath a descriptor on it.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeRecordedOldTree(platform, parent, name, oldRoot, installed);
/// ```
pub fn remove_recorded_old_tree(platform: &dyn Platform, parent: &std::fs::File, name: &OsStr, old: ObjectId, installed: &Path) -> Result<Removal, MigrateError> {
    return remove_recorded_old_tree_seamed(platform, parent, name, old, installed, &mut || {});
}

/// `remove_recorded_old_tree` with a call right after the directory was pinned and judged to be the old tree, where a control
///  moves it away and puts another directory at the name.
pub fn remove_recorded_old_tree_seamed(platform: &dyn Platform, parent: &std::fs::File, name: &OsStr, old: ObjectId, installed: &Path, after_pinned: &mut dyn FnMut()) -> Result<Removal, MigrateError> {
    let shown: PathBuf = PathBuf::from(name);
    let pinned: std::fs::File = match open_directory_at(parent, name) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Removal::Absent),
        Err(error) => return Ok(Removal::Kept(format!("{} cannot be opened as a directory below its parent ({error}), so it is not the old tree this conversion recorded", shown.display()))),
    };
    let found = identity_of(&pinned).map_err(|error: std::io::Error| { return MigrateError::io("examine", shown.as_path(), &error); })?;
    if !is_recorded_object(&old, &found) {
        return Ok(Removal::Kept(format!("{} is not the old tree this conversion recorded", shown.display())));
    }
    let through: PathBuf = handle_path(&pinned);
    if platform.is_subvolume_root(through.as_path())? {
        return Ok(Removal::Kept(format!("{} is a subvolume, so it is not the old tree this conversion recorded", shown.display())));
    }
    // Nothing is deleted unless the installed tree holds all of the old one; a difference is reported and settled by a person.
    match first_difference(through.as_path(), installed) {
        Ok(None) => {}
        Ok(Some(reason)) => return Ok(Removal::Differs(reason)),
        Err(error) => return Ok(Removal::Differs(format!("the old tree cannot be compared with the installed worktree ({error})"))),
    }
    let metadata: std::fs::Metadata = pinned.metadata().map_err(|error: std::io::Error| { return MigrateError::io("examine", shown.as_path(), &error); })?;
    if std::os::unix::fs::MetadataExt::mode(&metadata) & 0o700 != 0o700 {
        pinned.set_permissions(std::fs::Permissions::from_mode(std::os::unix::fs::MetadataExt::mode(&metadata) | 0o700)).map_err(|error: std::io::Error| { return MigrateError::io("make writable", shown.as_path(), &error); })?;
    }
    after_pinned();
    let listing = std::fs::read_dir(&through).map_err(|error: std::io::Error| { return MigrateError::io("list the old tree", shown.as_path(), &error); })?;
    for item in listing {
        let entry: std::fs::DirEntry = item.map_err(|error: std::io::Error| { return MigrateError::io("list the old tree", shown.as_path(), &error); })?;
        remove_tree(entry.path().as_path()).map_err(|error: std::io::Error| { return MigrateError::io("remove the old tree", entry.path().as_path(), &error); })?;
    }
    match identity_at(parent, name) {
        Ok(still) if is_recorded_object(&old, &still) => {}
        Ok(_) => return Ok(Removal::Kept(format!("{} no longer names the old tree, which is empty and was left where it is", shown.display()))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Removal::Removed),
        Err(error) => return Err(MigrateError::io("examine", shown.as_path(), &error)),
    }
    remove_at(parent, name, true).map_err(|error: std::io::Error| { return MigrateError::io("remove the old tree", shown.as_path(), &error); })?;
    return Ok(Removal::Removed);
}
