//! Tests for the pure library projection in `ui_screen_view`.

use super::*;
use music_player::template_model::TrackFields;

/// What: `identity_at(state, position)` is the identity of the page at a display position.
/// Why: the tests choose a page by position and the state keeps pages in the page index.
fn identity_at(state: &ScreenState, position: usize) -> String {
    return state.current_pages().page(position).map(|page| return page.identity.clone()).unwrap_or_default();
}

/// What: `label_at(state, position)` is the label of the page at a display position.
/// Why: the picker chooses a folder by label.
fn label_at(state: &ScreenState, position: usize) -> String {
    return state.current_pages().page(position).map(|page| return page.label.clone()).unwrap_or_default();
}

/// What: `no_supporting_line` is the supporting-line function that never shows text.
fn no_supporting_line(_track: &TrackFields) -> Option<String> {
    return None;
}

/// What: `state_with(names, current)` builds a state over the given queue names.
fn state_with(names: &[&str], current: Option<usize>) -> ScreenState {
    return ScreenState {
        paths: Vec::new(),
        names: std::sync::Arc::new(names.iter().map(|name| return name.to_string()).collect()),
        current_index: current,
        ..ScreenState::default()
    };
}

#[test]
fn empty_selected_key_selects_the_first_page() {
    let state = state_with(&["Abba/one.mp3", "Beta/two.mp3"], None);
    assert_eq!(state.selected_page_position(), 0);
    let mut unknown = state.clone();
    unknown.selected_key = "no such page".to_string();
    assert_eq!(unknown.selected_page_position(), 0);
}

#[test]
fn selected_page_scope_returns_the_indices_of_the_selected_page() {
    let mut state = state_with(&["Abba/one.mp3", "Beta/two.mp3"], None);
    state.selected_key = identity_at(&state, 1);
    assert_eq!(selected_page_scope(&state), vec![1]);
}

#[test]
fn page_key_containing_finds_the_page_of_a_queue_index() {
    let state = state_with(&["Abba/one.mp3", "Beta/two.mp3"], None);
    assert_eq!(state.page_key_of_index(1), Some(identity_at(&state, 1)));
    assert_eq!(state.page_key_of_index(9), None);
}

#[test]
fn view_marks_the_playing_row_and_names_the_selected_page() {
    let state = state_with(&["Abba/one.mp3", "Abba/two.mp3"], Some(1));
    let mut selected = state.clone();
    selected.selected_key = identity_at(&state, 0);
    let view = view(&selected, &no_supporting_line);
    let current_rows: Vec<&ListRow> = view.rows.iter().filter(|row| return row.current).collect();
    assert_eq!(current_rows.len(), 1);
    assert_eq!(current_rows[0].index, 1);
    assert_eq!(view.page_key, identity_at(&state, 0));
}

#[test]
fn view_page_key_is_empty_without_pages() {
    assert_eq!(view(&ScreenState::default(), &no_supporting_line).page_key, "");
}

#[test]
fn view_deck_title_is_the_playing_track_even_off_the_selected_page() {
    let mut state = state_with(&["Abba/one.mp3", "Beta/two.mp3"], Some(1));
    state.track_name = "Beta/two.mp3".to_string();
    state.selected_key = identity_at(&state, 0);
    let view = view(&state, &no_supporting_line);
    assert_eq!(view.deck_title, "two");
    assert_eq!(view.deck_subtitle, "");
}

#[test]
fn view_deck_title_is_empty_without_a_playing_track() {
    let state = state_with(&["Abba/one.mp3"], None);
    assert_eq!(view(&state, &no_supporting_line).deck_title, "");
}

#[test]
fn view_folder_title_follows_the_selected_page() {
    let mut state = state_with(&["Abba/one.mp3", "Beta/two.mp3"], None);
    state.selected_key = identity_at(&state, 1);
    let view = view(&state, &no_supporting_line);
    assert_eq!(view.folder_title, label_at(&state, 1));
}

#[test]
fn view_of_an_empty_queue_has_no_rows_and_no_picker_cells() {
    let view = view(&ScreenState::default(), &no_supporting_line);
    assert!(view.rows.is_empty());
    assert!(view.picker.cells.is_empty());
}

#[test]
fn select_folder_label_selects_the_page_closes_the_picker_and_returns_its_scope() {
    let mut state = state_with(&["Abba/one.mp3", "Beta/two.mp3"], None);
    state.picker_open = true;
    let label = label_at(&state, 1);
    let scope = select_folder_label(&mut state, &label);
    assert_eq!(scope, Some(vec![1]));
    assert!(!state.picker_open);
    assert_eq!(state.selected_key, identity_at(&state, 1));
}

#[test]
fn select_folder_label_with_unknown_label_changes_nothing() {
    let mut state = state_with(&["Abba/one.mp3"], None);
    state.picker_open = true;
    let before = state.clone();
    assert_eq!(select_folder_label(&mut state, "Missing"), None);
    assert_eq!(state, before);
}

