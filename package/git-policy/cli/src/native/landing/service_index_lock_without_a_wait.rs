//! What:
//!  The one attempt to take Git's index lock that never waits:
//!  the marker is written,
//!  the lock is placed by exclusive creation,
//!  a stale lock the proof reaches is removed and the placement tried once more,
//!  and every other holder leaves the batch open for the next round.
//! Why:
//!   Finishing a batch a killed service left open must not hold up the next batch for the whole limit;
//!  a foreign holder,
//!  or a service holder that still runs,
//!  leaves the batch open for the next round,
//!  which is why this face takes no announcement and no ceiling.
//!  It sits in a file of its own because the index lock's file was over the limit the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken: IndexLock | undefined = tryAcquire(index, seq);
//! ```

use crate::service_index_hold::IndexLock;
use crate::service_index_marker::marker_text;
use crate::service_index_placement::{
    Placed,
    Platform,
    SYSTEM,
    log_placement_failure,
    place_lock,
};
use crate::service_index_reclaim::{
    Reclaimed,
    reclaim,
};
use crate::service_index_staging::{
    refresh_age,
    remove_dead_staging,
    stage_marker_synced,
};
use crate::the_names_of_the_index_lock::lock_path_of;
use std::path::{
    Path,
    PathBuf,
};

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
pub fn try_acquire(
    index: &Path,
    seq: u64,
) -> Option<IndexLock> {
    return try_acquire_on(&SYSTEM, index, seq);
}

/// `try_acquire` with the calls of `platform`.
pub fn try_acquire_on(
    platform: &Platform,
    index: &Path,
    seq: u64,
) -> Option<IndexLock> {
    let lock: PathBuf = lock_path_of(index);
    let marker: String = match marker_text(seq, platform.identity) {
        Ok(text) => text,
        Err(error) => {
            crate::service_error!(
                "cannot take the index lock {}: this process has no birth identity to name in the lock ({error})",
                lock.display()
            );
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
    let attempt: std::io::Result<Placed> =
        match place_lock(platform, &staged, marker.as_str(), &lock) {
            // A stale lock removed (or replaced by then) is worth one more placement; a lock another reclaimer is removing, a live
            //  one, and one that cannot be removed leave the batch open for the next round.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                match reclaim(platform, &lock) {
                    Reclaimed::Removed | Reclaimed::Changed => {
                        place_lock(platform, &staged, marker.as_str(), &lock)
                    }
                    Reclaimed::Busy
                    | Reclaimed::Kept
                    | Reclaimed::Stuck
                    | Reclaimed::Unguarded(_)
                    | Reclaimed::Foreign {
                        ..
                    }
                    | Reclaimed::Unjudged {
                        ..
                    } => Err(error),
                }
            }
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
    return Some(IndexLock::placed(
        lock,
        index,
        marker,
        placed.held_or(held),
        (platform.inspect, platform.release),
    ));
}
