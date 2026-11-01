//! Window tests of every D106 shortcut on the testing backend: real key events go through the window's key scope, the
//! classifier and the actions, and each test reads the effect.
//!
//! What: a library window with a playing track and a recording host; each test presses one shortcut (or a group of
//! related ones) and checks what reached the host or the window.
//! Why: D106 says every shortcut is covered by a test that presses the real keys. The end-to-end run does the same through
//! the nested compositor; these tests also cover F5, Menu and Shift+F10, which the compositor 0.3.0 cannot press.

use slint::{ComponentHandle, Model};

use crate::Focus;
use crate::Keys;
use crate::ui_screen::read_state;
use crate::ui_screen_view::Route;
use crate::ui_test_support::{
    advance, chord, focus_label, fresh_window_at, key_down, key_up, load_library, settle, tab_labels, tap,
};
use music_player::track_menu::TrackMenuAction;

/// What: `unfolded` builds the two-pane window with the library loaded.
fn unfolded() -> (crate::AppWindow, std::rc::Rc<crate::ui_test_support::RecordingHost>) {
    let (app, host) = fresh_window_at(850.0, 900.0);
    load_library(&app);
    return (app, host);
}

/// What: `folded` builds the cover window with the library loaded.
fn folded() -> (crate::AppWindow, std::rc::Rc<crate::ui_test_support::RecordingHost>) {
    let (app, host) = fresh_window_at(443.0, 900.0);
    load_library(&app);
    return (app, host);
}

// ---- Playback ----

#[test]
fn space_toggles_play_and_pause_on_the_surface_and_in_the_lists_and_on_the_seek_bar() {
    let (app, host) = unfolded();
    tap(&app, " ");
    assert_eq!(host.sent_texts(), ["toggle"], "on the surface");
    chord(&app, &["Alt"], "2");
    tap(&app, " ");
    chord(&app, &["Alt"], "1");
    tap(&app, " ");
    assert_eq!(host.sent_texts(), ["toggle", "toggle", "toggle"], "in the track list and the folder list");
}

#[test]
fn space_activates_a_focused_button_and_does_not_toggle_playback() {
    let (app, host) = unfolded();
    tab_labels(&app, 5);
    assert_eq!(focus_label(&app), "previous");
    tap(&app, " ");
    assert_eq!(host.sent_texts(), ["prev"], "Space pressed the focused Previous button");
    tab_labels(&app, 4);
    assert_eq!(focus_label(&app), "search");
    tap(&app, " ");
    assert_eq!(app.get_route(), Route::Search.to_int(), "Space pressed the focused Search button");
}

#[test]
fn enter_activates_a_focused_button() {
    let (app, host) = unfolded();
    tab_labels(&app, 7);
    assert_eq!(focus_label(&app), "next");
    tap(&app, "Enter");
    assert_eq!(host.sent_texts(), ["next"]);
}

#[test]
fn ctrl_left_and_ctrl_right_go_to_the_previous_and_next_track() {
    let (app, host) = unfolded();
    chord(&app, &["Ctrl"], "Left");
    chord(&app, &["Ctrl"], "Right");
    assert_eq!(host.sent_texts(), ["prev", "next"]);
}

#[test]
fn ctrl_m_cycles_the_mode_and_a_held_ctrl_m_cycles_once() {
    let (app, host) = unfolded();
    let before = read_state(|state| return state.mode);
    let expected = format!("mode {:?}", crate::ui_keys_actions::next_mode(before));
    key_down(&app, "Ctrl");
    key_down(&app, "m");
    app.window().dispatch_event(slint::platform::WindowEvent::KeyPressRepeated { text: "m".into() });
    key_up(&app, "m");
    key_up(&app, "Ctrl");
    let sent = host.sent_texts();
    assert_eq!(sent.last().map(String::as_str), Some(expected.as_str()));
    assert_eq!(sent.len(), 2, "one scope and one mode, nothing for the repeat: {sent:?}");
}

#[test]
fn alt_arrows_seek_ten_seconds_and_with_shift_sixty() {
    let (app, host) = unfolded();
    chord(&app, &["Alt"], "Left");
    chord(&app, &["Alt"], "Right");
    chord(&app, &["Alt", "Shift"], "Left");
    chord(&app, &["Alt", "Shift"], "Right");
    assert_eq!(host.sent_texts(), ["seek 90", "seek 110", "seek 40", "seek 160"]);
}

#[test]
fn a_held_alt_right_keeps_seeking() {
    let (app, host) = unfolded();
    key_down(&app, "Alt");
    key_down(&app, "Right");
    app.window().dispatch_event(slint::platform::WindowEvent::KeyPressRepeated { text: slint::platform::Key::RightArrow.into() });
    app.window().dispatch_event(slint::platform::WindowEvent::KeyPressRepeated { text: slint::platform::Key::RightArrow.into() });
    key_up(&app, "Right");
    key_up(&app, "Alt");
    assert_eq!(host.sent_texts(), ["seek 110", "seek 110", "seek 110"]);
}

