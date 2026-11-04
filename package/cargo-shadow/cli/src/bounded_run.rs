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
//! the direct child is killed and reaped, the scope is killed,
//! and the run waits for the kernel's own statement that no thread is left (`populated 0`) or for the verified absence of the
//! scope.
//! A call that returns `Ok` has that statement,
//! and every failure after the program started that cannot show it is `NotQuiescent`.
//! Where no scope can be had (no user manager, no cgroup v2, an older kernel),
//! the child runs in a process group of its own,
//! a one-line notice names the missing capability,
//! the journal records `pgroup starting` before the start and the group after,
//! and no quiescence is claimed for what leaves the process group: a recovery of such a journal waits for an acknowledgement.
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

/// Imports `std::sync::{Arc, Once}` for this file.
use std::sync::{Arc, Once};

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
use crate::control_group::{ControlGroup, GroupUnavailable, Record, Route, RouteSettings, detect_route};

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
pub(crate) const QUIESCE_BOUND: Duration = Duration::from_secs(5);

/// How long the output workers are given to report once every writer is gone.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open:
/// with every writer gone the pipes are at their end at once,
/// so this only needs to cover the scheduling of the worker threads;
/// the largest output (`cargo metadata` of this repository) drains in well under it.
const READER_GRACE: Duration = Duration::from_secs(2);

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
    /// The settings the route is discovered with instead of the ordinary ones,
    /// so a test can make the host look like one that lacks a capability.
    static ROUTE_OVERRIDE: RefCell<Option<RouteSettings>> = const { RefCell::new(None) };

    /// A hook that runs inside the start of a run, after the scope's record is on disk and before the program starts,
    /// so a test can overlap other work with the start at a boundary of its own choosing.
    static START_HOOK: RefCell<Option<Rc<dyn Fn()>>> = const { RefCell::new(None) };

    /// A hook that runs right after the program has started and decides whether the completion of the startup fails: it returns
    /// `true` to make it fail, after the test has waited for whatever it needs to be ready (a writer, for one).
    static STARTUP_HOOK: RefCell<Option<Rc<dyn Fn() -> bool>>> = const { RefCell::new(None) };

    /// The ordered events of the recording and the launching of the runs of this thread, for the tests of their order.
    static TRACE: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// What:
/// `fn trace(event: impl FnOnce() -> String)` appends an event to this thread's trace in a test build and does nothing otherwise.
/// Why:
/// The order record write, durability barrier, launch is a property of the real boundaries, so the tests read an ordered trace of
/// them (certification group 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trace(event: () => string): void
/// ```
#[cfg(test)]
fn trace(event: impl FnOnce() -> String) {
    let text: String = event();
    TRACE.with(|slot| { slot.borrow_mut().push(text); });
}

/// The production build keeps no trace.
#[cfg(not(test))]
fn trace(_event: impl FnOnce() -> String) {}

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
    return append_record_with(log, record, true);
}

