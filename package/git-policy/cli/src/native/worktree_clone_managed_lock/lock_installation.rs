//! What:
//!  Write the managed Git lock again when it is gone.
//! Why:
//!  The project decided on 2026-11-22 that any wrapped command which holds the slot's claim and has proved the
//!  registration writes a lost lock again and prints one line saying it did
//!  (`doc/decision/cli-git-zfs-add-time-git-lock-put-back-when-lost.md`).
//!  The install is one exclusive write of the sentence recomposed from the record's nonce,
//!  because `O_EXCL` decides the winner when two commands install at once and no lock is held,
//!  and because a staging file would leave the registration unprotected for a moment
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "The lock is lost,
//! the registration is there").
//!  A kill inside the write leaves a prefix,
//!  which is still a lock that still stops a prune and reads as S2 or S3 afterwards,
//!  so nothing is staged,
//! renamed or flushed here:
//!  the record is the durable half and the lock's loss is the state S4,
//! installable again.
//!  A lock that stands is never overwritten,
//!  and nothing here decides whether the command may install:
//!  the caller holds the slot's claim and has proved the registration.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = writeTheLostLock(registration, slot, nonce); // { kind: 'wrote', line } | { kind: 'a-lock-stands' } | { kind: 'refused' }
//! ```

use crate::worktree_clone_binding_record::lock_words::lock_file_bytes;
use crate::worktree_clone_mount_protocol::RequestNonce;
use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::{
    Path,
    PathBuf,
};

/// Git's name for a registration's lock file.
pub const LOCK_FILE_NAME: &str = "locked";

/// The mode Git's own lock write leaves,
/// measured as `0644` (`P1` of the design's probes),
/// and the mode this install writes.
pub const LOCK_FILE_MODE: u32 = 0o644;

/// What:
///  What one install answered.
/// Why:
///  A written lock owes the command one line,
///  a lock that stands owes nothing,
///  and a refused write owes its reason,
///  so the three are not folded into one answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { kind: 'wrote', line } | { kind: 'a-lock-stands' } | { kind: 'refused', reason };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstallAnswer {
    /// The lock is written,
    /// and this is the line the command prints once.
    Wrote {
        /// The sentence the decision requires the command to print.
        line: String,
    },
    /// A lock already stands,
    /// so nothing was written and no line is printed.
    ALockStands,
    /// The write itself was refused.
    Refused {
        /// What the call answered.
        reason: String,
    },
}

/// What:
///  The one line a command prints after it wrote a lost lock again.
/// Why:
///  The design fixes these words,
///  and the line is the only report the command owes,
///  because a kill between the write and the line loses nothing but the report.
pub fn line_of_an_installed_lock(
    slot: &str,
    nonce: &RequestNonce,
) -> String {
    return format!(
        "the clone slot {slot} had lost its managed Git lock, so this command wrote it again for add nonce {}",
        nonce.as_str()
    );
}

/// What:
///  Write one registration's lost lock again,
///  exclusively and without following a link at the name.
/// Why:
///  The bytes are the sentence recomposed from the record's nonce,
///  so no second copy of them exists,
///  and an existing name is a lock another writer won rather than one to replace.
///
/// # Errors
/// The write's own refusal,
/// told apart from a lock that stands by the system's `AlreadyExists`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = open(join(registration, 'locked'), 'wx', 0o644); await file.write(lockFileBytes(nonce));
/// ```
pub fn write_the_lost_lock(
    registration: &Path,
    slot: &str,
    nonce: &RequestNonce,
) -> InstallAnswer {
    let path: PathBuf = registration.join(LOCK_FILE_NAME);
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .mode(LOCK_FILE_MODE)
        .custom_flags(libc::O_NOFOLLOW);
    let mut file: std::fs::File = match options.open(path.as_path()) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return InstallAnswer::ALockStands;
        }
        Err(error) => {
            return InstallAnswer::Refused {
                reason: error.to_string(),
            };
        }
    };
    return match file.write_all(lock_file_bytes(nonce).as_slice()) {
        Ok(()) => InstallAnswer::Wrote {
            line: line_of_an_installed_lock(slot, nonce),
        },
        Err(error) => InstallAnswer::Refused {
            reason: error.to_string(),
        },
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "lock_installation_tests.rs"]
mod tests;
