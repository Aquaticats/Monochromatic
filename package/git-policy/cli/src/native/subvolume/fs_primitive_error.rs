//! What:
//!  The typed failure of one Linux filesystem primitive (`statfs`, a btrfs ioctl, `FICLONE`, `renameat2`):
//!  which operation, on which path, and the operating system's error number.
//! Why:
//!  The primitives of the cheap-worktree ladder and the subvolume migration all fall through on failure, never panic;
//!  the caller decides from the error number whether the next rung applies (the filesystem lacks the feature) or the
//!  work must stop (a real fault), and the message names the operation and path a person can act on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class PrimitiveError extends Error { constructor(readonly operation: string, readonly path: string, readonly errno: number) { super(`...`); } }
//! ```

use std::ffi::CString;
use std::fmt;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// What:
///  One failed primitive call.
///  `&'static str` is text compiled into the executable (sibling `String` is owned);
///  `i32` is the C `errno` value (siblings `i64` and `u32` exist; the C API returns a 32-bit signed integer).
/// Why:
///   The operation names the call as the kernel documents it (for example `BTRFS_IOC_SNAP_CREATE_V2`), so a report
///  can be searched in the kernel sources.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrimitiveError = { operation: string; path: string; errno: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimitiveError {
    /// The call that failed, spelled as its kernel or libc name.
    pub operation: &'static str,
    /// The path the call was about.
    pub path: PathBuf,
    /// The `errno` the call left, 0 when the failure is not an operating-system error (a path with a NUL byte).
    pub errno: i32,
}

impl PrimitiveError {
    /// What:
    ///  The error for the call that just failed, reading `errno` from the thread.
    /// Why:
    ///   `errno` is overwritten by the next library call, so it is read before anything else runs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const fail = (operation: string, path: string) => new PrimitiveError(operation, path, lastErrno());
    /// ```
    pub fn last_os_error(operation: &'static str, path: &Path) -> PrimitiveError {
        let errno: i32 = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        return PrimitiveError { operation, path: path.to_path_buf(), errno };
    }

    /// What:
    ///  Whether the filesystem lacks the feature, as opposed to a fault in this call.
    /// Why:
    ///   A ladder rung that gets `true` falls to the next rung; `false` is reported. Not a btrfs filesystem answers an
    ///  ioctl with `ENOTTY`, a filesystem without reflinks answers `FICLONE` with `EOPNOTSUPP` or `EINVAL`, a clone
    ///  across filesystems answers `EXDEV`, and an overlay over a layer without the feature answers `ENOSYS`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const lacksFeature = (e: PrimitiveError) => [ENOTTY, EOPNOTSUPP, EINVAL, EXDEV, ENOSYS].includes(e.errno);
    /// ```
    pub fn means_feature_missing(&self) -> bool {
        return [libc::ENOTTY, libc::EOPNOTSUPP, libc::EINVAL, libc::EXDEV, libc::ENOSYS].contains(&self.errno);
    }
}

/// Prints the call, the path and the system's own words for the error number.
impl fmt::Display for PrimitiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            formatter,
            "{} on {} failed: {} (errno {})",
            self.operation,
            self.path.display(),
            std::io::Error::from_raw_os_error(self.errno),
            self.errno
        );
    }
}

/// A primitive error is an error value.
impl std::error::Error for PrimitiveError {}

/// What:
///  A path as the NUL-terminated C string the system calls take.
/// Why:
///   A path with an interior NUL byte can name no file, so it fails with `EINVAL` before any call, instead of
///  reaching a system call with a truncated name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cPath(operation: string, path: string): Buffer { if (path.includes('\0')) throw new PrimitiveError(operation, path, EINVAL); return Buffer.from(path + '\0'); }
/// ```
pub fn c_path(operation: &'static str, path: &Path) -> Result<CString, PrimitiveError> {
    return match CString::new(path.as_os_str().as_bytes()) {
        Ok(text) => Ok(text),
        Err(_) => Err(PrimitiveError { operation, path: path.to_path_buf(), errno: libc::EINVAL }),
    };
}

/// Primitive error controls stay out of the release executable.
#[cfg(test)]
#[path = "fs_primitive_error_tests.rs"]
mod tests;