/// What:
/// `fn append_record_with(log: &Path, record: &Record, sync: bool) -> io::Result<()>` appends one line to the journal's `groups` file
/// and forces it to disk when `sync` is set.
/// Why:
/// The launcher record is written without a sync (round 11): it is read only by a recovery after the wrapper was killed (the page
/// cache keeps it), or after a power loss, where a lost or torn line is read as no proof and the boot rule decides.
/// The line is one `write`, so a process kill leaves it whole or absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function appendRecordWith(log: string, record: Record, sync: boolean): void
/// ```
fn append_record_with(log: &Path, record: &Record, sync: bool) -> io::Result<()> {
    let file: OwnedFd = open(log, OFlags::WRONLY | OFlags::APPEND | OFlags::CLOEXEC | OFlags::NONBLOCK | OFlags::NOFOLLOW, Mode::empty()).map_err(io::Error::from)?;
    let kind: FileType = FileType::from_raw_mode(fstat(&file).map_err(io::Error::from)?.st_mode);
    if kind != FileType::RegularFile {
        return Err(io::Error::from(Errno::INVAL));
    }
    let line: String = format!("{record}\n");
    let kind_name: &str = match record {
        Record::Scope { .. } => "scope",
        Record::Settled { .. } => "settled",
        Record::Launcher { .. } => "launcher",
        Record::ProcessGroup(_) => "pgroup",
        Record::ProcessGroupStarting => "pgroup-starting",
    };
    trace(|| format!("attempt {kind_name}"));
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::RecordWrite) {
        return Err(io::Error::other("injected failure to write the containment record"));
    }
    let written: usize = rustix::io::write(&file, line.as_bytes()).map_err(io::Error::from)?;
    if written != line.len() {
        return Err(io::Error::new(io::ErrorKind::WriteZero, "the containment record was cut short"));
    }
    trace(|| format!("written {kind_name}"));
    if !sync {
        return Ok(());
    }
    // The record is durable before the program it describes can start: a sync that fails stops the start.
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::RecordSync) {
        trace(|| format!("sync {kind_name} whole={}", std::fs::read_to_string(log).is_ok_and(|text| { return text.ends_with(&line); })));
        return Err(io::Error::other("injected failure to make the containment record durable"));
    }
    trace(|| format!("sync {kind_name} whole={}", std::fs::read_to_string(log).is_ok_and(|text| { return text.ends_with(&line); })));
    fsync(&file).map_err(io::Error::from)?;
    trace(|| format!("synced {kind_name}"));
    return Ok(());
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
    // A child that cannot be reaped within its bound is "not reaped"; a test makes that so without a process in an uninterruptible wait.
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::ReapStuck) {
        return None;
    }
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
    /// In a transient scope the manager made: quiescence is shown by the kernel.
    Scope {
        /// The scope of this run.
        group: ControlGroup,
        /// The route it was planned on.
        route: Route,
    },
    /// In a process group of its own (the child's process id is the group): the weaker path.
    ProcessGroup,
}

/// The answer of the manager, kept for the life of the process.
/// It is read and written only with `try_lock`: a thread that finds the cache busy asks the manager itself instead of waiting for
/// the thread that holds it (contract V6: an internal lock returns at once).
pub(crate) struct RouteCache {
    /// The last answer, a route or the reason there is none.
    slot: std::sync::Mutex<Option<Result<Route, String>>>,
}

/// The methods of the type above.
impl RouteCache {
    /// What:
    /// `pub(crate) const fn new() -> RouteCache` makes an empty cache.
    /// Why:
    /// The production cache is a `static`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static new(): RouteCache
    /// ```
    pub(crate) const fn new() -> RouteCache {
        return RouteCache { slot: std::sync::Mutex::new(None) };
    }

    /// What:
    /// `pub(crate) fn get_or_discover(&self, discover: &dyn Fn() -> Result<Route, GroupUnavailable>) -> Result<Route, GroupUnavailable>`
    /// returns the cached answer, or asks `discover` and keeps its answer.
    /// Why:
    /// A busy cache is never waited for: the caller discovers on its own and keeps the answer only if the cache is free by then.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// getOrDiscover(discover: () => Route): Route  // throws GroupUnavailable
    /// ```
    pub(crate) fn get_or_discover(&self, discover: &dyn Fn() -> Result<Route, GroupUnavailable>) -> Result<Route, GroupUnavailable> {
        if let Ok(held) = self.slot.try_lock()
            && let Some(answer) = held.as_ref()
        {
            return answer.clone().map_err(|reason| { return GroupUnavailable { reason }; });
        }
        let found: Result<Route, GroupUnavailable> = discover();
        if let Ok(mut held) = self.slot.try_lock() {
            *held = Some(found.as_ref().map(Clone::clone).map_err(|unavailable| { return unavailable.reason.clone(); }));
        }
        return found;
    }
}

