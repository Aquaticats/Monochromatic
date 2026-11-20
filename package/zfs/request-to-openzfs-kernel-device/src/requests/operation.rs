//! What:
//!  The typed operation list:
//! one variant per `zfs_ioc_t` request this crate
//!  can express,
//! each with the request number OpenZFS `zfs-2.4.4` assigns in
//!  `include/sys/fs/zfs.h`,
//! and the two questions a caller asks of it (does it
//!  change state,
//! does it drive a listing cursor).
//! Why:
//!  The number is the whole request:
//! a typo sends a destroy as a read,
//! or a
//!  read as a snapshot.
//! Naming each number once,
//! beside the variant that
//!  carries it,
//! makes the table reviewable,
//! and the device-free tests pin every
//!  number against the header the design cites.
//!  There is no general-purpose ioctl here and no generic property setter,
//! so a
//!  caller cannot reach an operation the design did not pick.

/// The formatting traits the diagnostics implement.
use std::fmt;

/// `ZFS_IOC_OBJSET_STATS`:
/// dataset identity,
/// origin and property values.
const ZFS_IOC_OBJSET_STATS: u64 = 0x5a12;
/// `ZFS_IOC_DATASET_LIST_NEXT`:
/// the next immediate child dataset.
const ZFS_IOC_DATASET_LIST_NEXT: u64 = 0x5a14;
/// `ZFS_IOC_SNAPSHOT_LIST_NEXT`:
/// the next snapshot of one dataset.
const ZFS_IOC_SNAPSHOT_LIST_NEXT: u64 = 0x5a15;
/// `ZFS_IOC_SET_PROP`:
/// properties on one named dataset.
const ZFS_IOC_SET_PROP: u64 = 0x5a16;
/// `ZFS_IOC_DESTROY`:
/// one named dataset or snapshot.
const ZFS_IOC_DESTROY: u64 = 0x5a18;
/// `ZFS_IOC_PROMOTE`:
/// one named clone.
const ZFS_IOC_PROMOTE: u64 = 0x5a22;
/// `ZFS_IOC_SNAPSHOT`:
/// named snapshots of one pool.
const ZFS_IOC_SNAPSHOT: u64 = 0x5a23;
/// `ZFS_IOC_GET_FSACL`:
/// the delegations granted on one dataset.
const ZFS_IOC_GET_FSACL: u64 = 0x5a29;
/// `ZFS_IOC_DESTROY_SNAPS`:
/// an explicit snapshot name set.
const ZFS_IOC_DESTROY_SNAPS: u64 = 0x5a3b;
/// `ZFS_IOC_CLONE`:
/// a clone of one origin snapshot.
const ZFS_IOC_CLONE: u64 = 0x5a42;
/// `ZFS_IOC_GET_BOOKMARKS`:
/// every bookmark of one dataset.
const ZFS_IOC_GET_BOOKMARKS: u64 = 0x5a44;
/// `ZFS_IOC_CHANNEL_PROGRAM`:
/// one fixed root-owned program.
const ZFS_IOC_CHANNEL_PROGRAM: u64 = 0x5a48;

/// One request the kernel's `/dev/zfs` interface accepts from this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    /// `ZFS_IOC_OBJSET_STATS`.
    ObjsetStats,
    /// `ZFS_IOC_DATASET_LIST_NEXT`.
    DatasetListNext,
    /// `ZFS_IOC_SNAPSHOT_LIST_NEXT`.
    SnapshotListNext,
    /// `ZFS_IOC_SET_PROP`.
    SetProp,
    /// `ZFS_IOC_DESTROY`.
    Destroy,
    /// `ZFS_IOC_PROMOTE`.
    Promote,
    /// `ZFS_IOC_SNAPSHOT`.
    Snapshot,
    /// `ZFS_IOC_GET_FSACL`.
    GetFsacl,
    /// `ZFS_IOC_DESTROY_SNAPS`.
    DestroySnaps,
    /// `ZFS_IOC_CLONE`.
    Clone,
    /// `ZFS_IOC_GET_BOOKMARKS`.
    GetBookmarks,
    /// `ZFS_IOC_CHANNEL_PROGRAM`.
    ChannelProgram,
}

