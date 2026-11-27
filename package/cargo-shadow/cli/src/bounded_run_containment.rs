//! What:
//! How a run is contained and stopped:
//! the choice between a control group scope and a process group,
//! and the stop that shows nothing of the run is left.
//! Why:
//! Split out of `bounded_run.rs`,
//! which runs a child process within a time bound,
//! to keep that file under the code-line limit.
//! The items are re-exported from `bounded_run`,
//! so their paths there are unchanged.
//! The start of the child inside the containment is `bounded_run_start.rs`.

/// What:
/// `use crate::bounded_run::{Bound, POLL_INTERVAL, QUIESCE_BOUND, READER_GRACE, REAP_BOUND, RunError, ScopeState, WEAKER_NOTICE_GIVEN, append_record, find_route};`
/// imports the bounds of a run,
/// its error and journal state,
/// the flag that says the weaker notice was printed,
/// the one record write and the route search.
/// Why:
/// Every wait here is bounded by one of those values,
/// the stop writes the record that settles the run,
/// and the containment is planned on the route the search finds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Bound, POLL_INTERVAL, QUIESCE_BOUND, READER_GRACE, REAP_BOUND, RunError, ScopeState, WEAKER_NOTICE_GIVEN, append_record, find_route } from 'crate/bounded_run';
/// ```
use crate::bounded_run::{
    Bound,
    POLL_INTERVAL,
    QUIESCE_BOUND,
    READER_GRACE,
    REAP_BOUND,
    RunError,
    ScopeState,
    WEAKER_NOTICE_GIVEN,
    append_record,
    find_route,
};
/// What:
/// `use crate::bounded_run::{NOTICE_ENTERED, NOTICE_GATE, trace_text};` imports the flag and the gate that let a test hold the weaker notice,
/// and the one way a test build turns a trace event into text.
/// Why:
/// They exist in a test build only,
/// so this import is compiled for tests alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { NOTICE_ENTERED, NOTICE_GATE, trace_text } from 'crate/bounded_run';
/// ```
#[cfg(test)]
use crate::bounded_run::{
    NOTICE_ENTERED,
    NOTICE_GATE,
    trace_text,
};
/// What:
/// `use crate::control_group::{ControlGroup, GroupUnavailable, Record, Route};` imports the scope itself,
/// the reason a scope cannot be had,
/// the journal record and the route a scope is planned on.
/// Why:
/// The choice of containment holds a scope and its route,
/// the weaker path is chosen with the reason the scope was refused,
/// and the stop records the end of the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ControlGroup, GroupUnavailable, Record, Route } from 'crate/control_group';
/// ```
use crate::control_group::{
    ControlGroup,
    GroupUnavailable,
    Record,
    Route,
};
/// What:
/// `use rustix::event::{PollFd, PollFlags, Timespec, poll};` imports the descriptor watcher and its timeout shape.
/// Why:
/// A pidfd wakes the wait when the child ends,
/// instead of the wait sleeping for its whole interval.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node exposes no poll(2); this is what fs.watch does internally
/// ```
use rustix::event::{
    PollFd,
    PollFlags,
    Timespec,
    poll,
};
/// What:
/// `use rustix::io::Errno;` imports the operating-system error code type.
/// Why:
/// "No member of the group is left" is told from "some member is left" by the code of the signal test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // error.code === 'ESRCH'
/// ```
use rustix::io::Errno;
/// What:
/// `use rustix::process::{Pid, PidfdFlags, Signal, kill_process_group, pidfd_open, test_kill_process_group};` imports the process id type,
/// the flag of a pidfd,
/// the signal to send and the three calls that send it,
/// test it and open the descriptor.
/// Why:
/// The weaker path kills and tests a whole process group,
/// and the wait is woken by that group's leader ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.kill(-pid, 'SIGKILL') and process.kill(-pid, 0)
/// ```
use rustix::process::{
    Pid,
    PidfdFlags,
    Signal,
    kill_process_group,
    pidfd_open,
    test_kill_process_group,
};
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// A worker that could not report its output is an I/O failure of the run,
/// and so is a child that cannot be reaped.
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
/// The pidfd is closed when this value is dropped,
/// so no path of the wait can leak it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node closes a descriptor when its object is collected
/// ```
use std::os::fd::OwnedFd;
/// What:
/// `use std::process::{Child, ExitStatus};` imports the handle of a started child and the status it ends with.
/// Why:
/// Every stop and every wait here acts on the child the start answered,
/// and the reap answers the status it exited with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ChildProcess } from 'node:child_process';   // exitCode is the status
/// ```
use std::process::{
    Child,
    ExitStatus,
};
/// What:
/// `use std::rc::Rc;` imports the reference-counted pointer type (its sibling `Arc` is the one that is safe to share between threads).
/// Why:
/// The journal state is shared inside this one thread only,
/// so the cheaper single-threaded count is enough.
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
/// `use std::sync::atomic::{AtomicBool, Ordering};` imports a boolean that threads may set and read.
/// Why:
/// The cancel flag is set by the drop of the guard and read by every worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let cancelled = false; // single-threaded in Node
/// ```
use std::sync::atomic::{
    AtomicBool,
    Ordering,
};
/// What:
/// `use std::sync::mpsc::{Receiver, RecvTimeoutError};` imports the receiving end of a channel and the error a bounded receive answers.
/// Why:
/// Each output worker answers once through one channel,
/// and a worker that never answers leaves a prefix that must not be returned as the output.
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
/// `use std::thread;` imports the standard library's thread module so this file can name it unqualified.
/// Why:
/// A wait without a pidfd sleeps between polls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await new Promise((resolve) => setTimeout(resolve, ms))
/// ```
use std::thread;
/// What:
/// `use std::time::{Duration, Instant};` imports a span of time and a reading of the clock.
/// Why:
/// Every wait here computes a deadline from the moment it started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now() and a number of milliseconds
/// ```
use std::time::{
    Duration,
    Instant,
};

