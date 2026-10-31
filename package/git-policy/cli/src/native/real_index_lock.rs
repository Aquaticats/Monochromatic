//! What:
//!  The real `index.lock` a landing holds,
//!  created exclusively,
//!  identified by device,
//!  inode and
//!       filesystem,
//!  announced through `index~pid.lock`,
//!  and released only while it is still the file
//!       that was created.
//! Why:
//!  The landing's index install depends on the exact lock file it created;
//!  a replaced lock must
//!      never be deleted,
//!  and a lock this process did not create must never be released
//!      (`src/policy-engine/commit-landing-index-lock.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await using lock = await openRealIndexLock({ realIndexPath, timeoutMs });
//! ```

/// The lock identity record the transaction journal stores, and the numbered record's name.
use super::transaction_journal::{IndexLockRecord, LockIdentity, index_lock_record_filename};
/// The record's exact bytes.
use super::transaction_journal_encode::encode_index_lock;
/// Debug and warning text to the diagnostic log.
use super::diagnostic_log::{debug, warn};
/// The recovery install of a recorded post-index through the exact owned lock, and the release of that lock.
use super::recovery_files::{LockRelease, install_recovered_index, lock_release_name, release_owned_lock};
/// Recovery failures, which the install reports.
use super::recovery_error::RecoveryError;
#[cfg(unix)]
use super::transaction_journal::FileIdentity;
/// The waiting coordination and its failure.
use super::index_writer_coordination::{CoordinationFailure, wait_for_lock};
/// The filesystem identity of the worktree's volume.
#[cfg(unix)]
use super::filesystem_identity::filesystem_identity;
/// Announcing this process through the PID file, the journal record, and the lock's mode.
use super::private_storage::{protect_path, sync_directory, write_private_file};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "acquireRealIndexLock";

/// What:
///  A real `index.lock` created by this process.
/// Why:
///   Holding the identity lets the release prove the file is still the one created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RealIndexLock = { lockPath: string; identity: LockIdentity; installIndex(source): Promise<void> };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct RealIndexLock {
    /// The lock path,
    ///  `<real index>.lock`.
    lock_path: PathBuf,
    /// The real index the lock guards,
    ///  which an install replaces.
    real_index: PathBuf,
    /// The PID file path,
    ///  `<real index>~pid.lock`.
    pid_path: PathBuf,
    /// The identity recorded when the lock was created.
    identity: LockIdentity,
}

/// What:
///  The lock path of a real index.
/// Why:
///   Git names the lock beside the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${realIndexPath}.lock`
/// ```
pub fn real_lock_path(real_index: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = real_index.as_os_str().to_owned();
    name.push(".lock");
    return PathBuf::from(name);
}

/// What:
///  The PID file path of a real index.
/// Why:
///   Git's `core.lockfilePid` file lives beside the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lockPidPath(realIndexPath) // `${realIndexPath}~pid.lock`
/// ```
pub fn real_pid_path(real_index: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = real_index.as_os_str().to_owned();
    name.push("~pid.lock");
    return PathBuf::from(name);
}

impl RealIndexLock {
    /// What:
    ///  The identity recorded for the lock.
    /// Why:
    ///   The landing journal stores it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lock.identity
    /// ```
    pub fn identity(&self) -> &LockIdentity {
        return &self.identity;
    }

    /// What:
    ///  Whether the lock path still names the file this process created.
    /// Why:
    ///   A release or install must not act on a replaced lock.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await assertLockIdentity({ lockPath, identity });
    /// ```
    pub fn assert_identity(&self) -> Result<(), String> {
        return same_file_identity(self.lock_path.as_path(), &self.identity);
    }

    /// What:
    ///  Release the lock and this process's PID file,
    ///  refusing a lock that was replaced.
    /// Why:
    ///   A replaced lock belongs to another process and stays in place.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await lock[Symbol.asyncDispose]();
    /// ```
    pub fn release(self) -> Result<(), String> {
        self.assert_identity()?;
        if let Err(error) = std::fs::remove_file(&self.lock_path) {
            return Err(error.to_string());
        }
        return remove_own_pid_file(self.pid_path.as_path());
    }

