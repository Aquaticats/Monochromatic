//! What:
//!  The two renames that lose nothing:
//! the one that never replaces an existing name,
//! and the one that exchanges two names in a single step;
//!  and the test for a filesystem that cannot do either.
//! Why:
//!  A lock is placed and an earlier name is moved to its current name by one step that is as exclusive as `create_new`:
//!  when the target exists the call fails and both names stay,
//! so no data is written over.
//!  A file a caller read a moment ago is replaced without a window by the exchange:
//!  each name holds afterwards what the other held,
//! so whatever stood at the target is handed back instead of being overwritten unseen
//!  (`package_repair_cargo_lock.rs`,
//!  `replace_the_clean_s_own`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await renameNoReplace(from, to); // no Node API; the kernel call is renameat2(RENAME_NOREPLACE)
//! await exchange(from, to); // no Node API; the kernel call is renameat2(RENAME_EXCHANGE)
//! const moved = await renameNoReplaceOnTheBlockingPool(from, to); // the same rename, off a worker of the runtime
//! ```

/// The pool a rename of a user's file goes to,
/// so the directory write it makes never holds a worker of the runtime.
use crate::blocking_job::BlockingJobError;
/// One job of the blocking pool,
/// under the process's own bound on them.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// A borrowed `Path`,
/// and an owned `PathBuf` for the job that outlives the caller's borrow.
use std::path::{
    Path,
    PathBuf,
};

/// Renames `from` to `to` unless `to` exists:
/// one step that is as exclusive as `create_new` and leaves no second name.
#[cfg(target_os = "linux")]
pub fn rename_noreplace(
    from: &Path,
    to: &Path,
) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let source =
        std::ffi::CString::new(from.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    let target =
        std::ffi::CString::new(to.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    // SAFETY: both strings are valid NUL-terminated paths for the call.
    let done: i32 = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            target.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    return if done == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    };
}

#[cfg(not(target_os = "linux"))]
pub fn rename_noreplace(
    _from: &Path,
    _to: &Path,
) -> std::io::Result<()> {
    return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
}

/// Renames `from` to `to` and `to` to `from` in one step:
/// afterwards each name holds what the other held,
/// and neither file is ever without a name.
/// Why:
///   A caller that read the target a moment ago cannot check and then rename:
/// a write that lands between the two would be replaced without a copy,
/// and a comparison and a rename are two steps whatever order they are in.
/// The exchange is the one step that answers what it displaced,
/// so the caller judges the file it found instead of trusting the reading
/// (`package_repair_cargo_lock.rs`,
/// `replace_the_clean_s_own`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await exchange(from, to); // no Node API; the kernel call is renameat2(RENAME_EXCHANGE)
/// ```
#[cfg(target_os = "linux")]
pub fn exchange(
    from: &Path,
    to: &Path,
) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let source =
        std::ffi::CString::new(from.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    let target =
        std::ffi::CString::new(to.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    // SAFETY: both strings are valid NUL-terminated paths for the call.
    let done: i32 = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            target.as_ptr(),
            libc::RENAME_EXCHANGE,
        )
    };
    return if done == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    };
}

#[cfg(not(target_os = "linux"))]
pub fn exchange(
    _from: &Path,
    _to: &Path,
) -> std::io::Result<()> {
    return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
}

/// What:
///  What one rename that never replaces a name came to,
///  made as a job of the blocking pool.
/// Why:
///   A caller has to tell four outcomes apart:
/// the name is at the entry now,
/// another write took it (which a merge reports rather than fails),
/// the rename failed for another reason,
///  and the system gave the pool no thread,
/// where the rename did not run or may yet,
///  so no caller may read that as a refusal of the work itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheAnswerOfARename =
///   | { kind: 'renamed' }
///   | { kind: 'name-taken' }
///   | { kind: 'failed'; error: Error }
///   | { kind: 'no-thread'; refused: BlockingJobError };
/// ```
pub enum TheAnswerOfARename {
    /// The name was free, and the entry is at it now.
    Renamed,
    /// An entry stood at the name,
    /// so nothing moved and both names stay,
    /// with the error that says so.
    NameTaken(std::io::Error),
    /// The rename failed for another reason.
    Failed(std::io::Error),
    /// The system refused the pool a thread,
    /// so the rename did not run, or the pool ran it after all: the refusal says which.
    NoThreadToRunIt(BlockingJobError),
}

/// What:
///  Renames `from` to `to` without replacing a name,
///  as a job of the blocking pool.
/// Why:
///   A rename writes the directory the name lands in,
/// and that can wait on a disk,
/// a lock or a mount,
/// so a step that runs on a task hands the rename to the pool
///  (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Blocking files,
/// fsync and the runtime's own shutdown"),
/// where it holds no worker of the runtime
///  that would end its wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moved: TheAnswerOfARename = await renameNoReplaceOnTheBlockingPool(from, to);
/// ```
pub async fn rename_noreplace_on_the_blocking_pool(
    from: &Path,
    to: &Path,
) -> TheAnswerOfARename {
    let (source, landing): (PathBuf, PathBuf) = (from.to_path_buf(), to.to_path_buf());
    let moved: Result<std::io::Result<()>, BlockingJobError> =
        run_blocking_job_within_the_budget(move || {
            return rename_noreplace(source.as_path(), landing.as_path());
        })
        .await;
    return match moved {
        Ok(Ok(())) => TheAnswerOfARename::Renamed,
        // The name another write took is not a failure: the caller keeps both entries and names the one it moved aside.
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            TheAnswerOfARename::NameTaken(error)
        }
        Ok(Err(error)) => TheAnswerOfARename::Failed(error),
        Err(refused) => TheAnswerOfARename::NoThreadToRunIt(refused),
    };
}

/// Whether a failed rename without replacing means the filesystem cannot do one (renameat2(2):
/// `EINVAL`),
/// not that the lock exists or storage failed.
/// It is the answer for a refused exchange too (`ENOSYS`, `EOPNOTSUPP`).
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
