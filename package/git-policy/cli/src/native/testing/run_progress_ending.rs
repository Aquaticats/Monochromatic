//! What:
//!  Why one run was ended:
//! it showed nothing,
//! its counters stopped being readable,
//! or this system counts no work at all,
//! and the sentence a person reads for each.
//! Why:
//!   The three endings are told apart in words,
//! so a person can tell a stuck test from a system without the counters,
//! and the figure that decided it is in the same sentence.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Ending = { kind: 'quiet'; quietMs: number } | { kind: 'unreadable'; unreadMs: number } | { kind: 'unmeasurable'; elapsedMs: number };
//! ```

/// Time spans,
/// which every reason carries.
use std::time::Duration;

/// What:
///  Why one run was ended.
/// Why:
///   A run that showed nothing,
/// a run whose counters stopped being readable,
/// and a system that counts no work at all
///  end for different reasons,
/// and the message has to say which,
/// so the person can tell a stuck test from a system
///  without the counters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ending =
///     | { kind: 'quiet'; quietMs: number }
///     | { kind: 'unreadable'; unreadMs: number }
///     | { kind: 'unmeasurable'; elapsedMs: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Ending {
    /// Nothing moved for the window.
    Quiet(Duration),
    /// The window passed and no task's counters could be read,
    /// though earlier looks read them.
    Unreadable(Duration),
    /// The window passed and this system counts no work,
    /// so the wall clock is its bound.
    Unmeasurable(Duration),
}

/// The reason in words,
/// with the figure that decided it and the signs that were read.
impl std::fmt::Display for Ending {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self {
            Ending::Quiet(quiet) => write!(
                formatter,
                "showed no sign of work for {:.1} s (no task able to run or waiting for the disk, no processor time, no bytes moved, and no process or task started)",
                quiet.as_secs_f64()
            ),
            Ending::Unreadable(unread) => write!(
                formatter,
                "could not read its tasks' counters for {:.1} s (a task that ended while it was read, or a process that ran out of file descriptors), so it is ended rather than watched for ever",
                unread.as_secs_f64()
            ),
            Ending::Unmeasurable(elapsed) => write!(
                formatter,
                "ran {:.1} s and this system counts no work of its tasks, so its wall clock is the bound",
                elapsed.as_secs_f64()
            ),
        };
    }
}
