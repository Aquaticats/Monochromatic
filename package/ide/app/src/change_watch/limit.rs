//! Retries while this user's inotify watch limit is reached.
//! The limit is shared by every program of the user,
//!  so a failed watch is usually not this application's
//! doing,
//!  and retrying every directory on every safety sweep would only add failing calls.
//!  After the first
//! failure,
//!  a sweep may retry only when the backoff allows;
//!  a change of what is shown retries at once.

/// What:
///  `Duration` is a time span;
///  `Instant` a monotonic time point (`SystemTime`,
///  the wall-clock sibling,
///       can jump when the clock is set).
/// Why:
///  Backoff deadlines must not move when the user changes the clock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const now = performance.now();
/// ```
use std::time::{Duration, Instant};

/// Wait before the first sweep retry after the limit was reached.
pub const FIRST_LIMIT_RETRY: Duration = Duration::from_secs(2);

/// Longest wait between sweep retries;
///  the wait doubles from `FIRST_LIMIT_RETRY` up to this.
pub const LONGEST_LIMIT_RETRY: Duration = Duration::from_secs(64);

/// When the next sweep may retry watches that failed at the limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LimitBackoff {
    /// Earliest time a sweep may retry.
    next_retry: Instant,
    /// The wait that produced `next_retry`;
    ///  the next failure doubles it.
    wait: Duration,
}

/// Pure scheduling:
///  callers pass the current time,
///  so tests choose it.
impl LimitBackoff {
    /// The limit was just reached:
    ///  the first sweep retry waits `FIRST_LIMIT_RETRY`.
    pub fn reached(now: Instant) -> Self {
        return Self {
            next_retry: now + FIRST_LIMIT_RETRY,
            wait: FIRST_LIMIT_RETRY,
        };
    }

    /// True when a sweep may retry now.
    pub fn may_retry(&self, now: Instant) -> bool {
        return now >= self.next_retry;
    }

    /// A sweep retry hit the limit again:
    ///  wait twice as long,
    ///  up to `LONGEST_LIMIT_RETRY`.
    pub fn failed_again(&mut self, now: Instant) {
        // What: `saturating_mul` doubles without overflowing; `min` caps the result.
        // Why: The wait grows quickly while the limit stays reached, but a retry still happens about every minute.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.wait = Math.min(this.wait * 2, LONGEST_LIMIT_RETRY);
        // ```
        self.wait = self.wait.saturating_mul(2).min(LONGEST_LIMIT_RETRY);
        self.next_retry = now + self.wait;
    }

    /// How long until a retry is allowed;
    ///  zero once it is.
    pub fn until_retry(&self, now: Instant) -> Duration {
        // `saturating_duration_since` gives zero instead of failing when `now` is already past.
        return self.next_retry.saturating_duration_since(now);
    }

    /// The wait before the next sweep retry,
    ///  for the log line.
    pub fn wait(&self) -> Duration {
        return self.wait;
    }
}
