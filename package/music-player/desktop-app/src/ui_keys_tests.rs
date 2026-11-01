//! Tests for the pure keyboard mapping in `ui_keys` and for the pure parts of `ui_keys_actions`.
//!
//! What: key names, the macOS modifier swap, the classifier request, the mode cycle and the seek target.
//! Why: the window tests press real key events; these pin the small pure mappings they rely on.

use music_player::command::PlaybackMode;
use music_player::keymap::ShortcutAction;

use super::*;
use crate::ui_keys_actions::{next_mode, seek_target};

/// What: `input(key)` builds a plain key press with no modifiers.
fn input(key: &str) -> KeyInput {
    return KeyInput {
        key: key.to_string(),
        ctrl: false,
        meta: false,
        alt: false,
        shift: false,
        repeat: false,
        in_popup: false,
    };
}

/// What: `context_on(route, zone)` builds the context of a page with a focus zone.
fn context_on(route: Route, zone: FocusZone) -> KeyContext {
    return KeyContext { route, zone, picker_overlay: false, popup_open: false };
}

#[test]
fn slint_special_keys_map_to_the_names_the_classifier_reads() {
    assert_eq!(key_name_of("\u{f702}"), "ArrowLeft");
    assert_eq!(key_name_of("\u{f703}"), "ArrowRight");
    assert_eq!(key_name_of("\u{f700}"), "ArrowUp");
    assert_eq!(key_name_of("\u{f701}"), "ArrowDown");
    assert_eq!(key_name_of("\n"), "Enter");
    assert_eq!(key_name_of("\u{1b}"), "Escape");
    assert_eq!(key_name_of("\u{8}"), "Backspace");
    assert_eq!(key_name_of("\u{7f}"), "Delete");
    assert_eq!(key_name_of("\u{f708}"), "F5");
    assert_eq!(key_name_of("\u{f70d}"), "F10");
    assert_eq!(key_name_of("\u{f729}"), "Home");
    assert_eq!(key_name_of("\u{f72b}"), "End");
    assert_eq!(key_name_of("\u{f72c}"), "PageUp");
    assert_eq!(key_name_of("\u{f72d}"), "PageDown");
    assert_eq!(key_name_of("\u{f735}"), "ContextMenu");
    assert_eq!(key_name_of("\t"), "Tab");
    assert_eq!(key_name_of("\u{19}"), "Tab");
    assert_eq!(key_name_of("f"), "f");
    assert_eq!(key_name_of(" "), " ");
    assert_eq!(key_name_of("!"), "!");
}

#[test]
fn modifier_keys_are_recognised_and_only_shift_keys_are_shift() {
    for text in ["\u{10}", "\u{11}", "\u{12}", "\u{13}", "\u{14}", "\u{15}", "\u{16}", "\u{17}", "\u{18}"] {
        assert!(is_modifier_key(text), "{text:?}");
    }
    assert!(!is_modifier_key("a"));
    assert!(!is_modifier_key("\u{f702}"));
    assert!(is_shift_key("\u{10}"));
    assert!(is_shift_key("\u{15}"));
    assert!(!is_shift_key("\u{11}"));
}

#[test]
fn the_apple_map_swaps_control_and_meta_because_slint_does() {
    let flags = held_modifiers(true, false, true, false, true);
    assert_eq!(flags, HeldModifiers { ctrl: false, meta: true, alt: true, shift: false });
    let plain = held_modifiers(true, false, true, false, false);
    assert_eq!(plain, HeldModifiers { ctrl: true, meta: false, alt: true, shift: false });
}

#[test]
fn routes_map_to_the_classifier_routes() {
    assert_eq!(key_route(Route::Player), KeyRoute::Player);
    assert_eq!(key_route(Route::Search), KeyRoute::Search);
    assert_eq!(key_route(Route::Settings), KeyRoute::Settings);
    assert_eq!(key_route(Route::TemplateEditor), KeyRoute::EditorPage);
}

#[test]
fn the_request_carries_the_zone_the_route_and_the_popup_state() {
    let context = KeyContext { route: Route::Search, zone: FocusZone::TextField, picker_overlay: true, popup_open: true };
    let request = request_for(&KeyInput { in_popup: true, ..input("a") }, &context, KeyboardPlatform::Mac);
    assert_eq!(request.zone, FocusZone::TextField);
    assert_eq!(request.route, KeyRoute::Search);
    assert!(request.in_popup);
    assert!(request.picker_overlay);
    assert_eq!(request.platform, KeyboardPlatform::Mac);
    assert!(!request.composing);
}

#[test]
fn space_toggles_playback_on_the_player_route_and_is_left_to_the_search_field() {
    let on_player = request_for(&input(" "), &context_on(Route::Player, FocusZone::Surface), KeyboardPlatform::WindowsLinux);
    assert_eq!(shortcut_action(&on_player), ShortcutAction::Toggle);
    let in_search = request_for(&input(" "), &context_on(Route::Search, FocusZone::TextField), KeyboardPlatform::WindowsLinux);
    assert_eq!(shortcut_action(&in_search), ShortcutAction::Owner);
}

#[test]
fn next_mode_cycles_through_all_four_modes() {
    let mut mode = PlaybackMode::Repeat;
    let mut seen = Vec::new();
    for _ in 0..4 {
        seen.push(mode);
        mode = next_mode(mode);
    }
    assert_eq!(seen, vec![PlaybackMode::Repeat, PlaybackMode::InOrder, PlaybackMode::ShufflePage, PlaybackMode::ShuffleAll]);
    assert_eq!(mode, PlaybackMode::Repeat);
}

#[test]
fn a_seek_stays_inside_the_track_and_needs_a_loaded_track() {
    assert_eq!(seek_target(100.0, 200.0, 10.0), Some(110.0));
    assert_eq!(seek_target(100.0, 200.0, -60.0), Some(40.0));
    assert_eq!(seek_target(5.0, 200.0, -10.0), Some(0.0));
    assert_eq!(seek_target(195.0, 200.0, 60.0), Some(200.0));
    assert_eq!(seek_target(10.0, 0.0, 10.0), None);
}
