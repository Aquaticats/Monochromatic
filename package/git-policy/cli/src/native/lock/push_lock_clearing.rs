//! What:
//!  The wrapper's own fetch and push, run after the reference locks of a killed Git in their repository have been judged
//!  and the provably stale ones removed.
//! Why:
//!  The commit service pushes unattended, and the reconcile fetch runs inside it, so a lock that a killed Git left blocks
//!  them with nobody there to clear it. Clearing before every start also covers every repetition of a push (each repetition
//!  starts through here), and nothing is waited for. A failed attempt carries the notice, so the person who reads the
//!  failure (or the refusal Git prints and still exits 0 on) finds what was removed, what was left, and the command that clears a lock the wrapper will not remove.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = clearStaleRefLocks(rootsOfCommand(command), judge);
//! const attempt = await runBoundedPushOnce(command);
//! if (attempt.exitCode !== 0) attempt.output += outcome.notice();
//! ```

use crate::auto_push_single_flight::{PushAttempt, SingleFlightError};
use crate::bounded_push::{Patience, run_bounded_push_once};
use crate::diagnostic_log::warn;
use crate::git_directory_roots::roots_of_command;
use crate::stale_ref_lock_judge::Judge;
use crate::stale_ref_locks::clear_stale_ref_locks;
use std::ffi::OsString;
use std::process::Command;
use std::time::Duration;

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// What Git prints when another process holds the lock of a reference it updates, also from a push that exits 0.
const REFERENCE_LOCK_REFUSAL: &str = "cannot lock ref";

/// Run `command` as `run_bounded_push_once` does, after the stale reference locks of its repository are cleared by `judge`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runClearingLocks({ command, patience, grace, environment, judge }): Promise<PushAttempt>;
/// ```
pub fn run_clearing_locks(
    command: Command,
    patience: Patience,
    grace: Duration,
    environment: &[(OsString, OsString)],
    judge: &Judge,
) -> Result<PushAttempt, SingleFlightError> {
    let outcome = clear_stale_ref_locks(roots_of_command(&command).as_slice(), judge);
    let notice: String = outcome.notice();
    let mut attempt: PushAttempt = run_bounded_push_once(command, patience, grace, environment)?;
    // A push that delivered but could not update its tracking reference prints the refusal and exits 0, so the refusal is
    //  looked for as well as the exit status (the service repeats such a push on the same words).
    let failed: bool = attempt.exit_code != 0 || attempt.output.contains(REFERENCE_LOCK_REFUSAL);
    if failed && !notice.is_empty() {
        // The service shows this output, so the notice is not logged a second time.
        attempt.output = if attempt.output.is_empty() { notice.trim_end().to_owned() } else { format!("{}\n{}", attempt.output, notice.trim_end()) };
    } else if !notice.is_empty() {
        warn(TAG, notice.trim_end());
    }
    return Ok(attempt);
}

/// Controls for the clearing before the wrapper's own fetch and push.
#[cfg(test)]
#[path = "push_lock_clearing_tests.rs"]
mod tests;