#[test]
fn a_seek_never_leaves_the_track() {
    let (app, host) = unfolded();
    crate::ui_screen::apply_update(&app, &music_player::command::Update::Position(5.0));
    chord(&app, &["Alt"], "Left");
    crate::ui_screen::apply_update(&app, &music_player::command::Update::Position(195.0));
    chord(&app, &["Alt", "Shift"], "Right");
    assert_eq!(host.sent_texts(), ["seek 0", "seek 200"]);
}

#[test]
fn ctrl_shift_1_to_4_pick_a_mode_directly() {
    let (app, host) = unfolded();
    for (shifted, plain, expected) in [("!", "1", "Repeat"), ("@", "2", "InOrder"), ("#", "3", "ShufflePage"), ("$", "4", "ShuffleAll")] {
        host.clear();
        chord(&app, &["Ctrl", "Shift"], shifted);
        assert_eq!(host.sent_texts(), ["scope 3".to_owned(), format!("mode {expected}")], "Ctrl+Shift+{plain} as {shifted}");
    }
    host.clear();
    chord(&app, &["Ctrl", "Shift"], "1");
    assert_eq!(host.sent_texts().last().map(String::as_str), Some("mode Repeat"), "a layout that keeps the digit");
}

#[test]
fn ctrl_home_restarts_the_track() {
    let (app, host) = unfolded();
    chord(&app, &["Ctrl"], "Home");
    assert_eq!(host.sent_texts(), ["seek 0"]);
}

#[test]
fn a_focused_seek_bar_moves_with_the_arrows_and_the_mode_group_with_the_arrows() {
    let (app, host) = unfolded();
    tab_labels(&app, 4);
    assert_eq!(focus_label(&app), "seek");
    tap(&app, "Left");
    tap(&app, "Right");
    tap(&app, "Home");
    tap(&app, "End");
    assert_eq!(host.sent_texts(), ["seek 95", "seek 105", "seek 0", "seek 200"], "the slider's own arrows, 5 seconds");
    host.clear();
    tab_labels(&app, 4);
    assert_eq!(focus_label(&app), "mode-group");
    let mode = read_state(|state| return state.mode);
    crate::ui_screen::apply_update(&app, &music_player::command::Update::PlaybackMode(music_player::command::PlaybackMode::InOrder));
    tap(&app, "Right");
    tap(&app, "Left");
    let sent = host.sent_texts();
    assert!(sent.iter().any(|text| return text == "mode ShufflePage"), "Right selects the next mode: {sent:?} (was {mode:?})");
    assert!(sent.iter().any(|text| return text == "mode Repeat"), "Left selects the previous mode: {sent:?}");
}

// ---- Browsing ----

#[test]
fn alt_down_and_alt_up_show_the_next_and_previous_folder() {
    let (app, host) = unfolded();
    assert_eq!(app.get_folder_title(), "Abba");
    chord(&app, &["Alt"], "Down");
    assert_eq!(app.get_folder_title(), "Beck");
    chord(&app, &["Alt"], "Down");
    assert_eq!(app.get_folder_title(), "Cher");
    chord(&app, &["Alt"], "Up");
    assert_eq!(app.get_folder_title(), "Beck");
    assert_eq!(host.sent_texts(), ["scope 2", "scope 2", "scope 2"]);
    chord(&app, &["Alt"], "Up");
    chord(&app, &["Alt"], "Up");
    assert_eq!(app.get_folder_title(), "Abba", "the first folder stays the first");
    assert_eq!(host.sent_texts().len(), 4, "no scope for a step that goes nowhere");
}

#[test]
fn ctrl_shift_g_and_ctrl_g_show_the_playing_tracks_folder() {
    let (app, _host) = unfolded();
    chord(&app, &["Alt"], "Down");
    assert_eq!(app.get_folder_title(), "Beck");
    chord(&app, &["Ctrl", "Shift"], "G");
    assert_eq!(app.get_folder_title(), "Abba", "Ctrl+Shift+G");
    chord(&app, &["Alt"], "Down");
    chord(&app, &["Ctrl"], "g");
    assert_eq!(app.get_folder_title(), "Abba", "Ctrl+G");
}

#[test]
fn ctrl_shift_o_asks_for_the_system_folder_dialog() {
    let (app, host) = unfolded();
    chord(&app, &["Ctrl", "Shift"], "O");
    assert_eq!(host.dialogs.get(), 1);
}

