//! What:
//!  The service's hold on Git's own `index.lock` across a landing:
//! taken first,
//! by exclusive creation,
//! waited for without
//!  polling when another program holds it,
//! and released by moving the new index over the real one.
//! Why:
//!  Git's own `commit` takes the index lock before it moves the branch and gives it up after the index is written.
//! A landing
//!  that moved the branch first and took the lock afterwards left the index showing content a landed commit had replaced for
//!  as long as a foreign `git` held the lock,
//! and a commit prepared from that index could land a revert.
//! The wait for a
//!  foreign holder is event-driven:
//! an inotify watch on the Git directory wakes the service when the lock itself is removed or
//!  moved away,
//! and the only timer is the repetition of the holder's notice.
//! On
//!  platforms without inotify a held lock fails at once with the plain message.
//! The lock carries a marker with the batch
//!  number and the holding process from the moment it exists until it is removed after the new index is in place,
//! so a
//!  restart can tell its own stale lock from a foreign one wherever the service was killed,
//! and never takes over a lock
//!  whose holder still runs (`doc/planning/cli-git-resident-service.md`,
//! "Landing every command line").
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

use crate::service_index_attempts::{Placing, wait_and_place};
use crate::service_index_claim::{abandoned_here, keep_abandoned};
use crate::service_index_prestage::{Slot, put_slots, take_carrying, take_slots};
use crate::service_index_refused_lock::{Refused, keep_remembered, remembered_here};
use crate::wait_ceilings::LANDING_LOCK_CEILING;
use crate::service_protocol::FailureCode;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::service_index_prestage::Prestaged;
pub use crate::service_index_found::{WaitNotes, describe_lock};
pub use crate::service_index_hold::{IndexLock, Record};
pub use crate::service_index_marker::{LOCK_MARKER, Marked, marker_text, parse_marker};
pub use crate::service_index_ownership::{Ownership, inspect_ownership, unsettled_locks};
#[cfg(target_os = "linux")]
pub use crate::service_index_watch::{events_end_watch, events_name};
pub use crate::service_index_placement::{Placed, Platform, SYSTEM, placement_failure_text};
pub use crate::service_index_staging::{STAGED_INFIX, staged_path_for};
pub use crate::service_index_watch::Watch;
use crate::service_index_placement::{log_placement_failure, place_lock, reclaim};
pub use crate::service_index_reclaim::{Reclaimed, foreign_words, remove_stale, stale_lock_words, unjudged_words};
use crate::service_index_staging::{refresh_age, remove_dead_staging, stage_marker_synced};

/// How often the holder's notice is repeated while the wait lasts.
///  Exempt from a citation under `LQM` (class:
/// notice pacing):
/// it times a line the person reads or a log threshold,
/// and
///  nothing waits on it beyond the wait that already exists.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const NOTICE_EVERY: Duration = Duration::from_secs(10);

/// The environment variable that sets how often the holder's line is repeated,
/// in whole seconds.
pub const NOTICE_VARIABLE: &str = "CLI_GIT_WAIT_NOTICE_SECONDS";

/// How often the holder's line is repeated:
/// the variable when it holds a positive number of seconds,
/// ten seconds otherwise.
pub(crate) fn notice_every() -> Duration {
    return notice_interval(std::env::var(NOTICE_VARIABLE).ok().as_deref());
}

/// The interval a setting of `CLI_GIT_WAIT_NOTICE_SECONDS` stands for:
/// a positive whole number of seconds,
/// else ten seconds.
/// Why:
///   Zero would repeat the holder's line without a pause.
pub(crate) fn notice_interval(setting: Option<&str>) -> Duration {
    // A setting that is missing, is not a number or is zero leaves the default.
    let seconds: Option<u64> = match setting {
        Some(text) => text.trim().parse::<u64>().ok(),
        None => None,
    };
    return match seconds {
        Some(found) if found > 0 => Duration::from_secs(found),
        _ => NOTICE_EVERY,
    };
}

/// What the wait tells the clients of the batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Announce {
    /// A line for the person's terminal:
    /// who holds the lock and how long the wait lasts.
    Waiting(String),
    /// Who held the lock when the wait ran out,
    /// in words that fit "... is using this repository's index".
    Holder(String),
}

/// What:
///  Where the wait for the index lock tells the clients of the batch what it waits for.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The wait and the landing take the announcer as a value,
/// so a control can stand in for it;
/// the state the announcement needs
///  (the streams of the batch) is a small struct that implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Announcer { announce(note: Announce): void }
/// ```
pub trait Announcer {
    /// Tell the clients `note`.
    fn announce(&self, note: Announce);
}

