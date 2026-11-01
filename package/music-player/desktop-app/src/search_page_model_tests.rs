// What:     Unit tests for `search_page_model.rs`, pulled in by
//           `#[cfg(test)] #[path = "search_page_model_tests.rs"] mod tests;` at the bottom of
//           `search_page_model.rs`. Mirrors every `@Test` of `SearchPageModelTest.kt`, with the same inputs and
//           expected copy. One extra control covers the whitespace branch of `is_clear_visible`.
// Why:      Each copy string and branch is compared exactly, so a changed character fails the build.
//           Reaches the module items via `use super::*` because this file is the `tests` CHILD of search_page_model.
use super::*;

// What:     `fn folder_result(name: &str) -> SearchResult` test helper: a folder result whose own name is the text.
// Why:      Each case needs only the fields that the model reads, so the fixture stays small and explicit.
fn folder_result(name: &str) -> SearchResult {
    return SearchResult {
        kind: SearchResultKind::Folder,
        name: name.to_string(),
        parent_context: None,
        index_in_library: None,
        folder_name: name.to_string(),
        tier: 0,
        name_highlights: Vec::new(),
        parent_highlights: Vec::new(),
    };
}

// What:     `fn track_result(name: &str, parent: Option<&str>, index: usize) -> SearchResult` test helper: a track result.
// Why:      Track cases vary by parent and library index, which are the two fields the labels depend on.
fn track_result(name: &str, parent: Option<&str>, index: usize) -> SearchResult {
    return SearchResult {
        kind: SearchResultKind::Track,
        name: name.to_string(),
        parent_context: parent.map(|parent_path| return parent_path.to_string()),
        index_in_library: Some(index),
        folder_name: parent.unwrap_or("").to_string(),
        tier: 0,
        name_highlights: Vec::new(),
        parent_highlights: Vec::new(),
    };
}

