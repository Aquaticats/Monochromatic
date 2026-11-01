//! Tests for the pure activation rule in `row_activation`.

use super::*;

#[test]
fn the_current_track_always_toggles() {
    assert_eq!(row_activation_for(false, true), RowActivation::TogglePlay);
    assert_eq!(row_activation_for(true, true), RowActivation::TogglePlay);
}

#[test]
fn another_track_is_only_selected_with_the_setting_off() {
    assert_eq!(row_activation_for(false, false), RowActivation::SelectOnly);
}

#[test]
fn another_track_starts_with_the_setting_on() {
    assert_eq!(row_activation_for(true, false), RowActivation::SelectAndPlay);
}

#[test]
fn the_supporting_text_names_the_current_choice() {
    assert_eq!(auto_play_on_select_supporting_text(false), "Select first, then Play");
    assert_eq!(auto_play_on_select_supporting_text(true), "Select and play at once");
    assert_eq!(AUTO_PLAY_ON_SELECT_TITLE, "Auto-play on select");
}
