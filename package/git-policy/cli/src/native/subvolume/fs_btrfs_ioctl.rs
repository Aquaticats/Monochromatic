//! What:
//!  The btrfs ioctls the wrapper needs,
//! called in process:
//! create an empty subvolume,
//! snapshot a subvolume,
//!  destroy a subvolume,
//! and read what one subvolume's own record says about its identity and the transaction that made it.
//! Why:
//!  `libc` has `FICLONE` and the filesystem magic numbers but no `BTRFS_IOC_*` constants,
//! and the wrapper never starts
//!  the `btrfs` program (the repository's rule:
//! embed instead of spawning).
//! The request numbers and the 4096-byte
//!  argument structures are taken from the kernel header `include/uapi/linux/btrfs.h` (the host copy is
//!  `/usr/include/linux/btrfs.h`):
//! `BTRFS_IOCTL_MAGIC` is 0x94;
//! `BTRFS_IOC_SUBVOL_CREATE` is `_IOW(0x94, 14,
//!  struct btrfs_ioctl_vol_args)`;
//! `BTRFS_IOC_SNAP_DESTROY` is `_IOW(0x94, 15, struct btrfs_ioctl_vol_args)`;
//!  `BTRFS_IOC_SNAP_CREATE_V2` is `_IOW(0x94, 23, struct btrfs_ioctl_vol_args_v2)`.
//! Both structures are 4096 bytes:
//!  `vol_args` is an `s64` fd and 4088 name bytes;
//! `vol_args_v2` is an `s64` fd,
//! `u64` transid,
//! `u64` flags,
//! a
//!  32-byte union and 4040 name bytes.
//! `strace` of `btrfs subvolume snapshot` and `btrfs subvolume delete` shows the
//!  same shapes (`doc/planning/cli-git-btrfs-worktree.md`,
//! "Facts").
//! Each call is proven by one real call in a test on
//!  a btrfs fixture,
//! and the encoding of the request numbers is proven against `libc::FICLONE`.
//!  Permissions (kernel `fs/btrfs/ioctl.c`):
//! creating a subvolume is unrestricted;
//! a snapshot needs ownership of the
//!  source;
//! destroying one needs the `user_subvol_rm_allowed` mount option for an unprivileged user.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await btrfsIoctl(parentFd, 'SNAP_CREATE_V2', { fd: sourceFd, flags: 0, name });
//! ```

use crate::fs_primitive_error::PrimitiveError;
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// `BTRFS_IOCTL_MAGIC`.
const BTRFS_IOCTL_MAGIC: u32 = 0x94;

/// Bits the request number gives to the argument size,
/// after the type and number fields (`_IOC_SIZESHIFT`).
const IOC_SIZE_SHIFT: u32 = 16;

/// Bits to the type field (`_IOC_TYPESHIFT`).
const IOC_TYPE_SHIFT: u32 = 8;

/// What:
///  How an architecture encodes the direction of an ioctl request:
/// the value of `_IOC_WRITE` and where the field sits.
/// Why:
///   Linux has three encodings,
/// not one.
/// `include/uapi/asm-generic/ioctl.h` (x86,
/// arm,
/// arm64,
/// riscv,
/// s390,
/// loongarch)
///  gives the direction 2 bits at shift 30 with `_IOC_WRITE` 1;
/// `arch/powerpc`,
/// `arch/mips`,
/// `arch/sparc` and
///  `arch/alpha` define `_IOC_SIZEBITS` 13,
/// `_IOC_DIRBITS` 3 and `_IOC_WRITE` 4,
/// which puts the field at shift 29;
///  `arch/parisc` keeps 14 size bits and defines `_IOC_WRITE` 2.
/// A request number built with the wrong one reaches the
///  kernel as a different request and fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IocLayout = { write: number; directionShift: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IocLayout {
    /// `_IOC_WRITE`:
    /// the direction value for an argument user space writes.
    pub write: u32,
    /// `_IOC_READ`:
    /// the direction value for an argument the kernel fills in.
    pub read: u32,
    /// `_IOC_DIRSHIFT`:
    /// where the direction field starts.
    pub direction_shift: u32,
}

/// The asm-generic encoding.
pub const GENERIC_IOC_LAYOUT: IocLayout = IocLayout { write: 1, read: 2, direction_shift: 30 };

