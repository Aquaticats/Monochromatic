//! What:
//! Runs a child process in its own process group and collects its output within a time bound.
//! Why:
//! The owner's rule of 2026-11-03 is "nothing should ever wait forever or indefinitely".
//! At the bound the whole process group is stopped, and the caller gets an error that names what was running;
//! a call that returns has no member of the group left alive, so no descendant can write after the caller recorded,
//! verified or rolled back what the child did (review finding 4c F2).
//! Output is returned only when it is complete: a prefix is an error (review finding 4c F4).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded-run.ts
//! export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
//! ```

use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group};

/// How often the child and its group are asked whether they are gone.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// How long the direct child is given to be reaped after the group was killed.
/// Unmeasured: a killed process is reaped within milliseconds unless it is in an uninterruptible wait, which no bound can fix;
/// what must be measured is the time from `kill` to `wait` on the machines the shim runs on.
const REAP_BOUND: Duration = Duration::from_secs(5);

/// How long the members of the group are given to disappear after they were killed, or after the direct child exited normally.
/// Unmeasured, for the same reason as `REAP_BOUND`.
const QUIESCE_BOUND: Duration = Duration::from_secs(5);

/// How long the output readers are given to reach the end of the output once the group is gone.
/// With every writer gone the pipes are at their end at once;
/// a reader that does not finish means a process outside the group holds the pipe.
/// Unmeasured: what must be measured is the drain time of the largest output (`cargo metadata` of this repository).
const READER_GRACE: Duration = Duration::from_secs(2);

thread_local! {
    /// The file that records the process group of every child started by this thread, set while a transaction is open.
    static GROUP_LOG: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// What:
/// `pub fn set_group_log(path: Option<PathBuf>)` names the file in which this thread records the process group of each child it
/// starts (`None` stops recording).
/// Why:
/// A recovery that finds a killed run's journal needs to know whether a child of that run may still be running,
/// which the end of the wrapper's lock does not say.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function setGroupLog(path?: string): void
/// ```
pub fn set_group_log(path: Option<PathBuf>) {
    GROUP_LOG.with(|slot| { *slot.borrow_mut() = path; });
}

/// The bound of one run and the words an error uses.
#[derive(Clone, Debug)]
pub struct Bound {
    /// The longest the child may run.
    pub limit: Duration,
    /// What is running, for the error.
    pub what: String,
    /// The command to run again, for the error.
    pub rerun: String,
}

/// Why a bounded run did not produce output.
#[derive(Debug)]
pub enum RunError {
    /// The child could not be started or read.
    Io(
        /// The system error.
        io::Error,
    ),
    /// The child outlived its bound; its process group was killed.
    TimedOut {
        /// What was running.
        what: String,
        /// The bound it exceeded.
        limit: Duration,
        /// The command to run again.
        rerun: String,
    },
    /// Members of the process group could not be shown to be gone, so something may still write.
    NotQuiescent {
        /// What was running.
        what: String,
    },
    /// The output could not be read to its end.
    IncompleteOutput {
        /// What was running.
        what: String,
    },
}

impl std::fmt::Display for RunError {
    /// What:
    /// `fn fmt(&self, formatter) -> fmt::Result` words the error for the user.
    /// Why:
    /// Each variant says what blocked and what state it leaves.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            RunError::Io(error) => write!(formatter, "{error}"),
            RunError::TimedOut { what, limit, rerun } => write!(
                formatter,
                "{what} did not finish within {limit:?}; it and everything it started were stopped, and no file was restored over or removed by that. Run {rerun} again when it can finish"
            ),
            RunError::NotQuiescent { what } => write!(formatter, "{what} left a process running that could not be stopped, so nothing it may still write can be trusted; no file was changed by this tool"),
            RunError::IncompleteOutput { what } => write!(formatter, "the output of {what} could not be read to its end because a process outside its process group holds the pipe; the partial output was discarded"),
        };
    }
}

/// What:
/// `fn group_exists(group: Pid) -> bool` says whether any process is still in the group.
/// Why:
/// A process that exists but cannot be signalled (`EPERM`) still exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const groupExists = (group: number): boolean => { try { process.kill(-group, 0); return true; } catch (e) { return e.code !== 'ESRCH'; } };
/// ```
fn group_exists(group: Pid) -> bool {
    return match test_kill_process_group(group) {
        Ok(()) => true,
        Err(code) => code != Errno::SRCH,
    };
}

