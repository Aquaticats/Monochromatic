//! What:
//!  The repositories the service must look at again, and when: those with a claim that another instance holds and that may run
//!  out its lease while this service sits idle.
//! Why:
//!  A restarted service that finds the claims of the instance it replaces cannot take them before their lease is over, and
//!  nothing wakes it when the lease is: the next commit might be hours away, and the work would sit claimed by a holder that
//!  died. The work loop asks this schedule how long to wait for a wake before it looks again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const wait = schedule.nextWait(now); const due = schedule.takeDue(now); schedule.replace(ran, retries, now);
//! ```

use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Added to a wait so that the lease is over when the service looks, not exactly at its edge.
pub const RETRY_SLACK: Duration = Duration::from_secs(1);

/// What:
///  The repositories to look at again, each with the time to do it.
/// Why:
///   One entry per repository, replaced by the newest answer of a round that looked at it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RetrySchedule = Map<string, number>;
/// ```
#[derive(Debug, Default)]
pub struct RetrySchedule {
    /// The repositories and when each is due.
    due: Vec<(PathBuf, Instant)>,
}

/// Methods over the schedule.
impl RetrySchedule {
    /// Whether nothing is waiting to be looked at again.
    pub fn is_empty(&self) -> bool {
        return self.due.is_empty();
    }

    /// How long until the first entry is due (nothing at all when the schedule is empty, zero for one already due).
    pub fn next_wait(&self, now: Instant) -> Option<Duration> {
        let mut least: Option<Duration> = None;
        for (_directory, when) in &self.due {
            let wait: Duration = when.saturating_duration_since(now);
            least = Some(match least {
                Some(known) => known.min(wait),
                None => wait,
            });
        }
        return least;
    }

    /// Removes and returns the repositories that are due at `now`.
    pub fn take_due(&mut self, now: Instant) -> Vec<PathBuf> {
        // The entries due now leave the schedule in order; the others stay in order.
        let mut due: Vec<PathBuf> = Vec::new();
        let mut waiting: Vec<(PathBuf, Instant)> = Vec::new();
        for entry in std::mem::take(&mut self.due) {
            if entry.1 <= now {
                due.push(entry.0);
            } else {
                waiting.push(entry);
            }
        }
        self.due = waiting;
        return due;
    }

    /// Forgets what was scheduled for the repositories a round just ran, and schedules what that round asked for.
    pub fn replace(&mut self, ran: &[PathBuf], retries: Vec<(PathBuf, Duration)>, now: Instant) {
        let mut kept: Vec<(PathBuf, Instant)> = Vec::new();
        for entry in std::mem::take(&mut self.due) {
            if !ran.contains(&entry.0) {
                kept.push(entry);
            }
        }
        self.due = kept;
        for (directory, wait) in retries {
            self.due.push((directory, now + wait + RETRY_SLACK));
        }
    }
}

/// Schedule controls stay out of the release executable.
#[cfg(test)]
#[path = "service_retry_tests.rs"]
mod tests;
