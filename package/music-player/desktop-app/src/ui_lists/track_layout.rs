//! The published row layout of the track list:
//!  where every row of the whole list starts,
//!  handed to the window so its pane builds only the rows on screen.
//!
//! What:
//! `publish_track_layout` computes the row tops once per projection,
//!  and `row_at_offset` and `top_of_row` answer the window's two layout questions from them.
//! Why:
//! the list can hold a whole library's worth of rows while the pane builds a screenful,
//!  so the layout crosses as plain numbers instead of a window model,
//!  and the probes cost no copy of the list.

/// What:
///      `use std::cell::RefCell;`.
///  The single-threaded mutable cell that lets the layout callbacks answer from memory.
/// Why:
///       The tops belong to the window thread,
///  so a lock would add only a failure path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tops = { value: [] as number[] };
/// ```
use std::cell::RefCell;

/// What:
///      `use slint::ComponentHandle;`.
///  The trait that gives a window handle its generated `global::<T>()` accessor.
/// Why:
///       `publish_track_layout` reaches the window's `TrackLayout` global through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent: the global is part of the generated component
/// ```
use slint::ComponentHandle;

/// Imports the window whose layout global the row tops are published to.
use crate::AppWindow;
/// Imports the generated track layout global that the callbacks are installed on.
use crate::TrackLayout;
/// Imports the row type whose heights give the row tops.
use crate::ui_screen_rows::ListRow;
/// Imports the geometry calculations of the track list.
use crate::ui_screen_scroll::{
    row_at_px,
    row_tops_px,
    top_of_px,
};
thread_local! {
    /// What:
    /// the top of every row of the track list the window shows,
    /// and one more entry for the end of the content.
    /// Why:
    /// the pane builds only the rows on screen,
    /// and asks for the row at a scroll offset through a pure callback that
    /// must answer from plain memory;
    /// the window's own model would cost a copy of the row per probe.
    static TRACK_TOPS: &'static RefCell<Vec<f32>> = Box::leak(Box::new(RefCell::new(Vec::new())));
}

/// What:
/// `track_tops` returns this thread's row tops cell.
/// Why:
/// the window's pure layout callbacks read it for every probe,
/// so they borrow it directly:
/// a copy of the whole vector per probe would cost the length of the library.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trackTops = (): Cell<number[]> => TRACK_TOPS;
/// ```
fn track_tops() -> &'static RefCell<Vec<f32>> {
    return TRACK_TOPS.with(Clone::clone);
}

/// What:
/// `row_at_offset` answers the layout's question which row sits at a scroll offset.
/// Why:
/// the pane builds only the rows on screen,
/// and asks through this callback that answers from the published row tops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rowAtOffset = (offset: number, _generation: number): number => rowAtPx(trackTops, offset);
/// ```
fn row_at_offset(
    offset: f32,
    _generation: i32,
) -> i32 {
    return i32::try_from(row_at_px(&track_tops().borrow(), offset)).unwrap_or(i32::MAX);
}

/// What:
/// `top_of_row` answers the layout's question where a row starts.
/// Why:
/// the pane places each row it builds from the published row tops.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const topOfRow = (index: number, _generation: number): number => topOfPx(trackTops, index);
/// ```
fn top_of_row(
    index: i32,
    _generation: i32,
) -> f32 {
    return top_of_px(&track_tops().borrow(), usize::try_from(index).unwrap_or(0));
}

/// What:
/// `publish_track_layout` gives the window the layout of the whole track list:
/// where every row starts and how tall
/// the content is.
/// Why:
/// the pane builds only the rows that are on screen,
/// so it needs the layout of all the rows to place them,
/// to size the
/// scroll range and its scrollbar,
/// and to find the first row at a scroll offset.
/// The callbacks are installed on the first
/// call,
/// and each call bumps the generation so a pane asks again after the rows changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function publishTrackLayout(win: AppWindow, rows: ListRow[]): void {
///   tops = rowTopsPx(rows);
///   win.trackLayout.totalHeight = tops.at(-1) ?? 0;
///   win.trackLayout.generation += 1;
/// }
/// ```
pub(crate) fn publish_track_layout(
    window: &AppWindow,
    rows: &[ListRow],
) {
    let tops = row_tops_px(rows);
    let total = tops.last().copied().unwrap_or(0.0);
    *track_tops().borrow_mut() = tops;
    let layout = window.global::<TrackLayout>();
    if !layout.get_wired() {
        layout.on_row_at(row_at_offset);
        layout.on_top_of(top_of_row);
        layout.set_wired(true);
        tracing::debug!("the track layout callbacks are installed");
    }
    layout.set_total_height(total);
    layout.set_generation(layout.get_generation().wrapping_add(1));
    tracing::debug!(
        rows = rows.len(),
        total_height = total,
        "the track layout was published"
    );
}
