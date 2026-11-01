// What:     Tests for `ui_search_anchor.rs`, pulled in by `#[cfg(test)] #[path = "ui_search_anchor_tests.rs"] mod tests;`.
//           They state the policy for D111: the first visible row keeps its place on screen when the list is re-ranked.
// Why:      The policy is decided here as plain arithmetic, apart from ranking correctness and from the window.
use super::*;

// What:     `keys`, owned keys from literals.
// Why:      Each case lists a few keys in rank order.
fn keys(names: &[&str]) -> Vec<String> {
    return names.iter().map(|name| return name.to_string()).collect();
}

// What:     With no change in the list the offset is unchanged.
// Why:      An answer identical to the shown list must not move anything.
#[test]
fn an_unchanged_list_keeps_the_offset() {
    let list = keys(&["a", "b", "c", "d"]);
    assert_eq!(anchored_offset(&list, &list, 100.0, 72.0), 100.0);
}

// What:     A better result inserted above the first visible row pushes the offset down by one row.
// Why:      The row under the user's finger must stay where it is; the new row appears above the viewport.
#[test]
fn a_row_inserted_above_the_viewport_does_not_move_the_visible_row() {
    let old = keys(&["a", "b", "c", "d"]);
    let new = keys(&["x", "a", "b", "c", "d"]);
    assert_eq!(anchored_offset(&old, &new, 144.0, 72.0), 216.0);
}

// What:     At the top, a new first result is placed above the viewport and the first row stays put.
// Why:      The row the user is about to tap at the top of the list must not move either.
#[test]
fn a_new_first_result_does_not_push_the_top_row_down() {
    let old = keys(&["a", "b", "c"]);
    let new = keys(&["x", "a", "b", "c"]);
    assert_eq!(anchored_offset(&old, &new, 0.0, 72.0), 72.0);
}

// What:     A row inserted below the anchor leaves the offset alone.
// Why:      Rows below the first visible one may follow the new ranking.
#[test]
fn a_row_inserted_below_the_anchor_leaves_the_offset() {
    let old = keys(&["a", "b", "c"]);
    let new = keys(&["a", "x", "b", "c"]);
    assert_eq!(anchored_offset(&old, &new, 36.0, 72.0), 36.0);
}

// What:     When the first visible row is gone, the next surviving row anchors, at its old screen position.
// Why:      A result that drops out of the best five hundred must not throw the list back to the top.
#[test]
fn the_next_surviving_row_anchors_when_the_first_visible_row_is_gone() {
    let old = keys(&["a", "b", "c", "d"]);
    let new = keys(&["x", "c", "d"]);
    // Offset 72 shows "b" at the top; "b" is gone, "c" was 72 px lower than "b" and now sits at index 1.
    assert_eq!(anchored_offset(&old, &new, 72.0, 72.0), 0.0);
}

// What:     When no visible row survives, the offset is kept and limited to the new list.
// Why:      There is nothing to anchor to, and the offset must stay inside the list.
#[test]
fn without_a_surviving_row_the_offset_is_kept_within_the_new_list() {
    let old = keys(&["a", "b", "c", "d"]);
    let new = keys(&["x", "y"]);
    assert_eq!(anchored_offset(&old, &new, 216.0, 72.0), 144.0);
}

// What:     An empty old or new list returns a safe offset.
// Why:      The first answer after a clear and an answer with no results must not divide or index out of range.
#[test]
fn empty_lists_give_a_safe_offset() {
    assert_eq!(anchored_offset(&[], &keys(&["a"]), 50.0, 72.0), 50.0);
    assert_eq!(anchored_offset(&keys(&["a"]), &[], 50.0, 72.0), 0.0);
}

// What:     A track is keyed by its library position and a folder by its name.
// Why:      The same track must be recognized in a later list whatever its rank.
#[test]
fn results_are_keyed_by_position_or_name() {
    let track = SearchResult {
        kind: SearchResultKind::Track,
        name: "Song".to_string(),
        parent_context: None,
        index_in_library: Some(7),
        folder_name: "S".to_string(),
        tier: 0,
        name_highlights: Vec::new(),
        parent_highlights: Vec::new(),
    };
    let folder = SearchResult {
        kind: SearchResultKind::Folder,
        index_in_library: None,
        name: "Abba".to_string(),
        ..track.clone()
    };
    assert_eq!(result_key(&track), "t7");
    assert_eq!(result_key(&folder), "fAbba");
}
