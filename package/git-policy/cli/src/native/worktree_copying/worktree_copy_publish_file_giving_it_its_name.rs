//! What:
//!  The calls that give a filled file its destination name in one step,
//!  and the removal of a name an interrupted fallback left.
//! Why:
//!  A reader must never see a partial file at the destination's name,
//! so the name is taken by a link or a rename that does not replace,
//!  and the fallback's own temporary goes before it is made again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await linkUnnamed(file, destination); // or renameNoReplace(temporary, destination)
//! ```

use crate::worktree_copy_publish_file_making_the_file::{
    RENAME_UNSUPPORTED_CODES,
    invalid_name,
    is_unsupported,
};
use std::ffi::{
    CString,
    OsStr,
};
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// What:
///  Give the open file the destination name,
/// and never another object's name up for it.
/// Why:
///   A link is one step:
/// the name either names this file with every byte and its mode,
///  or it names what was there before,
///  so a reader never sees a partial file and nothing is ever replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await linkat(unnamed, destination); // EEXIST when anything is there
/// ```
pub(super) fn link_unnamed(
    file: &std::fs::File,
    destination: &Path,
) -> std::io::Result<()> {
    let text: CString = CString::new(destination.as_os_str().as_bytes()).map_err(invalid_name)?;
    // The descriptor's path is the file itself, so the link names the object and not a directory entry.
    let through_descriptor: CString =
        CString::new(format!("/proc/self/fd/{}", file.as_raw_fd())).map_err(invalid_name)?;
    // SAFETY: both strings are NUL-terminated and outlive the call.
    let linked: libc::c_int = unsafe {
        libc::linkat(
            libc::AT_FDCWD,
            through_descriptor.as_ptr(),
            libc::AT_FDCWD,
            text.as_ptr(),
            libc::AT_SYMLINK_FOLLOW,
        )
    };
    if linked == 0 {
        return Ok(());
    }
    let refusal: std::io::Error = std::io::Error::last_os_error();
    // A system without the descriptor directory still links the file by the empty name that names it.
    if refusal.raw_os_error() != Some(libc::ENOENT) && refusal.raw_os_error() != Some(libc::EPERM) {
        return Err(refusal);
    }
    // SAFETY: the empty string is a NUL-terminated string that outlives the call, and `file` is an open descriptor.
    let empty_after_descriptor: libc::c_int = unsafe {
        libc::linkat(
            file.as_raw_fd(),
            c"".as_ptr(),
            libc::AT_FDCWD,
            text.as_ptr(),
            libc::AT_EMPTY_PATH,
        )
    };
    if empty_after_descriptor == 0 {
        return Ok(());
    }
    return Err(std::io::Error::last_os_error());
}

/// What:
///  Move `temporary` to `destination`,
/// and never over anything already there.
/// Why:
///   A rename that does not replace is one step,
/// so the destination name either still holds what it held or holds the whole file;
///  a filesystem without that flag publishes by a hard link whose staging name is removed after,
/// which is one step as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await renameat2(dir, temporary, dir, destination, RENAME_NOREPLACE) } catch (e) { if (e.code === 'EEXIST') throw e; await link(temporary, destination); await unlink(temporary); }
/// ```
pub(super) fn move_temporary_into_place(
    directory: &std::fs::File,
    temporary: &OsStr,
    destination: &OsStr,
) -> std::io::Result<()> {
    let from: CString = CString::new(temporary.as_bytes()).map_err(invalid_name)?;
    let to: CString = CString::new(destination.as_bytes()).map_err(invalid_name)?;
    // SAFETY: both strings are NUL-terminated and outlive the call, and `directory` is an open descriptor.
    let renamed: libc::c_int = unsafe {
        libc::renameat2(
            directory.as_raw_fd(),
            from.as_ptr(),
            directory.as_raw_fd(),
            to.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if renamed == 0 {
        return Ok(());
    }
    let refusal: std::io::Error = std::io::Error::last_os_error();
    if is_unsupported(&refusal, RENAME_UNSUPPORTED_CODES) {
        // SAFETY: both strings are NUL-terminated and outlive the call, and `directory` is an open descriptor.
        let linked: libc::c_int = unsafe {
            libc::linkat(
                directory.as_raw_fd(),
                from.as_ptr(),
                directory.as_raw_fd(),
                to.as_ptr(),
                0,
            )
        };
        if linked != 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: the string is NUL-terminated and outlives the call, and `directory` is an open descriptor.
        let _unlinked: libc::c_int =
            unsafe { libc::unlinkat(directory.as_raw_fd(), from.as_ptr(), 0) };
        return Ok(());
    }
    return Err(refusal);
}

/// What:
///  Remove the entry of this name in `directory`,
/// accepting an absent one.
/// Why:
///   The fallback's temporary is named after the stage and the entry,
/// so a name left by an interrupted run of this very copy is the one that must go before it is made again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await unlinkat(dir, name).catch((e) => { if (e.code !== 'ENOENT') throw e; });
/// ```
pub(super) fn remove_present(
    directory: &std::fs::File,
    name: &OsStr,
) -> std::io::Result<()> {
    let text: CString = CString::new(name.as_bytes()).map_err(invalid_name)?;
    // SAFETY: the string is NUL-terminated and outlives the call, and `directory` is an open descriptor.
    let removed: libc::c_int = unsafe { libc::unlinkat(directory.as_raw_fd(), text.as_ptr(), 0) };
    if removed == 0 {
        return Ok(());
    }
    let refusal: std::io::Error = std::io::Error::last_os_error();
    if refusal.kind() == std::io::ErrorKind::NotFound {
        return Ok(());
    }
    return Err(refusal);
}
