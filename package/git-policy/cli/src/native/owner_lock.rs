//! What:
//!  Rename-published owner-lock directories with process-birth ownership:
//!  acquisition
//!       with dead-owner retirement,
//!  an unbounded wait while the owner lives,
//!  and release by
//!       rename before deletion.
//! Why:
//!  The landing,
//!  reservation,
//!  hook,
//!  capture and push locks all work this way in the
//!      incumbent (`src/owner-lock/owner-lock.ts`) and in its hook dispatcher,
//!  and both
//!      wrappers contend on the same directories while both are installed.
//!  A published lock
//!      always names its owner because its candidate is complete before the rename.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await using lock = await acquireOwnerLock({ lockDirectory });
//! ```

/// The owner record,
///  its reading and owner liveness.
use super::owner_lock_record::{
    OWNER_LOCK_RECORD_FILENAME, OwnerLockRecord, OwnerRecordError, PublishedOwner,
    encode_owner_lock_record, owner_lock_holder_is_alive, read_owner_lock_record,
};
/// Private creation and tree removal.
use super::private_storage::{create_private_directory, remove_tree, write_exclusive_synced};
/// The PID namespace this process records as its own.
use super::pid_namespace::current_pid_namespace;
/// The current process's birth identity.
use super::process_identity::{ProcessIdentityError, current_birth_identity};
/// Unguessable tokens and unique candidate names.
use super::random_id::{RandomSourceError, random_uuid};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// When a wait's notices are due.
use super::wait_progress::{Cadence, Ticker};
/// `Duration` is a span of time; `Instant` is a point on a clock that never goes backwards.
use std::time::{Duration, Instant};

/// Delay between acquisition attempts while a live owner holds the lock.
pub const DEFAULT_POLL_DELAY: Duration = Duration::from_millis(20);

/// What:
///  Why an owner-lock operation failed.
/// Why:
///   Each failure stops the caller with the lock named;
///  none is a reason to delete a
///       lock that might belong to a live owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class OwnerLockError extends Error {}
/// ```
#[derive(Debug)]
pub enum OwnerLockError {
    /// The blocking lock's record is malformed or unreadable.
    Record(OwnerRecordError),
    /// The current process has no identity,
    ///  or an owner's liveness could not be decided.
    Identity(ProcessIdentityError),
    /// No random token could be made.
    Random(RandomSourceError),
    /// A filesystem step failed.
    Io {
        /// What the step was doing,
        ///  in words.
        operation: &'static str,
        /// The path it acted on.
        path: PathBuf,
        /// The operating system's refusal.
        error: std::io::Error,
    },
    /// The lock named another owner when this holder released it.
    OwnershipChanged(PathBuf),
    /// A live owner still held the lock when the caller's wait limit ran out.
    WaitTimedOut {
        /// The lock that stayed held.
        path: PathBuf,
        /// How long the caller waited.
        waited: Duration,
        /// The live holder's process ID,
        ///  when its record could be read.
        holder_pid: Option<i64>,
    },
}

/// What:
///  Human-readable text for each failure.
/// Why:
///   Diagnostics name the lock and the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.message
/// ```
impl std::fmt::Display for OwnerLockError {
    /// Writes one sentence naming the lock and what went wrong.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Record(OwnerRecordError::Malformed(path)) => {
                return write!(
                    formatter,
                    "the owner lock record {} is malformed",
                    path.display()
                );
            }
            Self::Record(OwnerRecordError::Unreadable(path, error)) => {
                return write!(
                    formatter,
                    "the owner lock record {} could not be read: {error}",
                    path.display()
                );
            }
            Self::Identity(error) => return write!(formatter, "{error}"),
            Self::Random(error) => return write!(formatter, "{}", error.0),
            Self::Io {
                operation,
                path,
                error,
            } => {
                return write!(formatter, "{operation} {} failed: {error}", path.display());
            }
            Self::OwnershipChanged(path) => {
                return write!(
                    formatter,
                    "the owner lock {} changed owner while it was held",
                    path.display()
                );
            }
            Self::WaitTimedOut {
                path,
                waited,
                holder_pid,
            } => {
                let holder: String = match holder_pid {
                    Some(pid) => format!("process {pid}"),
                    None => String::from("a process whose record could not be read"),
                };
                return write!(
                    formatter,
                    "gave up after waiting {} seconds for the owner lock {}, which {holder} still holds; \
                     the lock was left in place. To go on, let that process finish and run the command again, \
                     check with `ps` whether it is stuck and stop it if so, or raise the limit with \
                     CLI_GIT_LOCK_WAIT_SECONDS (CLI_GIT_PUSH_LOCK_WAIT_SECONDS for the push lock)",
                    waited.as_secs(),
                    path.display()
                );
            }
        }
    }
}