/// What:
/// `fn group_gone_within(group: Pid, limit: Duration) -> bool` polls until no process is in the group or the limit passes.
/// Why:
/// Killed members are reaped by their parent or by init a moment later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupGoneWithin(group: number, limit: number): boolean
/// ```
fn group_gone_within(group: Pid, limit: Duration) -> bool {
    let deadline: Instant = Instant::now() + limit;
    loop {
        if !group_exists(group) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// What:
/// `fn reap(child: &mut Child) -> Option<ExitStatus>` waits for the direct child, a few milliseconds at a time, up to `REAP_BOUND`.
/// Why:
/// `Child::wait` has no deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reap(child: ChildProcess): ExitStatus | undefined
/// ```
fn reap(child: &mut Child) -> Option<ExitStatus> {
    let deadline: Instant = Instant::now() + REAP_BOUND;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {}
            Err(_) => return None,
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// What:
/// `fn stop_group(group: Pid, child: &mut Child) -> bool` kills the process group, reaps the direct child and waits for the group to
/// be gone; `true` only if both happened.
/// Why:
/// This is the one way a run is ended early, on timeout and on every error path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopGroup(group: number, child: ChildProcess): boolean
/// ```
fn stop_group(group: Pid, child: &mut Child) -> bool {
    let _ = kill_process_group(group, Signal::KILL);
    let reaped: bool = reap(child).is_some();
    return group_gone_within(group, QUIESCE_BOUND) && reaped;
}

/// What:
/// `fn collect(mut stream: impl Read + Send + 'static) -> Receiver<io::Result<Vec<u8>>>` reads a pipe to its end on its own thread.
/// Why:
/// A child that fills a pipe nobody reads blocks, and the result tells whether the end of the output was reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collect(stream: Readable): Promise<Buffer>
/// ```
fn collect(mut stream: impl Read + Send + 'static) -> Receiver<io::Result<Vec<u8>>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes: Vec<u8> = Vec::new();
        let outcome: io::Result<Vec<u8>> = stream.read_to_end(&mut bytes).map(|_| { return bytes; });
        let _ = sender.send(outcome);
    });
    return receiver;
}

/// What:
/// `fn finished(receiver: &Receiver<io::Result<Vec<u8>>>, what: &str) -> Result<Vec<u8>, RunError>` takes a reader's result within
/// `READER_GRACE`.
/// Why:
/// A reader that has not reached the end means the output is incomplete, which is an error and never a prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finished(reader: Promise<Buffer>, what: string): Buffer  // throws
/// ```
fn finished(receiver: &Receiver<io::Result<Vec<u8>>>, what: &str) -> Result<Vec<u8>, RunError> {
    return match receiver.recv_timeout(READER_GRACE) {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(error)) => Err(RunError::Io(error)),
        Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => Err(RunError::IncompleteOutput { what: what.to_string() }),
    };
}

/// What:
/// `pub fn output_within(command, input, bound) -> Result<Output, RunError>` runs the command in its own process group, feeds it
/// `input` and returns its complete output, or stops the whole group at the bound.
/// Why:
/// Every wait for a child process in cargo-shadow goes through here, so each has a bound and a named blocker.
/// The direct child is always reaped; on timeout and on every error the group is killed and shown to be gone;
/// after a normal exit any member left in the group is killed too, because it could still write after the caller moved on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
/// ```
pub fn output_within(command: &mut Command, input: Option<&[u8]>, bound: &Bound) -> Result<Output, RunError> {
    command.process_group(0).stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child: Child = command.spawn().map_err(RunError::Io)?;
    let group: Pid = Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)).ok_or_else(|| { return RunError::Io(io::Error::other("the child has no process id")); })?;
    GROUP_LOG.with(|slot| {
        if let Some(path) = slot.borrow().as_ref()
            && let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path)
        {
            let _ = writeln!(file, "{}", group.as_raw_nonzero());
        }
    });
    let writer: Option<Receiver<io::Result<()>>> = match (input, child.stdin.take()) {
        (Some(bytes), Some(mut stdin)) => {
            let owned: Vec<u8> = bytes.to_vec();
            let (sender, receiver) = mpsc::channel();
            thread::spawn(move || {
                let _ = sender.send(stdin.write_all(&owned));
            });
            Some(receiver)
        }
        _ => None,
    };
    let stdout = collect(child.stdout.take().ok_or_else(|| { return RunError::Io(io::Error::other("no standard output pipe")); })?);
    let stderr = collect(child.stderr.take().ok_or_else(|| { return RunError::Io(io::Error::other("no standard error pipe")); })?);
    let deadline: Instant = Instant::now() + bound.limit;
    let status: ExitStatus = loop {
        match child.try_wait() {
            Ok(Some(found)) => break found,
            Ok(None) => {}
            Err(error) => {
                stop_group(group, &mut child);
                return Err(RunError::Io(error));
            }
        }
        if Instant::now() >= deadline {
            if !stop_group(group, &mut child) {
                return Err(RunError::NotQuiescent { what: bound.what.clone() });
            }
            return Err(RunError::TimedOut { what: bound.what.clone(), limit: bound.limit, rerun: bound.rerun.clone() });
        }
        thread::sleep(POLL_INTERVAL);
    };
    // The parent is gone; whatever it left in its group could still write, so it is stopped before the caller moves on.
    if group_exists(group) {
        let _ = kill_process_group(group, Signal::KILL);
    }
    if !group_gone_within(group, QUIESCE_BOUND) {
        return Err(RunError::NotQuiescent { what: bound.what.clone() });
    }
    let out: Vec<u8> = finished(&stdout, &bound.what)?;
    let err: Vec<u8> = finished(&stderr, &bound.what)?;
    if let Some(receiver) = writer
        && let Ok(Err(error)) = receiver.recv_timeout(READER_GRACE)
        && error.kind() != io::ErrorKind::BrokenPipe
    {
        return Err(RunError::Io(error));
    }
    return Ok(Output { status, stdout: out, stderr: err });
}
