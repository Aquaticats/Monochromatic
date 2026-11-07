//! What:
//!  The rename that never replaces an existing name,
//!  and the test for a filesystem that cannot do it.
//! Why:
//!  A lock is placed and an earlier name is moved to its current name by one step that is as exclusive as `create_new`:
//!  when the target exists the call fails and both names stay,
//! so no data is written over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await renameNoReplace(from, to); // no Node API; the kernel call is renameat2(RENAME_NOREPLACE)
//! ```

use std::path::Path;

/// Renames `from` to `to` unless `to` exists:
/// one step that is as exclusive as `create_new` and leaves no second name.
#[cfg(target_os = "linux")]
pub fn rename_noreplace(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let source = std::ffi::CString::new(from.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    let target = std::ffi::CString::new(to.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    // SAFETY: both strings are valid NUL-terminated paths for the call.
    let done: i32 = unsafe { libc::renameat2(libc::AT_FDCWD, source.as_ptr(), libc::AT_FDCWD, target.as_ptr(), libc::RENAME_NOREPLACE) };
    return if done == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
}

#[cfg(not(target_os = "linux"))]
pub fn rename_noreplace(_from: &Path, _to: &Path) -> std::io::Result<()> {
    return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
}

/// Whether a failed rename without replacing means the filesystem cannot do one (renameat2(2):
/// `EINVAL`),
/// not that the lock exists or storage failed.
pub fn rename_unsupported(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(unix)]
    {
        return matches!(error.raw_os_error(), Some(code) if code == libc::EINVAL || code == libc::ENOSYS || code == libc::EOPNOTSUPP);
    }
    #[cfg(not(unix))]
    {
        return false;
    }
}
