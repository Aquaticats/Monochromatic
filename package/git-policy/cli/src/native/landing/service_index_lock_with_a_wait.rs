//! What:
//!  The attempt that takes Git's index lock and waits for a foreign holder:
//!  the watch that wakes it set before the first placement,
//!  the marker written once,
//!  the ceiling the wait ends at,
//!  and what the wait was for.
//! Why:
//!   Git's own protocol:
//!  creation fails while any process holds the lock,
//!  so the wait blocks on an inotify watch set before the first attempt and wakes only for the lock itself,
//!  and a release between the attempt and the wait is not missed.
//!  The marker is written once,
//!  before the first attempt,
//!  so the attempts themselves create and remove nothing in the watched directory,
//!  and the file was flushed while the service was idle when the preparation made it ahead.
//!  Every phase of the wait is timed apart,
//!  so a wait whose whole is far longer than its parts names the phase that took the time.
//!  It sits in a file of its own because the index lock's file was over the limit the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken: IndexLock = acquirePlacing(platform, index, seq, ceiling, announce, notes, prestaged)?;
//! ```

use crate::service_index_attempts::{
    Placing,
    wait_and_place,
};
use crate::service_index_found::WaitNotes;
use crate::service_index_hold::IndexLock;
use crate::service_index_marker::marker_text;
use crate::service_index_placement::{
    Placed,
    Platform,
    log_placement_failure,
};
use crate::service_index_prestage::Prestaged;
use crate::service_index_staging::{
    remove_dead_staging,
    stage_marker_synced,
};
use crate::service_index_watch::Watch;
use crate::service_protocol::FailureCode;
use crate::the_names_of_the_index_lock::{
    Announcer,
    LockFailure,
    lock_path_of,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// `acquire_noting_holder` with the calls of `platform`.
pub(crate) fn acquire_placing(
    platform: &Platform,
    index: &Path,
    seq: u64,
    ceiling: Duration,
    announce: &dyn Announcer,
    notes: &mut WaitNotes,
    prestaged: Option<Prestaged>,
) -> Result<IndexLock, LockFailure> {
    let lock: PathBuf = lock_path_of(index);
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let lock_name: std::ffi::OsString = lock
        .file_name()
        .map(std::ffi::OsStr::to_os_string)
        .unwrap_or_default();
    // The watch and the marker's text are timed apart from the making of the marker, so a wait whose whole is far longer than
    //  its parts names the phase that took the time.
    let watching_began: std::time::Instant = std::time::Instant::now();
    let mut watch: Option<Watch> = (platform.watch)(directory);
    notes.setup += watching_began.elapsed();
    let (marker, staged, held) = match prestaged {
        Some(made) => {
            // The file was flushed while the service was idle: what was waited for is what the preparation measured.
            notes.stage_kind = "pre";
            (made.marker, made.staged, made.held)
        }
        None => {
            let naming_began: std::time::Instant = std::time::Instant::now();
            let marker: String = match marker_text(seq, platform.identity) {
                Ok(text) => text,
                Err(error) => {
                    crate::service_error!(
                        "cannot take the index lock {}: this process has no birth identity to name in the lock ({error})",
                        lock.display()
                    );
                    return Err(LockFailure {
                        code: FailureCode::Unavailable,
                    });
                }
            };
            notes.setup += naming_began.elapsed();
            let staging_began: std::time::Instant = std::time::Instant::now();
            let staged_result = stage_marker_synced(platform.sync, &lock, seq, marker.as_str());
            notes.staging += staging_began.elapsed();
            notes.stage_kind = "cold";
            let (staged, held) = match staged_result {
                Ok(made) => made,
                Err(error) => {
                    log_placement_failure(&lock, &error);
                    return Err(LockFailure {
                        code: crate::service_land_follow::storage_failure(&error),
                    });
                }
            };
            (marker, staged, held)
        }
    };
    let placing_began: std::time::Instant = std::time::Instant::now();
    let outcome: Result<Placed, LockFailure> = wait_and_place(
        platform,
        &Placing {
            staged: &staged,
            held: &held,
            marker: marker.as_str(),
            lock: &lock,
            lock_name: lock_name.as_os_str(),
            index,
            ceiling,
        },
        &mut watch,
        announce,
        notes,
    );
    notes.placing = placing_began.elapsed();
    let removing_began: std::time::Instant = std::time::Instant::now();
    let _removed = std::fs::remove_file(&staged);
    notes.after += removing_began.elapsed();
    let placed: Placed = outcome?;
    remove_dead_staging(&lock);
    return Ok(IndexLock::placed(
        lock,
        index,
        marker,
        placed.held_or(held),
        (platform.inspect, platform.release),
    ));
}
