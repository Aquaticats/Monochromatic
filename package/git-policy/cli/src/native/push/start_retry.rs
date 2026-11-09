//! What:
//!  Starting a child process again when the operating system refuses it for lack of processes.
//! Why:
//!  A host that has run out of processes (a process limit,
//! a cgroup limit) refuses a start with `EAGAIN`,
//!  which Rust reports as `ErrorKind::WouldBlock`.
//!  A refused start started nothing,
//! so starting again is always safe,
//!  and such a shortage usually ends within moments.
//!  Without this the first refusal ends a commit that the next attempt would have finished.
//!  The wait is bounded,
//! so the wrapper still never stalls:
//! a shortage that outlasts the budget fails with the
//!  same error as before plus how often the start was refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (let delay = 10; ; delay = Math.min(delay * 2, 500)) { try { return start(); } catch (error) { if (!isEagain(error) || waited > 5000) throw error; await wait(delay); } }
//! ```

/// Diagnostic output to standard error for each refused start.
use crate::diagnostic_log::warn;
/// Error kinds,
/// and the result type of a start.
use std::io::{Error, ErrorKind, Result};
/// The child process types a start produces.
use std::process::{Child, Command, ExitStatus, Output};
/// A span of time.
use std::time::Duration;

/// How long a refused start may wait in total,
/// summed over its pauses.
pub const START_RETRY_BUDGET: Duration = Duration::from_secs(5);

/// The first pause after a refusal.
const FIRST_PAUSE: Duration = Duration::from_millis(10);

/// The longest single pause;
/// the pauses double until they reach it.
const LONGEST_PAUSE: Duration = Duration::from_millis(500);

/// Log tag of the retry warnings.
const TAG: &str = "startRetry";

/// What:
///  One start that the operating system may refuse for lack of processes,
/// and that the retry runs again.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The retry runs the start again after a refusal,
/// so the start is a value that holds what it needs (the command,
/// the request,
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

/// The pauses of one run of refusals:
/// how often the start was refused,
/// how long the run has paused,
/// and the next pause.
struct RefusalSchedule {
    /// How many times the start was refused so far.
    refusals: u32,
    /// The sum of the pauses taken so far.
    paused: Duration,
    /// The pause that follows the next refusal.
    delay: Duration,
}

impl RefusalSchedule {
    /// A run that has seen no refusal.
    const fn new() -> Self {
        return Self { refusals: 0, paused: Duration::ZERO, delay: FIRST_PAUSE };
    }

    /// What:
    ///  Counts one refusal and returns the pause that follows it,
    /// or the final error when the budget cannot hold that pause.
    /// Why:
    ///   The sleeping loop and the awaiting loop share one schedule,
    /// so the two cannot drift apart:
    /// the budget counts the pauses,
    ///  which dominate the time a refused start takes,
    /// and the final error keeps its kind and says how often the start was refused.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// schedule.pauseAfter(what, budgetMs, error) // the pause to take, or throws the final error
    /// ```
    fn pause_after(&mut self, what: &str, budget: Duration, error: Error) -> Result<Duration> {
        self.refusals += 1;
        if self.paused + self.delay > budget {
            return Err(Error::new(
                ErrorKind::WouldBlock,
                format!("{error}; the start was refused {} times over {} ms", self.refusals, self.paused.as_millis()),
            ));
        }
        warn(
            TAG,
            format!(
                "starting {what} was refused for lack of processes ({error}); trying again in {} ms",
                self.delay.as_millis()
            )
            .as_str(),
        );
        let pause: Duration = self.delay;
        self.paused += pause;
        self.delay = (self.delay * 2).min(LONGEST_PAUSE);
        return Ok(pause);
    }
}

/// What:
///  Run `start` until it is not refused for lack of processes,
/// pausing between tries.
/// Why:
///   Only `WouldBlock` is retried;
/// every other error is the caller's,
/// at once.
///  A test can drive it with a recording pause and no real waiting.
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
    let mut schedule: RefusalSchedule = RefusalSchedule::new();
    loop {
        let error: Error = match start.start() {
            Ok(started) => return Ok(started),
            Err(error) if error.kind() == ErrorKind::WouldBlock => error,
            Err(error) => return Err(error),
        };
        pause(schedule.pause_after(what, budget, error)?);
    }
}

/// What:
///  Run `start` until it is not refused for lack of processes,
/// waiting between tries on the runtime's timer.
/// Why:
///   The caller is a task of the runtime,
/// so the pause must not hold the worker it runs on:
/// a worker that sleeps for the
///  whole budget is a worker no other task can use.
/// The schedule and the budget are those of `retry_refused_start_with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retryRefusedStartAwaiting({ what, budgetMs, start });
/// ```
pub async fn retry_refused_start_awaiting_within<T, Start: RefusableStart<T>>(
    what: &str,
    budget: Duration,
    start: &mut Start,
) -> Result<T> {
    let mut schedule: RefusalSchedule = RefusalSchedule::new();
    loop {
        let error: Error = match start.start() {
            Ok(started) => return Ok(started),
            Err(error) if error.kind() == ErrorKind::WouldBlock => error,
            Err(error) => return Err(error),
        };
        tokio::time::sleep(schedule.pause_after(what, budget, error)?).await;
    }
}