/// The encoding of powerpc,
/// mips,
/// sparc and alpha:
/// a 3-bit direction field with `_IOC_WRITE` 4.
pub const WIDE_DIRECTION_IOC_LAYOUT: IocLayout = IocLayout { write: 4, read: 2, direction_shift: 29 };

/// The encoding of parisc:
/// `_IOC_WRITE` 2 with the generic field width.
pub const PARISC_IOC_LAYOUT: IocLayout = IocLayout { write: 2, read: 1, direction_shift: 30 };

/// The encoding of the architecture this code is built for.
#[cfg(any(target_arch = "powerpc", target_arch = "powerpc64", target_arch = "mips", target_arch = "mips32r6", target_arch = "mips64", target_arch = "mips64r6", target_arch = "sparc", target_arch = "sparc64"))]
pub const HOST_IOC_LAYOUT: IocLayout = WIDE_DIRECTION_IOC_LAYOUT;

/// The encoding of the architecture this code is built for.
#[cfg(not(any(target_arch = "powerpc", target_arch = "powerpc64", target_arch = "mips", target_arch = "mips32r6", target_arch = "mips64", target_arch = "mips64r6", target_arch = "sparc", target_arch = "sparc64")))]
pub const HOST_IOC_LAYOUT: IocLayout = GENERIC_IOC_LAYOUT;

/// The size of `struct btrfs_ioctl_vol_args` and `struct btrfs_ioctl_vol_args_v2`,
/// both 4096 bytes.
pub const VOLUME_ARGUMENT_BYTES: usize = 4096;

/// The size of `struct btrfs_ioctl_get_subvol_info_args`:
/// its fields come to 504 bytes,
/// which the compiler checks where the structure is defined.
pub const SUBVOLUME_INFO_BYTES: usize = 504;

/// The longest name `BTRFS_IOC_SNAP_CREATE_V2` takes:
/// `BTRFS_SUBVOL_NAME_MAX` (4039) plus its NUL,
/// in the header.
const V2_NAME_CAPACITY: usize = 4040;

/// The longest name the version 1 argument takes:
/// `BTRFS_PATH_NAME_MAX + 1`,
/// in the header.
const V1_NAME_CAPACITY: usize = 4088;

/// The longest file name any Linux directory entry has (`NAME_MAX`);
/// btrfs accepts nothing longer for a subvolume.
const NAME_MAX: usize = 255;

/// What:
///  The request number `_IOW(magic, number, size)` the C macro computes under a given architecture's encoding.
///  `u32` is an unsigned 32-bit integer (siblings `u64` and `i32`);
/// the macro's result fits 32 bits.
/// Why:
///   The numbers are computed from the header's definition instead of typed in as magic constants,
/// so a test can check
///  the encoding against `libc::FICLONE`,
/// which `libc` already defines as `_IOW(0x94, 9, int)` for each architecture.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const iow = (layout: IocLayout, nr: number, size: number) => (layout.write << layout.directionShift) | (size << 16) | (0x94 << 8) | nr;
/// ```
pub const fn ioc_write_with(layout: IocLayout, number: u32, size: usize) -> u32 {
    return (layout.write << layout.direction_shift) | ((size as u32) << IOC_SIZE_SHIFT) | (BTRFS_IOCTL_MAGIC << IOC_TYPE_SHIFT) | number;
}

/// What:
///  The request number `_IOR(magic, number, size)` the C macro computes under a given architecture's encoding,
/// for an
///  argument the kernel fills in and user space only reads back.
/// Why:
///   See `ioc_write_with`;
/// the direction field holds `_IOC_READ` alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ior = (layout: IocLayout, nr: number, size: number) => (layout.read << layout.directionShift) | (size << 16) | (0x94 << 8) | nr;
/// ```
pub const fn ioc_read_with(layout: IocLayout, number: u32, size: usize) -> u32 {
    return (layout.read << layout.direction_shift) | ((size as u32) << IOC_SIZE_SHIFT) | (BTRFS_IOCTL_MAGIC << IOC_TYPE_SHIFT) | number;
}

/// What:
///  The request number `_IOWR(magic, number, size)` the C macro computes under a given architecture's encoding,
/// for an
///  argument that user space fills in and the kernel fills in again.
/// Why:
///   See `ioc_write_with`;
/// the direction field holds both bits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const iowr = (layout: IocLayout, nr: number, size: number) => ((layout.write | layout.read) << layout.directionShift) | (size << 16) | (0x94 << 8) | nr;
/// ```
pub const fn ioc_read_write_with(layout: IocLayout, number: u32, size: usize) -> u32 {
    return ((layout.write | layout.read) << layout.direction_shift) | ((size as u32) << IOC_SIZE_SHIFT) | (BTRFS_IOCTL_MAGIC << IOC_TYPE_SHIFT) | number;
}

