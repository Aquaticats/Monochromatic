//! What:
//!  The mount of one folder as the route records it:
//!  the kernel's mount ID and its parent's,
//!  the mount point and source the kernel lists,
//!  the folder's device and inode,
//!  and the path its descriptor names now;
//!  and the flags an attach enforces.
//! Why:
//!  A mount is told from another mount of the same dataset by its mount ID,
//!  and from a bind of the same folder by its parent mount and its source
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  `nosuid` and `nodev` are enforced on the attach itself rather than trusted from a dataset property,
//!  because the measured setuid control gained effective UID 0 without them and kept the account's UID with them
//!  (item 7 of "VM measurements preceding acceptance").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mount = mountIdentityOf(folder); // { mountId, parentMountId, mountPoint, source, device, inode, observedPath }
//! fsmount(source, ATTACH_MOUNT_FLAGS);
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{mount_id_of, observed_path};
use crate::fs_mount_options::{MOUNTINFO_PATH, MountFacts, parse_mount_table};
use std::path::PathBuf;

/// Module tag for diagnostics.
const TAG: &str = "attachedMountIdentity";

/// What:
///  The flags an attach passes when it makes the clone's mount:
///  no set-user-ID execution and no device nodes.
/// Why:
///   The measured control showed a root-owned setuid executable gaining effective UID 0 on a clone mount
///  that was not restricted,
///  so both flags are enforced by the mount itself and then read back from `mountinfo`
///  (item 7 of "VM measurements preceding acceptance").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fsmount(fs, FSMOUNT_CLOEXEC, MS_NOSUID | MS_NODEV);
/// ```
pub const ATTACH_MOUNT_FLAGS: libc::c_ulong = libc::MS_NOSUID | libc::MS_NODEV;

/// What:
///  One mount, described by every fact an attach or a rollback records.
/// Why:
///   A report and a later unmount both compare several facts,
///  so one value carries them together:
///  the mount IDs,
///  the mount point and source the kernel lists,
///  the folder's device and inode,
///  and the path its descriptor names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountIdentity = { mountId: bigint; parentMountId: bigint; mountPoint: string; source: string; fileSystemType: string; device: bigint; inode: bigint; observedPath: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountIdentity {
    /// The kernel's ID of the mount the folder is on.
    pub mount_id: u64,
    /// The ID of the mount this one is mounted in.
    pub parent_mount_id: u64,
    /// The mount point the kernel lists for that mount.
    pub mount_point: PathBuf,
    /// The mount's source.
    pub source: String,
    /// The mount's filesystem type.
    pub file_system_type: String,
    /// The folder's device.
    pub device: u64,
    /// The folder's inode.
    pub inode: u64,
    /// The path the folder's descriptor names now.
    pub observed_path: PathBuf,
}

impl MountIdentity {
    /// The mount in words,
    /// naming every fact a report holds.
    pub fn describe(&self) -> String {
        return format!(
            "mount {} (parent mount {}), mount point {}, source {}, filesystem {}, device {}, inode {}, observed at {}",
            self.mount_id,
            self.parent_mount_id,
            self.mount_point.display(),
            self.source,
            self.file_system_type,
            self.device,
            self.inode,
            self.observed_path.display(),
        );
    }
}

/// What:
///  Why a mount's facts could not be read.
/// Why:
///   An attach that cannot read the kernel's own table must refuse rather than guess a mount identity,
///  and the sentence names the table and the way forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountIdentityRefusal = { kind: 'table' | 'no-entry' | 'fdinfo'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MountIdentityRefusal {
    /// The mount table or the descriptor's own file could not be read.
    Unreadable {
        /// The file that was read.
        path: PathBuf,
        /// What the read answered.
        reason: String,
    },
    /// The mount the descriptor is on has no line in the mount table.
    NoMountEntry {
        /// The mount the descriptor reported.
        mount_id: u64,
    },
}

impl MountIdentityRefusal {
    /// The refusal in words,
    /// naming the table and every way forward.
    pub fn message(&self) -> String {
        return match self {
            MountIdentityRefusal::Unreadable { path, reason } => format!(
                "the kernel's mount facts at {} could not be read: {reason}. Nothing is attached or rolled back on a mount identity that cannot be read. Ways forward: run the command again; when the read keeps failing, the process cannot read `/proc`, and an administrator checks that mount namespace before the clone is attached.",
                path.display()
            ),
            MountIdentityRefusal::NoMountEntry { mount_id } => format!(
                "the descriptor is on mount {mount_id}, which {} does not list. A mount this process cannot find is not one it acts on. Ways forward: run the command in the mount namespace that holds the clone's mount, and check with `findmnt --mountinfo` that the mount is listed there.",
                MOUNTINFO_PATH
            ),
        };
    }
}

/// What:
///  The facts the mount table lists for one mount ID.
/// Why:
///   The post-attach check and a rollback report need the mount's point,
///  source and type,
///  and a mount the table does not list is a refusal rather than an empty answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = mountFactsOf(42); // the table's line for mount 42
/// ```
///
/// # Errors
/// A mount table that cannot be read,
/// or one that lists no line for `mount_id`.
pub fn mount_facts_of(mount_id: u64) -> Result<MountFacts, MountIdentityRefusal> {
    let bytes: Vec<u8> = std::fs::read(MOUNTINFO_PATH).map_err(|error| return MountIdentityRefusal::Unreadable { path: PathBuf::from(MOUNTINFO_PATH), reason: error.to_string() })?;
    for facts in parse_mount_table(bytes.as_slice()) {
        if facts.mount_id == mount_id {
            return Ok(facts);
        }
    }
    return Err(MountIdentityRefusal::NoMountEntry { mount_id });
}

/// What:
///  Read the mount a folder's descriptor is on.
/// Why:
///   The attach records what it attached and a rollback names what it must take down,
///  and both rest on the kernel's own numbers rather than on a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = mountIdentityOf(heldTarget);
/// ```
///
/// # Errors
/// The descriptor's mount ID or observed path could not be read,
/// its mount table cannot be read,
/// or the table lists no line for its mount.
pub fn mount_identity_of(folder: &std::fs::File) -> Result<MountIdentity, MountIdentityRefusal> {
    let observed: PathBuf = observed_path(folder).map_err(|error| return MountIdentityRefusal::Unreadable { path: PathBuf::from("/proc/self/fd"), reason: error.to_string() })?;
    let mount_id: u64 = mount_id_of(folder).map_err(|error| return MountIdentityRefusal::Unreadable { path: PathBuf::from("/proc/self/fdinfo"), reason: error.to_string() })?;
    let facts: MountFacts = mount_facts_of(mount_id)?;
    let metadata: std::fs::Metadata = folder.metadata().map_err(|error| return MountIdentityRefusal::Unreadable { path: observed.clone(), reason: error.to_string() })?;
    debug(TAG, "read the mount identity of one folder");
    return Ok(MountIdentity {
        mount_id,
        parent_mount_id: facts.parent_id,
        mount_point: facts.mount_point,
        source: facts.source,
        file_system_type: facts.filesystem_type,
        device: metadata.dev(),
        inode: metadata.ino(),
        observed_path: observed,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "attached_mount_identity_tests.rs"]
mod tests;
