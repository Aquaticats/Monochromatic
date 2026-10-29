//! What:
//!  The settlement lock that serializes worktree-copy installation and recovery per common Git directory,
//!  its owner record, liveness-based retirement of dead owners, and the inherited-lease check nested Git runs use.
//! Why:
//!  Two copies into one repository must not interleave, but a commit in a linked worktree must never wait for them;
//!  only an owner proven alive may be waited for, and only an owner proven dead may be replaced.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await using lease = await acquireWorktreeCopyLock(commonDir);
//! ```

use super::index_lock_wait::{BackoffState, INITIAL_BACKOFF, jittered_wait, next_backoff};
use super::json_record::quote;
use super::live_lock::{LIVE_WAIT_VARIABLE, live_wait_limit};
use super::owner_lock_record::{OwnerLockRecord, owner_lock_holder_is_alive};
use super::pid_namespace::current_pid_namespace;
use super::process_identity::current_birth_identity;
use super::random_id::random_uuid;
use super::wait_progress::{Cadence, Ticker};
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use crate::worktree_copy_journal::ensure_journal_root;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// What:
///  The published name of the settlement lock inside the journal root.
/// Why:
///   Every process in one repository contends on this exact name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(journalRoot, 'settlement.lock')
/// ```
pub const SETTLEMENT_LOCK_NAME: &str = "settlement.lock";

/// What:
///  The owner record's file name inside a published lock directory.
/// Why:
///   Readers find the owner by this name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SETTLEMENT_OWNER_FILENAME = 'owner.json';
/// ```
pub const OWNER_RECORD_NAME: &str = "owner.json";

/// What:
///  How long an owner without evidence may hold the lock before acquisition gives up.
/// Why:
///   A malformed record must not block the repository forever, but a live owner is waited for without limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const UNPROVEN_OWNER_TIMEOUT_MS = 1_000;
/// ```
pub const UNPROVEN_OWNER_TIMEOUT_MS: u64 = 1_000;

/// What:
///  The delay between read-only checks while a live owner holds the lock.
/// Why:
///   Short enough to notice a release promptly, long enough not to spin.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LIVE_OWNER_POLL_MS = 20;
/// ```
const LIVE_OWNER_POLL: Duration = Duration::from_millis(20);

/// What:
///  The environment variable through which nested Git runs inherit a held lease.
/// Why:
///   A hook that runs the wrapper inside `git worktree add` must forward without waiting for its own parent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const WORKTREE_COPY_LEASE_ENV = 'CLI_GIT_WORKTREE_COPY_LEASE';
/// ```
pub const WORKTREE_COPY_LEASE_ENV: &str = "CLI_GIT_WORKTREE_COPY_LEASE";

/// What:
///  The identity an owner record names: a unguessable token, a process, and that process's birth identity.
/// Why:
///   Liveness is judged on the process and its birth, so a reused PID is never taken for the owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockOwner = { leaseToken: string; ownerPid: number; ownerBirthIdentity: string; schemaVersion: 1 };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockOwner {
    /// Capability passed to real-Git descendants.
    pub lease_token: String,
    /// The owner's process identifier.
    pub owner_pid: i64,
    /// The owner's process birth identity.
    pub owner_birth_identity: String,
    /// The owner's PID namespace, or nothing for a record written before the field existed.
    pub owner_pid_namespace: Option<String>,
}

/// What:
///  What a published lock directory currently says.
/// Why:
///   Acquisition acts differently on a live owner, a dead one, a vanished lock, and an unproven record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readPublishedOwner(lockDirectory): Promise<LockOwner | UnprovenOwner | typeof LOCK_RETRY>;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
enum Published {
    /// A parsed owner record.
    Owner(LockOwner),
    /// The lock vanished or changed while it was read.
    Retry,
    /// A record or file that proves nothing, with the evidence.
    Unproven(String),
}

/// What:
///  The text of one owner record, as the incumbent writes it.
/// Why:
///   Other wrappers read this record, so its key order and spelling are fixed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify({ leaseToken, ownerBirthIdentity, ownerPid, schemaVersion })}\n`
/// ```
pub fn encode_owner(owner: &LockOwner) -> String {
    // Keys stay in sorted order, so the optional namespace sits between `ownerPid` and `schemaVersion`.
    let namespace: String = owner
        .owner_pid_namespace
        .as_deref()
        .map_or_else(String::new, |name| { return format!("\"ownerPidNamespace\":{},", quote(name)); });
    return format!(
        "{{\"leaseToken\":{},\"ownerBirthIdentity\":{},\"ownerPid\":{},{namespace}\"schemaVersion\":1}}\n",
        quote(owner.lease_token.as_str()),
        quote(owner.owner_birth_identity.as_str()),
        owner.owner_pid
    );
}

