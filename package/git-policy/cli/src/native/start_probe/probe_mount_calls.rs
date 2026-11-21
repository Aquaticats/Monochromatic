//! What:
//!  The namespace,
//!  propagation,
//!  scratch-tmpfs,
//!  unmount and mount-table calls the probe's arms make.
//! Why:
//!   Every mount the probe makes lives in a private namespace of its own and
//!  dies with it,
//!  and the unmount by path and the read-back are the two calls that prove the
//!  arm rather than assuming it
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms").
//!  Each answers the errno its call gave and decides nothing:
//!  what an errno means belongs to the arm.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! unshareMountNamespace(); makeMountsPrivate(); mountScratchTmpfs(scratch);
//! ```

use super::probe_call_error::{CallError, failed};
use std::ffi::CString;
use std::os::fd::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// What:
///  Make this process's mount namespace,
///  and every mount inside it,
///  private.
/// Why:
///   A mount that propagates reaches the host mount table,
///  which is the one thing the probe must never do,
///  and the propagation is made before any mount exists in the namespace.
///
/// # Errors
/// The `mount_setattr` and its `mount` fallback both refused.
pub fn make_mounts_private() -> Result<(), CallError> {
    let attributes: libc::mount_attr = libc::mount_attr { attr_set: 0, attr_clr: 0, propagation: libc::MS_PRIVATE as u64, userns_fd: 0 };
    let root: CString = CString::new("/").expect("no NUL in the root path");
    // SAFETY: the root path and the attribute structure are this call's own and outlive it.
    let set: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_mount_setattr,
            libc::AT_FDCWD,
            root.as_ptr(),
            libc::AT_RECURSIVE,
            (&attributes as *const libc::mount_attr).cast::<libc::c_void>(),
            std::mem::size_of::<libc::mount_attr>(),
        ) as libc::c_int
    };
    if set == 0 {
        return Ok(());
    }
    let answered: CallError = failed("mount_setattr");
    // SAFETY: every argument is this call's own: the empty source, the root path and no data pointer.
    let mounted: libc::c_int = unsafe { libc::mount(std::ptr::null(), root.as_ptr(), std::ptr::null(), libc::MS_REC | libc::MS_PRIVATE, std::ptr::null()) };
    return if mounted == 0 { Ok(()) } else { Err(answered) };
}

/// The scratch tmpfs's size and inode count:
/// large enough for the slot,
/// the move target,
/// the refusal tree and one symbolic link,
/// and no larger.
pub const SCRATCH_TMPFS_OPTIONS: &str = "size=64k,nr_inodes=32,mode=0700";

/// What:
///  Mount a fresh tmpfs at the scratch mount point.
/// Why:
///   The probe's slot,
///  its move target and its refusal tree live inside a filesystem the probe
///  owns,
///  so nothing it writes is visible outside its namespace,
///  and the size bound keeps that filesystem no larger than the directories
///  it must hold.
///
/// # Errors
/// The `mount` call refused.
pub fn mount_scratch_tmpfs(target: &Path) -> Result<(), CallError> {
    let target_path: CString = c_path(target);
    let filesystem: CString = CString::new("tmpfs").expect("no NUL in a filesystem type");
    let data: CString = CString::new(SCRATCH_TMPFS_OPTIONS).expect("no NUL in a mount option");
    // SAFETY: every pointer is this call's own and outlives it.
    let mounted: libc::c_int = unsafe { libc::mount(filesystem.as_ptr(), target_path.as_ptr(), filesystem.as_ptr(), libc::MS_NOSUID | libc::MS_NODEV, data.as_ptr().cast::<libc::c_void>()) };
    return if mounted == 0 { Ok(()) } else { Err(failed("mount tmpfs")) };
}

