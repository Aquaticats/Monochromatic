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

/// Which account owns one process of the scan,
///  which tells a holder this account cannot read from another account's.
pub mod index_lock_holders_accounts;

/// Parsers of `lsof -F pcDi` records and Restart Manager lines,
///  matched by device and inode.
pub mod index_lock_holder_parsers;

/// Evidence about a foreign `index.lock` and the verdict it supports.
pub mod index_lock_evidence;

/// Whether a process's command name is Git's,
/// shared by both lock judgements.
pub mod git_command_name;

/// The sentences about a lock's holder:
/// its program,
/// start time and the way to end it,
/// and a summary of unreadable processes.
pub mod index_lock_holder_words;

/// The sentences a lock's evidence is reported in:
/// the line naming a holder,
/// the line at a wait's ceiling,
/// and the diagnostic for a PID number this process table does not have.
pub mod index_lock_evidence_words;

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

/// What one judgement reads of the machine it runs on:
/// its boot time,
/// what its process table shows,
/// and the time a file carries.
pub mod stale_ref_lock_judge_environment;

/// How one reference lock is judged.
pub mod stale_ref_lock_judge;

/// Removal of a proven stale lock with its companions.
pub mod stale_ref_lock_retire;

/// The wrapper's own fetch and push,
/// started after the stale reference locks of their repository are cleared.
pub mod push_lock_clearing;

/// The notice a forwarded fetch or push is preceded by when reference locks of a killed Git are found.
pub mod forwarded_lock_notice;

/// When a lock and its PID file were born, with the arm a control hides or coarsens.
pub mod lock_birth_times;

/// The record a command writes about the processes it starts in a repository,
/// and the proof that one of them is gone.
pub mod lock_holder_records;

/// What the wrapper's records of the processes it started say about a lock's holder.
pub mod lock_holder_record_reads;

/// The proof that one lock file is abandoned,
/// and the one removal that acts on it.
pub mod abandoned_lock_proving;

/// The two lines a person reads about a lock: what was cleared, and what stands.
pub mod abandoned_lock_words;

/// Controls for the proof and the removal.
#[cfg(test)]
#[path = "abandoned_lock_proving_tests.rs"]
mod abandoned_lock_proving_tests;

/// The pass a command makes over its repository's lock files.
pub mod abandoned_locks;

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
