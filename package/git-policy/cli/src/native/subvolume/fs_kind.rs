//! What:
//!  Which kind of filesystem holds a path (btrfs,
//! overlayfs,
//! other),
//! whether a directory is the root of a btrfs
//!  subvolume,
//! and how many bytes a user can still write there.
//! Why:
//!  Every rung of the cheap-worktree ladder starts from these three facts,
//! which the kernel answers in one
//!  `statfs` call and one inode number,
//! so nothing is spawned:
//! a snapshot needs btrfs and a subvolume root
//!  (inode 256),
//! a reflink works on btrfs and inside an overlay,
//! and everything else keeps Git's own checkout.
//!  `doc/planning/cli-git-btrfs-worktree.md` (section "Degradation ladder") has the measurements behind the ladder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kind = (await statfs(path)).type === 0x9123683e ? 'btrfs' : ...;
//! const isSubvolumeRoot = kind === 'btrfs' && (await stat(path, { bigint: true })).ino === 256n;
//! ```

use crate::fs_primitive_error::{PrimitiveError, c_path};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// The inode number of the root directory of every btrfs subvolume (`BTRFS_FIRST_FREE_OBJECTID`,
///  `include/uapi/linux/btrfs_tree.h`).
pub const BTRFS_SUBVOLUME_ROOT_INODE: u64 = 256;

/// What:
///  The inode number btrfs gives the stand-in directory it shows at an entry that names a subvolume a snapshot did not
///  carry (`BTRFS_EMPTY_SUBVOL_DIR_OBJECTID` in `linux/btrfs_tree.h`).
/// Why:
///   A directory of a btrfs tree has an inode number of 256 or more,
/// so this number names a stand-in and never a folder that was
///  written by anyone;
/// such a stand-in holds nothing and cannot be renamed,
/// so a step that moves directories must leave it alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BTRFS_STAND_IN_INODE = 2n;
/// ```
pub const BTRFS_STAND_IN_INODE: u64 = 2;

/// What:
///  The filesystem kinds the ladder tells apart.
///  `#[derive(...)]` generates copying,
/// debug printing and `==`.
/// Why:
///   Btrfs allows snapshots and reflinks,
/// overlayfs allows reflinks of files inside it,
/// anything else gets Git's checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FilesystemKind = 'btrfs' | 'overlayfs' | 'other';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilesystemKind {
    /// `BTRFS_SUPER_MAGIC`.
    Btrfs,
    /// `OVERLAYFS_SUPER_MAGIC`,
    /// the root of a container.
    Overlayfs,
    /// Every other `f_type`.
    Other,
}

/// What:
///  Map the `f_type` of a `statfs` answer to a kind.
///  `u64` is the widest unsigned integer;
/// siblings `u32` and `i64` are what the C field is on some targets.
/// Why:
///   The C field is a signed word on some targets and an unsigned one on others,
/// so only its low 32 bits are compared
///  with the magic numbers,
/// which are all 32-bit values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function classifyMagic(fType: bigint): FilesystemKind { const low = fType & 0xffffffffn; ... }
/// ```
pub fn classify_magic(f_type: u64) -> FilesystemKind {
    let low: u64 = f_type & 0xffff_ffff;
    if low == (libc::BTRFS_SUPER_MAGIC as u64 & 0xffff_ffff) {
        return FilesystemKind::Btrfs;
    }
    if low == (libc::OVERLAYFS_SUPER_MAGIC as u64 & 0xffff_ffff) {
        return FilesystemKind::Overlayfs;
    }
    return FilesystemKind::Other;
}

