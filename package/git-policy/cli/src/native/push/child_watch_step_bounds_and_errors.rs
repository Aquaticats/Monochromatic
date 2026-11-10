//! What:
//!  How a watched step may be ended and the errors that say it was:
//! the bound a step runs under,
//! the error of a step ended at
//!  its ceiling,
//! the error of a step left running,
//! and the text that names them.
//! Why:
//!  `child_watch.rs` runs the watched steps;
//! callers and tests only need these names to tell the endings apart,
//! and they do
//!  not use the running code,
//! so they sit in a file of their own and `child_watch.rs` re-exports them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class CeilingReached extends Error {} class NotKnown extends Error {}
//! const isNotKnown = (error: Error): boolean => error instanceof NotKnown;
//! ```

use crate::child_class::CHILD_WAIT_VARIABLE;
use crate::output_capture::Capture;
use std::process::Output;
use std::time::Duration;

/// What:
///  The sentence for a watched wait that showed no sign of work for the quiet window.
/// Why:
///   It names what was waited for,
/// the window,
/// where to change it and what to do next.
///  It sits beside the other wordings of a watch's ending,
/// so the synchronous wait and the task that awaits it say
///  the same (`child_watch.rs`,
/// `child_watch_output_watched_on_the_runtime.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} showed no sign of work for ${limit} seconds and was stopped ...`
/// ```
pub fn watched_timeout_text(waiting_for: &str, limit: Duration) -> String {
    return format!(
        "{waiting_for} showed no sign of work for {} seconds and was stopped ({CHILD_WAIT_VARIABLE} changes this). \
         If the step belonged to a commit, the next Git command of this wrapper finishes or undoes that commit, so \
         run the command again. If the step is quiet while it works, raise the window with {CHILD_WAIT_VARIABLE}=<seconds>; \
         if it stalls again, check with `ps` what that process waits for.",
        limit.as_secs()
    );
}

/// What:
///  Why nothing watches a step,
/// in the words the person reads.
/// Why:
///   A step with no watcher is announced by nobody,
/// ended by nobody for being quiet,
/// and has its ceiling carried by
///  the wait itself,
/// so the person has to be told that the reason is a refused thread and what to do about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} has no watcher: the system refused a thread for it, so nothing announces this wait ...`
/// ```
pub fn no_watcher_text(waiting_for: &str) -> String {
    return format!(
        "{waiting_for} has no watcher: the system refused a thread for it, so nothing announces this wait, nothing ends the step \
         for being quiet, and a step with a ceiling is ended at that ceiling by the wait itself. Hint: check the process limit \
         (`ulimit -u`) and the control group's `pids.max`, free some, and run the command again."
    );
}

/// What:
///  How long a watched step is waited for,
/// and what happens at the end.
/// Why:
///   A step that can be run twice is ended at a ceiling (owner,
/// 2026-11-03:
/// "nothing should ever wait forever or indefinitely").
/// A
///  write in place that cannot be run twice is never ended (owner,
/// 2026-11-04:
/// "Never cut a branch write (Recommended)"):
/// the
///  command that started it returns at its own bound plus a short extra wait with an error that says the result is not known,
///  and the step is left running (owner,
/// commit protocol revision 12,
/// "Short extra wait (Recommended)");
/// only the commit service's
///  landing thread,
/// whose clients are told through the protocol,
/// waits for it without a bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StepBound = { kind: 'end-at'; ms } | { kind: 'leave-running'; afterMs; extraMs } | { kind: 'unbounded' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepBound {
    /// The step is ended,
    /// with its whole process tree,
    /// when it has run this long,
    /// whatever it shows.
    EndAt(Duration),
    /// The command stops waiting `after` plus `extra` after the step began and returns an error saying the result is not known;
    ///  the step is left running and reaped when it ends.
    LeaveRunning {
        /// The command's own bound.
        after: Duration,
        /// The short extra wait for the write the step started.
        extra: Duration,
    },
    /// The step is waited for as long as it lives.
    Unbounded,
}

/// The error of a step ended at its ceiling,
/// which tells it apart from a step ended for being quiet:
/// a step that may be run again
///  is run again after the second and not after the first.
#[derive(Debug)]
pub struct CeilingReached(
    /// The sentence that names the step and the ceiling.
    pub String,
);

impl std::fmt::Display for CeilingReached {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.0.as_str());
    }
}

