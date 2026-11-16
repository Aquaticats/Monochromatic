//! What:
//!  The identity an entry had when a control made it,
//!  and what stands at that path now.
//! Why:
//!  A folder a control makes is named for a process number or a token,
//!  and both come round:
//! two runs of one process number met in one folder and failed each other at `AlreadyExists`,
//!  and a name drawn again lays a second folder over the path the first one had.
//!  So a cleanup entry that removes "the folder at this path" can remove a folder another run made after its own went,
//!  which is neither its to remove nor harmless to the run that owns it.
//!  A device and an inode are the identity the operating system keeps for exactly this question,
//!  and every cleanup here asks it before it removes anything.
//! Unix only,
//! like the controls that use it:
//! a folder a control makes for a socket name is a Unix shape.

/// The path type every caller names an entry with.
use std::path::Path;

/// What:
///  The device and the inode an entry had,
///  which no other entry has at the same time.
/// Why:
///   A path is a name,
/// and a name is what another run can take:
/// the device and the inode are what the kernel hands out once and never reuses while the entry exists.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct FolderIdentity {
    /// The device the entry is on.
    device: u64,
    /// The inode that entry has on its device.
    inode: u64,
}

/// What:
///  What stands at the path an entry had when it was made.
/// Why:
///   The three answers a cleanup can act on:
/// its own entry is still there and goes,
/// the entry is already gone and there is nothing to do,
/// or another entry took the name and is none of its business.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum FolderStanding {
    /// The very entry this identity was taken from.
    StillTheOne,
    /// Nothing stands there any more.
    Gone,
    /// Another entry, which took the name after the one this identity was taken from went.
    AnotherEntryNow,
}

/// What:
///  The identity of the entry at `path`,
///  or nothing where nothing stands there.
/// Why:
///   A symbolic link is read as itself and never as what it points at,
/// so a link a control made is identified by the link and not by the folder it names.
#[cfg(unix)]
pub(crate) fn identity_of(path: &Path) -> Option<FolderIdentity> {
    use std::os::unix::fs::MetadataExt as _;
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).ok()?;
    return Some(FolderIdentity { device: metadata.dev(), inode: metadata.ino() });
}

/// What:
///  A platform without a device and an inode for an entry has no identity to keep.
/// Why:
///   Every caller is a Unix control;
/// the other platforms compile this so the module is not Unix-only code in a portable crate.
#[cfg(not(unix))]
pub(crate) fn identity_of(path: &Path) -> Option<FolderIdentity> {
    std::fs::symlink_metadata(path).ok()?;
    return Some(FolderIdentity { device: 0, inode: 0 });
}

impl FolderIdentity {
    /// What:
    ///  What stands at `path` now,
    ///  read against the identity this entry had when it was made.
    /// Why:
    ///   A cleanup that removes only `StillTheOne` never takes a folder another run made at a name this one used.
    pub(crate) fn standing(&self, path: &Path) -> FolderStanding {
        return match identity_of(path) {
            None => FolderStanding::Gone,
            Some(current) if current == *self => FolderStanding::StillTheOne,
            Some(_another) => FolderStanding::AnotherEntryNow,
        };
    }
}
