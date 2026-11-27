//! Scroll re-centering of the source tile on the tile worker,
//!  and the window-thread step that swaps a finished tile in.
//!
//! The parts:
//!  `applied` decides whether a finished tile still describes the window,
//!  `pass` measures the displayed rows,
//!  and `wait` holds the scroll step before a finished tile is taken.

/// The tile worker and its result type.
use super::tile_build::{
    TileJob,
    TileWorker,
};
/// How much of the pass's output one poll takes.
use super::tile_pass::TAKE_LIMIT;
/// Parent state,
/// the window,
/// and the viewport's hold rule.
use super::{
    AppWindow,
    State,
};
/// The part of a tall tile's lines its raster holds.
use ide_app::shaped_text::Clip;
/// Timer and toolkit-string bridges for the poll and the error message.
use slint::{
    ComponentHandle,
    SharedString,
    TimerMode,
};
/// Poll period of the worker's reply slot,
/// the same as the other background pollers.
use std::{
    cell::RefCell,
    rc::Rc,
    time::{
        Duration,
        Instant,
    },
};

/// What the measurements are valid for,
///  posting the pass,
///  and applying what it measured.
mod pass;

/// The wait for the next scroll step before a finished tile is taken,
///  and the wake-up for its end.
pub(super) mod wait;

/// Whether a finished tile still describes the window,
///  and taking it in.
mod applied;

/// A finished tile taken in,
///  or counted as discarded.
pub(super) use applied::apply;
/// Applying what a pass measured,
///  from the poll.
use pass::apply_pass;
/// The measurement key the native tests ask for,
///  and the pass request that the render also uses.
#[cfg(test)]
pub(super) use pass::current_key;
/// The pass request that the render also uses.
pub(super) use pass::sync_pass;
/// Both timers of the tile poll run this.
use wait::poll_unless_step_due;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TileBounds {
    /// First materialized source line.
    pub(super) first: usize,
    /// Number of materialized source lines.
    pub(super) count: usize,
    /// Wrap width in logical pixels,
    /// or zero while the view has no width yet.
    pub(super) width: f32,
    /// The rows of those lines the raster holds,
    /// below the top of the first line's block;
    /// nothing for all of them.
    pub(super) clip: Option<Clip>,
}

/// The tile the reader's view needs next,
/// as last placed.
pub(super) fn requested(current: &State) -> TileBounds {
    return TileBounds {
        first: current.first,
        count: current.count,
        width: current.width,
        clip: current.clip,
    };
}

/// Forget any tile still wanted:
/// its result,
/// when it arrives,
/// is discarded.
pub(super) fn cancel(current: &mut State) {
    // What: `tile_job` is `None` when no worker result is wanted; `take`-like resets are plain assignments.
    // Why: The result of a cancelled job is dropped by `accepts` instead of being shown.
    current.tile_job = None;
    current.scroll_wanted = None;
    if let Some(worker) = current.tile_worker.as_mut() {
        worker.cancel();
    }
}

/// Queue `job` on the worker,
/// starting the worker and its poll timer on first use.
pub(super) fn submit(
    window: &AppWindow,
    state: &Rc<RefCell<State>>,
    current: &mut State,
    job: TileJob,
) -> anyhow::Result<()> {
    let bounds = job.bounds;
    warm(window, state, current)?;
    // What: `take` moves the worker out so `request` can run without a second borrow of the state.
    // Why: The worker is put back on every path, including a failed request.
    let Some(mut worker) = current.tile_worker.take() else {
        anyhow::bail!(
            "The source tile worker was not started; restart the application to scroll the source"
        );
    };
    let sent = worker.request(job);
    current.tile_worker = Some(worker);
    sent?;
    current.tile_job = Some(bounds);
    start_poll(window, state, current);
    return Ok(());
}

/// Start the tile worker with a shaper forked from the window's own,
/// if none is running.
/// What:
/// The fork shares the font registry the window already scanned,
/// so no font discovery runs on the worker.
/// Why:
/// Starting the worker when the first tile is shown keeps its start-up off the scroll path.
pub(super) fn warm(
    window: &AppWindow,
    state: &Rc<RefCell<State>>,
    current: &mut State,
) -> anyhow::Result<()> {
    if current.tile_worker.is_none() {
        let worker = TileWorker::new(current.shaper.fork())?;
        current.tile_worker = Some(worker);
        start_poll(window, state, current);
    }
    return Ok(());
}

/// Start the repeated poll of the worker's replies unless it is already running.
fn start_poll(
    window: &AppWindow,
    state: &Rc<RefCell<State>>,
    current: &mut State,
) {
    if current.tile_poll.running() {
        return;
    }
    let weak_window = window.as_weak();
    let weak_state = Rc::downgrade(state);
    // The poll holds weak handles only, so the timer never keeps the window or its state alive.
    current
        .tile_poll
        .start(TimerMode::Repeated, Duration::from_millis(20), move || {
            let (Some(active), Some(shared)) = (weak_window.upgrade(), weak_state.upgrade()) else {
                return;
            };
            poll_unless_step_due(&active, &shared);
        });
}

pub(super) fn poll(
    window: &AppWindow,
    state: &Rc<RefCell<State>>,
) {
    let (polled, pass) = {
        let mut current = state.borrow_mut();
        // This poll takes whatever has finished, so a wake-up still waiting for the end of the wait has no work left.
        current.tile_wake.stop();
        let Some(mut worker) = current.tile_worker.take() else {
            current.tile_poll.stop();
            return;
        };
        let polled = worker.poll();
        let pass = if polled.is_ok() {
            worker.take_pass(TAKE_LIMIT)
        } else {
            None
        };
        match &polled {
            Ok(_) => current.tile_worker = Some(worker),
            // A stopped worker is dropped here, so its thread is joined; the next re-center starts a fresh one.
            Err(_) => {
                current.tile_job = None;
                current.scroll_wanted = None;
                current.pass_posted = None;
            }
        }
        stop_when_idle(&mut current);
        (polled, pass)
    };
    // The tile comes first: it may change which lines the displayed tile holds, and so which lines the output may set.
    // Each apply is timed: it is the window thread's whole cost of taking in the worker's result.
    match polled {
        Ok(Some(reply)) => {
            let started = Instant::now();
            apply(window, state, reply);
            tracing::debug!(
                elapsed_us = started.elapsed().as_micros() as u64,
                "window thread applied a finished tile"
            );
        }
        Ok(None) => {}
        Err(error) => {
            tracing::error!(%error, "source tile worker stopped");
            window.set_error_message(SharedString::from(format!("{error:#}")));
        }
    }
    if let Some(output) = pass {
        let started = Instant::now();
        let lines = output.lines.len();
        apply_pass(window, state, output);
        tracing::debug!(
            elapsed_us = started.elapsed().as_micros() as u64,
            lines,
            "window thread applied the output of the measurement pass"
        );
    }
}

/// Stop the poll timer when no re-center is queued and the worker has nothing in flight;
/// `submit` starts it again.
/// What:
/// `Timer::stop` ends the repeated callback;
/// the timer is restarted on the next submission.
/// Why:
/// A cancelled job's reply is still unread while the worker is busy,
/// so the timer must keep polling until it is taken.
fn stop_when_idle(current: &mut State) {
    let in_flight = current
        .tile_worker
        .as_ref()
        .is_some_and(TileWorker::is_busy);
    if current.tile_job.is_none() && !in_flight {
        current.tile_poll.stop();
    }
}