/// Why the lock was not taken.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockFailure {
    /// The code the clients are answered with.
    pub code: FailureCode,
}

/// What:
///  One attempt to take the lock,
/// with no wait.
/// Why:
///   Finishing a batch a killed service left open must not hold up the next batch for the whole limit;
/// a foreign holder,
/// or
///  a service holder that still runs,
/// leaves the batch open for the next round.
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
            crate::service_error!("cannot take the index lock {}: this process has no birth identity to name in the lock ({error})", lock.display());
            return None;
        }
    };
    let (staged, held) = match stage_marker_synced(platform.sync, &lock, seq, marker.as_str()) {
        Ok(made) => made,
        Err(error) => {
            log_placement_failure(&lock, &error);
            return None;
        }
    };
    refresh_age(&held);
    let attempt: std::io::Result<Placed> = match place_lock(platform, &staged, marker.as_str(), &lock) {
        // A stale lock removed (or replaced by then) is worth one more placement; a lock another reclaimer is removing, a live
        //  one, and one that cannot be removed leave the batch open for the next round.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => match reclaim(platform, &lock) {
            Reclaimed::Removed | Reclaimed::Changed => place_lock(platform, &staged, marker.as_str(), &lock),
            Reclaimed::Busy | Reclaimed::Kept | Reclaimed::Stuck | Reclaimed::Unguarded(_) | Reclaimed::Foreign { .. } | Reclaimed::Unjudged { .. } => Err(error),
        },
        other => other,
    };
    let _removed = std::fs::remove_file(&staged);
    let placed: Placed = match attempt {
        Ok(found) => found,
        Err(error) => {
            log_placement_failure(&lock, &error);
            return None;
        }
    };
    remove_dead_staging(&lock);
    return Some(IndexLock::placed(lock, index, marker, placed.held_or(held), (platform.inspect, platform.release)));
}

/// The path of the lock of an index.
pub fn lock_path_of(index: &Path) -> PathBuf {
    let mut name = index.as_os_str().to_os_string();
    name.push(".lock");
    return PathBuf::from(name);
}

