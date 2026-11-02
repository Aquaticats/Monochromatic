//! What:
//!  The service's hold on Git's own `index.lock` across a landing: taken first, by exclusive creation, waited for without
//!  polling when another program holds it, and released by moving the new index over the real one.
//! Why:
//!  Git's own `commit` takes the index lock before it moves the branch and gives it up after the index is written. A landing
//!  that moved the branch first and took the lock afterwards left the index showing content a landed commit had replaced for
//!  as long as a foreign `git` held the lock, and a commit prepared from that index could land a revert. The wait for a
//!  foreign holder is event-driven: an inotify watch on the Git directory wakes the service when the lock itself is removed or
//!  moved away, and the only timer is the limit (`CLI_GIT_LOCK_WAIT_SECONDS`) and the repetition of the holder's notice. On
//!  platforms without inotify a held lock fails at once with the plain message. The lock carries a marker with the batch
//!  number and the holding process from the moment it exists until it is removed after the new index is in place, so a
//!  restart can tell its own stale lock from a foreign one wherever the service was killed, and never takes over a lock
//!  whose holder still runs (`doc/planning/cli-git-resident-service.md`, "Landing every command line").
//! The marker,
//! the inotify watch,
//! the placement of the marked file,
//! and the attempts and the wait
//! are in `service_index_marker.rs`,
//! `service_index_watch.rs`,
//! `service_index_placement.rs`
//! and `service_index_attempts.rs`,
//! and are re-exported here under the names callers use.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lock = await acquireIndexLock(indexPath, deadline, announce); ... await lock.releaseWith(newIndex);
//! ```

use super::service_index_attempts::{Placing, wait_and_place};
use super::service_protocol::FailureCode;
use std::path::{Path, PathBuf};
use std::io::Read;
use std::time::Duration;

pub use super::service_index_marker::{LOCK_MARKER, Marked, marker_text, parse_marker};
#[cfg(target_os = "linux")]
pub use super::service_index_watch::{events_end_watch, events_name};
pub use super::service_index_placement::{Platform, STAGED_INFIX, SYSTEM, staged_path_for};
pub use super::service_index_watch::Watch;
use super::service_index_marker::{holder_is_gone, is_reclaimable, marker_of};
use super::service_index_placement::{log_placement_failure, place_marked, reclaim, remove_dead_staging, stage_marker};

/// How often the holder's notice is repeated while the wait lasts.
pub const NOTICE_EVERY: Duration = Duration::from_secs(10);

/// The environment variable that sets how often the holder's line is repeated, in whole seconds.
pub const NOTICE_VARIABLE: &str = "CLI_GIT_WAIT_NOTICE_SECONDS";

/// How often the holder's line is repeated: the variable when it holds a positive number of seconds, ten seconds otherwise.
pub(super) fn notice_every() -> Duration {
    return std::env::var(NOTICE_VARIABLE).ok().and_then(|text| { return text.trim().parse::<u64>().ok(); }).filter(|seconds| { return *seconds > 0; }).map_or(NOTICE_EVERY, Duration::from_secs);
}

/// What the wait tells the clients of the batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Announce {
    /// A line for the person's terminal: who holds the lock and how long the wait lasts.
    Waiting(String),
    /// Who held the lock when the wait ran out, in words that fit "... is using this repository's index".
    Holder(String),
}

/// Why the lock was not taken.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockFailure {
    /// The code the clients are answered with.
    pub code: FailureCode,
}

/// The service's hold on one index lock; removed unless released.
#[derive(Debug)]
pub struct IndexLock {
    lock: PathBuf,
    index: PathBuf,
    /// The text this holder placed in the lock;
    /// the lock is released or removed only while it still holds exactly this.
    marker: String,
    armed: bool,
}

/// What:
///  One attempt to take the lock, with no wait.
/// Why:
///   Finishing a batch a killed service left open must not hold up the next batch for the whole limit; a foreign holder, or
///  a service holder that still runs, leaves the batch open for the next round.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tryAcquireIndexLock(index, seq)
/// ```
pub fn try_acquire(index: &Path, seq: u64) -> Option<IndexLock> {
    return try_acquire_on(&SYSTEM, index, seq);
}

