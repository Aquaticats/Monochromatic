//! What:
//!  A file's identity at the moment it was judged:
//! the removal checks that it is still that file.
//! Why:
//!  A freed inode number may be handed to the next file created at once,
//! so device and inode alone cannot tell the judged
//!  lock from a new lock a live Git made at the same path;
//! the size,
//! birth time and modification time are compared too.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Identity = { device: bigint; inode: bigint; size: number; bornNs?: bigint; modifiedNs: bigint };
//! ```
//!
//! No mount number is kept:
//! a descriptor is matched to the lock by device and inode,
//! which is the pair that names one file under a bind mount and across the subvolumes of a btrfs file system
//! (audit of 2026-11-09,
//! finding 4).

use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::SystemTime;

/// A lock's identity at the moment it was judged:
/// the removal checks that it is still this file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Identity {
    /// Device number.
    pub device: u64,
    /// Inode number;
    /// a freed inode may be reused at once,
    /// so it is never compared alone.
    pub inode: u64,
    /// Size in bytes.
    pub size: u64,
    /// Birth time in nanoseconds since the epoch,
    /// where the file system keeps it.
    pub born_ns: Option<u128>,
    /// Modification time in nanoseconds since the epoch.
    pub modified_ns: i128,
}

/// The identity of the file at `path`,
/// without following a link.
pub fn identity_of(path: &Path) -> Option<Identity> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).ok()?;
    let born_ns: Option<u128> = match metadata.created() {
        Ok(born) => match born.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(elapsed) => Some(elapsed.as_nanos()),
            Err(_) => None,
        },
        Err(_) => None,
    };
    let modified_ns: i128 = i128::from(metadata.mtime()) * 1_000_000_000 + i128::from(metadata.mtime_nsec());
    return Some(Identity { device: metadata.dev(), inode: metadata.ino(), size: metadata.len(), born_ns, modified_ns });
}
