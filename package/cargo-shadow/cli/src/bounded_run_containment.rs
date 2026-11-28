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
//! The start of the child inside the containment is `bounded_run_start.rs`,
//! and the end of the run,
//! with the stop of its scope unit after it,
//! is `bounded_run_end.rs`.

/// What:
/// `mod process_group;` declares the stop of a bare process group and its refusal's words.
/// Why:
/// The weaker path is a subject of its own beside the scope,
/// and the file that held both passed the code-line limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the stop of a process group, beside the stop of a scope
/// ```
#[path = "bounded_run_containment_process_group.rs"]
pub(crate) mod process_group;
/// What:
/// `use self::process_group::{GroupStop, stop_process_group};` re-exports what the weaker path's stop found and that stop.
/// Why:
/// The end of a run and the start that failed to record a group both call it,
/// and their paths stay `bounded_run_containment::stop_process_group` and `bounded_run_containment::GroupStop`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { GroupStop, stopProcessGroup } from './bounded_run_containment_process_group.ts';
/// ```
pub(crate) use self::process_group::{
    GroupStop,
    stop_process_group,
};

/// What:
/// `mod reap_of_the_child;` declares the wait for the direct child and what became of it.
/// Why:
/// The reap and the stop are separate subjects,
/// and a file that held both would pass the code-line limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the reap of the direct child, beside the stop that calls it
/// ```
#[path = "bounded_run_containment_reap_of_the_child.rs"]
pub(crate) mod reap_of_the_child;
/// What:
/// `use self::reap_of_the_child::{ReapShortfall, Reaped, reap};` re-exports the wait and its two answers.
/// Why:
/// The stop of a scope and of a process group both read them,
/// and their paths stay `bounded_run_containment::reap` and `bounded_run_containment::Reaped`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { ReapShortfall, Reaped, reap } from './bounded_run_containment_reap_of_the_child.ts';
/// ```
pub(crate) use self::reap_of_the_child::{
    ReapShortfall,
    Reaped,
    reap,
};

