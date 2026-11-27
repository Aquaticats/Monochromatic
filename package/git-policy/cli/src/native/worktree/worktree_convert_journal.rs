//! What:
//!  The claim one command at a time takes on a conversion of a worktree,
//!  and the kernel lock an older release left in the administrative directory.
//! Why:
//!  The journal names the identity of the old root and of the subvolume it created,
//!  which is what every later phase checks before it reuses,
//!  exchanges,
//!  destroys or deletes anything,
//!  and one command at a time owns a conversion through a numbered claim file
//!  (`single_owner_claim.rs`) taken before the journal is read,
//!  so no kernel lock is held.
//!  Releases up to 0.5.20 serialized conversions with an advisory kernel lock on a file they left in place,
//!  and an older command may still be converting the worktree,
//!  so this release probes for that lock without waiting and removes the file when nobody holds it.
//!  This file is the module's index:
//!  the journal's shape and its text are in `worktree_convert_journal_text.rs`,
//!  writing and reading the file are in `worktree_convert_journal_writes.rs`,
//!  and both were split out because this module was over the line limit,
//!  with every name they defined re-exported here so no caller changes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = await takeClaim({ directory: admin, prefix: CLAIM_PREFIX }); // busy when another command owns it
//! const heldByOlderRelease = await legacyLockIsHeld(admin); // false when absent, free or not a plain file
//! ```

use crate::subvolume_error::MigrateError;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{
    Path,
    PathBuf,
};

/// The journal's own name,
/// its shape and its text,
/// and the clock reading it records.
pub use crate::worktree_convert_journal_text::{
    JOURNAL_NAME,
    Journal,
    Phase,
    is_recorded_object,
    now_milliseconds,
};
/// The writing and the reading of the journal file,
/// and the sweep of a temporary file a killed write left.
pub use crate::worktree_convert_journal_writes::{
    TEMPORARY_PREFIX,
    io_step,
    sweep_temporary_files,
};

/// The name of the kernel lock file that releases up to 0.5.20 left in the administrative directory;
/// this release only probes for
///  it and removes it when nothing holds it.
pub const LOCK_NAME: &str = "cli-git-convert.lock";

/// The prefix of the numbered claim files that give one command at a time the conversion of a worktree.
pub const CLAIM_PREFIX: &str = "cli-git-convert.owner";

/// What:
///  Whether a command of an older release still holds the conversion's kernel lock on `admin`,
/// retiring the lock file when
///  none does.
/// Why:
///   Releases up to 0.5.20 serialized conversions with an advisory kernel lock on `cli-git-convert.lock` and left the file
///  in place.
/// This release serializes with a claim file instead and takes no kernel lock,
/// but an older command may still be
///  converting the worktree,
/// and two commands must never convert one worktree at once.
/// The probe asks without waiting and
///  gives the lock straight back;
/// a file nobody holds is removed,
/// so the leftover disappears after the upgrade.
/// A file that
///  is absent costs nothing,
/// and a file that is not a plain file is left where it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const heldByOlderRelease = await retireLegacyLock(admin); // false when absent, free (and now removed) or not a plain file
/// ```
pub fn legacy_lock_is_held(admin: &Path) -> Result<bool, MigrateError> {
    let path: PathBuf = admin.join(LOCK_NAME);
    let file: std::fs::File = match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&path)
    {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(MigrateError::io(
                "open the conversion lock an older release left",
                path.as_path(),
                &error,
            ));
        }
    };
    let plain: bool = io_step(
        file.metadata(),
        "examine the conversion lock an older release left",
        path.as_path(),
    )?
    .file_type()
    .is_file();
    if !plain {
        return Ok(false);
    }
    // SAFETY: `flock` takes an open descriptor, which `file` keeps open for the call.
    let result: libc::c_int =
        unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        let error: std::io::Error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::WouldBlock {
            return Ok(true);
        }
        return Err(MigrateError::io(
            "probe the conversion lock an older release left",
            path.as_path(),
            &error,
        ));
    }
    // Removed while the probe still holds it, so no older command can take the name between the probe and the removal.
    return match std::fs::remove_file(&path) {
        Ok(()) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(MigrateError::io(
            "remove the conversion lock an older release left",
            path.as_path(),
            &error,
        )),
    };
}
