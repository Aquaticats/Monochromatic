//! What:
//!  Starting a child process again when the operating system refuses it for lack of processes.
//! Why:
//!  A host that has run out of processes (a process limit, a cgroup limit) refuses a start with `EAGAIN`,
//!  which Rust reports as `ErrorKind::WouldBlock`.
//!  A refused start started nothing, so starting again is always safe,
//!  and such a shortage usually ends within moments.
//!  Without this the first refusal ends a commit that the next attempt would have finished.
//!  The wait is bounded, so the wrapper still never stalls: a shortage that outlasts the budget fails with the
//!  same error as before plus how often the start was refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (let delay = 10; ; delay = Math.min(delay * 2, 500)) { try { return start(); } catch (error) { if (!isEagain(error) || waited > 5000) throw error; await wait(delay); } }
//! ```

/// Diagnostic output to standard error for each refused start.
use crate::diagnostic_log::warn;
/// Error kinds, and the result type of a start.
use std::io::{Error, ErrorKind, Result};
/// The child process types a start produces.
use std::process::{Child, Command, ExitStatus, Output};
/// A span of time.
use std::time::Duration;

/// How long a refused start may wait in total, summed over its pauses.
pub const START_RETRY_BUDGET: Duration = Duration::from_secs(5);

/// The first pause after a refusal.
const FIRST_PAUSE: Duration = Duration::from_millis(10);

/// The longest single pause; the pauses double until they reach it.
const LONGEST_PAUSE: Duration = Duration::from_millis(500);

/// Log tag of the retry warnings.
const TAG: &str = "startRetry";

/// What:
///  One start that the operating system may refuse for lack of processes, and that the retry runs again.
///  A `trait` is a named set of methods that different types can each supply, like a TypeScript `interface`.
/// Why:
///   The retry runs the start again after a refusal, so the start is a value that holds what it needs (the command, the request,
///  the output it keeps) and can be called more than once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RefusableStart<T> { start(): T } // throws the refusal
/// ```
pub trait RefusableStart<T> {
    /// Try the start once.
    fn start(&mut self) -> Result<T>;
}

/// What:
///  Run `start` until it is not refused for lack of processes, pausing between tries.
/// Why:
///   Only `WouldBlock` is retried; every other error is the caller's, at once.
///  The budget counts the pauses, which dominate the time a refused start takes,
///  so a test can drive it with a recording pause and no real waiting.
///  The final error keeps its kind and says how often the start was refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retryRefusedStart({ what, budgetMs, pause, start });
/// ```
pub fn retry_refused_start_with<T>(
    what: &str,
    budget: Duration,
    pause: &mut dyn FnMut(Duration),
    start: &mut dyn RefusableStart<T>,
) -> Result<T> {
    // `mut` lets the loop count its refusals and the time it has paused.
    let mut refusals: u32 = 0;
    let mut paused: Duration = Duration::ZERO;
    let mut delay: Duration = FIRST_PAUSE;
    loop {
        let error: Error = match start.start() {
            Ok(started) => return Ok(started),
            Err(error) if error.kind() == ErrorKind::WouldBlock => error,
            Err(error) => return Err(error),
        };
        refusals += 1;
        if paused + delay > budget {
            return Err(Error::new(
                ErrorKind::WouldBlock,
                format!("{error}; the start was refused {refusals} times over {} ms", paused.as_millis()),
            ));
        }
        warn(
            TAG,
            format!(
                "starting {what} was refused for lack of processes ({error}); trying again in {} ms",
                delay.as_millis()
            )
            .as_str(),
        );
        pause(delay);
        paused += delay;
        delay = (delay * 2).min(LONGEST_PAUSE);
    }
}

