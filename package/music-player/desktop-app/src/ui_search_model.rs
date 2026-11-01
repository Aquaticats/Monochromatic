//! Pure Search page model: the rows, emphasis segments and status text for one query.
//!
//! What: `search_view` runs the library search and turns each result into plain rows.
//! Why: row order, emphasis spans and the status claim are decided here, so the window only places text.

use music_player::search_page_model::{
    is_current_track_result, search_emphasis_spans, search_row_kind_label, search_row_second_line, search_status_line,
};
use music_player::search_results::{SearchResult, SearchResultKind, SearchState};
#[cfg(test)]
use music_player::search_results::search_library;
#[cfg(test)]
use music_player::track::Track;

/// What: `Segment` is one piece of a result line, plain or emphasized (a query match).
/// Why: the pane draws emphasized pieces bold on a tint, so each line arrives pre-split.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Segment = { text: string; emphasized: boolean };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// Text of this piece.
    pub text: String,
    /// Whether this piece is a query match.
    pub emphasized: bool,
}

/// What: `SearchRowView` is one Search result row before it becomes a window struct.
/// Why: the window needs plain values, and the tests need the row without Slint.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchRowView = { kind: 0 | 1; title: Segment[]; parent: Segment[]; hasParent: boolean; ... };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRowView {
    /// 0 for a folder result, 1 for a track result.
    pub kind: i32,
    /// Title line split into emphasized pieces.
    pub title: Vec<Segment>,
    /// Parent folder line split into emphasized pieces; empty when `has_parent` is false.
    pub parent: Vec<Segment>,
    /// Whether the second line is shown.
    pub has_parent: bool,
    /// Whether this track is the current track.
    pub current: bool,
    /// Whether the current track is playing.
    pub playing: bool,
    /// What a screen reader announces for the row: the name, the kind and, for a track, the parent folder.
    pub spoken: String,
    /// Position of the result in the result list, which `open` reports back.
    pub index: i32,
}

/// What: `SearchView` is the Search page's rows and status block for one query.
/// Why: the status block shows when there are no rows, so both come from one query run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchView = { rows: SearchRowView[]; statusTitle: string; statusDetail: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchView {
    /// Result rows in result order; empty when the status block is shown.
    pub rows: Vec<SearchRowView>,
    /// Status block title, or empty when results are shown.
    pub status_title: String,
    /// Status block detail, or empty when there is none.
    pub status_detail: String,
    /// The result list the row indices refer to.
    pub results: Vec<SearchResult>,
}

/// What: `segments` splits `text` into plain and emphasized pieces at the highlight spans.
/// Why: the search highlights are character offsets, so the split walks characters, not bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const segments = (text: string, spans: Span[]): Segment[] => { ... };
/// ```
pub fn segments(text: &str, highlights: &[music_player::search_matcher::SearchHighlight]) -> Vec<Segment> {
    let spans = search_emphasis_spans(text, highlights);
    let mut pieces: Vec<Segment> = Vec::new();
    for (position, ch) in text.chars().enumerate() {
        let offset = i32::try_from(position).unwrap_or(i32::MAX);
        let emphasized = spans.iter().any(|span| return offset >= span.start && offset < span.end);
        match pieces.last_mut() {
            Some(last) if last.emphasized == emphasized => last.text.push(ch),
            _ => pieces.push(Segment {
                text: ch.to_string(),
                emphasized,
            }),
        }
    }
    return pieces;
}

/// What: `spoken_label` joins the name, the kind word and the parent folder of a result into one announcement.
/// Why: a result is one named action (D77): its own name once, the kind, and the parent context that tells equal names
/// apart. The highlight pieces and the kind icon add nothing, so the row's label is the only place the words are joined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spokenLabel = (name: string, kind: string, parent?: string) => [name, kind, parent].filter(Boolean).join(", ");
/// ```
fn spoken_label(name: &str, kind: &str, parent: Option<&str>) -> String {
    return match parent {
        Some(parent_text) => format!("{name}, {kind}, {parent_text}"),
        None => format!("{name}, {kind}"),
    };
}

