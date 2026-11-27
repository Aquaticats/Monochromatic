//! What:
//!  The calls that make one file in a directory this module holds open:
//!  caught before they can wait,
//!  an unnamed file where the filesystem has one,
//!  and a file under a name this copy chose otherwise.
//! Why:
//!  The open directory is what keeps a path swapped between two calls from sending the bytes elsewhere,
//! and the error codes below are the only answers that fall back to a named file;
//! any other refusal is reported as it is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const directory = openDirectory(parent);
//! const file = openUnnamed(directory, mode) ?? createNamed(directory, temporary, mode);
//! ```

use crate::worktree_copy_error::{
    WorktreeCopyError,
    failure_with_cause,
};
use std::ffi::{
    CString,
    OsStr,
};
use std::os::fd::{
    AsRawFd,
    FromRawFd,
    OwnedFd,
};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// What:
///  The operating-system errors that mean the filesystem cannot hold an unnamed file.
/// Why:
///   Only those refusals fall back to a temporary that has a name;
/// a full disk or a missing directory does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNNAMED_UNSUPPORTED_CODES = new Set(['EOPNOTSUPP', 'ENOSYS', 'EINVAL']);
/// ```
pub(super) const UNNAMED_UNSUPPORTED_CODES: &[i32] =
    &[libc::EOPNOTSUPP, libc::ENOSYS, libc::EINVAL];

/// What:
///  The operating-system errors that mean the filesystem cannot publish a temporary
///  by a rename that does not replace.
/// Why:
///   Only those refusals fall back to a hard link;
/// a name already taken is a refusal of the publication,
/// not of the filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RENAME_UNSUPPORTED_CODES = new Set(['EINVAL', 'ENOSYS', 'EOPNOTSUPP']);
/// ```
pub(super) const RENAME_UNSUPPORTED_CODES: &[i32] = &[libc::EINVAL, libc::ENOSYS, libc::EOPNOTSUPP];

/// What:
///  Whether a failure means the filesystem has no such operation.
/// Why:
///   The fallbacks exist for that one answer,
/// and a refusal for any other reason is reported as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unsupported = (error, codes) => codes.includes(error.errno);
/// ```
pub(super) fn is_unsupported(
    error: &std::io::Error,
    codes: &[i32],
) -> bool {
    return match error.raw_os_error() {
        Some(code) => codes.contains(&code),
        None => false,
    };
}

/// What:
///  The error a name this release cannot hand the kernel stands for.
/// Why:
///   Every name and path here becomes NUL-terminated text,
/// and a NUL byte inside one is not a name the kernel can take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const invalidName = () => Object.assign(new Error('invalid'), { code: 'EINVAL' });
/// ```
pub(super) fn invalid_name<T>(_refused: T) -> std::io::Error {
    return std::io::Error::from_raw_os_error(libc::EINVAL);
}

/// What:
///  The directory a name below it is created in,
/// held open for the calls that create it.
/// Why:
///   Every call below names one component of that directory,
/// so a path a rename swapped between two calls cannot send the
///  bytes somewhere else.
/// The open asks for a directory,
/// so a name that is no directory by now (a named pipe a dev server
///  left in the destination's place)
/// is refused at once instead of waiting for a writer that may never come
///  ("nothing should ever wait forever or indefinitely").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = await open(parent, O_DIRECTORY | O_CLOEXEC);
/// ```
pub(super) fn open_directory(path: &Path) -> Result<std::fs::File, WorktreeCopyError> {
    use std::os::unix::fs::OpenOptionsExt;
    let opened: std::io::Result<std::fs::File> = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC)
        .open(path);
    // `match` on the open names the directory the caller could not use.
    return match opened {
        Ok(directory) => Ok(directory),
        Err(error) => Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: the directory {} holding an ignored-state entry could not be opened.",
                crate::json_record::quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            Some(&error),
        )),
    };
}

/// What:
///  The final component of a path,
/// which is the one name every call here takes.
/// Why:
///   The directory is held open,
/// so only the last component is resolved,
/// and a path with no component names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lastName = (path) => path.basename(path);
/// ```
pub(super) fn last_name(path: &Path) -> Result<&OsStr, WorktreeCopyError> {
    return match path.file_name() {
        Some(name) => Ok(name),
        None => Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: {} names no entry to install.",
                crate::json_record::quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        )),
    };
}

/// What:
///  A file with no name in `directory`,
/// created with `mode`,
/// or the refusal the filesystem gave.
/// Why:
///   This is the preferred home of a file being filled:
/// nothing can read it before its bytes and mode are whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const descriptor = openat(dir, '.', O_TMPFILE | O_WRONLY, mode);
/// ```
pub(super) fn open_unnamed(
    directory: &std::fs::File,
    mode: u32,
) -> std::io::Result<std::fs::File> {
    let dot: CString = match CString::new(".") {
        Ok(text) => text,
        Err(_nul) => return Err(std::io::Error::from_raw_os_error(libc::EINVAL)),
    };
    // SAFETY: `dot` is a NUL-terminated string that outlives the call, and `directory` is an open descriptor.
    let descriptor: libc::c_int = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            dot.as_ptr(),
            libc::O_TMPFILE | libc::O_WRONLY | libc::O_CLOEXEC,
            mode as libc::mode_t,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe {
        OwnedFd::from_raw_fd(descriptor)
    }));
}

/// What:
///  Create a file under `name` in `directory`,
/// failing when anything is there,
/// a link included.
/// Why:
///   The fallback's temporary is created exclusively,
/// so it never takes a name a person's file already has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const descriptor = openat(dir, name, O_CREAT | O_EXCL | O_WRONLY, mode);
/// ```
pub(super) fn create_named(
    directory: &std::fs::File,
    name: &OsStr,
    mode: u32,
) -> std::io::Result<std::fs::File> {
    let text: CString = CString::new(name.as_bytes()).map_err(invalid_name)?;
    // SAFETY: `text` is a NUL-terminated string that outlives the call, and `directory` is an open descriptor.
    let descriptor: libc::c_int = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            text.as_ptr(),
            libc::O_CREAT | libc::O_EXCL | libc::O_WRONLY | libc::O_CLOEXEC,
            mode as libc::mode_t,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new descriptor that nothing else owns.
    return Ok(std::fs::File::from(unsafe {
        OwnedFd::from_raw_fd(descriptor)
    }));
}