/// What:
///  Parses one owner record, or names why it proves nothing.
/// Why:
///   Every field is checked before the owner's process is ever consulted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLockOwner(text: string): LockOwner | string;
/// ```
pub fn parse_owner(text: &str) -> Result<LockOwner, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|error| format!("not JSON ({error})"))?;
    let missing: &str = "missing or mistyped owner fields";
    let object = value
        .as_object()
        .ok_or_else(|| return String::from(missing))?;
    let lease_token: &str = object
        .get("leaseToken")
        .and_then(serde_json::Value::as_str)
        .filter(|token| return !token.is_empty())
        .ok_or_else(|| return String::from(missing))?;
    let owner_pid: i64 = object
        .get("ownerPid")
        .and_then(serde_json::Value::as_i64)
        .filter(|pid| return *pid >= 1)
        .ok_or_else(|| return String::from(missing))?;
    let birth: &str = object
        .get("ownerBirthIdentity")
        .and_then(serde_json::Value::as_str)
        .filter(|identity| return !identity.is_empty())
        .ok_or_else(|| return String::from(missing))?;
    if object
        .get("schemaVersion")
        .and_then(serde_json::Value::as_f64)
        != Some(1.0)
    {
        return Err(String::from(missing));
    }
    return Ok(LockOwner {
        lease_token: lease_token.to_string(),
        owner_pid,
        owner_birth_identity: birth.to_string(),
        // A present field must be non-empty text; anything else leaves the PID to decide, as before the field.
        owner_pid_namespace: object
            .get("ownerPidNamespace")
            .and_then(serde_json::Value::as_str)
            .filter(|namespace| { return !namespace.is_empty(); })
            .map(String::from),
    });
}

/// What:
///  Reads the owner a published lock names.
/// Why:
///   A vanished lock asks for another attempt; an unreadable or invalid one is unproven evidence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readPublishedOwner(lockDirectory);
/// ```
fn read_published(lock: &Path) -> Result<Published, WorktreeCopyError> {
    let owner_path: PathBuf = lock.join(OWNER_RECORD_NAME);
    let text: String = match std::fs::read_to_string(owner_path.as_path()) {
        Ok(text) => text,
        Err(error) => {
            // `match` on the operating-system code: vanished or non-directory means retry; unreadable means unproven.
            return match error.raw_os_error() {
                Some(code) if code == libc::ENOENT || code == libc::ENOTDIR => Ok(Published::Retry),
                Some(code) if code == libc::EACCES || code == libc::EISDIR => {
                    let name: &str = if code == libc::EACCES {
                        "EACCES"
                    } else {
                        "EISDIR"
                    };
                    Ok(Published::Unproven(format!(
                        "{} is unreadable ({name})",
                        quote(owner_path.to_string_lossy().as_ref())
                    )))
                }
                _ => Err(raw_failure(&error)),
            };
        }
    };
    return Ok(match parse_owner(text.as_str()) {
        Ok(owner) => Published::Owner(owner),
        Err(reason) => Published::Unproven(format!(
            "{} holds an invalid owner record: {reason}",
            quote(owner_path.to_string_lossy().as_ref())
        )),
    });
}

/// What:
///  Whether a recorded owner is alive with the same process birth.
/// Why:
///   A reused PID or an exited but unreaped process is dead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await lockOwnerIsAlive(owner);
/// ```
fn owner_alive(owner: &LockOwner, lock: &Path) -> Result<bool, WorktreeCopyError> {
    let record: OwnerLockRecord = OwnerLockRecord {
        token: owner.lease_token.clone(),
        owner_pid: owner.owner_pid,
        owner_birth_identity: owner.owner_birth_identity.clone(),
        transaction_id: None,
        owner_pid_namespace: owner.owner_pid_namespace.clone(),
    };
    return owner_lock_holder_is_alive(&record, lock).map_err(|error| return raw_failure(&error));
}

/// What:
///  The disposable settlement lock: released explicitly, or removed when dropped if still held.
/// Why:
///   Release reports an ownership change; the drop fallback cannot report, so it only removes what it still owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyLease = AsyncDisposable & { leaseToken: string };
/// ```
#[derive(Debug)]
pub struct WorktreeCopyLease {
    /// The published lock directory.
    lock_directory: PathBuf,
    /// The owner identity this process published.
    owner: LockOwner,
    /// Whether release has already run.
    released: bool,
}

