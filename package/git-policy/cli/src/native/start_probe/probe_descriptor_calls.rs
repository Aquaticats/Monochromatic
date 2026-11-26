//! What:
//!  The descriptor attach's calls:
//!  the file-system context,
//!  its source and options,
//!  the exclusive create,
//!  the mount descriptor and the move onto a held directory descriptor.
//! Why:
//!  The design attaches through descriptors rather than paths,
//!  and the exclusive create is the freshness oracle the record relies on
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms").
//!  A kernel older than 6.6 refuses the exclusive command with
//!  `EOPNOTSUPP`,
//!  which the arm records as a step that could not run instead of falling back
//!  to a plain create.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const context = fsopen('zfs'); fsconfig(context, 'source', source); fsconfigCreate(context, true);
//! const mount = fsmount(context); moveMount(mount, slotDescriptor);
//! ```

use super::probe_call_error::{
    CallError,
    failed,
};
use std::ffi::CString;
use std::os::fd::{
    FromRawFd,
    OwnedFd,
    RawFd,
};

/// The `fsopen` flag that keeps the context out of a started child.
pub const FSOPEN_CLOEXEC: libc::c_uint = libc::FSOPEN_CLOEXEC;
/// The `fsmount` flag that keeps the mount out of a started child.
pub const FSMOUNT_CLOEXEC: libc::c_uint = libc::FSMOUNT_CLOEXEC;
/// The flags the attach asks for:
/// read-only,
/// no set-user-ID execution and no device nodes.
pub const PROBE_MOUNT_ATTRIBUTES: u64 =
    libc::MOUNT_ATTR_RDONLY | libc::MOUNT_ATTR_NOSUID | libc::MOUNT_ATTR_NODEV;

/// The filesystem type the probe's source is mounted as.
pub const ZFS_FILESYSTEM: &str = "zfs";
/// The context option that names the dataset.
pub const SOURCE_OPTION: &str = "source";
/// The context's read-only option.
pub const READ_ONLY_OPTION: &str = "ro";
/// The context's `nosuid` option.
pub const NOSUID_OPTION: &str = "nosuid";
/// The context's `nodev` option.
pub const NODEV_OPTION: &str = "nodev";

/// What:
///  Open the file-system context of one filesystem type.
/// Why:
///   The attach starts here,
///  and its descriptor is the one that carries the source and the options.
///
/// # Errors
/// The call refused.
pub fn fsopen(filesystem: &str) -> Result<OwnedFd, CallError> {
    let type_name: CString = CString::new(filesystem).expect("no NUL in a filesystem type");
    // SAFETY: the type string is this call's own and outlives it; the descriptor it returns is checked before it is owned.
    let context: libc::c_int = unsafe {
        libc::syscall(libc::SYS_fsopen, type_name.as_ptr(), FSOPEN_CLOEXEC) as libc::c_int
    };
    if context < 0 {
        return Err(failed("fsopen"));
    }
    // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
    return Ok(unsafe { OwnedFd::from_raw_fd(context) });
}

/// What:
///  Set one string option of a context,
///  such as the source.
/// Why:
///   The context refuses a create until it names the dataset it is for.
///
/// # Errors
/// The call refused.
pub fn fsconfig_text(
    context: RawFd,
    key: &str,
    value: &str,
) -> Result<(), CallError> {
    let key_name: CString = CString::new(key).expect("no NUL in an option name");
    let option_value: CString = CString::new(value).expect("no NUL in an option value");
    // SAFETY: the context descriptor is the caller's, and both strings outlive the call.
    let set: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_fsconfig,
            context,
            libc::FSCONFIG_SET_STRING,
            key_name.as_ptr(),
            option_value.as_ptr(),
            0,
        ) as libc::c_int
    };
    return if set == 0 {
        Ok(())
    } else {
        Err(failed("fsconfig"))
    };
}

