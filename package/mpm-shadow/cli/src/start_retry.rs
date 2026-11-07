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
//! In TS you'd write (pseudocode):
//! ```ts
//! for (let delay = 10; ; delay = Math.min(delay * 2, 500)) { try { return start(); } catch (e) { if (!isEagain(e) || waited > 5000) throw e; await wait(delay); } }
//! ```

/// Imports the error kinds and result type of a start.
use std::io::{Error, Result};
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
pub fn is_refusal(error: &Error) -> bool {
    return error.kind() == std::io::ErrorKind::WouldBlock;
}
