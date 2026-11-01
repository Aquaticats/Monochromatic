//! What:
//!  Where a dead owner's real index lock is found, and the release of what the owner held when the records that prove
//!  the lock ours are partly lost.
//! Why:
//!  Recovery releases a dead owner's lock through its `index-lock-<n>.json` record, and reads the real index path from
//!  `preparing.json`.
//!  A power loss can take the name of either record,
//!  so this module finds the worktree's own index without `preparing.json`,
//!  recognizes the PID file that announces the dead owner,
//!  and releases the locks the surviving records or the PID file prove (`recovery_unrecorded_lock.rs` holds the proofs).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await releaseDeadOwnerLocks({ directory, names, realIndex: realIndexOfTransaction(directory), ownerPid });
//! ```

/// Debug lines to standard error.
use super::diagnostic_log::debug;
/// The transaction's pack keeps.
use super::pack_keep::remove_transaction_keeps;
/// The fail-closed recovery failure.
use super::recovery_error::RecoveryError;
/// Attempt records and the release of the locks they prove.
use super::recovery_evidence::{INDEX_LOCK_RECORD_PREFIX, attempt_numbers, release_recorded_locks};
/// The PID file's path and optional reads.
use super::recovery_files::{lock_pid_path, read_optional};
/// The proofs that release a lock without its record.
use super::recovery_unrecorded_lock::{UnrecordedLockRelease, release_announced_lock};
/// The preparing record's filename.
use super::transaction_journal::PREPARING_FILENAME;
/// The preparing record, which names the real index.
use super::transaction_journal_parse::parse_preparing;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The real index of a transaction directory:
///  the path its `preparing.json` names,
///  otherwise the worktree's own `index` beside the registry.
/// Why:
///   A lost `preparing.json` name leaves no record of the path,
///  and the default is where every commit without `GIT_INDEX_FILE` keeps it.
///  A lock that is not at the default path is not found, so it is not released.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// preparing?.realIndexPath ?? join(dirname(dirname(directory)), 'index')
/// ```
pub fn real_index_of_transaction(directory: &Path) -> PathBuf {
    if let Ok(Some(bytes)) = read_optional(directory.join(PREPARING_FILENAME).as_path())
        && let Ok(preparing) = parse_preparing(bytes.as_slice())
    {
        return PathBuf::from(preparing.real_index_path);
    }
    let worktree_directory: &Path = directory.parent().and_then(Path::parent).unwrap_or(directory);
    return worktree_directory.join("index");
}

/// What:
///  Whether the PID file beside the real index names exactly this owner.
/// Why:
///   Only such a PID file can announce a lock the owner took,
///  and recovery takes the landing lock before it looks at the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readFile(lockPidPath(realIndex), 'utf8').catch(() => '')) === `pid ${ownerPid}\n`
/// ```
pub fn announces_owner(real_index: &Path, owner_pid: i64) -> bool {
    return std::fs::read(lock_pid_path(real_index)).is_ok_and(|text| {
        return owner_pid > 0 && text == format!("pid {owner_pid}\n").as_bytes();
    });
}

/// What:
///  Release what a dead owner that holds no landing record holds:
///  the locks its records prove,
///  or, when it recorded none, the lock its PID file announces if the proofs hold;
///  then its PID file.
/// Why:
///   A lock record that was written and then lost leaves the PID file as the only link to the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await releaseDeadOwnerLocks({ directory, names, realIndex, ownerPid });
/// ```
pub fn release_dead_owner_locks(
    directory: &Path,
    names: &[String],
    real_index: &Path,
    owner_pid: i64,
) -> Result<(), RecoveryError> {
    if attempt_numbers(names, INDEX_LOCK_RECORD_PREFIX).is_empty() {
        let outcome: UnrecordedLockRelease = release_announced_lock(directory, real_index, owner_pid)?;
        debug("releaseDeadOwnerLocks", format!("lock without a record: {outcome:?}").as_str());
    }
    return release_recorded_locks(directory, names, real_index, owner_pid);
}

/// What:
///  Release the locks of a dead owner whose `preparing.json` is gone,
///  and its keeps when it recorded a lock;
///  whether it had progressed past the preparing record.
/// Why:
///   Without the record the real index is the worktree's own `index`
///  and the object directory is the common one,
///  which is where every commit without redirection keeps them;
///  a lock or keep elsewhere is not found, so it is not touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const progressed = await releaseWithoutPreparing({ objectDirectory, directory, names, transactionId, ownerPid });
/// ```
pub fn release_without_preparing(
    object_directory: &Path,
    directory: &Path,
    names: &[String],
    transaction_id: &str,
    owner_pid: i64,
) -> Result<bool, RecoveryError> {
    let real_index: PathBuf = real_index_of_transaction(directory);
    let recorded: bool = !attempt_numbers(names, INDEX_LOCK_RECORD_PREFIX).is_empty();
    if !recorded && !announces_owner(real_index.as_path(), owner_pid) {
        return Ok(false);
    }
    release_dead_owner_locks(directory, names, real_index.as_path(), owner_pid)?;
    if recorded {
        remove_transaction_keeps(object_directory, transaction_id)?;
    }
    return Ok(true);
}

/// Dead-owner lock controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_dead_owner_locks_tests.rs"]
mod tests;
