//! What:
//!  Where a control says which ordering must have happened before a run's limit may be reached by elapsed time.
//! Why:
//!  A run stopped at a fixed limit decides *when* it is stopped by the clock alone,
//! and a control that needs the run
//!  to have reached some state first would otherwise bet that the state arrives inside the limit:
//! on a loaded host a
//!  `strace`-wrapped Git took 2.9 s to take its lock against a 2.5 s limit,
//! so the run was stopped before the state the
//!  control was about to check ever existed
//! (`doc/troubleshooting/git-policy-cli-stop-lock-controls-wait-on-order-not-time.md`).
//! A control installs an ordering,
//!  and the limit is held until it holds,
//! so the stop is decided by the ordering and then by the time the control asked
//!  for,
//! never by time alone.
//! The hold has a ceiling,
//! so a control whose ordering never happens fails its own
//!  assertion with its own words instead of waiting for the run's watchdog.
//!  Nothing of this is in the release executable:
//! without a control a limit is reached by time as before,
//! and the
//!  thread-local that carries the hold is compiled out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = holdLimitForAnOrdering({ reached: () => lockExists(), ceilingMs: 18_000 });
//! // Clock.check(): return elapsed >= limit && !held.isHolding();
//! ```

#[cfg(test)]
use std::cell::RefCell;
use std::time::Duration;
#[cfg(test)]
use std::time::Instant;

/// The ordering a run's limit is held for,
/// and how long the hold may last.
#[cfg(test)]
pub(crate) struct LimitHeldForAnOrdering {
    /// Whether the ordering the control needs has happened.
    pub(crate) reached: fn() -> bool,
    /// How long the limit is held back at most.
    pub(crate) ceiling: Duration,
}

#[cfg(test)]
thread_local! {
    /// The ordering the limit of a run on this thread is held for,
    /// and when the hold began;
    /// nothing outside a control.
    static LIMIT_HELD_FOR: RefCell<Option<(LimitHeldForAnOrdering, Instant)>> = const { RefCell::new(None) };
}

/// What:
///  Puts `held` in force on this thread until the returned guard drops.
/// Why:
///   The hold belongs to the control that installed it and to no other:
/// a run on another thread,
/// and every run
///  after the guard drops,
/// reaches its limit by time as before.
#[cfg(test)]
pub(crate) fn hold_limit_for_an_ordering(held: LimitHeldForAnOrdering) -> LimitHoldGuard {
    LIMIT_HELD_FOR.with(|slot| { *slot.borrow_mut() = Some((held, Instant::now())); });
    return LimitHoldGuard;
}

/// Ends the hold of this thread's limit when dropped.
#[cfg(test)]
pub(crate) struct LimitHoldGuard;

#[cfg(test)]
impl Drop for LimitHoldGuard {
    fn drop(&mut self) {
        LIMIT_HELD_FOR.with(|slot| { *slot.borrow_mut() = None; });
    }
}

/// Whether a control on this thread is holding the limit for an ordering that has not happened yet,
/// and is still inside its ceiling.
#[cfg(test)]
fn limit_is_held() -> bool {
    let held: Option<bool> = LIMIT_HELD_FOR.with(|slot| {
        let installed = slot.borrow();
        let (held, began) = installed.as_ref()?;
        return Some(began.elapsed() < held.ceiling && !(held.reached)());
    });
    return held.unwrap_or(false);
}

/// Whether `limit` has passed since the run started,
/// on a run whose control may be holding it for an ordering.
#[cfg(test)]
pub(crate) fn limit_has_passed(elapsed: Duration, limit: Duration) -> bool {
    return elapsed >= limit && !limit_is_held();
}

/// Whether `limit` has passed since the run started.
#[cfg(not(test))]
pub(crate) fn limit_has_passed(elapsed: Duration, limit: Duration) -> bool {
    return elapsed >= limit;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "push_limit_held_for_an_ordering_tests.rs"]
mod tests;