/// What:
///  The request number `_IOW(magic, number, size)` for the architecture this code is built for.
///  `libc::Ioctl` is the width the platform's `ioctl` takes.
/// Why:
///   See `ioc_write_with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const iow = (nr: number, size: number) => iowFor(hostLayout, nr, size);
/// ```
pub const fn ioc_write(number: u32, size: usize) -> libc::Ioctl {
    return ioc_write_with(HOST_IOC_LAYOUT, number, size) as libc::Ioctl;
}

/// What:
///  The request number `_IOR(magic, number, size)` for the architecture this code is built for.
/// Why:
///   See `ioc_read_with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ior = (nr: number, size: number) => iorFor(hostLayout, nr, size);
/// ```
pub const fn ioc_read(number: u32, size: usize) -> libc::Ioctl {
    return ioc_read_with(HOST_IOC_LAYOUT, number, size) as libc::Ioctl;
}

/// `BTRFS_IOC_SUBVOL_CREATE`:
/// `_IOW(0x94, 14, struct btrfs_ioctl_vol_args)`.
pub const BTRFS_IOC_SUBVOL_CREATE: libc::Ioctl = ioc_write(14, VOLUME_ARGUMENT_BYTES);

/// `BTRFS_IOC_SNAP_DESTROY`:
/// `_IOW(0x94, 15, struct btrfs_ioctl_vol_args)`.
pub const BTRFS_IOC_SNAP_DESTROY: libc::Ioctl = ioc_write(15, VOLUME_ARGUMENT_BYTES);

/// `BTRFS_IOC_SNAP_CREATE_V2`:
/// `_IOW(0x94, 23, struct btrfs_ioctl_vol_args_v2)`.
pub const BTRFS_IOC_SNAP_CREATE_V2: libc::Ioctl = ioc_write(23, VOLUME_ARGUMENT_BYTES);

/// `BTRFS_IOC_SNAP_DESTROY_V2`:
/// `_IOW(0x94, 63, struct btrfs_ioctl_vol_args_v2)`.
pub const BTRFS_IOC_SNAP_DESTROY_V2: libc::Ioctl = ioc_write(63, VOLUME_ARGUMENT_BYTES);

/// `BTRFS_IOC_INO_LOOKUP`:
/// `_IOWR(0x94, 18, struct btrfs_ioctl_ino_lookup_args)`,
/// 4096 bytes.
pub const BTRFS_IOC_INO_LOOKUP: libc::Ioctl = ioc_read_write_with(HOST_IOC_LAYOUT, 18, VOLUME_ARGUMENT_BYTES) as libc::Ioctl;

/// `BTRFS_IOC_GET_SUBVOL_INFO`:
/// `_IOR(0x94, 60, struct btrfs_ioctl_get_subvol_info_args)`.
/// It reads one subvolume's own record,
/// so it needs no privilege the caller does not already have on the directory it opens.
pub const BTRFS_IOC_GET_SUBVOL_INFO: libc::Ioctl = ioc_read(60, SUBVOLUME_INFO_BYTES);

/// `BTRFS_SUBVOL_SPEC_BY_ID`:
/// the destroy names the subvolume by its id,
/// in the place of the name.
const BTRFS_SUBVOL_SPEC_BY_ID: u64 = 1 << 4;

/// `BTRFS_FIRST_FREE_OBJECTID`:
/// the inode number of the root directory of every subvolume.
const BTRFS_FIRST_FREE_OBJECTID: u64 = 256;

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

/// The request number states 504 bytes, so the structure must hold exactly that.
const _: () = assert!(std::mem::size_of::<SubvolumeInfoArgs>() == SUBVOLUME_INFO_BYTES);