impl WorktreeCopyLease {
    /// The token a nested Git run inherits.
    pub fn lease_token(&self) -> &str {
        return self.owner.lease_token.as_str();
    }

    /// Removes the lock when this process still owns it, and fails with the ownership wording otherwise.
    pub fn release(mut self) -> Result<(), WorktreeCopyError> {
        self.released = true;
        if !remove_lock_of(
            self.lock_directory.as_path(),
            self.owner.lease_token.as_str(),
        )? {
            return Err(failure_with_cause(
                format!(
                    "cli-git: worktree-copy lock ownership changed: {}.",
                    quote(self.lock_directory.to_string_lossy().as_ref())
                )
                .as_str(),
                None,
            ));
        }
        return Ok(());
    }
}

impl Drop for WorktreeCopyLease {
    /// Removes a lease that was never released explicitly, ignoring a failure that nothing can report.
    fn drop(&mut self) {
        if !self.released {
            let _ignored: Result<bool, WorktreeCopyError> = remove_lock_of(
                self.lock_directory.as_path(),
                self.owner.lease_token.as_str(),
            );
        }
    }
}

/// What:
///  Removes a published lock by renaming it away first, when it still names the expected owner.
/// Why:
///   A rename frees the published name atomically, so a concurrent publication is never emptied in place;
///  a replacement moved by a race is renamed back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeLockOf({ lockDirectory, leaseToken });
/// ```
fn remove_lock_of(lock: &Path, lease_token: &str) -> Result<bool, WorktreeCopyError> {
    return remove_lock_of_with(lock, lease_token, rename_lock);
}

/// The rename that frees a published lock name,
///  as a plain function so a test can supply another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(from, to);
/// ```
fn rename_lock(from: &Path, to: &Path) -> std::io::Result<()> {
    return std::fs::rename(from, to);
}

/// What:
///  `remove_lock_of` with the rename that frees the published name supplied by the caller.
/// Why:
///   A lock that vanishes between the read and the rename, and a rename that is refused, are races and faults a test
///  cannot otherwise provoke on demand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeLockOf({ lockDirectory, leaseToken, rename });
/// ```
fn remove_lock_of_with(
    lock: &Path,
    lease_token: &str,
    rename: fn(&Path, &Path) -> std::io::Result<()>,
) -> Result<bool, WorktreeCopyError> {
    match read_published(lock)? {
        Published::Owner(owner) if owner.lease_token == lease_token => {}
        _ => return Ok(false),
    }
    let retired: PathBuf = PathBuf::from(format!(
        "{}.{}.stale",
        lock.to_string_lossy(),
        random_uuid().map_err(|error| return raw_failure(&error.0))?
    ));
    // `match` on the rename: a vanished lock is simply not ours to remove; any other failure is fatal.
    match rename(lock, retired.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(raw_failure(&error)),
    }
    let moved: Published = read_published(retired.as_path())?;
    if !matches!(&moved, Published::Owner(owner) if owner.lease_token == lease_token) {
        // A replacement moved by the race goes back; a rename-back collision leaves both in place for inspection.
        let _ignored: std::io::Result<()> = std::fs::rename(retired.as_path(), lock);
        return Ok(false);
    }
    std::fs::remove_dir_all(retired.as_path()).map_err(|error| return raw_failure(&error))?;
    return Ok(true);
}

/// What:
///  Publishes this process's owner as a complete candidate directory, then renames it to the lock name.
/// Why:
///   The rename is the single atomic step that makes the lock visible, so a reader never sees an owner-less lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeCandidateOwner({ candidateDirectory, owner }); await rename(candidateDirectory, lockDirectory);
/// ```
fn publish_candidate(
    lock: &Path,
    owner: &LockOwner,
) -> Result<Option<WorktreeCopyLease>, WorktreeCopyError> {
    let candidate: PathBuf = PathBuf::from(format!(
        "{}.{}.pending",
        lock.to_string_lossy(),
        random_uuid().map_err(|error| return raw_failure(&error.0))?
    ));
    write_candidate(candidate.as_path(), owner)?;
    match std::fs::rename(candidate.as_path(), lock) {
        Ok(()) => {
            return Ok(Some(WorktreeCopyLease {
                lock_directory: lock.to_path_buf(),
                owner: owner.clone(),
                released: false,
            }));
        }
        Err(error) => {
            let _ignored: std::io::Result<()> = std::fs::remove_dir_all(candidate.as_path());
            // `match` on the error: an occupied name means another owner holds the lock; anything else is fatal.
            if matches!(error.kind(), std::io::ErrorKind::AlreadyExists)
                || error.raw_os_error() == Some(libc::ENOTEMPTY)
            {
                return Ok(None);
            }
            return Err(raw_failure(&error));
        }
    }
}

