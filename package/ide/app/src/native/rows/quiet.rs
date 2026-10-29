//! The scrolling rule's clock:
//!  whether the reader scrolled so recently that a wheel animation may still be running,
//!  and the instant a test holds that clock at.

/// The source state that records when the reader last scrolled.
use crate::native::State;
/// The held clock of the tests.
#[cfg(test)]
use std::cell::Cell;
/// `Duration` and `Instant` time the quiet that scrolling must be followed by.
use std::time::{Duration, Instant};

/// What:
///  How long the view must have stood still before a change of rows may move the scroll offset.
/// Why:
///  The toolkit animates a wheel notch for 180 ms (`WHEEL_SCROLL_DURATION` in Slint 1.18.1's
///      `internal/core/items/flickable.rs`) through a binding on the offset,
///  and assigning the offset removes
///      that binding:
///  the scroll would stop short.
///  Waiting slightly longer than the animation lets it finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SCROLL_QUIET = 200; // milliseconds
/// ```
const SCROLL_QUIET: Duration = Duration::from_millis(200);

/// What:
///  The moment the scrolling rule reads as now:
///  the system clock,
///  or in tests the instant they hold it at.
/// Why:
///  The rule asks whether 200 ms have passed since the last scroll step;
///  a test that sleeps through that time can be overtaken by a loaded host,
///  so a test holds this clock still and moves it by exactly the time it means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function quietNow(): number { return heldClock ?? performance.now(); }
/// ```
pub(in crate::native) fn quiet_now() -> Instant {
    #[cfg(test)]
    if let Some(held) = HELD_CLOCK.with(Cell::get) {
        return held;
    }
    return Instant::now();
}

// The instant a test holds the scrolling rule's clock at, if it does; each test thread has its own.
#[cfg(test)]
thread_local! {
    static HELD_CLOCK: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// Hold the scrolling rule's clock at `at`,
/// or let it run again with `None`.
#[cfg(test)]
pub(in crate::native) fn hold_clock(at: Option<Instant>) {
    HELD_CLOCK.set(at);
}

/// Whether the reader scrolled so recently that a wheel animation may still be running.
pub(super) fn scrolling(current: &State) -> bool {
    // False when the view never scrolled; otherwise whether the last scroll is still recent.
    return match current.scrolled_at {
        Some(at) => quiet_now().saturating_duration_since(at) < SCROLL_QUIET,
        None => false,
    };
}
