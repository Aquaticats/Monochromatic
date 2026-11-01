// What:     Tests for `open_folder_result` in `ui_search.rs`, pulled in by `#[cfg(test)] #[path = "ui_search_folder_tests.rs"] mod tests;`.
// Why:      D111: a folder result tapped while the scan has not produced the library rows must leave Search open, and a
//           folder result tapped after must select the folder page and return to the player.
use super::*;

// What:     A state with Search open and the given library names.
// Why:      Each case needs the route and the names only.
fn searching(names: &[&str]) -> ScreenState {
    let mut state: ScreenState = ScreenState {
        route: Route::Search,
        ..ScreenState::default()
    };
    let owned: Vec<String> = names.iter().map(|name| return name.to_string()).collect();
    let _shared = state.set_names(&owned);
    return state;
}

// What:     With no library rows yet, the tap changes nothing.
// Why:      The scan has found tracks for Search but the page index is built when the queue arrives.
#[test]
fn a_folder_tap_before_the_library_rows_exist_keeps_search_open() {
    let mut state: ScreenState = searching(&[]);
    assert_eq!(open_folder_result(&mut state, "Abba"), None);
    assert_eq!(state.route, Route::Search);
}

// What:     With the library rows, the tap selects the folder page, returns to the player and reports the page scope.
// Why:      D72: a folder result opens its folder without autoplay.
#[test]
fn a_folder_tap_after_the_library_rows_exist_opens_the_folder() {
    let mut state: ScreenState = searching(&["Abba/one.mp3", "Abba/two.mp3", "Beta/three.mp3"]);
    assert_eq!(open_folder_result(&mut state, "Abba"), Some(vec![0, 1]));
    assert_eq!(state.route, Route::Player);
    assert_eq!(state.selected_key, "folder:Abba");
}

// What:     A folder name that is not a page changes nothing.
// Why:      A stale result must not select a page that does not exist.
#[test]
fn a_folder_tap_for_an_unknown_folder_keeps_search_open() {
    let mut state: ScreenState = searching(&["Abba/one.mp3"]);
    assert_eq!(open_folder_result(&mut state, "Missing"), None);
    assert_eq!(state.route, Route::Search);
}