/// What:
///  Turn one flag option of a context on,
///  such as `ro`.
/// Why:
///   The read-only request is a flag of the generic context,
///  and the options are written before the create.
///
/// # Errors
/// The call refused.
pub fn fsconfig_flag(
    context: RawFd,
    key: &str,
) -> Result<(), CallError> {
    let key_name: CString = CString::new(key).expect("no NUL in an option name");
    // SAFETY: the context descriptor is the caller's and the key outlives the call.
    let set: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_fsconfig,
            context,
            libc::FSCONFIG_SET_FLAG,
            key_name.as_ptr(),
            std::ptr::null::<libc::c_char>(),
            0,
        ) as libc::c_int
    };
    return if set == 0 {
        Ok(())
    } else {
        Err(failed("fsconfig"))
    };
}

/// What:
///  Create the superblock from a context,
///  exclusively where the freshness oracle is asked for.
/// Why:
///   `FSCONFIG_CMD_CREATE_EXCL` answers `EBUSY` instead of reusing any extant
///  superblock,
///  which is what proves the attach is a fresh one.
///
/// # Errors
/// The call refused.
pub fn fsconfig_create(
    context: RawFd,
    exclusive: bool,
) -> Result<(), CallError> {
    let command: libc::c_uint = if exclusive {
        libc::FSCONFIG_CMD_CREATE_EXCL
    } else {
        libc::FSCONFIG_CMD_CREATE
    };
    // SAFETY: the context descriptor is the caller's and the call reads no pointer here.
    let created: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_fsconfig,
            context,
            command,
            std::ptr::null::<libc::c_char>(),
            std::ptr::null::<libc::c_char>(),
            0,
        ) as libc::c_int
    };
    return if created == 0 {
        Ok(())
    } else {
        Err(failed("fsconfig create"))
    };
}

/// What:
///  Make the mount from a created context,
///  with the attributes the probe asks for.
/// Why:
///   The descriptor is the mount before it is placed,
///  and its ID is what the read-back looks for.
///
/// # Errors
/// The call refused.
pub fn fsmount(context: RawFd) -> Result<OwnedFd, CallError> {
    // SAFETY: the context descriptor is the caller's and has been created; the returned descriptor is owned once.
    let mount: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_fsmount,
            context,
            FSMOUNT_CLOEXEC,
            PROBE_MOUNT_ATTRIBUTES,
        ) as libc::c_int
    };
    if mount < 0 {
        return Err(failed("fsmount"));
    }
    // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
    return Ok(unsafe { OwnedFd::from_raw_fd(mount) });
}

/// What:
///  Place a mount onto a held directory descriptor.
/// Why:
///   The design attaches through the descriptor rather than a path,
///  so a folder renamed after the walk cannot redirect the mount.
///
/// # Errors
/// The call refused.
pub fn move_mount_onto_descriptor(
    mount: RawFd,
    target: RawFd,
) -> Result<(), CallError> {
    let empty: CString = CString::new("").expect("no NUL in an empty path");
    // SAFETY: both descriptors are the caller's, and the two empty paths are NUL-terminated strings the flags read.
    let moved: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_move_mount,
            mount,
            empty.as_ptr(),
            target,
            empty.as_ptr(),
            libc::MOVE_MOUNT_F_EMPTY_PATH | libc::MOVE_MOUNT_T_EMPTY_PATH,
        ) as libc::c_int
    };
    return if moved == 0 {
        Ok(())
    } else {
        Err(failed("move_mount"))
    };
}

/// What:
///  Move a mount between two paths.
/// Why:
///  Arm 8's child resolves the slot and the move target by path and moves the
///  mount between them,
///  which is the probe-private move the design places in this step.
///
/// # Errors
/// The call refused.
pub fn move_mount_between_paths(
    from: &std::path::Path,
    to: &std::path::Path,
) -> Result<(), CallError> {
    use std::os::unix::ffi::OsStrExt;
    let from_path: CString = CString::new(from.as_os_str().as_bytes()).expect("no NUL in a path");
    let to_path: CString = CString::new(to.as_os_str().as_bytes()).expect("no NUL in a path");
    // SAFETY: both paths are this call's own and outlive it, and it reads no other pointer.
    let moved: libc::c_int = unsafe {
        libc::syscall(
            libc::SYS_move_mount,
            libc::AT_FDCWD,
            from_path.as_ptr(),
            libc::AT_FDCWD,
            to_path.as_ptr(),
            0,
        ) as libc::c_int
    };
    return if moved == 0 {
        Ok(())
    } else {
        Err(failed("move_mount"))
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_descriptor_calls_tests.rs"]
mod tests;