    /// What:
    ///  Install a computed index through the held lock,
    ///  consuming the lock,
    ///  then remove this process's PID file.
    /// Why:
    ///   `installIndex`:
    ///  the bytes are written into the lock this process created,
    ///  the lock is linked under a private name beside the post-index,
    ///  and that name is renamed over the real index,
    ///  so the new index keeps the lock's owner.
    ///  The lock is not released here on failure:
    ///  once the landing record is durable,
    ///  recovery completes the install through the same lock.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await lock.installIndex(postIndexPath);
    /// ```
    pub fn install(self, post_index: &Path) -> Result<(), RecoveryError> {
        install_recovered_index(
            self.lock_path.as_path(),
            self.real_index.as_path(),
            post_index,
            &self.identity,
        )?;
        debug(TAG, format!("installed {} through {}", self.real_index.display(), self.lock_path.display()).as_str());
        return remove_own_pid_file(self.pid_path.as_path()).map_err(RecoveryError);
    }

    /// What:
    ///  Release a lock a landing leaves without installing:
    ///  the lock is removed only while it is still the file this process created,
    ///  and the PID file goes either way.
    /// Why:
    ///   The incumbent's disposal warns when the lock is gone or another owner's now
    ///  and still removes its own PID file,
    ///  and throws only for an operating-system error.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await lock[Symbol.asyncDispose]();
    /// ```
    pub fn release_after_landing(self) -> Result<(), String> {
        match release_owned_lock(&self.identity, self.lock_path.as_path()) {
            Ok(LockRelease::Released) => {}
            Ok(other) => warn(
                TAG,
                format!("index lock was not this transaction's at release ({}): {}", lock_release_name(other), self.lock_path.display()).as_str(),
            ),
            Err(error) => return Err(error.0),
        }
        return remove_own_pid_file(self.pid_path.as_path());
    }
}

/// What:
///  Whether a path is a regular file with the recorded device and inode,
///  without following links.
/// Why:
///   Following a link would compare another file's identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const metadata = await lstat(path, { bigint: true });
/// ```
#[cfg(unix)]
fn same_file_identity(path: &Path, identity: &LockIdentity) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt;
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error) => return Err(error.to_string()),
    };
    if !metadata.is_file()
        || metadata.dev().to_string() != identity.file.device
        || metadata.ino().to_string() != identity.file.inode
    {
        return Err(format!("the index lock identity changed: {}", path.display()));
    }
    return Ok(());
}

/// What:
///  On platforms without an inode,
///  the identity cannot be checked,
///  so the lock is never trusted.
/// Why:
///   The incumbent refuses a landing without a filesystem identity too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new TypeError('filesystem identity unavailable');
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn same_file_identity(path: &Path, _identity: &LockIdentity) -> Result<(), String> {
    return Err(format!("the index lock identity cannot be checked here: {}", path.display()));
}

/// What:
///  Create the lock file exclusively with private permissions.
/// Why:
///   `O_EXCL` is the mutual exclusion;
///  an existing file means another owner holds the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// open(lockPath, O_CREAT | O_EXCL | O_RDWR, 0o600);
/// ```
#[cfg(unix)]
fn create_exclusive(path: &Path) -> Result<bool, String> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    match options.open(path) {
        Ok(_) => return Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Ok(false),
        Err(error) => return Err(error.to_string()),
    }
}

/// What:
///  Create the lock file exclusively,
///  on platforms without Unix modes.
/// Why:
///   Private permissions are not applied there;
///  the identity check still refuses the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// open(lockPath, O_CREAT | O_EXCL | O_RDWR);
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn create_exclusive(path: &Path) -> Result<bool, String> {
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    match options.open(path) {
        Ok(_) => return Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Ok(false),
        Err(error) => return Err(error.to_string()),
    }
}

