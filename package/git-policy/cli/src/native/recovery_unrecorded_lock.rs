//! What:
//!  The release of a real `index.lock` that a dead wrapper's PID file announces
//!  although the record that proves the lock ours (`index-lock-<n>.json`) is gone.
//! Why:
//!  Recovery releases a dead owner's lock only through its journal record,
//!  which names the exact file the owner created.
//!  A power loss that keeps the lock and its PID file but loses the record's name
//!  leaves a lock nothing proves ours,
//!  and every later commit then refuses it as an unproven owner.
//!  The surviving facts bind the lock to the dead wrapper without the record:
//!  the lock is empty,
//!  it was created after the transaction was published and not after the PID file was written,
//!  the PID file names the dead owner,
//!  no process of that number exists in the owner's PID namespace,
//!  and no process whose descriptors can be read holds the lock open.
//!  A lock another process created later is dated after the PID file,
//!  and a process that holds it open is found by the scan,
//!  so each refuses the release.
//!  Callers hold the landing lock,
//!  which keeps every other wrapper from taking the lock meanwhile.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const released = await releaseLockAnnouncedByDeadOwner({ directory, realIndex, ownerPid });
//! ```

/// Warnings and debug lines to standard error.
use super::diagnostic_log::{debug, warn};
/// The PID namespace rule.
use super::pid_namespace::{current_pid_namespace, is_other_namespace};
/// Process liveness by number.
use super::process_start::{ProcessStart, resolve_process_start};
/// Processes that hold a file open.
use super::process_use::{UseScan, UseTarget, processes_using};
/// The lock's path.
use super::real_index_lock::real_lock_path;
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// Attempt records and the release of the locks they prove.
use super::recovery_evidence::{INDEX_LOCK_RECORD_PREFIX, attempt_numbers, release_recorded_locks};
/// The transaction's pack keeps.
use super::pack_keep::remove_transaction_keeps;
/// The PID file's path and optional reads.
use super::recovery_files::{lock_pid_path, read_optional};
/// The preparing record's filename.
use super::transaction_journal::PREPARING_FILENAME;
/// The preparing record, which names the real index.
use super::transaction_journal_parse::parse_preparing;
/// The owner record.
use super::transaction_owner::parse_transaction_owner;
use super::transaction_registry::OWNER_FILENAME;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// File times.
use std::time::SystemTime;

/// What:
///  What became of the lock.
/// Why:
///   The caller carries on with the rest of the recovery whichever it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnrecordedLockRelease = 'released' | 'absent' | { kept: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnrecordedLockRelease {
    /// The lock was proven the dead owner's and removed.
    Released,
    /// No lock exists.
    Absent,
    /// The lock stays; the text names the proof that is missing.
    Kept(String),
}

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
///  The modification time of a path, without following links.
/// Why:
///   The order of three times is one of the proofs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await lstat(path)).mtime
/// ```
fn modified(path: &Path) -> Result<SystemTime, RecoveryError> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).map_err(|error| { return io_failure("inspecting", path, &error); })?;
    return metadata.modified().map_err(|error| { return io_failure("reading the change time of", path, &error); });
}

/// What:
///  The reason the owner's process number cannot be trusted in this PID namespace,
///  or nothing when it can.
/// Why:
///   The owner record is read again because the caller holds only the number,
///  and a number from another namespace names nothing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownerRecord.ownerPid === ownerPid && !isOtherNamespace(ownerRecord.ownerPidNamespace, currentPidNamespace())
/// ```
fn number_is_not_trustworthy(directory: &Path, owner_pid: i64) -> Option<String> {
    let owner_path: PathBuf = directory.join(OWNER_FILENAME);
    let Ok(Some(bytes)) = read_optional(owner_path.as_path()) else {
        return Some(String::from("the owner record is not readable, so the namespace of the owner is unknown"));
    };
    let Some(record) = parse_transaction_owner(bytes.as_slice()) else {
        return Some(String::from("the owner record is malformed, so the namespace of the owner is unknown"));
    };
    if record.owner_pid != owner_pid {
        return Some(String::from("the owner record names another process"));
    }
    if is_other_namespace(record.owner_pid_namespace.as_deref(), current_pid_namespace().as_deref()) {
        return Some(String::from("the owner ran in another PID namespace, where its number names nothing here"));
    }
    return None;
}

/// What:
///  The device and inode of a file.
/// Why:
///   The scan matches an open descriptor by them, and the removal checks that the file is still the one examined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [stat.dev, stat.ino]
/// ```
#[cfg(unix)]
fn identity_of(metadata: &std::fs::Metadata) -> (u64, u64) {
    /// The Unix metadata accessors, for the device and inode.
    use std::os::unix::fs::MetadataExt;
    return (metadata.dev(), metadata.ino());
}

/// What:
///  Without device and inode numbers every file has the same identity, and the scan cannot read a process table.
/// Why:
///   The scan then reports that it could not read, so the lock is kept; the crate does not build for Windows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [0, 0]
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn identity_of(_metadata: &std::fs::Metadata) -> (u64, u64) {
    return (0, 0);
}

