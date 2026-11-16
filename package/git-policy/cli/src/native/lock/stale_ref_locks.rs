//! What:
//!  The reference locks a killed Git left behind:
//! found,
//! judged one by one,
//! removed when their holder is proven gone,
//! and
//!  otherwise described with the command that clears them.
//! Why:
//!  A Git killed in the middle of a ref update (a terminal closed mid-fetch,
//! the out-of-memory killer,
//! a reboot) leaves an
//!  empty `<ref>.lock` or `packed-refs.lock`,
//! and every later fetch or push of the same reference then fails with
//!  "Unable to create ... File exists".
//! Git writes no process number into a lock and removes none it did not create,
//! and a
//!  live Git holds its locks with no descriptor open (`refs/packed-backend.c:1254-1261`,
//! `refs/files-backend.c:2888-2897`),
//!  so a lock is removed only on a proof (`SPEC.md`,
//! "Reference locks left by a Git that was killed"):
//! the process its pid
//!  file names is gone or was reused,
//! or the lock predates the boot.
//! Everything else is left alone and reported.
//!  This module extends the sweep of a stopped child's locks (`stopped_git_locks.rs`) to locks no wrapper-started child
//!  explains;
//! it reads files and `/proc` in process and starts no Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = clearStaleRefLocks(rootsOfCommand(command), Judge.here());
//! if (outcome.clearedAny()) await runAgain(command); else process.stderr.write(outcome.notice());
//! ```

/// The Git directories a command works in.
use crate::git_directory_roots::roots_of_command;
/// How a candidate is judged and what a judgement can say.
pub use crate::stale_ref_lock_judge::{Judge, Proof, Verdict};
/// The shapes of a finding and of the outcome of one clearing.
pub use crate::stale_ref_lock_found::{Found, Outcome, PidFile};
/// Candidates are found here.
use crate::stale_ref_lock_walk::ref_lock_candidates;
/// Judging and removing one candidate.
use crate::stale_ref_lock_retire::judge_and_clear;
/// The command whose repository is searched.
use std::process::Command;

/// The candidates of the repository a command will run in,
/// judged,
/// and the provably stale ones removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearStaleRefLocksFor(command: Command, judge: Judge): Outcome;
/// ```
pub fn clear_stale_ref_locks_for(command: &Command, judge: &Judge) -> Outcome {
    return clear_stale_ref_locks(roots_of_command(command).as_slice(), judge);
}

/// What:
///  Judge every reference lock below the given Git directories,
/// remove the ones whose holder is proven gone,
/// and report
///  the rest.
/// Why:
///   One entry for every caller,
/// so the person's forwarded `git fetch`,
/// the commit service's push and the stacked push all
///  decide by the same rule and word their notices the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearStaleRefLocks(roots: string[], judge: Judge): Outcome;
/// ```
pub fn clear_stale_ref_locks(roots: &[std::path::PathBuf], judge: &Judge) -> Outcome {
    return clear_stale_ref_locks_of(ref_lock_candidates(roots).as_slice(), judge);
}

/// What:
///  Judge the given reference locks,
/// remove the ones whose holder is proven gone,
/// and report the rest.
/// Why:
///   A caller that knows which locks can refuse its write judges those and no others:
///  a lock on another reference can refuse nothing it does,
/// and removing one would be a removal nothing here needs.
///  The judgement of each lock and the words for the ones left are the walk's own,
/// so both callers answer the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearStaleRefLocksOf(candidates: Candidate[], judge: Judge): Outcome;
/// ```
pub fn clear_stale_ref_locks_of(candidates: &[crate::stale_ref_lock_walk::Candidate], judge: &Judge) -> Outcome {
    let mut outcome: Outcome = Outcome::default();
    for candidate in candidates {
        judge_and_clear(candidate, judge, &mut outcome);
    }
    return outcome;
}

/// Controls for the clearing are in the test module.
#[cfg(test)]
#[path = "stale_ref_locks_tests.rs"]
mod tests;
