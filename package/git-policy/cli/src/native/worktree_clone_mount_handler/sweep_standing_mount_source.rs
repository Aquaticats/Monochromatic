//! What:
//!  One mount's own source,
//!  asked of the kernel by the mount's unique number.
//! Why:
//!  A rename of a dataset whose mountpoint is `legacy` changes the source the
//!  kernel prints for its mount and raises no mount event at all,
//!  so a copy of the mount table taken earlier can name another dataset's mount
//!  as this clone's own
//!  (`zfs-rename(8)`,
//!  "If a file system's mountpoint property is set to legacy or none,
//!  the file system is not unmounted even if this option is not given").
//!  `statmount(2)` answers for one mount by the unique number the kernel never
//!  reuses while it runs,
//!  and its `sb_source` string is the same `show_devname` the table prints,
//!  read now rather than when the table was copied
//!  (`fs/namespace.c`,
//!  `statmount_sb_source`,
//!  which calls the superblock's `show_devname` exactly as
//!  `fs/proc_namespace.c` does;
//!  `proc_pid_mountinfo(5)`,
//!  "the filesystem type and the source").
//!  `sb_source` arrived in Linux 6.13 and `statmount` itself in 6.8,
//!  so an older kernel answers nothing here and the sweep reads the live table
//!  instead,
//!  which is the call this route made before it kept a copy.
//!  The unique number comes from `statx(2)`'s `STATX_MNT_ID_UNIQUE`,
//!  which `statx(2)` documents as "guaranteed to not be reused while the system
//!  is running" (Linux 6.8 and later).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = sourceByMountNumber().sourceOf(uniqueMountNumber);
//! ```

/// The kernel's `statmount(2)` number,
/// which is 457 both on x86-64 and on the asm-generic numbering aarch64
/// follows;
/// every other architecture this package could be built for answers
/// "unavailable" rather than guess a number.
///
/// Why a control reads it:
///  the seccomp filter that proves the fallback has to name the call it
///  refuses,
///  and that number belongs here beside the call this build makes.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub(crate) const STATMOUNT_NUMBER: Option<libc::c_long> = Some(457);

/// The architectures this package does not declare a `statmount(2)` number
/// for,
/// which read the live table instead.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub(crate) const STATMOUNT_NUMBER: Option<libc::c_long> = None;

/// The bytes of `mnt_id_req` that every kernel from 6.8 reads,
/// which is the uapi's `MNT_ID_REQ_SIZE_VER0`:
/// the field mask lives in `fields`,
/// so a larger request shape is never needed here.
const MOUNT_NUMBER_REQUEST_BYTES: u32 = 24;

/// The bytes `struct statmount` takes,
/// which the uapi keeps at 512 by holding the room later fields need in its
/// spare words
/// (6.8 ends the structure with `__spare2[50]` and the newest uapi with
/// `__spare2[43]` plus the fields it took back);
/// the strings the answer carries begin right after it.
const STATMOUNT_HEADER_BYTES: usize = 512;

/// The most bytes one source string may take,
/// which is the kernel's dataset-name bound with room for the pool and the
/// container above it.
const MAXIMUM_SOURCE_BYTES: usize = 1_024;

/// Where `struct statmount` holds the mask of the fields the kernel filled,
/// the same offset from 6.8 on.
const MASK_AT: usize = 8;

/// Where `struct statmount` holds the offset of the mount's root string,
/// which is `/` for a dataset's own mount and the proof that the header size
/// above names the strings this build reads.
const MOUNT_ROOT_AT: usize = 104;

/// Where `struct statmount` holds the offset of the source string,
/// from Linux 6.13 on.
const SOURCE_AT: usize = 124;

/// The field the call is asked for:
/// the mount's root,
/// which no source check needs but every answer proves the string layout with.
const STATMOUNT_MOUNT_ROOT: u64 = 0x0000_0008;

/// The field the call is asked for:
/// the source string.
const STATMOUNT_SOURCE: u64 = 0x0000_0200;

/// The root a dataset's own mount shows,
/// which the answer's root string must read.
const THE_ROOT_OF_A_DATASETS_OWN_MOUNT: &str = "/";

/// What:
///  The `mnt_id_req` the call takes,
///  at the shape every kernel from 6.8 reads.
/// Why:
///  The first field is the size the kernel may read,
///  and the fourth carries the mask of fields asked for rather than a path or
///  a descriptor,
///  which is what `statmount(2)` documents for its request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountNumberRequest = { size: 24; flagsOrDescriptor: 0; mountNumber; fields };
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct MountNumberRequest {
    /// The bytes of this structure the kernel reads.
    size: u32,
    /// A mount namespace's descriptor or a mount's descriptor,
    /// which this call never passes.
    flags_or_descriptor: u32,
    /// The mount's unique number,
    /// from `statx(2)`'s unique mount identity.
    mount_number: u64,
    /// The `STATMOUNT_*` fields asked for.
    fields: u64,
}

