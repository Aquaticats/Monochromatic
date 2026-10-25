//! Scroll re-centering of the source tile on the tile worker,
//!  and the window-thread step that swaps a finished tile in.

/// Frame paint inputs and the frame presentation the swap hands over.
use super::render::{Frame, Paint, render, show, stamp_for};
/// The vertical mapping's side of the window:
/// measured rows and keeping the view still.
use super::rows;
/// The tile worker and its result type.
use super::tile_build::{TileBuilt, TileJob, TileWorker};
/// How much of the pass's output one poll takes.
use super::tile_pass::TAKE_LIMIT;
/// Parent state,
/// the window,
/// and the viewport's hold rule.
use super::{AppWindow, State, viewport::holds, wheel};
/// The part of a tall tile's lines its raster holds.
use ide_app::shaped_text::Clip;
/// Timer and toolkit-string bridges for the poll and the error message.
use slint::{ComponentHandle, SharedString, TimerMode};
/// Poll period of the worker's reply slot,
/// the same as the other background pollers.
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

/// What the measurements are valid for,
///  posting the pass,
///  and applying what it measured.
mod pass;

/// The wait for the next scroll step before a finished tile is taken,
///  and the wake-up for its end.
pub(super) mod wait;

/// Applying what a pass measured,
///  from the poll.
use pass::apply_pass;
/// The measurement key and the pass request that the render and the tests also use.
pub(super) use pass::{current_key, sync_pass};
/// Both timers of the tile poll run this.
use wait::poll_unless_step_due;

