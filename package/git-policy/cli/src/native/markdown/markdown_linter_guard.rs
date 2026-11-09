//! What:
//!  Run one job on a thread of its own and wait for it until it ends,
//! or until the thread has shown no sign of work for a
//!  window,
//! so a job that panics or never ends costs the caller one failed result,
//! not the process.
//! Why:
//!  The Markdown linter used to run as a child process,
//!  and the operating system gave the wrapper a free boundary:
//!  a child that hung could be killed and a child that crashed could not take the wrapper down.
//!  Linked into the wrapper,
//!  the linter shares its address space,
//!  so a boundary is rebuilt here from what the language offers,
//!  and it is a smaller one than a process gave:
//!  a hang and a panic are contained,
//! a stack overflow,
//! an out-of-memory abort or any other abort is not,
//!  and ends the wrapper like any other crash.
//!  The size of a finished result is bounded by the caller after the fact.
//!  The long-term answer is to run the linter in the resident service,
//! outside the client.
//!  A panic is caught on the job's thread and returned as text.
//!  A job that shows no sign of work for the window cannot be stopped from outside,
//!  so the wrapper stops waiting for it and leaves the thread to end with the process,
//!  which is short-lived;
//!  the thread owns everything it uses and shares nothing with the caller,
//!  so abandoning it leaves nothing locked.
//!  The thread has the linter's own stack size,
//!  because the main thread's stack is set by the platform and a stack overflow cannot be caught;
//!  the size makes an overflow as unlikely as in the executable,
//! it does not contain one.
//!  Silence ends the wait (owner,
//! 2026-11-03:
//! "Stop only on no progress";
//! pick 9 of
//!  `doc/planning/cli-git-shipped-limits-audit.md`,
//! the coordinator's under the owner's delegation and open to the owner's
//!  veto).
//! The signs of work are those the push reads,
//! taken from the job thread's own files under `/proc/self/task/`
//!  (`thread_quiet.rs`):
//! the bytes it read and wrote,
//! its processor time,
//! its state.
//! The longest quiet gap of every run is kept
//!  in the ledger under `monochromatic-lint` (`quiet_ledger.rs`).
//! Where `/proc` cannot be read only the ceiling ends the wait.
//!  The ceiling (`wait_ceilings.rs`,
//! `LINT_CEILING`) abandons a job that keeps showing work (owner,
//! 2026-11-04,
//! "Silence plus a
//!  ceiling (Recommended)"),
//! the same way:
//! the wrapper stops waiting and the thread ends with the process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const result = await Promise.race([worker.run(job), rejectWhenQuiet(worker, windowMs)]); // the worker is left running when it is quiet
//! ```

/// When a long wait prints its notices.
use crate::wait_progress::{Cadence, Ticker};
/// The longest quiet gap of each tool,
/// kept for the timing records.
use crate::quiet_ledger;
/// The thread's own number,
/// and the clock that says when a thread has been quiet for a window.
use crate::thread_quiet::{ThreadClock, current_thread_id};
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
/// type GuardFailure = { kind: 'start'; reason } | { kind: 'panicked'; reason } | { kind: 'quiet'; windowMs } | { kind: 'ceiling'; ceilingMs } | { kind: 'vanished' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuardFailure {
    /// The operating system would not start the thread.
    Start(String),
    /// The job panicked;
    /// the text is the panic's message.
    Panicked(String),
    /// The job showed no sign of work for the window and was abandoned.
    Quiet(Duration),
    /// The job ran as long as the ceiling allows,
    /// whatever it showed,
    /// and was abandoned.
    Ceiling(Duration),
    /// The thread ended without sending a result or a panic,
    ///  which no code path here does.
    Vanished,
}

/// What:
///  How long a guarded job may stay silent and how long it may run in all.
/// Why:
///   The window abandons a job that shows nothing;
/// the ceiling abandons one that keeps showing something (owner,
/// 2026-11-04,
///  "Silence plus a ceiling (Recommended)").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GuardLimits = { window: number; ceiling: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuardLimits {
    /// How long the job may show no sign of work.
    pub window: Duration,
    /// The longest the wrapper waits for the job,
    /// whatever it shows.
    pub ceiling: Duration,
}

