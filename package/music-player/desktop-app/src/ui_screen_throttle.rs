//! Coalesced projection for updates that arrive in bursts.
//!
//! What:
//! `project_soon` projects the library screen at once when the last projection was long enough ago,
//! and
//! otherwise schedules one projection for the end of the interval,
//! however many calls arrive in between.
//! Why:
//! the analysis sweep reports its measures in chunks of 128 tracks;
//! at one million tracks that is thousands of
//! reports,
//! and a projection per report would keep the event loop busy for the whole sweep.
//! A few projections per second
//! show the same rows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const projectSoon = throttle(project, 250);
//! ```

/// Imports the cell that holds the time of the last projection.
use std::cell::Cell;
/// Imports the duration and instant types that space the projections.
use std::time::{Duration, Instant};

/// Imports the component handle,
/// the timer and its mode that run the delayed projection.
use slint::{ComponentHandle, Timer, TimerMode};

/// Imports the window that is projected.
use crate::AppWindow;
/// Imports the projection that the throttle schedules.
use crate::ui_screen::project;

/// What:
///      `pub(crate) const PROJECT_INTERVAL: Duration`.
///  The shortest time between two throttled projections.
/// Why:
///       Four projections a second keep fresh lengths and peaks appearing without starving input.
pub(crate) const PROJECT_INTERVAL: Duration = Duration::from_millis(250);

thread_local! {
    /// When the last throttled projection ran.
    static LAST_PROJECTION: Cell<Option<Instant>> = const { Cell::new(None) };
    /// Whether a delayed projection is already scheduled.
    static PENDING: Cell<bool> = const { Cell::new(false) };
    /// The timer that runs the delayed projection.
    static DELAYED: &'static Timer = Box::leak(Box::new(Timer::default()));
}

/// What:
/// `delayed_timer` returns this thread's delayed-projection timer.
/// Why:
/// the timer is leaked once per thread,
/// so it outlives every borrow,
/// so `project_soon` can start it directly instead of passing a closure to `LocalKey::with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const delayedTimer = (): Timer => DELAYED;
/// ```
fn delayed_timer() -> &'static Timer {
    return DELAYED.with(Clone::clone);
}

/// What:
/// `project_now` projects and records the time.
/// Why:
/// both the immediate and the delayed path must count as the last projection.
fn project_now(app: &AppWindow) {
    LAST_PROJECTION.set(Some(Instant::now()));
    project(app);
}

/// What:
/// `project_soon` projects now or at the end of the interval.
/// Why:
/// see the module description.
pub(crate) fn project_soon(app: &AppWindow) {
    let now: Instant = Instant::now();
    let due: Option<Instant> = match LAST_PROJECTION.get() {
        Some(at) => Some(at + PROJECT_INTERVAL),
        None => None,
    };
    match due {
        Some(at) if at > now => {
            if PENDING.replace(true) {
                return;
            }
            tracing::debug!(wait_ms = (at - now).as_millis(), "projection delayed to the end of the interval");
            let weak = app.as_weak();
            delayed_timer().start(TimerMode::SingleShot, at - now, move || {
                PENDING.set(false);
                if let Some(window) = weak.upgrade() {
                    project_now(&window);
                }
            });
        }
        _ => project_now(app),
    }
}