/// What:
///  Check one name for a subvolume entry and copy it,
/// NUL terminated,
/// into a fixed buffer.
///  `const N: usize` is the buffer length (a TS tuple length is not a type parameter,
/// so this is a generic over a
///  number).
/// Why:
///   The kernel takes one path component:
/// a slash,
/// an empty name,
/// `.` or `..` would be a different request than the
///  caller meant,
/// so they fail with `EINVAL` here before any call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fillName(buffer: Uint8Array, name: Uint8Array): void { if (name.length === 0 || name.includes(0x2f) || name.length > 255) throw ...; buffer.set(name); }
/// ```
fn fill_name<const N: usize>(buffer: &mut [u8; N], name: &OsStr, operation: &'static str, parent: &Path) -> Result<(), PrimitiveError> {
    let bytes: &[u8] = name.as_bytes();
    let refused: bool = bytes.is_empty()
        || bytes.len() > NAME_MAX
        || bytes.len() >= N
        || bytes.contains(&b'/')
        || bytes.contains(&0)
        || bytes == b"."
        || bytes == b"..";
    if refused {
        return Err(PrimitiveError { operation, path: parent.join(name), errno: libc::EINVAL });
    }
    buffer[..bytes.len()].copy_from_slice(bytes);
    return Ok(());
}

/// What:
///  Open a directory for an ioctl on it.
/// Why:
///   The ioctls act on the directory descriptor;
/// the kernel checks write and search permission on it itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fd = await open(directory, 'r');
/// ```
fn open_directory(directory: &Path, operation: &'static str) -> Result<File, PrimitiveError> {
    return match File::open(directory) {
        Ok(file) => Ok(file),
        Err(error) => Err(PrimitiveError { operation, path: directory.to_path_buf(), errno: error.raw_os_error().unwrap_or(0) }),
    };
}

/// What:
///  Create an empty subvolume named `name` inside `parent` (`BTRFS_IOC_SUBVOL_CREATE`).
/// Why:
///   Creation is not restricted to privileged users.
/// An empty subvolume is an empty directory to Git,
/// so
///  `git worktree add` accepts it as a destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await btrfsIoctl(await open(parent), 'SUBVOL_CREATE', { name });
/// ```
pub fn subvolume_create(parent: &Path, name: &OsStr) -> Result<(), PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_SUBVOL_CREATE";
    let directory: File = open_directory(parent, operation)?;
    // SAFETY: `VolArgs` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: VolArgs = unsafe { std::mem::zeroed() };
    fill_name(&mut arguments.name, name, operation, parent)?;
    // SAFETY: `directory` is open for the call; `arguments` is a 4096-byte structure as the request number states.
    let result: libc::c_int = unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_SUBVOL_CREATE, &arguments) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error(operation, parent.join(name).as_path()));
    }
    return Ok(());
}

/// What:
///  Snapshot the subvolume at `source` as a new subvolume `name` inside `parent` (`BTRFS_IOC_SNAP_CREATE_V2`,
/// flags 0,
///  so the snapshot is writable).
/// Why:
///   The kernel requires the source to be a subvolume root on the same filesystem and owned by the caller
///  (`EINVAL`,
/// `EXDEV` and `EPERM` otherwise),
/// and shares every data block with the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await btrfsIoctl(await open(parent), 'SNAP_CREATE_V2', { fd: await open(source), flags: 0, name });
/// ```
pub fn snapshot_create(source: &Path, parent: &Path, name: &OsStr) -> Result<(), PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_SNAP_CREATE_V2";
    let directory: File = open_directory(parent, operation)?;
    let origin: File = open_directory(source, operation)?;
    // SAFETY: `VolArgsV2` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: VolArgsV2 = unsafe { std::mem::zeroed() };
    arguments.fd = i64::from(origin.as_raw_fd());
    fill_name(&mut arguments.name, name, operation, parent)?;
    // SAFETY: both descriptors are open for the call; `arguments` is a 4096-byte structure as the request number states.
    let result: libc::c_int = unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_SNAP_CREATE_V2, &arguments) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error(operation, parent.join(name).as_path()));
    }
    return Ok(());
}

/// What:
///  Destroy the subvolume `name` inside `parent` (`BTRFS_IOC_SNAP_DESTROY`).
/// Why:
///   For an unprivileged user the kernel needs the `user_subvol_rm_allowed` mount option (`EPERM` otherwise),
/// write
///  and search permission on the subvolume root,
/// and no nested subvolume inside (`ENOTEMPTY`).
/// Without the option an
///  empty subvolume is removed by `rmdir`,
/// which `btrfs_rmdir` allows for an empty subvolume root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await btrfsIoctl(await open(parent), 'SNAP_DESTROY', { name });
/// ```
pub fn snapshot_destroy(parent: &Path, name: &OsStr) -> Result<(), PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_SNAP_DESTROY";
    let directory: File = open_directory(parent, operation)?;
    // SAFETY: `VolArgs` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: VolArgs = unsafe { std::mem::zeroed() };
    fill_name(&mut arguments.name, name, operation, parent)?;
    // SAFETY: `directory` is open for the call; `arguments` is a 4096-byte structure as the request number states.
    let result: libc::c_int = unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_SNAP_DESTROY, &arguments) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error(operation, parent.join(name).as_path()));
    }
    return Ok(());
}

