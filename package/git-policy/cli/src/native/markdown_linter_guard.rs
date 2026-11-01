//! What:
//!  Run one job on a thread of its own and wait for it with a deadline,
//!  so a job that panics or never ends costs the caller one failed result,
//!  not the process.
//! Why:
//!  The Markdown linter used to run as a child process,
//!  and the operating system gave the wrapper a free boundary:
//!  a child that hung could be killed and a child that crashed could not take the wrapper down.
//!  Linked into the wrapper,
//!  the linter shares its address space,
//!  so a boundary is rebuilt here from what the language offers,
//!  and it is a smaller one than a process gave:
//!  a hang and a panic are contained, a stack overflow, an out-of-memory abort or any other abort is not,
//!  and ends the wrapper like any other crash.
//!  The size of a finished result is bounded by the caller after the fact.
//!  The long-term answer is to run the linter in the resident service, outside the client.
//!  A panic is caught on the job's thread and returned as text.
//!  A job that outlives its deadline cannot be stopped from outside,
//!  so the wrapper stops waiting for it and leaves the thread to end with the process,
//!  which is short-lived;
//!  the thread owns everything it uses and shares nothing with the caller,
//!  so abandoning it leaves nothing locked.
//!  The thread has the linter's own stack size,
//!  because the main thread's stack is set by the platform and a stack overflow cannot be caught;
//!  the size makes an overflow as unlikely as in the executable, it does not contain one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const result = await Promise.race([worker.run(job), rejectAfter(deadlineMs)]); // the worker is left running on a timeout
//! ```

/// When a long wait prints its notices.
use super::wait_progress::{Cadence, WaitReporter};
/// The thread builder the linter's own executable uses for everything that lints.
use monochromatic_lint::run_workers::lint_thread;
/// Panic containment and the payload type of a caught panic.
use std::panic::{AssertUnwindSafe, catch_unwind};
/// The sink of notices.
use std::io::Write;
/// A one-shot channel from the job's thread to the waiting caller.
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
/// Spans of time and the clock.
use std::time::{Duration, Instant};

/// What:
///  Why a guarded job produced no result.
/// Why:
///  Each cause names a different way forward for the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GuardFailure = { kind: 'start'; reason } | { kind: 'panicked'; reason } | { kind: 'timed-out'; deadlineMs } | { kind: 'vanished' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuardFailure {
    /// The operating system would not start the thread.
    Start(String),
    /// The job panicked; the text is the panic's message.
    Panicked(String),
    /// The job did not finish within the deadline and was abandoned.
    TimedOut(Duration),
    /// The thread ended without sending a result or a panic,
    ///  which no code path here does.
    Vanished,
}

/// What:
///  The text of a caught panic's payload.
/// Why:
///  A panic carries a `&str` or a `String`; anything else has no message to show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const panicText = (payload: unknown) => typeof payload === 'string' ? payload : 'a panic payload that is not text';
/// ```
fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&str>() {
        return String::from(*message);
    }
    return String::from("a panic payload that is not text");
}

/// What:
///  The notice a long wait prints.
/// Why:
///  The linter hashes every image a file links, which takes seconds for a large one,
///  and a silent wait looks like a hang;
///  the notice names the file and where the wait stops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noticeLine = (waitingFor, waitedMs, deadlineMs) => `cli-git: still waiting for ${waitingFor} after ...`;
/// ```
fn notice_line(waiting_for: &str, waited: Duration, deadline: Duration, repeat: bool) -> String {
    let hint: &str = if repeat {
        ""
    } else {
        " Hint: it is still working; if it looks stuck, wait for the limit or set the policy markdown/autofix to off for this command."
    };
    return format!(
        "cli-git: still waiting for {waiting_for} after {} seconds; giving up on it after {} seconds.{hint}",
        waited.as_secs(),
        deadline.as_secs()
    );
}

/// What:
///  Wait for the job's thread to report,
///  printing notices at the cadence,
///  until the deadline.
/// Why:
///  The wait sleeps on the channel,
///  so a job that finishes wakes it at once,
///  and wakes by itself only when a notice or the deadline is due.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function waitFor<T>(receiver, deadlineMs, reporter): Promise<T>; // throws GuardFailure
/// ```
fn wait_for<T>(
    receiver: &Receiver<Result<T, String>>,
    deadline: Duration,
    waiting_for: &str,
    reporter: &mut WaitReporter<'_>,
) -> Result<T, GuardFailure> {
    let started: Instant = Instant::now();
    loop {
        let elapsed: Duration = started.elapsed();
        let remaining: Duration = deadline.saturating_sub(elapsed);
        // A notice that is due goes out before the deadline is checked, so an abandoned job was announced.
        reporter.poll(|waited, repeat| {
            return notice_line(waiting_for, waited, deadline, repeat);
        });
        if remaining.is_zero() {
            return Err(GuardFailure::TimedOut(deadline));
        }
        // Wake for the deadline at the latest; a short slice keeps notices on schedule.
        let slice: Duration = remaining.min(NOTICE_POLL);
        match receiver.recv_timeout(slice) {
            Ok(Ok(value)) => return Ok(value),
            Ok(Err(reason)) => return Err(GuardFailure::Panicked(reason)),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return Err(GuardFailure::Vanished),
        }
    }
}

/// The longest the wait sleeps before it looks at the notice schedule again.
const NOTICE_POLL: Duration = Duration::from_millis(50);

/// What:
///  Run `job` on a new thread and return its result,
///  or why there is none.
///  `T` is whatever the job returns;
///  both the job and its result move between threads.
/// Why:
///  See the module comment.
///  The caller names what it waits for ("monochromatic-lint to check docs/a.md"),
///  which the notices print.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runGuarded<T>(job: () => T, deadlineMs, cadence, waitingFor, notices): Promise<T>; // throws GuardFailure
/// ```
pub fn run_guarded<T: Send + 'static>(
    job: impl FnOnce() -> T + Send + 'static,
    deadline: Duration,
    cadence: Cadence,
    waiting_for: &str,
    notices: &mut dyn Write,
) -> Result<T, GuardFailure> {
    let (sender, receiver): (Sender<Result<T, String>>, Receiver<Result<T, String>>) = channel();
    let spawned = lint_thread().name(String::from("cli-git-markdown-lint")).spawn(move || {
        let outcome: Result<T, String> = match catch_unwind(AssertUnwindSafe(job)) {
            Ok(value) => Ok(value),
            Err(payload) => Err(panic_text(payload.as_ref())),
        };
        // The caller may have given up on this job; a closed channel is that case, and there is nobody to tell.
        let _abandoned = sender.send(outcome);
    });
    if let Err(error) = spawned {
        return Err(GuardFailure::Start(error.to_string()));
    }
    let mut reporter: WaitReporter<'_> = WaitReporter::start(cadence, notices);
    return wait_for(&receiver, deadline, waiting_for, &mut reporter);
}

/// Panic, deadline and notice controls stay out of the release executable.
#[cfg(test)]
#[path = "markdown_linter_guard_tests.rs"]
mod tests;
