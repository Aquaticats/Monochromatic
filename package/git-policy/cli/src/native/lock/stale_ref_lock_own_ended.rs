//! What:
//!  What the wrapper itself knows about the child it stopped:
//! the processes of the tree that have ended,
//! and the locks a member of the tree was seen holding before the stop signalled it.
//! Why:
//!  The judge of reference locks proves a holder gone from the pid file beside a lock and from the process table,
//! and a container's
//! view of the machine gives it neither;
//! what the wrapper saw itself needs no inference about namespaces.
//! The file is apart from the judge so each stays within the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type OwnEnded = { pids: number[]; claimed: ClaimedLock[] };
//! ```

/// Paths of the locks.
use std::path::{Path, PathBuf};
/// The identity of a lock file as the judge reads it.
use crate::stale_ref_lock_identity::Identity;

/// What:
///  The processes the wrapper itself started,
/// stopped and saw end.
/// Why:
///   A lock the wrapper saw a member of the tree hold before it signalled that member is the wrapper's own proof once
///  the member has ended (cross-vendor review of 2026-11-09,
/// finding 2).
/// A number alone proves nothing:
///  a pid file holds the number its writer had in the writer's own PID namespace,
/// and a Git of a sibling namespace that shares the repository may have the same number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnEnded = { pids: number[]; claimed: ClaimedLock[] };
/// ```
#[derive(Clone, Debug)]
pub struct OwnEnded {
    /// The numbers of the processes of the stopped tree that have ended.
    pub pids: Vec<i64>,
    /// The locks a member of the tree was seen holding before the stop signalled it.
    pub claimed: Vec<ClaimedLock>,
}

/// A lock that a member of a stopped tree held when the wrapper stopped it.
/// Why:
///  Git's handler removes a lock's pid file first and the lock second,
/// so a kill that falls between the two takes away the only proof
///  of the holder.
/// The wrapper reads the proof before it signals:
/// the pid file beside the lock,
/// born at or after it,
/// names a member
///  of the tree that is still running.
/// The file is pinned by device,
/// inode and birth,
/// so another Git's lock made at the same path
///  later is never taken for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimedLock {
    /// The lock file.
    pub lock: PathBuf,
    /// Device of the lock when it was claimed.
    pub device: u64,
    /// Inode of the lock when it was claimed.
    pub inode: u64,
    /// Birth of the lock,
    /// where the file system keeps it,
    /// in nanoseconds since the epoch.
    pub born_ns: Option<u128>,
    /// The member of the tree the pid file named.
    pub pid: i64,
}

impl OwnEnded {
    /// The member of the stopped tree that held the lock at `lock` before the stop signalled it,
    /// when that member has ended and the
    ///  lock is still the file that was claimed (same device,
    /// inode and birth).
    pub fn holder_of_claimed(&self, lock: &Path, identity: &Identity) -> Option<i64> {
        for claim in &self.claimed {
            let same_file: bool = claim.lock == lock && claim.device == identity.device && claim.inode == identity.inode && claim.born_ns == identity.born_ns;
            if same_file && self.pids.contains(&claim.pid) {
                return Some(claim.pid);
            }
        }
        return None;
    }
}