/// What:
///  The identity of a freshly created lock file.
/// Why:
///   The journal records it;
///  a platform without device and inode numbers records nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { file: { device: String(stat.dev), inode: String(stat.ino) }, fsId: await resolveFsId(dirname(path)) };
/// ```
#[cfg(unix)]
fn created_identity(lock_path: &Path, real_index: &Path) -> Result<LockIdentity, String> {
    use std::os::unix::fs::MetadataExt;
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(lock_path) {
        Ok(found) => found,
        Err(error) => return Err(error.to_string()),
    };
    let directory: &Path = real_index.parent().unwrap_or(Path::new("."));
    let fs_id: String = match filesystem_identity(directory) {
        Some(found) => found,
        None => return Err(String::from("the filesystem identity of the worktree is unavailable")),
    };
    return Ok(LockIdentity {
        file: FileIdentity {
            device: metadata.dev().to_string(),
            inode: metadata.ino().to_string(),
        },
        fs_id,
    });
}

/// What:
///  On platforms without device and inode numbers,
///  no identity is recorded and the landing stops.
/// Why:
///   The Windows volume serial is not exposed by the standard library,
///  as the design notes say.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new TypeError('filesystem identity unavailable');
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn created_identity(lock_path: &Path, _real_index: &Path) -> Result<LockIdentity, String> {
    return Err(format!("the index lock identity cannot be recorded here: {}", lock_path.display()));
}

/// What:
///  Remove the PID file when it still names this process.
/// Why:
///   Another process may have announced itself meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if ((await readFile(pidPath, 'utf8')) === `pid ${process.pid}\n`) await rm(pidPath);
/// ```
fn remove_own_pid_file(pid_path: &Path) -> Result<(), String> {
    let own: String = format!("pid {}\n", std::process::id());
    match std::fs::read_to_string(pid_path) {
        Ok(text) => {
            if text == own && let Err(error) = std::fs::remove_file(pid_path) {
                return Err(error.to_string());
            }
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    }
}

/// What:
///  Create the lock exclusively,
///  waiting for a foreign holder under the classification rules,
///  and read the identity of the file created.
/// Why:
///   `openLockWaiting` plus the identity step of the landing;
///  a lock whose identity cannot be recorded is removed again,
///  because nothing proves it is ours.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = await openLockWaiting({ realIndexPath, timeoutMs });
/// ```
fn create_identified_lock(
    real_index: &Path,
    timeout_ms: u64,
    consequence: &str,
) -> Result<(PathBuf, LockIdentity), CoordinationFailure> {
    let lock_path: PathBuf = real_lock_path(real_index);
    loop {
        match create_exclusive(lock_path.as_path()) {
            Ok(true) => break,
            Ok(false) => wait_for_lock(real_index, timeout_ms, consequence)?,
            Err(reason) => return Err(CoordinationFailure::Stop(reason)),
        }
    }
    match created_identity(lock_path.as_path(), real_index) {
        Ok(identity) => return Ok((lock_path, identity)),
        Err(reason) => {
            let _ = std::fs::remove_file(lock_path.as_path());
            return Err(CoordinationFailure::Stop(reason));
        }
    }
}

/// What:
///  Take the real index lock for a landing so that its journal record exists before the lock does:
///  create a candidate file in the transaction directory,
///  journal the candidate's identity,
///  then publish the candidate as `index.lock` by a hard link,
///  which is as exclusive as `O_EXCL` and gives the lock the candidate's inode.
///  `Ok(None)` means the filesystem refused the link and nothing was left behind,
///  so the caller creates the lock in place and journals it afterwards.
/// Why:
///   A kill between the creation of `index.lock` and the write of its record leaves a lock that nothing proves
///  ours, which every later command refuses to remove (the lock may be Git's); no window like that remains when the
///  record comes first. A candidate that outlives a kill lies in the transaction directory, which recovery removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeCandidate(); await journalIdentity(candidate); await link(candidate, indexLock); await rm(candidate);
/// ```
fn create_journaled_lock(
    real_index: &Path,
    transaction_directory: &Path,
    attempt: i64,
    timeout_ms: u64,
    consequence: &str,
) -> Result<Option<(PathBuf, LockIdentity)>, CoordinationFailure> {
    let lock_path: PathBuf = real_lock_path(real_index);
    let candidate: PathBuf = transaction_directory.join(format!("index-lock-{attempt}.candidate"));
    match create_exclusive(candidate.as_path()) {
        Ok(true) => {}
        Ok(false) => return Err(CoordinationFailure::Stop(format!("{} already exists", candidate.display()))),
        Err(reason) => return Err(CoordinationFailure::Stop(reason)),
    }
    let identity: LockIdentity = match created_identity(candidate.as_path(), real_index) {
        Ok(found) => found,
        Err(reason) => {
            let _ = std::fs::remove_file(candidate.as_path());
            return Err(CoordinationFailure::Stop(reason));
        }
    };
    if let Err(reason) = write_lock_record(transaction_directory, attempt, &identity) {
        let _ = std::fs::remove_file(candidate.as_path());
        return Err(CoordinationFailure::Stop(reason));
    }
    loop {
        match std::fs::hard_link(candidate.as_path(), lock_path.as_path()) {
            Ok(()) => break,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if let Err(failure) = wait_for_lock(real_index, timeout_ms, consequence) {
                    // The lock was never taken, so nothing is journaled for it.
                    let _ = std::fs::remove_file(candidate.as_path());
                    let _ = std::fs::remove_file(transaction_directory.join(index_lock_record_filename(attempt)));
                    return Err(failure);
                }
            }
            Err(error) => {
                debug(
                    TAG,
                    format!("the filesystem refused to link the lock candidate ({error}); creating the lock in place").as_str(),
                );
                let _ = std::fs::remove_file(candidate.as_path());
                let record: PathBuf = transaction_directory.join(index_lock_record_filename(attempt));
                if let Err(removal) = std::fs::remove_file(record.as_path()) {
                    return Err(CoordinationFailure::Stop(format!("removing {} failed: {removal}", record.display())));
                }
                return Ok(None);
            }
        }
    }
    // The lock now has its own name; the candidate's name was only the way in.
    if let Err(error) = std::fs::remove_file(candidate.as_path()) {
        debug(TAG, format!("removing the lock candidate {} failed: {error}", candidate.display()).as_str());
    }
    return Ok(Some((lock_path, identity)));
}

