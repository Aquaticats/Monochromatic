//! A finished tile taken in:
//!  the identity checks that reject a stale one,
//!  and the swap that shows it,
//!  or counts it as discarded.

/// Frame paint inputs and the frame presentation the swap hands over.
use super::super::render::{
    Frame,
    Paint,
    render,
    show,
    stamp_for,
};
/// The vertical mapping's side of the window:
/// measured rows and keeping the view still.
use super::super::rows;
/// The tile worker's result type.
use super::super::tile_build::TileBuilt;
/// The window,
///  the parent state,
///  and the viewport's hold rules.
use super::super::{
    AppWindow,
    State,
    viewport::holds,
    wheel,
};
/// The measurement key a tile is built for.
use super::pass::{
    current_key,
    sync_pass,
};
/// The toolkit's component handle,
///  which lends the window, and its string type for an error message.
use slint::{
    ComponentHandle,
    SharedString,
};
/// Shared UI state.
use std::{
    cell::RefCell,
    rc::Rc,
};

/// Whether a finished tile still describes what is displayed,
/// wanted,
/// and the current view.
/// Stale inputs are rejected here,
/// so a result from before an edit,
/// a file change,
/// or a theme change is never shown.
/// The tile's own lines must have the heights the map has for them,
/// which `apply` makes so before asking.
fn accepts(
    current: &State,
    paint: &Paint,
    built: &TileBuilt,
    height: f32,
) -> bool {
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
fn measured_rows_match(
    current: &State,
    built: &TileBuilt,
) -> bool {
    for (line, rows) in built.measured.iter() {
        if current.row_map.rows_of(*line) != *rows {
            return false;
        }
    }
    return true;
}

/// Take at most one finished tile from the worker and what the pass measured,
/// and apply them when they are valid.

/// Swap a finished tile in,
/// or count it as discarded.
/// A failed build keeps the displayed tile on screen and names the failure,
/// never falling back to a synchronous build.
pub(in crate::native) fn apply(
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
