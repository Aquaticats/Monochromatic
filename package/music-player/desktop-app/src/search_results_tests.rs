// What:     Unit tests for `search_results.rs`, pulled in by
//           `#[cfg(test)] #[path = "search_results_tests.rs"] mod tests;` at the bottom of
//           `search_results.rs`. Mirrors the state, scope D, folder, track, and context cases of
//           `SearchResultsTest.kt`. The ordering and limit cases live in `search_results_order_tests.rs`.
// Why:      Split across two sibling files so each stays under the line budget. Reaches the module items via
//           `use super::*` because this file is the `tests` CHILD of search_results.
use super::*;

// What:     `fn library_of<S: AsRef<str>>(paths: &[S]) -> Vec<Track>` test helper: one track per display path.
// Why:      Each track needs its own URI so that library positions are distinct in the results.
fn library_of<S: AsRef<str>>(paths: &[S]) -> Vec<Track> {
    return paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            return Track {
                uri: format!("uri-{index}"),
                display_path: path.as_ref().to_string(),
            };
        })
        .collect();
}

// What:     `fn search(query: &str, paths: &[&str]) -> SearchState` test helper: searches an available library.
// Why:      Most cases list a few literal paths inline, and this keeps each case to one line.
fn search(query: &str, paths: &[&str]) -> SearchState {
    return search_library(query, &library_of(paths), true);
}

// What:     `fn items_of(state: &SearchState) -> &[SearchResult]` test helper: returns the items of a results state.
// Why:      Ordering cases read the result list directly, and any other state fails the test loudly.
fn items_of(state: &SearchState) -> &[SearchResult] {
    return match state {
        SearchState::Results { items, .. } => items.as_slice(),
        other => panic!("expected a results state, got {other:?}"),
    };
}

// What:     `fn only(items: &[SearchResult]) -> &SearchResult` test helper: returns the single result.
// Why:      Mirrors the Kotlin `single()` calls, so a wrong count fails loudly.
fn only(items: &[SearchResult]) -> &SearchResult {
    assert_eq!(items.len(), 1, "expected exactly one result");
    return &items[0];
}

// What:     `fn names_of(items: &[SearchResult]) -> Vec<&str>` test helper: lists result names in order.
// Why:      Ordering assertions compare names, which read more clearly than whole result objects.
fn names_of(items: &[SearchResult]) -> Vec<&str> {
    return items.iter().map(|item| return item.name.as_str()).collect();
}

// What:     `fn kinds_of(items: &[SearchResult]) -> Vec<SearchResultKind>` test helper: lists result kinds in order.
// Why:      Several cases check that a folder and a track appear in a given order.
fn kinds_of(items: &[SearchResult]) -> Vec<SearchResultKind> {
    return items.iter().map(|item| return item.kind).collect();
}

