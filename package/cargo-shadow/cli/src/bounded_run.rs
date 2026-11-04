//! What:
//! Runs a child process and collects its output within a time bound,
//! with everything the child started contained in a control group.
//! Why:
//! The owner's rule of 2026-11-03 is "nothing should ever wait forever or indefinitely".
//! Round 7 (sixth review 6a R1 to R4, R10; 6b2 2; 6c R2):
//! the child and everything it starts run in a control group this program makes and records in the journal before the child
//! starts (`control_group.rs`);
//! at the bound,
//! and after a normal exit,
//! the group is killed and the run waits for the kernel's own statement that no thread is left (`populated 0`).
//! A call that returns `Ok` has that statement.
//! The spawning process enters the group for the span of one `spawn` and leaves it again,
//! so no hook runs in the child and nothing is appended to the caller's `Command`.
//! Where no group can be had (the own group is not writable,
//! no cgroup v2),
//! the child runs in a process group of its own,
//! a one-line notice names the missing capability,
//! and no quiescence is claimed for what leaves the process group (the weaker path,
//! the coordinator's pick of round 7,
//! veto open).
//! Output is returned only when it is complete:
//! a prefix is an error (review finding 4c F4).
//! The limit:
//! a process that is held in an uninterruptible wait cannot be killed,
//! and that is `NotQuiescent`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded-run.ts
//! export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
//! ```

