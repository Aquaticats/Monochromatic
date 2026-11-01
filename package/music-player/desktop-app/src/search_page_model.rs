//! Pure decisions for the Search page:
//!  status copy,
//!  spoken row labels,
//!  emphasis ranges,
//!  and the query rules for
//! entering and returning to Search.
//!
//! Ported from `core/SearchPageModel.kt`.
//!  Nothing here touches Slint,
//!  so every rule is unit-tested on the host.
//! The copy strings are the Kotlin strings verbatim,
//!  including the curly quotes around a no-match query.

/// Highlight type,
///  whitespace test,
///  and UTF-16 offset helper from the matcher.
use crate::search_matcher::{SearchHighlight, is_kotlin_whitespace, to_offset};
/// Result rows,
///  their kinds,
///  and the search state the page shows.
use crate::search_results::{SearchResult, SearchResultKind, SearchState};

/// What:
///      `pub const SEARCH_PROMPT_TITLE: &str`.
///  The headline shown before any query is typed.
/// Why:
///       An empty query is an invitation,
///  not a failed lookup,
///  so it gets its own copy (D70).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_PROMPT_TITLE = "Search your music";
/// ```
pub const SEARCH_PROMPT_TITLE: &str = "Search your music";

/// What:
///      `pub const SEARCH_PROMPT_DETAIL: &str`.
///  The supporting line under the unqueried prompt.
/// Why:
///       The prompt pairs its headline with this line so the user knows Search covers the library (D70).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_PROMPT_DETAIL = "Type a name to explore your library.";
/// ```
pub const SEARCH_PROMPT_DETAIL: &str = "Type a name to explore your library.";

/// What:
///      `pub const SEARCH_NO_MATCH_PREFIX: &str`.
///  Opens the completed no-match headline,
///  before the entered query.
/// Why:
///       The headline names the query after this prefix,
///  and the prefix is the only fixed text before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_NO_MATCH_PREFIX = "No results for “";
/// ```
pub const SEARCH_NO_MATCH_PREFIX: &str = "No results for \u{201C}";

/// What:
///      `pub const SEARCH_NO_MATCH_SUFFIX: &str`.
///  Closes the completed no-match headline,
///  after the entered query.
/// Why:
///       Curly quotes match the review text,
///  and the query stays visibly delimited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_NO_MATCH_SUFFIX = "”";
/// ```
pub const SEARCH_NO_MATCH_SUFFIX: &str = "\u{201D}";

/// What:
///      `pub const SEARCH_NO_MATCH_DETAIL: &str`.
///  The support line under a completed no-match headline.
/// Why:
///       D70 keeps this support line with the query named in the headline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_NO_MATCH_DETAIL = "Try another name.";
/// ```
pub const SEARCH_NO_MATCH_DETAIL: &str = "Try another name.";

/// What:
///      `pub const SEARCH_UNAVAILABLE_TITLE: &str`.
///  Headlines a known inability to search the source.
/// Why:
///       D71 rejects "Library unavailable" because no owner can promise a recovery.
///  This wording names only the
///  known state,
///  so it makes no recovery promise and names no cause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_UNAVAILABLE_TITLE = "Search is unavailable";
/// ```
pub const SEARCH_UNAVAILABLE_TITLE: &str = "Search is unavailable";

/// What:
///      `pub const SEARCH_UNAVAILABLE_DETAIL: &str`.
///  The support line under the unavailable headline.
/// Why:
///       It states only what is known,
///  so the still-visible browser and deck are not described as broken (D71).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_UNAVAILABLE_DETAIL = "The current library can't be searched right now.";
/// ```
pub const SEARCH_UNAVAILABLE_DETAIL: &str = "The current library can't be searched right now.";

/// What:
///      `pub const SEARCH_EMPTY_INVENTORY_TITLE: &str`.
///  Headlines a usable source that holds no tracks.
/// Why:
///       A confirmed empty inventory is distinct from a finished no-match (D69),
///  so it is not worded as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_EMPTY_INVENTORY_TITLE = "No music found in your audio library.";
/// ```
pub const SEARCH_EMPTY_INVENTORY_TITLE: &str = "No music found in your audio library.";

/// What:
///      `pub const SEARCH_FIELD_LABEL: &str`.
///  The accessible name of the query field.
/// Why:
///       The field keeps the name "Search music" while the typed query is its separate value (D75).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_FIELD_LABEL = "Search music";
/// ```
pub const SEARCH_FIELD_LABEL: &str = "Search music";

/// What:
///      `pub const SEARCH_BACK_LABEL: &str`.
///  The accessible name of the visible Back control.
/// Why:
///       Back returns to the player (D64),
///  and the spoken name must say so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_BACK_LABEL = "Back to player";
/// ```
pub const SEARCH_BACK_LABEL: &str = "Back to player";

