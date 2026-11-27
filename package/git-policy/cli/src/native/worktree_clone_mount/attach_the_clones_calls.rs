//! What:
//!  The four kernel calls one attach is made of:
//!  `fsopen` for a context of the `zfs` filesystem type,
//!  `fsconfig` twice to name the dataset and ask for the superblock,
//!  `fsmount` for the mount descriptor with its two enforced attributes,
//!  and `move_mount` onto the held slot descriptor.
//! Why:
//!  The design attaches through descriptors rather than paths and never
//!  resolves a path inside the account's tree,
//!  so each call takes the descriptor the one before it returned
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  Every call's own errno reaches the refusal it makes,
//!  which `attach_the_clones_words.rs` writes
//!  and `attach_the_clone.rs` puts in order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const context = openContext('zfs', source);
//! setSource(context, source); create(context);
//! const mount = makeMount(context, target, ATTACH_FSMOUNT_ATTRIBUTES);
//! ```

use super::attach_the_clones_words::{
    ATTACH_CLOSE_ON_EXEC,
    AttachCall,
    AttachRefusal,
    SOURCE_OPTION,
    refused,
};
use std::ffi::{
    CString,
    c_int,
};
use std::os::fd::{
    FromRawFd,
    OwnedFd,
    RawFd,
};

/// The open file-system context of one filesystem type.
///
/// # Errors
/// `fsopen` refused.
pub(super) fn open_context(
    filesystem: &str,
    source: &str,
) -> Result<OwnedFd, AttachRefusal> {
    let file_system_type: CString = CString::new(filesystem).expect("no NUL in a filesystem type");
    // SAFETY: the type string is this call's own and outlives it; the descriptor it returns is checked before it is owned.
    let context: c_int = unsafe {
        libc::syscall(
            libc::SYS_fsopen,
            file_system_type.as_ptr(),
            ATTACH_CLOSE_ON_EXEC.0,
        ) as c_int
    };
    if context < 0 {
        return Err(refused(
            AttachCall::FileSystemContext,
            source,
            std::io::Error::last_os_error(),
        ));
    }
    // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
    return Ok(unsafe { OwnedFd::from_raw_fd(context) });
}

/// Name the dataset the context is for.
///
/// # Errors
/// `fsconfig` refused the option.
pub(super) fn set_source(
    context: RawFd,
    source: &str,
) -> Result<(), AttachRefusal> {
    let key: CString = CString::new(SOURCE_OPTION).expect("no NUL in an option name");
    let value: CString = CString::new(source).expect("no NUL in a dataset name");
    // SAFETY: the context descriptor is the caller's, and both strings outlive the call.
    let set: c_int = unsafe {
        libc::syscall(
            libc::SYS_fsconfig,
            context,
            libc::FSCONFIG_SET_STRING,
            key.as_ptr(),
            value.as_ptr(),
            0,
        ) as c_int
    };
    return if set == 0 {
        Ok(())
    } else {
        Err(refused(
            AttachCall::SourceOption,
            source,
            std::io::Error::last_os_error(),
        ))
    };
}

/// Create the superblock from the context.
///
/// # Errors
/// `fsconfig` refused the create.
pub(super) fn create(
    context: RawFd,
    source: &str,
) -> Result<(), AttachRefusal> {
    // SAFETY: the context descriptor is the caller's and the call reads no pointer here.
    let created: c_int = unsafe {
        libc::syscall(
            libc::SYS_fsconfig,
            context,
            libc::FSCONFIG_CMD_CREATE,
            std::ptr::null::<std::ffi::c_char>(),
            std::ptr::null::<std::ffi::c_char>(),
            0,
        ) as c_int
    };
    return if created == 0 {
        Ok(())
    } else {
        Err(refused(
            AttachCall::Create,
            source,
            std::io::Error::last_os_error(),
        ))
    };
}

/// Make the mount descriptor with the attributes the caller resolved.
///
/// # Errors
/// `fsmount` refused.
pub(super) fn make_mount(
    context: RawFd,
    source: &str,
    attributes: u64,
) -> Result<OwnedFd, AttachRefusal> {
    // SAFETY: the context descriptor is the caller's and has been created; the returned descriptor is owned once.
    let mount: c_int = unsafe {
        libc::syscall(
            libc::SYS_fsmount,
            context,
            ATTACH_CLOSE_ON_EXEC.1,
            attributes,
        ) as c_int
    };
    if mount < 0 {
        return Err(refused(
            AttachCall::MountDescriptor,
            source,
            std::io::Error::last_os_error(),
        ));
    }
    // SAFETY: the call answered a descriptor this process owns, and it is owned exactly once.
    return Ok(unsafe { OwnedFd::from_raw_fd(mount) });
}