/// What:
///  The text of a caught panic's payload.
/// Why:
///  A panic carries a `&str` or a `String`;
/// anything else has no message to show.
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
///  The linter hashes every image a file links,
/// which takes seconds for a large one,
///  and a silent wait looks like a hang;
///  the notice names the file and where the wait stops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noticeLine = (waitingFor, waitedMs, windowMs) => `monochromatic-git-policy-cli: still waiting for ${waitingFor} after ...`;
/// ```
fn notice_line(waiting_for: &str, waited: Duration, window: Duration, repeat: bool) -> String {
    let hint: &str = if repeat {
        ""
    } else {
        " Hint: it is still working; if it looks stuck, wait for the window or set the policy markdown/autofix to off for this command."
    };
    return format!(
        "monochromatic-git-policy-cli: still waiting for {waiting_for} after {} seconds; giving up on it only after {} seconds with no sign of work.{hint}",
        waited.as_secs(),
        window.as_secs()
    );
}

/// What:
///  Where a wait's notices go,
///  owned so that one can be written on a thread of its own and given back.
/// Why:
///  A notice is advisory output,
///  and the stream behind it may be a pipe whose reader has stopped draining it,
///  so the write can block for ever.
///  An owned sink is one a thread can hold while the waiting thread keeps judging its
///       deadlines and receiving the job's result;
///  a borrowed sink would keep the waiting thread inside the write.
///  Nothing is lost when a write never returns:
///  that thread ends with the process,
///  and this linter prints no further notice,
///  which is what a stream nobody reads can show anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NoticeSink = { writer?: Writable }; // undefined once a write never came back
/// ```
pub type NoticeSink = Option<Box<dyn Write + Send>>;

/// The name a notice's thread is given,
///  so a stack of a stuck command names what it is.
const NOTICE_THREAD_NAME: &str = "cli-git-markdown-notice";

/// What:
///  Start a thread that writes `line` to `sink` and hands the sink back.
/// Why:
///   The waiting thread must not be the one that blocks,
///  and the sink must come back for the next notice,
///  so the thread returns it over a channel the wait polls.
///  A thread the system will not start costs that notice and every later one,
///  because the sink went with it:
///  a notice is advisory,
///  and the wait itself goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeOnThread = (sink, line) => { const finished = channel(); thread(() => { writeln(sink, line); finished.send(sink) }); return finished; };
/// ```
fn write_on_a_thread(
    sink: Box<dyn Write + Send>,
    line: String,
) -> Option<Receiver<Box<dyn Write + Send>>> {
    let (finished, written): (Sender<Box<dyn Write + Send>>, Receiver<Box<dyn Write + Send>>) =
        channel();
    let started = std::thread::Builder::new()
        .name(String::from(NOTICE_THREAD_NAME))
        .spawn(move || {
            let mut writing: Box<dyn Write + Send> = sink;
            // Terminal output error: a notice is advisory; a failed write must not stop the wait.
            let _written = writeln!(writing, "{line}");
            // A waiter that answered already is gone, and then there is nothing to hand the sink to.
            let _handed_back = finished.send(writing);
        });
    return match started {
        Ok(_thread) => Some(written),
        Err(_unstartable) => None,
    };
}