/// `try_acquire` with the calls of `platform`.
pub fn try_acquire_on(platform: &Platform, index: &Path, seq: u64) -> Option<IndexLock> {
    let lock: PathBuf = lock_path_of(index);
    let marker: String = match marker_text(seq, platform.identity) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("cli-git-service: cannot take the index lock {}: this process has no birth identity to name in the lock ({error})", lock.display());
            return None;
        }
    };
    let staged: PathBuf = match stage_marker(&lock, seq, marker.as_str()) {
        Ok(path) => path,
        Err(error) => {
            log_placement_failure(&lock, &error);
            return None;
        }
    };
    let placed: std::io::Result<()> = match place_marked(platform, &staged, &lock) {
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && is_reclaimable(&lock) => {
            if reclaim(platform, &lock) { place_marked(platform, &staged, &lock) } else { Err(error) }
        }
        other => other,
    };
    let _removed = std::fs::remove_file(&staged);
    if let Err(error) = &placed {
        if error.kind() != std::io::ErrorKind::AlreadyExists {
            log_placement_failure(&lock, error);
        }
        return None;
    }
    remove_dead_staging(&lock);
    return Some(IndexLock { lock, index: index.to_path_buf(), marker, armed: true });
}

/// The path of the lock of an index.
pub fn lock_path_of(index: &Path) -> PathBuf {
    let mut name = index.as_os_str().to_os_string();
    name.push(".lock");
    return PathBuf::from(name);
}

/// What:
///  Takes `index.lock` by exclusive creation, waiting for a foreign holder until the limit.
/// Why:
///   Git's own protocol: creation fails while any process holds the lock. The wait blocks on an inotify watch set before the
///  first attempt, so a release between the attempt and the wait is not missed, and it wakes only for the lock itself. The
///  marker is written once, before the first attempt, so the attempts themselves create and remove nothing in the watched
///  directory. When the limit passes, nothing was changed; the clients learn who held the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireIndexLock(index, marker, limit, announce)
/// ```
///
/// # Errors
///
/// `LockFailure` with `IndexInUse` when the lock stayed held to the limit (or cannot be waited for), and with the storage
///  code when the lock cannot be created.
pub fn acquire(index: &Path, seq: u64, limit: Duration, announce: &dyn Fn(Announce)) -> Result<IndexLock, LockFailure> {
    return acquire_noting_holder(index, seq, limit, announce, &mut None);
}

/// What:
///  `acquire`,
/// leaving the words for the holder the evidence named in `holder` while the lock was waited for.
/// Why:
///   The landing's log line names who held the index when a commit waited for it,
/// also when the wait ended in a success,
///  which `acquire` alone reports to no one (the clients are told only when the wait fails).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireIndexLock(index, marker, limit, announce, (who) => { holder = who })
/// ```
///
/// # Errors
///
/// As `acquire`.
pub fn acquire_noting_holder(index: &Path, seq: u64, limit: Duration, announce: &dyn Fn(Announce), holder: &mut Option<String>) -> Result<IndexLock, LockFailure> {
    return acquire_placing(&SYSTEM, index, seq, limit, announce, holder);
}

/// `acquire` with the calls of `platform`.
///
/// # Errors
///
/// As `acquire`;
/// the log of the service says why a lock that could not be created or removed stays,
/// and why a process with no birth identity takes none.
pub fn acquire_on(platform: &Platform, index: &Path, seq: u64, limit: Duration, announce: &dyn Fn(Announce)) -> Result<IndexLock, LockFailure> {
    return acquire_placing(platform, index, seq, limit, announce, &mut None);
}

