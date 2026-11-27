//! What:
//!  The kernel structures the btrfs ioctls read and write,
//! laid out as `include/uapi/linux/btrfs.h` declares them:
//! the create and destroy argument,
//! the snapshot argument,
//! one time value of the subvolume record,
//! and the record the subvolume information request fills.
//! Why:
//!  The kernel reads these bytes at the offsets the C declaration gives,
//! so the field order,
//! the padding (`#[repr(C)]`) and the exact size matter,
//! and the size assertion below is what proves the record's layout.
//! Both the create and destroy argument and the snapshot argument are 4096 bytes:
//! `vol_args` is an `s64` fd and 4088 name bytes;
//! `vol_args_v2` is an `s64` fd,
//! `u64` transid,
//! `u64` flags,
//! a 32-byte union and 4040 name bytes.
//! The name capacities these structures reserve come with them,
//! because each is the structure's field length.
//! The request numbers that state each size are `fs_btrfs_ioctl_request_numbers.rs`;
//! the calls that fill these structures are `fs_btrfs_ioctl.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type VolArgs = { fd: bigint; name: Uint8Array /* 4088 bytes */ };
//! ```

use crate::fs_btrfs_ioctl_request_numbers::SUBVOLUME_INFO_BYTES;

/// The longest name `BTRFS_IOC_SNAP_CREATE_V2` takes:
/// `BTRFS_SUBVOL_NAME_MAX` (4039) plus its NUL,
/// in the header.
const V2_NAME_CAPACITY: usize = 4040;

/// The longest name the version 1 argument takes:
/// `BTRFS_PATH_NAME_MAX + 1`,
/// in the header.
const V1_NAME_CAPACITY: usize = 4088;

/// What:
///  `struct btrfs_ioctl_vol_args`:
/// a source descriptor and a name.
///  `#[repr(C)]` keeps the C field order and padding,
/// which the kernel reads.
/// Why:
///   The subvolume create and destroy ioctls take only a name;
/// the kernel ignores `fd` for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type VolArgs = { fd: bigint; name: Uint8Array /* 4088 bytes */ };
/// ```
#[repr(C)]
pub struct VolArgs {
    /// `__s64 fd`.
    pub fd: i64,
    /// `char name[BTRFS_PATH_NAME_MAX + 1]`,
    /// NUL terminated.
    pub name: [u8; V1_NAME_CAPACITY],
}

/// What:
///  `struct btrfs_ioctl_vol_args_v2`:
/// a source descriptor,
/// flags and a name.
/// Why:
///   The snapshot ioctl takes the source subvolume as `fd` and the new name in the parent directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type VolArgsV2 = { fd: bigint; transid: bigint; flags: bigint; union: Uint8Array /* 32 bytes */; name: Uint8Array /* 4040 bytes */ };
/// ```
#[repr(C)]
pub struct VolArgsV2 {
    /// `__s64 fd`:
    /// the source subvolume's descriptor for a snapshot.
    pub fd: i64,
    /// `__u64 transid`:
    /// output,
    /// the transaction that made the snapshot.
    pub transid: u64,
    /// `__u64 flags`:
    /// 0 for a writable snapshot (`BTRFS_SUBVOL_RDONLY` makes a read-only one).
    pub flags: u64,
    /// The 32-byte union (`size` and `qgroup_inherit`,
    /// or `unused[4]`),
    /// zero for no qgroup inheritance.
    pub union_bytes: [u8; 32],
    /// `char name[BTRFS_SUBVOL_NAME_MAX + 1]`,
    /// NUL terminated.
    pub name: [u8; V2_NAME_CAPACITY],
}

/// `struct btrfs_ioctl_timespec`:
/// a whole second and its nanoseconds,
/// padded to 16 bytes as the C structure is.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct IoctlTimespec {
    /// `__u64 sec`.
    pub sec: u64,
    /// `__u32 nsec`.
    pub nsec: u32,
}

/// What:
///  `struct btrfs_ioctl_get_subvol_info_args`:
/// one subvolume's identity,
/// its latest transaction,
/// and the transactions and instants of its creation,
/// change,
/// send and receive.
/// Why:
///   The kernel fills this structure from the subvolume's own root record,
/// so a reader tells a snapshot another party made from the one a command made by what the filesystem wrote,
/// not by what stands at a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SubvolumeInfoArgs = { treeid: bigint; name: Uint8Array /* 256 */; parentId: bigint; dirid: bigint; generation: bigint; flags: bigint; uuid: Uint8Array /* 16 */; parentUuid: Uint8Array /* 16 */; receivedUuid: Uint8Array /* 16 */; ctransid: bigint; otransid: bigint; stransid: bigint; rtransid: bigint; ctime: Timespec; otime: Timespec; stime: Timespec; rtime: Timespec; reserved: bigint[] /* 8 */ };
/// ```
#[repr(C)]
pub struct SubvolumeInfoArgs {
    /// `__u64 treeid`:
    /// the subvolume's id.
    pub treeid: u64,
    /// `char name[BTRFS_VOL_NAME_MAX + 1]`.
    pub name: [u8; 256],
    /// `__u64 parent_id`.
    pub parent_id: u64,
    /// `__u64 dirid`.
    pub dirid: u64,
    /// `__u64 generation`:
    /// the latest transaction that changed this subvolume.
    pub generation: u64,
    /// `__u64 flags`.
    pub flags: u64,
    /// `__u8 uuid[16]`.
    pub uuid: [u8; 16],
    /// `__u8 parent_uuid[16]`:
    /// the subvolume this one is a snapshot of,
    /// all zero when it is no snapshot.
    pub parent_uuid: [u8; 16],
    /// `__u8 received_uuid[16]`.
    pub received_uuid: [u8; 16],
    /// `__u64 ctransid`.
    pub ctransid: u64,
    /// `__u64 otransid`:
    /// the transaction that created this subvolume.
    pub otransid: u64,
    /// `__u64 stransid`.
    pub stransid: u64,
    /// `__u64 rtransid`.
    pub rtransid: u64,
    /// `struct btrfs_ioctl_timespec ctime`.
    pub ctime: IoctlTimespec,
    /// `struct btrfs_ioctl_timespec otime`.
    pub otime: IoctlTimespec,
    /// `struct btrfs_ioctl_timespec stime`.
    pub stime: IoctlTimespec,
    /// `struct btrfs_ioctl_timespec rtime`.
    pub rtime: IoctlTimespec,
    /// `__u64 reserved[8]`,
    /// which the kernel requires to be zero.
    pub reserved: [u64; 8],
}

/// The request number states 504 bytes,
/// so the structure must hold exactly that.
const _: () = assert!(std::mem::size_of::<SubvolumeInfoArgs>() == SUBVOLUME_INFO_BYTES);
