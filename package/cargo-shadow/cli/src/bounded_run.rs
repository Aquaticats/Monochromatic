//! What:
//! Runs a child process in its own process group and collects its output within a time bound,
//! with everything the child started contained.
//! Why:
//! The owner's rule of 2026-11-03 is "nothing should ever wait forever or indefinitely".
//! At the bound, and after a normal exit, every process the child started is stopped, wherever it went:
//! while the run is open this process is a child subreaper, so descendants that left the process group (`setsid`, `setpgid`) or
//! closed their pipes are still found as descendants and killed.
//! A call that returns `Ok` found no live descendant (round 6, fifth review 5a F1 and 5c N2).
//! The recorded process group of the child is on disk before the child runs its first instruction.
//! Output is returned only when it is complete:
//! a prefix is an error (review finding 4c F4).
//! The limits: a process that is held in an uninterruptible wait cannot be killed, and that is `NotQuiescent`;
//! after this process itself died nothing adopts the orphans, so a recovery only knows the recorded process group.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded-run.ts
//! export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
//! ```

/// Imports `std::cell::{Cell, RefCell}` for this file.
use std::cell::{Cell, RefCell};

/// Imports `std::ffi::{CStr, CString}` for this file.
use std::ffi::{CStr, CString};

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::marker::PhantomData` for this file.
use std::marker::PhantomData;

/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;

/// Imports `std::os::unix::ffi::OsStrExt` for this file.
use std::os::unix::ffi::OsStrExt;

/// Imports `std::os::unix::process::CommandExt` for this file.
use std::os::unix::process::CommandExt;

/// Imports `std::path::PathBuf` for this file.
use std::path::PathBuf;

/// Imports `std::process::{Child, Command, ExitStatus, Output, Stdio}` for this file.
use std::process::{Child, Command, ExitStatus, Output, Stdio};

/// Imports `std::rc::Rc` for this file.
use std::rc::Rc;

/// Imports `std::sync::Arc` for this file.
use std::sync::Arc;

/// Imports `std::sync::atomic::{AtomicBool, Ordering}` for this file.
use std::sync::atomic::{AtomicBool, Ordering};

/// Imports `std::sync::mpsc::{Receiver, RecvTimeoutError}` for this file.
use std::sync::mpsc::{Receiver, RecvTimeoutError};

/// Imports `std::thread` for this file.
use std::thread;

/// Imports `std::time::{Duration, Instant}` for this file.
use std::time::{Duration, Instant};

/// Imports `rustix::fs::{Mode, OFlags, fsync, open}` for this file.
use rustix::fs::{Mode, OFlags, fsync, open};

/// Imports `rustix::process::{Pid, Signal, getpid, kill_process_group}` for this file.
use rustix::process::{Pid, Signal, getpid, kill_process_group};

/// Imports `crate::bounded_contain::Containment` for this file.
use crate::bounded_contain::Containment;

/// Imports `crate::bounded_io::{collect, feed}` for this file.
use crate::bounded_io::{collect, feed};

/// How often the child is asked whether it has ended.
/// Delegated pick of round 6, open to veto: 10 ms keeps the latency of a finished command far under any human-visible delay
/// while costing one `wait` system call per interval.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// How long the direct child is given to be reaped after it was killed.
/// Delegated pick of round 6, open to veto: a killed process is reaped within milliseconds unless it is in an uninterruptible
/// wait, which no bound can fix; 5 s is long against that and short against a person's patience.
/// The time from kill to reap was not measured on slow machines.
const REAP_BOUND: Duration = Duration::from_secs(5);

/// How long the descendants are given to disappear after they were killed.
/// Delegated pick of round 6, open to veto, for the same reason as `REAP_BOUND`.
const QUIESCE_BOUND: Duration = Duration::from_secs(5);

/// How long the output workers are given to report once every writer is gone.
/// Delegated pick of round 6, open to veto: with every writer gone the pipes are at their end at once, so this only needs to
/// cover the scheduling of the worker threads; the largest output (`cargo metadata` of this repository) drains in well under it.
const READER_GRACE: Duration = Duration::from_secs(2);

thread_local! {
    /// The command the user ran,
    /// for the error of a bound that was exceeded.
    static RERUN: RefCell<Option<String>> = const { RefCell::new(None) };

    /// The scopes of the open transactions of this thread, innermost last.
    static SCOPES: RefCell<Vec<(u64, Rc<ScopeState>)>> = const { RefCell::new(Vec::new()) };

    /// The number the next scope gets.
    static NEXT_SCOPE: Cell<u64> = const { Cell::new(0) };
}

/// What a scope remembers.
struct ScopeState {
    /// The file in which the process group of each child started under this scope is recorded.
    log: PathBuf,
    /// Whether a run under this scope could not show that everything it started was stopped.
    lost: Cell<bool>,
}

/// The records and the verdict of one open transaction.
/// It belongs to the thread that opened it: dropping it, in any order, removes only its own entry.
pub struct RunScope {
    /// The number of this scope in the stack.
    id: u64,
    /// What the scope remembers, shared with the runs under it.
    state: Rc<ScopeState>,
    /// Keeps the scope on its thread, because the stack it is in is thread-local.
    _on_one_thread: PhantomData<*const ()>,
}

/// The methods of the type above.
impl RunScope {
    /// What:
    /// `pub fn open(group_log: PathBuf) -> RunScope` makes the runs of this thread record their process groups in `group_log`.
    /// Why:
    /// A recovery that finds a killed run's journal needs to know whether a child of that run may still be running.
    /// The record belongs to the transaction that owns this scope, not to the thread (round 6, 5b2 6).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static open(groupLog: string): RunScope
    /// ```
    pub fn open(group_log: PathBuf) -> RunScope {
        let id: u64 = NEXT_SCOPE.with(|next| { return next.replace(next.get() + 1); });
        let state: Rc<ScopeState> = Rc::new(ScopeState { log: group_log, lost: Cell::new(false) });
        SCOPES.with(|stack| { stack.borrow_mut().push((id, Rc::clone(&state))); });
        return RunScope { id, state, _on_one_thread: PhantomData };
    }

    /// What:
    /// `pub fn writers_may_remain(&self) -> bool` says whether a run under this scope could not show that everything it
    /// started was stopped.
    /// Why:
    /// Then the files of the transaction cannot be rolled back or finished safely, and the command must stop.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// writersMayRemain(): boolean
    /// ```
    pub fn writers_may_remain(&self) -> bool {
        return self.state.lost.get();
    }
}