impl std::error::Error for CeilingReached {}

/// The output of a step that ended:
/// its status and what it wrote to the captured files.
pub(crate) fn output_of(status: std::process::ExitStatus, captured: Option<&Capture>) -> Output {
    let Some(files) = captured else {
        return Output { status, stdout: Vec::new(), stderr: Vec::new() };
    };
    let shown: crate::output_capture::Captured = files.snapshot(crate::output_capture::STEP_OUTPUT_LIMIT);
    let mut stderr: Vec<u8> = shown.stderr;
    for note in &shown.notes {
        stderr.extend_from_slice(format!("\nmonochromatic-git-policy-cli: {note}\n").as_bytes());
    }
    // Read once, and a descendant that keeps the files open must not keep the pages of what was read.
    files.discard();
    return Output { status, stdout: shown.stdout, stderr };
}

/// What:
///  The sentence for a step left running:
/// what it was,
/// how long it was waited for,
/// and what to check.
/// Why:
///   The owner's rule for a write the command started and cannot confirm (commit protocol revision 12,
/// "Short extra wait"):
/// exit
///  non-zero saying the result is not known and what to look at.
/// The step is a write in place,
/// so the repository is what to check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} was still running ${after + extra} seconds after it began, and was left running ...`
/// ```
pub fn not_known_text(waiting_for: &str, after: Duration, extra: Duration) -> String {
    return format!(
        "{waiting_for} was still running {} seconds after it began (its bound of {} seconds and {} more for the write it started), so it \
         was left running and not ended: it is not known whether it finished. Check what it changes with `git status` and `git log -1`, \
         look at the process with `ps` and at the disk it writes to, and run the command again if it did not finish.",
        after.saturating_add(extra).as_secs(),
        after.as_secs(),
        extra.as_secs()
    );
}

/// The error of a step left running:
/// the command stopped waiting for it,
/// so it is not known whether it finished.
#[derive(Debug)]
pub struct NotKnown(
    /// The sentence that names the step,
    /// how long it was waited for,
    /// and what to check.
    pub String,
);

impl std::fmt::Display for NotKnown {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.0.as_str());
    }
}

impl std::error::Error for NotKnown {}

/// Whether `error` is the error of a step left running because the command stopped waiting for it.
pub fn is_not_known(error: &std::io::Error) -> bool {
    return match error.get_ref() {
        Some(inner) => inner.is::<NotKnown>(),
        None => false,
    };
}

/// Whether `error` is the error of a step ended at its ceiling.
pub fn is_ceiling(error: &std::io::Error) -> bool {
    return match error.get_ref() {
        Some(inner) => inner.is::<CeilingReached>(),
        None => false,
    };
}

/// What:
///  The error of a step the service's stop ended,
/// or that was not started because the service is stopping.
/// Why:
///  The landing treats it like a step ended at its ceiling:
/// the commit is not made and its work is kept on a side ref (`landing/service_land.rs`),
/// and nothing reruns the step,
/// because the service is going away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class EndedByStop extends Error {}
/// ```
#[derive(Debug)]
pub struct EndedByStop(
    /// The sentence that names the step and says the stop ended it.
    pub String,
);

impl std::fmt::Display for EndedByStop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.0.as_str());
    }
}

impl std::error::Error for EndedByStop {}

/// What:
///  The error for a step that the service's stop ended.
/// Why:
///  One place words it,
/// so the landing's announcement and the work loop's records say the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new EndedByStop(`${waitingFor} was ended because the service was stopped`);
/// ```
pub fn ended_by_the_stop(waiting_for: &str) -> std::io::Error {
    return std::io::Error::new(std::io::ErrorKind::Interrupted, EndedByStop(format!("{waiting_for} was ended because the service was stopped")));
}

/// What:
///  The error for a step that was not started because the service is stopping.
/// Why:
///  The same type as a step the stop ended,
/// so a caller that tells stops from failures tells both at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new EndedByStop(refusal.message);
/// ```
pub fn refused_while_stopping(refusal: &crate::stop_of_the_service_noticed_by_its_children::RefusedWhileStopping) -> std::io::Error {
    return std::io::Error::new(std::io::ErrorKind::Interrupted, EndedByStop(refusal.to_string()));
}

/// Whether an error is a step the service's stop ended or refused.
pub fn is_ended_by_stop(error: &std::io::Error) -> bool {
    return match error.get_ref() {
        Some(inner) => inner.is::<EndedByStop>(),
        None => false,
    };
}

