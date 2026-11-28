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

/// What:
/// `use crate::bounded_io::{collect, feed};` imports the two pipe workers:
/// one drains a descriptor into a list,
/// one writes a list into a descriptor.
/// Why:
/// Both run on their own threads and both stop when the cancel flag is set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { collect, feed } from 'crate/bounded_io';
/// ```
use crate::bounded_io::{
    collect,
    feed,
};
/// What:
/// `use rustix::process::Pid;` imports the operating system's process id type.
/// Why:
/// The descriptor that wakes the wait when the child ends is opened from the child's process id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number
/// ```
use rustix::process::Pid;
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// The failure of a pipe,
/// of a start and of a wait are all reported as the I/O variant of the run's error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an I/O failure is an ordinary thrown Error in TS
/// ```
use std::io;
/// What:
/// `use std::os::fd::OwnedFd;` imports the owning descriptor type (its siblings are `BorrowedFd`,
/// a view,
/// and `RawFd`,
/// the bare number).
/// Why:
/// The three pipes of the child are handed to the readers that own them until the run ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node closes a stream descriptor when the stream is closed
/// ```
use std::os::fd::OwnedFd;
/// What:
/// `use std::process::{Child, Command, ExitStatus, Output, Stdio};` imports the child handle,
/// the command builder,
/// the status it exits with,
/// the collected output and the choice of a standard stream.
/// Why:
/// The whole file is one call that starts a command and answers its output,
/// and every one of those five is named on the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ChildProcess, spawn } from 'node:child_process';
/// ```
use std::process::{
    Child,
    Command,
    ExitStatus,
    Output,
    Stdio,
};
/// What:
/// `use std::rc::Rc;` imports the reference-counted pointer type (its sibling `Arc` is the one that is safe to share between threads).
/// Why:
/// The state of the open scope is shared with the containment inside one thread only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // every object reference in TS is shared and counted by the garbage collector
/// ```
use std::rc::Rc;
/// What:
/// `use std::sync::Arc;` imports the atomically counted pointer type (its sibling `Rc` counts without atomics,
/// for one thread only).
/// Why:
/// The cancel flag is read by the pipe workers on other threads,
/// so the count must be atomic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // shared object references in TS need no count
/// ```
use std::sync::Arc;
/// What:
/// `use std::sync::atomic::AtomicBool;` imports a boolean that threads may set and read.
/// Why:
/// The cancel flag stops the pipe workers when the run ends,
/// however it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let cancelled = false; // single-threaded in Node
/// ```
use std::sync::atomic::AtomicBool;
/// What:
/// `use std::sync::mpsc::{Receiver, RecvTimeoutError};` imports the receiving end of a channel and the error a bounded receive answers.
/// Why:
/// Each output worker answers once through one channel,
/// and the input writer answers whether it took everything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a Promise with Promise.race against a timer
/// ```
use std::sync::mpsc::{
    Receiver,
    RecvTimeoutError,
};
/// What:
/// `use std::time::{Duration, Instant};` imports a span of time and a reading of the monotonic clock (sibling of `Instant`:
/// `SystemTime`,
/// the wall clock,
/// which can jump).
/// Why:
/// The deadline of one call is computed from the moment it starts,
/// on a clock that never jumps back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now() and a number of milliseconds
/// ```
use std::time::{
    Duration,
    Instant,
};

/// The items that only the tests of the bounded run use.
#[cfg(test)]
pub(crate) use crate::bounded_run_containment::weaker_notice;
/// The containment of a run:
/// its choice,
/// start and stop (split out of the bounded run module).
pub(crate) use crate::bounded_run_containment::{
    CancelOnDrop,
    Containment,
    contain,
    finished,
    lose,
    open_pidfd,
    settle,
    start_contained,
    wait_for_exit,
};
/// The end of a run:
/// its containment stopped and its scope unit retired (split out of the containment module).
pub(crate) use crate::bounded_run_end::{
    fail_started,
    stop_everything,
};

/// The write of one containment record (split out of the bounded run module).
pub(crate) use crate::bounded_run_record_writes::{
    append_record,
    append_record_with,
};

/// The cache of the control group route (split out of the bounded run module).
#[cfg(not(test))]
pub(crate) use crate::bounded_run_route_cache::find_route;
/// The test-only items of the same module.
#[cfg(test)]
pub(crate) use crate::bounded_run_route_cache::{
    RouteCache,
    find_route,
    use_route_settings_for_test,
};

/// The bound one call must finish within and the error that says which it did not meet.
mod bound_and_error;
/// The open scope of one run as this process remembers it.
mod scope;