/// How the scope ends.
impl Drop for RunScope {
    /// What:
    /// `fn drop(&mut self)` removes this scope's entry and no other.
    /// Why:
    /// Transactions may nest and end in any order.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        SCOPES.with(|stack| { stack.borrow_mut().retain(|(id, _)| { return *id != self.id; }); });
    }
}

/// The bound of one run and the words an error uses.
#[derive(Clone, Debug)]
pub struct Bound {
    /// The longest the child may run.
    pub limit: Duration,
    /// What is running,
    /// for the error.
    pub what: String,
    /// The command to run again,
    /// for the error.
    pub rerun: String,
}

/// Why a bounded run did not produce output.
#[derive(Debug)]
pub enum RunError {
    /// The child could not be started or a system call failed.
    Io(
        /// The system error.
        io::Error,
    ),
    /// The bound passed; the child and everything it started were stopped.
    TimedOut {
        /// What was running.
        what: String,
        /// The bound.
        limit: Duration,
        /// The command to run again.
        rerun: String,
    },
    /// Something the child started could not be stopped or shown to be gone.
    NotQuiescent {
        /// What was running.
        what: String,
    },
    /// The run could not be contained, so the child was not started.
    ContainmentUnavailable {
        /// What was to run.
        what: String,
        /// Why containment failed.
        reason: String,
    },
    /// The output could not be read to its end.
    IncompleteOutput {
        /// What was running.
        what: String,
    },
    /// The input could not be delivered to the end.
    IncompleteInput {
        /// What was running.
        what: String,
    },
}

/// How the error reads to a person.
impl std::fmt::Display for RunError {
    /// What:
    /// `fn fmt(&self, formatter)` writes the error with what was running and what it means for the files.
    /// Why:
    /// The texts say only what is established: the command may have changed files before it was stopped.
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
                "{what} did not finish within {limit:?}; it and every process it started were stopped. It may have changed files before it was stopped; this tool did not restore or remove anything because of that. Run {rerun} again when it can finish"
            ),
            RunError::NotQuiescent { what } => write!(
                formatter,
                "{what} started a process that could not be stopped or shown to be gone, so files it may still write cannot be trusted; this tool changed no file because of that and the journal of the run was kept"
            ),
            RunError::ContainmentUnavailable { what, reason } => write!(
                formatter,
                "{what} was not started because this tool could not make itself the parent of everything the command starts ({reason}); without that it cannot show that the command left nothing running"
            ),
            RunError::IncompleteOutput { what } => write!(formatter, "the output of {what} could not be read to its end; the partial output was discarded"),
            RunError::IncompleteInput { what } => write!(formatter, "{what} did not take all of its input, so its answer is not trusted"),
        };
    }
}

