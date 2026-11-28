//! What:
//!  One mount's facts as the kernel reports them for an open descriptor:
//!  the unique number that decides identity,
//!  the reused number the mount table prints,
//!  the namespace number,
//!  the propagation set,
//!  the peer group and the master,
//!  the mount point,
//!  the filesystem type and the source.
//! Why:
//!  The step changes one mount instance and must prove which one before it
//!  touches it,
//!  so the facts are read from a descriptor this process opened rather than
//!  from a path another act can replace
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "What it verifies before",
//!  whose F8 and F37 read the two numbers from `include/uapi/linux/mount.h`
//!  and `fs/namespace.c`).
//!  The descriptor's own mount is named by `statx(2)`'s unique number and then
//!  asked about with `statmount(2)`,
//!  which is the pair this package already calls
//!  (`src/native/subvolume/fs_beneath_mount_facts.rs`;
//!  `src/native/worktree_clone_mount_handler/sweep_standing_mount_source.rs`,
//!  which states the same kernel layout for its own question).
//!  The kernel resolves the number inside this process's own namespace,
//!  so an instance a lazy unmount detached answers `ENOENT` and no facts at
//!  all: a detached mount is never described here,
//!  which is the refusal the design's namespace check is about.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const unique = statx(fd, '', AT_EMPTY_PATH, STATX_MNT_ID_UNIQUE).stx_mnt_id;
//! const facts = statmount({ mntId: unique, mask: MNT_BASIC | POINT | FS_TYPE });
//! ```

use crate::checkout_propagation::mount_propagation_set::MountPropagation;
use crate::subvolume::fs_beneath_mount_facts::unique_mount_number_of;
use std::path::PathBuf;

/// The kernel's `statmount(2)` number,
/// which is 457 on x86-64 and on the asm-generic numbering aarch64 follows;
/// every other architecture this package could build for answers "unavailable"
/// rather than guess a number.
/// Control of this call reads the same number, and `sweep_standing_mount_source`
/// declares it for the sweep's own question.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
const STATMOUNT_NUMBER: Option<libc::c_long> = Some(457);

/// The architectures without a declared `statmount(2)` number,
/// which read no facts here and refuse instead.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const STATMOUNT_NUMBER: Option<libc::c_long> = None;

/// The bytes `mnt_id_req` takes at its first published shape,
/// which is the uapi's `MNT_ID_REQ_SIZE_VER0`:
/// the field mask travels in `param`,
/// so a newer shape is never needed here.
const MOUNT_NUMBER_REQUEST_BYTES: u32 = 24;

/// The bytes `struct statmount` takes,
/// which the uapi holds at 512 by keeping the room later fields need in its
/// spare words;
/// the strings the answer carries begin right after it.
const STATMOUNT_HEADER_BYTES: usize = 512;

/// The room this read gives the answer's strings:
/// a mount point and a filesystem type,
/// each bounded by the kernel's own path length,
/// which is far more than the two take.
const STATMOUNT_STRING_BYTES: usize = 2 * 4_096;

/// Where `struct statmount` holds the mask of the fields the kernel filled.
const MASK_AT: usize = 8;

/// Where `struct statmount` holds the offset of the filesystem type's string.
const FS_TYPE_AT: usize = 36;

/// Where `struct statmount` holds the mount's unique number.
const MOUNT_ID_AT: usize = 40;

/// Where `struct statmount` holds the number the mount table prints.
const MOUNT_ID_OLD_AT: usize = 56;

/// Where `struct statmount` holds the propagation set.
const PROPAGATION_AT: usize = 72;

/// Where `struct statmount` holds the peer group's number.
const PEER_GROUP_AT: usize = 80;

/// Where `struct statmount` holds the master's number.
const MASTER_AT: usize = 88;

/// Where `struct statmount` holds the offset of the mount point's string.
const MOUNT_POINT_AT: usize = 108;

/// Where `struct statmount` holds the offset of the source's string,
/// from Linux 6.13 on.
const SOURCE_AT: usize = 124;

