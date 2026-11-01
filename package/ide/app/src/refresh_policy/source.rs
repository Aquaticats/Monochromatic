//! Decide when the displayed file is reread.

/// Intervals shared with the directory schedule.
use super::{REREAD_GAP, SAFETY_SWEEP, UNWATCHED_SOURCE_POLL, WRITE_QUIET, WRITE_WAIT_LIMIT};
/// Notifications classify a change as finished or still being written.
use crate::change_watch::SourceChange;
/// What:
///  `Instant` is a monotonic point in time;
///  it never jumps with the wall clock.
/// Why:
///  Intervals stay correct across clock adjustments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const now = performance.now();
/// ```
use std::time::Instant;

/// Displayed-file reread schedule;
///  the native reload tick owns one inside the source state.
#[derive(Clone, Debug, Default)]
pub struct SourceRefresh {
    /// The displayed file's directory has a live watch,
    ///  so timers fall back to the safety sweep.
    watched: bool,
    /// The displayed file lies outside the project:
    ///  its folder is deliberately not watched,
    ///  and the
    /// safety sweep,
    ///  not the 250 ms timer for a failed watch,
    ///  rereads it.
    outside_project: bool,
    /// What:
    ///  `Option<Instant>` is a time or nothing (`number | undefined`).
    /// Why:
    ///  When the first unread notification arrived;
    ///  bounds the wait for an unfinished write.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pendingSince?: number;
    /// ```
    pending_since: Option<Instant>,
    /// Time of the latest notification when it was an unfinished write;
    ///  `None` when it was settled.
    unsettled_at: Option<Instant>,
    /// The pending change includes a real write notification;
    ///  a pending reread alone does not.
    notified: bool,
    /// When the last read was admitted by the reader;
    ///  `None` before the first read.
    last_request: Option<Instant>,
}

/// Pure scheduling:
///  callers pass the current time,
///  so tests choose it.
impl SourceRefresh {
    /// Record whether the displayed file's directory currently has a live watch.
    pub fn set_watched(&mut self, watched: bool) {
        if self.watched != watched {
            tracing::debug!(watched, "displayed-file refresh mode changed");
        }
        self.watched = watched;
    }

    /// Record whether the displayed file lies outside the project,
    ///  where nothing is watched on purpose.
    pub fn set_outside_project(&mut self, outside: bool) {
        if self.outside_project != outside {
            tracing::debug!(outside, "displayed file moved across the project boundary");
        }
        self.outside_project = outside;
    }

    /// Report whether the displayed file's directory currently has a live watch.
    pub fn is_watched(&self) -> bool {
        return self.watched;
    }

    /// True while a notification is unread:
    ///  the next read is the one it asked for,
    ///  which waited for the
    /// writer.
    ///  Every other read (the timer,
    ///  the sweep,
    ///  a highlighting retry,
    ///  the first read) is not.
    pub fn has_unread_change(&self) -> bool {
        return self.pending_since.is_some() && self.notified;
    }

    /// Record a notification;
    ///  the latest write classification wins,
    ///  so delete then rewrite waits for the write.
    /// A reread keeps whatever write is already pending:
    ///  an unfinished write still waits.
    pub fn changed(&mut self, change: SourceChange, now: Instant) {
        let first = self.pending_since.is_none();
        if first {
            self.pending_since = Some(now);
        }
        // What: `match` on the three-variant enum chooses which timestamp to keep.
        // Why: A later close-write settles an earlier unfinished write; a later write unsettles a delete;
        //      a reread with no write behind it changes nothing that a real notification already set.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (change === 'settled') { this.unsettledAt = undefined; this.notified = true; }
        // else if (change === 'unsettled') { this.unsettledAt = now; this.notified = true; }
        // else if (first) this.notified = false;
        // ```
        match change {
            SourceChange::Settled => {
                self.unsettled_at = None;
                self.notified = true;
            }
            SourceChange::Unsettled => {
                self.unsettled_at = Some(now);
                self.notified = true;
            }
            SourceChange::Reread => {
                if first {
                    self.notified = false;
                }
            }
        }
    }

    /// True when a read should start now:
    ///  the first read,
    ///  a settled or quiet change,
    ///  missing
    /// highlighting,
    ///  or the timer.
    ///  `highlight_missing` says the displayed revision has no accepted highlighting.
    /// While an unfinished write is waiting,
    ///  neither highlighting nor the timer starts a read.
    pub fn due(&self, now: Instant, highlight_missing: bool) -> bool {
        // What: `let ... else` binds `Some(last)` or returns early when there was no read yet.
        // Why: The first read happens immediately, as it did under polling.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.lastRequest === undefined) return true;
        // ```
        let Some(last) = self.last_request else {
            return true;
        };
        // Requests outside the timers wait this long after the previous read started, so a file
        // rewritten many times per second, or one that cannot be read, is not reread on every tick.
        let rested = now.saturating_duration_since(last) >= REREAD_GAP;
        // What: `if let Some(since) = ...` runs only while a notification is unread; every path in it returns.
        // Why: An unread notification alone decides. A timer or a highlighting request must not read a
        //      file whose write is unfinished; that wait ends by itself within `WRITE_WAIT_LIMIT`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.pendingSince !== undefined) return rested && (settled || quiet || waitedLongEnough);
        // ```
        if let Some(since) = self.pending_since {
            if !rested {
                return false;
            }
            match self.unsettled_at {
                None => {
                    return true;
                }
                Some(at) => {
                    return now.saturating_duration_since(at) >= WRITE_QUIET
                        || now.saturating_duration_since(since) >= WRITE_WAIT_LIMIT;
                }
            }
        }
        if highlight_missing && rested {
            return true;
        }
        // A file outside the project is not watched on purpose; it is not a failed watch to poll often.
        let interval = if self.watched || self.outside_project {
            SAFETY_SWEEP
        } else {
            UNWATCHED_SOURCE_POLL
        };
        return now.saturating_duration_since(last) >= interval;
    }

    /// A read was admitted:
    ///  clear pending notifications;
    ///  later ones schedule another read.
    pub fn requested(&mut self, now: Instant) {
        self.last_request = Some(now);
        self.pending_since = None;
        self.unsettled_at = None;
        self.notified = false;
    }
}
