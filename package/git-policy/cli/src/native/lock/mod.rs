//! What:
//!  Locks and their owners:
//!  the index lock,
//!  stale reference locks,
//!  and the claims that let one command work on a worktree.
//! Why:
//!  Every lock question has one answer in one place.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `holder_scan_evidence.rs`:
//!   what one scan of a lock's holders reports:
//!   the mechanism it read,
//!   the processes holding the file open,
//!   and the reasons it could not see a process or identify a descriptor.
//! - `git_command_name.rs`:
//!   whether a process's command name is Git's,
//!   shared by both lock judgements.
//! - `index_lock_holder_parsers.rs`:
//!   parsers of `lsof -F pcDi` records and Restart Manager lines,
//!   matched by device and inode.
//! - `index_lock_holder_words.rs`:
//!   the sentences about a lock's holder:
//!   its program,
//!   start time and the way to end it,
//!   and a summary of unreadable processes.
//! - `index_lock_evidence_words.rs`:
//!   the sentences a lock's evidence is reported in:
//!   the line naming a holder,
//!   the line at a wait's ceiling,
//!   and the diagnostic for a PID number this process table does not have.
//! - `index_lock_holders_accounts.rs`:
//!   which account owns one process of the scan,
//!   which tells a holder this account cannot read from another account's.
//! - `proc_fd_holder_scan.rs`:
//!   the `/proc/<pid>/fd` scan for an open descriptor on one file,
//!   newest process number first,
//!   and the limit that cuts it short.
//! - `index_lock_holders.rs`:
//!   processes holding a lock file open,
//!   found by device and inode.
//! - `index_lock_evidence.rs`:
//!   evidence about a foreign `index.lock` and the verdict it supports.
//! - `index_wait_outcome.rs`:
//!   what came of one wait:
//!   not held,
//!   released,
//!   a gone owner,
//!   the ceiling,
//!   or another command's holder.
//! - `index_lock_looks.rs`:
//!   the looks at `index.lock` that do not wait and what each proves,
//!   with the sentences their lines are written in.
//! - `index_holder_wait.rs`:
//!   the wait for any program's hold on `index.lock` before a command changes the checkout.
//! - `index_lock_wait.rs`:
//!   waiting for a foreign index.lock under the classification rules and Git's backoff schedule.
//! - `index_writer_commands.rs`:
//!   index-writer classification and the commands that coordinate with a landing.
//! - `index_writer_forward.rs`:
//!   running a coordinated forwarded index writer:
//!   the landing lock held while real Git runs as a child.
//! - `lock_birth_times.rs`:
//!   when a lock and its PID file were born,
//!   with the arm a control hides or coarsens.
//! - `lock_holder_records.rs`:
//!   the record a command writes about the processes it starts in a repository,
//!   and the proof that one of them is gone.
//! - `lock_holder_record_reads.rs`:
//!   what the wrapper's records of the processes it started say about a lock's holder.
//! - `single_owner_claim_record.rs`:
//!   the text of one claim file and the record it reads back as.
//! - `single_owner_claim_files.rs`:
//!   the numbered claim files one item's claims are kept in:
//!   their generation names,
//!   the private file every write goes through,
//!   and the publishing,
//!   reading and releasing of one.
//! - `single_owner_claim_liveness.rs`:
//!   the owner a claim names and whether that process still runs.
//! - `single_owner_claim.rs`:
//!   the numbered claim files that let exactly one command work on one worktree or checkout,
//!   without a lock.
//! - `stale_ref_lock_names.rs`:
//!   the names a lock and its pid file are written under,
//!   where the pid file sits,
//!   and the process number it names.
//! - `stale_ref_lock_found.rs`:
//!   the record of one judged reference lock and the outcome of a clearing pass.
//! - `stale_ref_lock_identity.rs`:
//!   a file's identity at the moment it was judged,
//!   so the removal can tell it from a new file at the same path.
//! - `stale_ref_lock_judge_environment.rs`:
//!   what one judgement reads of the machine it runs on:
//!   its boot time,
//!   what its process table shows,
//!   and the time a file carries.
//! - `stale_ref_lock_judge_limits.rs`:
//!   the shipped limits a judgement rests on,
//!   with the measurements each was chosen from.
//! - `stale_ref_lock_judge_words.rs`:
//!   the sentences an unproven verdict is reported in:
//!   what the pid file's number is now,
//!   why it proves nothing,
//!   and that the lock looks older than the boot.
//! - `stale_ref_lock_judge.rs`:
//!   how one reference lock is judged.
//! - `stale_ref_lock_judge_staging.rs`:
//!   how the staging file `packed-refs.new` is judged when no lock stands beside it.
//! - `stale_ref_lock_message.rs`:
//!   what the wrapper prints about reference locks,
//!   and the quoted command that clears one it will not remove.
//! - `stale_ref_lock_walk.rs`:
//!   which files below a Git directory are reference locks.
//! - `stale_ref_lock_retire.rs`:
//!   removal of a proven stale lock with its companions.
//! - `stale_ref_locks.rs`:
//!   the reference locks a killed Git left behind:
//!   judged one by one,
//!   removed on a proof,
//!   otherwise described.
//! - `forwarded_lock_notice.rs`:
//!   the notice a forwarded fetch or push is preceded by when reference locks of a killed Git are found.
//! - `push_lock_clearing.rs`:
//!   the wrapper's own fetch and push,
//!   started after the stale reference locks of their repository are cleared.
//! - `stopped_git_locks.rs`:
//!   the lock files a stopped Git child leaves behind,
//!   found and removed once the child and what it started are gone.
//! - `abandoned_lock_proving.rs`:
//!   the proof that one lock file is abandoned,
//!   and the one removal that acts on it.
//! - `abandoned_lock_words.rs`:
//!   the two lines a person reads about a lock:
//!   what was cleared,
//!   and what stands.
//! - `abandoned_locks.rs`:
//!   the pass a command makes over its repository's lock files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // lock/index.ts: one `export * from './file'` per module below.
//! ```