/// What:
///  The first proof that fails for a lock, or nothing when every proof holds.
/// Why:
///   Every refusal names the proof that is missing, so the warning says why the lock stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// firstMissingProof({ directory, lock, pidFile, ownerPid })
/// ```
fn missing_proof(
    directory: &Path,
    lock: &std::fs::Metadata,
    real_index: &Path,
    owner_pid: i64,
    probe: fn(i64) -> ProcessStart,
    scan: fn(&UseTarget<'_>) -> UseScan,
) -> Result<Option<String>, RecoveryError> {
    if !lock.is_file() {
        return Ok(Some(String::from("it is not a regular file")));
    }
    if lock.len() > 0 {
        return Ok(Some(String::from("it holds bytes, so something wrote to it")));
    }
    if !announces_owner(real_index, owner_pid) {
        return Ok(Some(String::from("no PID file names the dead owner")));
    }
    if let Some(reason) = number_is_not_trustworthy(directory, owner_pid) {
        return Ok(Some(reason));
    }
    if probe(owner_pid) != ProcessStart::Missing {
        return Ok(Some(format!("a process numbered {owner_pid} exists, so the number may have been reused")));
    }
    let created: SystemTime = lock.modified().map_err(|error| {
        return io_failure("reading the change time of", real_lock_path(real_index).as_path(), &error);
    })?;
    let announced: SystemTime = modified(lock_pid_path(real_index).as_path())?;
    let published: SystemTime = modified(directory.join(OWNER_FILENAME).as_path())?;
    if created > announced {
        return Ok(Some(String::from("it was created after the PID file was written, so another process took it")));
    }
    if created < published {
        return Ok(Some(String::from("it was created before the transaction was published")));
    }
    let (device, inode): (u64, u64) = identity_of(lock);
    let found: UseScan = scan(&UseTarget::File { device, inode });
    if let Some(user) = found.users.first() {
        return Ok(Some(format!("process {} holds it open", user.pid)));
    }
    // A process of this account whose descriptors cannot be read (`systemd --user` is one: it holds capabilities, which
    // makes it non-dumpable) does not keep the lock: the proofs above show who created it, and neither Git nor a
    // wrapper hands a lock to another process, so the scan only adds a veto for a holder it can see.
    if !found.unreadable.is_empty() {
        debug(
            "releaseAnnouncedLock",
            format!("{} processes of this account could not be scanned: {}", found.unreadable.len(), found.unreadable[0]).as_str(),
        );
    }
    return Ok(None);
}

/// What:
///  Release the real `index.lock` when the proofs hold,
///  with the process probe and the open-file scan passed in.
/// Why:
///   Controls give the lock a live holder, a reused number or a stale PID file without waiting for real ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await releaseLockAnnouncedByDeadOwnerWith({ directory, realIndex, ownerPid, probe, scan });
/// ```
pub fn release_announced_lock_with(
    directory: &Path,
    real_index: &Path,
    owner_pid: i64,
    probe: fn(i64) -> ProcessStart,
    scan: fn(&UseTarget<'_>) -> UseScan,
) -> Result<UnrecordedLockRelease, RecoveryError> {
    let lock: PathBuf = real_lock_path(real_index);
    let examined: std::fs::Metadata = match std::fs::symlink_metadata(lock.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(UnrecordedLockRelease::Absent),
        Err(error) => return Err(io_failure("inspecting", lock.as_path(), &error)),
    };
    if let Some(reason) = missing_proof(directory, &examined, real_index, owner_pid, probe, scan)? {
        if announces_owner(real_index, owner_pid) {
            warn(
                "releaseAnnouncedLock",
                format!(
                    "cli-git left {} in place although the PID file names the dead owner {owner_pid}: {reason}",
                    lock.display()
                )
                .as_str(),
            );
        }
        return Ok(UnrecordedLockRelease::Kept(reason));
    }
    // The identity is read again at the removal, so a lock replaced since the proofs is left alone.
    match std::fs::symlink_metadata(lock.as_path()) {
        Ok(now) if identity_of(&now) == identity_of(&examined) => {}
        Ok(_) => return Ok(UnrecordedLockRelease::Kept(String::from("it was replaced while the proofs were read"))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(UnrecordedLockRelease::Absent),
        Err(error) => return Err(io_failure("inspecting", lock.as_path(), &error)),
    }
    if let Err(error) = std::fs::remove_file(lock.as_path()) {
        if error.kind() == std::io::ErrorKind::NotFound {
            return Ok(UnrecordedLockRelease::Absent);
        }
        return Err(io_failure("removing the index lock the dead owner announced", lock.as_path(), &error));
    }
    debug(
        "releaseAnnouncedLock",
        format!("released {} announced by dead owner {owner_pid}", lock.display()).as_str(),
    );
    return Ok(UnrecordedLockRelease::Released);
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

/// What:
///  Release the real `index.lock` a dead owner's PID file announces when the proofs hold,
///  by the host's process table.
/// Why:
///   The production release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await releaseLockAnnouncedByDeadOwner({ directory, realIndex, ownerPid })
/// ```
pub fn release_announced_lock(directory: &Path, real_index: &Path, owner_pid: i64) -> Result<UnrecordedLockRelease, RecoveryError> {
    return release_announced_lock_with(directory, real_index, owner_pid, resolve_process_start, processes_using);
}

/// Unrecorded-lock controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_unrecorded_lock_tests.rs"]
mod tests;