/// What:
///  Wrap a filesystem error with the step and path that failed.
/// Why:
///   One constructor keeps every filesystem failure named the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new OwnerLockError(`${operation} ${path} failed: ${error.message}`)
/// ```
fn io_error(operation: &'static str, path: &Path, error: std::io::Error) -> OwnerLockError {
    return OwnerLockError::Io {
        operation,
        path: path.to_path_buf(),
        error,
    };
}

/// What:
///  Whether a rename failed because another directory already holds the name.
/// Why:
///   `rename` onto a non-empty directory fails with `EEXIST` or `ENOTEMPTY` on Unix and
///       with an access-denied error on Windows (Node's `EPERM`);
///  the incumbent's
///       `isOccupiedError` accepts exactly these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isOccupiedError(error: unknown): boolean;
/// ```
pub fn is_occupied_error(error: &std::io::Error) -> bool {
    return matches!(
        error.kind(),
        std::io::ErrorKind::AlreadyExists
            | std::io::ErrorKind::DirectoryNotEmpty
            | std::io::ErrorKind::PermissionDenied
    );
}

/// What:
///  A unique sibling name `<lock>.<uuid>.<suffix>` for a candidate or a retired lock.
/// Why:
///   Candidates (`.pending`) and retired locks (`.stale`) never collide with each other,
///       and every reader ignores both names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${lockDirectory}.${randomUUID()}.${suffix}`
/// ```
fn sibling_name(lock_directory: &Path, suffix: &str) -> Result<PathBuf, OwnerLockError> {
    let unique: String = random_uuid().map_err(OwnerLockError::Random)?;
    // `as_os_str().to_owned()` copies the raw path bytes so the suffix can be appended.
    let mut name: std::ffi::OsString = lock_directory.as_os_str().to_owned();
    name.push(format!(".{unique}.{suffix}"));
    return Ok(PathBuf::from(name));
}

/// What:
///  Read a lock's owner,
///  turning record failures into lock failures.
/// Why:
///   Every step of acquisition and release reads the owner the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readOwnerLockRecord(lockDirectory)
/// ```
fn read_owner(lock_directory: &Path) -> Result<PublishedOwner, OwnerLockError> {
    return read_owner_lock_record(lock_directory).map_err(OwnerLockError::Record);
}

/// What:
///  A held owner lock.
///  Releasing it proves the lock still names this holder first.
///       Dropping an unreleased lock releases it and ignores a failure.
/// Why:
///   An early return or an unwinding error must not leave a live-looking lock behind;
///       an explicit `release` reports a changed owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnerLock = AsyncDisposable & { lockDirectory: string; token: string };
/// ```
#[derive(Debug)]
pub struct OwnerLock {
    /// Published lock directory.
    directory: PathBuf,
    /// Token published in the lock.
    token: String,
    /// Whether the lock was already released.
    released: bool,
}

/// Accessors and release of a held lock.
impl OwnerLock {
    /// What:
    ///  A view of a lock an ancestor process holds, which releases nothing.
    /// Why:
    ///   An invocation nested under a landing-lock holder proves it with an inherited lease and proceeds under the
    ///  ancestor's lock instead of waiting for it (`inheritedLandingLock` in the incumbent).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { lockDirectory, token: process.env[LANDING_LEASE_ENV], [Symbol.asyncDispose]: async () => {} }
    /// ```
    pub fn inherited(directory: &Path, token: &str) -> OwnerLock {
        return OwnerLock {
            directory: directory.to_path_buf(),
            token: String::from(token),
            released: true,
        };
    }

    /// What:
    ///  The published lock directory.
    /// Why:
    ///   Leases and diagnostics name the lock.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lock.lockDirectory
    /// ```
    pub fn directory(&self) -> &Path {
        return self.directory.as_path();
    }

    /// What:
    ///  The token this holder published.
    /// Why:
    ///   A lease proves which acquisition it belongs to.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lock.token
    /// ```
    pub fn token(&self) -> &str {
        return self.token.as_str();
    }

