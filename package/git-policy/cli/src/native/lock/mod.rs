//! What:
//!  Locks and their owners:
//!  the index lock,
//!  stale reference locks,
//!  and the claims that let one command work on a worktree.
//! Why:
//!  Every lock question has one answer in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lock/index.ts: one `export * from './file'` per module below.
//! ```

/// Controls for the lock files a stopped Git child must not leave behind.
#[cfg(test)]
mod git_stop_lock_tests;

/// Processes holding a lock file open,
///  found by device and inode.
pub mod index_lock_holders;

/// Parsers of `lsof -F pcDi` records and Restart Manager lines,
///  matched by device and inode.
pub mod index_lock_holder_parsers;

/// Evidence about a foreign `index.lock` and the verdict it supports.
pub mod index_lock_evidence;

/// Index-writer classification and the commands that coordinate with a landing.
pub mod index_writer_commands;

/// The wait for any program's hold on `index.lock` before a command changes the checkout.
pub mod index_holder_wait;

/// Waiting for a foreign index.lock under the classification rules and Git's backoff schedule.
pub mod index_lock_wait;

/// Running a coordinated forwarded index writer:
///  the landing lock held while real Git runs as a child.
pub mod index_writer_forward;

/// The lock files a stopped Git child leaves behind,
/// found and removed once the child and what it started are gone.
pub mod stopped_git_locks;

/// The reference locks a killed Git left behind:
/// judged one by one,
/// removed on a proof,
/// otherwise described.
pub mod stale_ref_locks;

/// The record of one judged reference lock and the outcome of a clearing pass.
pub mod stale_ref_lock_found;

/// Which files below a Git directory are reference locks.
pub mod stale_ref_lock_walk;

/// A file's identity at the moment it was judged,
/// so the removal can tell it from a new file at the same path.
pub mod stale_ref_lock_identity;

/// How one reference lock is judged.
pub mod stale_ref_lock_judge;

/// Removal of a proven stale lock with its companions.
pub mod stale_ref_lock_retire;

/// The processes that carry a number inside a PID namespace nested in the reader's,
/// which a pid file may name.
pub mod stale_ref_lock_nested_numbers;

/// The wrapper's own fetch and push,
/// started after the stale reference locks of their repository are cleared.
pub mod push_lock_clearing;

/// The notice a forwarded fetch or push is preceded by when reference locks of a killed Git are found.
pub mod forwarded_lock_notice;

/// What the wrapper prints about reference locks,
/// and the quoted command that clears one it will not remove.
pub mod stale_ref_lock_message;

/// Real Git processes killed with `SIGKILL` while they hold reference locks,
/// for the controls.
#[cfg(test)]
pub(crate) mod stale_ref_lock_killed_git;

/// The numbered claim files that let exactly one command work on one worktree or checkout,
/// without a lock.
#[cfg(target_os = "linux")]
pub mod single_owner_claim;
