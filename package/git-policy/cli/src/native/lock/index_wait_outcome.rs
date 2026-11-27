//! What:
//!  What came of one wait for any program's hold on Git's `index.lock`:
//!  nobody held it,
//!  a holder let it go,
//!  its PID file names a process that is gone,
//!  it was still held at the ceiling,
//!  or another program's command holds it and the merge does not wait.
//! Why:
//!  Every caller of a wait answers these five cases and no others,
//!  so one record names them once,
//!  and a merge that saves itself can say who it left the checkout to.
//!  It is a record of its own so that a look at the lock and the wait for it
//!  can each be read without the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type IndexWait = { kind: 'not-held' } | { kind: 'released'; waited: Duration } | { kind: 'owner-gone'; waited: Duration } | { kind: 'gave-up'; waited: Duration } | { kind: 'held-by-another-command'; holder: string };
//! ```

/// `Duration` is a span of monotonic time,
/// and not a wall-clock `SystemTime`.
use std::time::Duration;

/// What came of the wait.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexWait {
    /// Nobody held the lock when the command asked.
    NotHeld,
    /// A holder had the lock and let it go.
    Released(
        /// How long the wait lasted.
        Duration,
    ),
    /// The lock's PID file names a process that no longer runs,
    /// so nothing will release it;
    /// the diagnostic was written,
    ///  nothing was written by the wait and the lock was left in place.
    OwnerGone(
        /// How long the wait lasted.
        Duration,
    ),
    /// The lock was still held at the ceiling;
    /// the line naming the blocker was written,
    /// nothing was written by the wait and the lock was left
    ///  in place.
    GaveUp(
        /// How long the wait lasted.
        Duration,
    ),
    /// Another program's command holds the lock,
    /// and the merge does not wait for it:
    /// the holder is named so the merge that saves itself can say who it left the checkout to.
    HeldByAnotherCommand(
        /// The holder,
        /// as the person reads it.
        String,
    ),
}