/// Imports `std::cell::{Cell, RefCell}` for this file.
use std::cell::{Cell, RefCell};

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::marker::PhantomData` for this file.
use std::marker::PhantomData;

/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;

/// Imports `std::os::unix::process::CommandExt` for this file.
use std::os::unix::process::CommandExt;

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

/// Imports `std::process::{Child, Command, ExitStatus, Output, Stdio}` for this file.
use std::process::{Child, Command, ExitStatus, Output, Stdio};

/// Imports `std::rc::Rc` for this file.
use std::rc::Rc;

/// Imports `std::sync::{Arc, Mutex, MutexGuard, Once, TryLockError}` for this file.
use std::sync::{Arc, Mutex, MutexGuard, Once, TryLockError};

/// Imports `std::sync::atomic::{AtomicBool, Ordering}` for this file.
use std::sync::atomic::{AtomicBool, Ordering};

/// Imports `std::sync::mpsc::{Receiver, RecvTimeoutError}` for this file.
use std::sync::mpsc::{Receiver, RecvTimeoutError};

/// Imports `std::thread` for this file.
use std::thread;

/// Imports `std::time::{Duration, Instant}` for this file.
use std::time::{Duration, Instant};

/// Imports `rustix::fs::{FileType, Mode, OFlags, fstat, fsync, open}` for this file.
use rustix::fs::{FileType, Mode, OFlags, fstat, fsync, open};

/// Imports `rustix::event::{PollFd, PollFlags, Timespec, poll}` for this file.
use rustix::event::{PollFd, PollFlags, Timespec, poll};

/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;

/// Imports `rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group}` for this file.
use rustix::process::{Pid, PidfdFlags, Signal, kill_process_group, pidfd_open, test_kill_process_group};

/// Imports `crate::bounded_io::{collect, feed}` for this file.
use crate::bounded_io::{collect, feed};

/// Imports the control group pieces for this file.
use crate::control_group::{ControlGroup, GroupUnavailable, Record, own_group_directory};

/// How often the child is asked whether it has ended.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open:
/// 10 ms keeps the latency of a finished command far under any human-visible delay
/// while costing one `wait` system call per interval.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// How long the direct child is given to be reaped after it was killed.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open:
/// a killed process is reaped within milliseconds unless it is in an uninterruptible wait,
/// which no bound can fix;
/// the kill of a control group and its `populated 0` took 1 ms to 5 ms in the probes of 2026-11-04
/// (`doc/planning/cargo-shadow-update.md`, "Round 7: control group probes"), so 5 s is a thousand times that.
const REAP_BOUND: Duration = Duration::from_secs(5);

/// How long the control group is given to become unpopulated after it was killed.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open,
/// with the measurement of `REAP_BOUND` beside it:
/// 1 ms to 5 ms in the probes, 5 s as the bound that ends the wait for a member that cannot be killed.
const QUIESCE_BOUND: Duration = Duration::from_secs(5);

/// How long the output workers are given to report once every writer is gone.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open:
/// with every writer gone the pipes are at their end at once,
/// so this only needs to cover the scheduling of the worker threads;
/// the largest output (`cargo metadata` of this repository) drains in well under it.
const READER_GRACE: Duration = Duration::from_secs(2);

/// Keeps two threads from entering a group at the same time.
/// A process that is inside a group while it spawns would put any other thread's child into that group too;
/// the window is one `spawn`, and a thread that cannot enter before its bound has passed gives up.
static SPAWN_WINDOW: Mutex<()> = Mutex::new(());

/// Makes the one-line notice of the weaker path appear once per process.
static WEAKER_NOTICE: Once = Once::new();

thread_local! {
    /// The command the user ran,
    /// for the error of a bound that was exceeded.
    static RERUN: RefCell<Option<String>> = const { RefCell::new(None) };

    /// The scopes of the open transactions of this thread,
    /// innermost last.
    static SCOPES: RefCell<Vec<(u64, Rc<ScopeState>)>> = const { RefCell::new(Vec::new()) };

    /// The number the next scope gets.
    static NEXT_SCOPE: Cell<u64> = const { Cell::new(0) };
}

#[cfg(test)]
thread_local! {
    /// The directory groups are made in instead of the own group,
    /// so a test can make the host look like one that cannot give a group.
    static GROUP_PARENT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// What a scope remembers.
struct ScopeState {
    /// The file in which the containment of each child started under this scope is recorded.
    log: PathBuf,
    /// Whether a run under this scope could not show that everything it started was stopped.
    lost: Cell<bool>,
    /// Whether a run under this scope was contained by a process group only.
    process_group_only: Cell<bool>,
}

/// The records and the verdict of one open transaction.
/// It belongs to the thread that opened it:
/// dropping it,
/// in any order,
/// removes only its own entry.
pub struct RunScope {
    /// The number of this scope in the stack.
    id: u64,
    /// What the scope remembers,
    /// shared with the runs under it.
    state: Rc<ScopeState>,
    /// Keeps the scope on its thread,
    /// because the stack it is in is thread-local.
    _on_one_thread: PhantomData<*const ()>,
}

/// The methods of the type above.
impl RunScope {
    /// What:
    /// `pub fn open(group_log: PathBuf) -> RunScope` makes the runs of this thread record their process groups in `group_log`.
    /// Why:
    /// A recovery that finds a killed run's journal needs to know whether a child of that run may still be running.
    /// The record belongs to the transaction that owns this scope,
    /// not to the thread (round 6,
    /// 5b2 6).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static open(groupLog: string): RunScope
    /// ```
    pub fn open(group_log: PathBuf) -> RunScope {
        let id: u64 = NEXT_SCOPE.with(|next| { return next.replace(next.get() + 1); });
        let state: Rc<ScopeState> = Rc::new(ScopeState { log: group_log, lost: Cell::new(false), process_group_only: Cell::new(false) });
        SCOPES.with(|stack| { stack.borrow_mut().push((id, Rc::clone(&state))); });
        return RunScope { id, state, _on_one_thread: PhantomData };
    }

    /// What:
    /// `pub fn writers_may_remain(&self) -> bool` says whether a run under this scope could not show that everything it
    /// started was stopped.
    /// Why:
    /// Then the files of the transaction cannot be rolled back or finished safely,
    /// and the command must stop.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// writersMayRemain(): boolean
    /// ```
    pub fn writers_may_remain(&self) -> bool {
        return self.state.lost.get();
    }

    /// What:
    /// `pub fn process_group_only(&self) -> bool` says whether a run under this scope was contained by a process group only,
    /// because no control group could be had.
    /// Why:
    /// Quiescence was not shown for what leaves a process group,
    /// and a caller may say so.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// processGroupOnly(): boolean
    /// ```
    pub fn process_group_only(&self) -> bool {
        return self.state.process_group_only.get();
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
    /// The bound passed;
    /// the child and everything it started were stopped.
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
    /// The texts say only what is established:
    /// the command may have changed files before it was stopped.
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
            RunError::IncompleteOutput { what } => write!(formatter, "the output of {what} could not be read to its end; the partial output was discarded"),
            RunError::IncompleteInput { what } => write!(formatter, "{what} did not take all of its input, so its answer is not trusted"),
        };
    }
}

/// What:
/// `fn current_scope() -> Option<Rc<ScopeState>>` is the innermost open scope of this thread.
/// Why:
/// The run records into,
/// and reports to,
/// the transaction that is open around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentScope(): ScopeState | undefined
/// ```
fn current_scope() -> Option<Rc<ScopeState>> {
    return SCOPES.with(|stack| { return stack.borrow().last().map(|(_, state)| { return Rc::clone(state); }); });
}

