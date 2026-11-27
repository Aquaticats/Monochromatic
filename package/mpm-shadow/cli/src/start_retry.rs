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
//! For the two paths a start through this tool resolves,
//! the real `mpm` the shadow starts and the program a launcher's record names,
//! every writer traced or read on 2026-11-27 stages the new file under another name
//! and renames it into place:
//! `uv tool install` and `uv tool upgrade` for `mpm`,
//! `cargo binstall`,
//! `cargo install` and this tool's archive install for a launcher's program.
//! Two writers this tool starts do open a published path for writing:
//! `go install` where its work directory is on another file system (3.5 ms,
//! traced,
//! and the only refusal this round measured),
//! and `pnpm add --global` (0.09 ms,
//! traced);
//! neither writes a path a start through this tool resolves.
//! A window a retry could carry does exist,
//! and it is not a writer's:
//! a `fork` copies a write descriptor,
//! and the child holds that copy from its `fork` to its own `execve`,
//! about 1 to 3 ms in this crate's own tests,
//! which the record's threaded arm refused in 5 to 12 of 20 starts.
//! `start_failure` explains that refusal rather than retrying it,
//! and the record sets the two options before the owner.
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
