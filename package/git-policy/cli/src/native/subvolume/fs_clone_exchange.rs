//! What:
//!  Two more Linux primitives:
//! clone a whole file's data blocks into another file (`FICLONE`),
//! and swap two paths
//!  atomically (`renameat2` with `RENAME_EXCHANGE`).
//! Why:
//!  A reflink copy shares the data blocks of a file instead of copying them (btrfs,
//! XFS with reflink,
//! and files inside
//!  one overlayfs whose layers share such a filesystem),
//! and an exchange swaps a directory with its replacement in one
//!  step,
//! so no process ever sees the path missing.
//! Both fail with a typed error the caller falls through on:
//!  `FICLONE` answers `EXDEV` across filesystems,
//! `EOPNOTSUPP` or `EINVAL` where the filesystem has no reflinks;
//! the
//!  exchange answers `EINVAL` or `EXDEV` where the filesystem cannot swap.
//! The ladder of
//!  `doc/planning/cli-git-btrfs-worktree.md` uses the clone as its second rung.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await copyFile(source, destination, constants.COPYFILE_FICLONE_FORCE);
//! await renameExchange(first, second); // no Node API; the kernel call is renameat2(RENAME_EXCHANGE)
//! ```

use crate::fs_primitive_error::{PrimitiveError, c_path};
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// What:
///  Clone the whole content of `source` into `destination` (`FICLONE`).
///  `&File` borrows an open file (sibling `&Path` names one that is not open).
/// Why:
///   The destination must be open for writing and empty or about to be replaced entirely;
/// the kernel makes it share
///  the source's data blocks.
/// `path` is only used to word the error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyFile(sourcePath, destinationPath, constants.COPYFILE_FICLONE_FORCE);
/// ```
pub fn clone_file(source: &File, destination: &File, path: &Path) -> Result<(), PrimitiveError> {
    // SAFETY: both descriptors are open for the call, and `FICLONE` takes the source descriptor as its argument.
    let result: libc::c_int = unsafe { libc::ioctl(destination.as_raw_fd(), libc::FICLONE, source.as_raw_fd()) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error("FICLONE", path));
    }
    return Ok(());
}

/// What:
///  Swap two existing paths atomically (`renameat2(AT_FDCWD, first, AT_FDCWD, second, RENAME_EXCHANGE)`).
/// Why:
///   After the call `first` names what `second` named and the other way round,
/// with no moment at which either path is
///  missing.
/// This is what `mv --exchange` runs,
/// so the migration needs no `mv`.
/// Both paths must exist and be on one
///  filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await renameExchange(first, second); // atomic swap
/// ```
pub fn exchange_paths(first: &Path, second: &Path) -> Result<(), PrimitiveError> {
    let first_name = c_path("renameat2", first)?;
    let second_name = c_path("renameat2", second)?;
    // SAFETY: both names are NUL-terminated strings that outlive the call; `AT_FDCWD` makes relative names relative to
    // the working directory, as `rename` does.
    let result: libc::c_int = unsafe {
        libc::renameat2(libc::AT_FDCWD, first_name.as_ptr(), libc::AT_FDCWD, second_name.as_ptr(), libc::RENAME_EXCHANGE)
    };
    if result != 0 {
        return Err(PrimitiveError::last_os_error("renameat2(RENAME_EXCHANGE)", first));
    }
    return Ok(());
}

/// Clone and exchange controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_clone_exchange_tests.rs"]
mod tests;