#[test]
fn f5_and_ctrl_shift_f5_do_nothing_because_the_library_refreshes_by_itself() {
    let (app, host) = unfolded();
    tap(&app, "F5");
    chord(&app, &["Ctrl", "Shift"], "F5");
    chord(&app, &["Alt"], "2");
    tap(&app, "Down");
    tap(&app, "F5");
    chord(&app, &["Ctrl", "Shift"], "F5");
    assert!(host.sent_texts().iter().all(|text| return !text.contains("rescan") && !text.contains("reanalyse")), "{:?}", host.sent_texts());
    assert!(host.row_actions.borrow().is_empty(), "no row action ran");
}

#[test]
fn ctrl_q_quits() {
    let (app, host) = unfolded();
    chord(&app, &["Ctrl"], "q");
    assert_eq!(host.quits.get(), 1);
}

#[test]
fn ctrl_q_quits_from_every_page() {
    let (app, host) = unfolded();
    chord(&app, &["Ctrl", "Alt"], "s");
    chord(&app, &["Ctrl"], "q");
    assert_eq!(host.quits.get(), 1, "from Settings");
    tap(&app, "Escape");
    chord(&app, &["Ctrl"], "f");
    chord(&app, &["Ctrl"], "q");
    assert_eq!(host.quits.get(), 2, "from Search");
}

#[test]
fn ctrl_f_ctrl_alt_s_and_ctrl_o_open_search_settings_and_the_picker() {
    let (app, _host) = folded();
    chord(&app, &["Ctrl"], "o");
    assert!(read_state(|state| return state.picker_open), "Ctrl+O opens the picker on the folded layout");
    chord(&app, &["Ctrl"], "o");
    assert!(!read_state(|state| return state.picker_open), "and closes it again");
    chord(&app, &["Ctrl"], "f");
    assert_eq!(app.get_route(), Route::Search.to_int());
    tap(&app, "Escape");
    chord(&app, &["Ctrl", "Alt"], "s");
    assert_eq!(app.get_route(), Route::Settings.to_int());
}

#[test]
fn ctrl_o_focuses_the_folder_list_on_the_unfolded_layout() {
    let (app, _host) = unfolded();
    chord(&app, &["Ctrl"], "o");
    assert_eq!(focus_label(&app), "folder-list");
}

// ---- Lists ----

#[test]
fn the_track_list_ring_starts_on_the_playing_row_and_moves_with_the_list_keys() {
    let (app, host) = unfolded();
    chord(&app, &["Alt"], "2");
    assert_eq!(read_state(|state| return state.ring_track), Some(1), "the ring starts on the playing track");
    tap(&app, "Down");
    assert_eq!(read_state(|state| return state.ring_track), Some(2));
    tap(&app, "Down");
    assert_eq!(read_state(|state| return state.ring_track), Some(2), "the last row of the page stops the ring");
    tap(&app, "Up");
    assert_eq!(read_state(|state| return state.ring_track), Some(1));
    tap(&app, "Home");
    assert_eq!(read_state(|state| return state.ring_track), Some(0));
    tap(&app, "End");
    assert_eq!(read_state(|state| return state.ring_track), Some(2));
    tap(&app, "PageUp");
    assert_eq!(read_state(|state| return state.ring_track), Some(0), "a page up goes to the first row");
    tap(&app, "PageDown");
    assert_eq!(read_state(|state| return state.ring_track), Some(2), "a page down goes to the last row");
    assert!(host.sent_texts().is_empty(), "moving the ring plays nothing");
}

#[test]
fn enter_selects_the_row_under_the_ring_and_on_the_selected_row_it_toggles_play() {
    let (app, host) = unfolded();
    chord(&app, &["Alt"], "2");
    tap(&app, "Down");
    tap(&app, "Enter");
    assert_eq!(host.sent_texts(), ["select 2"], "Enter on another row only selects it (D124)");
    host.clear();
    tap(&app, "Up");
    tap(&app, "Enter");
    assert_eq!(host.sent_texts(), ["toggle"], "row 1 is the selected row");
}

#[test]
fn the_folder_ring_starts_on_the_current_folder_and_letters_jump() {
    let (app, host) = unfolded();
    chord(&app, &["Alt"], "1");
    assert_eq!(read_state(|state| return state.ring_folder.clone()), Some("Abba".to_owned()));
    tap(&app, "c");
    assert_eq!(read_state(|state| return state.ring_folder.clone()), Some("Cher".to_owned()));
    tap(&app, "Z");
    assert_eq!(read_state(|state| return state.ring_folder.clone()), Some("Zed".to_owned()));
    tap(&app, "Up");
    assert_eq!(read_state(|state| return state.ring_folder.clone()), Some("Cher".to_owned()));
    tap(&app, "Home");
    assert_eq!(read_state(|state| return state.ring_folder.clone()), Some("Abba".to_owned()));
    tap(&app, "End");
    assert_eq!(read_state(|state| return state.ring_folder.clone()), Some("Zed".to_owned()));
    tap(&app, "q");
    assert_eq!(read_state(|state| return state.ring_folder.clone()), Some("Zed".to_owned()), "a letter nobody starts with changes nothing");
    assert!(host.sent_texts().is_empty(), "no letter played anything");
}