/// What:
///  The id of the subvolume that holds the open file or directory (`BTRFS_IOC_INO_LOOKUP` with the root directory's inode
///  number,
/// the query the kernel answers for any user).
/// Why:
///   An id names one subvolume for good,
/// while a path names whatever is there when the call runs;
/// the destroy by id
///  therefore removes the subvolume that was looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const id = await subvolumeId(await open(path));
/// ```
pub fn subvolume_id(held: &File, path: &Path) -> Result<u64, PrimitiveError> {
    // `treeid` 0 asks for the subvolume of the file itself, and `objectid` 256 is the root directory of it; the rest is the
    // path buffer the kernel zeroes.
    let mut arguments: [u64; VOLUME_ARGUMENT_BYTES / 8] = [0; VOLUME_ARGUMENT_BYTES / 8];
    arguments[1] = BTRFS_FIRST_FREE_OBJECTID;
    // SAFETY: `held` is open for the call; `arguments` is a 4096-byte, 8-byte-aligned buffer as the request number states.
    let result: libc::c_int = unsafe { libc::ioctl(held.as_raw_fd(), BTRFS_IOC_INO_LOOKUP, arguments.as_mut_ptr()) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error("BTRFS_IOC_INO_LOOKUP", path));
    }
    return Ok(arguments[0]);
}

/// What:
///  Destroy the subvolume with id `subvolume` (`BTRFS_IOC_SNAP_DESTROY_V2` with `BTRFS_SUBVOL_SPEC_BY_ID`),
/// through any
///  open directory of the same filesystem.
/// Why:
///   The kernel looks the subvolume up by id,
/// so a rename or a replacement of its path after the id was read cannot redirect
///  the call to another subvolume.
/// An unprivileged user needs the same mount option as for the destroy by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await btrfsIoctl(anyDirectory, 'SNAP_DESTROY_V2', { subvolid });
/// ```
pub fn snapshot_destroy_by_id(directory: &File, subvolume: u64, path: &Path) -> Result<(), PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_SNAP_DESTROY_V2";
    // SAFETY: `VolArgsV2` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: VolArgsV2 = unsafe { std::mem::zeroed() };
    arguments.flags = BTRFS_SUBVOL_SPEC_BY_ID;
    // The id takes the place of the name in the structure's last union.
    arguments.name[..8].copy_from_slice(&subvolume.to_ne_bytes());
    // SAFETY: `directory` is open for the call; `arguments` is a 4096-byte structure as the request number states.
    let result: libc::c_int = unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_SNAP_DESTROY_V2, &arguments) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error(operation, path));
    }
    return Ok(());
}

/// What:
///  Destroy the subvolume `name` inside the open directory `parent` (`BTRFS_IOC_SNAP_DESTROY`).
/// Why:
///   The call acts on the directory that is held,
/// not on a path that can be resolved again,
/// so a replaced ancestor cannot
///  redirect it.
/// `parent_path` is only the text of the error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await btrfsIoctl(parentHandle, 'SNAP_DESTROY', { name });
/// ```
pub fn snapshot_destroy_in(parent: &File, name: &OsStr, parent_path: &Path) -> Result<(), PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_SNAP_DESTROY";
    // SAFETY: `VolArgs` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: VolArgs = unsafe { std::mem::zeroed() };
    fill_name(&mut arguments.name, name, operation, parent_path)?;
    // SAFETY: `parent` is open for the call; `arguments` is a 4096-byte structure as the request number states.
    let result: libc::c_int = unsafe { libc::ioctl(parent.as_raw_fd(), BTRFS_IOC_SNAP_DESTROY, &arguments) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error(operation, parent_path.join(name).as_path()));
    }
    return Ok(());
}

