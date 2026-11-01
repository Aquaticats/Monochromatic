//! What:
//!  One time budget for everything a lock holder does while it holds the push lock.
//! Why:
//!  A waiter gives up after a limit, and a holder that is still working inside its rights must not make a waiter give
//!  up. Every bounded step of the holder (the LFS upload, the push, the fetches and merge pushes of a reconcile, the
//!  Git queries between them) used to have a full deadline of its own, so the holder's total had no bound that the
//!  waiter's limit could be derived from. While a budget is in force, each step's deadline is the smaller of its own
//!  deadline and what is left of the budget, and no step starts once the budget is spent. The waiter's limit is then
//!  the budget plus what one in-flight step may overrun it by, plus a margin (`bounded_push::lock_wait_limit`).
//!  The budget belongs to the thread that holds the lock; a process holds one push lock at a time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! using budget = enterHolderBudget(3 * pushDeadline); const deadline = Math.min(stepDeadline, budget.remaining());
//! ```

/// The monotonic clock and time spans.
use std::time::{Duration, Instant};

thread_local! {
    /// When the budget in force began and how long it is, or nothing when no lock is held.
    static IN_FORCE: std::cell::Cell<Option<(Instant, Duration)>> = const { std::cell::Cell::new(None) };
}

/// Ends the budget when the holder lets go of the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using budget = enterHolderBudget(duration);
/// ```
pub struct HolderBudget;

impl Drop for HolderBudget {
    /// Leaves no budget in force.
    fn drop(&mut self) {
        IN_FORCE.with(|cell| {
            cell.set(None);
        });
    }
}

/// What:
///  Start the budget of a holder that has just taken the lock.
/// Why:
///   The clock starts when the lock is taken, which is when the waiters' waits start counting on the holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function enterHolderBudget(total: number): Disposable;
/// ```
pub fn enter(total: Duration) -> HolderBudget {
    IN_FORCE.with(|cell| {
        cell.set(Some((Instant::now(), total)));
    });
    return HolderBudget;
}

/// What is left of the budget in force, or nothing when none is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remaining(): number | undefined
/// ```
pub fn remaining() -> Option<Duration> {
    return IN_FORCE.with(|cell| {
        return cell.get().map(|(began, total)| {
            return total.saturating_sub(began.elapsed());
        });
    });
}

/// Whether a budget is in force and spent, so no further step may start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// remaining() === 0
/// ```
pub fn exhausted() -> bool {
    return remaining().is_some_and(|left| {
        return left.is_zero();
    });
}

/// A step's deadline under the budget in force: the smaller of its own deadline and what is left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.min(deadline, remaining() ?? Infinity)
/// ```
pub fn clamp(deadline: Duration) -> Duration {
    return remaining().map_or(deadline, |left| {
        return deadline.min(left);
    });
}

/// The budget is controlled in the test module.
#[cfg(test)]
#[path = "holder_budget_tests.rs"]
mod tests;
