//! Applying Search answers to the window, in place.
//!
//! What: `apply_answer` takes one answer from the Search service on the window thread, drops it when a newer query
//! exists, and updates the one rows model so only rows that changed are rewritten. A refresh of the shown query keeps
//! the first visible row where it is (`ui_search_anchor`); a new query shows from the top.
//! Why: D111 requires results to update in place while a scan adds tracks: the query, the caret, the scroll position
//! and the row about to be tapped must not change because the library grew.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function applyAnswer(win: AppWindow, a: Answer) { if (a.sequence !== latest) return; syncRows(model, view.rows); anchor(a.refresh); }
//! ```

use std::cell::RefCell;
use std::rc::Rc;

use music_player::search_results::SearchResult;
use music_player::search_service::SearchAnswer;
use slint::{Model, ModelRc, VecModel};

use crate::ui_screen::{model_of, read_state, update_state};
use crate::ui_search_anchor::{SEARCH_ROW_HEIGHT_PX, anchored_offset, result_key};
use crate::ui_search_model::{SearchRowView, SearchView, search_view_of};
use crate::{AppWindow, SearchRow, SearchSegment};

/// What:
///      `pub struct SearchUi`.
///  What the window thread remembers about the Search page between answers.
/// Why:
///       Staleness, tap resolution and anchoring all compare against the last applied answer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SearchUi {
    /// Sequence number of the newest query sent to the service; older answers are dropped.
    pub sequence: u64,
    /// Results of the last applied answer; a tap resolves its position in this list.
    pub results: Vec<SearchResult>,
    /// Rows of the last applied answer, to find which rows changed.
    pub rows: Vec<SearchRowView>,
    /// The query the shown rows answer, or `None` before any answer.
    pub shown_query: Option<String>,
}

thread_local! {
    /// The one rows model of the Search page; answers edit it instead of replacing it.
    static ROWS: RefCell<Option<Rc<VecModel<SearchRow>>>> = const { RefCell::new(None) };
}

/// What: `install_rows_model` creates the rows model and hands it to the window.
/// Why: replacing the model on every answer would rebuild every row and reset the list; one model edited in place does not.
pub(crate) fn install_rows_model(window: &AppWindow) {
    let model: Rc<VecModel<SearchRow>> = Rc::new(VecModel::default());
    window.set_search_rows(ModelRc::from(Rc::clone(&model)));
    ROWS.with(|slot| *slot.borrow_mut() = Some(model));
}

/// What: `search_segment` converts a text piece to the window struct.
/// Why: the window draws emphasized pieces on a tint.
fn search_segment(piece: &crate::ui_search_model::Segment) -> SearchSegment {
    return SearchSegment {
        text: piece.text.as_str().into(),
        emphasized: piece.emphasized,
    };
}

/// What: `search_row` converts one row view to the window struct.
/// Why: the pure model has no Slint types.
fn search_row(row: &SearchRowView) -> SearchRow {
    return SearchRow {
        kind: row.kind,
        title_segments: model_of(row.title.iter().map(search_segment).collect()),
        parent_segments: model_of(row.parent.iter().map(search_segment).collect()),
        has_parent: row.has_parent,
        current: row.current,
        playing: row.playing,
        spoken: row.spoken.as_str().into(),
        index: row.index,
    };
}

/// What: `sync_rows` makes the model equal to `new_rows`, rewriting only rows that differ.
/// Why: an untouched row keeps its widgets, so the list does not flicker or jump on a refresh.
fn sync_rows(old_rows: &[SearchRowView], new_rows: &[SearchRowView]) {
    ROWS.with(|slot| {
        let Some(model) = slot.borrow().clone() else {
            tracing::warn!("the Search rows model was not installed");
            return;
        };
        for (position, row) in new_rows.iter().enumerate() {
            if position < model.row_count() {
                if old_rows.get(position) != Some(row) {
                    model.set_row_data(position, search_row(row));
                }
            } else {
                model.push(search_row(row));
            }
        }
        while model.row_count() > new_rows.len() {
            model.remove(model.row_count() - 1);
        }
    });
}

/// What: `show_view` writes the status block and the rows of a view and remembers them.
/// Why: both the immediate empty-query state and service answers end here.
pub(crate) fn show_view(window: &AppWindow, view: SearchView, query: &str, anchor: bool) {
    let old_rows: Vec<SearchRowView> = read_state(|state| return state.search_ui.rows.clone());
    let old_keys: Vec<String> = read_state(|state| return state.search_ui.results.iter().map(result_key).collect());
    let new_keys: Vec<String> = view.results.iter().map(result_key).collect();
    let offset_px: f32 = -window.get_search_scroll_y();
    sync_rows(&old_rows, &view.rows);
    window.set_search_status_title(view.status_title.as_str().into());
    window.set_search_status_detail(view.status_detail.as_str().into());
    let target: f32 = if anchor {
        anchored_offset(&old_keys, &new_keys, offset_px, SEARCH_ROW_HEIGHT_PX)
    } else {
        0.0
    };
    if anchor && (target - offset_px).abs() > f32::EPSILON {
        window.set_search_anchor_offset(target);
        window.set_search_anchor_tick(window.get_search_anchor_tick() + 1);
    }
    update_state(|state| {
        state.search_ui.results = view.results;
        state.search_ui.rows = view.rows;
        state.search_ui.shown_query = Some(query.to_string());
        // The highlighted result is the first one for a new query, stays where it is when the list is refreshed in place
        // (clamped to the list), and is none when there is no result.
        let count = state.search_ui.results.len();
        state.search_ring = match (count, state.search_ring) {
            (0, _) => None,
            (_, Some(ring)) if anchor => Some(ring.min(count - 1)),
            _ => Some(0),
        };
    });
    window.set_search_ring(read_state(|state| return state.search_ring).and_then(|row| return i32::try_from(row).ok()).unwrap_or(-1));
}

/// What: `apply_answer` applies one service answer on the window thread.
/// Why: see the module description. An answer to an older query is dropped, so a slow ranking never overwrites a newer one.
pub(crate) fn apply_answer(window: &AppWindow, answer: SearchAnswer) {
    let (sequence, current_index, playing) = read_state(|state| return (state.search_ui.sequence, state.current_index, state.playing));
    if answer.sequence != sequence {
        tracing::debug!(answer = answer.sequence, newest = sequence, "dropped an answer to an older query");
        return;
    }
    let apply_started: std::time::Instant = std::time::Instant::now();
    let view: SearchView = search_view_of(&answer.state, current_index, playing, answer.complete);
    let same_query: bool = read_state(|state| return state.search_ui.shown_query.as_deref() == Some(answer.query.as_str()));
    let in_place: bool = answer.refresh && same_query;
    tracing::debug!(
        sequence = answer.sequence,
        refresh = answer.refresh,
        in_place,
        known = answer.known_tracks,
        complete = answer.complete,
        rows = view.rows.len(),
        "applying a search answer"
    );
    show_view(window, view, &answer.query, in_place);
    tracing::debug!(sequence = answer.sequence, apply_ms = apply_started.elapsed().as_millis(), "applied a search answer");
}
