//! What:
//!  The request numbers the btrfs ioctls are called by,
//! and the encodings they are computed from:
//! the `_IOW`,
//! `_IOR` and `_IOWR` shapes of the kernel header,
//! the three architecture encodings,
//! and one constant per request this program makes.
//! Why:
//!  `libc` has `FICLONE` and the filesystem magic numbers but no `BTRFS_IOC_*` constants,
//! and the wrapper never starts the `btrfs` program (the repository's rule:
//! embed instead of spawning).
//! The numbers and the argument sizes they state are taken from the kernel header
//! `include/uapi/linux/btrfs.h` (the host copy is `/usr/include/linux/btrfs.h`):
//! `BTRFS_IOCTL_MAGIC` is 0x94;
//! `BTRFS_IOC_SUBVOL_CREATE` is `_IOW(0x94, 14, struct btrfs_ioctl_vol_args)`;
//! `BTRFS_IOC_SNAP_DESTROY` is `_IOW(0x94, 15, struct btrfs_ioctl_vol_args)`;
//! `BTRFS_IOC_SNAP_CREATE_V2` is `_IOW(0x94, 23, struct btrfs_ioctl_vol_args_v2)`.
//! Both argument structures are 4096 bytes:
//! `vol_args` is an `s64` fd and 4088 name bytes;
//! `vol_args_v2` is an `s64` fd,
//! `u64` transid,
//! `u64` flags,
//! a 32-byte union and 4040 name bytes.
//! `strace` of `btrfs subvolume snapshot` and `btrfs subvolume delete` shows the same shapes
//! (`doc/planning/cli-git-btrfs-worktree.md`,
//! "Facts").
//! The encoding is proven against `libc::FICLONE`,
//! and every number by one real call in a test on a btrfs fixture.
//! The numbers are computed from the header's definitions instead of typed in as magic constants,
//! so a control can check the encoding,
//! and each request's number,
//! the sizes it states and the two values it takes in its argument live in one place.
//! The argument structures those sizes describe are `fs_btrfs_ioctl_argument_structures.rs`;
//! the calls that make the requests are `fs_btrfs_ioctl.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const SUBVOL_CREATE = iow(HOST_IOC_LAYOUT, 14, VOLUME_ARGUMENT_BYTES);
//! ```

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
pub const GENERIC_IOC_LAYOUT: IocLayout = IocLayout {
    write: 1,
    read: 2,
    direction_shift: 30,
};

/// The encoding of powerpc,
/// mips,
/// sparc and alpha:
/// a 3-bit direction field with `_IOC_WRITE` 4.
pub const WIDE_DIRECTION_IOC_LAYOUT: IocLayout = IocLayout {
    write: 4,
    read: 2,
    direction_shift: 29,
};

/// The encoding of parisc:
/// `_IOC_WRITE` 2 with the generic field width.
pub const PARISC_IOC_LAYOUT: IocLayout = IocLayout {
    write: 2,
    read: 1,
    direction_shift: 30,
};

/// The encoding of the architecture this code is built for.
#[cfg(any(
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "mips",
    target_arch = "mips32r6",
    target_arch = "mips64",
    target_arch = "mips64r6",
    target_arch = "sparc",
    target_arch = "sparc64"
))]
pub const HOST_IOC_LAYOUT: IocLayout = WIDE_DIRECTION_IOC_LAYOUT;

/// The encoding of the architecture this code is built for.
#[cfg(not(any(
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "mips",
    target_arch = "mips32r6",
    target_arch = "mips64",
    target_arch = "mips64r6",
    target_arch = "sparc",
    target_arch = "sparc64"
)))]
pub const HOST_IOC_LAYOUT: IocLayout = GENERIC_IOC_LAYOUT;

/// The size of `struct btrfs_ioctl_vol_args` and `struct btrfs_ioctl_vol_args_v2`,
/// both 4096 bytes.
pub const VOLUME_ARGUMENT_BYTES: usize = 4096;

/// The size of `struct btrfs_ioctl_get_subvol_info_args`:
/// its fields come to 504 bytes,
/// which the compiler checks where the structure is defined.
pub const SUBVOLUME_INFO_BYTES: usize = 504;

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
pub const fn ioc_write_with(
    layout: IocLayout,
    number: u32,
    size: usize,
) -> u32 {
    return (layout.write << layout.direction_shift)
        | ((size as u32) << IOC_SIZE_SHIFT)
        | (BTRFS_IOCTL_MAGIC << IOC_TYPE_SHIFT)
        | number;
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
pub const fn ioc_read_with(
    layout: IocLayout,
    number: u32,
    size: usize,
) -> u32 {
    return (layout.read << layout.direction_shift)
        | ((size as u32) << IOC_SIZE_SHIFT)
        | (BTRFS_IOCTL_MAGIC << IOC_TYPE_SHIFT)
        | number;
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
pub const fn ioc_read_write_with(
    layout: IocLayout,
    number: u32,
    size: usize,
) -> u32 {
    return ((layout.write | layout.read) << layout.direction_shift)
        | ((size as u32) << IOC_SIZE_SHIFT)
        | (BTRFS_IOCTL_MAGIC << IOC_TYPE_SHIFT)
        | number;
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
pub const fn ioc_write(
    number: u32,
    size: usize,
) -> libc::Ioctl {
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
pub const fn ioc_read(
    number: u32,
    size: usize,
) -> libc::Ioctl {
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
pub const BTRFS_IOC_INO_LOOKUP: libc::Ioctl =
    ioc_read_write_with(HOST_IOC_LAYOUT, 18, VOLUME_ARGUMENT_BYTES) as libc::Ioctl;

/// `BTRFS_IOC_GET_SUBVOL_INFO`:
/// `_IOR(0x94, 60, struct btrfs_ioctl_get_subvol_info_args)`.
/// It reads one subvolume's own record,
/// so it needs no privilege the caller does not already have on the directory it opens.
pub const BTRFS_IOC_GET_SUBVOL_INFO: libc::Ioctl = ioc_read(60, SUBVOLUME_INFO_BYTES);
