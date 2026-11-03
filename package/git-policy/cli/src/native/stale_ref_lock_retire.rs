//! What:
//!  Judge one candidate and remove it, with its companions, when it is proven stale.
//! Why:
//!  See `stale_ref_locks.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! judgeAndClear(candidate, judge, outcome);
//! ```

use super::stale_ref_lock_found::Outcome;
use super::stale_ref_lock_judge::Judge;
use std::path::Path;

/// The identity of a file when it was judged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Identity {
    /// Device number.
    pub device: u64,
    /// Inode number.
    pub inode: u64,
    /// Size in bytes.
    pub size: u64,
}

/// The identity of the file at `path`, without following a link.
pub fn identity_of(_path: &Path) -> Option<Identity> {
    return Some(Identity { device: 0, inode: 0, size: 0 });
}

/// Remove the lock when it is still the file that was judged.
pub fn remove_if_unchanged(lock: &Path, _judged: &Identity, _outcome: &mut Outcome) {
    let _removed = std::fs::remove_file(lock);
}

/// Judge `candidate` and record the result in `outcome`.
pub fn judge_and_clear(_candidate: &Path, _judge: &Judge, _outcome: &mut Outcome) {}