/// What:
///  Takes `index.lock` by exclusive creation,
/// waiting for a foreign holder until it lets go.
/// Why:
///   Git's own protocol:
/// creation fails while any process holds the lock.
/// The wait blocks on an inotify watch set before the
///  first attempt,
/// so a release between the attempt and the wait is not missed,
/// and it wakes only for the lock itself.
/// The
///  marker is written once,
/// before the first attempt,
/// so the attempts themselves create and remove nothing in the watched
///  directory.
/// A holder is waited for until it lets go or `LANDING_LOCK_CEILING` passes,
/// and a dead owner (a PID file naming a process that is gone) ends the wait at once with
///  nothing changed;
/// the clients learn who held the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireIndexLock(index, marker, announce)
/// ```
///
/// # Errors
///
/// `LockFailure` with `IndexInUse` when the lock's owner is dead,
/// and with the storage
///  code when the lock cannot be created.
pub fn acquire(index: &Path, seq: u64, announce: &dyn Announcer) -> Result<IndexLock, LockFailure> {
    return acquire_noting_holder(index, seq, announce, &mut None);
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
/// await acquireIndexLock(index, marker, announce, (who) => { holder = who })
/// ```
///
/// # Errors
///
/// As `acquire`.
pub fn acquire_noting_holder(index: &Path, seq: u64, announce: &dyn Announcer, holder: &mut Option<String>) -> Result<IndexLock, LockFailure> {
    let mut notes: WaitNotes = WaitNotes::default();
    let mut memory: TheLockWaitsMemory = carry_the_lock_waits_memory();
    let taken: Result<IndexLock, LockFailure> = acquire_placing(&SYSTEM, index, seq, LANDING_LOCK_CEILING, announce, &mut notes, None, &mut memory);
    keep_the_lock_waits_memory(memory);
    holder.clone_from(&notes.holder);
    return taken;
}

/// What:
///  `acquire`,
/// leaving in `notes` what the wait was for:
/// the holder's words,
/// what the first and the last look at the held lock found,
/// and how many looks there were.
/// Why:
///   The landing's log says who held the index when a commit waited for it,
/// whatever the wait ended in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireIndexLock(index, marker, announce, notes)
/// ```
///
/// # Errors
///
/// As `acquire`.
pub fn acquire_noting_wait(index: &Path, seq: u64, announce: &dyn Announcer, notes: &mut WaitNotes) -> Result<IndexLock, LockFailure> {
    return acquire_noting_wait_within(index, seq, LANDING_LOCK_CEILING, announce, notes);
}

/// `acquire_noting_wait` that gives up on a foreign holder after `ceiling`.
///
/// # Errors
///
/// As `acquire`,
/// and `IndexInUse` when the lock was still held at the ceiling.
pub fn acquire_noting_wait_within(index: &Path, seq: u64, ceiling: Duration, announce: &dyn Announcer, notes: &mut WaitNotes) -> Result<IndexLock, LockFailure> {
    let mut memory: TheLockWaitsMemory = carry_the_lock_waits_memory();
    let taken: Result<IndexLock, LockFailure> = acquire_carrying(index, seq, ceiling, announce, notes, &mut memory);
    keep_the_lock_waits_memory(memory);
    return taken;
}

/// What:
///  The landing's own memory of a lock wait:
/// the markers made ahead it may still take,
/// the foreign lock an earlier commit gave up on,
/// and the claims it placed whose answer could not be read back.
/// Why:
///   Every one of them is the landing's,
/// kept in its task's state
/// (`landing_state_carried_by_its_task.rs`),
/// and a job of the blocking pool cannot read or write that state.
/// So the task that asks for the lock carries this into the job and keeps what the job answers,
/// and the acquisition is one implementation for both callers
/// (`acquire_carrying` is the job's,
/// `acquire_noting_wait_within` the caller's).
/// What a memory that is lost costs is its own:
/// a marker made ahead is staged cold again,
/// a memory of a refused lock leaves the next commit a full wait for the same file,
/// and a claim that is lost stays beside the lock until a later reclaim judges it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const memory = carryTheLockWaitsMemory(); const held = acquireCarrying(index, seq, ceiling, announce, notes, memory); keepTheLockWaitsMemory(memory)
/// ```
#[derive(Default)]
pub(crate) struct TheLockWaitsMemory {
    /// The markers made ahead,
    /// one for each batch after the one that made them.
    pub(crate) prestaged: Vec<Slot>,
    /// The foreign lock the last commit gave up on,
    /// with how long the commits that waited for it waited in all.
    pub(crate) refused: Option<Refused>,
    /// The claims placed beside a stale lock whose answer could not be read back,
    /// with the marker text each was written from.
    pub(crate) abandoned: Vec<(PathBuf, String)>,
}

/// What:
///  Takes the landing's memory of its lock waits out of its state.
/// Why:
///   The state belongs to the task,
/// and the job that waits for the lock is another thread;
/// the memory travels between them as a value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const memory = { prestaged: takeSlots(), refused: rememberedHere(), abandoned: abandonedHere() }
/// ```
pub(crate) fn carry_the_lock_waits_memory() -> TheLockWaitsMemory {
    return TheLockWaitsMemory { prestaged: take_slots(), refused: remembered_here(), abandoned: abandoned_here() };
}

/// What:
///  Keeps what a lock wait answered as the landing's memory.
/// Why:
///   The caller is the landing's task,
/// which is the one owner of its state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepTheLockWaitsMemory(memory)
/// ```
pub(crate) fn keep_the_lock_waits_memory(memory: TheLockWaitsMemory) {
    put_slots(memory.prestaged);
    keep_remembered(memory.refused);
    keep_abandoned(memory.abandoned);
}

/// What:
///  `acquire_noting_wait_within` with the landing's memory carried rather than read from its state,
/// which is what a job of the blocking pool calls.
/// Why:
///   The wait for a foreign `index.lock` is what a landing must not do on a worker,
/// and everything it remembers between waits is the landing's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = acquireCarrying(index, seq, ceiling, announce, notes, memory)
/// ```
///
/// # Errors
///
/// As `acquire_noting_wait_within`.
pub(crate) fn acquire_carrying(index: &Path, seq: u64, ceiling: Duration, announce: &dyn Announcer, notes: &mut WaitNotes, memory: &mut TheLockWaitsMemory) -> Result<IndexLock, LockFailure> {
    // The whole of the preparation counts as staging time: taking the marker made ahead, removing the markers it cannot use,
    //  and the cold staging that follows; `acquire_placing` adds its own cold staging to this.
    let preparing: std::time::Instant = std::time::Instant::now();
    let prepared: Option<Prestaged> = take_carrying(&mut memory.prestaged, index, seq);
    notes.staging = preparing.elapsed();
    return acquire_placing(&SYSTEM, index, seq, ceiling, announce, notes, prepared, memory);
}

/// `acquire` with the calls of `platform`.
///
/// # Errors
///
/// As `acquire`;
/// the log of the service says why a lock that could not be created or removed stays,
/// and why a process with no birth identity takes none.
pub fn acquire_on(platform: &Platform, index: &Path, seq: u64, announce: &dyn Announcer) -> Result<IndexLock, LockFailure> {
    let mut memory: TheLockWaitsMemory = carry_the_lock_waits_memory();
    let taken: Result<IndexLock, LockFailure> = acquire_placing(platform, index, seq, LANDING_LOCK_CEILING, announce, &mut WaitNotes::default(), None, &mut memory);
    keep_the_lock_waits_memory(memory);
    return taken;
}

/// `acquire_on` that gives up on a foreign holder after `ceiling`,
/// leaving in `notes` what the wait was for.
///
/// # Errors
///
/// As `acquire_on`,
/// and `IndexInUse` when the lock was still held at the ceiling.
pub fn acquire_within(platform: &Platform, index: &Path, seq: u64, ceiling: Duration, announce: &dyn Announcer) -> Result<IndexLock, LockFailure> {
    let mut memory: TheLockWaitsMemory = carry_the_lock_waits_memory();
    let taken: Result<IndexLock, LockFailure> = acquire_placing(platform, index, seq, ceiling, announce, &mut WaitNotes::default(), None, &mut memory);
    keep_the_lock_waits_memory(memory);
    return taken;
}

/// `acquire_noting_holder` with the calls of `platform`.
fn acquire_placing(
    platform: &Platform,
    index: &Path,
    seq: u64,
    ceiling: Duration,
    announce: &dyn Announcer,
    notes: &mut WaitNotes,
    prestaged: Option<Prestaged>,
    memory: &mut TheLockWaitsMemory,
) -> Result<IndexLock, LockFailure> {
    let lock: PathBuf = lock_path_of(index);
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let lock_name: std::ffi::OsString = lock.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
    let mut watch: Option<Watch> = (platform.watch)(directory);
    let (marker, staged, held) = match prestaged {
        Some(made) => {
            // The file was flushed while the service was idle: what was waited for is what the preparation measured.
            notes.stage_kind = "pre";
            (made.marker, made.staged, made.held)
        }
        None => {
            let marker: String = match marker_text(seq, platform.identity) {
                Ok(text) => text,
                Err(error) => {
                    crate::service_error!("cannot take the index lock {}: this process has no birth identity to name in the lock ({error})", lock.display());
                    return Err(LockFailure { code: FailureCode::Unavailable });
                }
            };
            let staging_began: std::time::Instant = std::time::Instant::now();
            let staged_result = stage_marker_synced(platform.sync, &lock, seq, marker.as_str());
            notes.staging += staging_began.elapsed();
            notes.stage_kind = "cold";
            let (staged, held) = match staged_result {
                Ok(made) => made,
                Err(error) => {
                    log_placement_failure(&lock, &error);
                    return Err(LockFailure { code: crate::service_land_follow::storage_failure(&error) });
                }
            };
            (marker, staged, held)
        }
    };
    let placing_began: std::time::Instant = std::time::Instant::now();
    let outcome: Result<Placed, LockFailure> = wait_and_place(
        platform,
        &Placing { staged: &staged, held: &held, marker: marker.as_str(), lock: &lock, lock_name: lock_name.as_os_str(), index, ceiling },
        &mut watch,
        announce,
        notes,
        memory,
    );
    notes.placing = placing_began.elapsed();
    let _removed = std::fs::remove_file(&staged);
    let placed: Placed = outcome?;
    remove_dead_staging(&lock);
    return Ok(IndexLock::placed(lock, index, marker, placed.held_or(held), (platform.inspect, platform.release)));
}

/// Controls for the removal of a stale lock stay out of the release executable.
#[cfg(test)]
#[path = "service_index_reclaim_tests.rs"]
mod reclaim_tests;

/// Controls for the claim chain stay out of the release executable.
#[cfg(test)]
#[path = "service_index_claim_tests.rs"]
mod claim_tests;

/// Controls for the machine ID in a marker stay out of the release executable.
#[cfg(test)]
#[path = "service_index_machine_tests.rs"]
mod machine_tests;

/// Controls for the lock taken by exclusive creation stay out of the release executable.
#[cfg(test)]
#[path = "service_index_lock_by_creation_tests.rs"]
mod creation_tests;

/// Index lock controls stay out of the release executable.
#[cfg(test)]
#[path = "service_index_lock_tests.rs"]
mod tests;