/// What:
///  Position and width of one materialized tile:
///  its first line,
/// its line count,
/// the wrap width its lines are broken at,
///  and the part of those lines its raster holds when they are too tall for one.
/// Why:
///  The requested tile and the displayed tile are both values of this type,
///  so a result can be checked against the window it was built for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TileBounds = { first: number; count: number; width: number; clip?: Clip };
/// ```
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
fn start_poll(window: &AppWindow, state: &Rc<RefCell<State>>, current: &mut State) {
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

/// Whether a finished tile still describes what is displayed,
/// wanted,
/// and the current view.
/// Stale inputs are rejected here,
/// so a result from before an edit,
/// a file change,
/// or a theme change is never shown.
/// The tile's own lines must have the heights the map has for them,
/// which `apply` makes so before asking.
fn accepts(current: &State, paint: &Paint, built: &TileBuilt, height: f32) -> bool {
    return current.tile_job.is_some()
        && built.file == current.file_generation
        && built.active == paint.active
        && built.bounds.width == current.width
        && measured_rows_match(current, built)
        && (holds(current, built.bounds, current.offset, height)
            || wheel::holds_end(current, built.bounds, height))
        && stamp_for(current, built.bounds, paint) == built.stamp;
}

/// Whether every line the tile measured has the row count the map holds for it.
fn measured_rows_match(current: &State, built: &TileBuilt) -> bool {
    for (line, rows) in built.measured.iter() {
        if current.row_map.rows_of(*line) != *rows {
            return false;
        }
    }
    return true;
}

/// Take at most one finished tile from the worker and what the pass measured,
/// and apply them when they are valid.
pub(super) fn poll(window: &AppWindow, state: &Rc<RefCell<State>>) {
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

/// Swap a finished tile in,
/// or count it as discarded.
/// A failed build keeps the displayed tile on screen and names the failure,
/// never falling back to a synchronous build.
pub(super) fn apply(
    window: &AppWindow,
    state: &Rc<RefCell<State>>,
    reply: anyhow::Result<TileBuilt>,
) {
    let built = match reply {
        Ok(built) => built,
        Err(error) => {
            tracing::error!(%error, "source tile build failed");
            // The job is over: the poll may stop, and a placement for the same tile may be queued again.
            let message = format!("Cannot render source: {error:#}");
            {
                let mut current = state.borrow_mut();
                current.tile_job = None;
                #[cfg(debug_assertions)]
                current.worker_failures.push(message.clone());
            }
            window.set_error_message(SharedString::from(message));
            return;
        }
    };
    let factor = window.window().scale_factor();
    let mut current = state.borrow_mut();
    #[cfg(debug_assertions)]
    {
        current.worker_tiles += 1;
        current.worker_rows_shaped += built.rows_shaped;
    }
    // What: The heights the tile's own lines measured go into the table first, and the map follows them.
    // Why: The tile was painted with those heights; the map must agree before the tile can be shown.
    //      A change above the view moves the offset with it, even while the reader scrolls.
    let height = window.get_viewport_height();
    let view_now = (-window.get_scroll_y(), window.get_viewport_width(), height);
    let mut scrolled = None;
    let mut remapped = false;
    if built.key == current_key(&current, factor) {
        remapped = rows::absorb(&mut current, built.measured.iter().copied());
        // The map follows the table also when an earlier change of it was still waiting for the reader to stop.
        scrolled = rows::settle(&mut current, factor, view_now, true);
        remapped = remapped || scrolled.is_some();
    }
    let paint = Paint::new(window, &current);
    // What: `accepts` is a conjunction of cheap identity checks, and the stamp rebuild only runs when the rest pass.
    // Why: A stale tile is common while scrolling fast, and each rejection must cost almost nothing.
    if !accepts(&current, &paint, &built, height) {
        #[cfg(debug_assertions)]
        {
            current.tiles_discarded += 1;
        }
        if current.tile_job == Some(built.requested) {
            current.tile_job = None;
        }
        // A tile is still wanted when the window waits for one: the displayed tile stays on screen until it arrives,
        // so a discarded tile (a hint arrived while it was built, for instance) must be asked for again.
        let wanted = current.scroll_wanted.is_some();
        tracing::debug!(
            first = built.bounds.first,
            wanted,
            remapped,
            "discarded stale source tile"
        );
        drop(current);
        if let Some(offset) = scrolled {
            // What: The offset the map's new heights moved the view to is handed to the window here, with the frame
            //       that is displayed, before the render that follows.
            // Why: The map has taken the heights already, so the render finds nothing left to settle and would read
            //      the old offset from the window: the window and the next tile would then disagree about the view.
            show(
                window,
                state,
                Frame {
                    scene: None,
                    matches_changed: false,
                    scale: factor,
                    scrolled: Some(offset),
                    anchor_line: window.get_language_anchor_line(),
                    notice: None,
                    sign: None,
                },
            );
        }
        if remapped || wanted {
            // The map moved even though the tile was not shown, or a tile is still wanted; the window follows.
            render(window, state);
        }
        return;
    }
    let bounds = built.bounds;
    let scene = built.scene;
    // The map may have moved since the job was queued; the tile's rows move with it, and its own lines did not change.
    let mut view = built.view;
    view.rebase(&current.row_map);
    // What: A tile at a new width asks for the measurement walk again.
    // Why: A measurement never touches the displayed tile, so the lines the old tile held kept the heights of the
    //      old width; the walk finds the worker's kept rows by lookup, so asking again costs little.
    if current.shown.width != bounds.width {
        current.pass_posted = None;
    }
    current.shaped = Some(view);
    current.frame_stamp = Some(built.stamp);
    current.shown = bounds;
    // The requested window now equals what is shown; a newer job still in flight may replace it.
    current.first = bounds.first;
    current.count = bounds.count;
    current.clip = bounds.clip;
    current.scroll_wanted = None;
    if current.tile_job == Some(built.requested) {
        current.tile_job = None;
    }
    #[cfg(debug_assertions)]
    {
        current.tiles_applied += 1;
    }
    sync_pass(window, state, &mut current, factor);
    drop(current);
    show(
        window,
        state,
        Frame {
            scene: Some(scene),
            matches_changed: true,
            scale: factor,
            scrolled,
            anchor_line: window.get_language_anchor_line(),
            notice: None,
            sign: None,
        },
    );
}
