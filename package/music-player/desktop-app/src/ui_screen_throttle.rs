//! Coalesced projection for updates that arrive in bursts.
//!
//! What: `project_soon` projects the library screen at once when the last projection was long enough ago, and
//! otherwise schedules one projection for the end of the interval, however many calls arrive in between.
//! Why: the analysis sweep reports its measures in chunks of 128 tracks; at one million tracks that is thousands of
//! reports, and a projection per report would keep the event loop busy for the whole sweep. A few projections per second
//! show the same rows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const projectSoon = throttle(project, 250);
//! ```

use std::cell::Cell;
use std::time::{Duration, Instant};

use slint::{ComponentHandle, Timer, TimerMode};

use crate::AppWindow;
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
    static DELAYED: Timer = Timer::default();
}

/// What: `project_now` projects and records the time.
/// Why: both the immediate and the delayed path must count as the last projection.
fn project_now(app: &AppWindow) {
    LAST_PROJECTION.with(|last| last.set(Some(Instant::now())));
    project(app);
}

/// What: `project_soon` projects now or at the end of the interval.
/// Why: see the module description.
pub(crate) fn project_soon(app: &AppWindow) {
    let now: Instant = Instant::now();
    let due: Option<Instant> = LAST_PROJECTION.with(|last| return last.get().map(|at| return at + PROJECT_INTERVAL));
    match due {
        Some(at) if at > now => {
            if PENDING.with(|pending| return pending.replace(true)) {
                return;
            }
            tracing::debug!(wait_ms = (at - now).as_millis(), "projection delayed to the end of the interval");
            let weak = app.as_weak();
            DELAYED.with(|timer| {
                timer.start(TimerMode::SingleShot, at - now, move || {
                    PENDING.with(|pending| pending.set(false));
                    if let Some(window) = weak.upgrade() {
                        project_now(&window);
                    }
                });
            });
        }
        _ => project_now(app),
    }
}