#[test]
fn library_heading_names_the_opened_folder() {
    let mut state = state_with(&[], None);
    state.source_root_name = Some("Albums".to_string());
    assert_eq!(state.library_heading(), "Albums");
}

#[test]
fn library_heading_falls_back_to_folders_without_an_opened_folder() {
    let mut state = state_with(&[], None);
    assert_eq!(state.library_heading(), "Folders");
    state.source_root_name = Some("  ".to_string());
    assert_eq!(state.library_heading(), "Folders");
}

/// What: `playing_state` builds a state whose playing track is the first queue entry in a subfolder.
/// Why: the deck folder and the folder scroll row both read the playing track's directory.
fn playing_state() -> ScreenState {
    let mut state = state_with(&["Camellia/Live/Ghost.flac", "Camellia/Ghost.flac"], Some(0));
    state.track_name = "Camellia/Live/Ghost.flac".to_string();
    return state;
}

#[test]
fn view_fills_deck_folder_with_the_playing_tracks_folder() {
    let view = view(&playing_state(), &no_supporting_line);
    assert_eq!(view.deck_folder, "Live");
}

#[test]
fn view_deck_folder_is_empty_without_a_playing_track() {
    let state = state_with(&["Abba/one.mp3"], None);
    let view = view(&state, &no_supporting_line);
    assert_eq!(view.deck_folder, "");
}

#[test]
fn view_deck_folder_of_a_root_track_is_its_page_label() {
    let mut state = state_with(&["Loose.flac"], Some(0));
    state.track_name = "Loose.flac".to_string();
    assert_eq!(view(&state, &no_supporting_line).deck_folder, "L", "loose files sit in the virtual folder of their letter");
}

#[test]
fn view_deck_title_loses_the_prefix_the_playing_folder_shares_and_does_not_repeat_the_folder() {
    let mut state = state_with(
        &["かめりあ(Camellia)/かめりあ(Camellia) - Aegis.wav", "かめりあ(Camellia)/かめりあ(Camellia) - Another.wav", "Beta/two.mp3"],
        Some(0),
    );
    state.track_name = "かめりあ(Camellia)/かめりあ(Camellia) - Aegis.wav".to_string();
    state.selected_key = identity_at(&state, 1);
    let shown = view(&state, &no_supporting_line);
    assert_eq!(shown.deck_title, "Aegis", "the deck folder is shown beside it, so the title does not repeat it");
    assert_eq!(shown.deck_folder, "かめりあ(Camellia)");
}

#[test]
fn view_rows_lose_the_prefix_their_folder_shares() {
    let mut state = state_with(&["Abba/Abba - one.mp3", "Abba/Abba - two.mp3"], None);
    state.selected_key = identity_at(&state, 0);
    let shown = view(&state, &no_supporting_line);
    let titles: Vec<&str> = shown.rows.iter().map(|row| return row.title.as_str()).collect();
    assert_eq!(titles, vec!["one", "two"]);
}

#[test]
fn loose_tracks_sit_in_virtual_letter_folders_in_natural_order() {
    let mut state = ScreenState::default();
    state.set_names(&["Loose 10.wav".to_string(), "Loose 2.wav".to_string(), "Rock/x.wav".to_string(), "alpha.wav".to_string()]);
    let shown = view(&state, &no_supporting_line);
    let cells: Vec<&str> = shown.picker.cells.iter().map(|cell| return cell.label.as_str()).collect();
    assert_eq!(cells, vec!["A", "L", "R"], "the rail files the virtual folders by letter");
    let mut loose = state.clone();
    loose.selected_key = identity_at(&state, 1);
    let letter = view(&loose, &no_supporting_line);
    assert_eq!(letter.folder_title, "A");
    let l_page = {
        let mut l = state.clone();
        l.selected_key = identity_at(&state, 2);
        view(&l, &no_supporting_line)
    };
    assert_eq!(l_page.folder_title, "L");
    let titles: Vec<&str> = l_page.rows.iter().map(|row| return row.title.as_str()).collect();
    assert_eq!(titles, vec!["10", "2"], "the shared word is dropped (D86); queue order is the walk's order and the view keeps it");
}

#[test]
fn a_single_loose_file_is_still_a_virtual_letter_folder() {
    let mut state = ScreenState::default();
    state.set_names(&["The only track.wav".to_string()]);
    let shown = view(&state, &no_supporting_line);
    assert_eq!(shown.folder_title, "T");
    assert_eq!(shown.rows.len(), 1);
}

#[test]
fn the_opened_folder_name_arriving_later_does_not_change_the_pages_or_the_selection() {
    let mut state = ScreenState::default();
    state.set_names(&["Rock/a.wav".to_string()]);
    state.selected_key = identity_at(&state, 0);
    state.set_source_root_name(Some("Music".to_string()));
    assert_eq!(label_at(&state, 0), "Rock");
    assert_eq!(state.selected_page_position(), 0);
}