/// What:
///      `pub const SEARCH_CLEAR_LABEL: &str`.
///  The accessible name of the Clear control.
/// Why:
///       Clear erases the query only,
///  so its spoken name says it clears the search (D65).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_CLEAR_LABEL = "Clear search";
/// ```
pub const SEARCH_CLEAR_LABEL: &str = "Clear search";

/// What:
///      `pub const SEARCH_FOLDER_ACTION: &str`.
///  The spoken action of a folder result.
/// Why:
///       A folder result opens its folder without autoplay (D72),
///  and the action label must say that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_FOLDER_ACTION = "Open folder without autoplay";
/// ```
pub const SEARCH_FOLDER_ACTION: &str = "Open folder without autoplay";

/// What:
///      `pub const SEARCH_TRACK_SELECT_ACTION: &str`.
///  The spoken action of a track that is not the current track.
/// Why:
///       A different track is only selected and loaded paused (D127),
///  so the action label is Select track (D77),
///  not Play track,
///  unless Auto-play on select is on (D129),
///  which starts it and brings Play track back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_TRACK_SELECT_ACTION = "Select track";
/// ```
pub const SEARCH_TRACK_SELECT_ACTION: &str = "Select track";

/// What:
///      `pub const SEARCH_TRACK_PLAY_ACTION: &str`.
///  The spoken action of the current track while it is paused.
/// Why:
///       The current track toggles playback (D127),
///  and while it is paused the toggle starts it (D77).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_TRACK_PLAY_ACTION = "Play track";
/// ```
pub const SEARCH_TRACK_PLAY_ACTION: &str = "Play track";

/// What:
///      `pub const SEARCH_TRACK_PAUSE_ACTION: &str`.
///  The spoken action of the current track while it plays.
/// Why:
///       The current track toggles playback,
///  and while it plays the toggle pauses it (D73,
///  D77).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_TRACK_PAUSE_ACTION = "Pause track";
/// ```
pub const SEARCH_TRACK_PAUSE_ACTION: &str = "Pause track";

/// What:
///      `pub const SEARCH_TRACK_TOGGLE_ACTION: &str`.
///  The action when the playing state of the current track is unknown.
/// Why:
///       Claiming Play or Pause without knowing the state would be false,
///  so the label names the toggle only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_TRACK_TOGGLE_ACTION = "Play or pause track";
/// ```
pub const SEARCH_TRACK_TOGGLE_ACTION: &str = "Play or pause track";

/// What:
///      `pub const SEARCH_FOLDER_KIND: &str`.
///  The spoken kind word of a folder result.
/// Why:
///       The kind is spoken once,
///  because the decorative icon alone is not an accessible cue (D77).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_FOLDER_KIND = "Folder";
/// ```
pub const SEARCH_FOLDER_KIND: &str = "Folder";

/// What:
///      `pub const SEARCH_TRACK_KIND: &str`.
///  The spoken kind word of a track result.
/// Why:
///       Like the folder kind,
///  it is spoken once so the icon is not an extra accessibility stop (D77).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_TRACK_KIND = "Track";
/// ```
pub const SEARCH_TRACK_KIND: &str = "Track";

/// What:
///      `pub const SEARCH_RESTORED_RESULT_INDEX: usize = 0`.
///  The result position a restored query starts from.
/// Why:
///       If a restored query ever returns,
///  its results start at the top rather than a deep position (D67).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SEARCH_RESTORED_RESULT_INDEX = 0;
/// ```
pub const SEARCH_RESTORED_RESULT_INDEX: usize = 0;

/// What:
///      `pub struct SearchStatusLine { pub title: String, pub detail: Option<&'static str> }`.
///  A status headline
///  with an optional support line.
/// Why:
///       The page draws the same two-line status anatomy for every non-result state.
///  The title is owned because a
///  no-match title carries the query.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchStatusLine = { title: string; detail: string | null };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchStatusLine {
    /// Headline text of the status.
    pub title: String,
    /// Support text under the headline,
    ///  or `None` when the status needs only a headline.
    pub detail: Option<&'static str>,
}

/// What:
///      `pub struct SearchRowSemantics { ... }`.
///  The spoken kind,
///  action,
///  and optional playback state of one row.
/// Why:
///       The page merges the row into one stop,
///  so kind,
///  state,
///  and action are kept as separate fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchRowSemantics = { kindLabel: string; actionLabel: string; stateLabel: string | null };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchRowSemantics {
    /// Kind word of the result,
    ///  spoken once.
    pub kind_label: &'static str,
    /// Action the row performs when activated.
    pub action_label: &'static str,
    /// Playback state of the row when it is the current track,
    ///  or `None` otherwise.
    pub state_label: Option<&'static str>,
}