/// Where `struct statmount` holds the namespace's number,
/// which Linux 6.9 added; a kernel older than that leaves the word spare and
/// the answer's mask says so.
const NAMESPACE_ID_AT: usize = 112;

/// The field asked for: the mount's own numbers and its set.
const STATMOUNT_MNT_BASIC: u64 = 0x0000_0002;

/// The field asked for: the mount point's string.
const STATMOUNT_MNT_POINT: u64 = 0x0000_0010;

/// The field asked for: the filesystem type's string.
const STATMOUNT_FS_TYPE: u64 = 0x0000_0020;

/// The field asked for: the namespace's number.
const STATMOUNT_MNT_NS_ID: u64 = 0x0000_0040;

/// The field asked for: the source's string,
/// which Linux 6.13 added and an older kernel leaves out of the answer's mask.
const STATMOUNT_SB_SOURCE: u64 = 0x0000_0200;

/// The mask of the fields this read asks for,
/// which is one constant so the request and the checks read the same list.
const ASKED_FIELDS: u64 = STATMOUNT_MNT_BASIC
    | STATMOUNT_MNT_POINT
    | STATMOUNT_FS_TYPE
    | STATMOUNT_MNT_NS_ID
    | STATMOUNT_SB_SOURCE;

/// What:
///  The `mnt_id_req` the call takes,
///  at the shape every kernel from 6.8 reads.
/// Why:
///  The kernel refuses a request whose `size` is below the first published
///  shape,
///  refuses a spare word that is not zero,
///  and reads the requested fields from `param`
///  (`fs/namespace.c`,
///  `copy_mnt_id_req` and `prepare_kstatmount`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountNumberRequest = { size: 24; spare: 0; mountNumber: number; fields: number };
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct MountNumberRequest {
    /// The bytes of this structure the kernel reads.
    size: u32,
    /// A spare word the kernel requires to be zero.
    spare: u32,
    /// The mount's unique number.
    mount_number: u64,
    /// The `STATMOUNT_*` fields asked for.
    fields: u64,
}

/// What:
///  One mount, as the kernel reports it for a descriptor.
/// Why:
///  The step decides on these fields and the record carries them,
///  so one value holds both numbers,
///  the set,
///  the peer group and master a set detached,
///  and the three names the checks compare.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DescriptorMount = { mountId: number; mountIdOld: number; mountNamespaceId?: number; propagation; peerGroup?: number; master?: number; mountPoint: string; filesystemType: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DescriptorMount {
    /// The number the kernel never reuses,
    /// which decides whether two reads name one instance.
    pub mount_id: u64,
    /// The number `/proc/self/mountinfo` prints,
    /// which the kernel hands out again,
    /// so it only finds that table's row.
    pub mount_id_old: u64,
    /// The mount namespace's number,
    /// where the kernel reports one.
    pub namespace_id: Option<u64>,
    /// The mount's propagation set.
    pub propagation: MountPropagation,
    /// The shared peer group's number,
    /// where the mount is shared.
    pub peer_group: Option<u64>,
    /// The master's number,
    /// where the mount receives another's mounts.
    pub master: Option<u64>,
    /// The mount point,
    /// as the kernel names it from this process's root.
    pub mount_point: PathBuf,
    /// The filesystem type,
    /// such as `zfs`.
    pub filesystem_type: String,
    /// The source,
    /// which is the dataset's own name for a ZFS mount,
    /// where this kernel reports one.
    pub source: Option<String>,
}

/// What:
///  Why one descriptor's mount could not be read.
/// Why:
///  The refused call and its errno are what the record and the summary line
///  carry,
///  and a kernel that cannot answer at all reads differently from a call that
///  refused,
///  so the two are one value rather than one sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountReadFailure = { call: 'statx' | 'statmount'; errno: number } | { call: 'statmount'; unsupported: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MountReadFailure {
    /// The kernel's call refused,
    /// with the error number it answered.
    Refused {
        /// The call's own name.
        call: &'static str,
        /// The error number.
        errno: i32,
    },
    /// This kernel or this build cannot answer,
    /// so no set may be changed on the strength of the read.
    Unsupported {
        /// What this build or this kernel lacks.
        why: String,
    },
}

