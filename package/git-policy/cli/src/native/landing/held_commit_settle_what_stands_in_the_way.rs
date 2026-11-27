//! What:
//!  The reference locks that can refuse the branch write of a held commit,
//!  the ones still standing,
//!  whether the write may be tried now,
//!  and whether a settlement may remove one at all.
//! Why:
//!   A live holder lets go within its step,
//!  so a landing waits for one and lands;
//!  a command that merely resumes a held commit does not wait at all,
//!  it looks once and says what stands.
//!  Only the locks that can refuse this write are asked for,
//!  so a lock of another reference never keeps a settlement from trying.
//!  It sits in a file of its own because the settlement was over the limit the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const locks = standingLocks(candidates); if (locks.length === 0) return { ready: true };
//! ```

use crate::held_commit_record::Held;
use std::path::PathBuf;
use std::time::Instant;

/// Whether a settlement may remove a reference lock that the judgement proves stale.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Clearing {
    /// Never remove one:
    /// the caller is a command resuming work,
    /// not a landing.
    Never,
    /// Clear the ones a proof reaches,
    /// as the landing that waits for them does.
    Proved,
}

/// The Git directories whose reference locks can refuse a batch's branch write:
/// the common one holds the references,
/// and a linked worktree keeps its own `HEAD.lock` in its own Git directory.
fn lock_roots(held: &Held) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = vec![held.common_dir.clone()];
    if held.git_dir != held.common_dir {
        roots.push(held.git_dir.clone());
    }
    return roots;
}

/// The locks of `candidates` that stand now.
fn standing_locks(candidates: &[crate::stale_ref_lock_walk::Candidate]) -> Vec<PathBuf> {
    let mut locks: Vec<PathBuf> = Vec::new();
    for candidate in candidates {
        if candidate.lock.symlink_metadata().is_ok() {
            locks.push(candidate.lock.clone());
        }
    }
    return locks;
}

/// What:
///  Whether the branch write may be tried now,
///  and the locks in its way when it may not.
/// Why:
///   A live holder lets go within its step,
///  so a landing waits for one and lands;
///  a command that merely resumes a held commit does not wait at all,
///  it looks once and says what stands.
///  Under `Clearing::Proved` this is the landing's own wait with no time left in it,
///  which judges every lock in its way once and clears the ones the proof reaches.
///  Only the locks that can refuse this write are asked for (`locks_that_can_refuse`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const locks = standingLocks(candidates); if (locks.length === 0) return { ready: true }; ... { ready: false, locks }
/// ```
pub(crate) fn readiness(
    held: &Held,
    clearing: Clearing,
) -> Result<(), Vec<PathBuf>> {
    let candidates: Vec<crate::stale_ref_lock_walk::Candidate> =
        crate::stale_ref_lock_walk::locks_that_can_refuse_a_write(
            lock_roots(held).as_slice(),
            held.reference.bytes(),
            crate::held_commit_settle_move::kept_reference(held.seq).as_bytes(),
        );
    if matches!(clearing, Clearing::Proved) {
        let waiting = crate::service_land_reference_lock::wait_out_the_reference_locks_of(
            candidates.as_slice(),
            &crate::stale_ref_locks::Judge::here(),
            Instant::now(),
        );
        if matches!(waiting, crate::service_land_reference_lock::Waiting::Gone) {
            return Ok(());
        }
    }
    let locks: Vec<PathBuf> = standing_locks(candidates.as_slice());
    return if locks.is_empty() { Ok(()) } else { Err(locks) };
}