/// What:
///  Split a path into its parent directory and last component.
/// Why:
///   The ioctls take the parent and a one-component name;
/// callers hold whole paths.
/// A path with no parent or no last
///  component (`/`,
/// `..`) fails with `EINVAL`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [parent, name] = [dirname(path), basename(path)];
/// ```
pub fn split_parent(path: &Path, operation: &'static str) -> Result<(std::path::PathBuf, std::ffi::OsString), PrimitiveError> {
    let Some(name) = path.file_name() else {
        return Err(PrimitiveError { operation, path: path.to_path_buf(), errno: libc::EINVAL });
    };
    let Some(parent) = path.parent() else {
        return Err(PrimitiveError { operation, path: path.to_path_buf(), errno: libc::EINVAL });
    };
    let parent_path: std::path::PathBuf = if parent.as_os_str().is_empty() { std::path::PathBuf::from(".") } else { parent.to_path_buf() };
    return Ok((parent_path, name.to_os_string()));
}

/// What:
///  Create an empty subvolume at a whole path.
/// Why:
///   The convenience over `subvolume_create` for callers that hold the destination path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await subvolumeCreate(dirname(path), basename(path));
/// ```
pub fn subvolume_create_at(path: &Path) -> Result<(), PrimitiveError> {
    let (parent, name) = split_parent(path, "BTRFS_IOC_SUBVOL_CREATE")?;
    return subvolume_create(parent.as_path(), name.as_os_str());
}

/// What:
///  Destroy the subvolume at a whole path.
/// Why:
///   The convenience over `snapshot_destroy` for callers that hold the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await snapshotDestroy(dirname(path), basename(path));
/// ```
pub fn subvolume_destroy_at(path: &Path) -> Result<(), PrimitiveError> {
    let (parent, name) = split_parent(path, "BTRFS_IOC_SNAP_DESTROY")?;
    return snapshot_destroy(parent.as_path(), name.as_os_str());
}

/// What:
///  Snapshot the subvolume at `source` to the whole path `destination`.
/// Why:
///   The convenience over `snapshot_create` for callers that hold the destination path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await snapshotCreate(source, dirname(destination), basename(destination));
/// ```
pub fn snapshot_create_at(source: &Path, destination: &Path) -> Result<(), PrimitiveError> {
    let (parent, name) = split_parent(destination, "BTRFS_IOC_SNAP_CREATE_V2")?;
    return snapshot_create(source, parent.as_path(), name.as_os_str());
}

/// What:
///  The facts the subvolume's own record gives about it:
/// its identity,
/// the subvolume it is a snapshot of,
/// its latest transaction,
/// and the transaction that created it.
/// Why:
///   A command that records which snapshot it is about to make needs these facts before the object exists and after it,
/// and a later command checks them against what stands at the name,
/// so one reader serves both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SubvolumeInfo = { uuid: Uint8Array /* 16 */; parentUuid: Uint8Array /* 16 */; generation: bigint; transidAtCreation: bigint };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubvolumeInfo {
    /// The subvolume's own UUID.
    pub uuid: [u8; 16],
    /// The UUID of the subvolume this one is a snapshot of,
    /// all zero for any other subvolume.
    pub parent_uuid: [u8; 16],
    /// The latest transaction that changed this subvolume.
    pub generation: u64,
    /// The transaction that created this subvolume.
    pub transid_at_creation: u64,
}

/// What:
///  Read what the subvolume holding `path` says about itself (`BTRFS_IOC_GET_SUBVOL_INFO`).
/// Why:
///   The request is answered for the subvolume the opened directory belongs to,
/// so any directory inside it answers,
/// and the kernel needs no privilege the caller does not already have on that directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const info = await subvolumeInfo(path); // { uuid, parentUuid, generation, transidAtCreation }
/// ```
pub fn subvolume_info(path: &Path) -> Result<SubvolumeInfo, PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_GET_SUBVOL_INFO";
    let directory: File = open_directory(path, operation)?;
    // SAFETY: `SubvolumeInfoArgs` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: SubvolumeInfoArgs = unsafe { std::mem::zeroed() };
    // SAFETY: `directory` is open for the call; `arguments` is the 504-byte structure the request number states.
    let result: libc::c_int = unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_GET_SUBVOL_INFO, &mut arguments) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error(operation, path));
    }
    return Ok(SubvolumeInfo {
        uuid: arguments.uuid,
        parent_uuid: arguments.parent_uuid,
        generation: arguments.generation,
        transid_at_creation: arguments.otransid,
    });
}

/// Btrfs ioctl controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_btrfs_ioctl_tests.rs"]
mod tests;
