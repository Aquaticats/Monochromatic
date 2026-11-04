//! What:
//!  Remove a directory tree, making each directory the user owns writable on the way down, and never going into a nested
//!  subvolume or a mount point.
//! Why:
//!  The migration preserves permission bits, so a checkout can hold a mode-555 directory with files in it. Unlinking a
//!  file needs write permission on its directory, so `std::fs::remove_dir_all` stops at the first such directory and
//!  both the deletion of the old tree and the emptying of a subvolume fail partway. Every directory is opened below its
//!  parent's descriptor without following links and given the owner's read, write and search bits through its
//!  descriptor before the tree is removed; a directory the user does not own keeps its mode and the removal reports the
//!  operating system's refusal. A nested subvolume or a mount point inside the tree is not part of it (to the kernel it is
//!  a directory, and `remove_dir_all` would delete what it holds), so the removal refuses before it deletes anything
//!  (`subvolume_remove_walk.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await rm(path, { recursive: true, force: true }); // after chmod u+rwx on every directory, never into a nested subvolume
//! ```

use crate::fs_beneath::{open_directory_widening, remove_at};
use crate::subvolume_remove_walk::{empty_directory, verify_below};
use std::path::Path;

/// What:
///  Remove `path`, whatever it is: a file or link is unlinked, a directory is emptied and removed.
/// Why:
///   Links are never followed; see the module description for the permissions and the boundaries. The empty directory
///  that remains is removed by name from the parent that was opened first, which succeeds only if it is still empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeTree(path);
/// ```
pub fn remove_tree(path: &Path) -> std::io::Result<()> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir() {
        return std::fs::remove_file(path);
    }
    let name: &std::ffi::OsStr = path.file_name().ok_or_else(|| { return std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{} has no last component to remove", path.display())); })?;
    let parent_path: &Path = path.parent().filter(|parent: &&Path| { return !parent.as_os_str().is_empty(); }).unwrap_or_else(|| { return Path::new("."); });
    let parent: std::fs::File = std::fs::File::open(parent_path)?;
    let top: std::fs::File = open_directory_widening(&parent, name).map_err(|error: std::io::Error| {
        if error.raw_os_error() == Some(libc::EXDEV) {
            return std::io::Error::new(
                std::io::ErrorKind::CrossesDevices,
                format!("{} is a mount point, so its content is not part of any tree to remove; unmount it first (`umount {}`)", path.display(), path.display()),
            );
        }
        return error;
    })?;
    verify_below(&top, path)?;
    empty_directory(&top, path)?;
    drop(top);
    return remove_at(&parent, name, true);
}

/// Removal controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_remove_tests.rs"]
mod tests;
