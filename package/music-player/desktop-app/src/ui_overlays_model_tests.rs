//! Tests for the pure first-run and feedback models in `ui_overlays_model`.

use super::*;
use music_player::feedback::FeedbackMessage;

#[test]
fn first_run_is_hidden_before_the_library_loads() {
    assert_eq!(first_run_view(&ScreenState::default()), None);
}

#[test]
fn first_run_is_hidden_when_the_library_has_tracks() {
    let state = ScreenState {
        loaded: true,
        names: std::sync::Arc::new(vec!["Abba/one.mp3".to_string()]),
        ..ScreenState::default()
    };
    assert_eq!(first_run_view(&state), None);
}

#[test]
fn first_run_shows_open_folder_and_settings_when_no_folder_is_open() {
    let state = ScreenState {
        root_known: true,
        ..ScreenState::default()
    };
    let view = first_run_view(&state).expect("a start with no folder open shows the first-run pane");
    assert_eq!(view.title, "No folder is open");
    assert_eq!(view.primary_label, "Open a folder");
    assert_eq!(view.secondary_labels, vec!["Settings".to_string()]);
    assert!(view.explanation.is_empty());
}

#[test]
fn first_run_names_the_opened_folder_when_it_has_no_audio() {
    let state = ScreenState {
        root_known: true,
        loaded: true,
        source_root_name: Some("Albums".to_string()),
        ..ScreenState::default()
    };
    let view = first_run_view(&state).expect("an opened folder with no audio shows the first-run pane");
    assert_eq!(view.title, "No audio found in\nAlbums");
    assert_eq!(view.primary_label, "Open a folder");
}

#[test]
fn first_run_says_the_library_is_being_read_not_that_no_folder_is_open() {
    let state = ScreenState {
        root_known: true,
        reading: true,
        source_root_name: Some("Albums".to_string()),
        ..ScreenState::default()
    };
    let view = first_run_view(&state).expect("a folder being read shows the reading pane");
    assert_eq!(view.title, "Reading your library");
    assert!(view.primary_label.is_empty());
}

#[test]
fn reading_wins_over_an_earlier_empty_library_and_over_an_old_list() {
    let earlier_empty = ScreenState { root_known: true, loaded: true, reading: true, source_root_name: Some("Old".to_string()), ..ScreenState::default() };
    assert_eq!(first_run_view(&earlier_empty).map(|view| return view.title), Some("Reading your library".to_string()));
    let old_list = ScreenState {
        reading: true,
        names: std::sync::Arc::new(vec!["Abba/one.mp3".to_string()]),
        ..ScreenState::default()
    };
    assert_eq!(first_run_view(&old_list).map(|view| return view.title), Some("Reading your library".to_string()));
}

#[test]
fn the_deck_is_disabled_while_reading_and_without_tracks_only() {
    assert!(ScreenState { reading: true, ..ScreenState::default() }.deck_disabled());
    assert!(ScreenState::default().deck_disabled());
    let with_tracks = ScreenState { names: std::sync::Arc::new(vec!["a.mp3".to_string()]), ..ScreenState::default() };
    assert!(!with_tracks.deck_disabled());
}

#[test]
fn first_run_waits_for_the_opened_folder_to_load() {
    let state = ScreenState {
        root_known: true,
        loaded: false,
        source_root_name: Some("Albums".to_string()),
        ..ScreenState::default()
    };
    assert_eq!(first_run_view(&state), None);
}

#[test]
fn first_run_is_hidden_until_the_engine_reports_the_folder() {
    let state = ScreenState {
        loaded: true,
        ..ScreenState::default()
    };
    assert_eq!(first_run_view(&state), None);
}

#[test]
fn feedback_rows_keep_order_kind_and_action_label() {
    let queue = FeedbackQueue::EMPTY
        .show(
            FeedbackMessage::new("error-1", "Copy failed", FeedbackKind::Error, None).expect("valid message"),
            0,
        )
        .show(
            FeedbackMessage::new("undo-1", "Moved to trash", FeedbackKind::Undo, Some("Undo")).expect("valid message"),
            0,
        );
    let rows = feedback_rows(&queue);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, "undo-1");
    assert_eq!(rows[0].kind, 1);
    assert_eq!(rows[0].action_label, "Undo");
    assert_eq!(rows[1].id, "error-1");
    assert_eq!(rows[1].kind, 0);
    assert_eq!(rows[1].action_label, "");
}

#[test]
fn a_short_message_stays_four_seconds_and_a_message_with_an_action_stays_eight() {
    assert_eq!(visible_time_ms("Copied a.wav to the clipboard", Hold::Brief), 4_000);
    assert_eq!(visible_time_ms("Moved a.wav to the trash", Hold::Long), 8_000, "Undo needs time to be pressed");
}

#[test]
fn a_long_message_stays_in_proportion_to_its_length_up_to_fifteen_seconds() {
    let hundred = "x".repeat(100);
    assert_eq!(visible_time_ms(&hundred, Hold::Brief), 8_000, "2 s plus 60 ms for each of 100 characters");
    assert_eq!(visible_time_ms(&hundred, Hold::Long), 8_000, "the larger of the floor and the reading time");
    let long = "x".repeat(150);
    assert_eq!(visible_time_ms(&long, Hold::Brief), 11_000);
    let huge = "x".repeat(10_000);
    assert_eq!(visible_time_ms(&huge, Hold::Brief), 15_000, "never longer than 15 s");
    assert_eq!(visible_time_ms("", Hold::Brief), 4_000);
}

#[test]
fn reading_time_counts_characters_not_bytes() {
    let japanese = "\u{3042}".repeat(100);
    assert_eq!(visible_time_ms(&japanese, Hold::Brief), 8_000);
}
