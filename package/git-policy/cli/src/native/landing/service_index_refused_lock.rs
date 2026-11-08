//! What:
//!  The service's memory of the one foreign `index.lock` that already outlasted a commit's wait:
//! which file it was and how long that commit waited for it.
//! Why:
//!  Commits of one worktree queue behind each other on the service's one landing thread,
//! and each began its own wait for the
//! same lock:
//! on 2026-11-07 four commits ended after 678 to 1,229 seconds because every one waited a full ceiling behind the one
//! ahead of it for the same file.
//! The lock file is the same file (device,
//! inode and change time agree),
//! so a second wait for it can only repeat the first;
//! the commit behind is refused at once,
//! told how long the lock has already been held against a commit,
//! and run again by the person.
//! A lock that changed (its holder wrote to it) or went away and came back is another lock and is waited for in full.
//! The memory is per thread because one thread lands every batch of a service.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! remember(lock, waited); const earlier = recall(lock); // Duration | undefined
//! ```

use crate::index_lock_evidence::{LockMetadata, read_lock_metadata};
use crate::landing_state_carried_by_its_task::{LandingState, on_the_landing_state};
use std::path::Path;
use std::time::Duration;

/// The lock that outlasted a wait,
/// and how long that wait was.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Refused {
    /// Device,
    /// inode and change time of the lock file.
    identity: LockMetadata,
    /// How long the commits that waited for this file waited in all.
    waited: Duration,
}

/// Keeps `refused` as the lock the landing last gave up on,
/// or nothing.
fn keep_the_refused_lock(state: &LandingState, refused: Option<Refused>) {
    *state.refused_lock.borrow_mut() = refused;
}

/// The lock the landing last gave up on,
/// when one is remembered.
fn refused_lock_remembered(state: &LandingState, (): ()) -> Option<Refused> {
    return *state.refused_lock.borrow();
}

/// What:
///  Remembers that the lock at `lock` outlasted a wait of `waited` in all.
/// Why:
///   The next landing's first look at the same file reads it back with `recall`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remember('/repo/.git/index.lock', 600_000)
/// ```
pub fn remember(lock: &Path, waited: Duration) {
    // A lock that cannot be read now is remembered as nothing: the next wait is then a full one, the safe side.
    let Ok(Some(identity)) = read_lock_metadata(lock) else {
        forget();
        return;
    };
    on_the_landing_state(Some(Refused { identity, waited }), keep_the_refused_lock);
}

/// What:
///  How long earlier commits already waited for the lock at `lock`,
/// when it is the same file that outlasted their wait.
/// Why:
///   A wait for the same file adds nothing to what they found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recall('/repo/.git/index.lock') // 600_000 | undefined
/// ```
pub fn recall(lock: &Path) -> Option<Duration> {
    let Ok(Some(identity)) = read_lock_metadata(lock) else {
        return None;
    };
    let remembered: Refused = on_the_landing_state((), refused_lock_remembered)?;
    return if remembered.identity == identity { Some(remembered.waited) } else { None };
}

/// Forgets the lock this thread gave up on:
/// the lock was taken,
/// or is gone.
pub fn forget() {
    on_the_landing_state(None, keep_the_refused_lock);
}
