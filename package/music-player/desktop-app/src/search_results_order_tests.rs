// What:     Unit tests for the ordering and limit cases of `search_results.rs`, pulled in by
//           `#[cfg(test)] #[path = "search_results_order_tests.rs"] mod order_tests;` at the bottom of
//           `search_results.rs`. Mirrors the mixed-order, tie-break, and limit cases of `SearchResultsTest.kt`.
// Why:      Split from `search_results_tests.rs` to keep each file under the line budget. Reaches the module items
//           via `use super::*` because this file is the `order_tests` CHILD of search_results.
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

// What:     `fn names_of(items: &[SearchResult]) -> Vec<&str>` test helper: lists result names in order.
// Why:      Ordering assertions compare names, which read more clearly than whole result objects.
fn names_of(items: &[SearchResult]) -> Vec<&str> {
    return items.iter().map(|item| return item.name.as_str()).collect();
}

// What:     `fn truncated_of(state: &SearchState) -> bool` test helper: returns the truncated flag of a results state.
// Why:      The limit cases check the flag next to the list length, and any other state fails the test loudly.
fn truncated_of(state: &SearchState) -> bool {
    return match state {
        SearchState::Results { truncated, .. } => *truncated,
        other => panic!("expected a results state, got {other:?}"),
    };
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The exact `Cam` track precedes the `Camellia` folder, and the later-word match comes last (D61).
#[test]
fn mixed_order_follows_tier_then_length() {
    let state: SearchState = search(
        "cam",
        &[
            "Other/Live at Camellia.flac",
            "Other/Camellia Waltz.flac",
            "Other/Cam.flac",
            "Camellia/Interlude.flac",
        ],
    );
    assert_eq!(
        names_of(items_of(&state)),
        vec!["Cam", "Camellia", "Camellia Waltz", "Live at Camellia"]
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Tier comes before length, so a short later-word match never outranks a prefix match.
#[test]
fn tier_beats_shorter_length() {
    let state: SearchState = search("cam", &["Other/Live Cam.flac", "Other/Camellia Waltz.flac"]);
    assert_eq!(
        names_of(items_of(&state)),
        vec!["Camellia Waltz", "Live Cam"]
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Within a tier, a shorter name is the closer match, so `Cams` precedes `Camellia`.
#[test]
fn shorter_name_first_within_tier() {
    let state: SearchState = search("cam", &["A/Camellia.flac", "A/Cams.flac"]);
    assert_eq!(names_of(items_of(&state)), vec!["Cams", "Camellia"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Names of the same tier and length are ordered as the folder rail orders names, not by raw code units.
#[test]
fn collator_orders_equal_length_names() {
    let state: SearchState = search("cam", &["A/Cambo.flac", "A/Camas.flac"]);
    assert_eq!(names_of(items_of(&state)), vec!["Camas", "Cambo"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Equal titles are distinguished by the display path, so their order is fixed by the path (D77).
#[test]
fn equal_names_order_by_display_path() {
    let state: SearchState = search("cam", &["B/Cam.flac", "A/Cam.flac"]);
    let parents: Vec<Option<&str>> = items_of(&state)
        .iter()
        .map(|item| return item.parent_context.as_deref())
        .collect();
    assert_eq!(parents, vec![Some("A"), Some("B")]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Two tracks with the same path must still come out in one fixed order, by library position.
#[test]
fn identical_display_paths_order_by_library_index() {
    let state: SearchState = search("cam", &["A/Cam.flac", "A/Cam.flac"]);
    let indexes: Vec<Option<usize>> = items_of(&state)
        .iter()
        .map(|item| return item.index_in_library)
        .collect();
    assert_eq!(indexes, vec![Some(0), Some(1)]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The display path decides their order, and the folder `Cam` sorts before `Cam.flac`.
#[test]
fn same_name_folder_and_track_order_by_path() {
    let state: SearchState = search("cam", &["Cam/Other.flac", "Cam.flac"]);
    let kinds: Vec<SearchResultKind> = items_of(&state)
        .iter()
        .map(|item| return item.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![SearchResultKind::Folder, SearchResultKind::Track]
    );
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The list must stop at the limit, and the state must say that more matches existed.
#[test]
fn limit_caps_results_and_flags_truncation() {
    let paths: Vec<String> = (0..=SEARCH_RESULT_LIMIT)
        .map(|index| return format!("Cam{index}.flac"))
        .collect();
    let state: SearchState = search_library("cam", &library_of(&paths), true);
    assert_eq!(items_of(&state).len(), SEARCH_RESULT_LIMIT);
    assert!(truncated_of(&state));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Reaching the limit is not the same as exceeding it, so the flag stays false.
#[test]
fn exactly_limit_is_not_truncated() {
    let paths: Vec<String> = (1..=SEARCH_RESULT_LIMIT)
        .map(|index| return format!("Cam{index}.flac"))
        .collect();
    let state: SearchState = search_library("cam", &library_of(&paths), true);
    assert_eq!(items_of(&state).len(), SEARCH_RESULT_LIMIT);
    assert!(!truncated_of(&state));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      The exact `Cam` track must survive the limit even though it is the last track added.
#[test]
fn limit_keeps_strongest_matches() {
    let mut paths: Vec<String> = (1..=SEARCH_RESULT_LIMIT)
        .map(|index| return format!("Live Cam{index}.flac"))
        .collect();
    paths.push("Cam.flac".to_string());
    let state: SearchState = search_library("cam", &library_of(&paths), true);
    let items: &[SearchResult] = items_of(&state);
    assert_eq!(items[0].name, "Cam");
    assert_eq!(items[0].tier, 0);
    assert_eq!(items.len(), SEARCH_RESULT_LIMIT);
    assert!(truncated_of(&state));
}