/// What:
///  Writes one candidate lock directory with its owner record, synced, and removes it on failure.
/// Why:
///   A candidate is never visible under the lock name until its owner record is complete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeCandidateOwner({ candidateDirectory, owner }): Promise<void>;
/// ```
fn write_candidate(candidate: &Path, owner: &LockOwner) -> Result<(), WorktreeCopyError> {
    let written: std::io::Result<()> = (|| {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new().mode(0o700).create(candidate)?;
        crate::private_storage::write_exclusive_synced(
            candidate.join(OWNER_RECORD_NAME).as_path(),
            encode_owner(owner).as_bytes(),
        )?;
        return Ok(());
    })();
    if let Err(error) = written {
        let _ignored: std::io::Result<()> = std::fs::remove_dir_all(candidate);
        return Err(raw_failure(&error));
    }
    return Ok(());
}

/// What:
///  The owner identity of this process with a fresh token.
/// Why:
///   Every acquisition publishes a token no other process can guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await currentLockOwner();
/// ```
fn current_owner() -> Result<LockOwner, WorktreeCopyError> {
    let birth: String = current_birth_identity().map_err(|error| {
        return failure_with_cause(
            "cli-git: current worktree-copy lock owner identity is unavailable.",
            Some(&error),
        );
    })?;
    let token: String = random_uuid().map_err(|error| return raw_failure(&error.0))?;
    return Ok(LockOwner {
        lease_token: token,
        owner_pid: i64::from(std::process::id()),
        owner_birth_identity: birth,
        owner_pid_namespace: current_pid_namespace(),
    });
}

/// What:
///  One unit-interval random number from the operating system.
/// Why:
///   Git's backoff is jittered, so waiters do not move in lockstep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.random()
/// ```
fn unit_random() -> Result<f64, WorktreeCopyError> {
    let mut bytes: [u8; 8] = [0; 8];
    getrandom::fill(&mut bytes).map_err(|error| return raw_failure(&error))?;
    return Ok((u64::from_le_bytes(bytes) >> 11) as f64 / (1u64 << 53) as f64);
}

/// What:
///  The lock as a waiter sees it at one attempt.
/// Why:
///   Each result tells the acquisition loop whether to take the lock, wait for a live owner, or back off.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttemptResult = { kind: 'acquired' } | { kind: 'live' } | { kind: 'retry' } | { kind: 'unproven' };
/// ```
enum Attempt {
    /// The lock is ours.
    Acquired(WorktreeCopyLease),
    /// A live owner holds it.
    Live(LockOwner),
    /// The lock changed or a dead owner was retired; try again.
    Retry,
    /// An owner without evidence holds it.
    Unproven(String),
}

/// What:
///  One publication attempt and the classification of whatever blocks it.
/// Why:
///   A dead owner is retired on the spot, so one attempt can end the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await attemptAcquire({ lockDirectory, owner });
/// ```
fn attempt(lock: &Path, owner: &LockOwner) -> Result<Attempt, WorktreeCopyError> {
    if let Some(lease) = publish_candidate(lock, owner)? {
        return Ok(Attempt::Acquired(lease));
    }
    return match read_published(lock)? {
        Published::Retry => Ok(Attempt::Retry),
        Published::Unproven(evidence) => Ok(Attempt::Unproven(evidence)),
        Published::Owner(holder) => {
            if owner_alive(&holder, lock)? {
                return Ok(Attempt::Live(holder));
            }
            let _retired: bool = remove_lock_of(lock, holder.lease_token.as_str())?;
            Ok(Attempt::Retry)
        }
    };
}