/// The start of a child inside the containment (split out of this module).
pub(crate) use crate::bounded_run_start::start_contained;
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
pub(crate) fn reap(child: &mut Child) -> Option<ExitStatus> {
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
/// Measured on 2026-11-04 (debug build,
/// 40 runs of `true`):
/// the sleep-polling run took 11.3 ms at best,
/// almost all of it the poll interval.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitForExit(watch: Descriptor | undefined, remainingMs: number): void
/// ```
pub(crate) fn wait_for_exit(
    watch: Option<&OwnedFd>,
    remaining: Duration,
) {
    let Some(descriptor) = watch else {
        thread::sleep(POLL_INTERVAL.min(remaining));
        return;
    };
    let timeout: Timespec = Timespec {
        tv_sec: i64::try_from(remaining.as_secs()).unwrap_or(0),
        tv_nsec: i64::from(remaining.subsec_nanos()),
    };
    let mut watched = [PollFd::new(descriptor, PollFlags::IN)];
    // An interrupted or failed poll only means the next turn of the loop asks `try_wait` again.
    let _ = poll(&mut watched, Some(&timeout));
}

/// How a run is contained.
pub(crate) enum Containment {
    /// In a transient scope the manager made:
    /// quiescence is shown by the kernel.
    Scope {
        /// The scope of this run.
        group: ControlGroup,
        /// The route it was planned on.
        route: Route,
    },
    /// In a process group of its own (the child's process id is the group):
    /// the weaker path.
    ProcessGroup,
}

/// What:
/// `pub(crate) fn weaker_notice(unavailable: &GroupUnavailable) -> String` is the one-line notice of the weaker path.
/// Why:
/// It names the missing capability and says what cannot be shown,
/// so a person knows the guarantee they have (contract,
/// "The weaker path").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function weakerNotice(unavailable: GroupUnavailable): string
/// ```
pub(crate) fn weaker_notice(unavailable: &GroupUnavailable) -> String {
    return format!(
        "cargo-shadow: no delegated control group ({unavailable}); a process that leaves the process group cannot be stopped or found, so quiescence is not shown and a recovery of this run waits for `cargo-shadow acknowledge`"
    );
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
pub(crate) fn contain() -> Containment {
    return match find_route() {
        Ok(route) => match ControlGroup::plan(&route) {
            Ok(group) => Containment::Scope {
                group,
                route,
            },
            Err(error) => weaker(GroupUnavailable {
                reason: format!("a scope name cannot be made ({error})"),
            }),
        },
        Err(unavailable) => weaker(unavailable),
    };
}

/// What:
/// `fn weaker(unavailable: GroupUnavailable) -> Containment` gives the notice that the run is contained by a process group only,
/// and returns that containment.
/// Why:
/// A function replaces the closure that `contain` called at both refusals.
pub(crate) fn weaker(unavailable: GroupUnavailable) -> Containment {
    // The notice this run is given is part of the trace, so a test sees the notice of the run and not a text it formats itself.
    // The text is computed, so this statement exists in a test build only.
    #[cfg(test)]
    trace_text(format!("notice {}", weaker_notice(&unavailable)));
    if !WEAKER_NOTICE_GIVEN.swap(true, std::sync::atomic::Ordering::SeqCst) {
        #[cfg(test)]
        {
            NOTICE_ENTERED.store(true, std::sync::atomic::Ordering::SeqCst);
            drop(NOTICE_GATE.lock());
        }
        eprintln!("{}", weaker_notice(&unavailable));
    }
    return Containment::ProcessGroup;
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
pub(crate) fn stop_process_group(
    child: &mut Child,
    group: Pid,
) -> bool {
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
/// on timeout,
/// on a normal exit and on every error path.
/// The direct child goes first:
/// it is the process that becomes the program,
/// so once it is reaped nothing of this program's can
/// still start the program in a scope that is not there yet.
/// A scope that cannot be stopped,
/// and any failure to inspect or stop it,
/// is "not shown gone".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopEverything(containment: Containment, child: ChildProcess): boolean
/// ```
pub(crate) fn stop_everything(
    containment: &Containment,
    child: &mut Child,
) -> bool {
    return match containment {
        Containment::Scope {
            group,
            ..
        } => {
            let _ = child.kill();
            let reaped: bool = reap(child).is_some();
            let stopped: bool = match group.stop(QUIESCE_BOUND) {
                Ok(done) => done,
                Err(error) => {
                    eprintln!(
                        "cargo-shadow: warning: the scope {} could not be shown empty ({error})",
                        group.directory().display()
                    );
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
/// stopped,
/// so a recovery may read an absent scope as one that ended.
/// Why:
/// Until then an absent scope may be one the launcher has not made yet (round 9,
/// 8a F1).
/// A record that cannot be written leaves the journal unsettled,
/// which the next recovery treats as uncertainty to acknowledge:
/// the safe side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settle(containment: Containment, scope: ScopeState | undefined): void
/// ```
pub(crate) fn settle(
    containment: &Containment,
    scope: &Option<Rc<ScopeState>>,
) {
    if let (
        Containment::Scope {
            group,
            ..
        },
        Some(state),
    ) = (containment, scope)
        && let Err(error) = append_record(
            &state.log,
            &Record::Settled {
                path: group.directory().to_path_buf(),
            },
        )
    {
        eprintln!(
            "cargo-shadow: warning: the end of the run in {} could not be recorded ({error}); a recovery will ask for an acknowledgement",
            group.directory().display()
        );
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
pub(crate) fn fail_started(
    containment: &Containment,
    child: &mut Child,
    scope: &Option<Rc<ScopeState>>,
    bound: &Bound,
    error: RunError,
) -> RunError {
    if stop_everything(containment, child) {
        settle(containment, scope);
        return error;
    }
    lose(scope);
    return RunError::NotQuiescent {
        what: bound.what.clone(),
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
pub(crate) fn finished(
    receiver: &Receiver<io::Result<Vec<u8>>>,
    what: &str,
) -> Result<Vec<u8>, RunError> {
    return match receiver.recv_timeout(READER_GRACE) {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(error)) => Err(RunError::Io(error)),
        Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
            Err(RunError::IncompleteOutput {
                what: what.to_string(),
            })
        }
    };
}

/// Sets the cancel flag when the run ends,
/// however it ends,
/// so no worker outlives the call.
pub(crate) struct CancelOnDrop(
    /// The flag the pipe workers look at.
    pub(crate) Arc<AtomicBool>,
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
pub(crate) fn lose(scope: &Option<Rc<ScopeState>>) {
    if let Some(state) = scope {
        state.lost.set(true);
    }
}

/// What:
/// `fn reason_of(unavailable: &GroupUnavailable) -> String` clones the reason of a refusal.
/// Why:
/// `Result::map_err` takes it by path instead of a closure.
pub(crate) fn reason_of(unavailable: &GroupUnavailable) -> String {
    return unavailable.reason.clone();
}

/// What:
/// `fn open_pidfd(pid: Pid) -> Option<OwnedFd>` opens a descriptor that becomes readable when the process ends,
/// or `None` when the kernel refuses.
/// Why:
/// `Option::and_then` takes it by path instead of a closure.
pub(crate) fn open_pidfd(pid: Pid) -> Option<OwnedFd> {
    return pidfd_open(pid, PidfdFlags::empty()).ok();
}