    /// What:
    ///  Release the lock,
    ///  failing when it no longer names this holder.
    ///       `mut self` takes ownership:
    ///  a released lock cannot be used again.
    /// Why:
    ///   The release is the incumbent's `releaseHeldLock`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await lock[Symbol.asyncDispose]();
    /// ```
    pub fn release(mut self) -> Result<(), OwnerLockError> {
        // A view of an ancestor's lock was never taken by this holder, so it has nothing to release.
        if self.released {
            return Ok(());
        }
        self.released = true;
        return release_held_lock(self.directory.as_path(), self.token.as_str());
    }
}

/// Dropping an unreleased lock releases it.
impl Drop for OwnerLock {
    /// What:
    ///  Release the lock unless `release` already did;
    ///  a failure is ignored.
    /// Why:
    ///   Nothing can be reported from a drop,
    ///  and a lock that changed owner is not this
    ///       holder's to remove.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// // `await using` disposal on an early exit.
    /// ```
    fn drop(&mut self) {
        if !self.released {
            self.released = true;
            let _ = release_held_lock(self.directory.as_path(), self.token.as_str());
        }
    }
}

/// What:
///  Move a renamed-away directory back to the published name when it turned out to
///       belong to someone else.
/// Why:
///   A race can move a live replacement aside;
///  putting it back is all that can be done,
///       and an occupied name means a newer owner already published,
///  which is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await rename(moved, lockDirectory); } catch (e) { if (!isOccupiedError(e)) throw e; }
/// ```
fn restore_moved(moved: &Path, lock_directory: &Path) -> Result<(), OwnerLockError> {
    match std::fs::rename(moved, lock_directory) {
        Ok(()) => return Ok(()),
        Err(error) if is_occupied_error(&error) => return Ok(()),
        Err(error) => return Err(io_error("restoring the owner lock", lock_directory, error)),
    }
}

/// What:
///  Release a held lock by renaming it away,
///  then deleting the renamed copy.
/// Why:
///   Emptying the published directory in place would let a concurrent acquirer's rename
///       replace it mid-removal and lose the next owner's record;
///  the rename frees the name
///       atomically.
///  A lock that names another owner before or after the move is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function releaseHeldLock({ lockDirectory, token }): Promise<void>;
/// ```
fn release_held_lock(lock_directory: &Path, token: &str) -> Result<(), OwnerLockError> {
    if !names_token(&read_owner(lock_directory)?, token) {
        return Err(OwnerLockError::OwnershipChanged(
            lock_directory.to_path_buf(),
        ));
    }
    let retired: PathBuf = sibling_name(lock_directory, "stale")?;
    if let Err(error) = std::fs::rename(lock_directory, retired.as_path()) {
        return Err(io_error("releasing the owner lock", lock_directory, error));
    }
    return settle_moved_release(retired.as_path(), lock_directory, token);
}

/// What:
///  Finish a release after the lock was renamed away:
///  delete it when it is this holder's,
///       or put it back and report the change when a race moved someone else's lock.
/// Why:
///   Between the owner check and the rename another process may have replaced the lock;
///       the moved copy is checked again before anything is deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moved = await readOwnerLockRecord(retired); if (moved.token !== token) { await restore(); throw ...; } await rm(retired);
/// ```
fn settle_moved_release(
    retired: &Path,
    lock_directory: &Path,
    token: &str,
) -> Result<(), OwnerLockError> {
    if !names_token(&read_owner(retired)?, token) {
        restore_moved(retired, lock_directory)?;
        return Err(OwnerLockError::OwnershipChanged(
            lock_directory.to_path_buf(),
        ));
    }
    if let Err(error) = remove_tree(retired) {
        return Err(io_error("removing the released lock", retired, error));
    }
    return Ok(());
}

/// What:
///  Whether a read owner is present and published `token`.
/// Why:
///   Release and retirement both ask whether the lock still holds a given acquisition.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (owner !== LOCK_BUSY) && (owner.token === token)
/// ```
fn names_token(owner: &PublishedOwner, token: &str) -> bool {
    match owner {
        PublishedOwner::Owner(record) => return record.token == token,
        PublishedOwner::Busy => return false,
    }
}