/// What:
///  Take the real index lock,
///  waiting for a foreign holder under the classification rules,
///  then
///       record its identity and announce this process through the PID file.
/// Why:
///   `openLockWaiting` plus the identity and PID steps of the landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = await openLockWaiting({ realIndexPath, timeoutMs });
/// ```
pub fn acquire_real_index_lock(
    real_index: &Path,
    timeout_ms: u64,
    consequence: &str,
) -> Result<RealIndexLock, CoordinationFailure> {
    let (lock_path, identity): (PathBuf, LockIdentity) = create_identified_lock(real_index, timeout_ms, consequence)?;
    let pid_path: PathBuf = real_pid_path(real_index);
    if let Err(error) = write_private_file(pid_path.as_path(), format!("pid {}\n", std::process::id()).as_bytes()) {
        let _ = std::fs::remove_file(lock_path.as_path());
        return Err(CoordinationFailure::Stop(error.to_string()));
    }
    return Ok(RealIndexLock {
        lock_path,
        real_index: real_index.to_path_buf(),
        pid_path,
        identity,
    });
}

/// What:
///  Write `index-lock-<n>.json` for a lock just created,
///  then make the directory entry durable.
/// Why:
///   Recovery can prove ownership of the lock before any landing record exists,
///  so the record must be durable before the PID file announces the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeJournalRecord({ directory: transactionDirectory, filename: indexLockRecordFilename(attempt), record });
/// ```
fn write_lock_record(transaction_directory: &Path, attempt: i64, identity: &LockIdentity) -> Result<(), String> {
    let record: IndexLockRecord = IndexLockRecord {
        attempt,
        lock: identity.clone(),
    };
    let path: PathBuf = transaction_directory.join(index_lock_record_filename(attempt));
    if let Err(error) = write_private_file(path.as_path(), encode_index_lock(&record).as_bytes()) {
        return Err(format!("writing {} failed: {error}", path.display()));
    }
    if let Err(error) = sync_directory(transaction_directory) {
        return Err(format!("syncing {} failed: {error}", transaction_directory.display()));
    }
    return Ok(());
}