/// What one bounded call is allowed and how it fails (split out of this module).
pub use self::bound_and_error::{
    Bound,
    ROUTE_REFUSED_OPENING,
    RunError,
};
/// The mark that a run's writers may remain,
/// set by a test (split out of this module).
#[cfg(test)]
pub use self::scope::mark_scope_lost_for_test;
/// The open scope of one run and the command a timed-out run repeats (split out of this module).
pub use self::scope::{
    RunScope,
    rerun_text,
    set_rerun,
};
/// The innermost open scope of this thread and the state it remembers (split out of this module).
pub(crate) use self::scope::{
    ScopeState,
    current_scope,
};
/// The hook a test puts right after the launcher was spawned (split out of this module).
#[cfg(test)]
use crate::bounded_run_trace_and_hooks::AFTER_LAUNCH_HOOK;
/// The route a test makes the host look like one that lacks a capability with (split out of this module).
#[cfg(test)]
pub(crate) use crate::bounded_run_trace_and_hooks::ROUTE_OVERRIDE;
/// The hook a test puts inside the start of a run (split out of this module).
#[cfg(test)]
pub(crate) use crate::bounded_run_trace_and_hooks::START_HOOK;
/// The hook a test puts after the program started,
/// and the trace it reads (split out of this module).
#[cfg(test)]
use crate::bounded_run_trace_and_hooks::STARTUP_HOOK;
/// The notice gate,
/// the trace reader and the hooks a test drives (split out of this module).
#[cfg(test)]
pub(crate) use crate::bounded_run_trace_and_hooks::{
    NOTICE_ENTERED,
    NOTICE_GATE,
    after_launch_for_test,
    on_start_for_test,
    on_startup_for_test,
    take_trace_for_test,
    trace_text,
};
/// The trace of a run and the weaker path's notice gate (split out of this module).
pub(crate) use crate::bounded_run_trace_and_hooks::{
    WEAKER_NOTICE_GIVEN,
    trace,
};
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
    if let Some(hook) = AFTER_LAUNCH_HOOK.with(|slot| {
        return slot.borrow().clone();
    }) {
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
pub fn output_within(
    command: &mut Command,
    input: Option<&[u8]>,
    bound: &Bound,
) -> Result<Output, RunError> {
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
pub fn output_within_clean_environment(
    command: &mut Command,
    input: Option<&[u8]>,
    bound: &Bound,
) -> Result<Output, RunError> {
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
pub(crate) fn configure_stdio(
    command: &mut Command,
    has_input: bool,
) {
    command
        .stdin(if has_input {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
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
fn run_bounded(
    command: &mut Command,
    input: Option<&[u8]>,
    bound: &Bound,
    clean_environment: bool,
) -> Result<Output, RunError> {
    let scope: Option<Rc<ScopeState>> = current_scope();
    let deadline: Instant = Instant::now() + bound.limit;
    configure_stdio(command, input.is_some());
    // What:     `contain` answers the scope the run is given, and `?` gives the caller the refusal instead, which is the route
    //           no wait of this tool ends.
    // Why:      A refused route stops the run before anything is started, so no program writes a file behind another unit's stop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const containment: Containment = contain()  // throws RunError
    // ```
    let containment: Containment = contain()?;
    let mut child: Child = match start_contained(
        command,
        &containment,
        scope.as_ref(),
        input.is_some(),
        clean_environment,
    ) {
        Ok(started) => started,
        Err(error) => {
            // Nothing was started (the launcher could not be spawned or the record could not be written): the scope will never be made.
            settle(&containment, &scope);
            return Err(error);
        }
    };
    // A test decides here, after the program has started, whether the completion of the startup fails (certification group 2).
    #[cfg(test)]
    if let Some(hook) = STARTUP_HOOK.with(|slot| {
        return slot.borrow().clone();
    }) && hook()
    {
        return Err(fail_started(
            &containment,
            &mut child,
            &scope,
            bound,
            RunError::Io(io::Error::other("injected failure to complete the startup")),
        ));
    }
    let cancel: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let _cancel_on_exit: CancelOnDrop = CancelOnDrop(Arc::clone(&cancel));
    let writer: Option<Receiver<io::Result<()>>> = match (input, child.stdin.take()) {
        (Some(bytes), Some(stdin)) => Some(feed(
            OwnedFd::from(stdin),
            bytes.to_vec(),
            Arc::clone(&cancel),
        )),
        _ => None,
    };
    let (Some(stdout_handle), Some(stderr_handle)) = (child.stdout.take(), child.stderr.take())
    else {
        return Err(fail_started(
            &containment,
            &mut child,
            &scope,
            bound,
            RunError::Io(io::Error::other("no standard output or error pipe")),
        ));
    };
    let stdout = collect(OwnedFd::from(stdout_handle), Arc::clone(&cancel));
    let stderr = collect(OwnedFd::from(stderr_handle), Arc::clone(&cancel));
    // A descriptor that becomes readable when the child ends, so the wait wakes at the exit instead of at the next poll.
    let exit_watch: Option<OwnedFd> =
        Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)).and_then(open_pidfd);
    let status: ExitStatus = loop {
        match child.try_wait() {
            Ok(Some(found)) => break found,
            Ok(None) => {}
            Err(error) => {
                return Err(fail_started(
                    &containment,
                    &mut child,
                    &scope,
                    bound,
                    RunError::Io(error),
                ));
            }
        }
        if Instant::now() >= deadline {
            let timed_out: RunError = RunError::TimedOut {
                what: bound.what.clone(),
                limit: bound.limit,
                rerun: bound.rerun.clone(),
            };
            return Err(fail_started(
                &containment,
                &mut child,
                &scope,
                bound,
                timed_out,
            ));
        }
        wait_for_exit(
            exit_watch.as_ref(),
            deadline.saturating_duration_since(Instant::now()),
        );
    };
    // A normal exit still stops what the child left behind, and the call returns only with the proof that nothing is left.
    if let Err(shortfall) = stop_everything(&containment, &mut child) {
        lose(&scope, shortfall.clone());
        return Err(RunError::NotQuiescent {
            what: bound.what.clone(),
            shortfall,
        });
    }
    settle(&containment, &scope);
    let out: Vec<u8> = finished(&stdout, &bound.what)?;
    let err: Vec<u8> = finished(&stderr, &bound.what)?;
    if let Some(receiver) = writer {
        match receiver.recv_timeout(READER_GRACE) {
            Ok(Ok(())) => {}
            Ok(Err(error)) if error.kind() == io::ErrorKind::BrokenPipe => {}
            Ok(Err(error)) => return Err(RunError::Io(error)),
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
                return Err(RunError::IncompleteInput {
                    what: bound.what.clone(),
                });
            }
        }
    }
    return Ok(Output {
        status,
        stdout: out,
        stderr: err,
    });
}

/// The tests of the bounded child runs.
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "bounded_run_tests.rs"]
mod bounded_run_tests;
