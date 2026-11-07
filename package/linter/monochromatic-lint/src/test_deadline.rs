//! What:
//!  A deadline for a probe that a wrong loop could keep running forever.
//! Why:
//!  A mutated or broken scanner loop that stops advancing never fails an assertion,
//!  it only hangs the whole test run until an outer limit kills it.
//!  Running the probe on its own thread with a deadline turns that hang into an ordinary test failure.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await Promise.race([work(), rejectAfter(DEADLINE)]);
//! ```

/// Import the channel that carries the probe's answer and the deadline type.
use std::{sync::mpsc, time::Duration};

/// How long a probe may run before it counts as not finishing.
/// The probes of these controls scan a few dozen bytes:
///  the container run of the whole suite (883 tests,
///  `tests-batch1.log` of 2026-11-07) reported 0.17 s for the slowest closure test and
///  under 0.02 s for every scan test,
///  so thirty seconds is orders of magnitude over a finishing probe
///  and still well inside the 180 second limit of a mutation campaign.
const DEADLINE: Duration = Duration::from_secs(30);

/// What:
///  Run a probe on a new thread and return its answer,
///  or fail the test when it panics or does not finish before the deadline.
/// Why:
///  The probe's thread is left to run when the deadline passes,
///  because a thread cannot be stopped,
///  and the failing test ends the process's interest in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishedWithinDeadline<T>(work: () => T): T;
/// ```
pub(crate) fn finished_within_deadline<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> T {
    let (sender, receiver) = mpsc::channel::<T>();
    std::thread::spawn(move || {
        // A send fails only after the deadline passed and the receiver is gone, which nobody reads.
        let _ = sender.send(work());
    });
    return match receiver.recv_timeout(DEADLINE) {
        Ok(answer) => answer,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("the probe did not finish within {DEADLINE:?}")
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("the probe panicked before it answered")
        }
    };
}
