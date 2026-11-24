//! What:
//!  The mount calls the companion's child makes inside its own namespaces:
//!  the private propagation of every mount in it,
//!  the scratch tmpfs,
//!  the bind mount the crossing control plants,
//!  and the unmount the second child makes.
//! Why:
//!  Every one of these lives and dies with the child's namespace,
//!  so nothing the companion mounts reaches the host's mount table
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion").
//!  The propagation is made with `mount_setattr` and,
//!  where that call refuses,
//!  with `mount`,
//!  and the answer the design's skip list names is the last call's own.
//!  Nothing here allocates.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! makeMountsPrivate('/'); mountTmpfs('tmpfs', scratch, 'size=64k,mode=0700'); bindMount(slot, bound); umount(moveTarget);
//! ```

use super::companion_calls::{CompanionCall, Failure, failed};
use std::ffi::CStr;

/// What:
///  Make every mount in this process's namespace private,
///  first with `mount_setattr` and then,
///  where that call refuses,
///  with `mount`.
/// Why:
///  A mount that propagates reaches the host mount table,
///  which is the one thing the companion must never do.
///
/// # Errors
/// Both calls refused.
pub fn make_mounts_private(root: &CStr) -> Result<(), Failure> {
    let attributes: libc::mount_attr = libc::mount_attr { attr_set: 0, attr_clr: 0, propagation: libc::MS_PRIVATE as u64, userns_fd: 0 };
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
    // SAFETY: every argument is this call's own: no source, the root path and no data.
    let mounted: libc::c_int = unsafe { libc::mount(std::ptr::null(), root.as_ptr(), std::ptr::null(), libc::MS_REC | libc::MS_PRIVATE, std::ptr::null()) };
    if mounted == 0 {
        return Ok(());
    }
    // The fallback refused, and it is the call whose errno the design's `EACCES` bullet names when the propagation is made this way.
    return Err(failed(CompanionCall::MountPropagation));
}

/// What:
///  Mount a fresh tmpfs at the scratch mount point.
///
/// # Errors
/// The call refused.
pub fn mount_tmpfs(tmpfs: &CStr, target: &CStr, options: &CStr) -> Result<(), Failure> {
    // SAFETY: every pointer is this call's own and outlives it.
    let mounted: libc::c_int = unsafe { libc::mount(tmpfs.as_ptr(), target.as_ptr(), tmpfs.as_ptr(), libc::MS_NOSUID | libc::MS_NODEV, options.as_ptr().cast::<libc::c_void>()) };
    return if mounted == 0 { Ok(()) } else { Err(failed(CompanionCall::MountTmpfs)) };
}

/// What:
///  Bind one path onto another inside this namespace,
///  which is how the crossing control plants a mount across a walk component.
///
/// # Errors
/// The call refused.
pub fn bind_mount(source: &CStr, target: &CStr) -> Result<(), Failure> {
    // SAFETY: both paths are this call's own and outlive it; a bind mount reads no data pointer.
    let mounted: libc::c_int = unsafe { libc::mount(source.as_ptr(), target.as_ptr(), std::ptr::null(), libc::MS_BIND, std::ptr::null()) };
    return if mounted == 0 { Ok(()) } else { Err(failed(CompanionCall::MountTmpfs)) };
}

/// What:
///  Unmount one path,
///  which must not answer `EBUSY`.
///
/// # Errors
/// The call refused.
pub fn unmount(target: &CStr) -> Result<(), Failure> {
    // SAFETY: the path is this call's own and outlives it; no lazy detach is asked for.
    let unmounted: libc::c_int = unsafe { libc::umount2(target.as_ptr(), 0) };
    return if unmounted == 0 { Ok(()) } else { Err(failed(CompanionCall::Umount)) };
}
