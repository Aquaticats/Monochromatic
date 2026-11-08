//! What:
//! Runs a child process and collects its output within a time bound,
//! with everything the child started contained in a delegated control group.
//! Why:
//! The owner's rule of 2026-11-03 is "nothing should ever wait forever or indefinitely".
//! Round 8 (contract `doc/planning/cargo-shadow-guarantees.md`):
//! the child runs as `systemd-run --user --scope --unit=<name> --property=Delegate=yes --collect -- <program>`,
//! so the manager puts it in a scope of its own (`control_group.rs`) and no thread of this program is ever moved between groups;
//! the scope's record is forced to disk before the program starts;
//! at the bound,
//! and after a normal exit,
//! the direct child is killed and reaped,
//! the scope is killed,
//! and the run waits for the kernel's own statement that no thread is left (`populated 0`) or for the verified absence of the
//! scope.
//! A call that returns `Ok` has that statement,
//! and every failure after the program started that cannot show it is `NotQuiescent`.
//! Where no scope can be had (no user manager,
//! no cgroup v2,
//! an older kernel),
//! the child runs in a process group of its own,
//! a one-line notice names the missing capability,
//! the journal records `pgroup starting` before the start and the group after,
//! and no quiescence is claimed for what leaves the process group:
//! a recovery of such a journal waits for an acknowledgement.
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


/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{PathBuf};

/// Imports `std::process::{Child, Command, ExitStatus, Output, Stdio}` for this file.
use std::process::{Child, Command, ExitStatus, Output, Stdio};

/// Imports `std::rc::Rc` for this file.
use std::rc::Rc;

/// Imports `std::sync::Arc` for this file.
use std::sync::Arc;

/// Imports `std::sync::atomic::{AtomicBool, Ordering}` for this file.
use std::sync::atomic::{AtomicBool};

/// Imports `std::sync::mpsc::{Receiver, RecvTimeoutError}` for this file.
use std::sync::mpsc::{Receiver, RecvTimeoutError};


/// Imports `std::time::{Duration, Instant}` for this file.
use std::time::{Duration, Instant};




/// Imports `rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group}` for this file.
use rustix::process::{Pid};

/// Imports `crate::bounded_io::{collect, feed}` for this file.
use crate::bounded_io::{collect, feed};


/// The containment of a run:
/// its choice,
/// start and stop (split out of the bounded run module).
pub(crate) use crate::bounded_run_containment::{wait_for_exit, Containment, contain, stop_everything, settle, fail_started, finished, CancelOnDrop, lose, reason_of, open_pidfd, start_contained};
/// The items that only the tests of the bounded run use.
#[cfg(test)]
pub(crate) use crate::bounded_run_containment::weaker_notice;

/// The write of one containment record (split out of the bounded run module).
pub(crate) use crate::bounded_run_record_writes::{append_record, append_record_with};

/// The cache of the control group route (split out of the bounded run module).
#[cfg(not(test))]
pub(crate) use crate::bounded_run_route_cache::{find_route};
/// The test-only items of the same module.
#[cfg(test)]
pub(crate) use crate::bounded_run_route_cache::{RouteCache, find_route, use_route_settings_for_test};

/// The route types of the test-only route override.
#[cfg(test)]
use crate::control_group::RouteSettings;

/// How often the child is asked whether it has ended.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open:
/// 10 ms keeps the latency of a finished command far under any human-visible delay
/// while costing one `wait` system call per interval.
pub(crate) const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// How long the direct child is given to be reaped after it was killed.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open:
/// a killed process is reaped within milliseconds unless it is in an uninterruptible wait,
/// which no bound can fix;
/// the kill of a control group and its `populated 0` took 1 ms to 5 ms in the probes of 2026-11-04
/// (`doc/planning/cargo-shadow-update.md`,
/// "Round 7:
/// control group probes"),
/// so 5 s is a thousand times that.
pub(crate) const REAP_BOUND: Duration = Duration::from_secs(5);

/// How long the control group is given to become unpopulated after it was killed.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open,
/// with the measurement of `REAP_BOUND` beside it:
/// 1 ms to 5 ms in the probes,
/// 5 s as the bound that ends the wait for a member that cannot be killed.
pub(crate) const QUIESCE_BOUND: Duration = Duration::from_secs(5);

