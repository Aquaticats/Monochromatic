//! What:
//!  The btrfs ioctls the wrapper needs,
//! called in process:
//! create an empty subvolume,
//! snapshot a subvolume,
//!  destroy a subvolume,
//! and read what one subvolume's own record says about its identity and the transaction that made it.
//! Why:
//!  The wrapper never starts the `btrfs` program (the repository's rule:
//! embed instead of spawning),
//! so each of these jobs is one ioctl on an open directory.
//! Each call is proven by one real call in a test on
//!  a btrfs fixture.
//!  Permissions (kernel `fs/btrfs/ioctl.c`):
//! creating a subvolume is unrestricted;
//! a snapshot needs ownership of the
//!  source;
//! destroying one needs the `user_subvol_rm_allowed` mount option for an unprivileged user.
//!
//! The request numbers are `fs_btrfs_ioctl_request_numbers.rs`,
//! the argument structures they read and write are `fs_btrfs_ioctl_argument_structures.rs`,
//! and both are taken from the kernel header `include/uapi/linux/btrfs.h`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await btrfsIoctl(parentFd, 'SNAP_CREATE_V2', { fd: sourceFd, flags: 0, name });
//! ```

pub use super::fs_btrfs_ioctl_argument_structures::{
    IoctlTimespec,
    SubvolumeInfoArgs,
    VolArgs,
    VolArgsV2,
};
pub use super::fs_btrfs_ioctl_request_numbers::{
    BTRFS_IOC_GET_SUBVOL_INFO,
    BTRFS_IOC_INO_LOOKUP,
    BTRFS_IOC_SNAP_CREATE_V2,
    BTRFS_IOC_SNAP_DESTROY,
    BTRFS_IOC_SNAP_DESTROY_V2,
    BTRFS_IOC_SUBVOL_CREATE,
    GENERIC_IOC_LAYOUT,
    HOST_IOC_LAYOUT,
    IocLayout,
    PARISC_IOC_LAYOUT,
    SUBVOLUME_INFO_BYTES,
    VOLUME_ARGUMENT_BYTES,
    WIDE_DIRECTION_IOC_LAYOUT,
    ioc_read,
    ioc_read_with,
    ioc_read_write_with,
    ioc_write,
    ioc_write_with,
};
use crate::fs_primitive_error::PrimitiveError;
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// `BTRFS_SUBVOL_SPEC_BY_ID`:
/// the destroy names the subvolume by its id,
/// in the place of the name.
const BTRFS_SUBVOL_SPEC_BY_ID: u64 = 1 << 4;

/// `BTRFS_FIRST_FREE_OBJECTID`:
/// the inode number of the root directory of every subvolume.
const BTRFS_FIRST_FREE_OBJECTID: u64 = 256;