/// `acquire_noting_holder` with the calls of `platform`.
fn acquire_placing(platform: &Platform, index: &Path, seq: u64, limit: Duration, announce: &dyn Fn(Announce), holder: &mut Option<String>) -> Result<IndexLock, LockFailure> {
    let lock: PathBuf = lock_path_of(index);
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let lock_name: std::ffi::OsString = lock.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
    let mut watch: Option<Watch> = Watch::on(directory);
    let marker: String = match marker_text(seq, platform.identity) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("cli-git-service: cannot take the index lock {}: this process has no birth identity to name in the lock ({error})", lock.display());
            return Err(LockFailure { code: FailureCode::Unavailable });
        }
    };
    let staged: PathBuf = stage_marker(&lock, seq, marker.as_str()).map_err(|error| {
        log_placement_failure(&lock, &error);
        return LockFailure { code: super::service_land_follow::storage_failure(&error) };
    })?;
    let outcome: Result<(), LockFailure> = wait_and_place(platform, &Placing { staged: &staged, lock: &lock, lock_name: lock_name.as_os_str(), index, limit }, &mut watch, announce, holder);
    let _removed = std::fs::remove_file(&staged);
    outcome?;
    remove_dead_staging(&lock);
    return Ok(IndexLock { lock, index: index.to_path_buf(), marker, armed: true });
}

impl IndexLock {
    /// Whether the lock at this path still holds exactly the text this holder placed.
    fn still_holds_marker(&self) -> bool {
        let mut text: Vec<u8> = Vec::new();
        let limit: u64 = u64::try_from(self.marker.len()).unwrap_or(u64::MAX).saturating_add(1);
        return std::fs::File::open(&self.lock).and_then(|file| { return file.take(limit).read_to_end(&mut text); }).is_ok() && text == self.marker.as_bytes();
    }

    /// What:
    ///  Installs `built`, which sits in the index's own directory, as the index by one rename, then gives the lock up.
    /// Why:
    ///   A reader sees the old index or the new one and never a part of either, as with Git's rename of its lock. The lock,
    ///  still marked, is removed only after the rename, so a kill between the two leaves a lock the next instance knows as the
    ///  service's own. Copying the new index over the lock first left a lock without the marker for as long as the copy ran,
    ///  and a kill then left a lock no restart would take over.
    /// A lock that no longer holds this holder's marker was removed and replaced by another program
    /// (a person deleting it,
    /// a second service):
    /// the index is then not installed and the other program's lock is not touched,
    /// so the two never both believe they hold it without one of them failing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lock.releaseWith(built) // rename(built, index); unlink(lock)
    /// ```
    ///
    /// # Errors
    ///
    /// The storage failure when the new index cannot be put in place;
    /// the lock is then removed and the old index stays.
    /// An error naming the lock when it no longer holds this holder's marker;
    /// it and the old index stay.
    pub fn release_with(mut self, built: &Path) -> Result<(), std::io::Error> {
        if !self.still_holds_marker() {
            self.armed = false;
            let message: String = format!(
                "the index lock {} no longer holds this service's marker: another program removed or replaced it, so the index was not replaced and the lock was left to that program",
                self.lock.display()
            );
            eprintln!("cli-git-service: {message}");
            return Err(std::io::Error::other(message));
        }
        // No index existed and nothing was staged when `built` is absent: there is nothing to install.
        if built.exists() {
            std::fs::rename(built, &self.index)?;
        }
        std::fs::remove_file(&self.lock)?;
        self.armed = false;
        return Ok(());
    }
}

impl Drop for IndexLock {
    fn drop(&mut self) {
        if self.armed && self.still_holds_marker() {
            let _removed = std::fs::remove_file(&self.lock);
        }
    }
}

/// What:
///  Removes an index lock left by a service that was killed, when it still carries the service's marker for `seq` and its
///  holder is gone.
/// Why:
///   Only the marker proves the lock is ours; a foreign lock, one a Git holds, or one whose holder still runs is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeStaleLock(index, seq)
/// ```
pub fn remove_stale(index: &Path, seq: u64) {
    let lock: PathBuf = lock_path_of(index);
    if marker_of(lock.as_path()).is_some_and(|marked| { return marked.seq == seq && holder_is_gone(marked.holder.as_ref()); }) {
        let _removed = std::fs::remove_file(&lock);
    }
}

/// Index lock controls stay out of the release executable.
#[cfg(test)]
#[path = "service_index_lock_tests.rs"]
mod tests;
