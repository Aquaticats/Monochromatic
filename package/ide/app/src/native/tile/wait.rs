//! The wait for the next scroll step before a finished tile is taken,
//!  and the single wake-up for the end of that wait.

/// The tile poll that takes a finished tile.
use super::poll;
/// Parent state and the window.
use crate::native::{AppWindow, State};
/// Weak handles of the window and a single-shot timer.
use slint::{ComponentHandle, TimerMode};
/// The shared window state and the wait.
use std::{cell::RefCell, rc::Rc, time::Duration};

/// What:
///  Take the finished tile unless a scroll step is due,
///  and when one is due,
///  look again as soon as the wait for it ends.
/// Why:
///  The repeated poll comes a whole period after the one that left the tile,
///  so a tile left a little before the wait ends would otherwise wait most of a period more
///  (more than one frame at 60 Hz);
///  the single wake-up cuts that to the end of the wait.
///  Both timers run this function and `poll` stays the only place a tile is taken,
///  so a wake-up that a step made stale only finds the step due again and waits once more.
pub(super) fn poll_unless_step_due(window: &AppWindow, state: &Rc<RefCell<State>>) {
    let wait_left = step_wait_left(&state.borrow());
    let Some(left) = wait_left else {
        poll(window, state);
        return;
    };
    tracing::debug!(
        left_us = left.as_micros() as u64,
        "leaving a finished tile for the next scroll step"
    );
    wake_when_wait_ends(window, state, left);
}

/// Look at the tile worker again after `left`,
/// once,
/// replacing any wake-up still waiting.
/// What:
/// `start` on the running timer replaces its pending wake-up,
/// so at most one is ever waiting.
/// Why:
/// The handles are weak,
/// so the wake-up never keeps the window or its state alive.
fn wake_when_wait_ends(window: &AppWindow, state: &Rc<RefCell<State>>, left: Duration) {
    let weak_window = window.as_weak();
    let weak_state = Rc::downgrade(state);
    state
        .borrow()
        .tile_wake
        .start(TimerMode::SingleShot, wake_delay(left), move || {
            let (Some(active), Some(shared)) = (weak_window.upgrade(), weak_state.upgrade()) else {
                return;
            };
            poll_unless_step_due(&active, &shared);
        });
}

/// What:
///  The wait that remains,
///  rounded up to a whole millisecond.
/// Why:
///  The toolkit's timer clock counts milliseconds,
///  so a wake-up for less than one fires at once,
///  finds a little of the wait left,
///  and sets itself again,
///  in a spin on the window thread until the clock moves on.
pub(in crate::native) fn wake_delay(left: Duration) -> Duration {
    return Duration::from_millis(left.as_micros().div_ceil(1000) as u64);
}

/// What:
///  How long after a scroll step the poll leaves a finished tile for the next step to take.
/// Why:
///  A scroll draws a frame at every step of the input,
/// and the setters that swap a tile in ask the toolkit for a frame of their own;
/// a tile taken between two steps is a second frame in that step's time.
///  In the nested compositor's recording the text then did not move for one tick,
/// in every scroll measured (`doc/handover/ide-gpu-stages-5-6.md`).
///  A step of an input at 50 Hz or more comes sooner than this wait,
/// and takes the tile in its own frame;
/// the poll,
/// or the wake-up it sets for the end of this wait,
/// takes a tile no step came for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STEP_WAIT_MS = 20;
/// ```
const STEP_WAIT: Duration = Duration::from_millis(20);

/// How much of the wait remains when the view stepped so recently that the next step,
/// and the frame it draws,
/// is due before the poll comes again;
/// nothing once the wait has passed or when the view never stepped.
fn step_wait_left(current: &State) -> Option<Duration> {
    // `checked_sub` is `None` when the last step is older than the wait.
    return match current.stepped_at {
        Some(at) => STEP_WAIT.checked_sub(at.elapsed()),
        None => None,
    };
}
