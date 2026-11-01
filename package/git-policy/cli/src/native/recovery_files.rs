//! What:
//!  No-follow file operations recovery and landing share:
//!  existence probes,
//!  exact reads,
//!       byte comparison,
//!  owner-preserving hard links,
//!  proof that a real `index.lock` is the
//!       exact file a transaction created,
//!  and installing an index through such a lock.
//! Why:
//!  Recovery removes or installs through a real `index.lock` only when the journal proves the
//!      lock is the transaction's own,
//!  by device,
//!  inode and filesystem identity,
//!  never by path;
//!      the incumbent's `commit-transaction-recovery-files.ts`,
//!  `-validation.ts` and
//!      `commit-transaction-install-link.ts` define these steps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await installRecoveredIndex({ lockPath, realIndexPath, postIndexPath, journal });
//! ```

/// Debug diagnostics.
use super::diagnostic_log::debug;
/// The filesystem identity recorded with a lock.
use super::filesystem_identity::filesystem_identity;
/// Private reads and directory syncs.
use super::private_storage::{ReadRefusal, read_regular_file, sync_directory};
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// The recorded identity of a real `index.lock`.
use super::transaction_journal::LockIdentity;
/// The trait that gives files `.write_all`.
use std::io::Write;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Largest recovery artifact read whole:
///  indexes of very large repositories stay well below it.
pub const RECOVERY_FILE_LIMIT: u64 = 2 * 1024 * 1024 * 1024;

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
///  The exact bytes of a regular recovery file,
///  read without following a link.
/// Why:
///   `readRegularRecoveryFile`:
///  a link or a non-regular file is unsafe state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readRegularRecoveryFile(path: string): Promise<Uint8Array>;
/// ```
pub fn read_recovery_file(path: &Path) -> Result<Vec<u8>, RecoveryError> {
    match read_regular_file(path, RECOVERY_FILE_LIMIT) {
        Ok(bytes) => return Ok(bytes),
        Err(ReadRefusal::Missing) => {
            return Err(RecoveryError(format!(
                "Transaction recovery file is missing: {}",
                path.display()
            )));
        }
        Err(ReadRefusal::NotRegular | ReadRefusal::TooLarge) => {
            return Err(RecoveryError(format!(
                "Unsafe transaction recovery file: {}",
                path.display()
            )));
        }
        Err(ReadRefusal::Io(error)) => return Err(io_failure("reading", path, &error)),
    }
}

/// What:
///  The device and inode of a path as decimal text,
///  without following a link,
///  with
///       whether it is a regular file.
/// Why:
///   Identities are recorded and compared as the incumbent's `String(metadata.dev)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { dev, ino } = await lstat(path, { bigint: true }); [String(dev), String(ino)]
/// ```
pub fn file_identity_of(path: &Path) -> std::io::Result<(String, String, bool)> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path)?;
    return Ok(metadata_identity(&metadata));
}

/// What:
///  The device,
///  inode and regular-file flag of metadata.
/// Why:
///   Shared by path and descriptor identities.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [String(metadata.dev), String(metadata.ino), metadata.isFile()]
/// ```
pub fn metadata_identity(metadata: &std::fs::Metadata) -> (String, String, bool) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        return (
            metadata.dev().to_string(),
            metadata.ino().to_string(),
            metadata.file_type().is_file(),
        );
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        // Windows has no stable device and inode in the standard library; the identity is
        // recorded as unknown, so a recovery there fails closed instead of guessing.
        return (String::new(), String::new(), metadata.file_type().is_file());
    }
}

/// What:
///  Whether two files hold the same bytes,
///  streamed in fixed chunks through no-follow
///       handles;
///  a file that cannot be opened is a failure.
/// Why:
///   `snapshotFilesEqual`:
///  recovery compares the real index with recorded snapshots,
///  which
///       can be large,
///  so neither is read whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function snapshotFilesEqual({ leftPath, rightPath }): Promise<boolean>;
/// ```
pub fn files_equal(left: &Path, right: &Path) -> Result<bool, RecoveryError> {
    let mut first: std::fs::File = open_no_follow(left)?;
    let mut second: std::fs::File = open_no_follow(right)?;
    // `mut` allows reusing both buffers for every chunk.
    let mut first_buffer: Vec<u8> = vec![0; COMPARISON_CHUNK];
    let mut second_buffer: Vec<u8> = vec![0; COMPARISON_CHUNK];
    loop {
        let first_count: usize = match fill(&mut first, first_buffer.as_mut_slice()) {
            Ok(count) => count,
            Err(error) => return Err(io_failure("reading", left, &error)),
        };
        let second_count: usize = match fill(&mut second, second_buffer.as_mut_slice()) {
            Ok(count) => count,
            Err(error) => return Err(io_failure("reading", right, &error)),
        };
        if first_count != second_count
            || first_buffer[..first_count] != second_buffer[..second_count]
        {
            return Ok(false);
        }
        if first_count < COMPARISON_CHUNK {
            return Ok(true);
        }
    }
}