#[test]
fn enter_in_the_folder_list_shows_the_folder_under_the_ring() {
    let (app, host) = unfolded();
    chord(&app, &["Alt"], "1");
    tap(&app, "c");
    tap(&app, "Enter");
    assert_eq!(app.get_folder_title(), "Cher");
    assert_eq!(host.sent_texts(), ["scope 2"]);
    assert_eq!(focus_label(&app), "folder-list", "the keyboard stays in the folder list on the two-pane layout");
}

#[test]
fn enter_in_the_folded_pickers_list_shows_the_folder_and_moves_focus_to_the_track_list() {
    let (app, _host) = folded();
    chord(&app, &["Alt"], "1");
    advance(32);
    tap(&app, "b");
    tap(&app, "Enter");
    advance(32);
    assert_eq!(app.get_folder_title(), "Beck");
    assert!(!read_state(|state| return state.picker_open), "the picker closed");
    assert_eq!(focus_label(&app), "track-list");
}

#[test]
fn escape_closes_the_folded_picker_and_focuses_the_track_list() {
    let (app, _host) = folded();
    chord(&app, &["Ctrl"], "o");
    advance(32);
    assert!(read_state(|state| return state.picker_open));
    tap(&app, "Escape");
    advance(32);
    assert!(!read_state(|state| return state.picker_open));
    assert_eq!(focus_label(&app), "track-list");
    assert_eq!(app.get_route(), Route::Player.to_int(), "the page did not change");
}

// ---- Track rows ----

#[test]
fn the_row_chords_act_on_the_row_under_the_ring() {
    let (app, host) = unfolded();
    chord(&app, &["Alt"], "2");
    tap(&app, "Down");
    tap(&app, "Delete");
    chord(&app, &["Ctrl", "Shift"], "C");
    chord(&app, &["Ctrl", "Shift"], "E");
    assert_eq!(
        *host.row_actions.borrow(),
        [TrackMenuAction::MoveToTrash, TrackMenuAction::CopyFilename, TrackMenuAction::ShowInFileManager]
    );
    assert_eq!(read_state(|state| return state.menu_index), Some(2), "they all named row 2");
}

#[test]
fn the_row_chords_do_nothing_outside_the_track_list() {
    let (app, host) = unfolded();
    tap(&app, "Delete");
    chord(&app, &["Ctrl", "Shift"], "C");
    chord(&app, &["Alt"], "1");
    tap(&app, "Delete");
    assert!(host.row_actions.borrow().is_empty());
}

#[test]
fn the_menu_key_and_alt_enter_open_the_menu_of_the_ringed_row_and_shift_f10_does_not() {
    for open in [(&[][..], "Menu"), (&["Alt"][..], "Enter")] {
        let (app, _host) = unfolded();
        chord(&app, &["Alt"], "2");
        tap(&app, "Down");
        chord(&app, open.0, open.1);
        advance(32);
        assert!(app.get_menu_open(), "{open:?} opened the menu");
        assert_eq!(read_state(|state| return state.menu_index), Some(2));
        assert_eq!(app.get_menu_ring(), 0, "it opens with its first row highlighted");
    }
    let (app, _host) = unfolded();
    chord(&app, &["Alt"], "2");
    tap(&app, "Down");
    chord(&app, &["Shift"], "F10");
    advance(32);
    assert!(!app.get_menu_open(), "Shift+F10 is Run in JetBrains and opens nothing (D114)");
}

#[test]
fn an_access_letter_runs_its_row_and_closes_the_menu() {
    for action in music_player::track_menu::track_menu_actions_for() {
        let (app, host) = unfolded();
        chord(&app, &["Alt"], "2");
        tap(&app, "Menu");
        advance(32);
        assert!(app.get_menu_open());
        let letter = music_player::track_menu::access_key(*action).to_string();
        tap(&app, &letter);
        assert_eq!(*host.row_actions.borrow(), [*action], "{letter} ran {action:?}");
        assert!(!app.get_menu_open(), "{letter} closed the menu");
    }
}

#[test]
fn a_letter_that_is_not_an_access_key_leaves_the_menu_open_and_runs_nothing() {
    let (app, host) = unfolded();
    chord(&app, &["Alt"], "2");
    tap(&app, "Menu");
    advance(32);
    tap(&app, "z");
    chord(&app, &["Ctrl"], "p");
    assert!(app.get_menu_open());
    assert!(host.row_actions.borrow().is_empty());
}