/// What:
/// `mod refusal_tests;` declares the controls of what a refusal says when a stop could not settle the run.
/// Why:
/// A unit control belongs in a sidecar file,
/// and declaring it here keeps the crate root's list of modules out of the change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a sidecar test file next to the module it tests
/// ```
#[cfg(test)]
#[path = "bounded_run_containment_refusal_tests.rs"]
mod refusal_tests;

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
/// `use crate::control_group_stop::report::StopReport;` imports what the stop of a scope answers.
/// Why:
/// The shortfall of a run that cannot be shown stopped is built from that report,
/// so the same facts reach the refusal and the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { StopReport } from 'crate/control_group_stop/report';
/// ```
use crate::control_group_stop::report::StopReport;
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
/// `use rustix::process::{Pid, PidfdFlags, pidfd_open};` imports the process id type,
/// the flag of a pidfd and the call that opens one.
/// Why:
/// The wait is woken by the child's pidfd becoming readable;
/// the process group's own signal and test live in `bounded_run_containment_process_group.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // fs.watch on the child's pidfd
/// ```
use rustix::process::{
    Pid,
    PidfdFlags,
    pidfd_open,
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
/// `use std::process::Child;` imports the handle of a started child.
/// Why:
/// Every stop and every wait here acts on the child the start answered,
/// and the reap only needs to know that it ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ChildProcess } from 'node:child_process';
/// ```
use std::process::Child;
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
/// `use std::time::Duration;` imports a span of time (its sibling `Instant` is a point on the clock).
/// Why:
/// The wait here is given the time that is left,
/// and its own deadline lives with the reap and the process group.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds
/// ```
use std::time::Duration;

/// The start of a child inside the containment (split out of this module).
pub(crate) use crate::bounded_run_start::start_contained;
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
        /// Shared behind an `Rc`.
        /// Its siblings are `Box`,
        /// which owns its value,
        /// and `Arc`,
        /// which shares across threads.
        /// Why `Rc` and not `Box` or `Arc`:
        /// this variant has to stay small enough for `clippy::large_enum_variant`
        /// now that a route names two programs,
        /// and the count is cheap because one thread uses it.
        group: Rc<ControlGroup>,
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
/// `fn contain() -> Result<Containment, RunError>` plans a scope,
/// or says in one line which capability is missing and falls back to a process group,
/// or refuses the call where the route is one no wait of this tool ends.
/// Why:
/// The notice is printed once per process,
/// and a refused route is the one answer that must not become a process group:
/// the manager has a job on the slice, so a scope made there is refused, and a scope launcher that waited for the job would wait
/// with no bound of this tool's (`RouteRefused`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function contain(): Containment  // throws RunError when the route is refused
/// ```
pub(crate) fn contain() -> Result<Containment, RunError> {
    return match find_route() {
        Ok(route) => match ControlGroup::plan(&route) {
            Ok(group) => Ok(Containment::Scope {
                group: Rc::new(group),
                route,
            }),
            Err(error) => Ok(weaker(GroupUnavailable {
                reason: format!("a scope name cannot be made ({error})"),
                stops_the_call: false,
            })),
        },
        // What:     `Err(unavailable) if unavailable.stops_the_call` is the refusal of a moment no wait ends, which stops the run
        //           before anything is started.
        // Why:      The weaker path would run the command behind another unit's stop, which is the wait this refusal exists to
        //           avoid.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (unavailable.stopsTheCall) throw new RunError.RouteRefused(unavailable.reason)
        // ```
        Err(unavailable) if unavailable.stops_the_call => Err(RunError::RouteRefused {
            reason: unavailable.reason,
        }),
        Err(unavailable) => Ok(weaker(unavailable)),
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
/// `fn stop_containment(containment, child) -> Result<(), String>` kills and reaps the direct child,
/// stops what the containment holds,
/// and answers `Ok` only if all of it is shown gone,
/// or the words a refusal carries saying what was not shown and why.
/// Why:
/// This is the proof of quiescence on its own,
/// so `stop_everything` can add the end of the scope unit after it on every outcome,
/// the refusal paths included.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopContainment(containment: Containment, child: ChildProcess): string | undefined
/// ```
pub(crate) fn stop_containment(
    containment: &Containment,
    child: &mut Child,
) -> Result<(), String> {
    return match containment {
        Containment::Scope {
            group,
            ..
        } => {
            let _ = child.kill();
            let reaped: Reaped = reap(child);
            let report: StopReport = match group.stop(QUIESCE_BOUND) {
                Ok(found) => found,
                Err(error) => {
                    eprintln!(
                        "cargo-shadow: warning: the scope {} could not be shown empty ({error})",
                        group.directory().display()
                    );
                    return Err(format!(
                        "the stop of {} could not be shown empty ({error})",
                        group.directory().display()
                    ));
                }
            };
            let scope: String = report.sentence(group.directory());
            return match refusal_of_scope(&report, &reaped, &scope) {
                Some(refusal) => Err(refusal),
                None => Ok(()),
            };
        }
        Containment::ProcessGroup => match Pid::from_raw(i32::try_from(child.id()).unwrap_or(0)) {
            Some(group) => match stop_process_group(child, group) {
                GroupStop::Gone => Ok(()),
                other => Err(other.refusal()),
            },
            None => Err(GroupStop::Unnamed.refusal()),
        },
    };
}

/// What:
/// `fn refusal_of_scope(report: &StopReport, reaped: &Reaped, scope: &str) -> Option<String>` is the refusal of a scope that did
/// not settle the run,
/// or `None` when it did.
/// Why:
/// Which of the two the stop could not show (the program,
/// or what the scope left) is one decision,
/// and every branch of it is a sentence that must say only what was found,
/// so the words are built in one place a control can drive with any report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusalOfScope(report: StopReport, reaped: Reaped, scope: string): string | undefined
/// ```
pub(crate) fn refusal_of_scope(
    report: &StopReport,
    reaped: &Reaped,
    scope: &str,
) -> Option<String> {
    // A direct child that was not reaped is a reason of its own, and the sentence after it says what the scope itself held.
    if let Reaped::NotReaped(shortfall) = reaped {
        return Some(format!("{}; {scope}", not_reaped_text(shortfall)));
    }
    if report.empty() {
        return None;
    }
    // The kernel did not read the scope as empty: the refusal names what the listing found and nothing more.
    return Some(format!("{}; {scope}", report.headline()));
}

/// What:
/// `fn not_reaped_text(shortfall: &ReapShortfall) -> String` is why the direct child could not be shown ended.
/// Why:
/// A bound that passed and a wait the kernel would not answer are different facts,
/// and only the first was a wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notReapedText(shortfall: ReapShortfall): string
/// ```
fn not_reaped_text(shortfall: &ReapShortfall) -> String {
    return match shortfall {
        ReapShortfall::Bound => {
            format!("the program did not end within {REAP_BOUND:?} after it was killed")
        }
        ReapShortfall::Unreadable(reason) => format!(
            "the exit of the program could not be read after it was killed ({reason}), so it cannot be shown to have ended"
        ),
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
/// `fn lose(scope: &Option<Rc<ScopeState>>, reason: String)` marks the open transaction as one whose writers may remain,
/// and keeps the reason a refusal carries.
/// Why:
/// The caller of the transaction reads the mark and stops the command instead of rolling back or finishing,
/// and two failed stops under one transaction must both be named:
/// the reason is appended,
/// never replaced (issue 6528's review,
/// Low 9).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lose(scope: ScopeState | undefined, reason: string): void
/// ```
pub(crate) fn lose(
    scope: &Option<Rc<ScopeState>>,
    reason: String,
) {
    if let Some(state) = scope {
        state.lost.set(true);
        let kept: Option<String> = state.lost_reason.borrow_mut().take();
        *state.lost_reason.borrow_mut() = match kept {
            Some(earlier) => Some(format!("{earlier}; {reason}")),
            None => Some(reason),
        };
    }
}

/// What:
/// `fn open_pidfd(pid: Pid) -> Option<OwnedFd>` opens a descriptor that becomes readable when the process ends,
/// or `None` when the kernel refuses.
/// Why:
/// `Option::and_then` takes it by path instead of a closure.
pub(crate) fn open_pidfd(pid: Pid) -> Option<OwnedFd> {
    return pidfd_open(pid, PidfdFlags::empty()).ok();
}