impl MountReadFailure {
    /// The error number the answer carries,
    /// where one call refused.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const errno = (failure: MountReadFailure): number | undefined => failure.errno;
    /// ```
    pub fn errno(&self) -> Option<i32> {
        return match self {
            MountReadFailure::Refused {
                call: _call,
                errno,
            } => Some(*errno),
            MountReadFailure::Unsupported {
                why: _why,
            } => None,
        };
    }

    /// The sentence a person reads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const message = (failure: MountReadFailure): string => ...;
    /// ```
    pub fn message(&self) -> String {
        return match self {
            MountReadFailure::Refused {
                call,
                errno,
            } => format!("{call} answered error {errno}"),
            MountReadFailure::Unsupported {
                why,
            } => format!("no facts could be read for the descriptor's mount: {why}"),
        };
    }
}

/// What:
///  The mount the given descriptor is on,
///  read by the kernel's own two calls.
/// Why:
///  The descriptor proves which mount the step is about,
///  and the unique number in the answer is the only one that proves the same
///  instance at the read after a set,
///  so both calls take the descriptor and no path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = readTheMount(handle); // statx for the unique number, then statmount for the rest
/// ```
///
/// # Errors
/// The `statx` or `statmount` call refused,
/// or this kernel cannot answer the fields the step needs.
pub fn read_the_mount(descriptor: &std::fs::File) -> Result<DescriptorMount, MountReadFailure> {
    let Some(number) = STATMOUNT_NUMBER else {
        return Err(MountReadFailure::Unsupported {
            why: String::from("this architecture declares no statmount(2) number in this build"),
        });
    };
    let unique: u64 = unique_mount_number_of(descriptor).map_err(|error| {
        return MountReadFailure::Refused {
            call: "statx",
            errno: error.raw_os_error().unwrap_or(libc::EIO),
        };
    })?;
    let request: MountNumberRequest = MountNumberRequest {
        size: MOUNT_NUMBER_REQUEST_BYTES,
        spare: 0,
        mount_number: unique,
        fields: ASKED_FIELDS,
    };
    let mut buffer: Vec<u8> = vec![0_u8; STATMOUNT_HEADER_BYTES + STATMOUNT_STRING_BYTES];
    // SAFETY: the request and the buffer are this call's own and outlive it, the buffer is writable for the length passed,
    // and the kernel writes only into that buffer. The flags argument is zero, which the call documents as its only value.
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
        return Err(MountReadFailure::Refused {
            call: "statmount",
            errno: std::io::Error::last_os_error()
                .raw_os_error()
                .unwrap_or(libc::EIO),
        });
    }
    return facts_of(buffer.as_slice(), unique);
}