#[test]
fn each_menu_row_underlines_its_access_letter_in_its_label() {
    let (app, _host) = unfolded();
    chord(&app, &["Alt"], "2");
    tap(&app, "Menu");
    advance(32);
    let items = app.get_menu_items();
    assert_eq!(items.row_count(), music_player::track_menu::track_menu_actions_for().len());
    for (item, action) in items.iter().zip(music_player::track_menu::track_menu_actions_for()) {
        let (before, access, after) = music_player::track_menu::access_label(*action);
        assert_eq!(item.label_before.as_str(), before);
        assert_eq!(item.access.as_str(), access);
        assert_eq!(item.label_after.as_str(), after);
        assert_eq!(format!("{}{}{}", item.label_before, item.access, item.label_after), item.label.as_str(), "the parts join into the label");
    }
}

#[test]
fn up_down_enter_and_escape_work_the_open_menu() {
    let (app, host) = unfolded();
    chord(&app, &["Alt"], "2");
    tap(&app, "Menu");
    advance(32);
    assert!(app.get_menu_open());
    tap(&app, "Down");
    assert_eq!(app.get_menu_ring(), 1);
    tap(&app, "Down");
    tap(&app, "Down");
    assert_eq!(app.get_menu_ring(), 3);
    tap(&app, "Up");
    assert_eq!(app.get_menu_ring(), 2);
    tap(&app, "Escape");
    assert!(!app.get_menu_open(), "Escape closes the menu");
    assert_eq!(app.get_route(), Route::Player.to_int(), "and only the menu");
    tap(&app, "Menu");
    advance(32);
    tap(&app, "Down");
    tap(&app, "Enter");
    assert_eq!(*host.row_actions.borrow(), [TrackMenuAction::ShuffleFromHere], "Enter ran the highlighted row");
    assert!(!app.get_menu_open(), "and closed the menu");
}

#[test]
fn the_row_chords_work_while_the_menu_is_open_and_close_it() {
    let (app, host) = unfolded();
    chord(&app, &["Alt"], "2");
    tap(&app, "Menu");
    advance(32);
    tap(&app, "Delete");
    assert_eq!(*host.row_actions.borrow(), [TrackMenuAction::MoveToTrash]);
    assert!(!app.get_menu_open());
}

#[test]
fn every_menu_row_shows_its_shortcut_right_aligned_in_the_row() {
    let (app, _host) = unfolded();
    chord(&app, &["Alt"], "2");
    tap(&app, "Menu");
    advance(32);
    let shortcuts: Vec<String> = app.get_menu_items().iter().map(|item| return item.shortcut.to_string()).collect();
    assert_eq!(shortcuts, ["Enter", "", "", "Ctrl+Shift+E", "Ctrl+Shift+C", "Delete"]);
}

// ---- Tooltips and badges ----

#[test]
fn the_tooltips_name_the_shortcut_of_every_button_that_has_one() {
    let (app, _host) = unfolded();
    let texts = app.global::<Keys>().get_texts();
    assert_eq!(texts.search, "Ctrl+F");
    assert_eq!(texts.settings, "Ctrl+Alt+S");
    assert_eq!(texts.open_folder, "Ctrl+Shift+O");
    assert_eq!(texts.folder_trigger, "Ctrl+O");
    assert_eq!(texts.previous, "Ctrl+Left");
    assert_eq!(texts.play, "Space");
    assert_eq!(texts.next, "Ctrl+Right");
    assert_eq!(texts.seek_back, "Alt+Left");
    assert_eq!(texts.seek_forward, "Alt+Right");
    assert_eq!(texts.mode_repeat, "Ctrl+Shift+1");
    assert_eq!(texts.mode_shuffle_all, "Ctrl+Shift+4");
    assert_eq!(texts.mode_cycle, "Ctrl+M");
    assert_eq!(texts.back, "Escape");
    assert_eq!(texts.editor_reset, "Ctrl+Shift+Backspace");
}

#[test]
fn badges_appear_after_300_ms_of_held_control_and_vanish_on_release() {
    let (app, _host) = unfolded();
    let badges = || return app.global::<Keys>().get_badges();
    key_down(&app, "Ctrl");
    advance(250);
    assert_eq!(badges().search, "", "not yet at 250 ms");
    advance(100);
    assert_eq!(badges().search, "F", "shown after 300 ms");
    assert_eq!(badges().folder_trigger, "O");
    assert_eq!(badges().previous, "Left");
    assert_eq!(badges().open_folder, "", "the Shift chords wait for Shift");
    key_up(&app, "Ctrl");
    assert_eq!(badges().search, "", "gone on release");
    assert_eq!(badges().folder_trigger, "");
}