/// What:
///      `pub fn search_status_line(state: &SearchState) -> Option<SearchStatusLine>`.
///  Picks the copy for a
///  non-result state.
/// Why:
///       Each state needs its own truthful wording (D69 to D71),
///  and a result list has no status line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function searchStatusLine(state: SearchState): StatusLine | null { /* switch on state.kind */ }
/// ```
pub fn search_status_line(state: &SearchState) -> Option<SearchStatusLine> {
    return match state {
        SearchState::EmptyQuery => Some(SearchStatusLine {
            title: SEARCH_PROMPT_TITLE.to_string(),
            detail: Some(SEARCH_PROMPT_DETAIL),
        }),
        SearchState::EmptyInventory => Some(SearchStatusLine {
            title: SEARCH_EMPTY_INVENTORY_TITLE.to_string(),
            detail: None,
        }),
        SearchState::Unavailable => Some(SearchStatusLine {
            title: SEARCH_UNAVAILABLE_TITLE.to_string(),
            detail: Some(SEARCH_UNAVAILABLE_DETAIL),
        }),
        SearchState::NoMatch(query) => Some(SearchStatusLine {
            title: format!("{SEARCH_NO_MATCH_PREFIX}{query}{SEARCH_NO_MATCH_SUFFIX}"),
            detail: Some(SEARCH_NO_MATCH_DETAIL),
        }),
        SearchState::Results { .. } => None,
    };
}

/// What:
///      `pub fn search_row_second_line(result: &SearchResult) -> Option<String>`.
///  The parent folder line of a row.
/// Why:
///       A track names its parent folder (D77).
///  A root-level track has no parent folder,
///  and a folder has no
///  second line either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const searchRowSecondLine = (r) => r.kind === "track" ? r.parentContext : null;
/// ```
pub fn search_row_second_line(result: &SearchResult) -> Option<String> {
    return match result.kind {
        SearchResultKind::Folder => None,
        SearchResultKind::Track => result.parent_context.clone(),
    };
}

/// What:
///      `pub fn search_row_kind_label(result: &SearchResult) -> &'static str`.
///  The spoken kind word of a row.
/// Why:
///       The kind is spoken once in the merged row (D77),
///  so the decorative icon carries this label.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const searchRowKindLabel = (r) => r.kind === "folder" ? "Folder" : "Track";
/// ```
pub fn search_row_kind_label(result: &SearchResult) -> &'static str {
    return match result.kind {
        SearchResultKind::Folder => SEARCH_FOLDER_KIND,
        SearchResultKind::Track => SEARCH_TRACK_KIND,
    };
}

/// What:
///      `pub fn is_current_track_result(result: &SearchResult, current_track_index: Option<usize>) -> bool`.
///  Reports whether a result is the track the player currently holds.
/// Why:
///       A track result is the current track when its library index equals the player's index (D73),
///  and a folder
///  never is,
///  because only tracks carry a library index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isCurrentTrackResult = (r, idx) => r.kind === "track" && idx !== null && r.indexInLibrary === idx;
/// ```
pub fn is_current_track_result(result: &SearchResult, current_track_index: Option<usize>) -> bool {
    if result.kind != SearchResultKind::Track {
        return false;
    }
    return match current_track_index {
        Some(current_index) => result.index_in_library == Some(current_index),
        None => false,
    };
}

/// What:
///      `pub fn search_row_action_label(result: &SearchResult, current_track_index: Option<usize>, is_playing: Option<bool>) -> &'static str`.
///  Names what activating a row does.
/// Why:
///       D77 requires the real action in the spoken row.
///  The current track's label depends on its playing state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const searchRowActionLabel = (r, idx, playing) => ...; // folder, select, play, pause, or toggle
/// ```
pub fn search_row_action_label(
    result: &SearchResult,
    current_track_index: Option<usize>,
    is_playing: Option<bool>,
    auto_play_on_select: bool,
) -> &'static str {
    if result.kind == SearchResultKind::Folder {
        return SEARCH_FOLDER_ACTION;
    }
    if !is_current_track_result(result, current_track_index) {
        return if auto_play_on_select { SEARCH_TRACK_PLAY_ACTION } else { SEARCH_TRACK_SELECT_ACTION };
    }
    return match is_playing {
        Some(true) => SEARCH_TRACK_PAUSE_ACTION,
        Some(false) => SEARCH_TRACK_PLAY_ACTION,
        None => SEARCH_TRACK_TOGGLE_ACTION,
    };
}

