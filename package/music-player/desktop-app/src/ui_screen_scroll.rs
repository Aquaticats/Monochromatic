//! Scroll requests for the track list: which projection scrolls the list, and to which row.
//!
//! What: `decide_scroll_request` says which request, if any, one projection gets, and `scroll_offset_for` turns a
//! request into the y offset the list scrolls to.
//! Why: the list scrolls only when asked. A change of the playing track alone never scrolls, so a click on a row
//! leaves the scroll position where it is.

use crate::ui_screen_rows::{ListRow, current_row_position};
use crate::ui_screen_view::{ScreenState, ScreenView};

/// Height of a track row without a second line, in logical pixels. Mirrors `row-min-height` in `ui/tracks.slint`.
const ROW_MIN_HEIGHT_PX: f32 = 48.0;
/// Height of a track row with a second line, in logical pixels. Mirrors `row-two-line-height` in `ui/tracks.slint`.
const ROW_TWO_LINE_HEIGHT_PX: f32 = 72.0;
/// Height of a subfolder header, in logical pixels. Mirrors `header-height` in `ui/tracks.slint`.
const HEADER_HEIGHT_PX: f32 = 54.0;

/// What: `ScrollRequest` names why the track list scrolls on one projection.
/// Why: each reason brings a different row to the top, and a projection without a reason leaves the list alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScrollRequest = "top" | "playingRow" | "playingFolder";
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollRequest {
    /// Scroll to the start of the folder.
    Top,
    /// Scroll so the playing row is at the top.
    PlayingRow,
    /// Scroll so the subfolder header above the playing row is at the top, or the start of the folder when the
    /// playing track sits directly in it.
    PlayingFolder,
}

/// What: `ScrollInput` holds what one projection knows when it decides whether the list scrolls.
/// Why: the decision reads only these values, so the page and playing-track rules can be tested without a window.
pub struct ScrollInput<'a> {
    /// The explicit request set since the last projection, or None.
    pub pending: Option<ScrollRequest>,
    /// Identity of the page shown by the last projection that had rows, or None before any such projection.
    pub previous_page_key: Option<&'a str>,
    /// Identity of the page this projection shows.
    pub page_key: &'a str,
    /// Whether this projection has no rows to show.
    pub rows_empty: bool,
    /// Whether one of the rows shown is the playing track.
    pub playing_row_shown: bool,
    /// Whether the startup reveal has happened or the user has already acted.
    pub startup_done: bool,
}

/// What: `row_height_px` gives the height of one list row, from the same rules as the track list in `ui/tracks.slint`.
/// Why: the scroll offset of a row is the sum of the heights above it, and the list does not report its layout back.
pub fn row_height_px(row: &ListRow) -> f32 {
    if row.kind == 1 {
        return HEADER_HEIGHT_PX;
    }
    return if row.supporting.is_empty() {
        ROW_MIN_HEIGHT_PX
    } else {
        ROW_TWO_LINE_HEIGHT_PX
    };
}

/// What: `row_offset_px` gives the distance from the top of the list to the top of the row at `index`.
/// Why: the list scrolls by offset, and only the rows above `index` decide where that row starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rowOffsetPx = (rows: ListRow[], index: number): number =>
///   rows.slice(0, index).reduce((sum, row) => sum + rowHeightPx(row), 0);
/// ```
pub fn row_offset_px(rows: &[ListRow], index: usize) -> f32 {
    return rows.iter().take(index).map(row_height_px).sum();
}

/// What: `row_tops_px` gives the distance from the top of the list to the top of every row, and one more entry for the end
/// of the last row, which is the height of the whole content.
/// Why: the pane builds only the rows on screen, so it asks for the row at a scroll offset and for where a row starts; one
/// pass over the rows answers both with a binary search however long the list is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rowTopsPx = (rows: ListRow[]): number[] => {
///   const tops = [0];
///   for (const row of rows) tops.push(tops[tops.length - 1] + rowHeightPx(row));
///   return tops;
/// };
/// ```
pub fn row_tops_px(rows: &[ListRow]) -> Vec<f32> {
    let mut tops = Vec::with_capacity(rows.len() + 1);
    let mut top = 0.0_f32;
    tops.push(top);
    for row in rows {
        top += row_height_px(row);
        tops.push(top);
    }
    return tops;
}

