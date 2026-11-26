//! What:
//!  The descriptor calls the companion's child makes:
//!  the no-crossing walk,
//!  the file-system context of its own tmpfs,
//!  the create,
//!  the mount descriptor,
//!  the move onto the held slot,
//!  and the mount facts read back.
//! Why:
//!  The design walks its own tmpfs and attaches a second tmpfs through
//!  descriptors rather than paths,
//!  so the call sequence is the probe's own over a filesystem the companion
//!  owns
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "What it runs").
//!  Nothing here reads ZFS,
//! the key path or `/dev/zfs`.
//!  No call allocates:
//! every name came from the parent and every buffer is the
//!  caller's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slot = openat2(root, 'slot', NO_XDEV | NO_SYMLINKS | BENEATH);
//! fsconfigCreate(fsopen('tmpfs')); moveMount(fsmount(context), slot);
//! ```

use super::companion_calls::{
    CompanionCall,
    Failure,
    failed,
};
use std::ffi::CStr;
use std::os::fd::RawFd;

/// `RESOLVE_NO_XDEV`:
/// no mount point is crossed.
const RESOLVE_NO_XDEV: u64 = 0x01;
/// `RESOLVE_NO_MAGICLINKS`:
/// no `/proc` style link is followed.
const RESOLVE_NO_MAGICLINKS: u64 = 0x02;
/// `RESOLVE_NO_SYMLINKS`:
/// no symbolic link is followed.
const RESOLVE_NO_SYMLINKS: u64 = 0x04;
/// `RESOLVE_BENEATH`:
/// the lookup stays below the directory it starts from.
const RESOLVE_BENEATH: u64 = 0x08;

/// The arguments of `openat2`,
/// laid out as the kernel's `struct open_how`.
#[repr(C)]
struct OpenHow {
    /// The `open` flags.
    flags: u64,
    /// The mode of a created file.
    mode: u64,
    /// The `RESOLVE_*` bits.
    resolve: u64,
}

/// The flags a directory is opened with.
const DIRECTORY_FLAGS: libc::c_int =
    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW;

/// What:
///  Open one directory by path,
///  following no link.
///
/// # Errors
/// The call refused.
pub fn open_directory(path: &CStr) -> Result<RawFd, Failure> {
    // SAFETY: the path outlives the call and the descriptor it answers is owned once.
    let opened: libc::c_int = unsafe { libc::open(path.as_ptr(), DIRECTORY_FLAGS) };
    return if opened < 0 {
        Err(failed(CompanionCall::OpenDirectory))
    } else {
        Ok(opened)
    };
}

/// What:
///  Open one directory below a held one,
///  crossing no mount,
///  following no link and leaving nothing beneath.
/// Why:
///  This is the walk the companion repeats from the probe,
///  and the kernel's own resolve bits are what refuse a crossing.
///
/// # Errors
/// The call refused,
///  with the kernel's answer.
pub fn open_directory_beneath(
    held: RawFd,
    name: &CStr,
) -> Result<RawFd, Failure> {
    let how: OpenHow = OpenHow {
        flags: (libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC) as u64,
        mode: 0,
        resolve: RESOLVE_NO_XDEV | RESOLVE_NO_MAGICLINKS | RESOLVE_NO_SYMLINKS | RESOLVE_BENEATH,
    };
    // SAFETY: the held descriptor is the caller's, the name and the structure outlive the call, and the answer is owned once.
    let opened: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_openat2,
            held,
            name.as_ptr(),
            (&how as *const OpenHow).cast::<libc::c_void>(),
            std::mem::size_of::<OpenHow>(),
        ) as libc::c_int
    };
    return if opened < 0 {
        Err(failed(CompanionCall::Openat2))
    } else {
        Ok(opened)
    };
}

/// What:
///  Open the file-system context of the tmpfs the attach makes.
///
/// # Errors
/// The call refused.
pub fn fsopen_tmpfs(tmpfs: &CStr) -> Result<RawFd, Failure> {
    // SAFETY: the type string outlives the call and the descriptor it answers is owned once.
    let context: libc::c_int = unsafe {
        libc::syscall(libc::SYS_fsopen, tmpfs.as_ptr(), libc::FSOPEN_CLOEXEC) as libc::c_int
    };
    return if context < 0 {
        Err(failed(CompanionCall::FsOpen))
    } else {
        Ok(context)
    };
}

/// What:
///  Create the superblock from a context.
/// Why:
///  The probe's exclusive create is that probe's freshness oracle;
///  this mount is the companion's own tmpfs,
///  so a plain create is the call the design names here.
///
/// # Errors
/// The call refused.
pub fn fsconfig_create(context: RawFd) -> Result<(), Failure> {
    // SAFETY: the context is the caller's and the call reads no pointer here.
    let created: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_fsconfig,
            context,
            libc::FSCONFIG_CMD_CREATE,
            std::ptr::null::<libc::c_char>(),
            std::ptr::null::<libc::c_char>(),
            0,
        ) as libc::c_int
    };
    return if created == 0 {
        Ok(())
    } else {
        Err(failed(CompanionCall::FsConfig))
    };
}

/// What:
///  Make the mount from a created context.
///
/// # Errors
/// The call refused.
pub fn fsmount(context: RawFd) -> Result<RawFd, Failure> {
    // SAFETY: the context is the caller's and has been created; the descriptor it answers is owned once.
    let mount: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_fsmount,
            context,
            libc::FSMOUNT_CLOEXEC,
            libc::MOUNT_ATTR_RDONLY | libc::MOUNT_ATTR_NOSUID | libc::MOUNT_ATTR_NODEV,
        ) as libc::c_int
    };
    return if mount < 0 {
        Err(failed(CompanionCall::FsMount))
    } else {
        Ok(mount)
    };
}

/// What:
///  Place a mount onto a held directory descriptor.
/// Why:
///  The descriptor is the slot the walk proved,
///  so a folder renamed after the walk cannot redirect the mount.
///
/// # Errors
/// The call refused.
pub fn move_mount_onto_descriptor(
    mount: RawFd,
    target: RawFd,
) -> Result<(), Failure> {
    // SAFETY: both descriptors are the caller's, and the two empty paths are NUL-terminated strings the flags read.
    let moved: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_move_mount,
            mount,
            c"".as_ptr(),
            target,
            c"".as_ptr(),
            libc::MOVE_MOUNT_F_EMPTY_PATH | libc::MOVE_MOUNT_T_EMPTY_PATH,
        ) as libc::c_int
    };
    return if moved == 0 {
        Ok(())
    } else {
        Err(failed(CompanionCall::MoveMount))
    };
}

/// What:
///  Move a mount between two paths inside this namespace,
///  which is the move the second child makes before its unmount.
///
/// # Errors
/// The call refused.
pub fn move_mount_between_paths(
    from: &CStr,
    to: &CStr,
) -> Result<(), Failure> {
    // SAFETY: both paths are the caller's and outlive the call, and it reads no other pointer.
    let moved: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_move_mount,
            libc::AT_FDCWD,
            from.as_ptr(),
            libc::AT_FDCWD,
            to.as_ptr(),
            0,
        ) as libc::c_int
    };
    return if moved == 0 {
        Ok(())
    } else {
        Err(failed(CompanionCall::MoveMount))
    };
}
