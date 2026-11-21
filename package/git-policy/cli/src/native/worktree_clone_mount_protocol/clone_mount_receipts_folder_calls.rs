//! What:
//!  The descriptor-relative calls the receipts folder makes:
//!  opening the directory and a receipt,
//!  creating the staging sibling,
//!  renaming it over the receipt,
//!  and syncing the directory.
//! Why:
//!  Every call names the directory's own descriptor,
//!  so a path replaced after the directory was proven cannot steer a read or
//!  a write
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  The staging write is exclusive and follows no link,
//!  and the rename is one step in one directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const file = openat(directory, name, O_RDONLY | O_NOFOLLOW);
//! ```

use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::path::Path;

/// What:
///  A path as the NUL-terminated text `open` takes.
/// Why:
///   A path with an interior NUL cannot be opened,
///  and the refusal says so rather than truncating it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = cStringOf(path); // throws on an interior NUL
/// ```
pub(super) fn c_string_of(path: &Path) -> Result<CString, String> {
    return CString::new(path.as_os_str().as_encoded_bytes()).map_err(|_nul| return format!("the path {} holds a NUL byte", path.display()));
}

/// What:
///  Create one staging file in the directory,
///  never over an existing name and never through a link.
/// Why:
///   The staging sibling has one writer at a time,
///  and anything already there is another run's file rather than this run's to
///  truncate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openat(directory, staging, O_CREAT | O_EXCL | O_WRONLY | O_NOFOLLOW, 0644);
/// ```
pub(super) fn create_at(directory: &std::fs::File, name: &str, mode: u32) -> std::io::Result<std::fs::File> {
    let text: CString = CString::new(name).map_err(|_nul| return std::io::Error::from_raw_os_error(libc::EINVAL))?;
    // SAFETY: `text` is a NUL-terminated string that outlives the call, and `directory` is an open descriptor.
    let descriptor: libc::c_int = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            text.as_ptr(),
            libc::O_CREAT | libc::O_EXCL | libc::O_WRONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            mode as libc::mode_t,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// What:
///  Open one receipt in the directory,
///  following no link and naming no folder.
/// Why:
///   A link named after a GUID is another object's file,
///  and the caller must find the receipt or no file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openat(directory, guid, O_RDONLY | O_NOFOLLOW); // null when absent
/// ```
pub(super) fn open_receipt_at(directory: &std::fs::File, name: &str) -> std::io::Result<Option<std::fs::File>> {
    let Ok(text) = CString::new(name) else {
        return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
    };
    // SAFETY: `text` is a NUL-terminated string that outlives the call, and `directory` is an open descriptor.
    let descriptor: libc::c_int =
        unsafe { libc::openat(directory.as_raw_fd(), text.as_ptr(), libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
    if descriptor < 0 {
        let error: std::io::Error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(error);
    }
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(Some(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) })));
}

/// What:
///  Rename the staging sibling over the receipt's own name,
///  in the same directory.
/// Why:
///   Both names live in one directory,
///  so the rename is one step and a reader sees one whole receipt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await renameat(directory, staging, directory, guid);
/// ```
pub(super) fn rename_at(directory: &std::fs::File, from: &str, to: &str) -> std::io::Result<()> {
    let old: CString = CString::new(from).map_err(|_nul| return std::io::Error::from_raw_os_error(libc::EINVAL))?;
    let new: CString = CString::new(to).map_err(|_nul| return std::io::Error::from_raw_os_error(libc::EINVAL))?;
    // SAFETY: both strings are NUL-terminated and outlive the call, and `directory` is an open descriptor.
    let renamed: libc::c_int = unsafe { libc::renameat(directory.as_raw_fd(), old.as_ptr(), directory.as_raw_fd(), new.as_ptr()) };
    if renamed != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// What:
///  Sync the directory so a rename survives a crash.
/// Why:
///   The receipt's whole point is to outlive the process that filed it,
///  and a rename is durable only after its directory is synced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await fsync(directory);
/// ```
pub(super) fn sync_directory(directory: &std::fs::File) -> std::io::Result<()> {
    return match rustix::fs::fsync(directory) {
        Ok(()) => Ok(()),
        Err(error) => Err(std::io::Error::from_raw_os_error(error.raw_os_error())),
    };
}
