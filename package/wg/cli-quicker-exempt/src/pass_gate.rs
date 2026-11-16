//! When one pass of the policy may spend another move.
//! A pass is one walk over the process list:
//! a manager that answers slowly,
//! or refuses every move,
//! must not keep the watcher walking a list it cannot act on,
//! and a manager that stopped answering is left alone for a while instead of being asked again at once.
//! The gate is the whole of that policy,
//! kept apart from the decisions about which process to move,
//! so the two can be read and tested one at a time.

/// The durations and instants of a pass.
use std::time::{Duration, Instant};

/// Failed moves one pass may spend before it postpones the rest,
/// so a manager that fails slowly cannot stall the watcher for long.
const FAILED_ATTEMPTS_PER_PASS: u32 = 2;

/// Time one pass may spend on moves before it postpones the rest.
/// A pass can overrun it by the one attempt in progress,
/// which the client timeout bounds.
const PASS_MOVE_BUDGET: Duration = Duration::from_secs(2);

/// How long no move is attempted after the manager did not answer in time.
pub const BLOCK_AFTER_TIMEOUT: Duration = Duration::from_secs(10);

/// What one pass has spent,
/// and when moves may go ahead again.
#[derive(Debug, Default)]
pub struct PassGate {
    /// Failed moves spent in the current pass.
    failed_attempts: u32,
    /// No move is attempted before this time.
    blocked_until: Option<Instant>,
    /// When the current pass began.
    started: Option<Instant>,
}

/// Recording what a pass spent,
/// and asking whether it may spend more.
impl PassGate {
    /// A gate of a pass that has not begun.
    pub fn new() -> PassGate {
        return PassGate::default();
    }

    /// Starts a pass at `now`:
    /// failed moves spent earlier no longer count against the next ones.
    pub fn begin(&mut self, now: Instant) {
        self.failed_attempts = 0;
        self.started = Some(now);
    }

    /// Counts one failed move against the current pass.
    pub fn note_failed_attempt(&mut self) {
        self.failed_attempts = self.failed_attempts.saturating_add(1);
    }

    /// Stops every move until `BLOCK_AFTER_TIMEOUT` has passed,
    /// which is what a manager that did not answer earns.
    pub fn block(&mut self, now: Instant) {
        self.blocked_until = Some(now + BLOCK_AFTER_TIMEOUT);
    }

    /// Reports whether moves are paused after a client timeout.
    pub fn is_blocked(&self, now: Instant) -> bool {
        return match self.blocked_until {
            Some(until) => now < until,
            None => false,
        };
    }

    /// Why a move must wait now,
    /// or `None` when it may go ahead.
    pub fn postponement(&self, now: Instant) -> Option<&'static str> {
        if self.is_blocked(now) {
            return Some("the user's service manager did not answer in time, so moves are paused for a few seconds");
        }
        if self.failed_attempts >= FAILED_ATTEMPTS_PER_PASS {
            return Some("earlier moves in this pass failed, and a pass spends at most two failed attempts");
        }
        if let Some(started) = self.started
            && now.duration_since(started) >= PASS_MOVE_BUDGET
        {
            return Some("this pass used its time budget for moves");
        }
        return None;
    }
}