/// What:
/// `fn current_scope() -> Option<Rc<ScopeState>>` is the innermost open scope of this thread.
/// Why:
/// The run records into, and reports to, the transaction that is open around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentScope(): ScopeState | undefined
/// ```
fn current_scope() -> Option<Rc<ScopeState>> {
    return SCOPES.with(|stack| { return stack.borrow().last().map(|(_, state)| { return Rc::clone(state); }); });
}

/// What:
/// `fn record_own_group(path: &CStr) -> io::Result<()>` appends the calling process's own id, which is its process group, to the
/// record and forces it to disk.
/// Why:
/// It runs in the forked child before `exec`, so the record exists before the child can write anything.
/// It uses only system calls and a stack buffer, because nothing else is safe there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordOwnGroup(path: string): void
/// ```
fn record_own_group(path: &CStr) -> io::Result<()> {
    let file: OwnedFd = open(path, OFlags::WRONLY | OFlags::APPEND | OFlags::CLOEXEC, Mode::empty()).map_err(io::Error::from)?;
    let mut digits: [u8; 24] = [0; 24];
    let mut position: usize = digits.len() - 1;
    digits[position] = b'\n';
    let mut number: u32 = u32::try_from(getpid().as_raw_nonzero().get()).unwrap_or(0);
    loop {
        position -= 1;
        digits[position] = b'0' + u8::try_from(number % 10).unwrap_or(0);
        number /= 10;
        if number == 0 {
            break;
        }
    }
    let line: &[u8] = &digits[position..];
    let written: usize = rustix::io::write(&file, line).map_err(io::Error::from)?;
    if written != line.len() {
        return Err(io::Error::new(io::ErrorKind::WriteZero, "the process group record was cut short"));
    }
    return fsync(&file).map_err(io::Error::from);
}

/// What:
/// `fn reap(child: &mut Child) -> Option<ExitStatus>` waits for the direct child,
/// a few milliseconds at a time,
/// up to `REAP_BOUND`.
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
/// `fn stop_everything(containment, child, group) -> bool` kills the process group, then every descendant wherever it went,
/// reaps the direct child, and returns `true` only if all of it is shown gone.
/// Why:
/// This is the one way a run is ended early,
/// on timeout and on every error path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopEverything(containment: Containment, child: ChildProcess, group: number): boolean
/// ```
fn stop_everything(containment: &Containment, child: &mut Child, group: Pid) -> bool {
    let _ = kill_process_group(group, Signal::KILL);
    let reaped: bool = reap(child).is_some();
    let direct: Option<i32> = i32::try_from(child.id()).ok();
    let gone: bool = containment.stop_all(direct, QUIESCE_BOUND).unwrap_or(false);
    return gone && reaped;
}

/// What:
/// `fn finished(receiver: &Receiver<io::Result<Vec<u8>>>, what: &str) -> Result<Vec<u8>, RunError>` takes the result of an output worker.
/// Why:
/// A worker that does not report means the output is not complete, and a prefix must never be returned as the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finished(receiver: Promise<Buffer>, what: string): Buffer  // throws RunError
/// ```
fn finished(receiver: &Receiver<io::Result<Vec<u8>>>, what: &str) -> Result<Vec<u8>, RunError> {
    return match receiver.recv_timeout(READER_GRACE) {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(error)) => Err(RunError::Io(error)),
        Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => Err(RunError::IncompleteOutput { what: what.to_string() }),
    };
}

/// Sets the cancel flag when the run ends, however it ends, so no worker outlives the call.
struct CancelOnDrop(Arc<AtomicBool>);