/// What: `row_view` converts one result into a row at `position` in the result list.
/// Why: folder and track results share one row shape, with the track-only fields left empty for folders.
fn row_view(result: &SearchResult, position: usize, current_index: Option<usize>, playing: bool) -> SearchRowView {
    let current = is_current_track_result(result, current_index);
    let parent_text = search_row_second_line(result);
    let parent_segments = match &parent_text {
        Some(text) => segments(text, &result.parent_highlights),
        None => Vec::new(),
    };
    return SearchRowView {
        kind: match result.kind {
            SearchResultKind::Folder => 0,
            SearchResultKind::Track => 1,
        },
        title: segments(&result.name, &result.name_highlights),
        parent: parent_segments,
        has_parent: parent_text.is_some(),
        current,
        playing: current && playing,
        spoken: spoken_label(&result.name, search_row_kind_label(result), parent_text.as_deref()),
        index: i32::try_from(position).unwrap_or(-1),
    };
}

/// What: `SEARCH_SCANNING_TITLE` is the status headline while the library is still being scanned and nothing matched yet.
/// Why: a partial scan cannot establish that no name matches (D69), so the page says it is still looking instead of
/// claiming "no results" or "no music". This copy is new with D111 and is open for the human's review.
pub const SEARCH_SCANNING_TITLE: &str = "Still looking through your library";

/// What: `SEARCH_SCANNING_DETAIL` is the support line under `SEARCH_SCANNING_TITLE`.
/// Why: it tells the user that the list will fill in by itself, so no retry is needed.
pub const SEARCH_SCANNING_DETAIL: &str = "Results appear here as more music is found.";

/// What: `status_for` gives the status block for a state, or none when results are shown.
/// Why: an unfinished scan turns the two "nothing here" verdicts into the scanning copy (D69, D111).
fn status_for(state: &SearchState, complete: bool) -> (String, String) {
    let unfinished_nothing = !complete && matches!(state, SearchState::NoMatch(_) | SearchState::EmptyInventory);
    if unfinished_nothing {
        return (SEARCH_SCANNING_TITLE.to_string(), SEARCH_SCANNING_DETAIL.to_string());
    }
    return match search_status_line(state) {
        Some(line) => (line.title, line.detail.unwrap_or("").to_string()),
        None => (String::new(), String::new()),
    };
}

/// What: `search_view_of` turns a ranked state into the rows or the status block.
/// Why: the index answers on a worker thread; this part runs on the window thread and only formats.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const searchViewOf = (state: SearchState, current?: number, playing?: boolean, complete?: boolean): SearchView => { ... };
/// ```
pub fn search_view_of(state: &SearchState, current_index: Option<usize>, playing: bool, complete: bool) -> SearchView {
    let results: Vec<SearchResult> = match state {
        SearchState::Results { items, .. } => items.clone(),
        _ => Vec::new(),
    };
    let rows: Vec<SearchRowView> = results
        .iter()
        .enumerate()
        .map(|(position, result)| return row_view(result, position, current_index, playing))
        .collect();
    let (status_title, status_detail) = status_for(state, complete);
    return SearchView {
        rows,
        status_title,
        status_detail,
        results,
    };
}

/// What: `search_view` runs the query over a track list and returns the rows or the status block.
/// Why: an empty query, an empty library and a query with no match each show a different status claim. The window
/// uses `search_view_of` on an answer from the shared index; this form builds a throwaway index for tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const searchView = (query: string, tracks: Track[], current?: number, playing?: boolean): SearchView => { ... };
/// ```
#[cfg(test)]
pub fn search_view(query: &str, tracks: &[Track], current_index: Option<usize>, playing: bool) -> SearchView {
    return search_view_of(&search_library(query, tracks, true), current_index, playing, true);
}

#[cfg(test)]
#[path = "ui_search_model_tests.rs"]
mod tests;
