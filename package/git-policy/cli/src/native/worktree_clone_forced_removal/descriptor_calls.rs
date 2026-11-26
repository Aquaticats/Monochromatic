//! What:
//!  The descriptor-relative calls the act and the progress file make:
//!  opening a directory,
//!  opening and creating one entry below it,
//!  making and removing one directory,
//!  renaming one entry over another,
//!  and syncing a directory.
//! Why:
//!  Every name these calls take is resolved against a descriptor the caller
//!  proved,
//!  so a path replaced after the proof cannot steer a write,
//!  and the two callers,
//!  the act on a holder's namespace and the progress file beside the receipt,
//!  spell the calls one way
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "The act's liveness",
//!  and the receipts folder's own rule).
//!  The create is exclusive and follows no link,
//!  the rename is one step in one directory,
//!  and no call takes a path that a caller could have swapped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = openDirectory(path); const entry = openBelow(folder, name); createAt(folder, name, 0o600);
//! ```

use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// What:
///  A name as the NUL-terminated text the calls take.
/// Why:
///  A name with an interior NUL cannot be opened,
///  and the refusal says so rather than cutting it short.
fn name_of(name: &str) -> Result<CString, std::io::Error> {
    return CString::new(name).map_err(|_nul| return std::io::Error::from_raw_os_error(libc::EINVAL));
}

/// What:
///  Open one directory,
///  readable and following no link.
/// Why:
///  Every later call is relative to this descriptor,
///  and a link where the directory should be is another program's.
///
/// # Errors
/// The path could not be opened,
/// or it is not a directory.
pub fn open_directory(path: &Path) -> Result<std::fs::File, std::io::Error> {
    let text: CString = CString::new(path.as_os_str().as_bytes()).map_err(|_nul| return std::io::Error::from_raw_os_error(libc::EINVAL))?;
    // SAFETY: `text` is one NUL-terminated path that outlives the call, and the descriptor it answers is owned exactly
    // once, by the `File` returned.
    let descriptor: RawFd = unsafe { libc::open(text.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: the call answered a descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// What:
///  Open one directory below another,
///  following no link.
/// Why:
///  Way 7's directory is this act's own,
///  and a link in its place belongs to another program.
///
/// # Errors
/// The entry could not be opened,
/// or it is not a directory.
pub fn open_directory_below(directory: &std::fs::File, name: &str) -> Result<std::fs::File, std::io::Error> {
    let text: CString = name_of(name)?;
    // SAFETY: `text` is one NUL-terminated name that outlives the call, and `directory` holds a live descriptor.
    let descriptor: RawFd = unsafe { libc::openat(directory.as_raw_fd(), text.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: the call answered a descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// What:
///  Open one plain file below a directory to read it,
///  following no link and without waiting for a writer.
/// Why:
///  A file a person or another program controls may be a named pipe,
///  and an open of one without `O_NONBLOCK` waits for a writer with no end,
///  so this open cannot wait and the reader's own bound applies to the bytes.
///  A caller that needs a regular file checks that through this descriptor,
///  never through a path read again.
///
/// # Errors
/// The entry could not be opened,
/// which a link in its place answers with `ELOOP`.
pub fn open_file_below(directory: &std::fs::File, name: &str) -> Result<std::fs::File, std::io::Error> {
    let text: CString = name_of(name)?;
    // SAFETY: `text` is one NUL-terminated name that outlives the call, and `directory` holds a live descriptor.
    let descriptor: RawFd = unsafe { libc::openat(directory.as_raw_fd(), text.as_ptr(), libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC) };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: the call answered a descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// What:
///  Create one entry below a directory,
///  never over an existing name and never through a link.
/// Why:
///  A staging name has one writer at a time,
///  and anything already there is another run's file rather than this run's
///  to truncate.
///
/// # Errors
/// The entry exists,
/// or the call refused.
pub fn create_at(directory: &std::fs::File, name: &str, mode: u32) -> Result<std::fs::File, std::io::Error> {
    let text: CString = name_of(name)?;
    // SAFETY: `text` is one NUL-terminated name that outlives the call, and `directory` holds a live descriptor.
    let descriptor: RawFd = unsafe {
        libc::openat(directory.as_raw_fd(), text.as_ptr(), libc::O_CREAT | libc::O_EXCL | libc::O_WRONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC, mode as libc::mode_t)
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: the call answered a descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) }));
}

/// What:
///  Make one directory below another,
///  with the mode the caller names.
/// Why:
///  Way 7 needs one directory of its own at the parent mount's root,
///  mode `0700`,
///  and it is made before the move so the move has a destination.
///
/// # Errors
/// The entry exists,
/// or the call refused.
pub fn make_directory_at(directory: &std::fs::File, name: &str, mode: u32) -> Result<(), std::io::Error> {
    let text: CString = name_of(name)?;
    // SAFETY: `text` is one NUL-terminated name that outlives the call, and `directory` holds a live descriptor.
    let made: libc::c_int = unsafe { libc::mkdirat(directory.as_raw_fd(), text.as_ptr(), mode as libc::mode_t) };
    if made != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// What:
///  Remove one plain entry below a directory,
///  following no link.
/// Why:
///  A writer that failed after it made its staging sibling removes it,
///  so its own next attempt is never blocked by its failure.
///
/// # Errors
/// The entry is absent,
/// or the call refused.
pub fn remove_entry_at(directory: &std::fs::File, name: &str) -> Result<(), std::io::Error> {
    let text: CString = name_of(name)?;
    // SAFETY: `text` is one NUL-terminated name that outlives the call, and `directory` holds a live descriptor.
    let removed: libc::c_int = unsafe { libc::unlinkat(directory.as_raw_fd(), text.as_ptr(), 0) };
    if removed != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// What:
///  Remove one directory below another,
///  which must be empty.
/// Why:
///  The design leaves nothing in the checkout after the act,
///  and the kernel refuses a directory something else filled.
///
/// # Errors
/// The directory is absent or not empty,
/// or the call refused.
pub fn remove_directory_at(directory: &std::fs::File, name: &str) -> Result<(), std::io::Error> {
    let text: CString = name_of(name)?;
    // SAFETY: `text` is one NUL-terminated name that outlives the call, and `directory` holds a live descriptor.
    let removed: libc::c_int = unsafe { libc::unlinkat(directory.as_raw_fd(), text.as_ptr(), libc::AT_REMOVEDIR) };
    if removed != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// What:
///  Rename one entry over another,
///  in the same directory.
/// Why:
///  Both names live in one directory,
///  so the rename is one step and a reader sees one whole file.
///
/// # Errors
/// The call refused.
pub fn rename_at(directory: &std::fs::File, from: &str, to: &str) -> Result<(), std::io::Error> {
    let old: CString = name_of(from)?;
    let new: CString = name_of(to)?;
    // SAFETY: both names are NUL-terminated, outlive the call, and name entries of the same live descriptor.
    let renamed: libc::c_int = unsafe { libc::renameat(directory.as_raw_fd(), old.as_ptr(), directory.as_raw_fd(), new.as_ptr()) };
    if renamed != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// What:
///  Sync a directory so a rename inside it survives a crash.
/// Why:
///  A file whose whole point is to outlive the process that wrote it is
///  durable only after its directory is synced.
///
/// # Errors
/// The call refused.
pub fn sync_directory(directory: &std::fs::File) -> Result<(), std::io::Error> {
    return match rustix::fs::fsync(directory) {
        Ok(()) => Ok(()),
        Err(error) => Err(std::io::Error::from_raw_os_error(error.raw_os_error())),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "descriptor_calls_tests.rs"]
mod tests;