#[test]
fn a_quick_chord_never_flashes_the_badges() {
    let (app, _host) = unfolded();
    key_down(&app, "Ctrl");
    advance(100);
    tap(&app, "f");
    key_up(&app, "Ctrl");
    advance(500);
    assert_eq!(app.global::<Keys>().get_badges().search, "");
}

#[test]
fn adding_shift_narrows_the_badges_to_the_shift_chords() {
    let (app, _host) = unfolded();
    let badges = || return app.global::<Keys>().get_badges();
    key_down(&app, "Ctrl");
    advance(350);
    assert_eq!(badges().search, "F");
    key_down(&app, "Shift");
    assert_eq!(badges().search, "", "the plain chord's badge left");
    assert_eq!(badges().open_folder, "O");
    assert_eq!(badges().deck_folder, "G", "Ctrl+Shift+G shows the playing folder");
    assert_eq!(badges().mode_repeat, "1");
    assert_eq!(badges().mode_shuffle_all, "4");
    key_up(&app, "Shift");
    assert_eq!(badges().search, "F", "releasing Shift brings the Control badges back");
    assert_eq!(badges().mode_repeat, "");
    key_up(&app, "Ctrl");
    assert_eq!(badges().search, "");
}

#[test]
fn alt_held_shows_the_seek_and_focus_badges() {
    let (app, _host) = unfolded();
    key_down(&app, "Alt");
    advance(350);
    let badges = app.global::<Keys>().get_badges();
    assert_eq!(badges.seek_back, "Left");
    assert_eq!(badges.seek_forward, "Right");
    assert_eq!(badges.folder_list, "1");
    assert_eq!(badges.track_list, "2");
    key_up(&app, "Alt");
}

#[test]
fn badges_never_take_pointer_input_and_never_move_other_content() {
    let (app, _host) = unfolded();
    let before: Vec<(f32, f32, f32, f32)> = ["TrackListPane", "FolderPickerPane", "TransportDeck", "HeaderActions"]
        .iter()
        .flat_map(|name| return i_slint_backend_testing::ElementHandle::find_by_element_type_name(&app, name))
        .map(|element| {
            let position = element.absolute_position();
            let size = element.size();
            return (position.x, position.y, size.width, size.height);
        })
        .collect();
    key_down(&app, "Ctrl");
    advance(350);
    key_down(&app, "Shift");
    let after: Vec<(f32, f32, f32, f32)> = ["TrackListPane", "FolderPickerPane", "TransportDeck", "HeaderActions"]
        .iter()
        .flat_map(|name| return i_slint_backend_testing::ElementHandle::find_by_element_type_name(&app, name))
        .map(|element| {
            let position = element.absolute_position();
            let size = element.size();
            return (position.x, position.y, size.width, size.height);
        })
        .collect();
    assert_eq!(before, after, "no element moved or resized while badges showed");
    let source = include_str!("../ui/focus.slint");
    let badge = &source[source.find("export component KeyBadge").expect("the badge exists")..];
    assert!(!badge.contains("TouchArea") && !badge.contains("FocusScope"), "the badge component has no input handler");
    key_up(&app, "Shift");
    key_up(&app, "Ctrl");
}

// ---- Double Shift ----

/// What: `tap_shift` presses and releases Shift.
fn tap_shift(app: &crate::AppWindow) {
    key_down(app, "Shift");
    advance(40);
    key_up(app, "Shift");
}

#[test]
fn two_quick_shift_taps_open_search_on_the_player_page() {
    let (app, _host) = unfolded();
    tap_shift(&app);
    advance(100);
    tap_shift(&app);
    assert_eq!(app.get_route(), Route::Search.to_int());
}

#[test]
fn two_slow_shift_taps_do_not() {
    let (app, _host) = unfolded();
    tap_shift(&app);
    advance(600);
    tap_shift(&app);
    assert_eq!(app.get_route(), Route::Player.to_int());
}

#[test]
fn a_key_between_the_shift_taps_cancels_the_double_tap() {
    let (app, host) = unfolded();
    tap_shift(&app);
    advance(30);
    chord(&app, &["Ctrl"], "m");
    advance(30);
    tap_shift(&app);
    assert_eq!(app.get_route(), Route::Player.to_int());
    assert!(host.sent_texts().iter().any(|text| return text.starts_with("mode")), "the key between the taps was used: {:?}", host.sent_texts());
}

#[test]
fn shift_held_with_another_key_is_not_a_tap() {
    let (app, _host) = unfolded();
    key_down(&app, "Shift");
    tap(&app, "a");
    key_up(&app, "Shift");
    advance(50);
    tap_shift(&app);
    assert_eq!(app.get_route(), Route::Player.to_int());
}