/// What:
/// `fn find_route() -> Result<Route, GroupUnavailable>` finds out what this host can give.
/// Why:
/// The answer of the manager does not change while the process lives, so it is asked once;
/// a test asks again, with its own settings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findRoute(): Route  // throws GroupUnavailable
/// ```
#[cfg(not(test))]
fn find_route() -> Result<Route, GroupUnavailable> {
    static ROUTE: RouteCache = RouteCache::new();
    return ROUTE.get_or_discover(&|| { return detect_route(&RouteSettings::default()); });
}

/// What:
/// `fn find_route() -> Result<Route, GroupUnavailable>` is the test build's version:
/// it asks again every time, with the settings of the test.
/// Why:
/// A test makes the host look like one that lacks a capability.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findRoute(): Route  // throws GroupUnavailable
/// ```
#[cfg(test)]
fn find_route() -> Result<Route, GroupUnavailable> {
    let settings: RouteSettings = ROUTE_OVERRIDE.with(|slot| { return slot.borrow().clone(); }).unwrap_or_default();
    return detect_route(&settings);
}

/// What:
/// `pub fn use_route_settings_for_test(settings: Option<RouteSettings>)` makes the runs of this thread discover their route with
/// `settings` (`None` restores the ordinary ones).
/// Why:
/// A host that lacks `systemd-run`, a reachable manager or a cgroup v2 mount is made by pointing the settings at nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function useRouteSettingsForTest(settings?: RouteSettings): void
/// ```
#[cfg(test)]
pub(crate) fn use_route_settings_for_test(settings: Option<RouteSettings>) {
    ROUTE_OVERRIDE.with(|slot| { *slot.borrow_mut() = settings; });
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
/// `pub(crate) fn weaker_notice(unavailable: &GroupUnavailable) -> String` is the one-line notice of the weaker path.
/// Why:
/// It names the missing capability and says what cannot be shown,
/// so a person knows the guarantee they have (contract, "The weaker path").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function weakerNotice(unavailable: GroupUnavailable): string
/// ```
pub(crate) fn weaker_notice(unavailable: &GroupUnavailable) -> String {
    return format!("cargo-shadow: no delegated control group ({unavailable}); a process that leaves the process group cannot be stopped or found, so quiescence is not shown and a recovery of this run waits for `cargo-shadow acknowledge`");
}

/// What:
/// `fn contain() -> Containment` plans a scope,
/// or says in one line which capability is missing and falls back to a process group.
/// Why:
/// The notice is printed once per process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function contain(): Containment
/// ```
fn contain() -> Containment {
    let weaker = |unavailable: GroupUnavailable| -> Containment {
        WEAKER_NOTICE.call_once(|| { eprintln!("{}", weaker_notice(&unavailable)); });
        return Containment::ProcessGroup;
    };
    return match find_route() {
        Ok(route) => match ControlGroup::plan(&route) {
            Ok(group) => Containment::Scope { group, route },
            Err(error) => weaker(GroupUnavailable { reason: format!("a scope name cannot be made ({error})") }),
        },
        Err(unavailable) => weaker(unavailable),
    };
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
/// it kills and reaps the direct child,
/// stops what the containment holds,
/// and returns `true` only if all of it is shown gone.
/// Why:
/// This is the one way a run is ended,
/// on timeout, on a normal exit and on every error path.
/// The direct child goes first: it is the process that becomes the program, so once it is reaped nothing of this program's can
/// still start the program in a scope that is not there yet.
/// A scope that cannot be stopped, and any failure to inspect or stop it, is "not shown gone".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopEverything(containment: Containment, child: ChildProcess): boolean
/// ```
fn stop_everything(containment: &Containment, child: &mut Child) -> bool {
    return match containment {
        Containment::Scope { group, .. } => {
            let _ = child.kill();
            let reaped: bool = reap(child).is_some();
            let stopped: bool = match group.stop(QUIESCE_BOUND) {
                Ok(done) => done,
                Err(error) => {
                    eprintln!("cargo-shadow: warning: the scope {} could not be shown empty ({error})", group.directory().display());
                    false
                }
            };
            reaped && stopped
        }
        Containment::ProcessGroup => match Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)) {
            Some(group) => stop_process_group(child, group),
            None => false,
        },
    };
}