/// What:
/// `fn append_record(log: &Path, record: &Record) -> io::Result<()>` appends one line to the journal's `groups` file and forces it to
/// disk.
/// Why:
/// The line must be durable before the child it describes starts.
/// The file is opened without waiting and without following a link,
/// and must be a regular file,
/// so a record path replaced by a pipe fails at once instead of blocking the run (6a R4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function appendRecord(log: string, record: Record): void
/// ```
fn append_record(log: &Path, record: &Record) -> io::Result<()> {
    let file: OwnedFd = open(log, OFlags::WRONLY | OFlags::APPEND | OFlags::CLOEXEC | OFlags::NONBLOCK | OFlags::NOFOLLOW, Mode::empty()).map_err(io::Error::from)?;
    let kind: FileType = FileType::from_raw_mode(fstat(&file).map_err(io::Error::from)?.st_mode);
    if kind != FileType::RegularFile {
        return Err(io::Error::from(Errno::INVAL));
    }
    let line: String = format!("{record}\n");
    let written: usize = rustix::io::write(&file, line.as_bytes()).map_err(io::Error::from)?;
    if written != line.len() {
        return Err(io::Error::new(io::ErrorKind::WriteZero, "the containment record was cut short"));
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
/// `fn wait_for_exit(watch: Option<&OwnedFd>, remaining: Duration)` waits until the child ends or `remaining` passes.
/// Why:
/// A pidfd wakes at the exit;
/// without one (a kernel before 5.3) the wait is a sleep of `POLL_INTERVAL`.
/// Measured on 2026-11-04 (debug build, 40 runs of `true`): the sleep-polling run took 11.3 ms at best,
/// almost all of it the poll interval.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitForExit(watch: Descriptor | undefined, remainingMs: number): void
/// ```
fn wait_for_exit(watch: Option<&OwnedFd>, remaining: Duration) {
    let Some(descriptor) = watch else {
        thread::sleep(POLL_INTERVAL.min(remaining));
        return;
    };
    let timeout: Timespec = Timespec { tv_sec: i64::try_from(remaining.as_secs()).unwrap_or(0), tv_nsec: i64::from(remaining.subsec_nanos()) };
    let mut watched = [PollFd::new(descriptor, PollFlags::IN)];
    // An interrupted or failed poll only means the next turn of the loop asks `try_wait` again.
    let _ = poll(&mut watched, Some(&timeout));
}

/// How a run is contained.
enum Containment {
    /// In a control group: quiescence is shown by the kernel.
    Group(ControlGroup),
    /// In a process group of its own (the child's process id is the group): the weaker path.
    ProcessGroup,
}

/// What:
/// `fn group_parent() -> io::Result<PathBuf>` is the directory new groups are made in.
/// Why:
/// It is the group this process is in;
/// tests replace it to make the host look like one without delegation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupParent(): string
/// ```
fn group_parent() -> io::Result<PathBuf> {
    #[cfg(test)]
    if let Some(parent) = GROUP_PARENT_OVERRIDE.with(|slot| { return slot.borrow().clone(); }) {
        return Ok(parent);
    }
    return own_group_directory();
}

/// What:
/// `pub fn use_group_parent_for_test(parent: Option<PathBuf>)` makes the runs of this thread make their groups in `parent`
/// (`None` restores the own group).
/// Why:
/// A plain directory is not a control group,
/// which is the host without delegation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function useGroupParentForTest(parent?: string): void
/// ```
#[cfg(test)]
pub fn use_group_parent_for_test(parent: Option<PathBuf>) {
    GROUP_PARENT_OVERRIDE.with(|slot| { *slot.borrow_mut() = parent; });
}

/// What:
/// `pub(crate) fn weaker_notice(unavailable: &GroupUnavailable) -> String` is the one-line notice of the weaker path.
/// Why:
/// It names the missing capability and says what cannot be shown,
/// so a person knows the guarantee they have (the coordinator's pick of round 7, veto open).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function weakerNotice(unavailable: GroupUnavailable): string
/// ```
pub(crate) fn weaker_notice(unavailable: &GroupUnavailable) -> String {
    return format!("cargo-shadow: no writable control group ({unavailable}); a process that leaves the process group cannot be stopped or found, so quiescence is not shown");
}

/// What:
/// `fn contain() -> Containment` makes a control group,
/// or says in one line which capability is missing and falls back to a process group.
/// Why:
/// The notice is printed once per process;
/// it names the capability, and says what cannot be shown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function contain(): Containment
/// ```
fn contain() -> Containment {
    let made: Result<ControlGroup, GroupUnavailable> = match group_parent() {
        Ok(parent) => ControlGroup::create_in(&parent),
        Err(error) => Err(GroupUnavailable { reason: format!("the control group of this process cannot be found ({error})") }),
    };
    return match made {
        Ok(group) => Containment::Group(group),
        Err(unavailable) => {
            WEAKER_NOTICE.call_once(|| { eprintln!("{}", weaker_notice(&unavailable)); });
            Containment::ProcessGroup
        }
    };
}

/// What:
/// `fn take_window(deadline: Instant) -> Option<MutexGuard<'static, ()>>` waits for the spawn window until `deadline`.
/// Why:
/// Nothing waits without a bound;
/// the holder is inside one `spawn`, so the wait is short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeWindow(deadline: number): Guard | undefined
/// ```
fn take_window(deadline: Instant) -> Option<MutexGuard<'static, ()>> {
    loop {
        match SPAWN_WINDOW.try_lock() {
            Ok(held) => return Some(held),
            Err(TryLockError::Poisoned(poisoned)) => return Some(poisoned.into_inner()),
            Err(TryLockError::WouldBlock) => {}
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(Duration::from_millis(1));
    }
}

/// What:
/// `fn stop_process_group(child: &mut Child, group: Pid) -> bool` kills the process group,
/// reaps the direct child,
/// and returns `true` once no member of the group is left.
/// Why:
/// This is all the weaker path can show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopProcessGroup(child: ChildProcess, group: number): boolean
/// ```
fn stop_process_group(child: &mut Child, group: Pid) -> bool {
    let reaped: bool = {
        let _ = kill_process_group(group, Signal::KILL);
        reap(child).is_some()
    };
    let deadline: Instant = Instant::now() + QUIESCE_BOUND;
    loop {
        if test_kill_process_group(group) == Err(Errno::SRCH) {
            return reaped;
        }
        if Instant::now() >= deadline {
            return false;
        }
        let _ = kill_process_group(group, Signal::KILL);
        thread::sleep(POLL_INTERVAL);
    }
}

/// What:
/// `fn stop_everything(containment, child) -> bool` ends the run early or at its normal exit:
/// it kills what the containment holds,
/// reaps the direct child,
/// and returns `true` only if all of it is shown gone.
/// Why:
/// This is the one way a run is ended,
/// on timeout, on a normal exit and on every error path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopEverything(containment: Containment, child: ChildProcess): boolean
/// ```
fn stop_everything(containment: &Containment, child: &mut Child) -> bool {
    return match containment {
        Containment::Group(group) => {
            let empty: bool = group.stop(QUIESCE_BOUND).unwrap_or(false);
            let reaped: bool = reap(child).is_some();
            if empty && let Err(error) = group.remove() {
                eprintln!("cargo-shadow: warning: the empty control group {} could not be removed ({error})", group.directory().display());
            }
            empty && reaped
        }
        Containment::ProcessGroup => match Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)) {
            Some(group) => stop_process_group(child, group),
            None => false,
        },
    };
}

