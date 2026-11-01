//! Scroll anchoring for Search results that are re-ranked while the user looks at them (D111).
//!
//! What: `anchored_offset` computes the scroll offset that keeps the first visible row where it is on screen after the
//! result list is replaced by a refresh, and `result_key` names a result so it can be found in the new list.
//! Why: while a scan adds tracks the ranking changes under the user's eyes. A tap lands on whatever row is under
//! the finger, so the row the user is about to tap must not move. Rows above the viewport can change freely because
//! they are out of sight; rows below it follow the new ranking.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const offset = anchoredOffset(oldKeys, newKeys, scrollTop, rowHeight); // like CSS overflow-anchor
//! ```

use std::collections::HashMap;

use music_player::search_results::{SearchResult, SearchResultKind};

/// What:
///      `pub const SEARCH_ROW_HEIGHT_PX: f32 = 72.0`.
///  The height of one result row in the Search page.
/// Why:
///       Rows have a fixed height in `search.slint`, so a scroll offset maps to a row by division.
pub const SEARCH_ROW_HEIGHT_PX: f32 = 72.0;

/// What: `result_key` names a result so that the same track or folder is recognized in a later list.
/// Why: a track is identified by its position in the library, a folder by its name.
pub fn result_key(result: &SearchResult) -> String {
    return match (result.kind, result.index_in_library) {
        (SearchResultKind::Track, Some(index)) => format!("t{index}"),
        _ => format!("f{}", result.name),
    };
}

/// What: `anchored_offset` returns the scroll distance, in pixels from the top, that keeps the anchor row in place.
/// Why: the anchor is the first old row at or after the first visible one that the new list still holds; it keeps its
/// distance from the top of the viewport. When no visible row survives, the old offset is kept, limited to the new list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anchoredOffset(oldKeys: string[], newKeys: string[], offset: number, rowHeight: number): number { ... }
/// ```
pub fn anchored_offset(old_keys: &[String], new_keys: &[String], offset_px: f32, row_height_px: f32) -> f32 {
    let offset: f32 = offset_px.max(0.0);
    let limit: f32 = (new_keys.len() as f32 * row_height_px).max(0.0);
    if old_keys.is_empty() || new_keys.is_empty() {
        return offset.min(limit);
    }
    let first_visible: usize = ((offset / row_height_px).floor() as usize).min(old_keys.len() - 1);
    let new_positions: HashMap<&str, usize> = new_keys
        .iter()
        .enumerate()
        .map(|(position, key)| return (key.as_str(), position))
        .collect();
    let visible_rows: usize = 12;
    for (old_position, old_key) in old_keys.iter().enumerate().skip(first_visible).take(visible_rows) {
        if let Some(new_position) = new_positions.get(old_key.as_str()) {
            let old_screen_top: f32 = old_position as f32 * row_height_px - offset;
            let anchored: f32 = *new_position as f32 * row_height_px - old_screen_top;
            return anchored.clamp(0.0, limit);
        }
    }
    return offset.min(limit);
}

/// Unit tests in `ui_search_anchor_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "ui_search_anchor_tests.rs"]
mod tests;