/// What:
///  What the kernel answered about one mount's source.
/// Why:
///  A kernel without the call,
///  without the field or without a unique number answers nothing,
///  and a caller must tell that from a source it read,
///  because only the second may decide that a mount is this clone's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountSourceAnswer = { kind: 'source'; source: string } | { kind: 'no-answer'; why: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MountSourceAnswer {
    /// The source the mount reports now.
    Source(String),
    /// This kernel or this build cannot say,
    /// and the caller reads the live table instead.
    NoAnswer(String),
}

/// What:
///  One mount's source by its unique number,
///  as a seam a control drives.
/// Why:
///  The release call is a raw system call a control cannot make answer,
///  so a control supplies an answer and the release reads the kernel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface MountSourceByNumber { sourceOf(uniqueMountNumber): MountSourceAnswer }
/// ```
pub(crate) trait MountSourceByNumber {
    /// The source the mount with this unique number reports now.
    fn source_of(
        &self,
        unique_mount_number: Option<u64>,
    ) -> MountSourceAnswer;
}

/// What:
///  The release answer:
///  `statmount(2)` by the mount's unique number,
///  asking for its source and its root.
/// Why:
///  The root string is asked for as well,
///  although no source check reads it,
///  because it proves the size this build assumes the header takes:
///  a kernel that laid the header out differently would put another string
///  where the source is expected,
///  and a root that is not `/` says so,
///  so the answer is "no answer" and the sweep reads the live table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = new KernelMountSource().sourceOf(unique);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct KernelMountSource;

impl MountSourceByNumber for KernelMountSource {
    fn source_of(
        &self,
        unique_mount_number: Option<u64>,
    ) -> MountSourceAnswer {
        let Some(number) = STATMOUNT_NUMBER else {
            return MountSourceAnswer::NoAnswer(String::from(
                "this architecture declares no statmount(2) number in this build",
            ));
        };
        let Some(mount_number) = unique_mount_number else {
            return MountSourceAnswer::NoAnswer(String::from(
                "this kernel gave no unique mount number for the slot",
            ));
        };
        let request: MountNumberRequest = MountNumberRequest {
            size: MOUNT_NUMBER_REQUEST_BYTES,
            flags_or_descriptor: 0,
            mount_number,
            fields: STATMOUNT_MOUNT_ROOT | STATMOUNT_SOURCE,
        };
        let mut buffer: Vec<u8> = vec![0_u8; STATMOUNT_HEADER_BYTES + MAXIMUM_SOURCE_BYTES];
        // SAFETY: the request and the buffer are this call's own, both outlive it, the buffer is writable for the length
        // passed, and `statmount` writes only into that buffer. The flags argument is zero, which the call documents as
        // the only value it takes today.
        let answered: libc::c_long = unsafe {
            libc::syscall(
                number,
                &request as *const MountNumberRequest,
                buffer.as_mut_ptr(),
                buffer.len(),
                0_u32,
            )
        };
        if answered < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            return MountSourceAnswer::NoAnswer(format!(
                "statmount(2) for mount {mount_number} answered {error}"
            ));
        }
        let mask: u64 = number_at(&buffer, MASK_AT);
        if mask & STATMOUNT_SOURCE == 0 {
            return MountSourceAnswer::NoAnswer(String::from(
                "this kernel does not report a mount's source, which statmount(2) gained in Linux 6.13",
            ));
        }
        let root: &str = match string_at(&buffer, field_at(&buffer, MOUNT_ROOT_AT) as usize) {
            Some(root) => root,
            None => {
                return MountSourceAnswer::NoAnswer(String::from(
                    "the answer's root string runs past the buffer",
                ));
            }
        };
        if root != THE_ROOT_OF_A_DATASETS_OWN_MOUNT {
            return MountSourceAnswer::NoAnswer(format!(
                "the answer's root is `{root}`, so this build reads the header of another layout"
            ));
        }
        return match string_at(&buffer, field_at(&buffer, SOURCE_AT) as usize) {
            Some(source) => MountSourceAnswer::Source(String::from(source)),
            None => MountSourceAnswer::NoAnswer(String::from(
                "the answer's source string runs past the buffer",
            )),
        };
    }
}

/// One little-endian number of `bytes` at `offset`,
/// or zero when the answer is shorter than that.
fn number_at(
    bytes: &[u8],
    offset: usize,
) -> u64 {
    let mut value: u64 = 0;
    for index in 0..bytes.len().saturating_sub(offset).min(8) {
        value |= u64::from(bytes[offset + index]) << (8 * index);
    }
    return value;
}

/// One little-endian 32-bit field of `bytes` at `offset`,
/// or zero when the answer is shorter than that.
fn field_at(
    bytes: &[u8],
    offset: usize,
) -> u32 {
    return u32::try_from(number_at(bytes, offset) & 0xffff_ffff).unwrap_or(0);
}

/// The text at `offset` in the strings part of an answer,
/// or nothing when it runs past the buffer or is not text.
fn string_at(
    bytes: &[u8],
    offset: usize,
) -> Option<&str> {
    let start: usize = STATMOUNT_HEADER_BYTES.checked_add(offset)?;
    let rest: &[u8] = bytes.get(start..)?;
    let end: usize = rest.iter().position(|byte| return *byte == 0)?;
    return std::str::from_utf8(&rest[..end]).ok();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_standing_mount_source_tests.rs"]
mod tests;
