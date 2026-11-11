//! What:
//!  Remove a directory tree beneath a held parent directory:
//! every directory is opened by one plain name below the handle of
//!  its parent without following a link,
//! and every entry is unlinked from the handle of the directory that holds it.
//! Why:
//!  A path string is resolved again by every call that takes it,
//! so a directory replaced by a link between a check and the
//!  removal redirects the removal.
//! Here no call resolves a path:
//! a link,
//! a mount point or a directory on another device is
//!  an error that names it,
//! and the identity of the top directory is compared with the recorded one on the object that is
//!  opened,
//! so a replacement is never emptied.
//! A whole first pass looks at every directory below the tree and deletes nothing
//!  (`subvolume_remove_walk.rs`):
//! a nested subvolume or a mount point anywhere in it refuses the removal
//!  before one entry is unlinked,
//! so the ordinary directories beside it are still there when the reason comes back and the person
//!  decides what happens to the tree.
//! The one directory of another device that is no boundary is the stand-in btrfs shows where a snapshot left a nested
//!  subvolume out:
//! it holds nothing and belongs to the tree being removed,
//! so it is removed as the empty directory it
//!  is (`subvolume_remove_walk.rs` makes the same exception for the same reason).
//! A directory the owner cannot write to is given the owner's permission bits
//!  through its descriptor on the way down,
//! as `subvolume_remove.rs` does for the old tree of a conversion.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeTreeBeneath({ label, parent, name, expected }); // throws a reason; the entry is left as it is when it is not the expected object
//! ```

use crate::fs_beneath::{ObjectId, device_of, handle_path, identity_at, identity_of, open_directory_at, remove_at};
use crate::subvolume_remove_walk::verify_below;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeTrashRemove";

/// What:
///  The inode number btrfs gives the empty directory that stands for a nested subvolume in a snapshot
/// (`BTRFS_EMPTY_SUBVOL_DIR_OBJECTID`),
/// read from the module that owns the number.
/// Why:
///   Such a directory reports a device of its own and holds nothing,
/// so it is no boundary of the tree being removed;
///  the leave-out and the stand-in step read the same number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STAND_IN_INODE = BTRFS_STAND_IN_INODE;
/// ```
const BTRFS_STAND_IN_INODE: u64 = crate::fs_kind::BTRFS_STAND_IN_INODE;

/// What a removal found.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Removal {
    /// The tree was removed.
    Removed,
    /// Nothing was at the name.
    Absent,
}

/// How a message names an entry:
/// built only when a message is made,
/// so a removal that succeeds builds no path.
#[derive(Clone, Copy)]
struct Describe<'a> {
    /// The path of the top of the tree being removed.
    top: &'a Path,
    /// The names below the top that lead to the directory the message is about.
    names: &'a [OsString],
    /// One more name below those,
    /// when the message is about an entry of that directory.
    entry: Option<&'a OsStr>,
}

impl<'a> Describe<'a> {
    /// The description of the directory reached from `top` by `names`.
    fn new(top: &'a Path, names: &'a [OsString]) -> Self {
        return Self { top, names, entry: None };
    }

    /// The description of the entry `entry` of the directory this one names.
    fn with_entry(self, entry: &'a OsStr) -> Self {
        return Self { entry: Some(entry), ..self };
    }

    /// The path the message names;
    /// it is joined only when a message is made.
    fn path(&self) -> PathBuf {
        let mut path: PathBuf = self.top.to_path_buf();
        for part in self.names {
            path.push(part);
        }
        if let Some(entry) = self.entry {
            path.push(entry);
        }
        return path;
    }
}

/// What:
///  A step's outcome with a failure turned into the message `<path> <what> (<error>)`,
/// the path made only on a failure.
/// Why:
///   Every step of the removal words its refusal this way,
/// and a named function stands where a closure would capture the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failedAt<T>(outcome: () => T, shown: Describe, what: string): T { try { return outcome(); } catch (error) { throw `${shown.path()} ${what} (${error})`; } }
/// ```
fn failed_at<T>(outcome: std::io::Result<T>, shown: &Describe<'_>, what: &str) -> Result<T, String> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("{} {what} ({error})", shown.path().display())),
    };
}

/// One directory of the walk:
/// its handle,
/// its name in the directory above,
/// and the entries still to remove.
struct Frame {
    /// The directory,
    /// held open so every call below it names a plain entry of this object.
    dir: File,
    /// Its name in the directory above.
    name: OsString,
    /// The entries not yet removed,
    /// each with whether the listing says it is a directory.
    entries: Vec<(OsString, bool)>,
}

/// Whether an error says the entry is already gone.
fn is_gone(error: &std::io::Error) -> bool {
    return error.kind() == std::io::ErrorKind::NotFound;
}

/// The names in an open directory,
/// each with whether it is a directory (a link is not one).
fn list_entries(dir: &File, shown: Describe<'_>) -> Result<Vec<(OsString, bool)>, String> {
    let listing = failed_at(std::fs::read_dir(handle_path(dir)), &shown, "cannot be listed")?;
    let mut entries: Vec<(OsString, bool)> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = failed_at(listed, &shown, "cannot be listed")?;
        let name: OsString = entry.file_name();
        let is_directory: bool = failed_at(entry.file_type(), &shown.with_entry(name.as_os_str()), "cannot be examined")?.is_dir();
        entries.push((name, is_directory));
    }
    return Ok(entries);
}