/// What:
///  Wait for the job's thread to report,
///  printing notices at the cadence,
///  until the thread has shown no sign of work for the window or the ceiling is reached.
///  `cadence` times the notices and
///  `&mut NoticeSink` lends the sink the next notice is written from,
///  and takes back the one a finished write returns.
/// Why:
///  The wait sleeps on the channel,
///  so a job that finishes wakes it at once,
///  and wakes by itself only when a notice or a look at the thread is due.
///  The thread's number arrives first on its own channel;
/// until it does the thread is starting,
/// which is not quiet.
///  Every deadline is judged before a notice is started and never after it,
///  and the notice itself is written on another thread,
///  so a stream that never takes the line cannot move any deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function waitFor<T>(receiver, ids, limits, sink, waitingFor): Promise<T>; // throws GuardFailure
/// ```
fn wait_for<T>(
    receiver: &Receiver<Result<T, String>>,
    thread_numbers: &Receiver<i32>,
    limits: GuardLimits,
    cadence: Cadence,
    waiting_for: &str,
    notices: &mut NoticeSink,
) -> (Result<T, GuardFailure>, Duration) {
    let window: Duration = limits.window;
    let began: Instant = Instant::now();
    let mut ticker: Ticker = Ticker::new(cadence);
    let mut printed: u32 = 0;
    let mut writing: Option<Receiver<Box<dyn Write + Send>>> = None;
    let mut clock: Option<ThreadClock> = None;
    let mut longest: Duration = Duration::ZERO;
    loop {
        // What: A notice write that came back hands its sink over, so the next due notice can use it.
        // Why:  The write happens on its own thread; collecting it here is what keeps one notice in flight at a time.
        //
        // In TS you'd write (pseudocode):
        // if (writing?.tryRecv()) { sink = returned; writing = undefined; }
        if let Some(finished) = writing.as_ref()
            && let Ok(returned) = finished.try_recv()
        {
            *notices = Some(returned);
            writing = None;
        }
        if clock.is_none()
            && let Ok(number) = thread_numbers.try_recv()
        {
            clock = Some(ThreadClock::start(number, window, false));
        }
        if let Some(watching) = clock.as_mut() {
            let quiet: bool = watching.quiet();
            longest = watching.longest_quiet();
            if quiet {
                return (Err(GuardFailure::Quiet(window)), longest);
            }
        }
        // The ceiling abandons a job that keeps showing work, as the window abandons one that shows none (owner, 2026-11-04,
        // "Silence plus a ceiling (Recommended)").
        if began.elapsed() >= limits.ceiling {
            return (Err(GuardFailure::Ceiling(limits.ceiling)), longest);
        }
        // What: A notice that is due goes out now that no deadline says to stop, on a thread of its own.
        // Why:  An abandoned job was announced, and the wait never holds the stream that carries the announcement.
        //
        // In TS you'd write (pseudocode):
        // if (writing === undefined && sink !== undefined && ticker.due(elapsed)) writing = writeOnThread(sink.take(), line);
        if writing.is_none() {
            let waited: Duration = began.elapsed();
            if ticker.due(waited)
                && let Some(sink) = notices.take()
            {
                let line: String = notice_line(waiting_for, waited, window, printed > 0);
                printed += 1;
                writing = write_on_a_thread(sink, line);
            }
        }
        match receiver.recv_timeout(NOTICE_POLL) {
            Ok(Ok(value)) => return (Ok(value), longest),
            Ok(Err(reason)) => return (Err(GuardFailure::Panicked(reason)), longest),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return (Err(GuardFailure::Vanished), longest),
        }
    }
}

/// The longest the wait sleeps before it looks at the notice schedule again.
///  Exempt from a citation under `LQM` (class:
/// poll):
/// it sets how often a condition is looked at again,
/// so it adds at
///  most one poll interval to a wait that already exists.
/// Classed once in `doc/planning/cli-git-shipped-limits-audit.md`,
///  "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
const NOTICE_POLL: Duration = Duration::from_millis(50);

/// What:
///  Run `job` on a new thread and return its result,
///  or why there is none.
///  `T` is whatever the job returns;
///  both the job and its result move between threads.
/// Why:
///  See the module comment.
///  The caller names what it waits for ("monochromatic-lint to check docs/a.md"),
///  which the notices print,
///  and hands over the sink they go to (`NoticeSink`),
///  so no deadline of this wait can be held by a stream that never takes a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runGuarded<T>(job: () => T, { windowMs, ceilingMs }, cadence, waitingFor, notices): Promise<T>; // throws GuardFailure
/// ```
pub fn run_guarded<T: Send + 'static>(
    job: impl FnOnce() -> T + Send + 'static,
    limits: GuardLimits,
    cadence: Cadence,
    waiting_for: &str,
    notices: &mut NoticeSink,
) -> Result<T, GuardFailure> {
    let (sender, receiver): (Sender<Result<T, String>>, Receiver<Result<T, String>>) = channel();
    let (numbers, thread_numbers): (Sender<i32>, Receiver<i32>) = channel();
    let spawned = lint_thread().name(String::from("cli-git-markdown-lint")).spawn(move || {
        // The waiting side judges this thread by its number; a closed channel is a waiter that gave up.
        if let Some(number) = current_thread_id() {
            let _told = numbers.send(number);
        }
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
    let (result, longest): (Result<T, GuardFailure>, Duration) =
        wait_for(&receiver, &thread_numbers, limits, cadence, waiting_for, notices);
    quiet_ledger::note("monochromatic-lint", longest, matches!(result, Err(GuardFailure::Quiet(_))));
    return result;
}

/// Panic,
/// quiet-window and notice controls stay out of the release executable.
#[cfg(test)]
#[path = "markdown_linter_guard_tests.rs"]
mod tests;
