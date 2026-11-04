//! What:
//!  How long a Git step is waited for and what happens at the bound: ended at the class ceiling, never ended but left
//!  running after the ceiling plus a short wait, or unbounded on the landing thread.
//! Why:
//!  The runner (`transaction_git.rs`) asks for the bound of each step it watches; the choice depends on the step's
//!  class and on the rules for which steps may be ended (`transaction_git_step_rerun_and_end_rules.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // stepBound(request): StepBound
//! ```

/// The kind of step a command belongs to, and its bounded waits.
use crate::child_class::{ChildClass, class_of};
use crate::child_watch::StepBound;
use crate::transaction_git_request_and_reply_types::GitRequest;
use crate::transaction_git_step_rerun_and_end_rules::{is_side_ref_write, may_be_ended};
use crate::wait_ceilings::{OBJECT_STEP_CEILING, PLUMBING_STEP_CEILING};
/// A span of time, for the ceilings.
use std::time::Duration;

/// What:
///  How long the request's step is waited for, and what happens then.
/// Why:
///   A step that can be run twice is ended at the ceiling of its class, whatever it shows. A step that cannot is never ended
///  (owner, 2026-11-04: "Never cut a branch write (Recommended)"): on the commit service's landing thread it is waited for, and the
///  clients that wait behind it are told what it is stuck in (`service_busy.rs`); anywhere else the command that started it stops
///  waiting at the bound of the step's class plus the short extra wait, with an error saying the result is not known, and the step is
///  left running. In a unit test the bound of the class can be shortened for the thread that runs the control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stepBound(request): StepBound { return !cannotRunTwice(request) ? endAt(classCeiling) : onLandingThread() ? unbounded : leaveRunning(classCeiling, SHORT_EXTRA_WAIT) }
/// ```
pub(crate) fn step_bound(request: &GitRequest) -> StepBound {
    #[cfg(test)]
    let shortened: Option<Duration> = STEP_CEILING_FOR_THIS_THREAD.with(std::cell::Cell::get);
    #[cfg(not(test))]
    let shortened: Option<Duration> = None;
    let class_ceiling: Duration = shortened.unwrap_or(match class_of(request.arguments.as_slice()) {
        ChildClass::Plumbing => PLUMBING_STEP_CEILING,
        ChildClass::Object => OBJECT_STEP_CEILING,
    });
    if may_be_ended(request) {
        return StepBound::EndAt(class_ceiling);
    }
    // A write of one of the wrapper's own side refs is never ended, but it is not the branch write the owner's "Never cut a branch write
    //  (Recommended)" is about, and "nothing should ever wait forever or indefinitely" holds for it: even the landing thread stops
    //  waiting at the bound and leaves it running, with the not-known error.
    if is_side_ref_write(request) {
        return leave_running_bound(class_ceiling);
    }
    return not_ended_bound(class_ceiling);
}

/// The bound of a step that is never ended and never waited for without a bound, on any thread: left running after the class ceiling
///  plus the short extra wait.
fn leave_running_bound(class_ceiling: Duration) -> StepBound {
    #[cfg(test)]
    let extra: Duration = EXTRA_WAIT_FOR_THIS_THREAD.with(std::cell::Cell::get).unwrap_or(crate::wait_ceilings::SHORT_EXTRA_WAIT);
    #[cfg(not(test))]
    let extra: Duration = crate::wait_ceilings::SHORT_EXTRA_WAIT;
    return StepBound::LeaveRunning { after: class_ceiling, extra };
}

/// What:
///  The bound of a step that is never ended: unbounded on the landing thread, else left running after the class ceiling plus the
///  short extra wait.
/// Why:
///   Both `step_bound` and `step_bound_checked` give a step this bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// onLandingThread() ? unbounded : leaveRunning(classCeiling, SHORT_EXTRA_WAIT)
/// ```
fn not_ended_bound(class_ceiling: Duration) -> StepBound {
    if crate::service_busy::on_landing_thread() {
        return StepBound::Unbounded;
    }
    return leave_running_bound(class_ceiling);
}

#[cfg(test)]
thread_local! {
    /// A shortened ceiling for the steps the current thread starts, so a control ends a step in a moment.
    pub(crate) static STEP_CEILING_FOR_THIS_THREAD: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
    /// A shortened extra wait for the steps the current thread leaves running.
    pub(crate) static EXTRA_WAIT_FOR_THIS_THREAD: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}