/// What:
/// `fn settle(containment: &Containment, scope: &Option<Rc<ScopeState>>)` records that the run's launcher is reaped and its scope
/// stopped, so a recovery may read an absent scope as one that ended.
/// Why:
/// Until then an absent scope may be one the launcher has not made yet (round 9, 8a F1).
/// A record that cannot be written leaves the journal unsettled, which the next recovery treats as uncertainty to acknowledge:
/// the safe side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settle(containment: Containment, scope: ScopeState | undefined): void
/// ```
fn settle(containment: &Containment, scope: &Option<Rc<ScopeState>>) {
    if let (Containment::Scope { group, .. }, Some(state)) = (containment, scope)
        && let Err(error) = append_record(&state.log, &Record::Settled { path: group.directory().to_path_buf() })
    {
        eprintln!("cargo-shadow: warning: the end of the run in {} could not be recorded ({error}); a recovery will ask for an acknowledgement", group.directory().display());
    }
}

/// What:
/// `fn fail_started(containment, child, scope, bound, error) -> RunError` ends a run that failed after its program started.
/// Why:
/// Any failure on the containment path after the start is "not quiescent" unless everything is shown gone (contract A2):
/// it must not become an ordinary failure that lets the caller roll back or accept under a writer that may remain (7a F3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failStarted(containment: Containment, child: ChildProcess, scope: ScopeState | undefined, bound: Bound, error: RunError): RunError
/// ```
fn fail_started(containment: &Containment, child: &mut Child, scope: &Option<Rc<ScopeState>>, bound: &Bound, error: RunError) -> RunError {
    if stop_everything(containment, child) {
        settle(containment, scope);
        return error;
    }
    lose(scope);
    return RunError::NotQuiescent { what: bound.what.clone() };
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
fn configure_stdio(command: &mut Command, has_input: bool) {
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
    let exit_watch: Option<OwnedFd> = Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)).and_then(|pid| { return pidfd_open(pid, PidfdFlags::empty()).ok(); });
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
/// `fn program_resolves(command: &Command) -> bool` says whether the program of the command exists as an executable file,
/// searched on the `PATH` the command will have when its name has no slash.
/// Why:
/// `systemd-run` reports a program it cannot start as an exit status like any other,
/// so a program that does not exist is found before the launcher is started and reported as the I/O error it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function programResolves(command: Command): boolean
/// ```
fn program_resolves(command: &Command) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let executable = |candidate: &Path| -> bool { return std::fs::metadata(candidate).map(|found| { return found.is_file() && found.permissions().mode() & 0o111 != 0; }).unwrap_or(false); };
    let program: &std::ffi::OsStr = command.get_program();
    let relative_to: PathBuf = command.get_current_dir().map(Path::to_path_buf).unwrap_or_default();
    if program.as_encoded_bytes().contains(&b'/') {
        return executable(&relative_to.join(program));
    }
    let own_path: Option<std::ffi::OsString> = command.get_envs().find(|(name, _)| { return *name == "PATH"; }).and_then(|(_, value)| { return value.map(std::ffi::OsStr::to_os_string); }).or_else(|| { return std::env::var_os("PATH"); });
    return own_path.is_some_and(|search| { return std::env::split_paths(&search).any(|directory| { return executable(&relative_to.join(directory).join(program)); }); });
}