/// What: `row_at_px` gives the index of the row that contains `offset` pixels from the top of the list, clamped to the
/// first and last row, or 0 for an empty list.
/// Why: the pane's first built row is the one at the top of its viewport; an offset above the list (a bounce) is the first
/// row and one below the end is the last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rowAtPx = (tops: number[], offset: number): number =>
///   Math.max(0, Math.min(tops.length - 2, upperBound(tops, offset) - 1));
/// ```
pub fn row_at_px(tops: &[f32], offset: f32) -> usize {
    let rows = tops.len().saturating_sub(1);
    if rows == 0 {
        return 0;
    }
    let after = tops.partition_point(|top| return *top <= offset);
    return after.saturating_sub(1).min(rows - 1);
}

/// What: `top_of_px` gives where the row at `index` starts, or the end of the content for an index past the last row.
/// Why: the pane places its first built row at this distance from the top of the content.
pub fn top_of_px(tops: &[f32], index: usize) -> f32 {
    return tops.get(index).or_else(|| return tops.last()).copied().unwrap_or(0.0);
}

/// What: `playing_position` gives the list index of the playing row, or None when no row on this page plays.
/// Why: `current_row_position` reports -1 for no row, and the scroll code needs an index it can use or skip.
fn playing_position(rows: &[ListRow]) -> Option<usize> {
    return usize::try_from(current_row_position(rows)).ok();
}

/// What: `folder_position` gives the list index of the subfolder header above the playing row, or 0 when the playing
/// track sits directly in the folder or nothing plays.
/// Why: a track directly in the folder has no header above it, and the folder then begins at the top of the list.
fn folder_position(rows: &[ListRow]) -> usize {
    let Some(playing) = playing_position(rows) else {
        return 0;
    };
    return rows
        .iter()
        .take(playing + 1)
        .rposition(|row| return row.kind == 1)
        .unwrap_or(0);
}

/// What: `scroll_offset_for` gives the y offset the list scrolls to for one request.
/// Why: Top is the start of the list, and the other requests bring their row to the top; a request whose row is
/// missing scrolls to the start of the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scrollOffsetFor = (request: ScrollRequest, rows: ListRow[]): number => { ... };
/// ```
pub fn scroll_offset_for(request: ScrollRequest, rows: &[ListRow]) -> f32 {
    let target = match request {
        ScrollRequest::Top => 0,
        ScrollRequest::PlayingRow => playing_position(rows).unwrap_or(0),
        ScrollRequest::PlayingFolder => folder_position(rows),
    };
    return row_offset_px(rows, target);
}

/// What: `decide_scroll_request` picks the request for one projection.
/// Why: an explicit request always wins. Otherwise the first projection that shows the restored playing row reveals
/// it (the restore reports the track after the rows, so the reveal waits for a playing row), the first projection that
/// has rows otherwise starts at the top, a
/// projection on a different page starts at the top, and every other projection leaves the scroll position alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decideScrollRequest = (input: ScrollInput): ScrollRequest | null => { ... };
/// ```
pub fn decide_scroll_request(input: ScrollInput<'_>) -> Option<ScrollRequest> {
    if let Some(request) = input.pending {
        return Some(request);
    }
    if input.rows_empty {
        return None;
    }
    if !input.startup_done && input.playing_row_shown {
        return Some(ScrollRequest::PlayingRow);
    }
    return match input.previous_page_key {
        None => Some(ScrollRequest::PlayingRow),
        Some(previous) if previous != input.page_key => Some(ScrollRequest::Top),
        Some(_) => None,
    };
}

/// What: `next_scroll_request` consumes the pending request, decides this projection's request, and records the page.
/// Why: the pending request is used by exactly one projection, and the page is recorded only once rows are shown, so
/// a list that is still empty at startup does not count as an earlier page.
pub fn next_scroll_request(state: &mut ScreenState, screen: &ScreenView) -> Option<ScrollRequest> {
    let playing_row_shown = screen.rows.iter().any(|row| return row.kind == 0 && row.current);
    let request = decide_scroll_request(ScrollInput {
        pending: state.scroll_request.take(),
        previous_page_key: state.last_page_key.as_deref(),
        page_key: screen.page_key.as_str(),
        rows_empty: screen.rows.is_empty(),
        playing_row_shown,
        startup_done: state.startup_reveal_done,
    });
    if request == Some(ScrollRequest::PlayingRow) && playing_row_shown {
        state.startup_reveal_done = true;
    }
    if !screen.rows.is_empty() {
        state.last_page_key = Some(screen.page_key.clone());
    }
    return request;
}

#[cfg(test)]
#[path = "ui_screen_scroll_tests.rs"]
mod tests;
