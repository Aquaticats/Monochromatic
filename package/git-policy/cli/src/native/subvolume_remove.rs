//! What:
//!  Remove a directory tree, making each directory the user owns writable on the way down.
//! Why:
//!  The migration preserves permission bits, so a checkout can hold a mode-555 directory with files in it. Unlinking a
//!  file needs write permission on its directory, so `std::fs::remove_dir_all` stops at the first such directory and
//!  both the deletion of the old tree and the emptying of a subvolume fail partway. Every directory is opened without
//!  following links and given the owner's read, write and search bits through its descriptor before the tree is
//!  removed; a directory the user does not own keeps its mode and the removal reports the operating system's refusal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await rm(path, { recursive: true, force: true }); // after chmod u+rwx on every directory in it
//! ```

use super::subvolume_anchor::Anchor;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// What:
///  Give the owner read, write and search permission on every directory below `top` (and on `top`), never following a
///  link.
///  The walk keeps a list of directories still to visit, not recursion, so depth costs no stack.
/// Why:
///   See the module description. A directory that cannot be opened is skipped here; the removal that follows names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await makeDirectoriesWritable(top);
/// ```
fn make_directories_writable(top: &Path) {
    let mut pending: Vec<PathBuf> = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let Ok(anchor) = Anchor::open_below(top, relative.as_path()) else { continue };
        if let Ok(metadata) = anchor.file().metadata() && metadata.mode() & 0o700 != 0o700 {
            let _widened = anchor.file().set_permissions(std::fs::Permissions::from_mode(metadata.mode() | 0o700));
        }
        let Ok(listing) = std::fs::read_dir(anchor.path()) else { continue };
        for item in listing.flatten() {
            if item.file_type().is_ok_and(|kind: std::fs::FileType| { return kind.is_dir(); }) {
                pending.push(relative.join(item.file_name()));
            }
        }
    }
}

/// What:
///  Remove `path`, whatever it is: a file or link is unlinked, a directory is emptied and removed.
/// Why:
///   Links are never followed, as `remove_dir_all` guarantees; see the module description for the permissions.
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
    make_directories_writable(path);
    return std::fs::remove_dir_all(path);
}

/// Removal controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_remove_tests.rs"]
mod tests;