/// The methods of `Operation`.
impl Operation {
    /// Every operation,
    /// in request-number order.
    pub const ALL: [Operation; 12] = [
        Operation::ObjsetStats,
        Operation::DatasetListNext,
        Operation::SnapshotListNext,
        Operation::SetProp,
        Operation::Destroy,
        Operation::Promote,
        Operation::Snapshot,
        Operation::GetFsacl,
        Operation::DestroySnaps,
        Operation::Clone,
        Operation::GetBookmarks,
        Operation::ChannelProgram,
    ];

    /// The request number the ioctl call carries.
    pub fn code(self) -> u64 {
        return match self {
            Operation::ObjsetStats => ZFS_IOC_OBJSET_STATS,
            Operation::DatasetListNext => ZFS_IOC_DATASET_LIST_NEXT,
            Operation::SnapshotListNext => ZFS_IOC_SNAPSHOT_LIST_NEXT,
            Operation::SetProp => ZFS_IOC_SET_PROP,
            Operation::Destroy => ZFS_IOC_DESTROY,
            Operation::Promote => ZFS_IOC_PROMOTE,
            Operation::Snapshot => ZFS_IOC_SNAPSHOT,
            Operation::GetFsacl => ZFS_IOC_GET_FSACL,
            Operation::DestroySnaps => ZFS_IOC_DESTROY_SNAPS,
            Operation::Clone => ZFS_IOC_CLONE,
            Operation::GetBookmarks => ZFS_IOC_GET_BOOKMARKS,
            Operation::ChannelProgram => ZFS_IOC_CHANNEL_PROGRAM,
        };
    }

    /// Whether the request can change pool or dataset state,
    /// and so is never
    /// replayed when its outcome is unknown.
    pub fn mutates(self) -> bool {
        return match self {
            Operation::ObjsetStats
            | Operation::DatasetListNext
            | Operation::SnapshotListNext
            | Operation::GetFsacl
            | Operation::GetBookmarks => false,
            Operation::SetProp
            | Operation::Destroy
            | Operation::Promote
            | Operation::Snapshot
            | Operation::DestroySnaps
            | Operation::Clone
            | Operation::ChannelProgram => true,
        };
    }

    /// Whether the request advances the kernel's listing cursor in
    /// `zc_cookie`,
    /// and so answers its end with `ESRCH` instead of a failure.
    pub fn lists(self) -> bool {
        return matches!(
            self,
            Operation::DatasetListNext | Operation::SnapshotListNext
        );
    }

    /// The C constant's name without the `ZFS_IOC_` prefix,
    /// as the kernel's
    /// registration table spells it.
    pub fn as_str(self) -> &'static str {
        return match self {
            Operation::ObjsetStats => "OBJSET_STATS",
            Operation::DatasetListNext => "DATASET_LIST_NEXT",
            Operation::SnapshotListNext => "SNAPSHOT_LIST_NEXT",
            Operation::SetProp => "SET_PROP",
            Operation::Destroy => "DESTROY",
            Operation::Promote => "PROMOTE",
            Operation::Snapshot => "SNAPSHOT",
            Operation::GetFsacl => "GET_FSACL",
            Operation::DestroySnaps => "DESTROY_SNAPS",
            Operation::Clone => "CLONE",
            Operation::GetBookmarks => "GET_BOOKMARKS",
            Operation::ChannelProgram => "CHANNEL_PROGRAM",
        };
    }
}

/// How the value prints to a person.
impl fmt::Display for Operation {
    /// Prints the C operation name.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}", self.as_str());
    }
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "operation_tests.rs"]
mod tests;