/// What:
///      `pub fn search_row_state_label(result: &SearchResult, current_track_index: Option<usize>, is_playing: Option<bool>) -> Option<&'static str>`.
///  Names the current-track state,
///  or returns `None` for other rows.
/// Why:
///       The state is structured speech (D77),
///  kept apart from the title so it is not a copied sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const searchRowStateLabel = (r, idx, playing) => isCurrent ? (playing ? "Current track, playing" : ...) : null;
/// ```
pub fn search_row_state_label(
    result: &SearchResult,
    current_track_index: Option<usize>,
    is_playing: Option<bool>,
) -> Option<&'static str> {
    if !is_current_track_result(result, current_track_index) {
        return None;
    }
    return match is_playing {
        Some(true) => Some("Current track, playing"),
        Some(false) => Some("Current track, paused"),
        None => Some("Current track"),
    };
}

/// What:
///      `pub fn search_row_semantics(...) -> SearchRowSemantics`.
///  Bundles the spoken kind,
///  action,
///  and state of one row.
/// Why:
///       The page sets these three values on one merged node,
///  so they are computed in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const searchRowSemantics = (r, idx, playing) => ({ kindLabel, actionLabel, stateLabel });
/// ```
pub fn search_row_semantics(
    result: &SearchResult,
    current_track_index: Option<usize>,
    is_playing: Option<bool>,
    auto_play_on_select: bool,
) -> SearchRowSemantics {
    return SearchRowSemantics {
        kind_label: search_row_kind_label(result),
        action_label: search_row_action_label(result, current_track_index, is_playing, auto_play_on_select),
        state_label: search_row_state_label(result, current_track_index, is_playing),
    };
}

/// What:
///      `pub fn search_emphasis_spans(text: &str, highlights: &[SearchHighlight]) -> Vec<SearchHighlight>`.
///  Passes
///  highlight ranges through when they fit the text.
/// Why:
///       A range outside the text would crash the annotated string,
///  so each range is checked against the text.
///  The text length is counted in characters,
///  matching the offsets of `search_matcher`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const searchEmphasisSpans = (text, hs) => hs.filter(h => h.start >= 0 && h.start < h.end && h.end <= text.length);
/// ```
pub fn search_emphasis_spans(text: &str, highlights: &[SearchHighlight]) -> Vec<SearchHighlight> {
    let text_length: i32 = to_offset(text.chars().count());
    return highlights
        .iter()
        .copied()
        .filter(|highlight| {
            return highlight.start >= 0
                && highlight.start < highlight.end
                && highlight.end <= text_length;
        })
        .collect();
}

/// What:
///      `pub fn initial_query_for_visit(previous: Option<&str>, same_visit: bool) -> String`.
///  The query a Search
///  visit starts with.
/// Why:
///       A new visit starts empty (D66),
///  while a same visit keeps what the user typed.
///  A missing previous query
///  within one visit becomes empty text,
///  not a missing value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const initialQueryForVisit = (prev, same) => same ? (prev ?? "") : "";
/// ```
pub fn initial_query_for_visit(previous: Option<&str>, same_visit: bool) -> String {
    if !same_visit {
        return String::new();
    }
    return previous.unwrap_or("").to_string();
}

/// What:
///      `pub fn should_request_query_focus(same_visit: bool) -> bool`.
///  Says whether entry focuses the field.
/// Why:
///       Opening Search requests edit focus and a keyboard (D63),
///  but a same-visit recomposition does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shouldRequestQueryFocus = (same) => !same;
/// ```
pub fn should_request_query_focus(same_visit: bool) -> bool {
    return !same_visit;
}

/// What:
///      `pub fn should_preserve_row_visibility(previous_query: &str, current_query: &str) -> bool`.
///  Compares queries.
/// Why:
///       Refocusing without a query change must keep the intended row visible (D68),
///  and a changed query resets it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shouldPreserveRowVisibility = (prev, cur) => prev === cur;
/// ```
pub fn should_preserve_row_visibility(previous_query: &str, current_query: &str) -> bool {
    return previous_query == current_query;
}

/// What:
///      `pub fn is_clear_visible(query: &str) -> bool`.
///  Says whether the Clear control appears.
/// Why:
///       Clear is shown only when there is something to clear (D65 and the header rule).
///  Kotlin's `isNotBlank`
///  counts any character outside Kotlin's whitespace set,
///  so the test uses that same set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isClearVisible = (q) => q.trim().length > 0;
/// ```
pub fn is_clear_visible(query: &str) -> bool {
    return query
        .chars()
        .any(|character| return !is_kotlin_whitespace(character));
}

/// Unit tests in `search_page_model_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "search_page_model_tests.rs"]
mod tests;