/// Give the owner read,
/// write and search permission on an open directory;
/// a directory it cannot be given them on keeps its
///  mode and the removal that follows reports the operating system's refusal.
fn make_writable(dir: &File, shown: Describe<'_>) {
    let Ok(metadata) = dir.metadata() else { return };
    let mode: u32 = metadata.permissions().mode();
    if mode & 0o700 == 0o700 {
        return;
    }
    if let Err(error) = dir.set_permissions(std::fs::Permissions::from_mode(mode | 0o700)) {
        crate::diagnostic_log::debug(TAG, format!("{} keeps its mode ({error})", shown.path().display()).as_str());
    }
}

/// Open the directory `name` below `parent` and describe it as a frame;
/// the reason is the error.
fn enter(parent: &File, name: &OsStr, device: u64, shown: Describe<'_>) -> Result<Frame, String> {
    let dir: File = failed_at(open_directory_at(parent, name), &shown, "is not a plain directory that can be entered, so it is left alone")?;
    let facts: std::fs::Metadata = failed_at(dir.metadata(), &shown, "cannot be examined")?;
    // A stand-in of a nested subvolume is the one directory here with a device of its own that is no boundary: btrfs
    //  builds it for a subvolume a snapshot left out, it holds nothing, and it belongs to the tree being removed.
    if facts.dev() != device && facts.ino() != BTRFS_STAND_IN_INODE {
        return Err(format!("{} is on another device, so it is left alone", shown.path().display()));
    }
    make_writable(&dir, shown);
    let entries: Vec<(OsString, bool)> = list_entries(&dir, shown)?;
    return Ok(Frame { dir, name: name.to_os_string(), entries });
}

/// What:
///  Remove the directory `name` below `parent` with everything in it;
/// `label` is the path of `parent`,
/// used only in messages.
/// Why:
///   With `expected` set,
/// the directory is removed only when it is that object:
/// the identity is compared by name before the
///  directory is opened and again on the opened object,
/// so a replacement made between the two is left alone.
/// The top
///  directory must be on the device of `parent`.
/// Nothing is followed and nothing outside the directory is touched.
/// An entry
///  that vanishes meanwhile counts as removed.
/// A boundary below the top directory is refused before one entry is unlinked,
/// and one that appears
///  afterwards is refused again where it is met.
/// Directories are entered with a work stack,
/// so depth costs no recursion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = await removeTreeBeneath({ label, parent, name, expected }); // 'removed' | 'absent'
/// ```
pub fn remove_tree_beneath(label: &Path, parent: &File, name: &OsStr, expected: Option<ObjectId>) -> Result<Removal, String> {
    let top: PathBuf = label.join(name);
    let device: u64 = failed_at(device_of(parent), &Describe::new(label, &[]), "cannot be examined")?;
    match identity_at(parent, name) {
        Ok(found) => {
            if let Some(wanted) = expected && found != wanted {
                return Err(format!("{} is not the object that was set aside ({} instead of {}), so it is left alone", top.display(), found.encode(), wanted.encode()));
            }
        }
        Err(error) if is_gone(&error) => return Ok(Removal::Absent),
        Err(error) => return Err(format!("{} cannot be examined ({error})", top.display())),
    }
    let first: Frame = enter(parent, name, device, Describe::new(top.as_path(), &[]))?;
    if let Some(wanted) = expected {
        let opened: ObjectId = failed_at(identity_of(&first.dir), &Describe::new(top.as_path(), &[]), "cannot be examined")?;
        if opened != wanted {
            return Err(format!("{} is not the object that was set aside ({} instead of {}), so it is left alone", top.display(), opened.encode(), wanted.encode()));
        }
    }
    // The whole tree is looked at first, and nothing is deleted: a nested subvolume or a mount point anywhere in it
    //  refuses the removal before one entry is unlinked, so the person still has every file the removal was not allowed
    //  to destroy. The walk below checks the same boundary again at each directory, which catches one that appeared in
    //  between.
    verify_below(&first.dir, top.as_path()).map_err(|refused| format!("{} cannot be removed ({refused})", top.display()))?;
    let mut stack: Vec<Frame> = vec![first];
    // The names from the top directory down to the frame being worked on, for messages.
    let mut shown: Vec<OsString> = Vec::new();
    // The path of the frame being worked on, from the names kept in `shown`.
    loop {
        let Some(frame) = stack.last_mut() else { break };
        match frame.entries.pop() {
            Some((entry, true)) => {
                let next: Frame = enter(&frame.dir, entry.as_os_str(), device, Describe::new(top.as_path(), shown.as_slice()).with_entry(entry.as_os_str()))?;
                shown.push(entry);
                stack.push(next);
            }
            Some((entry, false)) => match remove_at(&frame.dir, entry.as_os_str(), false) {
                Ok(()) => {}
                Err(error) if is_gone(&error) => {}
                Err(error) => return Err(format!("{} cannot be removed ({error})", Describe::new(top.as_path(), shown.as_slice()).with_entry(entry.as_os_str()).path().display())),
            },
            None => {
                let Some(done) = stack.pop() else { break };
                let holder: &File = match stack.last() {
                    Some(above) => &above.dir,
                    None => parent,
                };
                match remove_at(holder, done.name.as_os_str(), true) {
                    Ok(()) => {}
                    Err(error) if is_gone(&error) => {}
                    Err(error) => return Err(format!("{} cannot be removed ({error})", Describe::new(top.as_path(), shown.as_slice()).path().display())),
                }
                shown.pop();
            }
        }
    }
    return Ok(Removal::Removed);
}

/// Tree removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_trash_remove_tests.rs"]
mod tests;