/// What:
/// `fn run_start_hook()` runs the hook a test put inside the start of a run, if any.
/// Why:
/// It is the boundary at which a test overlaps other work with a start, or kills a wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runStartHook(): void
/// ```
#[cfg(test)]
fn run_start_hook() {
    if let Some(hook) = START_HOOK.with(|slot| { return slot.borrow().clone(); }) {
        hook();
    }
}

/// The production build has no hook.
#[cfg(not(test))]
fn run_start_hook() {}

/// What:
/// `fn note_launcher(log: &Path, scope: &Path, child: &Child)` appends the launcher record: the process id, the start time and the
/// process id namespace of the process that is about to become the program.
/// Why:
/// A recovery that finds this process gone knows it can never join the scope, so it can read the scope as final without an
/// acknowledgement (round 11).
/// A line that cannot be written is only a missing proof: the journal then waits for an acknowledgement like an older journal, so the
/// run goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteLauncher(log: string, scope: string, child: ChildProcess): void
/// ```
fn note_launcher(log: &Path, scope: &Path, child: &Child) {
    let pid: i32 = i32::try_from(child.id()).unwrap_or(0);
    let noted: io::Result<()> = crate::control_group::process_start(pid).and_then(|start| {
        let pid_namespace: u64 = crate::control_group::pid_namespace_id()?;
        return append_record_with(log, &Record::Launcher { pid, start, pid_namespace, path: scope.to_path_buf() }, false);
    });
    if let Err(error) = noted {
        eprintln!("cargo-shadow: warning: the launcher of {} could not be recorded ({error}); if this wrapper is killed, a recovery will ask for an acknowledgement", scope.display());
    }
}

/// What:
/// `fn start_contained(command, containment, scope, has_input, clean_environment) -> Result<Child, RunError>` records the
/// containment in the journal and starts the child inside it.
/// Why:
/// For a scope the record is forced to disk before the program starts,
/// and a record that cannot be written means the program is not started.
/// For the weaker path the process group is only known once the child exists,
/// so a `pgroup starting` line is forced to disk first and the group's line follows the start;
/// a child whose group cannot be recorded is stopped at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startContained(command: Command, containment: Containment, scope: ScopeState | undefined, hasInput: boolean, cleanEnvironment: boolean): ChildProcess  // throws RunError
/// ```
fn start_contained(command: &mut Command, containment: &Containment, scope: Option<&Rc<ScopeState>>, has_input: bool, clean_environment: bool) -> Result<Child, RunError> {
    match containment {
        Containment::Scope { group, route } => {
            if !program_resolves(command) {
                return Err(RunError::Io(io::Error::new(io::ErrorKind::NotFound, format!("{:?} was not found (os error 2)", command.get_program()))));
            }
            if let Some(state) = scope {
                append_record(&state.log, &group.record()).map_err(|error| {
                    return RunError::Io(io::Error::new(error.kind(), format!("{error} (a program whose scope cannot be recorded is not started)")));
                })?;
            }
            run_start_hook();
            let mut launch: Command = group.launcher(route, command, clean_environment);
            configure_stdio(&mut launch, has_input);
            trace(|| { return "launch scope".to_string(); });
            let child: Child = launch.spawn().map_err(RunError::Io)?;
            if let Some(state) = scope {
                note_launcher(&state.log, group.directory(), &child);
            }
            return Ok(child);
        }
        Containment::ProcessGroup => {
            if let Some(state) = scope {
                state.process_group_only.set(true);
            }
            command.process_group(0);
            // The child's group is its process id, known only after the spawn: a "starting" line is durable before it, so a
            // wrapper killed in between is seen by a recovery as one that may have a child running.
            if let Some(state) = scope {
                append_record(&state.log, &Record::ProcessGroupStarting).map_err(|error| {
                    return RunError::Io(io::Error::new(error.kind(), format!("{error} (a child whose process group cannot be announced is not started)")));
                })?;
            }
            run_start_hook();
            trace(|| { return "launch pgroup".to_string(); });
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