/// Bytes compared per chunk.
const COMPARISON_CHUNK: usize = 64 * 1024;

/// What:
///  Read until `buffer` is full or the file ends;
///  returns how many bytes were read.
/// Why:
///   A short read must not end the comparison early.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let total = 0; while (total < buffer.length) { const { bytesRead } = await handle.read(buffer, total); if (!bytesRead) break; total += bytesRead; }
/// ```
fn fill(file: &mut std::fs::File, buffer: &mut [u8]) -> std::io::Result<usize> {
    use std::io::Read;
    let mut total: usize = 0;
    while total < buffer.len() {
        let count: usize = file.read(&mut buffer[total..])?;
        if count == 0 {
            break;
        }
        total += count;
    }
    return Ok(total);
}

/// What:
///  Open an existing file for reading without following a final link.
/// Why:
///   Snapshots and the real index are compared as the files themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await open(path, O_RDONLY | O_NOFOLLOW)
/// ```
fn open_no_follow(path: &Path) -> Result<std::fs::File, RecoveryError> {
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    match options.open(path) {
        Ok(file) => return Ok(file),
        Err(error) => return Err(io_failure("opening", path, &error)),
    }
}

/// What:
///  A file's bytes,
///  following links as Node's `readFile` does,
///  or nothing when missing.
/// Why:
///   Conclusion entries and index snapshots may be absent.
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
///  Hard-link `source` at `linked` and prove the new name is the expected regular file.
/// Why:
///   `createOwnedFileLink`:
///  an install or a stabilized artifact keeps the exact inode,
///  so a
///       file swapped in under the old name is detected instead of installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function createOwnedFileLink({ sourcePath, linkedPath, expectedDevice, expectedInode }): Promise<void>;
/// ```
pub fn create_owned_file_link(
    source: &Path,
    linked: &Path,
    expected_device: &str,
    expected_inode: &str,
) -> Result<(), RecoveryError> {
    remove_file_if_present(linked)?;
    if let Err(error) = std::fs::hard_link(source, linked) {
        return Err(io_failure("linking", linked, &error));
    }
    let matches: bool = match file_identity_of(linked) {
        Ok((device, inode, regular)) => {
            regular && device == expected_device && inode == expected_inode
        }
        Err(_) => false,
    };
    if !matches {
        remove_file_if_present(linked)?;
        return Err(RecoveryError(format!(
            "Commit transaction file link identity changed: {}",
            linked.display()
        )));
    }
    return Ok(());
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
///  Why `lock_path` is not the exact `index.lock` the journal recorded,
///  or nothing when it
///       is:
///  a regular file,
///  not a link,
///  with the recorded device,
///  inode and filesystem identity.
///       A failure to inspect it is an error,
///  never a mismatch.
/// Why:
///   The incumbent's `assertOwnedLock` throws its recovery error only for a mismatch;
///  an
///       `lstat` or filesystem-identity failure propagates,
///  and `releaseOwnedLock` treats only the
///       mismatch as another owner's lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await assertOwnedLock({ journal, lockPath }); return undefined; } catch (e) { if (e instanceof CommitTransactionRecoveryError) return e.message; throw e; }
/// ```
pub fn owned_lock_mismatch(
    journal: &LockIdentity,
    lock_path: &Path,
) -> Result<Option<String>, RecoveryError> {
    return owned_lock_mismatch_with(journal, lock_path, filesystem_identity);
}

/// What:
///  The question of `owned_lock_mismatch` with the filesystem-identity mechanism passed in.
/// Why:
///   A lock another process removes between the file inspection and the identity lookup is a race
///       that a control can only reproduce by replacing the mechanism.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function ownedLockMismatch({ journal, lockPath, resolveFsId }): Promise<string | undefined>;
/// ```
fn owned_lock_mismatch_with(
    journal: &LockIdentity,
    lock_path: &Path,
    identity_of_filesystem: fn(&Path) -> Option<String>,
) -> Result<Option<String>, RecoveryError> {
    let (device, inode, regular) = match file_identity_of(lock_path) {
        Ok(found) => found,
        Err(error) => return Err(io_failure("inspecting the index lock", lock_path, &error)),
    };
    if !regular {
        return Ok(Some(format!(
            "Index lock is not a regular owned file: {}",
            lock_path.display()
        )));
    }
    let Some(filesystem) = identity_of_filesystem(lock_path) else {
        return Err(RecoveryError(format!(
            "unable to resolve the filesystem identity for {}",
            lock_path.display()
        )));
    };
    if filesystem != journal.fs_id || device != journal.file.device || inode != journal.file.inode {
        return Ok(Some(format!(
            "Index lock identity changed: {}",
            lock_path.display()
        )));
    }
    return Ok(None);
}

/// What:
///  Require `lock_path` to be the exact `index.lock` the journal recorded.
/// Why:
///   `assertOwnedLock`:
///  cli-git never deletes or installs through a foreign lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function assertOwnedLock({ journal, lockPath }): Promise<void>; // throws when foreign
/// ```
pub fn assert_owned_lock(journal: &LockIdentity, lock_path: &Path) -> Result<(), RecoveryError> {
    if let Some(mismatch) = owned_lock_mismatch(journal, lock_path)? {
        return Err(RecoveryError(mismatch));
    }
    return Ok(());
}

/// What:
///  What happened when a lock the transaction no longer needs was released.
/// Why:
///   Callers log it;
///  only an owned lock is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnedLockRelease = 'released' | 'absent' | 'foreign';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockRelease {
    /// The owned lock was removed.
    Released,
    /// No lock was there.
    Absent,
    /// Another owner's lock is there now and was left in place.
    Foreign,
}