#[test]
fn a_held_shift_with_repeats_is_not_a_double_tap() {
    let (app, _host) = unfolded();
    key_down(&app, "Shift");
    app.window().dispatch_event(slint::platform::WindowEvent::KeyPressRepeated { text: slint::platform::Key::Shift.into() });
    app.window().dispatch_event(slint::platform::WindowEvent::KeyPressRepeated { text: slint::platform::Key::Shift.into() });
    key_up(&app, "Shift");
    assert_eq!(app.get_route(), Route::Player.to_int());
}

#[test]
fn double_shift_does_nothing_in_settings_and_in_the_editor_page() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    load_library(&app);
    chord(&app, &["Ctrl", "Alt"], "s");
    assert_eq!(app.get_route(), Route::Settings.to_int());
    tap_shift(&app);
    advance(100);
    tap_shift(&app);
    assert_eq!(app.get_route(), Route::Settings.to_int(), "Settings stays");
    tap(&app, "Enter");
    assert_eq!(app.get_route(), Route::TemplateEditor.to_int());
    tap_shift(&app);
    advance(100);
    tap_shift(&app);
    assert_eq!(app.get_route(), Route::TemplateEditor.to_int(), "the editor page stays");
}

#[test]
fn ctrl_f_does_nothing_in_settings_and_in_the_editor_page() {
    let (app, _host) = unfolded();
    chord(&app, &["Ctrl", "Alt"], "s");
    chord(&app, &["Ctrl"], "f");
    assert_eq!(app.get_route(), Route::Settings.to_int());
    let (narrow, _narrow_host) = fresh_window_at(443.0, 900.0);
    load_library(&narrow);
    chord(&narrow, &["Ctrl", "Alt"], "s");
    tap(&narrow, "Enter");
    assert_eq!(narrow.get_route(), Route::TemplateEditor.to_int());
    chord(&narrow, &["Ctrl"], "f");
    assert_eq!(narrow.get_route(), Route::TemplateEditor.to_int());
}

#[test]
fn ctrl_alt_s_in_settings_does_not_rebuild_the_editor() {
    let (app, _host) = unfolded();
    chord(&app, &["Ctrl", "Alt"], "s");
    app.invoke_editor_text_edited("Edited".into(), 6, 6);
    settle();
    let before = read_state(|state| return state.editor.clone());
    chord(&app, &["Ctrl", "Alt"], "s");
    assert_eq!(read_state(|state| return state.editor.clone()), before, "the draft survived");
}

#[test]
fn the_settings_editor_chords_apply_and_reset_in_the_two_pane_page() {
    let (app, _host) = unfolded();
    chord(&app, &["Ctrl", "Alt"], "s");
    app.invoke_editor_text_edited("Plain".into(), 5, 5);
    settle();
    assert!(read_state(|state| return state.editor.as_ref().is_some_and(|editor| return editor.can_reset())));
    chord(&app, &["Ctrl", "Shift"], "Backspace");
    assert!(read_state(|state| return state.editor.as_ref().is_some_and(|editor| return !editor.can_reset())), "reset to the default");
    app.invoke_editor_text_edited("Plain again".into(), 11, 11);
    settle();
    chord(&app, &["Ctrl"], "Enter");
    assert_eq!(app.get_route(), Route::Settings.to_int(), "Ctrl+Enter applies and leaves the editor, not Settings");
    assert_eq!(focus_label(&app), "settings-list", "focus returned to the list");
}

#[test]
fn escape_on_the_editor_page_returns_to_the_settings_list_and_ctrl_enter_does_too() {
    for leave in [(&[][..], "Escape"), (&["Ctrl"][..], "Enter")] {
        let (app, _host) = fresh_window_at(443.0, 900.0);
        load_library(&app);
        chord(&app, &["Ctrl", "Alt"], "s");
        tap(&app, "Enter");
        assert_eq!(app.get_route(), Route::TemplateEditor.to_int());
        chord(&app, leave.0, leave.1);
        assert_eq!(app.get_route(), Route::Settings.to_int(), "{leave:?}");
    }
}

#[test]
fn ctrl_shift_backspace_resets_the_template_on_the_editor_page() {
    let (app, _host) = fresh_window_at(443.0, 900.0);
    load_library(&app);
    chord(&app, &["Ctrl", "Alt"], "s");
    tap(&app, "Enter");
    app.invoke_editor_text_edited("Plain".into(), 5, 5);
    settle();
    assert!(read_state(|state| return state.editor.as_ref().is_some_and(|editor| return editor.can_reset())));
    chord(&app, &["Ctrl", "Shift"], "Backspace");
    assert!(read_state(|state| return state.editor.as_ref().is_some_and(|editor| return !editor.can_reset())));
}