/// How the guard acts.
impl Drop for CancelOnDrop {
    /// What:
    /// `fn drop(&mut self)` tells the pipe workers to stop.
    /// Why:
    /// A worker must not keep a thread, a descriptor and a growing buffer after the call returned.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// What:
/// `fn lose(scope: &Option<Rc<ScopeState>>)` marks the open transaction as one whose writers may remain.
/// Why:
/// The caller of the transaction reads the mark and stops the command instead of rolling back or finishing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lose(scope: ScopeState | undefined): void
/// ```
fn lose(scope: &Option<Rc<ScopeState>>) {
    if let Some(state) = scope {
        state.lost.set(true);
    }
}

/// What:
/// `pub fn output_within(command, input, bound) -> Result<Output, RunError>` runs the command, feeds it `input`, and returns
/// its complete output, or an error naming what was running.
/// Why:
/// Every external process of cargo-shadow runs through here, so nothing waits forever and nothing is left running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
/// ```
pub fn output_within(command: &mut Command, input: Option<&[u8]>, bound: &Bound) -> Result<Output, RunError> {
    let containment: Containment = Containment::begin().map_err(|error| { return RunError::ContainmentUnavailable { what: bound.what.clone(), reason: error.to_string() }; })?;
    let scope: Option<Rc<ScopeState>> = current_scope();
    command.process_group(0).stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(state) = &scope {
        let record: CString = CString::new(state.log.as_os_str().as_bytes()).map_err(|error| { return RunError::Io(io::Error::new(io::ErrorKind::InvalidInput, error)); })?;
        // SAFETY: the closure runs in the forked child before `exec` and calls only `open`, `getpid`, `write` and `fsync` on
        // data that was built before the fork (a `CString` and a stack buffer); it allocates nothing and takes no lock.
        unsafe {
            command.pre_exec(move || { return record_own_group(&record); });
        }
    }
    let mut child: Child = command.spawn().map_err(|error| {
        return RunError::Io(io::Error::new(error.kind(), format!("{error} (a child whose process group cannot be recorded is not started)")));
    })?;
    let group: Pid = Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)).ok_or_else(|| { return RunError::Io(io::Error::other("the child has no process id")); })?;
    let cancel: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let _cancel_on_exit: CancelOnDrop = CancelOnDrop(Arc::clone(&cancel));
    let writer: Option<Receiver<io::Result<()>>> = match (input, child.stdin.take()) {
        (Some(bytes), Some(stdin)) => Some(feed(OwnedFd::from(stdin), bytes.to_vec(), Arc::clone(&cancel))),
        _ => None,
    };
    let stdout_pipe: OwnedFd = OwnedFd::from(child.stdout.take().ok_or_else(|| { return RunError::Io(io::Error::other("no standard output pipe")); })?);
    let stderr_pipe: OwnedFd = OwnedFd::from(child.stderr.take().ok_or_else(|| { return RunError::Io(io::Error::other("no standard error pipe")); })?);
    let stdout = collect(stdout_pipe, Arc::clone(&cancel));
    let stderr = collect(stderr_pipe, Arc::clone(&cancel));
    let deadline: Instant = Instant::now() + bound.limit;
    let status: ExitStatus = loop {
        match child.try_wait() {
            Ok(Some(found)) => break found,
            Ok(None) => {}
            Err(error) => {
                if !stop_everything(&containment, &mut child, group) {
                    lose(&scope);
                    return Err(RunError::NotQuiescent { what: bound.what.clone() });
                }
                return Err(RunError::Io(error));
            }
        }
        if Instant::now() >= deadline {
            if !stop_everything(&containment, &mut child, group) {
                lose(&scope);
                return Err(RunError::NotQuiescent { what: bound.what.clone() });
            }
            return Err(RunError::TimedOut { what: bound.what.clone(), limit: bound.limit, rerun: bound.rerun.clone() });
        }
        thread::sleep(POLL_INTERVAL);
    };
    let _ = kill_process_group(group, Signal::KILL);
    if !containment.stop_all(None, QUIESCE_BOUND).unwrap_or(false) {
        lose(&scope);
        return Err(RunError::NotQuiescent { what: bound.what.clone() });
    }
    let out: Vec<u8> = finished(&stdout, &bound.what)?;
    let err: Vec<u8> = finished(&stderr, &bound.what)?;
    if let Some(receiver) = writer {
        match receiver.recv_timeout(READER_GRACE) {
            Ok(Ok(())) => {}
            Ok(Err(error)) if error.kind() == io::ErrorKind::BrokenPipe => {}
            Ok(Err(error)) => return Err(RunError::Io(error)),
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => return Err(RunError::IncompleteInput { what: bound.what.clone() }),
        }
    }
    return Ok(Output { status, stdout: out, stderr: err });
}

/// What:
/// `pub fn mark_scope_lost_for_test()` marks the innermost open scope of this thread as one whose writers may remain.
/// Why:
/// Tests of the callers need the verdict a run gives when a descendant cannot be stopped, which a real run cannot be made to give.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function markScopeLostForTest(): void
/// ```
#[cfg(test)]
pub fn mark_scope_lost_for_test() {
    lose(&current_scope());
}

/// What:
/// `pub fn set_rerun(command: Option<String>)` names the command the user ran,
/// for the errors of runs started by this thread.
/// Why:
/// An error that says "run it again" must say which command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function setRerun(command?: string): void
/// ```
pub fn set_rerun(command: Option<String>) {
    RERUN.with(|slot| { *slot.borrow_mut() = command; });
}

/// What:
/// `pub fn rerun_text() -> String` is the text for the command to run again.
/// Why:
/// Bounds are built far from the shim that knows the arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function rerunText(): string
/// ```
pub fn rerun_text() -> String {
    return RERUN.with(|slot| { return slot.borrow().clone().map(|command| { return format!("`{command}`"); }).unwrap_or_else(|| { return "the command you ran".to_string(); }); });
}
