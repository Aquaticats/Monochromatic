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
//!
//! The one attempt without a wait is `service_index_lock_without_a_wait.rs`,
//! the attempt that waits for a foreign holder is `service_index_lock_with_a_wait.rs`,
//! and the marker,
//! the watch,
//! the placement,
//! and the attempts and the wait are the files named above;
//! this file holds the faces the landing and the wrapper's own index writers call.

use crate::service_index_lock_with_a_wait::acquire_placing;
pub use crate::service_index_lock_without_a_wait::{
    try_acquire,
    try_acquire_on,
};
/// The failure code the controls of this hold assert on.
#[cfg(test)]
use crate::service_protocol::FailureCode;
use crate::wait_ceilings::LANDING_LOCK_CEILING;
use std::path::Path;
/// The path type the controls of this hold build their fixtures with.
#[cfg(test)]
use std::path::PathBuf;
use std::time::Duration;

pub use crate::service_index_found::{
    WaitNotes,
    describe_lock,
};
pub use crate::service_index_hold::{
    IndexLock,
    Record,
};
pub use crate::service_index_marker::{
    LOCK_MARKER,
    Marked,
    marker_text,
    parse_marker,
};
pub use crate::service_index_ownership::{
    Ownership,
    inspect_ownership,
    unsettled_locks,
};
pub use crate::service_index_placement::{
    Placed,
    Platform,
    SYSTEM,
    placement_failure_text,
};
pub use crate::service_index_prestage::Prestaged;
use crate::service_index_prestage::take;
pub use crate::service_index_reclaim::{
    Reclaimed,
    foreign_words,
    reclaim,
    remove_stale,
    stale_lock_words,
    unjudged_words,
};
pub use crate::service_index_staging::{
    STAGED_INFIX,
    staged_path_for,
};
pub use crate::service_index_watch::Watch;
#[cfg(target_os = "linux")]
pub use crate::service_index_watch::{
    events_end_watch,
    events_name,
};
#[cfg(test)]
pub(crate) use crate::the_names_of_the_index_lock::notice_interval;
pub use crate::the_names_of_the_index_lock::{
    Announce,
    Announcer,
    LockFailure,
    NOTICE_EVERY,
    NOTICE_VARIABLE,
    lock_path_of,
    storage_failure,
};

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
pub fn acquire(
    index: &Path,
    seq: u64,
    announce: &dyn Announcer,
) -> Result<IndexLock, LockFailure> {
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
pub fn acquire_noting_holder(
    index: &Path,
    seq: u64,
    announce: &dyn Announcer,
    holder: &mut Option<String>,
) -> Result<IndexLock, LockFailure> {
    let mut notes: WaitNotes = WaitNotes::default();
    let taken: Result<IndexLock, LockFailure> = acquire_placing(
        &SYSTEM,
        index,
        seq,
        LANDING_LOCK_CEILING,
        announce,
        &mut notes,
        None,
    );
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
pub fn acquire_noting_wait(
    index: &Path,
    seq: u64,
    announce: &dyn Announcer,
    notes: &mut WaitNotes,
) -> Result<IndexLock, LockFailure> {
    return acquire_noting_wait_within(index, seq, LANDING_LOCK_CEILING, announce, notes);
}

/// `acquire_noting_wait` that gives up on a foreign holder after `ceiling`.
///
/// # Errors
///
/// As `acquire`,
/// and `IndexInUse` when the lock was still held at the ceiling.
pub fn acquire_noting_wait_within(
    index: &Path,
    seq: u64,
    ceiling: Duration,
    announce: &dyn Announcer,
    notes: &mut WaitNotes,
) -> Result<IndexLock, LockFailure> {
    // The whole of the preparation counts as staging time: taking the marker made ahead, removing the markers it cannot use,
    //  and the cold staging that follows; `acquire_placing` adds its own cold staging to this.
    let preparing: std::time::Instant = std::time::Instant::now();
    let prepared: Option<Prestaged> = take(index, seq);
    notes.staging = preparing.elapsed();
    return acquire_placing(&SYSTEM, index, seq, ceiling, announce, notes, prepared);
}

/// `acquire` with the calls of `platform`.
///
/// # Errors
///
/// As `acquire`;
/// the log of the service says why a lock that could not be created or removed stays,
/// and why a process with no birth identity takes none.
pub fn acquire_on(
    platform: &Platform,
    index: &Path,
    seq: u64,
    announce: &dyn Announcer,
) -> Result<IndexLock, LockFailure> {
    return acquire_placing(
        platform,
        index,
        seq,
        LANDING_LOCK_CEILING,
        announce,
        &mut WaitNotes::default(),
        None,
    );
}

/// `acquire_on` that gives up on a foreign holder after `ceiling`,
/// leaving in `notes` what the wait was for.
///
/// # Errors
///
/// As `acquire_on`,
/// and `IndexInUse` when the lock was still held at the ceiling.
pub fn acquire_within(
    platform: &Platform,
    index: &Path,
    seq: u64,
    ceiling: Duration,
    announce: &dyn Announcer,
) -> Result<IndexLock, LockFailure> {
    return acquire_placing(
        platform,
        index,
        seq,
        ceiling,
        announce,
        &mut WaitNotes::default(),
        None,
    );
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
