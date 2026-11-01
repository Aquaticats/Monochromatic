// What:     Tests for `activate_result` in `ui_search.rs`, pulled in by
//           `#[cfg(test)] #[path = "ui_search_activation_tests.rs"] mod activation_tests;`.
// Why:      D127: a Search result that names a track follows the row click rule (D124): a track that is not current is
//           only selected, the current track toggles play and pause, and a folder result still opens its folder.
use super::*;
use crate::ui_test_support::fresh_window_at;
use crate::ui_test_support::load_library;
use music_player::search_matcher::SearchHighlight;
use music_player::search_results::SearchResult;

// What:     A track result at library position `index`.
// Why:      The cases need only the kind, the position and the folder.
fn track_result(index: usize) -> SearchResult {
    return SearchResult {
        kind: SearchResultKind::Track,
        name: format!("Track {index}"),
        parent_context: None,
        index_in_library: Some(index),
        folder_name: "Abba".to_owned(),
        tier: 0,
        name_highlights: Vec::<SearchHighlight>::new(),
        parent_highlights: Vec::<SearchHighlight>::new(),
    };
}

// What:     Puts Search on screen with the given results and the given current track, then activates `position`.
// Why:      Every case differs only in the results, the current track and the activated position.
fn activate(results: Vec<SearchResult>, current: Option<usize>, position: usize) -> (Vec<String>, Route) {
    return activate_with_setting(false, results, current, position);
}

// What:     `activate` with Auto-play on select set first.
// Why:      D129: the setting changes what a track result does, and nothing else about the activation.
fn activate_with_setting(
    auto_play_on_select: bool,
    results: Vec<SearchResult>,
    current: Option<usize>,
    position: usize,
) -> (Vec<String>, Route) {
    let (app, host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    update_state(|state| {
        state.auto_play_on_select = auto_play_on_select;
        state.route = Route::Search;
        state.current_index = current;
        state.search_ui.results = results;
    });
    activate_result(&app, host.as_ref(), position);
    let route = read_state(|state| return state.route);
    return (host.sent_texts(), route);
}

// What:     A track result that is not the current track only selects it.
// Why:      D127: no `play` follows the `select`, so the track loads paused and any playing track stops.
#[test]
fn a_track_result_that_is_not_current_only_selects_the_track() {
    let (sent, route) = activate(vec![track_result(2)], Some(0), 0);
    assert_eq!(sent, vec!["select 2".to_string()]);
    assert_eq!(route, Route::Player, "the page returns to the player (D74)");
}

// What:     With nothing current, a track result still only selects.
// Why:      A first activation after launch must not start audio.
#[test]
fn a_track_result_with_nothing_current_only_selects_the_track() {
    let (sent, _route) = activate(vec![track_result(1)], None, 0);
    assert_eq!(sent, vec!["select 1".to_string()]);
}

// What:     The result of the current track toggles play and pause.
// Why:      D127: the second activation of the same result is the row's current-track toggle.
#[test]
fn the_result_of_the_current_track_toggles_play_and_pause() {
    let (sent, route) = activate(vec![track_result(2)], Some(2), 0);
    assert_eq!(sent, vec!["toggle".to_string()]);
    assert_eq!(route, Route::Player);
}

// What:     A folder result sends the page scope and no track command.
// Why:      D72 stays: a folder result opens its folder without selecting or playing a track.
#[test]
fn a_folder_result_still_opens_its_folder_without_a_track_command() {
    let folder = SearchResult {
        kind: SearchResultKind::Folder,
        name: "Beck".to_owned(),
        folder_name: "Beck".to_owned(),
        index_in_library: None,
        ..track_result(0)
    };
    let (sent, _route) = activate(vec![folder], Some(0), 0);
    assert_eq!(sent, vec!["scope 2".to_string()]);
}

// What:     A folder result that also carries a library position still opens its folder.
// Why:      The kind decides, not the position, so a folder never selects a track by accident (found by mutation: the
//           kind check could be dropped while every other case kept passing).
#[test]
fn a_folder_result_with_a_library_position_still_opens_its_folder() {
    let folder = SearchResult {
        kind: SearchResultKind::Folder,
        name: "Beck".to_owned(),
        folder_name: "Beck".to_owned(),
        index_in_library: Some(1),
        ..track_result(0)
    };
    let (sent, _route) = activate(vec![folder], Some(0), 0);
    assert_eq!(sent, vec!["scope 2".to_string()]);
}

// What:     With Auto-play on select on, a track result that is not current is selected and starts at once.
// Why:      D129: the setting restores the behavior before D127 for results.
#[test]
fn with_auto_play_on_a_track_result_that_is_not_current_selects_and_starts_the_track() {
    let (sent, route) = activate_with_setting(true, vec![track_result(2)], Some(0), 0);
    assert_eq!(sent, vec!["select 2".to_string(), "play".to_string()]);
    assert_eq!(route, Route::Player, "the page returns to the player (D74)");
}

// What:     With Auto-play on select on and nothing current, a track result selects and starts.
// Why:      The first activation after launch is the case the setting is for.
#[test]
fn with_auto_play_on_a_track_result_with_nothing_current_selects_and_starts_the_track() {
    let (sent, _route) = activate_with_setting(true, vec![track_result(1)], None, 0);
    assert_eq!(sent, vec!["select 1".to_string(), "play".to_string()]);
}

// What:     With Auto-play on select on, the result of the current track still toggles.
// Why:      D129: the current track toggles in both modes, so a second activation pauses instead of restarting.
#[test]
fn with_auto_play_on_the_result_of_the_current_track_still_toggles() {
    let (sent, _route) = activate_with_setting(true, vec![track_result(2)], Some(2), 0);
    assert_eq!(sent, vec!["toggle".to_string()]);
}

// What:     With Auto-play on select on, a folder result still only opens its folder.
// Why:      D129: folders are not tracks, so the setting does not reach them.
#[test]
fn with_auto_play_on_a_folder_result_still_opens_its_folder_without_a_track_command() {
    let folder = SearchResult {
        kind: SearchResultKind::Folder,
        name: "Beck".to_owned(),
        folder_name: "Beck".to_owned(),
        index_in_library: None,
        ..track_result(0)
    };
    let (sent, _route) = activate_with_setting(true, vec![folder], Some(0), 0);
    assert_eq!(sent, vec!["scope 2".to_string()]);
}