// What:     `fn span(start: i32, end: i32) -> SearchHighlight` test helper: builds one expected highlight range.
// Why:      Highlight assertions compare lists of ranges, and the literal struct form is long.
fn span(start: i32, end: i32) -> SearchHighlight {
    return SearchHighlight { start, end };
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A blank query is not a failed lookup, so it must not render a no-match verdict (D70).
#[test]
fn blank_query_is_empty_query() {
    assert_eq!(
        search_library("   ", &library_of(&["Cam.flac"]), true),
        SearchState::EmptyQuery
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Such a query matches nothing, so it must show the prompt rather than a no-match verdict.
#[test]
fn combining_only_query_is_empty_query() {
    assert_eq!(
        search_library("\u{0301}", &library_of(&["Cam.flac"]), true),
        SearchState::EmptyQuery
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An explicit source failure takes precedence over ordinary empty-query copy (D69).
#[test]
fn unavailable_wins_over_blank_query() {
    assert_eq!(
        search_library("", &library_of(&["Cam.flac"]), false),
        SearchState::Unavailable
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A known failure must not be hidden behind a result list built from a stale or partial source.
#[test]
fn unavailable_wins_over_query() {
    assert_eq!(
        search_library("cam", &library_of(&["Cam.flac"]), false),
        SearchState::Unavailable
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A confirmed empty inventory is different from a finished no-match (D69), so it has its own state.
#[test]
fn empty_library_is_empty_inventory() {
    assert_eq!(
        search_library("cam", &library_of::<&str>(&[]), true),
        SearchState::EmptyInventory
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      An unavailable source and a blank query are reported before the empty-inventory state.
#[test]
fn empty_library_keeps_higher_precedence_states() {
    assert_eq!(
        search_library("  ", &library_of::<&str>(&[]), true),
        SearchState::EmptyQuery
    );
    assert_eq!(
        search_library("cam", &library_of::<&str>(&[]), false),
        SearchState::Unavailable
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Only a library with no tracks is an empty inventory, so tracks with no hit remain a no-match.
#[test]
fn non_empty_library_without_hit_is_no_match() {
    assert_eq!(
        search("zzq", &["Cam.flac"]),
        SearchState::NoMatch("zzq".to_string())
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The page names what was typed, and surrounding spaces are not part of the text the user meant.
#[test]
fn no_match_carries_trimmed_query() {
    assert_eq!(
        search("  zzq  ", &["Cam.flac"]),
        SearchState::NoMatch("zzq".to_string())
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      D60 keeps matching folders visible through their own names.
#[test]
fn folder_own_name_matches() {
    assert_eq!(
        kinds_of(items_of(&search("cam", &["Camellia/Waltz.flac"]))),
        vec![SearchResultKind::Folder]
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The track `Another Xronixle` must not appear for `cam` just because its folder is `Camellia` (D60).
#[test]
fn parent_folder_match_does_not_produce_tracks() {
    assert_eq!(
        names_of(items_of(&search(
            "cam",
            &["Camellia/Another Xronixle.flac"]
        ))),
        vec!["Camellia"]
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A track result must name its title, kind, folder context, and library position.
#[test]
fn track_own_title_matches_with_parent_context() {
    let state: SearchState = search("cam", &["Albums/Cam.flac"]);
    let result: &SearchResult = only(items_of(&state));
    assert_eq!(result.kind, SearchResultKind::Track);
    assert_eq!(result.name, "Cam");
    assert_eq!(result.parent_context.as_deref(), Some("Albums"));
    assert_eq!(result.index_in_library, Some(0));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A root-level track is still a track with a title, so it must be found by that title (D77).
#[test]
fn unfiled_tracks_are_searched_by_title() {
    let state: SearchState = search("cam", &["Cam.flac"]);
    let result: &SearchResult = only(items_of(&state));
    assert_eq!(result.kind, SearchResultKind::Track);
    assert_eq!(result.parent_context, None);
    assert_eq!(result.folder_name, "C", "a root-level track sits in the virtual folder of its letter, as in the picker");
}

// What:     Root-level tracks name the same virtual folder in Search as the picker's page for them.
// Why:      LS1: Search said `Unfiled` while the picker said `A`; non-letter initials share the `#` folder.
#[test]
fn root_level_tracks_name_their_picker_letter_folder() {
    let state: SearchState = search("e", &["Eleven.flac", "5 elements.wav", "élan.wav", "Rock/Echo.flac"]);
    let folders: Vec<(String, String)> = items_of(&state)
        .iter()
        .filter(|item| return item.kind == SearchResultKind::Track)
        .map(|item| return (item.name.clone(), item.folder_name.clone()))
        .collect();
    assert!(folders.contains(&("Eleven".to_string(), "E".to_string())), "{folders:?}");
    assert!(folders.contains(&("5 elements".to_string(), "#".to_string())), "{folders:?}");
    assert!(folders.contains(&("élan".to_string(), "#".to_string())), "{folders:?}");
    assert!(folders.contains(&("Echo".to_string(), "Rock".to_string())), "{folders:?}");
    assert!(!folders.iter().any(|(_, folder)| return folder == "Unfiled"));
    let adele: SearchState = search("adele", &["Adele-Meow.mp3"]);
    assert_eq!(only(items_of(&adele)).folder_name, "A");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A root-level track has no context line, so the query `un` must not highlight anything there.
#[test]
fn root_track_has_no_context_highlight() {
    let state: SearchState = search("un", &["Un Cam.flac"]);
    let result: &SearchResult = only(items_of(&state));
    assert_eq!(result.name_highlights, vec![span(0, 2)]);
    assert_eq!(result.parent_context, None);
    assert_eq!(result.parent_highlights, Vec::<SearchHighlight>::new());
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The `Unfiled` group is a container for root-level tracks, not a folder the user can open.
#[test]
fn unfiled_group_is_not_a_folder_result() {
    assert_eq!(
        search("unf", &["Other.flac"]),
        SearchState::NoMatch("unf".to_string())
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A real folder must stay searchable by its own name, even though it shares the reserved label.
#[test]
fn real_folder_named_unfiled_is_a_folder_result() {
    assert_eq!(
        names_of(items_of(&search("unf", &["Unfiled/Song.flac"]))),
        vec!["Unfiled"]
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Adding root-level tracks must not duplicate the real folder or add the root-level group as a result.
#[test]
fn real_folder_named_unfiled_keeps_only_its_own_folder_result() {
    assert_eq!(
        names_of(items_of(&search(
            "unf",
            &["Unfiled/Song.flac", "Root.flac"]
        ))),
        vec!["Unfiled"]
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The playback position must be the library position, not the result position, so each track keeps it.
#[test]
fn index_in_library_is_preserved() {
    let state: SearchState = search("cam", &["x/a.flac", "Cam.flac", "y/Cam.flac"]);
    let indexes: Vec<Option<usize>> = items_of(&state)
        .iter()
        .map(|item| return item.index_in_library)
        .collect();
    assert_eq!(indexes, vec![Some(1), Some(2)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The second line names the folder as displayed, including every subfolder segment (D77).
#[test]
fn parent_context_is_the_displayed_folder_path() {
    let state: SearchState = search("cam", &["Album/Disc 1/Cam.flac"]);
    let result: &SearchResult = only(items_of(&state));
    assert_eq!(result.parent_context.as_deref(), Some("Album/Disc 1"));
    assert_eq!(result.folder_name, "Album");
}

// What:     `#[test]` marks the next function as a test case.
// Why:      D59 emphasizes the query in the parent context too, while the parent never produces a match.
#[test]
fn parent_highlights_mark_word_starts_in_context() {
    let state: SearchState = search("cam", &["Mix Cam/Cam.flac"]);
    let results: Vec<&SearchResult> = items_of(&state)
        .iter()
        .filter(|item| return item.kind == SearchResultKind::Track)
        .collect();
    assert_eq!(results.len(), 1);
    let result: &SearchResult = results[0];
    assert_eq!(result.name_highlights, vec![span(0, 3)]);
    assert_eq!(result.parent_context.as_deref(), Some("Mix Cam"));
    assert_eq!(result.parent_highlights, vec![span(4, 7)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A folder has no library position and no parent context, so those fields must be empty.
#[test]
fn folder_result_has_no_track_fields() {
    let state: SearchState = search("cam", &["Camellia/Waltz.flac"]);
    let result: &SearchResult = only(items_of(&state));
    assert_eq!(result.kind, SearchResultKind::Folder);
    assert_eq!(result.folder_name, "Camellia");
    assert_eq!(result.parent_context, None);
    assert_eq!(result.index_in_library, None);
    assert_eq!(result.parent_highlights, Vec::<SearchHighlight>::new());
    assert_eq!(result.name_highlights, vec![span(0, 3)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Selecting the owning page needs the top-level folder, which is the page the folder index names.
#[test]
fn track_folder_name_is_the_top_level_folder() {
    let state: SearchState = search("cam", &["Top/Sub/Cam.flac"]);
    let result: &SearchResult = only(items_of(&state));
    assert_eq!(result.folder_name, "Top");
    assert_eq!(result.parent_context.as_deref(), Some("Top/Sub"));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The title is the file name without its last extension, so `flac` must not match `Dream.flac`.
#[test]
fn extension_is_not_matched() {
    assert_eq!(
        search("flac", &["Dream.flac"]),
        SearchState::NoMatch("flac".to_string())
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      With two terms, the title `Live at Camellia` matches `live cam` through its first term at the start.
#[test]
fn multi_term_track_match_uses_title_only() {
    let state: SearchState = search("live cam", &["Other/Live at Camellia.flac"]);
    assert_eq!(only(items_of(&state)).tier, 1);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A result list must not show a track that only satisfies part of the query.
#[test]
fn multi_term_no_match_when_one_term_is_missing() {
    assert_eq!(
        search("live zzz", &["Other/Live at Camellia.flac"]),
        SearchState::NoMatch("live zzz".to_string())
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Japanese titles use the same prefix rule, so `ラジオ` finds `ラジオ体操` at tier one.
#[test]
fn japanese_track_title_matches() {
    let state: SearchState = search("ラジオ", &["演歌/ラジオ体操.flac"]);
    let result: &SearchResult = only(items_of(&state));
    assert_eq!(result.name, "ラジオ体操");
    assert_eq!(result.tier, 1);
}