/// What:
///  Run `start`, pausing for real between refused tries within the standard budget.
/// Why:
///   The production entry; tests use `retry_refused_start_with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retryRefusedStart({ what, start });
/// ```
pub fn retry_refused_start<T>(what: &str, start: &mut dyn RefusableStart<T>) -> Result<T> {
    return retry_refused_start_with(what, START_RETRY_BUDGET, &mut sleep_for, start);
}

/// Wait for `delay` on this thread: the pause between two refused starts.
fn sleep_for(delay: Duration) {
    std::thread::sleep(delay);
}

/// Starts of a command that ride out a short shortage of processes.
pub trait RetryStart {
    /// Start the child without waiting for it.
    fn spawn_retrying(&mut self) -> Result<Child>;
    /// Start the child and collect its output.
    fn output_retrying(&mut self) -> Result<Output>;
    /// Start the child with inherited streams and wait for it.
    fn status_retrying(&mut self) -> Result<ExitStatus>;
}

/// `Command::spawn` as a start the retry can run again.
struct SpawnStart<'a> {
    /// The command to start.
    command: &'a mut Command,
}

impl RefusableStart<Child> for SpawnStart<'_> {
    fn start(&mut self) -> Result<Child> {
        return self.command.spawn();
    }
}

/// `Command::output` as a start the retry can run again.
struct OutputStart<'a> {
    /// The command to start.
    command: &'a mut Command,
}

impl RefusableStart<Output> for OutputStart<'_> {
    fn start(&mut self) -> Result<Output> {
        return self.command.output();
    }
}

/// `Command::status` as a start the retry can run again.
struct StatusStart<'a> {
    /// The command to start.
    command: &'a mut Command,
}

impl RefusableStart<ExitStatus> for StatusStart<'_> {
    fn start(&mut self) -> Result<ExitStatus> {
        return self.command.status();
    }
}

/// The three ways of starting a command, each retried when the start is refused.
impl RetryStart for Command {
    /// `Command::spawn`, retried.
    fn spawn_retrying(&mut self) -> Result<Child> {
        let what: String = self.get_program().to_string_lossy().into_owned();
        return retry_refused_start(what.as_str(), &mut SpawnStart { command: self });
    }

    /// `Command::output`, retried.
    ///  A refusal comes only from the start, so no child has run when it is retried.
    fn output_retrying(&mut self) -> Result<Output> {
        let what: String = self.get_program().to_string_lossy().into_owned();
        return retry_refused_start(what.as_str(), &mut OutputStart { command: self });
    }

    /// `Command::status`, retried.
    fn status_retrying(&mut self) -> Result<ExitStatus> {
        let what: String = self.get_program().to_string_lossy().into_owned();
        return retry_refused_start(what.as_str(), &mut StatusStart { command: self });
    }
}

/// `Command::exec` as a start the retry can run again: it returns only the error that ended the replacement.
#[cfg(unix)]
struct ExecStart<'a> {
    /// The command that replaces this process.
    command: &'a mut Command,
}

#[cfg(unix)]
impl RefusableStart<()> for ExecStart<'_> {
    fn start(&mut self) -> Result<()> {
        /// The Unix extension that adds `exec` to a command.
        use std::os::unix::process::CommandExt;
        return Err(self.command.exec());
    }
}

/// What:
///  Replace this process with the command, trying again when the replacement is refused for lack of processes.
///  Returns only the error that ended the attempts.
/// Why:
///   `exec` returns only when it failed, and a failed `exec` replaced nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.execve(command) with the same retry
/// ```
#[cfg(unix)]
pub fn exec_retrying(command: &mut Command) -> Error {
    let what: String = command.get_program().to_string_lossy().into_owned();
    let refused: Result<()> = retry_refused_start(what.as_str(), &mut ExecStart { command });
    match refused {
        Err(error) => return error,
        // `exec` never returns success; the start above only ever returns an error.
        Ok(()) => return Error::other("exec returned without an error"),
    }
}

/// Retry controls stay out of the release executable.
#[cfg(test)]
#[path = "start_retry_tests.rs"]
mod tests;