/// What:
///  `unshare(CLONE_NEWNS)`:
///  a private mount namespace of this process's own.
/// Why:
///   Every mount the probe makes lives in this namespace and dies with it,
///  which is what makes an interrupted probe leave nothing in the host table.
///
/// # Errors
/// The call refused.
pub fn unshare_mount_namespace() -> Result<(), CallError> {
    // SAFETY: `unshare` reads the one flag word and touches no memory of ours.
    let made: libc::c_int = unsafe { libc::unshare(libc::CLONE_NEWNS) };
    return if made == 0 { Ok(()) } else { Err(failed("unshare")) };
}

/// What:
///  Unmount one path,
///  which must not answer `EBUSY`.
/// Why:
///  A busy unmount is this arm's failure,
///  and the design names that answer rather than retrying it.
///
/// # Errors
/// The call refused.
pub fn unmount(target: &Path) -> Result<(), CallError> {
    let target_path: CString = c_path(target);
    // SAFETY: the path is this call's own and outlives it; no lazy detach is asked for.
    let unmounted: libc::c_int = unsafe { libc::umount2(target_path.as_ptr(), 0) };
    return if unmounted == 0 { Ok(()) } else { Err(failed("umount2")) };
}

/// What:
///  Bind one folder onto another inside this namespace,
///  which the refusal control plants across a component.
/// Why:
///  The walk must refuse to cross a mount,
///  and a bind mount is the cheapest one to plant inside the probe's own tmpfs.
///
/// # Errors
/// The call refused.
pub fn bind_mount(source: &Path, target: &Path) -> Result<(), CallError> {
    let source_path: CString = c_path(source);
    let target_path: CString = c_path(target);
    // SAFETY: both paths are this call's own and outlive it; a bind mount reads no data pointer.
    let mounted: libc::c_int = unsafe { libc::mount(source_path.as_ptr(), target_path.as_ptr(), std::ptr::null(), libc::MS_BIND, std::ptr::null()) };
    return if mounted == 0 { Ok(()) } else { Err(failed("mount bind")) };
}

/// What:
///  The mount ID one path's descriptor is on,
///  read from the kernel's own `fdinfo`.
/// Why:
///   The read-back needs the ID the `fsmount` descriptor created,
///  before the move makes it a placed mount,
///  and this is the wrapper's one reader for that number.
///
/// # Errors
/// The descriptor's `fdinfo` could not be read or carries no mount ID.
pub fn mount_id_of(fd: RawFd) -> Result<u64, CallError> {
    let path: String = format!("/proc/self/fdinfo/{fd}");
    let text: String = std::fs::read_to_string(path.as_str()).map_err(|_| return CallError { call: "fdinfo", errno: libc::ENOENT })?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("mnt_id:") {
            let Ok(number) = rest.trim().parse::<u64>() else {
                return Err(CallError { call: "fdinfo", errno: libc::EINVAL });
            };
            return Ok(number);
        }
    }
    return Err(CallError { call: "fdinfo", errno: libc::ENOENT });
}

/// What:
///  The mount table of this process's own namespace,
///  read bounded.
/// Why:
///   The read-back compares the mount the attach made with what the kernel
///  lists,
///  and the table belongs to the worker's private namespace rather than the
///  host's.
///
/// # Errors
/// The table could not be read or is longer than the bound.
pub fn read_own_mount_table() -> Result<Vec<crate::fs_mount_options::MountFacts>, CallError> {
    let text: Vec<u8> = crate::plain_file::read_plain_file(Path::new(crate::fs_mount_options::MOUNTINFO_PATH), crate::plain_file::MOUNT_TABLE_LIMIT)
        .map_err(|_| return CallError { call: "mountinfo", errno: libc::E2BIG })?;
    return Ok(crate::fs_mount_options::parse_mount_table(text.as_slice()));
}

/// The path as the NUL-terminated bytes the calls take.
fn c_path(path: &Path) -> CString {
    return CString::new(path.as_os_str().as_bytes()).expect("a path holds no NUL");
}


/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_mount_calls_tests.rs"]
mod tests;