/// What:
///  Waits, reading only, while a live owner keeps the lock.
/// Why:
///   The wait ends when the owner releases, replaces the lock, or stops running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitWhileOwnerLives({ lockDirectory, owner });
/// ```
fn wait_while_owner_lives(
    lock: &Path,
    owner: &LockOwner,
    limit: Duration,
    notices: &mut dyn Write,
    cadence: Cadence,
) -> Result<(), WorktreeCopyError> {
    let started: Instant = Instant::now();
    // The caller printed the first line when the wait began, so the repeats start one interval later.
    let mut ticker: Ticker = Ticker::announced_at_start(cadence);
    loop {
        std::thread::sleep(LIVE_OWNER_POLL);
        // A notice that is due goes out before the limit is checked, so a refused wait was announced.
        if ticker.due(started.elapsed()) {
            let line: String = live_holder_progress_line(lock, owner, started.elapsed(), limit);
            // The notice is advisory: a failed write to standard error must not stop the wait.
            let _written = notices.write_all(line.as_bytes());
        }
        if started.elapsed() >= limit {
            let message: String = format!(
                "cli-git: gave up after waiting {} seconds for PID {}, which still holds worktree-copy settlement {}; {} sets how long cli-git waits. cli-git left the lock in place. To go on, let that process finish and run the command again, check with `ps` whether it is stuck and stop it if so, or raise the limit.",
                started.elapsed().as_secs(),
                owner.owner_pid,
                quote(lock.to_string_lossy().as_ref()),
                LIVE_WAIT_VARIABLE
            );
            return Err(failure_with_cause(message.as_str(), None));
        }
        match read_published(lock)? {
            Published::Owner(current) if current.lease_token == owner.lease_token => {
                if !owner_alive(&current, lock)? {
                    return Ok(());
                }
            }
            _ => return Ok(()),
        }
    }
}

/// What:
///  The stderr line that names a live holder.
/// Why:
///   A waiting user learns which process to look at and that cli-git is still waiting on purpose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// liveHolderLine({ lockDirectory, owner });
/// ```
fn live_holder_line(lock: &Path, owner: &LockOwner, limit: Duration) -> String {
    return format!(
        "cli-git: waiting for PID {}, which holds worktree-copy settlement {} (proven by its live process identity); giving up after {} seconds ({} changes this), leaving the lock in place.\n",
        owner.owner_pid,
        quote(lock.to_string_lossy().as_ref()),
        limit.as_secs(),
        LIVE_WAIT_VARIABLE
    );
}

/// What:
///  The stderr line that repeats while a live holder keeps the settlement lock.
/// Why:
///   A long installation by another process must not look like a hang:
///  each repeat names the holder, the lock, how long the wait has lasted and where it gives up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// liveHolderProgressLine({ lockDirectory, owner, waitedMs, limitMs });
/// ```
fn live_holder_progress_line(lock: &Path, owner: &LockOwner, waited: Duration, limit: Duration) -> String {
    return format!(
        "cli-git: still waiting for PID {}, which holds worktree-copy settlement {} (proven by its live process identity) after {} seconds; giving up after {} seconds ({} changes this), leaving the lock in place.\n",
        owner.owner_pid,
        quote(lock.to_string_lossy().as_ref()),
        waited.as_secs(),
        limit.as_secs(),
        LIVE_WAIT_VARIABLE
    );
}

/// What:
///  Acquires the settlement lock: waits for a proven-live owner up to this process's wait limit, retires a dead one,
///  and gives an owner without evidence the bounded wait before failing.
/// Why:
///   Installation and recovery of one repository must never interleave, and no lock is taken from an owner that might still run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await using lease = await acquireWorktreeCopyLock(commonDir);
/// ```
pub fn acquire_worktree_copy_lock(
    common_dir: &Path,
    stderr: &mut dyn Write,
) -> Result<WorktreeCopyLease, WorktreeCopyError> {
    return acquire_worktree_copy_lock_for(common_dir, stderr, live_wait_limit());
}

/// What:
///  Acquires the settlement lock, waiting for each proven-live owner for at most `live_limit`.
/// Why:
///   A live owner that never releases would otherwise hold every later command for good;
///  the refusal names it and leaves its lock in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await using lease = await acquireWorktreeCopyLock(commonDir, { liveLimitMs });
/// ```
pub fn acquire_worktree_copy_lock_for(
    common_dir: &Path,
    stderr: &mut dyn Write,
    live_limit: Duration,
) -> Result<WorktreeCopyLease, WorktreeCopyError> {
    return acquire_worktree_copy_lock_with(common_dir, stderr, live_limit, Cadence::from_environment());
}