/// What:
///  The incumbent's name of a release outcome.
/// Why:
///   Debug lines read `attempt 1 lock released`,
///  as the incumbent's do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// release // 'released' | 'absent' | 'foreign'
/// ```
pub fn lock_release_name(release: LockRelease) -> &'static str {
    match release {
        LockRelease::Released => return "released",
        LockRelease::Absent => return "absent",
        LockRelease::Foreign => return "foreign",
    }
}

/// What:
///  Remove the real-index lock only when it is still the exact file the transaction made.
/// Why:
///   `releaseOwnedLock`:
///  a lock another process holds now is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function releaseOwnedLock({ journal, lockPath }): Promise<OwnedLockRelease>;
/// ```
pub fn release_owned_lock(
    journal: &LockIdentity,
    lock_path: &Path,
) -> Result<LockRelease, RecoveryError> {
    return release_owned_lock_with(journal, lock_path, filesystem_identity);
}

/// What:
///  The release of `release_owned_lock` with the filesystem-identity mechanism passed in.
/// Why:
///   Controls reproduce a concurrent recoverer removing the lock mid-release by replacing the mechanism.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function releaseOwnedLock({ journal, lockPath, resolveFsId }): Promise<OwnedLockRelease>;
/// ```
fn release_owned_lock_with(
    journal: &LockIdentity,
    lock_path: &Path,
    identity_of_filesystem: fn(&Path) -> Option<String>,
) -> Result<LockRelease, RecoveryError> {
    if !recovery_path_exists(lock_path)? {
        return Ok(LockRelease::Absent);
    }
    let proof: Result<Option<String>, RecoveryError> =
        owned_lock_mismatch_with(journal, lock_path, identity_of_filesystem);
    let proven: Option<String> = match proof {
        Ok(found) => found,
        Err(error) => {
            // Another recoverer of the same dead transaction may have released the lock after the existence
            // probe; a lock that is gone now was released by that peer, whereas one that still exists but
            // cannot be identified stays an error.
            if recovery_path_exists(lock_path)? {
                return Err(error);
            }
            debug(
                "releaseOwnedLock",
                format!("lock released by another recoverer: {}", lock_path.display()).as_str(),
            );
            return Ok(LockRelease::Absent);
        }
    };
    if let Some(mismatch) = proven {
        debug(
            "releaseOwnedLock",
            format!("leaving lock another owner holds: {mismatch}").as_str(),
        );
        return Ok(LockRelease::Foreign);
    }
    if let Err(error) = std::fs::remove_file(lock_path) {
        if error.kind() == std::io::ErrorKind::NotFound {
            debug(
                "releaseOwnedLock",
                format!("lock released by another recoverer: {}", lock_path.display()).as_str(),
            );
            return Ok(LockRelease::Absent);
        }
        return Err(io_failure(
            "removing the owned index lock",
            lock_path,
            &error,
        ));
    }
    debug(
        "releaseOwnedLock",
        format!("released owned lock {}", lock_path.display()).as_str(),
    );
    return Ok(LockRelease::Released);
}

