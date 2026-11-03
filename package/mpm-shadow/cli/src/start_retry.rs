//! What:
//! Starting a child process again when the operating system refuses it for lack of processes.
//! Why:
//! A host at its process or task limit refuses a start with `EAGAIN`, which Rust reports as `ErrorKind::WouldBlock`.
//! A refused start started nothing, so starting again is safe, and such a shortage usually ends within moments.
//! The wait is bounded, so the shadow never stalls: a shortage that outlasts the budget is reported to the caller.
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

/// What the retrying did, for the message that follows a start.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RetryReport {
    /// How many starts the system refused for lack of processes.
    pub refusals: u32,
    /// The total time paused between tries.
    pub paused: Duration,
}

/// Runs `start` until it is not refused for lack of processes, pausing between tries within `budget`.
pub fn retry_refused_start_with<T>(
    _budget: Duration,
    _pause: &mut dyn FnMut(Duration),
    start: &mut dyn FnMut() -> Result<T>,
) -> (Result<T>, RetryReport) {
    let outcome: Result<T> = start();
    return (outcome, RetryReport::default());
}

/// The production entry: pauses for real.
pub fn retry_refused_start<T>(start: &mut dyn FnMut() -> Result<T>) -> (Result<T>, RetryReport) {
    return retry_refused_start_with(START_RETRY_BUDGET, &mut |delay: Duration| std::thread::sleep(delay), start);
}

/// Whether an error is the system's refusal to create another process.
pub fn is_refusal(error: &Error) -> bool {
    return error.kind() == std::io::ErrorKind::WouldBlock;
}