/// What:
///  Retire a lock whose owner was found dead,
///  unless it changed owner meanwhile.
/// Why:
///   The liveness check took time,
///  during which the dead-looking owner may have released
///       and a live acquirer published.
///  The lock is renamed to a unique stale name,
///  and a live
///       replacement moved by the race is restored.
///  The caller retries afterwards either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function retireDeadLock({ lockDirectory, deadToken }): Promise<typeof LOCK_BUSY>;
/// ```
fn retire_dead_lock(lock_directory: &Path, dead_token: &str) -> Result<(), OwnerLockError> {
    return retire_dead_lock_with(lock_directory, dead_token, rename_directory);
}

/// What:
///  Rename a directory on the real filesystem.
/// Why:
///   `std::fs::rename` is generic over its path types, so the retirement's injectable step needs this plain function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(from, to);
/// ```
fn rename_directory(from: &Path, to: &Path) -> std::io::Result<()> {
    return std::fs::rename(from, to);
}

/// What:
///  Retire a dead owner's lock with the rename step supplied by the caller.
/// Why:
///   The lock can vanish between the owner check and the rename, a window only a racing process opens;
///  a rename that fails the way a vanished lock fails lets a test reach that arm without a race.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function retireDeadLockWith({ lockDirectory, deadToken, rename }): Promise<void>;
/// ```
fn retire_dead_lock_with(
    lock_directory: &Path,
    dead_token: &str,
    rename: fn(&Path, &Path) -> std::io::Result<()>,
) -> Result<(), OwnerLockError> {
    if !names_token(&read_owner(lock_directory)?, dead_token) {
        return Ok(());
    }
    let stale: PathBuf = sibling_name(lock_directory, "stale")?;
    match rename(lock_directory, stale.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(io_error(
                "retiring the dead owner lock",
                lock_directory,
                error,
            ));
        }
    }
    return settle_moved_retirement(stale.as_path(), lock_directory, dead_token);
}

/// What:
///  Finish a retirement after the lock was renamed away:
///  delete the dead owner's lock,
///       or put back a live replacement that a race moved instead.
/// Why:
///   A concurrent acquirer may have published between the owner check and the rename;
///  a
///       moved lock that names another token is not the dead owner's to delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moved = await readOwnerLockRecord(stale); if (moved !== LOCK_BUSY && moved.token !== deadToken) { await restore(); return; } await rm(stale);
/// ```
fn settle_moved_retirement(
    stale: &Path,
    lock_directory: &Path,
    dead_token: &str,
) -> Result<(), OwnerLockError> {
    if let PublishedOwner::Owner(record) = read_owner(stale)?
        && record.token != dead_token
    {
        return restore_moved(stale, lock_directory);
    }
    if let Err(error) = remove_tree(stale) {
        return Err(io_error("removing the retired lock", stale, error));
    }
    return Ok(());
}

/// What:
///  Write a complete unpublished candidate:
///  a private directory and its owner record.
/// Why:
///   Publication by rename makes a lock appear with its owner record already complete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeCandidate({ candidateDirectory, record }): Promise<void>;
/// ```
fn write_candidate(candidate: &Path, record: &OwnerLockRecord) -> Result<(), OwnerLockError> {
    if let Err(error) = create_private_directory(candidate) {
        return Err(io_error("creating the lock candidate", candidate, error));
    }
    let record_path: PathBuf = candidate.join(OWNER_LOCK_RECORD_FILENAME);
    let bytes: String = encode_owner_lock_record(record);
    if let Err(error) = write_exclusive_synced(record_path.as_path(), bytes.as_bytes()) {
        // The candidate was never published and nothing else names it, so it is removed with the failure: left
        // beside the lock, it would stay for good.
        if let Err(removal) = remove_tree(candidate) {
            super::diagnostic_log::debug(
                "writeCandidate",
                format!("removing the unpublished lock candidate {} failed: {removal}", candidate.display()).as_str(),
            );
        }
        return Err(io_error(
            "writing the lock record",
            record_path.as_path(),
            error,
        ));
    }
    return Ok(());
}

