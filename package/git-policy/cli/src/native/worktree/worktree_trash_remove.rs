//! What:
//!  Remove a directory tree beneath a held parent directory: every directory is opened by one plain name below the handle of
//!  its parent without following a link, and every entry is unlinked from the handle of the directory that holds it.
//! Why:
//!  A path string is resolved again by every call that takes it, so a directory replaced by a link between a check and the
//!  removal redirects the removal. Here no call resolves a path: a link, a mount point or a directory on another device is
//!  an error that names it, and the identity of the top directory is compared with the recorded one on the object that is
//!  opened, so a replacement is never emptied. A directory the owner cannot write to is given the owner's permission bits
//!  through its descriptor on the way down, as `subvolume_remove.rs` does for the old tree of a conversion.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeTreeBeneath({ label, parent, name, expected }); // throws a reason; the entry is left as it is when it is not the expected object
//! ```

use crate::fs_beneath::{ObjectId, device_of, handle_path, identity_at, identity_of, open_directory_at, remove_at};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeTrashRemove";

/// What a removal found.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Removal {
    /// The tree was removed.
    Removed,
    /// Nothing was at the name.
    Absent,
}

/// How a message names an entry: built only when a message is made, so a removal that succeeds builds no path.
type Describe<'a> = &'a dyn Fn() -> PathBuf;

/// One directory of the walk: its handle, its name in the directory above, and the entries still to remove.
struct Frame {
    /// The directory, held open so every call below it names a plain entry of this object.
    dir: File,
    /// Its name in the directory above.
    name: OsString,
    /// The entries not yet removed, each with whether the listing says it is a directory.
    entries: Vec<(OsString, bool)>,
}

/// Whether an error says the entry is already gone.
fn is_gone(error: &std::io::Error) -> bool {
    return error.kind() == std::io::ErrorKind::NotFound;
}

/// The names in an open directory, each with whether it is a directory (a link is not one).
fn list_entries(dir: &File, shown: Describe<'_>) -> Result<Vec<(OsString, bool)>, String> {
    let listing = std::fs::read_dir(handle_path(dir)).map_err(|error: std::io::Error| { return format!("{} cannot be listed ({error})", shown().display()); })?;
    let mut entries: Vec<(OsString, bool)> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error: std::io::Error| { return format!("{} cannot be listed ({error})", shown().display()); })?;
        let is_directory: bool = entry.file_type().map_err(|error: std::io::Error| { return format!("{} cannot be examined ({error})", shown().join(entry.file_name()).display()); })?.is_dir();
        entries.push((entry.file_name(), is_directory));
    }
    return Ok(entries);
}

/// Give the owner read, write and search permission on an open directory; a directory it cannot be given them on keeps its
///  mode and the removal that follows reports the operating system's refusal.
fn make_writable(dir: &File, shown: Describe<'_>) {
    let Ok(metadata) = dir.metadata() else { return };
    let mode: u32 = metadata.permissions().mode();
    if mode & 0o700 == 0o700 {
        return;
    }
    if let Err(error) = dir.set_permissions(std::fs::Permissions::from_mode(mode | 0o700)) {
        crate::diagnostic_log::debug(TAG, format!("{} keeps its mode ({error})", shown().display()).as_str());
    }
}

/// Open the directory `name` below `parent` and describe it as a frame; the reason is the error.
fn enter(parent: &File, name: &OsStr, device: u64, shown: Describe<'_>) -> Result<Frame, String> {
    let dir: File = open_directory_at(parent, name).map_err(|error: std::io::Error| { return format!("{} is not a plain directory that can be entered, so it is left alone ({error})", shown().display()); })?;
    let found: u64 = device_of(&dir).map_err(|error: std::io::Error| { return format!("{} cannot be examined ({error})", shown().display()); })?;
    if found != device {
        return Err(format!("{} is on another device, so it is left alone", shown().display()));
    }
    make_writable(&dir, shown);
    let entries: Vec<(OsString, bool)> = list_entries(&dir, shown)?;
    return Ok(Frame { dir, name: name.to_os_string(), entries });
}

/// What:
///  Remove the directory `name` below `parent` with everything in it; `label` is the path of `parent`, used only in messages.
/// Why:
///   With `expected` set, the directory is removed only when it is that object: the identity is compared by name before the
///  directory is opened and again on the opened object, so a replacement made between the two is left alone. The top
///  directory must be on the device of `parent`. Nothing is followed and nothing outside the directory is touched. An entry
///  that vanishes meanwhile counts as removed. Directories are entered with a work stack, so depth costs no recursion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = await removeTreeBeneath({ label, parent, name, expected }); // 'removed' | 'absent'
/// ```
pub fn remove_tree_beneath(label: &Path, parent: &File, name: &OsStr, expected: Option<ObjectId>) -> Result<Removal, String> {
    let top: PathBuf = label.join(name);
    let device: u64 = device_of(parent).map_err(|error: std::io::Error| { return format!("{} cannot be examined ({error})", label.display()); })?;
    match identity_at(parent, name) {
        Ok(found) => {
            if let Some(wanted) = expected && found != wanted {
                return Err(format!("{} is not the object that was set aside ({} instead of {}), so it is left alone", top.display(), found.encode(), wanted.encode()));
            }
        }
        Err(error) if is_gone(&error) => return Ok(Removal::Absent),
        Err(error) => return Err(format!("{} cannot be examined ({error})", top.display())),
    }
    let first: Frame = enter(parent, name, device, &|| { return top.clone(); })?;
    if let Some(wanted) = expected {
        let opened: ObjectId = identity_of(&first.dir).map_err(|error: std::io::Error| { return format!("{} cannot be examined ({error})", top.display()); })?;
        if opened != wanted {
            return Err(format!("{} is not the object that was set aside ({} instead of {}), so it is left alone", top.display(), opened.encode(), wanted.encode()));
        }
    }
    let mut stack: Vec<Frame> = vec![first];
    // The names from the top directory down to the frame being worked on, for messages.
    let mut shown: Vec<OsString> = Vec::new();
    // The path of the frame being worked on, from the names kept in `shown`.
    let here = |names: &[OsString]| -> PathBuf { return names.iter().fold(top.clone(), |path: PathBuf, part: &OsString| { return path.join(part); }); };
    loop {
        let Some(frame) = stack.last_mut() else { break };
        match frame.entries.pop() {
            Some((entry, true)) => {
                let next: Frame = enter(&frame.dir, entry.as_os_str(), device, &|| { return here(shown.as_slice()).join(&entry); })?;
                shown.push(entry);
                stack.push(next);
            }
            Some((entry, false)) => match remove_at(&frame.dir, entry.as_os_str(), false) {
                Ok(()) => {}
                Err(error) if is_gone(&error) => {}
                Err(error) => return Err(format!("{} cannot be removed ({error})", here(shown.as_slice()).join(&entry).display())),
            },
            None => {
                let Some(done) = stack.pop() else { break };
                let holder: &File = stack.last().map_or(parent, |above: &Frame| { return &above.dir; });
                match remove_at(holder, done.name.as_os_str(), true) {
                    Ok(()) => {}
                    Err(error) if is_gone(&error) => {}
                    Err(error) => return Err(format!("{} cannot be removed ({error})", here(shown.as_slice()).display())),
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
