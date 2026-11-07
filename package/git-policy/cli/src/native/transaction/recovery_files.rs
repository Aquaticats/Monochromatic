//! What:
//!  The small file operations the shadow,
//! the pack keep files,
//! the conclusion copies and the worktree installs share:
//!  existence probes,
//! an optional read,
//! removal of one file,
//! and the name of Git's PID file beside a lock.
//! Why:
//!  Each fails closed with the path and the operation named (`RecoveryError`),
//! as the incumbent's
//!  `commit-transaction-recovery-files.ts` does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (await recoveryPathExists(path)) await rm(path, { force: true });
//! ```

/// The fail-closed recovery failure.
use crate::recovery_error::{RecoveryError, io_failure};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Whether anything exists at `path`,
///  following links,
///  as Node's `access` does.
///       Any error other than "missing" is reported.
/// Why:
///   The incumbent's `recoveryPathExists`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoveryPathExists(path: string): Promise<boolean>;
/// ```
pub fn recovery_path_exists(path: &Path) -> Result<bool, RecoveryError> {
    match std::fs::metadata(path) {
        Ok(_) => return Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(io_failure("probing", path, &error)),
    }
}

/// What:
///  Whether anything exists at `path` without following a final link.
/// Why:
///   The incumbent's `pathPresent` and `pathExists` (both `lstat`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pathPresent(path: string): Promise<boolean>;
/// ```
pub fn path_present(path: &Path) -> Result<bool, RecoveryError> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => return Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(io_failure("probing", path, &error)),
    }
}

/// What:
///  A file's bytes,
///  following links as Node's `readFile` does,
///  or nothing when missing.
/// Why:
///   Conclusion entries may be absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readOptional(path: string): Promise<Uint8Array | typeof ENTRY_ABSENT>;
/// ```
pub fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, RecoveryError> {
    match std::fs::read(path) {
        Ok(bytes) => return Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_failure("reading", path, &error)),
    }
}

/// What:
///  Remove one file,
///  treating a missing one as removed.
/// Why:
///   `rm(path, { force: true })` for single files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { force: true });
/// ```
pub fn remove_file_if_present(path: &Path) -> Result<(), RecoveryError> {
    match std::fs::remove_file(path) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_failure("removing", path, &error)),
    }
}

/// What:
///  The PID file Git writes beside a real index lock.
/// Why:
///   With `core.lockfilePid=true`,
///  Git writes `index~pid.lock` beside `index.lock`;
///  the
///       incumbent's `lockPidPath` names it this way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lockPidPath('/repo/.git/index') // '/repo/.git/index~pid.lock'
/// ```
pub fn lock_pid_path(real_index: &Path) -> PathBuf {
    // `as_os_str().to_owned()` copies the raw path so the suffix can be appended.
    let mut name: std::ffi::OsString = real_index.as_os_str().to_owned();
    name.push("~pid.lock");
    return PathBuf::from(name);
}

/// Recovery file controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_files_tests.rs"]
pub(crate) mod tests;