/// What:
///  One publication attempt:
///  publish a fresh candidate,
///  or retire a dead owner's lock
///       that blocks it.
///  `Ok(None)` means busy:
///  try again.
/// Why:
///   The incumbent's `attemptAcquire`,
///  step for step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function attemptAcquire({ lockDirectory, record }): Promise<OwnerLock | typeof LOCK_BUSY>;
/// ```
fn attempt_acquire(
    lock_directory: &Path,
    record: &OwnerLockRecord,
) -> Result<Option<OwnerLock>, OwnerLockError> {
    let candidate: PathBuf = sibling_name(lock_directory, "pending")?;
    write_candidate(candidate.as_path(), record)?;
    match std::fs::rename(candidate.as_path(), lock_directory) {
        Ok(()) => {
            return Ok(Some(OwnerLock {
                directory: lock_directory.to_path_buf(),
                token: record.token.clone(),
                released: false,
            }));
        }
        Err(error) => {
            if let Err(removal) = remove_tree(candidate.as_path()) {
                return Err(io_error(
                    "removing the lock candidate",
                    candidate.as_path(),
                    removal,
                ));
            }
            if !is_occupied_error(&error) {
                return Err(io_error("publishing the owner lock", lock_directory, error));
            }
        }
    }
    let PublishedOwner::Owner(published) = read_owner(lock_directory)? else {
        return Ok(None);
    };
    if owner_lock_holder_is_alive(&published, lock_directory).map_err(OwnerLockError::Identity)? {
        return Ok(None);
    }
    retire_dead_lock(lock_directory, published.token.as_str())?;
    return Ok(None);
}

/// What:
///  A fresh owner record for the current process.
/// Why:
///   Every acquisition publishes a new token with this process's PID and birth identity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function currentOwnerRecord(transactionId?: string): Promise<OwnerLockRecord>;
/// ```
fn current_owner_record(transaction_id: Option<&str>) -> Result<OwnerLockRecord, OwnerLockError> {
    return Ok(OwnerLockRecord {
        token: random_uuid().map_err(OwnerLockError::Random)?,
        owner_pid: i64::from(std::process::id()),
        owner_birth_identity: current_birth_identity().map_err(OwnerLockError::Identity)?,
        // `.map(String::from)` copies the borrowed ID into owned text when there is one.
        transaction_id: transaction_id.map(String::from),
        owner_pid_namespace: current_pid_namespace(),
    });
}

/// What:
///  One publication attempt without waiting.
///  `Ok(None)` means another owner holds the
///       lock,
///  or a dead owner's lock was just retired and the caller should try again.
/// Why:
///   The reservation is taken this way,
///  by a requester that re-checks its turn between
///       attempts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function tryAcquireOwnerLock({ lockDirectory, transactionId }): Promise<OwnerLock | typeof LOCK_BUSY>;
/// ```
pub fn try_acquire_owner_lock(
    lock_directory: &Path,
    transaction_id: Option<&str>,
) -> Result<Option<OwnerLock>, OwnerLockError> {
    return attempt_acquire(lock_directory, &current_owner_record(transaction_id)?);
}

/// What:
///  Retire a lock whose recorded owner is dead;
///  an absent lock or a live owner's lock is
///       left alone.
/// Why:
///   Recovery retires a dead reservation holder's lock this way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function retireLockOfDeadOwner(lockDirectory: string): Promise<void>;
/// ```
pub fn retire_lock_of_dead_owner(lock_directory: &Path) -> Result<(), OwnerLockError> {
    let PublishedOwner::Owner(published) = read_owner(lock_directory)? else {
        return Ok(());
    };
    if owner_lock_holder_is_alive(&published, lock_directory).map_err(OwnerLockError::Identity)? {
        return Ok(());
    }
    return retire_dead_lock(lock_directory, published.token.as_str());
}

/// What:
///  Something told once that an acquirer has to wait,
///  such as a stderr notice.
///       A `trait` is an interface a type implements.
/// Why:
///   A waiter may tell the user whose lock it waits for;
///  the lock module stays silent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OnWait = () => void;
/// ```
pub trait WaitNotice {
    /// Called once,
    ///  when the first attempt finds the lock held.
    fn waiting(&mut self);

    /// Called when the wait has lasted the cadence's grace,
    ///  and again at every interval after it,
    ///  with how long the wait has lasted.
    ///  Brief contention never reaches it;
    ///  a notice that only needs the first moment ignores it.
    fn slow(&mut self, _waited: Duration) {}

    /// Whether `waiting` already said so out loud,
    ///  so the first `slow` call comes one interval after the start instead of after the grace.
    fn announces_at_start(&self) -> bool {
        return false;
    }
}

