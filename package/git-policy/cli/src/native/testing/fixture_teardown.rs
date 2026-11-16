//! What:
//!  The two steps a fixture tree's teardown takes:
//!  giving this user back the permission a control closed,
//!  and removing the tree,
//!  trying again while a landed commit's child is still finishing.
//! Why:
//!  A tree that a control made read-only inside its fixture outlived the test that made it,
//!  and so did a tree a maintenance child was still gaining entries in,
//!  and neither is the control's to leave in a place every run shares.
//!  They are called on every platform,
//!  and a platform without Unix permissions has nothing to give back.
//!  They moved out of `test_support.rs` when that file passed the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! restoreOwnerAccess(root); await settleFixture(root, keep); // chmod the owner's bits back, then rm -rf with retries
//! ```

/// The path types a tree is named with.
use std::path::{Path, PathBuf};

/// What:
///  Give this user back the read,
/// write and search permission on every directory of a tree that has lost it,
///  without following a symbolic link.
/// Why:
///   A control that writes a directory its owner may not write (`0500`,
/// `0555`,
/// `0000`) proves what the product does with one,
/// and removing that directory afterwards needs read to list it and write and search to unlink what it holds,
/// so a fixture left in that state outlived the test that made it
/// (issue 6282:
/// fixtures named `rollback-read-only`,
/// `remove-old-read-only`,
/// `migrate-unreadable-dir` and `sync-unreadable-dir`),
/// and a build-tree sweeper could not delete the tree they sat in.
/// Only directories are changed,
/// only ones this user owns,
/// and only the owner's three bits:
/// a file with no permissions of its own is unlinked from a writable directory,
/// and a group or other-user bit a control set stays as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreOwnerAccess(root: string): void // lstat each entry, chmod OWNER_RWX on the directories this user owns
/// ```
#[cfg(unix)]
pub(crate) fn restore_owner_access(root: &Path) {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    let own: u32 = unsafe { libc::geteuid() };
    let owner_bits: u32 = 0o700;
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(metadata) = std::fs::symlink_metadata(&directory) else {
            continue;
        };
        if !metadata.is_dir() {
            continue;
        }
        let mode: u32 = metadata.mode() & 0o7777;
        if metadata.uid() == own && mode & owner_bits != owner_bits {
            let _widened = std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(mode | owner_bits));
        }
        // The directory may have been closed to this user until a moment ago, so it is listed after the mode is restored.
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            let Ok(found) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if found.is_dir() {
                stack.push(path);
            }
        }
    }
}

/// What:
///  On a platform without Unix permissions there is nothing to give back.
/// Why:
///   The teardown calls this on every platform.
#[cfg(not(unix))]
pub(crate) fn restore_owner_access(_root: &Path) {}

/// What:
///  Remove a fixture tree unless the person asked to keep it,
/// trying again for up to five seconds.
/// Why:
///   A landed commit may leave a maintenance child finishing for a moment;
/// a tree already gone is the explicit
///  `remove` having run first,
/// not a failure.
/// A control that made a directory read-only inside its fixture is answered here,
/// so the fixture goes however the control ended,
/// panic included.
pub(crate) fn settle_fixture(root: &Path, keep: bool) {
    if keep {
        return;
    }
    restore_owner_access(root);
    for _ in 0..50 {
        match std::fs::remove_dir_all(root) {
            Ok(()) => return,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
        }
    }
}