/// What:
///  Take the real index lock for a landing attempt:
///  create it,
///  journal its identity in `index-lock-<n>.json`,
///  fix its mode,
///  and only then announce this process through the PID file.
/// Why:
///   `acquireRealIndexLock`:
///  the journal comes before the PID file,
///  so a crash never leaves a PID file naming a lock that recovery cannot prove ours;
///  an existing PID file is left alone,
///  as the incumbent does.
///  A failure after the lock exists removes the lock again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await using lock = await acquireRealIndexLock({ realIndexPath, transactionDirectory, attempt, timeoutMs });
/// ```
pub fn acquire_landing_index_lock(
    real_index: &Path,
    transaction_directory: &Path,
    attempt: i64,
    timeout_ms: u64,
    consequence: &str,
) -> Result<RealIndexLock, CoordinationFailure> {
    if let Some((lock_path, identity)) = create_journaled_lock(real_index, transaction_directory, attempt, timeout_ms, consequence)? {
        let announced: Result<(), String> = announce_held_lock(real_index, lock_path.as_path());
        if let Err(reason) = announced {
            let _ = std::fs::remove_file(lock_path.as_path());
            return Err(CoordinationFailure::Stop(reason));
        }
        return Ok(RealIndexLock {
            lock_path,
            real_index: real_index.to_path_buf(),
            pid_path: real_pid_path(real_index),
            identity,
        });
    }
    let (lock_path, identity): (PathBuf, LockIdentity) = create_identified_lock(real_index, timeout_ms, consequence)?;
    let announced: Result<(), String> = announce_landing_lock(real_index, transaction_directory, attempt, lock_path.as_path(), &identity);
    if let Err(reason) = announced {
        let _ = std::fs::remove_file(lock_path.as_path());
        return Err(CoordinationFailure::Stop(reason));
    }
    return Ok(RealIndexLock {
        lock_path,
        real_index: real_index.to_path_buf(),
        pid_path: real_pid_path(real_index),
        identity,
    });
}

/// What:
///  The steps between creating a landing's lock and holding it:
///  journal record,
///  mode,
///  PID file.
/// Why:
///   One failure path removes the lock,
///  whichever step failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeJournalRecord(...); await protectPath({ path: lockPath, directory: false }); await writeOwnPidFile(pidPath);
/// ```
fn announce_landing_lock(
    real_index: &Path,
    transaction_directory: &Path,
    attempt: i64,
    lock_path: &Path,
    identity: &LockIdentity,
) -> Result<(), String> {
    write_lock_record(transaction_directory, attempt, identity)?;
    return announce_held_lock(real_index, lock_path);
}

/// What:
///  The steps between holding a landing's lock and announcing it:
///  mode,
///  PID file.
/// Why:
///   The journal record is already durable, whether it was written before the lock was published or after it was
///  created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await protectPath({ path: lockPath, directory: false }); await writeOwnPidFile(pidPath);
/// ```
fn announce_held_lock(real_index: &Path, lock_path: &Path) -> Result<(), String> {
    if let Err(error) = protect_path(lock_path, 0o600) {
        return Err(format!("protecting {} failed: {error}", lock_path.display()));
    }
    let pid_path: PathBuf = real_pid_path(real_index);
    match write_private_file(pid_path.as_path(), format!("pid {}\n", std::process::id()).as_bytes()) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            debug(TAG, format!("a PID file already exists beside the lock: {}", pid_path.display()).as_str());
            return Ok(());
        }
        Err(error) => return Err(format!("writing {} failed: {error}", pid_path.display())),
    }
}

/// Real-lock controls stay out of the release executable.
#[cfg(test)]
#[path = "real_index_lock_tests.rs"]
mod tests;