/// Controls for the lock files a stopped Git child must not leave behind.
#[cfg(test)]
mod git_stop_lock_tests;

/// What one scan of a lock's holders reports:
/// the mechanism it read,
/// the processes holding the file open,
/// and the reasons it could not see a process or identify a descriptor.
pub mod holder_scan_evidence;

/// Processes holding a lock file open,
///  found by device and inode.
pub mod index_lock_holders;

/// Which account owns one process of the scan,
///  which tells a holder this account cannot read from another account's.
pub mod index_lock_holders_accounts;

/// The `/proc/<pid>/fd` scan for an open descriptor on one file,
///  newest process number first,
///  and the limit that cuts it short.
#[cfg(unix)]
pub mod proc_fd_holder_scan;

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

/// What came of one wait:
///  not held,
///  released,
///  a gone owner,
///  the ceiling,
///  or another command's holder.
pub mod index_wait_outcome;

/// The looks at `index.lock` that do not wait and what each proves,
///  with the sentences their lines are written in.
pub mod index_lock_looks;

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

/// The names a lock and its pid file are written under,
/// where the pid file sits,
/// and the process number it names.
pub mod stale_ref_lock_names;

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

/// The shipped limits a judgement rests on,
/// with the measurements each was chosen from.
pub mod stale_ref_lock_judge_limits;

/// The sentences an unproven verdict is reported in:
/// what the pid file's number is now,
/// why it proves nothing,
/// and that the lock looks older than the boot.
pub mod stale_ref_lock_judge_words;

/// How the staging file `packed-refs.new` is judged when no lock stands beside it.
pub mod stale_ref_lock_judge_staging;

/// Removal of a proven stale lock with its companions.
pub mod stale_ref_lock_retire;

/// The wrapper's own fetch and push,
/// started after the stale reference locks of their repository are cleared.
pub mod push_lock_clearing;

/// The notice a forwarded fetch or push is preceded by when reference locks of a killed Git are found.
pub mod forwarded_lock_notice;

/// When a lock and its PID file were born,
/// with the arm a control hides or coarsens.
pub mod lock_birth_times;

/// What a read of the abandoned lock's PID file reports,
/// with the arm a control changes one term of.
#[cfg(test)]
#[path = "lock_metadata_arm.rs"]
pub(crate) mod lock_metadata_arm;

/// The record a command writes about the processes it starts in a repository,
/// and the proof that one of them is gone.
pub mod lock_holder_records;

/// What the wrapper's records of the processes it started say about a lock's holder.
pub mod lock_holder_record_reads;

/// The proof that one lock file is abandoned,
/// and the one removal that acts on it.
pub mod abandoned_lock_proving;

/// The two lines a person reads about a lock:
/// what was cleared,
/// and what stands.
pub mod abandoned_lock_words;

/// Controls of the records a command writes of itself and its children.
#[cfg(test)]
#[path = "lock_holder_records_tests.rs"]
mod lock_holder_records_tests;

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

/// The disposable repository a control builds so a real Git takes a reference lock in it,
/// for the controls.
#[cfg(test)]
pub(crate) mod stale_ref_lock_stage;

/// The text of one claim file and the record it reads back as.
#[cfg(target_os = "linux")]
pub mod single_owner_claim_record;

/// The numbered claim files one item's claims are kept in:
/// their generation names,
/// the private file every write goes through,
/// and the publishing,
/// reading and releasing of one.
#[cfg(target_os = "linux")]
pub mod single_owner_claim_files;

/// The owner a claim names and whether that process still runs.
#[cfg(target_os = "linux")]
pub mod single_owner_claim_liveness;

/// The numbered claim files that let exactly one command work on one worktree or checkout,
/// without a lock.
#[cfg(target_os = "linux")]
pub mod single_owner_claim;