/// How long the output workers are given to report once every writer is gone.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open:
/// with every writer gone the pipes are at their end at once,
/// so this only needs to cover the scheduling of the worker threads;
/// the largest output (`cargo metadata` of this repository) drains in well under it.
pub(crate) const READER_GRACE: Duration = Duration::from_secs(2);

/// Makes the one-line notice of the weaker path appear once per process.
/// It is a flag that one caller wins with a swap,
/// not a `Once`:
/// a caller that loses returns at once,
/// where `Once::call_once` would make
/// it wait,
/// with no deadline,
/// for the winner's write to standard error (fifth certification pass,
/// V6:
/// no wait the runner controls may
/// depend on another run's slow write).
pub(crate) static WEAKER_NOTICE_GIVEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A test holds this lock to keep the winner of the notice inside its emission,
/// so that another run can show it does not wait for it.
#[cfg(test)]
pub(crate) static NOTICE_GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Set when the winner of the notice reached its emission (a test build only).
#[cfg(test)]
pub(crate) static NOTICE_ENTERED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
    /// The settings the route is discovered with instead of the ordinary ones,
    /// so a test can make the host look like one that lacks a capability.
    pub(crate) static ROUTE_OVERRIDE: RefCell<Option<RouteSettings>> = const { RefCell::new(None) };

    /// A hook that runs inside the start of a run,
    /// after the scope's record is on disk and before the program starts,
    /// so a test can overlap other work with the start at a boundary of its own choosing.
    pub(crate) static START_HOOK: RefCell<Option<Rc<dyn Fn()>>> = const { RefCell::new(None) };

    /// A hook that runs right after the scope's launcher was spawned and before the launcher record is appended,
    /// with the launcher's process id,
    /// so a test can kill the wrapper inside that window.
    static AFTER_LAUNCH_HOOK: RefCell<Option<Rc<dyn Fn(u32)>>> = const { RefCell::new(None) };

    /// A hook that runs right after the program has started and decides whether the completion of the startup fails:
    /// it returns
    /// `true` to make it fail,
    /// after the test has waited for whatever it needs to be ready (a writer,
    /// for one).
    static STARTUP_HOOK: RefCell<Option<Rc<dyn Fn() -> bool>>> = const { RefCell::new(None) };

    /// The ordered events of the recording and the launching of the runs of this thread,
    /// for the tests of their order.
    static TRACE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// What:
/// `fn scope_state_of(entry: &(u64, Rc<ScopeState>)) -> Rc<ScopeState>` clones the state of one stack entry.
/// Why:
/// `Option::map` takes it by path instead of a closure.
fn scope_state_of(entry: &(u64, Rc<ScopeState>)) -> Rc<ScopeState> {
    return Rc::clone(&entry.1);
}

/// What:
/// `fn trace(event: fmt::Arguments<'_>)` appends an event to this thread's trace in a test build and does nothing otherwise.
/// Why:
/// The order record write,
/// durability barrier,
/// launch is a property of the real boundaries,
/// so the tests read an ordered trace of
/// them (certification group 2).
/// The event is a `format_args!` value:
/// nothing is formatted unless a test build turns it into text,
/// and a production build drops it unread.
/// An event whose arguments would compute something (read a file,
/// call a function with a cost) is not passed here;
/// its statement carries `#[cfg(test)]` and calls `trace_text`,
/// so a production build never evaluates it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trace(event: string): void  // the argument is a template that is only rendered in a test build
/// ```
#[cfg(test)]
pub(crate) fn trace(event: std::fmt::Arguments<'_>) {
    trace_text(event.to_string());
}

/// What:
/// `fn trace_text(text: String)` appends an already formatted event to this thread's trace.
/// Why:
/// The statements that compute their text are compiled in a test build only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function traceText(text: string): void
/// ```
#[cfg(test)]
pub(crate) fn trace_text(text: String) {
    TRACE.with(|slot| { slot.borrow_mut().push(text); });
}