#[test]
fn the_search_page_moves_through_results_opens_one_and_escape_leaves() {
    let (app, host) = unfolded();
    chord(&app, &["Ctrl"], "f");
    assert_eq!(app.get_route(), Route::Search.to_int());
    seed_results(&app, 3);
    assert_eq!(app.get_search_ring(), 0);
    tap(&app, "Down");
    assert_eq!(app.get_search_ring(), 1);
    tap(&app, "Down");
    tap(&app, "Down");
    assert_eq!(app.get_search_ring(), 2, "the last result stops the highlight");
    tap(&app, "Up");
    assert_eq!(app.get_search_ring(), 1);
    tap(&app, "Enter");
    assert_eq!(app.get_route(), Route::Player.to_int(), "Enter opened the result and left Search");
    assert_eq!(host.sent_texts(), ["select 4"], "D127: a result that is not the current track is only selected, nothing plays");
    chord(&app, &["Ctrl"], "f");
    tap(&app, "Escape");
    assert_eq!(app.get_route(), Route::Player.to_int(), "Escape leaves Search");
}

#[test]
fn bare_keys_do_nothing_while_the_search_field_has_focus_and_typing_still_works() {
    let (app, host) = unfolded();
    chord(&app, &["Ctrl"], "f");
    assert_eq!(app.global::<Focus>().get_zone(), 7);
    for key in [" ", "a", "F5", "Delete", "Menu", "Home", "End", "Left", "Right"] {
        tap(&app, key);
    }
    chord(&app, &["Ctrl"], "Left");
    chord(&app, &["Ctrl"], "m");
    chord(&app, &["Ctrl"], "Home");
    assert!(host.sent_texts().is_empty(), "nothing reached the engine: {:?}", host.sent_texts());
    assert!(host.row_actions.borrow().is_empty());
    assert_eq!(app.get_route(), Route::Search.to_int());
    assert!(read_state(|state| return state.search_query.clone()).contains('a'), "the field still received the typing");
}

#[test]
fn bare_keys_do_nothing_while_the_template_line_has_focus() {
    let (app, host) = unfolded();
    chord(&app, &["Ctrl", "Alt"], "s");
    let field = i_slint_backend_testing::ElementHandle::find_by_element_id(&app, "TemplateEditor::field-input")
        .next()
        .expect("the template line exists");
    let position = field.absolute_position();
    let size = field.size();
    let center = slint::LogicalPosition::new(position.x + size.width / 2.0, position.y + size.height / 2.0);
    app.window().dispatch_event(slint::platform::WindowEvent::PointerMoved { position: center });
    app.window().dispatch_event(slint::platform::WindowEvent::PointerPressed { position: center, button: slint::platform::PointerEventButton::Left });
    app.window().dispatch_event(slint::platform::WindowEvent::PointerReleased { position: center, button: slint::platform::PointerEventButton::Left });
    settle();
    assert_eq!(app.global::<Focus>().get_zone(), 7);
    for key in [" ", "x", "F5", "Delete", "Home"] {
        tap(&app, key);
    }
    chord(&app, &["Ctrl"], "Left");
    assert!(host.sent_texts().is_empty(), "nothing reached the engine: {:?}", host.sent_texts());
    assert_eq!(app.get_route(), Route::Settings.to_int());
}

/// What: `seed_results` puts `count` track results on the Search page, as an answer from the Search service would.
/// Why: the service answers on a worker thread; the test needs the rows at once.
fn seed_results(app: &crate::AppWindow, count: usize) {
    use music_player::search_results::{SearchResult, SearchResultKind};
    let results: Vec<SearchResult> = (0..count)
        .map(|position| {
            return SearchResult {
                kind: SearchResultKind::Track,
                name: crate::ui_test_support::LIBRARY[position + 3].to_owned(),
                parent_context: None,
                index_in_library: Some(position + 3),
                folder_name: "Beck".to_owned(),
                tier: 0,
                name_highlights: Vec::new(),
                parent_highlights: Vec::new(),
            };
        })
        .collect();
    let rows: Vec<crate::ui_search_model::SearchRowView> = (0..count)
        .map(|position| {
            return crate::ui_search_model::SearchRowView {
                kind: 1,
                title: Vec::new(),
                parent: Vec::new(),
                has_parent: false,
                current: false,
                playing: false,
                spoken: String::new(),
                index: i32::try_from(position).unwrap_or(0),
            };
        })
        .collect();
    let view = crate::ui_search_model::SearchView {
        rows,
        results,
        status_title: String::new(),
        status_detail: String::new(),
    };
    crate::ui_search_apply::show_view(app, view, "b", false);
    settle();
}