// What:     `fn span(start: i32, end: i32) -> SearchHighlight` test helper: builds one highlight range.
// Why:      Emphasis cases compare lists of ranges, and the literal struct form is long.
fn span(start: i32, end: i32) -> SearchHighlight {
    return SearchHighlight { start, end };
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The prompt is an invitation, and its wording is fixed by the review text (D70).
#[test]
fn prompt_status_uses_review_copy() {
    let line: SearchStatusLine =
        search_status_line(&SearchState::EmptyQuery).expect("prompt has a status line");
    assert_eq!(line.title, "Search your music");
    assert_eq!(line.detail, Some("Type a name to explore your library."));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A confirmed empty inventory is not a finished no-match, so it carries no no-match support line (D69).
#[test]
fn empty_inventory_status_has_headline_only() {
    let line: SearchStatusLine =
        search_status_line(&SearchState::EmptyInventory).expect("inventory has a status line");
    assert_eq!(line.title, "No music found in your audio library.");
    assert_eq!(line.detail, None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      D71 rejects "Library unavailable" and any recovery promise, so the copy names only the known fact.
#[test]
fn unavailable_status_names_no_cause() {
    let line: SearchStatusLine =
        search_status_line(&SearchState::Unavailable).expect("unavailable has a status line");
    assert_eq!(line.title, "Search is unavailable");
    assert_eq!(
        line.detail,
        Some("The current library can't be searched right now.")
    );
    assert!(!line.title.contains("Library unavailable"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      D70 requires the headline to name the entered query, with the try-another-name support line.
#[test]
fn no_match_status_names_query() {
    let line: SearchStatusLine = search_status_line(&SearchState::NoMatch("zzq".to_string()))
        .expect("no match has a status line");
    assert_eq!(line.title, "No results for \u{201C}zzq\u{201D}");
    assert_eq!(line.detail, Some("Try another name."));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Status copy would hide the list, so Results must return no status line.
#[test]
fn results_have_no_status_line() {
    let results: SearchState = SearchState::Results {
        items: Vec::new(),
        truncated: false,
    };
    assert_eq!(search_status_line(&results), None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A folder has no parent folder to name, so no second line is drawn for it.
#[test]
fn folder_second_line_is_absent() {
    assert_eq!(search_row_second_line(&folder_result("Camellia")), None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      D77 uses the parent folder as the disambiguating second line.
#[test]
fn track_second_line_is_parent_folder() {
    assert_eq!(
        search_row_second_line(&track_result("Night", Some("Camellia/Live"), 3)),
        Some("Camellia/Live".to_string())
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A root-level track has no parent folder, so no second line is invented for it.
#[test]
fn root_track_has_no_second_line() {
    assert_eq!(
        search_row_second_line(&track_result("Night", None, 0)),
        None
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The spoken row must not describe a parent that does not exist.
#[test]
fn root_track_semantics_names_no_library_root() {
    let semantics: SearchRowSemantics =
        search_row_semantics(&track_result("Night", None, 0), None, None, false);
    assert_eq!(semantics.kind_label, "Track");
    assert_eq!(semantics.action_label, "Select track");
    assert_eq!(semantics.state_label, None);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The kind is spoken once in the row (D77), so each kind needs its own word.
#[test]
fn kind_labels_name_folder_and_track() {
    assert_eq!(search_row_kind_label(&folder_result("Camellia")), "Folder");
    assert_eq!(
        search_row_kind_label(&track_result("Night", Some("Live"), 1)),
        "Track"
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A track result is the playing track only when its library index equals the player's index (D73).
#[test]
fn current_track_matches_library_index() {
    assert!(is_current_track_result(
        &track_result("Night", Some("Live"), 4),
        Some(4)
    ));
    assert!(!is_current_track_result(
        &track_result("Night", Some("Live"), 4),
        Some(5)
    ));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Only tracks carry a library index, so a folder must never take the toggle action.
#[test]
fn folder_is_never_current_track() {
    assert!(!is_current_track_result(
        &folder_result("Camellia"),
        Some(0)
    ));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      With no current track, no row may claim the current-track state.
#[test]
fn no_current_track_means_no_current_row() {
    assert!(!is_current_track_result(
        &track_result("Night", Some("Live"), 0),
        None
    ));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A folder result opens its folder without starting playback (D72).
#[test]
fn folder_action_opens_without_autoplay() {
    assert_eq!(
        search_row_action_label(&folder_result("Camellia"), Some(2), Some(true), false),
        "Open folder without autoplay"
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A different track is only selected (D127), so its label is Select track and not Play track.
#[test]
fn other_track_action_selects() {
    assert_eq!(
        search_row_action_label(&track_result("Night", Some("Live"), 1), Some(2), Some(true), false),
        "Select track"
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The current-track toggle pauses while playing (D73 and D77).
#[test]
fn playing_current_track_action_pauses() {
    assert_eq!(
        search_row_action_label(&track_result("Night", Some("Live"), 2), Some(2), Some(true), false),
        "Pause track"
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The current-track toggle resumes while paused, so the label is Play track (D77).
#[test]
fn paused_current_track_action_plays() {
    assert_eq!(
        search_row_action_label(
            &track_result("Night", Some("Live"), 2),
            Some(2),
            Some(false),
            false
        ),
        "Play track"
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      When the playing state is not known, the label names the toggle instead of a claimed state.
#[test]
fn unknown_state_current_track_action_toggles() {
    assert_eq!(
        search_row_action_label(&track_result("Night", Some("Live"), 2), Some(2), None, false),
        "Play or pause track"
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Structured state speech belongs to the current track alone (D77).
#[test]
fn state_label_is_null_for_other_rows() {
    assert_eq!(
        search_row_state_label(&folder_result("Camellia"), Some(2), Some(true)),
        None
    );
    assert_eq!(
        search_row_state_label(&track_result("Night", Some("Live"), 1), Some(2), Some(true)),
        None
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The state is spoken apart from the title, so it names the playing state directly.
#[test]
fn state_label_reports_playing_current_track() {
    assert_eq!(
        search_row_state_label(&track_result("Night", Some("Live"), 2), Some(2), Some(true)),
        Some("Current track, playing")
    );
    assert_eq!(
        search_row_state_label(
            &track_result("Night", Some("Live"), 2),
            Some(2),
            Some(false)
        ),
        Some("Current track, paused")
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The page reads all three labels from one call, so the bundle must match the pieces.
#[test]
fn semantics_bundle_combines_kind_action_and_state() {
    let semantics: SearchRowSemantics =
        search_row_semantics(&track_result("Night", Some("Live"), 2), Some(2), Some(true), false);
    assert_eq!(semantics.kind_label, "Track");
    assert_eq!(semantics.action_label, "Pause track");
    assert_eq!(semantics.state_label, Some("Current track, playing"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Valid ranges pass through unchanged, in their original order, for the annotated string.
#[test]
fn emphasis_keeps_valid_ranges() {
    let highlights: Vec<SearchHighlight> = vec![span(0, 3), span(5, 8)];
    assert_eq!(search_emphasis_spans("Cam Cama", &highlights), highlights);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A range past the end would crash the annotated string, so it must be dropped.
#[test]
fn emphasis_drops_out_of_range_ranges() {
    assert_eq!(
        search_emphasis_spans("Cam", &[span(0, 4)]),
        Vec::<SearchHighlight>::new()
    );
    assert_eq!(
        search_emphasis_spans("Cam", &[span(-1, 2)]),
        Vec::<SearchHighlight>::new()
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An empty range paints nothing, so it is not a highlight.
#[test]
fn emphasis_drops_empty_ranges() {
    assert_eq!(
        search_emphasis_spans("Cam", &[span(1, 1)]),
        Vec::<SearchHighlight>::new()
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A new visit starts with an empty query even when the previous text exists (D66).
#[test]
fn new_visit_starts_empty() {
    assert_eq!(initial_query_for_visit(Some("cam"), false), "");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Within one visit the typed query stays as the user left it.
#[test]
fn same_visit_keeps_query() {
    assert_eq!(initial_query_for_visit(Some("cam"), true), "cam");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A continuing visit with no stored query must still produce an empty string, not a missing value.
#[test]
fn same_visit_without_previous_is_empty() {
    assert_eq!(initial_query_for_visit(None, true), "");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Opening Search requests edit focus (D63), while a continuing visit does not request it again.
#[test]
fn focus_requested_only_on_new_visit() {
    assert!(should_request_query_focus(false));
    assert!(!should_request_query_focus(true));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      D68 keeps the intended row visible when the query has not changed.
#[test]
fn unchanged_query_preserves_row_visibility() {
    assert!(should_preserve_row_visibility("cam", "cam"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A new query has new results, so the old position is not preserved.
#[test]
fn changed_query_resets_row_visibility() {
    assert!(!should_preserve_row_visibility("cam", "cami"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Clear appears only when there is text to remove.
#[test]
fn clear_visible_for_non_blank_query() {
    assert!(is_clear_visible("cam"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A blank query has nothing to clear, so the control is hidden.
#[test]
fn clear_hidden_for_blank_query() {
    assert!(!is_clear_visible(""));
    assert!(!is_clear_visible("   "));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Kotlin's `isNotBlank` treats a non-breaking space and an information separator as blank, so Clear stays hidden.
#[test]
fn clear_hidden_for_non_breaking_space_and_separator_only_query() {
    assert!(!is_clear_visible("\u{A0}\u{1C}"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A restored query begins at the top of its results, never a deep prior position (D67).
#[test]
fn restored_results_start_at_first_index() {
    assert_eq!(SEARCH_RESTORED_RESULT_INDEX, 0);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      With Auto-play on select on (D129) a result for another track starts it, so the label is Play track again.
#[test]
fn other_track_action_plays_with_auto_play_on() {
    assert_eq!(
        search_row_action_label(&track_result("Night", Some("Live"), 1), Some(2), Some(true), true),
        "Play track"
    );
    assert_eq!(
        search_row_semantics(&track_result("Night", Some("Live"), 1), None, None, true).action_label,
        "Play track"
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The setting changes only the label of a track that is not current.
#[test]
fn auto_play_does_not_change_the_folder_or_current_track_labels() {
    assert_eq!(
        search_row_action_label(&folder_result("Camellia"), Some(2), Some(true), true),
        "Open folder without autoplay"
    );
    assert_eq!(
        search_row_action_label(&track_result("Night", Some("Live"), 2), Some(2), Some(true), true),
        "Pause track"
    );
}