/// What:
///  The notice that says nothing.
/// Why:
///   Most acquirers wait silently,
///  as the incumbent's default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const onWait = undefined;
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct SilentWait;

/// Waiting silently does nothing.
impl WaitNotice for SilentWait {
    /// Does nothing.
    fn waiting(&mut self) {}
}

/// What:
///  Acquire a lock,
///  waiting at most `limit` while a live owner holds it,
///  and telling `notice.slow` once the wait has lasted the cadence's grace and again at every interval.
/// Why:
///   No acquisition can wait forever,
///  so a holder that is alive but stuck fails its waiters instead of stalling them.
///  The elapsed time is measured on a clock that never goes backwards,
///  and tests choose a short cadence instead of waiting out the standard one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function acquireWithin({ lockDirectory, pollDelayMs, onWait, limitMs, cadence }): Promise<OwnerLock>;
/// ```
pub fn acquire_within(
    lock_directory: &Path,
    poll_delay: Duration,
    notice: &mut dyn WaitNotice,
    limit: Duration,
    cadence: Cadence,
) -> Result<OwnerLock, OwnerLockError> {
    let record: OwnerLockRecord = current_owner_record(None)?;
    let started: Instant = Instant::now();
    // `mut` lets the loop remember that the wait was announced and when its next notice is due.
    let mut notified: bool = false;
    let mut ticker: Ticker = if notice.announces_at_start() {
        Ticker::announced_at_start(cadence)
    } else {
        Ticker::new(cadence)
    };
    loop {
        if let Some(lock) = attempt_acquire(lock_directory, &record)? {
            return Ok(lock);
        }
        if !notified {
            notified = true;
            notice.waiting();
        }
        if ticker.due(started.elapsed()) {
            notice.slow(started.elapsed());
        }
        if started.elapsed() >= limit {
            let holder_pid: Option<i64> = match read_owner(lock_directory) {
                Ok(PublishedOwner::Owner(published)) => Some(published.owner_pid),
                Ok(PublishedOwner::Busy) | Err(_) => None,
            };
            return Err(OwnerLockError::WaitTimedOut {
                path: lock_directory.to_path_buf(),
                waited: started.elapsed(),
                holder_pid,
            });
        }
        std::thread::sleep(poll_delay);
    }
}

/// What:
///  Acquire a lock in a test,
///  waiting at most `TEST_ACQUIRE_LIMIT` while a live owner holds it.
///       `notice` is told once,
///  when the first attempt finds the lock busy;
///       `&mut dyn WaitNotice` lends any value implementing that interface.
/// Why:
///   Tests take locks to set up or to prove a release;
///  a lock a mutant never releases must fail the test within the limit,
///  not hang it until the mutation run's timeout.
///  A dead owner's lock is retired on the next attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function acquireOwnerLock({ lockDirectory, pollDelayMs = 20, onWait }): Promise<OwnerLock>;
/// ```
#[cfg(test)]
pub fn acquire_owner_lock(
    lock_directory: &Path,
    poll_delay: Duration,
    notice: &mut dyn WaitNotice,
) -> Result<OwnerLock, OwnerLockError> {
    return acquire_within(lock_directory, poll_delay, notice, TEST_ACQUIRE_LIMIT, Cadence::standard());
}

/// How long a test waits for a lock before failing.
#[cfg(test)]
const TEST_ACQUIRE_LIMIT: Duration = Duration::from_secs(20);

/// What:
///  Acquire a lock,
///  giving up with `OwnerLockError::WaitTimedOut` once `limit` has passed while a live owner holds it.
/// Why:
///   A holder that is alive but stuck would otherwise stall every waiter forever.
///  The held lock is never taken over,
///  because a live owner may still be inside its critical section;
///  the caller decides what a refusal means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function acquireOwnerLockFor({ lockDirectory, pollDelayMs, onWait, limitMs }): Promise<OwnerLock>; // rejects after limitMs
/// ```
pub fn acquire_owner_lock_for(
    lock_directory: &Path,
    poll_delay: Duration,
    notice: &mut dyn WaitNotice,
    limit: Duration,
) -> Result<OwnerLock, OwnerLockError> {
    return acquire_within(lock_directory, poll_delay, notice, limit, Cadence::from_environment());
}

/// Lock contention,
///  retirement and release controls stay out of the release executable.
#[cfg(test)]
#[path = "owner_lock_tests.rs"]
mod tests;