/// The production build keeps no trace.
#[cfg(not(test))]
pub(crate) fn trace(_event: std::fmt::Arguments<'_>) {}

/// What:
/// `pub(crate) fn take_trace_for_test() -> Vec<String>` returns and clears this thread's trace.
/// Why:
/// A test reads the events of the run it just made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function takeTraceForTest(): string[]
/// ```
#[cfg(test)]
pub(crate) fn take_trace_for_test() -> Vec<String> {
    return TRACE.with(|slot| { return std::mem::take(&mut *slot.borrow_mut()); });
}

/// What:
/// `pub(crate) fn on_startup_for_test(hook: Option<Rc<dyn Fn() -> bool>>)` sets the hook that runs after the program started and may
/// make the completion of the startup fail.
/// Why:
/// The row A2/R4 requires a failure after the child and a writer are running to be as conservative as a failed stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function onStartupForTest(hook?: () => boolean): void
/// ```
#[cfg(test)]
pub(crate) fn on_startup_for_test(hook: Option<Rc<dyn Fn() -> bool>>) {
    STARTUP_HOOK.with(|slot| { *slot.borrow_mut() = hook; });
}

/// What a scope remembers.
pub(crate) struct ScopeState {
    /// The file in which the containment of each child started under this scope is recorded.
    pub(crate) log: PathBuf,
    /// Whether a run under this scope could not show that everything it started was stopped.
    pub(crate) lost: Cell<bool>,
    /// Whether a run under this scope was contained by a process group only.
    pub(crate) process_group_only: Cell<bool>,
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
        let id: u64 = NEXT_SCOPE.get();
        NEXT_SCOPE.set(id + 1);
        let state: Rc<ScopeState> = Rc::new(ScopeState { log: group_log, lost: Cell::new(false), process_group_only: Cell::new(false) });
        // The stack is taken out of its cell and put back, so no closure holds the borrow; nothing else runs in between.
        let mut stack: Vec<(u64, Rc<ScopeState>)> = SCOPES.take();
        stack.push((id, Rc::clone(&state)));
        SCOPES.set(stack);
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
        // Taken out of its cell and put back: nothing else runs in between.
        let mut stack: Vec<(u64, Rc<ScopeState>)> = SCOPES.take();
        let mut index: usize = 0;
        while index < stack.len() {
            if stack[index].0 == self.id {
                stack.remove(index);
            } else {
                index += 1;
            }
        }
        SCOPES.set(stack);
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
    // Taken out of its cell and put back: nothing else runs in between.
    let stack: Vec<(u64, Rc<ScopeState>)> = SCOPES.take();
    let innermost: Option<Rc<ScopeState>> = stack.last().map(scope_state_of);
    SCOPES.set(stack);
    return innermost;
}

/// What:
/// `pub fn on_start_for_test(hook: Option<Rc<dyn Fn()>>)` sets the hook that runs inside the start of each run of this thread.
/// Why:
/// The spawn-isolation tests overlap an unrelated spawn and a second run with the start of the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function onStartForTest(hook?: () => void): void
/// ```
#[cfg(test)]
pub(crate) fn on_start_for_test(hook: Option<Rc<dyn Fn()>>) {
    START_HOOK.with(|slot| { *slot.borrow_mut() = hook; });
}

/// What:
/// `pub fn after_launch_for_test(hook: Option<Rc<dyn Fn(u32)>>)` sets the hook that runs after the scope's launcher was spawned and
/// before its record is appended,
/// with the launcher's process id.
/// Why:
/// The wrapper's pending-launcher window is real,
/// and a test kills a real wrapper inside it (sixth pass,
/// group 12).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function afterLaunchForTest(hook?: (pid: number) => void): void
/// ```
#[cfg(test)]
pub(crate) fn after_launch_for_test(hook: Option<Rc<dyn Fn(u32)>>) {
    AFTER_LAUNCH_HOOK.with(|slot| { *slot.borrow_mut() = hook; });
}

/// What:
/// `fn run_after_launch_hook(pid: u32)` runs the pending-launcher hook,
/// if any.
/// Why:
/// The production build has no hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runAfterLaunchHook(pid: number): void
/// ```
pub(crate) fn run_after_launch_hook(pid: u32) {
    #[cfg(test)]
    if let Some(hook) = AFTER_LAUNCH_HOOK.with(|slot| { return slot.borrow().clone(); }) {
        hook(pid);
    }
    #[cfg(not(test))]
    let _ = pid;
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
    return run_bounded(command, input, bound, false);
}

/// What:
/// `pub fn output_within_clean_environment(command, input, bound) -> Result<Output, RunError>` is `output_within` for a command
/// whose environment was cleared (`env_clear`) and rebuilt from the variables set on it.
/// Why:
/// A cleared environment cannot be read back from a `Command`,
/// and the scope launcher must still reach the manager with the caller's variables,
/// so the caller says so and the program is started through `env --ignore-environment`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function outputWithinCleanEnvironment(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
/// ```
pub fn output_within_clean_environment(command: &mut Command, input: Option<&[u8]>, bound: &Bound) -> Result<Output, RunError> {
    return run_bounded(command, input, bound, true);
}

/// What:
/// `fn configure_stdio(command: &mut Command, has_input: bool)` pipes the standard streams of a command.
/// Why:
/// Both the caller's command (the weaker path) and the scope launcher need the same pipes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configureStdio(command: Command, hasInput: boolean): void
/// ```
pub(crate) fn configure_stdio(command: &mut Command, has_input: bool) {
    command.stdin(if has_input { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
}

/// What:
/// `fn run_bounded(command, input, bound, clean_environment) -> Result<Output, RunError>` is the run itself.
/// Why:
/// See `output_within`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runBounded(command: Command, input: Buffer | undefined, bound: Bound, cleanEnvironment: boolean): Output  // throws RunError
/// ```
fn run_bounded(command: &mut Command, input: Option<&[u8]>, bound: &Bound, clean_environment: bool) -> Result<Output, RunError> {
    let scope: Option<Rc<ScopeState>> = current_scope();
    let deadline: Instant = Instant::now() + bound.limit;
    configure_stdio(command, input.is_some());
    let containment: Containment = contain();
    let mut child: Child = match start_contained(command, &containment, scope.as_ref(), input.is_some(), clean_environment) {
        Ok(started) => started,
        Err(error) => {
            // Nothing was started (the launcher could not be spawned or the record could not be written): the scope will never be made.
            settle(&containment, &scope);
            return Err(error);
        }
    };
    // A test decides here, after the program has started, whether the completion of the startup fails (certification group 2).
    #[cfg(test)]
    if let Some(hook) = STARTUP_HOOK.with(|slot| { return slot.borrow().clone(); })
        && hook()
    {
        return Err(fail_started(&containment, &mut child, &scope, bound, RunError::Io(io::Error::other("injected failure to complete the startup"))));
    }
    let cancel: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let _cancel_on_exit: CancelOnDrop = CancelOnDrop(Arc::clone(&cancel));
    let writer: Option<Receiver<io::Result<()>>> = match (input, child.stdin.take()) {
        (Some(bytes), Some(stdin)) => Some(feed(OwnedFd::from(stdin), bytes.to_vec(), Arc::clone(&cancel))),
        _ => None,
    };
    let (Some(stdout_handle), Some(stderr_handle)) = (child.stdout.take(), child.stderr.take()) else {
        return Err(fail_started(&containment, &mut child, &scope, bound, RunError::Io(io::Error::other("no standard output or error pipe"))));
    };
    let stdout = collect(OwnedFd::from(stdout_handle), Arc::clone(&cancel));
    let stderr = collect(OwnedFd::from(stderr_handle), Arc::clone(&cancel));
    // A descriptor that becomes readable when the child ends, so the wait wakes at the exit instead of at the next poll.
    let exit_watch: Option<OwnedFd> = Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)).and_then(open_pidfd);
    let status: ExitStatus = loop {
        match child.try_wait() {
            Ok(Some(found)) => break found,
            Ok(None) => {}
            Err(error) => return Err(fail_started(&containment, &mut child, &scope, bound, RunError::Io(error))),
        }
        if Instant::now() >= deadline {
            let timed_out: RunError = RunError::TimedOut { what: bound.what.clone(), limit: bound.limit, rerun: bound.rerun.clone() };
            return Err(fail_started(&containment, &mut child, &scope, bound, timed_out));
        }
        wait_for_exit(exit_watch.as_ref(), deadline.saturating_duration_since(Instant::now()));
    };
    // A normal exit still stops what the child left behind, and the call returns only with the proof that nothing is left.
    if !stop_everything(&containment, &mut child) {
        lose(&scope);
        return Err(RunError::NotQuiescent { what: bound.what.clone() });
    }
    settle(&containment, &scope);
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
    RERUN.set(command);
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
    // Taken out of its cell and put back: nothing else runs in between.
    let held: Option<String> = RERUN.take();
    let text: String = match held.clone() {
        Some(command) => format!("`{command}`"),
        None => "the command you ran".to_string(),
    };
    RERUN.set(held);
    return text;
}