/// What:
///  Acquires the settlement lock, waiting for each proven-live owner for at most `live_limit`
///  and repeating the holder line at the cadence's interval.
/// Why:
///   Tests choose a short cadence; the production entry reads it from the environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await using lease = await acquireWorktreeCopyLock(commonDir, { liveLimitMs, cadence });
/// ```
pub fn acquire_worktree_copy_lock_with(
    common_dir: &Path,
    stderr: &mut dyn Write,
    live_limit: Duration,
    cadence: Cadence,
) -> Result<WorktreeCopyLease, WorktreeCopyError> {
    let lock: PathBuf = ensure_journal_root(common_dir)?.join(SETTLEMENT_LOCK_NAME);
    let owner: LockOwner = current_owner()?;
    let mut named: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut unproven_since: Option<Instant> = None;
    let mut backoff: BackoffState = INITIAL_BACKOFF;
    loop {
        // `match` on the attempt: a held lock is waited for, a retry backs off, and an acquisition ends the loop.
        let unproven: Option<String> = match attempt(lock.as_path(), &owner)? {
            Attempt::Acquired(lease) => return Ok(lease),
            Attempt::Live(holder) => {
                unproven_since = None;
                if named.insert(holder.lease_token.clone()) {
                    let line: String = live_holder_line(lock.as_path(), &holder, live_limit);
                    stderr
                        .write_all(line.as_bytes())
                        .map_err(|error| return raw_failure(&error))?;
                }
                wait_while_owner_lives(lock.as_path(), &holder, live_limit, stderr, cadence)?;
                continue;
            }
            Attempt::Retry => None,
            Attempt::Unproven(evidence) => Some(evidence),
        };
        let started: Instant = *unproven_since.get_or_insert_with(Instant::now);
        if let Some(evidence) = unproven.filter(|_| {
            return started.elapsed() >= Duration::from_millis(UNPROVEN_OWNER_TIMEOUT_MS);
        }) {
            {
                let message: String = format!(
                    "cli-git: worktree-copy settlement lock {} stayed in place for {} ms and cli-git could not prove whether its owner is alive. Evidence: {}. cli-git left the lock in place; remove it only after confirming no cli-git process is using this repository.",
                    quote(lock.to_string_lossy().as_ref()),
                    UNPROVEN_OWNER_TIMEOUT_MS,
                    evidence
                );
                return Err(failure_with_cause(message.as_str(), None));
            }
        }
        let delay_ms: u64 = jittered_wait(backoff, unit_random()?);
        backoff = next_backoff(backoff);
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
}

/// What:
///  Takes the settlement lock if it is free or its dead owner is retired; reports a held lock instead of waiting.
/// Why:
///   A command that neither creates nor moves worktrees must not wait behind a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lease = await tryAcquireWorktreeCopyLock(commonDir); // WORKTREE_COPY_LOCK_HELD when held
/// ```
pub fn try_acquire_worktree_copy_lock(
    common_dir: &Path,
) -> Result<Option<WorktreeCopyLease>, WorktreeCopyError> {
    let lock: PathBuf = ensure_journal_root(common_dir)?.join(SETTLEMENT_LOCK_NAME);
    let owner: LockOwner = current_owner()?;
    // Ten attempts absorb a lock that keeps changing; a live or unproven holder ends the attempts at once.
    for _attempt in 0..10 {
        // `match` on the attempt: an acquisition returns the lease, a retry tries again, and any holder means held.
        match attempt(lock.as_path(), &owner)? {
            Attempt::Acquired(lease) => return Ok(Some(lease)),
            Attempt::Retry => continue,
            Attempt::Live(_) | Attempt::Unproven(_) => return Ok(None),
        }
    }
    return Err(failure_with_cause(
        format!(
            "cli-git: worktree-copy settlement lock under {} kept changing while it was acquired.",
            quote(common_dir.to_string_lossy().as_ref())
        )
        .as_str(),
        None,
    ));
}

/// What:
///  Whether the environment's lease token names the live owner of the published lock.
/// Why:
///   A nested Git run that carries the outer run's token forwards without waiting for the lock it is already inside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await validatesInheritedWorktreeCopyLease({ commonDir, leaseToken });
/// ```
pub fn validate_inherited_lease(
    common_dir: &Path,
    lease_token: &str,
) -> Result<bool, WorktreeCopyError> {
    if lease_token.is_empty() {
        return Ok(false);
    }
    let lock: PathBuf = ensure_journal_root(common_dir)?.join(SETTLEMENT_LOCK_NAME);
    return match read_published(lock.as_path())? {
        Published::Owner(owner) if owner.lease_token == lease_token => owner_alive(&owner, lock.as_path()),
        _ => Ok(false),
    };
}

/// Lock controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_lock_tests.rs"]
mod tests;