/// What:
///  The kind of filesystem that holds `path`,
/// and the bytes a user can write there,
/// from one `statfs` call.
/// Why:
///   The two answers come from the same call,
/// so a caller that wants both pays once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { type, bavail, bsize } = await statfs(path);
/// ```
pub fn filesystem_facts(path: &Path) -> Result<(FilesystemKind, u64), PrimitiveError> {
    let name = c_path("statfs", path)?;
    // SAFETY: `name` is a NUL-terminated string that outlives the call, and `buffer` is a zeroed `statfs` the call fills.
    let mut buffer: libc::statfs = unsafe { std::mem::zeroed() };
    let result: libc::c_int = unsafe { libc::statfs(name.as_ptr(), &mut buffer) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error("statfs", path));
    }
    let free: u64 = (buffer.f_bavail as u64).saturating_mul(buffer.f_bsize as u64);
    return Ok((classify_magic(buffer.f_type as u64), free));
}

/// What:
///  The kind of filesystem that holds `path`.
/// Why:
///   Most callers need only the kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kind = (await filesystemFacts(path)).kind;
/// ```
pub fn filesystem_kind(path: &Path) -> Result<FilesystemKind, PrimitiveError> {
    let facts = filesystem_facts(path)?;
    return Ok(facts.0);
}

/// What:
///  Whether `path` is on an XFS filesystem.
/// Why:
///   XFS shares data blocks between files when it was made with reflink support,
/// which the first clone reveals,
/// so the
///  clone rung of the cheap-worktree ladder is tried there although the ladder's `FilesystemKind` has no XFS.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isXfs = async (path) => (await statfs(path)).type === 0x58465342;
/// ```
pub fn is_xfs(path: &Path) -> Result<bool, PrimitiveError> {
    let name = c_path("statfs", path)?;
    // SAFETY: `name` is a NUL-terminated string that outlives the call, and `buffer` is a zeroed `statfs` the call fills.
    let mut buffer: libc::statfs = unsafe { std::mem::zeroed() };
    let result: libc::c_int = unsafe { libc::statfs(name.as_ptr(), &mut buffer) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error("statfs", path));
    }
    return Ok((buffer.f_type as u64 & 0xffff_ffff) == (libc::XFS_SUPER_MAGIC as u64 & 0xffff_ffff));
}

/// What:
///  Whether `path` is the root directory of a btrfs subvolume.
/// Why:
///   Only a subvolume root can be the source of a snapshot.
/// Inode 256 is the root of every subvolume;
/// on another
///  filesystem the same inode number means nothing,
/// so the filesystem is checked first and a directory on a
///  filesystem other than btrfs is never a subvolume root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSubvolumeRoot = async (path) => (await filesystemKind(path)) === 'btrfs' && (await stat(path, { bigint: true })).ino === 256n;
/// ```
pub fn is_subvolume_root(path: &Path) -> Result<bool, PrimitiveError> {
    if filesystem_kind(path)? != FilesystemKind::Btrfs {
        return Ok(false);
    }
    let metadata = match std::fs::metadata(path) {
        Ok(found) => found,
        Err(error) => return Err(PrimitiveError { operation: "stat", path: path.to_path_buf(), errno: error.raw_os_error().unwrap_or(0) }),
    };
    return Ok(metadata.is_dir() && metadata.ino() == BTRFS_SUBVOLUME_ROOT_INODE);
}

/// What:
///  Whether `path` is the empty stand-in btrfs shows where a snapshot left a nested subvolume out.
/// Why:
///   The inode number and the directory type are the whole test,
/// and a path that cannot be examined is no stand-in;
///  no filesystem call is made,
/// because a stand-in is inode 2 only inside a btrfs tree and a real directory elsewhere has
///  another number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isStandIn = (path) => { const found = lstatSync(path); return found.isDirectory() && found.ino === 2n; };
/// ```
pub fn is_stand_in(path: &Path) -> bool {
    return match std::fs::symlink_metadata(path) {
        Ok(found) => found.is_dir() && found.ino() == BTRFS_STAND_IN_INODE,
        Err(_unreadable) => false,
    };
}

/// Filesystem kind controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_kind_tests.rs"]
mod tests;
