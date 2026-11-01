//! Window tests of the empty, reading and failure states: what the pane, the deck, the rows and the messages show.
//!
//! What: each test builds a window on the testing backend, feeds it engine updates and reads the window's properties.
//! Why: EM1, EM2, EM5, PB1 and PB2 of the second polish review are about what the window shows when the library is
//! being read, empty, or has a file that fails; a unit test of the model alone would not see the pane or the deck.

use music_player::command::Update;
use slint::{ComponentHandle, Model};
use std::path::PathBuf;

use crate::DeckState;
use crate::ui_test_support::{fresh_window_at, load_library, settle};

/// What: `feedback_texts` lists the texts of the visible messages.
/// Why: the tests read what the user is told.
fn feedback_texts(app: &crate::AppWindow) -> Vec<String> {
    return app.get_feedback_items().iter().map(|item| return item.text.to_string()).collect();
}

#[test]
fn a_folder_being_read_shows_the_reading_pane_and_a_disabled_deck_not_no_folder_is_open() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    crate::ui_screen::apply_update(&app, &Update::Reading { name: "Music".to_string() });
    settle();
    assert_eq!(app.get_first_run_state(), 1, "the pane is shown");
    assert_eq!(app.get_first_run_title().as_str(), "Reading your library");
    assert!(app.global::<DeckState>().get_disabled(), "the deck is off while there is nothing to play");
    assert_eq!(app.get_folded_title().as_str(), "Music", "the folded header names the library, never only a caret (EM5)");
    assert_eq!(app.get_folder_title().as_str(), "", "the wide right pane does not repeat the name");
}

#[test]
fn the_pane_and_the_disabled_deck_end_when_the_queue_arrives_with_tracks() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    crate::ui_screen::apply_update(&app, &Update::Reading { name: "Music".to_string() });
    load_library(&app);
    assert_eq!(app.get_first_run_state(), 0);
    assert!(!app.global::<DeckState>().get_disabled());
}

#[test]
fn an_empty_folder_after_reading_names_the_folder_and_keeps_the_deck_off() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    crate::ui_screen::apply_update(&app, &Update::Reading { name: "lib-noaudio".to_string() });
    crate::ui_screen::apply_update(&app, &Update::Queue(Vec::new()));
    crate::ui_screen::apply_update(&app, &Update::SourceRoot(Some("lib-noaudio".to_string())));
    settle();
    assert_eq!(app.get_first_run_title().as_str(), "No audio found in\nlib-noaudio");
    assert!(app.global::<DeckState>().get_disabled());
}

#[test]
fn no_folder_open_is_shown_only_when_the_engine_says_so() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    assert_eq!(app.get_first_run_state(), 0, "before any report nothing claims that no folder is open");
    crate::ui_screen::apply_update(&app, &Update::SourceRoot(None));
    settle();
    assert_eq!(app.get_first_run_title().as_str(), "No folder is open");
}

#[test]
fn toggling_the_folder_picker_of_an_empty_library_opens_nothing() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    crate::ui_screen::apply_update(&app, &Update::SourceRoot(None));
    app.invoke_toggle_picker();
    settle();
    assert!(!app.get_picker_open(), "an empty picker is never opened (EM5)");
}

#[test]
fn an_unplayable_file_shows_no_message_in_the_window_and_is_marked_on_its_row() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    let broken = PathBuf::from("/nonexistent-library").join(crate::ui_test_support::LIBRARY[2]);
    crate::ui_screen::apply_update(&app, &Update::Unplayable(vec![broken]));
    settle();
    // D122: the words about the file go to the desktop as a system notification (see unplayable_notification_logic_tests.rs
    // for the notification request); the window draws no message of any kind for it.
    assert_eq!(feedback_texts(&app), Vec::<String>::new(), "no message is drawn in the window for an unplayable file");
    let marked: Vec<String> = app
        .get_track_rows()
        .iter()
        .filter(|row| return row.kind == 0 && row.supporting.as_str() == "Can't be played")
        .map(|row| return row.title.to_string())
        .collect();
    assert_eq!(marked.len(), 1, "exactly the failed row is marked: {marked:?}");
}

