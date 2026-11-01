//! Small pieces of the update traffic policy engine:
//! the clock and the stop check that make long passes testable and interruptible,
//! the retry wait,
//! and the check whether the user's manager can move a cgroup's processes.

/// Names a user chose,
/// delimited so a cause ends them where the line does.
use crate::report_cause::quote_name;
/// Retry timing.
use std::time::{Duration, Instant};

/// Wait before the second attempt;
/// each later wait doubles.
const RETRY_BACKOFF_BASE: Duration = Duration::from_secs(1);
/// Longest wait between attempts for one process.
const RETRY_BACKOFF_MAX: Duration = Duration::from_secs(60);
/// Reports whether a cgroup path can be adopted by the user's manager.
/// The path must lie in the user manager's subtree,
/// and must not be the cgroup of a service's main process.
pub fn adoptable_cgroup(cgroup: &str, uid: u32) -> Result<(), String> {
    let prefix = format!("/user.slice/user-{uid}.slice/user@{uid}.service/");
    if !cgroup.starts_with(&prefix) {
        return Err(format!(
            "its cgroup {} is outside the user manager's subtree, which the manager cannot move processes out of",
            quote_name(cgroup)
        ));
    }
    let leaf = cgroup.rsplit('/').next().unwrap_or("");
    if leaf.ends_with(".service") {
        return Err(format!("it is the main process of the service {}, and moving it would empty that service", quote_name(leaf)));
    }
    return Ok(());
}

/// The wait before attempt number `attempt` plus one:
/// one second after the first failure,
/// doubling,
/// capped.
pub fn retry_backoff(attempt: u32) -> Duration {
    let doublings = attempt.saturating_sub(1).min(6);
    return RETRY_BACKOFF_BASE.saturating_mul(1 << doublings).min(RETRY_BACKOFF_MAX);
}

/// A source of the current time.
pub trait Clock {
    /// The current time.
    fn now(&self) -> Instant;
}

/// The real clock.
pub struct SystemClock;

/// The real clock reads the system.
impl Clock for SystemClock {
    /// The current time.
    fn now(&self) -> Instant {
        return Instant::now();
    }
}

/// Tells a long pass whether to stop early,
/// for example because a shutdown signal arrived.
pub trait StopCheck {
    /// Whether the pass should stop now.
    fn stop_requested(&self) -> bool;
}

/// A check that never asks to stop,
/// for tests.
#[cfg(test)]
pub struct NeverStop;

/// The check that never stops.
#[cfg(test)]
impl StopCheck for NeverStop {
    /// Never.
    fn stop_requested(&self) -> bool {
        return false;
    }
}
