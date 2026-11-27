//! What:
//! Starting a child process again when the operating system refuses it for lack of processes.
//! Why:
//! A host at its process or task limit refuses a start with `EAGAIN`,
//! which Rust reports as `ErrorKind::WouldBlock`.
//! A refused start started nothing,
//! so starting again is safe,
//! and such a shortage usually ends within moments.
//! The wait is bounded,
//! so the shadow never stalls:
//! a shortage that outlasts the budget is reported to the caller.
//!
//! A start refused because another program holds the program's file open for writing
//! (`ETXTBSY`,
//! `ErrorKind::ExecutableFileBusy`) is not retried,
//! and `is_refusal` stays what its name says.
//! Read from the code,
//! every writer this tool starts or stands in front of gives the new program a name of its own
//! and names the finished file or the finished directory into place with a rename,
//! so none of them ever writes the path a start resolves.
//! Measured 2026-11-27 on the same syscall shapes,
//! the launcher placement,
//! the gunzipped download,
//! the plain download and the `tar` extraction held the file they were writing open
//! for 2.9 to 20.1 ms and never at the published path,
//! and `uv tool install --force` wrote the entry point under a temporary name in `bin/`
//! and renamed it.
//! A writer that does hold the published file open is outside this tool
//! and keeps it for as long as its own work takes,
//! so there is no measured window a retry could carry;
//! `start_failure` explains that refusal instead of hiding it behind a stall.
//! The arms,
//! the probe and the syscall timestamps are in
//! `doc/troubleshooting/mpm-shadow-a-start-refused-because-the-program-is-being-written.md`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (let delay = 10; ; delay = Math.min(delay * 2, 500)) { try { return start(); } catch (e) { if (!isEagain(e) || waited > 5000) throw e; await wait(delay); } }
//! ```

/// Imports the error kinds and result type of a start.
use std::io::{
    Error,
    Result,
};
/// Imports the span of time.
use std::time::Duration;

/// How long refused starts may pause in total.
pub const START_RETRY_BUDGET: Duration = Duration::from_secs(5);

/// What the retrying did,
/// for the message that follows a start.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RetryReport {
    /// How many starts the system refused for lack of processes.
    pub refusals: u32,
    /// The total time paused between tries.
    pub paused: Duration,
}

/// The first pause after a refusal.
const FIRST_PAUSE: Duration = Duration::from_millis(10);

/// The longest single pause;
/// the pauses double until they reach it.
const LONGEST_PAUSE: Duration = Duration::from_millis(500);

/// One attempt to start a program,
/// which the retry repeats while the system refuses to create a process.
///
/// A struct implements it when the attempt needs state,
/// and any `FnMut() -> Result<T>` implements it as well.
pub trait StartAttempt<T> {
    /// Tries to start the program once.
    fn attempt(&mut self) -> Result<T>;
}

/// Lets a plain function or closure stand in for a start attempt.
impl<T, Attempt: FnMut() -> Result<T>> StartAttempt<T> for Attempt {
    /// Runs the function.
    fn attempt(&mut self) -> Result<T> {
        return self();
    }
}

/// What:
/// Runs `start` until it is not refused for lack of processes,
/// pausing between tries so that the pauses total at most `budget`.
/// Why:
/// Only a refusal is retried;
/// every other error is the caller's at once.
/// The pause is injected so a test drives the whole budget without waiting.
/// The report says how often the system refused and how long was paused,
/// for the message the caller prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function retryRefusedStart<T>(budgetMs: number, pause: (ms: number) => void, start: () => T): [Result<T>, RetryReport]
/// ```
pub fn retry_refused_start_with<T>(
    budget: Duration,
    pause: &mut dyn FnMut(Duration),
    start: &mut dyn StartAttempt<T>,
) -> (Result<T>, RetryReport) {
    let mut report: RetryReport = RetryReport::default();
    let mut delay: Duration = FIRST_PAUSE;
    loop {
        let error: Error = match start.attempt() {
            Ok(started) => return (Ok(started), report),
            Err(error) => error,
        };
        if !is_refusal(&error) {
            return (Err(error), report);
        }
        report.refusals += 1;
        if report.paused + delay > budget {
            return (Err(error), report);
        }
        pause(delay);
        report.paused += delay;
        delay = (delay * 2).min(LONGEST_PAUSE);
    }
}

/// The production entry:
/// pauses for real.
pub fn retry_refused_start<T>(start: &mut dyn StartAttempt<T>) -> (Result<T>, RetryReport) {
    return retry_refused_start_with(START_RETRY_BUDGET, &mut std::thread::sleep, start);
}

/// Whether an error is the system's refusal to create another process.
/// A refusal to start a file that is being written is not one of these:
/// the module document above says why it is explained rather than retried.
pub fn is_refusal(error: &Error) -> bool {
    return error.kind() == std::io::ErrorKind::WouldBlock;
}