#[test]
fn a_removed_file_is_explained_in_a_message() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    let gone = PathBuf::from("/nonexistent-library/Elsewhere/gone.flac");
    crate::ui_screen::apply_update(&app, &Update::FileGone { path: gone.clone(), was_playing: true });
    assert_eq!(feedback_texts(&app), vec!["gone.flac was removed from the folder, so playback stopped.".to_string()]);
}

#[test]
fn a_missing_audio_output_is_explained_in_a_message() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    crate::ui_screen::apply_update(&app, &Update::AudioUnavailable { reason: "audio error: core connect: CreationFailed".to_string() });
    assert_eq!(feedback_texts(&app), vec![crate::ui_notices::audio_unavailable_message().to_string()]);
}

#[test]
fn a_reconcile_that_clears_the_selection_resets_the_elapsed_label() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    assert_eq!(app.get_position_text().as_str(), "1:40");
    crate::ui_screen::apply_update(
        &app,
        &Update::Reconciled { names: vec!["a.flac".to_string()], index: None, name: String::new(), duration: 0.0 },
    );
    settle();
    assert_eq!(app.get_position_text().as_str(), "0:00", "no stale elapsed time next to a zero length (PB2)");
}

#[test]
fn the_folder_on_screen_moves_to_its_neighbor_when_its_last_file_is_removed() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    let names: Vec<String> = ["A/1.flac", "B/1.flac", "C/1.flac"].iter().map(|name| return (*name).to_string()).collect();
    crate::ui_screen::apply_update(&app, &Update::Queue(names));
    crate::ui_screen::update_state(|state| {
        state.selected_key = "folder:B".to_string();
    });
    let after: Vec<String> = ["A/1.flac", "C/1.flac"].iter().map(|name| return (*name).to_string()).collect();
    crate::ui_screen::apply_update(&app, &Update::Reconciled { names: after, index: None, name: String::new(), duration: 0.0 });
    settle();
    assert_eq!(app.get_folder_title().as_str(), "C", "the folder now at B's place, not the first folder");
}

#[test]
fn play_with_nothing_selected_first_scopes_the_engine_to_the_folder_on_screen() {
    let (app, host) = fresh_window_at(850.0, 900.0);
    let names: Vec<String> = ["A/1.flac", "B/1.flac"].iter().map(|name| return (*name).to_string()).collect();
    crate::ui_screen::apply_update(&app, &Update::Queue(names));
    settle();
    host.clear();
    crate::ui_wire::toggle_play(host.as_ref());
    assert_eq!(host.sent_texts(), ["scope 1", "toggle"]);
}

#[test]
fn play_with_a_track_loaded_sends_only_the_toggle() {
    let (app, host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    host.clear();
    crate::ui_wire::toggle_play(host.as_ref());
    assert_eq!(host.sent_texts(), ["toggle"]);
}

#[test]
fn the_header_button_that_opens_a_folder_says_folder_not_dir() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    settle();
    let folder: Vec<_> = i_slint_backend_testing::ElementHandle::find_by_accessible_label(&app, "Open folder").collect();
    let dir: Vec<_> = i_slint_backend_testing::ElementHandle::find_by_accessible_label(&app, "Open dir").collect();
    assert!(!folder.is_empty(), "the button is named Open folder (EM4)");
    assert!(dir.is_empty(), "no control is named Open dir");
}

#[test]
fn a_file_marked_unplayable_loses_its_mark_when_a_measured_peak_arrives_for_it() {
    let (app, _host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    let path = PathBuf::from("/nonexistent-library").join(crate::ui_test_support::LIBRARY[2]);
    crate::ui_screen::apply_update(&app, &Update::Unplayable(vec![path.clone()]));
    settle();
    let marked = |window: &crate::AppWindow| {
        return window.get_track_rows().iter().filter(|row| return row.kind == 0 && row.supporting.as_str() == "Can't be played").count();
    };
    assert_eq!(marked(&app), 1);
    let measure = music_player::track_measure::TrackMeasure { duration_secs: Some(10.0), peak_dbtp: Some(-3.0), peak_exact: true };
    crate::ui_screen::apply_update(&app, &Update::Measures(vec![(path, measure)]));
    settle();
    assert_eq!(marked(&app), 0, "the replaced file decodes now, so the mark goes (D113)");
}