/// What:
///  Give an open file the access and modification times of `source`.
/// Why:
///   `applyIndexTimestamps`:
///  an installed index keeps the times of the index it was
///       computed from,
///  so Git's racy-entry protection still re-hashes same-second edits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await applyIndexTimestamps({ sourcePath, handle });
/// ```
pub fn apply_timestamps(source: &Path, file: &std::fs::File) -> std::io::Result<()> {
    let metadata: std::fs::Metadata = std::fs::metadata(source)?;
    let times: std::fs::FileTimes = std::fs::FileTimes::new()
        .set_accessed(metadata.accessed()?)
        .set_modified(metadata.modified()?);
    return file.set_times(times);
}

/// What:
///  Install a recorded post-index through the exact owned lock:
///  write it into the lock,
///       link the lock under a private name,
///  rename that name over the real index,
///  then remove
///       the lock.
/// Why:
///   `installRecoveredIndex`:
///  the install never goes through the mutable lock path,
///  and the
///       lock is proven to be the owned one before and after each step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installRecoveredIndex({ lockPath, realIndexPath, postIndexPath, journal }): Promise<void>;
/// ```
pub fn install_recovered_index(
    lock_path: &Path,
    real_index: &Path,
    post_index: &Path,
    journal: &LockIdentity,
) -> Result<(), RecoveryError> {
    let bytes: Vec<u8> = read_recovery_file(post_index)?;
    let mut lock: std::fs::File = open_lock_no_follow(lock_path)?;
    let opened: std::fs::Metadata = match lock.metadata() {
        Ok(metadata) => metadata,
        Err(error) => return Err(io_failure("inspecting the index lock", lock_path, &error)),
    };
    let (device, inode, _) = metadata_identity(&opened);
    if device != journal.file.device || inode != journal.file.inode {
        return Err(RecoveryError(format!(
            "Index lock identity changed: {}",
            lock_path.display()
        )));
    }
    let written: std::io::Result<()> = write_lock_contents(&mut lock, bytes.as_slice(), post_index);
    if let Err(error) = written {
        return Err(io_failure(
            "writing the recovered index into",
            lock_path,
            &error,
        ));
    }
    assert_owned_lock(journal, lock_path)?;
    let install: PathBuf = parent_of(post_index).join("install.index");
    create_owned_file_link(
        lock_path,
        install.as_path(),
        journal.file.device.as_str(),
        journal.file.inode.as_str(),
    )?;
    drop(lock);
    if let Err(error) = std::fs::rename(install.as_path(), real_index) {
        return Err(io_failure(
            "installing the recovered index",
            real_index,
            &error,
        ));
    }
    assert_owned_lock(journal, lock_path)?;
    if let Err(error) = std::fs::remove_file(lock_path) {
        return Err(io_failure(
            "removing the owned index lock",
            lock_path,
            &error,
        ));
    }
    let parent: &Path = parent_of(real_index);
    if let Err(error) = sync_directory(parent) {
        return Err(io_failure("syncing", parent, &error));
    }
    return Ok(());
}

/// What:
///  The parent directory of a path,
///  or the path itself when it has none.
/// Why:
///   Recorded paths are absolute files,
///  so they always have a parent;
///  a path without one
///       names itself instead of panicking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dirname(path)
/// ```
fn parent_of(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) => return parent,
        None => return path,
    }
}

/// What:
///  Open an existing lock for reading and writing without following a link.
/// Why:
///   The lock must be the file itself,
///  never a planted link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await open(lockPath, O_RDWR | O_NOFOLLOW)
/// ```
fn open_lock_no_follow(lock_path: &Path) -> Result<std::fs::File, RecoveryError> {
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    match options.open(lock_path) {
        Ok(file) => return Ok(file),
        Err(error) => return Err(io_failure("opening the index lock", lock_path, &error)),
    }
}

/// What:
///  Replace a lock's contents with `bytes`,
///  give it the source's times,
///  and sync it.
/// Why:
///   The steps between the identity checks of `install_recovered_index`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lock.truncate(0); await lock.writeFile(bytes); await applyIndexTimestamps(...); await lock.sync();
/// ```
fn write_lock_contents(
    lock: &mut std::fs::File,
    bytes: &[u8],
    source: &Path,
) -> std::io::Result<()> {
    lock.set_len(0)?;
    lock.write_all(bytes)?;
    apply_timestamps(source, lock)?;
    return lock.sync_all();
}

/// What:
///  The path of Git's owner PID file for a lock,
///  `<base>~pid.lock` beside `<base>.lock`.
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