/// What:
///  One start that is awaited between two pauses of the runtime's timer,
/// such as a Git step whose child is awaited rather than
///  blocked on.
/// Why:
///   `RefusableStart` runs a start that returns at once and sleeps between tries;
/// a start that itself awaits (a watched Git
///  step on the runtime,
/// `transaction/transaction_git_awaiting.rs`) must be awaited by the same budget rather than blocked,
///  so it needs a face of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RefusableAwaitingStart<T> { start(): Promise<T> }
/// ```
pub trait RefusableAwaitingStart<T> {
    /// Try the start once,
    /// awaiting the run it makes.
    fn start(&mut self) -> impl std::future::Future<Output = Result<T>> + Send;
}

/// What:
///  Run an awaited `start` until it is not refused for lack of processes,
/// pausing on the runtime's timer between
///  tries,
/// and give Git's own last words when the budget is spent on a refusal.
/// Why:
///   The same budget and the same schedule as `retry_refused_start_awaiting_within`,
/// for a start that must itself be awaited;
/// a worker thread that slept through the budget
///  would be a worker no other task could use,
/// and one that blocked on the child would be worse.
///  A refusal whose last output says the system refused Git is the caller's answer once the budget is spent,
/// as
///  `run_git_pausing` reads it for a synchronous caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retryRefusedAwaitingStart({ what, budgetMs, start }); // resolves with Git's last words after the budget
/// ```
pub async fn retry_refused_awaiting_start<T, Start: RefusableAwaitingStart<T>>(what: &str, budget: Duration, start: &mut Start) -> Result<T> {
    // `mut` lets the loop count its refusals and the time it has paused.
    let mut schedule: RefusalSchedule = RefusalSchedule::new();
    loop {
        let error: Error = match start.start().await {
            Ok(started) => return Ok(started),
            Err(error) if error.kind() == ErrorKind::WouldBlock => error,
            Err(error) => return Err(error),
        };
        tokio::time::sleep(schedule.pause_after(what, budget, error)?).await;
    }
}

/// What:
///  Run `start`,
/// pausing for real between refused tries within the standard budget.
/// Why:
///   The production entry;
/// tests use `retry_refused_start_with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retryRefusedStart({ what, start });
/// ```
pub fn retry_refused_start<T>(what: &str, start: &mut dyn RefusableStart<T>) -> Result<T> {
    return retry_refused_start_with(what, START_RETRY_BUDGET, &mut sleep_for, start);
}

/// Wait for `delay` on this thread:
/// the pause between two refused starts.
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

/// What:
///  Start a command without waiting for it,
/// pausing on the runtime's timer between refused tries within the standard budget.
/// Why:
///   The start of a child by a task of the runtime;
/// synchronous callers use `RetryStart::spawn_retrying`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = await spawnRetryingWithoutBlocking(command);
/// ```
pub async fn spawn_retrying_without_blocking(command: &mut Command) -> Result<Child> {
    let what: String = command.get_program().to_string_lossy().into_owned();
    return retry_refused_start_awaiting_within(what.as_str(), START_RETRY_BUDGET, &mut SpawnStart { command }).await;
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

/// The three ways of starting a command,
/// each retried when the start is refused.
impl RetryStart for Command {
    /// `Command::spawn`,
    /// retried.
    fn spawn_retrying(&mut self) -> Result<Child> {
        let what: String = self.get_program().to_string_lossy().into_owned();
        return retry_refused_start(what.as_str(), &mut SpawnStart { command: self });
    }

    /// `Command::output`,
    /// retried.
    ///  A refusal comes only from the start,
    /// so no child has run when it is retried.
    fn output_retrying(&mut self) -> Result<Output> {
        let what: String = self.get_program().to_string_lossy().into_owned();
        return retry_refused_start(what.as_str(), &mut OutputStart { command: self });
    }

    /// `Command::status`,
    /// retried.
    fn status_retrying(&mut self) -> Result<ExitStatus> {
        let what: String = self.get_program().to_string_lossy().into_owned();
        return retry_refused_start(what.as_str(), &mut StatusStart { command: self });
    }
}

/// `Command::exec` as a start the retry can run again:
/// it returns only the error that ended the replacement.
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
///  Replace this process with the command,
/// trying again when the replacement is refused for lack of processes.
///  Returns only the error that ended the attempts.
/// Why:
///   `exec` returns only when it failed,
/// and a failed `exec` replaced nothing.
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