/// The longest file name any Linux directory entry has (`NAME_MAX`);
/// btrfs accepts nothing longer for a subvolume.
const NAME_MAX: usize = 255;

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
fn fill_name<const N: usize>(
    buffer: &mut [u8; N],
    name: &OsStr,
    operation: &'static str,
    parent: &Path,
) -> Result<(), PrimitiveError> {
    let bytes: &[u8] = name.as_bytes();
    let refused: bool = bytes.is_empty()
        || bytes.len() > NAME_MAX
        || bytes.len() >= N
        || bytes.contains(&b'/')
        || bytes.contains(&0)
        || bytes == b"."
        || bytes == b"..";
    if refused {
        return Err(PrimitiveError {
            operation,
            path: parent.join(name),
            errno: libc::EINVAL,
        });
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
fn open_directory(
    directory: &Path,
    operation: &'static str,
) -> Result<File, PrimitiveError> {
    return match File::open(directory) {
        Ok(file) => Ok(file),
        Err(error) => Err(PrimitiveError {
            operation,
            path: directory.to_path_buf(),
            errno: error.raw_os_error().unwrap_or(0),
        }),
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
pub fn subvolume_create(
    parent: &Path,
    name: &OsStr,
) -> Result<(), PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_SUBVOL_CREATE";
    let directory: File = open_directory(parent, operation)?;
    // SAFETY: `VolArgs` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: VolArgs = unsafe { std::mem::zeroed() };
    fill_name(&mut arguments.name, name, operation, parent)?;
    // SAFETY: `directory` is open for the call; `arguments` is a 4096-byte structure as the request number states.
    let result: libc::c_int =
        unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_SUBVOL_CREATE, &arguments) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error(
            operation,
            parent.join(name).as_path(),
        ));
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
pub fn snapshot_create(
    source: &Path,
    parent: &Path,
    name: &OsStr,
) -> Result<(), PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_SNAP_CREATE_V2";
    let directory: File = open_directory(parent, operation)?;
    let origin: File = open_directory(source, operation)?;
    // SAFETY: `VolArgsV2` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: VolArgsV2 = unsafe { std::mem::zeroed() };
    arguments.fd = i64::from(origin.as_raw_fd());
    fill_name(&mut arguments.name, name, operation, parent)?;
    // SAFETY: both descriptors are open for the call; `arguments` is a 4096-byte structure as the request number states.
    let result: libc::c_int =
        unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_SNAP_CREATE_V2, &arguments) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error(
            operation,
            parent.join(name).as_path(),
        ));
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
pub fn snapshot_destroy(
    parent: &Path,
    name: &OsStr,
) -> Result<(), PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_SNAP_DESTROY";
    let directory: File = open_directory(parent, operation)?;
    // SAFETY: `VolArgs` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: VolArgs = unsafe { std::mem::zeroed() };
    fill_name(&mut arguments.name, name, operation, parent)?;
    // SAFETY: `directory` is open for the call; `arguments` is a 4096-byte structure as the request number states.
    let result: libc::c_int =
        unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_SNAP_DESTROY, &arguments) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error(
            operation,
            parent.join(name).as_path(),
        ));
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
pub fn subvolume_id(
    held: &File,
    path: &Path,
) -> Result<u64, PrimitiveError> {
    // `treeid` 0 asks for the subvolume of the file itself, and `objectid` 256 is the root directory of it; the rest is the
    // path buffer the kernel zeroes.
    let mut arguments: [u64; VOLUME_ARGUMENT_BYTES / 8] = [0; VOLUME_ARGUMENT_BYTES / 8];
    arguments[1] = BTRFS_FIRST_FREE_OBJECTID;
    // SAFETY: `held` is open for the call; `arguments` is a 4096-byte, 8-byte-aligned buffer as the request number states.
    let result: libc::c_int = unsafe {
        libc::ioctl(
            held.as_raw_fd(),
            BTRFS_IOC_INO_LOOKUP,
            arguments.as_mut_ptr(),
        )
    };
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
pub fn snapshot_destroy_by_id(
    directory: &File,
    subvolume: u64,
    path: &Path,
) -> Result<(), PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_SNAP_DESTROY_V2";
    // SAFETY: `VolArgsV2` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: VolArgsV2 = unsafe { std::mem::zeroed() };
    arguments.flags = BTRFS_SUBVOL_SPEC_BY_ID;
    // The id takes the place of the name in the structure's last union.
    arguments.name[..8].copy_from_slice(&subvolume.to_ne_bytes());
    // SAFETY: `directory` is open for the call; `arguments` is a 4096-byte structure as the request number states.
    let result: libc::c_int =
        unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_SNAP_DESTROY_V2, &arguments) };
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
pub fn snapshot_destroy_in(
    parent: &File,
    name: &OsStr,
    parent_path: &Path,
) -> Result<(), PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_SNAP_DESTROY";
    // SAFETY: `VolArgs` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: VolArgs = unsafe { std::mem::zeroed() };
    fill_name(&mut arguments.name, name, operation, parent_path)?;
    // SAFETY: `parent` is open for the call; `arguments` is a 4096-byte structure as the request number states.
    let result: libc::c_int =
        unsafe { libc::ioctl(parent.as_raw_fd(), BTRFS_IOC_SNAP_DESTROY, &arguments) };
    if result != 0 {
        return Err(PrimitiveError::last_os_error(
            operation,
            parent_path.join(name).as_path(),
        ));
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
pub fn split_parent(
    path: &Path,
    operation: &'static str,
) -> Result<(std::path::PathBuf, std::ffi::OsString), PrimitiveError> {
    let Some(name) = path.file_name() else {
        return Err(PrimitiveError {
            operation,
            path: path.to_path_buf(),
            errno: libc::EINVAL,
        });
    };
    let Some(parent) = path.parent() else {
        return Err(PrimitiveError {
            operation,
            path: path.to_path_buf(),
            errno: libc::EINVAL,
        });
    };
    let parent_path: std::path::PathBuf = if parent.as_os_str().is_empty() {
        std::path::PathBuf::from(".")
    } else {
        parent.to_path_buf()
    };
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
pub fn snapshot_create_at(
    source: &Path,
    destination: &Path,
) -> Result<(), PrimitiveError> {
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
///  Whether an ioctl's answer is a request that failed.
/// Why:
///   An ioctl answers `-1` and sets `errno` when the request failed,
/// and every answer from zero up is the request's own value:
///  `BTRFS_IOC_GET_SUBVOL_INFO` answers 1 for a subvolume whose root-backref search did not end at a leaf boundary,
/// so a check against zero reads a filled
///  structure as an error and reports whatever `errno` the process last set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAFailure = (answered: number) => answered < 0;
/// ```
fn is_a_failure(answered: &libc::c_int) -> bool {
    return *answered < 0;
}

/// What:
///  The record the request filled,
/// read from the answer it gave,
/// with the request itself handed in as a closure,
/// so a control on any filesystem can answer with the value it names.
/// Why:
///   The answer is a value the request defines rather than a status,
/// so the one place that decides whether the request failed is also the place a portable control drives,
///  with an answer of 1 and an answer of 0 and an answer of -1:
/// a comparison against zero written back here fails wherever the suite runs,
/// and no nested subvolume's record is
///  lost on a host whose kernel never answers 1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordFromAnswer(operation: string, path: string, answerTheRequest: (arguments: SubvolumeInfoArgs) => number): SubvolumeInfo
/// ```
fn record_from_answer(
    operation: &'static str,
    path: &Path,
    answer_the_request: impl FnOnce(&mut SubvolumeInfoArgs) -> libc::c_int,
) -> Result<SubvolumeInfo, PrimitiveError> {
    // SAFETY: `SubvolumeInfoArgs` is plain integers and bytes, so the all-zero value is a valid value.
    let mut arguments: SubvolumeInfoArgs = unsafe { std::mem::zeroed() };
    // What: the answer the caller's request gave, read once.
    // Why: both the failure check and the record this returns need it, and `errno` stops being reliable after any call.
    let answered: libc::c_int = answer_the_request(&mut arguments);
    if is_a_failure(&answered) {
        return Err(PrimitiveError::last_os_error(operation, path));
    }
    return Ok(SubvolumeInfo {
        uuid: arguments.uuid,
        parent_uuid: arguments.parent_uuid,
        generation: arguments.generation,
        transid_at_creation: arguments.otransid,
    });
}

/// What:
///  Read what the subvolume holding `path` says about itself (`BTRFS_IOC_GET_SUBVOL_INFO`).
/// Why:
///   The request is answered for the subvolume the opened directory belongs to,
/// so any directory inside it answers,
/// and the kernel needs no privilege the caller does not already have on that directory.
/// The kernel answers 1 for a subvolume that has a parent backref
/// (or 0 when that search ends at a leaf boundary)
/// and 0 for the top-level subvolume,
/// so the answer is a value the request defines rather than a status,
/// and the reading of it lives in one function a control on any filesystem drives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const info = await subvolumeInfo(path); // { uuid, parentUuid, generation, transidAtCreation }
/// ```
pub fn subvolume_info(path: &Path) -> Result<SubvolumeInfo, PrimitiveError> {
    let operation: &'static str = "BTRFS_IOC_GET_SUBVOL_INFO";
    let directory: File = open_directory(path, operation)?;
    // What: the request itself, handed to the one reader so this call site never holds its answer.
    // Why: a comparison here would answer for every nested subvolume with a stale `errno` and no portable control could see it.
    let answer_the_request = |arguments: &mut SubvolumeInfoArgs| -> libc::c_int {
        // SAFETY: `directory` is open for the call; `arguments` is the 504-byte structure the request number states.
        return unsafe { libc::ioctl(directory.as_raw_fd(), BTRFS_IOC_GET_SUBVOL_INFO, arguments) };
    };
    return record_from_answer(operation, path, answer_the_request);
}

/// Btrfs ioctl controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_btrfs_ioctl_tests.rs"]
mod tests;