/// What:
///  What a watch saw of a wait that has just ended:
/// whether the service's stop ended the child,
/// or its ceiling
///  passed,
/// or it showed no sign of work.
/// Why:
///   The wait itself is made two ways (a synchronous caller's bridge and a task of the runtime),
/// and the endings must read the same
///  from both,
/// so the checks that word them live here and both faces call them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outlived = { endedByTheStop: boolean; ceilingMs?: number; ceilingHit: boolean; theWatchCouldEndIt: boolean; expired: boolean; limitMs: number };
/// ```
pub(crate) struct Outlived {
    /// Whether the service's stop ended the child.
    pub(crate) ended_by_the_stop: bool,
    /// The ceiling the wait ran under,
    /// or nothing when it had none.
    pub(crate) ceiling: Option<Duration>,
    /// Whether the ceiling passed and the child was ended because of it.
    pub(crate) ceiling_hit: bool,
    /// Whether the watch's pin proved which process to stop,
    /// and so whether its ceiling could be the ending at all.
    pub(crate) the_watch_could_end_it: bool,
    /// Whether the child showed no sign of work for the quiet window and was ended.
    pub(crate) expired: bool,
    /// The quiet window the wait ran under.
    pub(crate) limit: Duration,
}

/// What:
///  The sentence for a step ended at its ceiling:
/// what it was,
/// how long the wrapper lets one step run,
/// and whether it ended.
/// Why:
///   The ceiling's ending is worded in two places:
/// the watch that ended the step,
/// and the wait for its output
///  that ended it when nothing else could,
/// which alone knows whether the kernel let the step go.
/// One wording keeps the two from
///  disagreeing about the same ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} ran for ${seconds} seconds, the longest the wrapper lets one step run, ${leftRunning ? 'and could not be ended, so it was left running' : 'and was ended'}`
/// ```
pub fn step_ended_at_its_ceiling_text(waiting_for: &str, ceiling: Duration, left_running: bool) -> String {
    let ending: &str = if left_running {
        "and could not be ended, so it was left running, and the system collects it when its call returns"
    } else {
        "and was ended"
    };
    return format!("{waiting_for} ran for {} seconds, the longest the wrapper lets one step run, {ending}", ceiling.as_secs());
}

/// What:
///  The error a watched wait ended with,
/// or nothing when it ended on its own.
/// Why:
///   One wording of the three endings,
/// for the synchronous face of the wait and the task that awaits it
/// (`push/child_watch.rs`,
/// `push/child_watch_output_watched_on_the_runtime.rs`).
/// The stop is told first,
/// because a step the stop ended must be treated as one,
/// whatever its own end looked like;
/// then the ceiling (a step that keeps working is not a quiet one),
/// then the quiet window.
///  A watch that proved no process signalled nothing,
/// so neither of its own endings is one it caused:
/// the wait for the
///  step's output ends the step instead and words it,
/// and that answer stands here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (outlived.endedByTheStop) return endedByTheStop(waitingFor);
/// if (!outlived.theWatchCouldEndIt) return undefined;
/// if (outlived.ceiling !== undefined && outlived.ceilingHit) return ceilingReached(waitingFor, outlived.ceiling);
/// if (outlived.expired) return watchedTimeout(waitingFor, outlived.limit);
/// return undefined;
/// ```
pub(crate) fn ending_of_a_watched_step(waiting_for: &str, outlived: Outlived) -> Option<std::io::Error> {
    let Outlived { ended_by_the_stop: ended, ceiling, ceiling_hit, the_watch_could_end_it, expired, limit } = outlived;
    if ended {
        return Some(ended_by_the_stop(waiting_for));
    }
    // A watch whose pin proved nothing signalled nothing, so neither the ceiling it read nor the quiet window it saw is the step's
    //  ending: the wait for the step's output is what ended it, says so, and claims the end that happened.
    if !the_watch_could_end_it {
        return None;
    }
    if let Some(reached) = ceiling
        && ceiling_hit
    {
        return Some(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            CeilingReached(step_ended_at_its_ceiling_text(waiting_for, reached, false)),
        ));
    }
    if expired {
        return Some(std::io::Error::new(std::io::ErrorKind::TimedOut, watched_timeout_text(waiting_for, limit)));
    }
    return None;
}