/// What:
///  The facts one answer holds,
///  or the refusal the answer's own mask asks for.
/// Why:
///  The kernel sets a bit in the answer's mask for each field it filled,
///  so a field the mask does not name is a field this kernel does not report,
///  and reading it would read a spare word as a number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = (answer, unique) => ({ mountId: mask & MNT_BASIC ? answer.mntId : unsupported(), ... });
/// ```
fn facts_of(
    answer: &[u8],
    unique: u64,
) -> Result<DescriptorMount, MountReadFailure> {
    let mask: u64 = number_at(answer, MASK_AT);
    if mask & STATMOUNT_MNT_BASIC == 0 {
        return Err(MountReadFailure::Unsupported {
            why: String::from("this kernel did not report the mount's own numbers"),
        });
    }
    let bits: u64 = number_at(answer, PROPAGATION_AT);
    let Some(propagation) = MountPropagation::of_the_kernel_set(bits) else {
        return Err(MountReadFailure::Unsupported {
            why: format!(
                "this kernel reported the propagation set {bits}, which this build does not know"
            ),
        });
    };
    if mask & STATMOUNT_MNT_POINT == 0 || mask & STATMOUNT_FS_TYPE == 0 {
        return Err(MountReadFailure::Unsupported {
            why: String::from("this kernel did not report the mount point or the filesystem type"),
        });
    }
    let Some(mount_point) = string_at(answer, field_at(answer, MOUNT_POINT_AT)) else {
        return Err(MountReadFailure::Unsupported {
            why: String::from("the answer's mount point runs past the buffer"),
        });
    };
    let Some(filesystem_type) = string_at(answer, field_at(answer, FS_TYPE_AT)) else {
        return Err(MountReadFailure::Unsupported {
            why: String::from("the answer's filesystem type runs past the buffer"),
        });
    };
    let source: Option<String> = match mask & STATMOUNT_SB_SOURCE == 0 {
        // A kernel older than 6.13 reports no source, and the table's own row answers for it instead.
        true => None,
        false => {
            let Some(text) = string_at(answer, field_at(answer, SOURCE_AT)) else {
                return Err(MountReadFailure::Unsupported {
                    why: String::from("the answer's source runs past the buffer"),
                });
            };
            Some(String::from(text))
        }
    };
    let namespace_id: Option<u64> = match mask & STATMOUNT_MNT_NS_ID == 0 {
        // A kernel older than the field leaves its word spare, so this build says "not reported" rather than guess.
        true => None,
        false => Some(number_at(answer, NAMESPACE_ID_AT)),
    };
    let peer_group: u64 = number_at(answer, PEER_GROUP_AT);
    let master: u64 = number_at(answer, MASTER_AT);
    let answered_id: u64 = number_at(answer, MOUNT_ID_AT);
    if answered_id != unique {
        return Err(MountReadFailure::Unsupported {
            why: format!(
                "the answer names mount {answered_id} and the descriptor's own number is {unique}"
            ),
        });
    }
    return Ok(DescriptorMount {
        mount_id: answered_id,
        mount_id_old: number_at(answer, MOUNT_ID_OLD_AT) & 0xffff_ffff,
        namespace_id: nonzero(namespace_id.unwrap_or(0)),
        propagation,
        peer_group: nonzero(peer_group),
        master: nonzero(master),
        mount_point: PathBuf::from(mount_point),
        filesystem_type: String::from(filesystem_type),
        source,
    });
}

/// A number the kernel reports as zero when the fact does not hold,
/// as no number at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nonzero = (n: number): number | undefined => n === 0 ? undefined : n;
/// ```
fn nonzero(number: u64) -> Option<u64> {
    return match number {
        0 => None,
        other => Some(other),
    };
}

/// One little-endian number of `bytes` at `offset`,
/// or zero when the answer is shorter than that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const numberAt = (bytes: Uint8Array, offset: number): number => ...; // little-endian u64
/// ```
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldAt = (bytes: Uint8Array, offset: number): number => readUint32LE(bytes, offset);
/// ```
fn field_at(
    bytes: &[u8],
    offset: usize,
) -> u32 {
    return u32::try_from(number_at(bytes, offset) & 0xffff_ffff).unwrap_or(0);
}

/// The text at `offset` in the strings part of an answer,
/// or nothing when it runs past the buffer or is not text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stringAt = (bytes: Uint8Array, offset: number): string | undefined => new TextDecoder().decode(bytes.subarray(512).subarray(0, ...NUL));
/// ```
fn string_at(
    bytes: &[u8],
    offset: u32,
) -> Option<&str> {
    let start: usize = STATMOUNT_HEADER_BYTES.checked_add(usize::try_from(offset).ok()?)?;
    let rest: &[u8] = bytes.get(start..)?;
    let end: usize = rest.iter().position(|byte| return *byte == 0)?;
    return std::str::from_utf8(&rest[..end]).ok();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mount_descriptor_read_tests.rs"]
mod tests;