/// What:
/// `fn finished(receiver: &Receiver<io::Result<Vec<u8>>>, what: &str) -> Result<Vec<u8>, RunError>` takes the result of an output worker.
/// Why:
/// A worker that does not report means the output is not complete,
/// and a prefix must never be returned as the output.
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

/// Sets the cancel flag when the run ends,
/// however it ends,
/// so no worker outlives the call.
struct CancelOnDrop(
    /// The flag the pipe workers look at.
    Arc<AtomicBool>,
);

/// How the guard acts.
impl Drop for CancelOnDrop {
    /// What:
    /// `fn drop(&mut self)` tells the pipe workers to stop.
    /// Why:
    /// A worker must not keep a thread,
    /// a descriptor and a growing buffer after the call returned.
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
/// `pub fn output_within(command, input, bound) -> Result<Output, RunError>` runs the command,
/// feeds it `input`,
/// and returns
/// its complete output,
/// or an error naming what was running.
/// Why:
/// Every external process of cargo-shadow runs through here,
/// so nothing waits forever and nothing is left running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
/// ```
pub fn output_within(command: &mut Command, input: Option<&[u8]>, bound: &Bound) -> Result<Output, RunError> {
    let scope: Option<Rc<ScopeState>> = current_scope();
    let deadline: Instant = Instant::now() + bound.limit;
    command.stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
    let containment: Containment = contain();
    let spawned: Result<Child, RunError> = start_contained(command, &containment, scope.as_ref(), deadline, bound);
    let mut child: Child = match spawned {
        Ok(found) => found,
        Err(error) => {
            // Nothing was started, or the child was already stopped; an empty group is removed so none is left behind.
            if let Containment::Group(group) = &containment {
                let _ = group.remove();
            }
            return Err(error);
        }
    };
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
    // A descriptor that becomes readable when the child ends, so the wait wakes at the exit instead of at the next poll.
    let exit_watch: Option<OwnedFd> = Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)).and_then(|pid| { return pidfd_open(pid, PidfdFlags::empty()).ok(); });
    let status: ExitStatus = loop {
        match child.try_wait() {
            Ok(Some(found)) => break found,
            Ok(None) => {}
            Err(error) => {
                if !stop_everything(&containment, &mut child) {
                    lose(&scope);
                    return Err(RunError::NotQuiescent { what: bound.what.clone() });
                }
                return Err(RunError::Io(error));
            }
        }
        if Instant::now() >= deadline {
            if !stop_everything(&containment, &mut child) {
                lose(&scope);
                return Err(RunError::NotQuiescent { what: bound.what.clone() });
            }
            return Err(RunError::TimedOut { what: bound.what.clone(), limit: bound.limit, rerun: bound.rerun.clone() });
        }
        wait_for_exit(exit_watch.as_ref(), deadline.saturating_duration_since(Instant::now()));
    };
    // A normal exit still stops what the child left behind, and the call returns only with the proof that nothing is left.
    if !stop_everything(&containment, &mut child) {
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
/// `fn start_contained(command, containment, scope, deadline, bound) -> Result<Child, RunError>` records the containment in the
/// journal and starts the child inside it.
/// Why:
/// For a control group the record is forced to disk before the child exists,
/// and a record that cannot be written means the child is not started.
/// For the weaker path the process group is only known once the child exists,
/// so the record follows the start and a child whose record cannot be written is stopped at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startContained(command: Command, containment: Containment, scope: ScopeState | undefined, deadline: number, bound: Bound): ChildProcess  // throws RunError
/// ```
fn start_contained(command: &mut Command, containment: &Containment, scope: Option<&Rc<ScopeState>>, deadline: Instant, bound: &Bound) -> Result<Child, RunError> {
    match containment {
        Containment::Group(group) => {
            if let Some(state) = scope {
                append_record(&state.log, &group.record()).map_err(|error| {
                    return RunError::Io(io::Error::new(error.kind(), format!("{error} (a child whose control group cannot be recorded is not started)")));
                })?;
            }
            let Some(_window) = take_window(deadline) else {
                return Err(RunError::TimedOut { what: bound.what.clone(), limit: bound.limit, rerun: bound.rerun.clone() });
            };
            return group.spawn(command).map_err(RunError::Io);
        }
        Containment::ProcessGroup => {
            if let Some(state) = scope {
                state.process_group_only.set(true);
            }
            command.process_group(0);
            // The child's group is its process id, known only after the spawn: a "starting" line is durable before it, so a
            // wrapper killed in between is seen by a recovery as one that may have a child running (round 7, advisor).
            if let Some(state) = scope {
                append_record(&state.log, &Record::ProcessGroupStarting).map_err(|error| {
                    return RunError::Io(io::Error::new(error.kind(), format!("{error} (a child whose process group cannot be announced is not started)")));
                })?;
            }
            let mut child: Child = command.spawn().map_err(RunError::Io)?;
            let leader: i32 = i32::try_from(child.id()).unwrap_or(0);
            if let Some(state) = scope
                && let Err(error) = append_record(&state.log, &Record::ProcessGroup(leader))
            {
                let stopped: bool = match Pid::from_raw(leader) {
                    Some(group) => stop_process_group(&mut child, group),
                    None => false,
                };
                if !stopped {
                    lose(&Some(Rc::clone(state)));
                }
                return Err(RunError::Io(io::Error::new(error.kind(), format!("{error} (the child was stopped because its process group could not be recorded)"))));
            }
            return Ok(child);
        }
    }
}

/// What:
/// `pub fn mark_scope_lost_for_test()` marks the innermost open scope of this thread as one whose writers may remain.
/// Why:
/// Tests of the callers need the verdict a run gives when a descendant cannot be stopped,
/// which a real run cannot be made to give.
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
