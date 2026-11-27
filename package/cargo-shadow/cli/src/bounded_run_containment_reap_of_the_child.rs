//! What:
//! The reap of the direct child a run started:
//! whether it ended,
//! or the bound passed first,
//! or the kernel refused to answer.
//! Why:
//! `Child::wait` has no deadline,
//! so the wait is this program's own,
//! and a refusal must not state a wait that was not the wait:
//! a `try_wait` the kernel refuses answers at once (issue 6528's review,
//! Low 3).
//! Split out of `bounded_run_containment.rs`,
//! which keeps the stop that calls it,
//! to keep that file under the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded-run-containment-reap-of-the-child.ts
//! export function reap(child: ChildProcess): 'ended' | { shortfall: string }
//! ```

/// What:
/// `use crate::bounded_run::{POLL_INTERVAL, REAP_BOUND};` imports how often the child is asked and how long it is given.
/// Why:
/// The wait asks at one interval and ends at one bound,
/// and both are the package's own picks rather than numbers written twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { POLL_INTERVAL, REAP_BOUND } from 'crate/bounded_run';
/// ```
use crate::bounded_run::{
    POLL_INTERVAL,
    REAP_BOUND,
};
/// What:
/// `use std::process::Child;` imports the handle of a started child.
/// Why:
/// The wait asks that handle whether the child ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ChildProcess } from 'node:child_process';
/// ```
use std::process::Child;
/// What:
/// `use std::thread;` imports the thread sleep of the interval between two questions.
/// Why:
/// A wait without a pidfd sleeps between polls,
/// and the reap is that wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await new Promise((resolve) => setTimeout(resolve, ms))
/// ```
use std::thread;
/// What:
/// `use std::time::Instant;` imports a point on the clock (its sibling `Duration` is a span of time).
/// Why:
/// The reap computes its deadline from the moment it started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now()
/// ```
use std::time::Instant;

/// Why a direct child was not reaped.
pub(crate) enum ReapShortfall {
    /// The bound passed before the child ended.
    Bound,
    /// Its exit could not be read,
    /// with the reason,
    /// so no wait was spent on a question the kernel refused.
    Unreadable(
        /// Why the exit could not be read.
        String,
    ),
}

/// What became of the direct child when the run ended.
pub(crate) enum Reaped {
    /// The child ended,
    /// so it was reaped.
    Ended,
    /// The child was not reaped,
    /// with the reason.
    NotReaped(
        /// Why the child was not reaped.
        ReapShortfall,
    ),
}

/// What:
/// `fn reap(child: &mut Child) -> Reaped` waits for the direct child,
/// a few milliseconds at a time,
/// up to `REAP_BOUND`,
/// and says whether the wait was spent or the kernel refused to answer.
/// Why:
/// `Child::wait` has no deadline,
/// and a refusal must not state a wait that was not the wait:
/// a failed `try_wait` answers at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reap(child: ChildProcess): 'ended' | { shortfall: string }
/// ```
pub(crate) fn reap(child: &mut Child) -> Reaped {
    // A child that cannot be reaped within its bound is "not reaped"; a test makes that so without a process in an uninterruptible wait.
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::ReapStuck) {
        return Reaped::NotReaped(ReapShortfall::Bound);
    }
    let deadline: Instant = Instant::now() + REAP_BOUND;
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => return Reaped::Ended,
            Ok(None) => {}
            // The kernel will not answer again, so no further wait is spent; the refusal says the exit could not be read.
            Err(error) => return Reaped::NotReaped(ReapShortfall::Unreadable(error.to_string())),
        }
        if Instant::now() >= deadline {
            return Reaped::NotReaped(ReapShortfall::Bound);
        }
        thread::sleep(POLL_INTERVAL);
    }
}
