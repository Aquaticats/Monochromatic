//! Tests for the D106 binding table.
//!
//! What: every row of `BINDINGS` is turned into the key press that should hit it and classified, so a row that is
//! shadowed by an earlier one, scoped to nowhere, or mistyped fails; the chord texts are compared with D106.
//! Why: the table is data, and a wrong row would otherwise stay silent until a person pressed that key.

use super::*;
use crate::keymap::{FocusZone, KeyRoute, shortcut_action};

/// What: `request_for_row` builds a key press that satisfies the row's modifiers, key, platform and scope.
/// Why: each row must classify to its own action when pressed where it applies.
fn request_for_row(binding: &Binding, platform: KeyboardPlatform, key: &str) -> ShortcutRequest {
    let held = binding.required(platform);
    let (route, zone, in_popup, picker_overlay) = match binding.scope {
        Scope::Anywhere | Scope::PlayerPage | Scope::PlaybackArea | Scope::SpaceZones => {
            (KeyRoute::Player, FocusZone::Surface, false, false)
        }
        Scope::ListZone | Scope::TrackList => (KeyRoute::Player, FocusZone::TrackList, false, false),
        Scope::FolderList => (KeyRoute::Player, FocusZone::FolderList, false, false),
        Scope::Popup => (KeyRoute::Player, FocusZone::Surface, true, false),
        Scope::PickerOverlay => (KeyRoute::Player, FocusZone::Surface, false, true),
        Scope::SearchPage => (KeyRoute::Search, FocusZone::Button, false, false),
        Scope::SearchField => (KeyRoute::Search, FocusZone::TextField, false, false),
        Scope::SettingsIdle | Scope::EditorOrSettings => (KeyRoute::Settings, FocusZone::SettingsList, false, false),
        Scope::SettingsField => (KeyRoute::Settings, FocusZone::TextField, false, false),
        Scope::EditorPage => (KeyRoute::EditorPage, FocusZone::TextField, false, false),
        Scope::LicensesPage => (KeyRoute::Licenses, FocusZone::Button, false, false),
    };
    return ShortcutRequest {
        platform,
        key: key.to_owned(),
        ctrl: held.ctrl,
        meta: held.meta,
        alt: held.alt,
        shift: held.shift,
        repeat: false,
        zone,
        route,
        in_popup,
        picker_overlay,
        composing: false,
    };
}

#[test]
fn every_row_classifies_to_its_own_action_on_every_map_it_applies_to_for_every_key_name() {
    for (index, binding) in BINDINGS.iter().enumerate() {
        for platform in KeyboardPlatform::ALL {
            if !binding.applies_on(platform) {
                continue;
            }
            for key in binding.keys {
                let request = request_for_row(binding, platform, key);
                assert_eq!(
                    shortcut_action(&request),
                    binding.action,
                    "row {index} ({:?}) on {platform:?} with key {key:?}",
                    binding.action
                );
            }
        }
    }
}

#[test]
fn every_row_with_a_letter_key_also_matches_the_capital_letter() {
    for binding in BINDINGS.iter() {
        for key in binding.keys {
            if key.chars().count() == 1 && key.chars().all(char::is_alphabetic) {
                assert!(binding.matches_key(&key.to_uppercase()), "{:?} with {key:?}", binding.action);
            }
        }
    }
}

#[test]
fn a_row_does_not_match_with_an_extra_or_missing_modifier() {
    for binding in BINDINGS.iter().filter(|row| return row.scope != Scope::Popup && !row.menu) {
        let platform = KeyboardPlatform::WindowsLinux;
        if !binding.applies_on(platform) {
            continue;
        }
        let Some(key) = binding.keys.first() else {
            continue;
        };
        let exact = request_for_row(binding, platform, key);
        let with_meta = ShortcutRequest { meta: !exact.meta, ..exact.clone() };
        assert_ne!(shortcut_action(&with_meta), binding.action, "{:?} with Meta toggled", binding.action);
        let with_alt = ShortcutRequest { alt: !exact.alt, ..exact.clone() };
        assert_ne!(shortcut_action(&with_alt), binding.action, "{:?} with Alt toggled", binding.action);
    }
}

#[test]
fn every_action_a_control_or_a_menu_row_shows_has_a_chord_text_on_both_maps() {
    let actions = [
        ShortcutAction::Search,
        ShortcutAction::Picker,
        ShortcutAction::Settings,
        ShortcutAction::Reveal,
        ShortcutAction::Toggle,
        ShortcutAction::Mode,
        ShortcutAction::Previous,
        ShortcutAction::Next,
        ShortcutAction::SeekBack,
        ShortcutAction::SeekForward,
        ShortcutAction::SeekBackLong,
        ShortcutAction::SeekForwardLong,
        ShortcutAction::ModeRepeat,
        ShortcutAction::ModeInOrder,
        ShortcutAction::ModeShuffleFolder,
        ShortcutAction::ModeShuffleAll,
        ShortcutAction::Restart,
        ShortcutAction::PreviousFolder,
        ShortcutAction::NextFolder,
        ShortcutAction::RevealFolder,
        ShortcutAction::FocusFolders,
        ShortcutAction::FocusTracks,
        ShortcutAction::OpenFolderDialog,
        ShortcutAction::Quit,
        ShortcutAction::RowMenu,
        ShortcutAction::RowTrash,
        ShortcutAction::RowCopy,
        ShortcutAction::RowShow,
        ShortcutAction::ListActivate,
        ShortcutAction::Back,
        ShortcutAction::EditorReset,
        ShortcutAction::EditorApply,
    ];
    for action in actions {
        for platform in KeyboardPlatform::ALL {
            assert!(!shortcut_text(action, platform).is_empty(), "{action:?} on {platform:?}");
        }
    }
}

#[test]
fn chord_texts_match_the_decision_on_windows_and_linux() {
    let text = |action: ShortcutAction| return shortcut_text(action, KeyboardPlatform::WindowsLinux);
    assert_eq!(text(ShortcutAction::Search), "Ctrl+F");
    assert_eq!(text(ShortcutAction::Picker), "Ctrl+O");
    assert_eq!(text(ShortcutAction::Settings), "Ctrl+Alt+S");
    assert_eq!(text(ShortcutAction::Reveal), "Ctrl+G");
    assert_eq!(text(ShortcutAction::Toggle), "Space");
    assert_eq!(text(ShortcutAction::Mode), "Ctrl+M");
    assert_eq!(text(ShortcutAction::Previous), "Ctrl+Left");
    assert_eq!(text(ShortcutAction::Next), "Ctrl+Right");
    assert_eq!(text(ShortcutAction::SeekBack), "Alt+Left");
    assert_eq!(text(ShortcutAction::SeekForwardLong), "Alt+Shift+Right");
    assert_eq!(text(ShortcutAction::ModeShuffleAll), "Ctrl+Shift+4");
    assert_eq!(text(ShortcutAction::Restart), "Ctrl+Home");
    assert_eq!(text(ShortcutAction::PreviousFolder), "Alt+Up");
    assert_eq!(text(ShortcutAction::RevealFolder), "Ctrl+Shift+G");
    assert_eq!(text(ShortcutAction::FocusFolders), "Alt+1");
    assert_eq!(text(ShortcutAction::FocusTracks), "Alt+2");
    assert_eq!(text(ShortcutAction::OpenFolderDialog), "Ctrl+Shift+O");
    assert_eq!(text(ShortcutAction::Quit), "Ctrl+Q");
    assert_eq!(text(ShortcutAction::RowMenu), "Menu");
    assert_eq!(text(ShortcutAction::RowTrash), "Delete");
    assert_eq!(text(ShortcutAction::RowCopy), "Ctrl+Shift+C");
    assert_eq!(text(ShortcutAction::RowShow), "Ctrl+Shift+E");
    assert_eq!(text(ShortcutAction::ListActivate), "Enter");
    assert_eq!(text(ShortcutAction::Back), "Escape");
    assert_eq!(text(ShortcutAction::EditorReset), "Ctrl+Shift+Backspace");
}

#[test]
fn chord_texts_use_the_command_and_option_glyphs_on_the_mac_map() {
    let text = |action: ShortcutAction| return shortcut_text(action, KeyboardPlatform::Mac);
    assert_eq!(text(ShortcutAction::Search), "\u{2318}F");
    assert_eq!(text(ShortcutAction::Settings), "\u{2318},");
    assert_eq!(text(ShortcutAction::Reveal), "\u{2318}L");
    assert_eq!(text(ShortcutAction::Previous), "\u{2318}\u{21E7}[");
    assert_eq!(text(ShortcutAction::Mode), "\u{2303}M");
    assert_eq!(text(ShortcutAction::SeekForward), "\u{2325}Right");
    assert_eq!(text(ShortcutAction::OpenFolderDialog), "\u{2318}\u{21E7}O");
    assert_eq!(text(ShortcutAction::Quit), "\u{2318}Q");
}

#[test]
fn an_action_without_a_row_has_an_empty_text() {
    assert_eq!(shortcut_text(ShortcutAction::Owner, KeyboardPlatform::WindowsLinux), "");
    assert_eq!(shortcut_text(ShortcutAction::ListLetter('a'), KeyboardPlatform::Mac), "");
}

#[test]
fn key_labels_read_like_the_keycaps() {
    assert_eq!(key_label("ArrowLeft"), "Left");
    assert_eq!(key_label("ArrowDown"), "Down");
    assert_eq!(key_label(" "), "Space");
    assert_eq!(key_label("ContextMenu"), "Menu");
    assert_eq!(key_label("f"), "F");
    assert_eq!(key_label("F5"), "F5");
    assert_eq!(key_label("PageDown"), "PageDown");
}

#[test]
fn the_held_set_is_empty_only_without_modifiers() {
    assert!(HeldModifiers::default().is_empty());
    assert!(!HeldModifiers { alt: true, ..HeldModifiers::default() }.is_empty());
}

#[test]
fn rows_marked_for_the_open_menu_are_the_menu_navigation_and_the_row_chords() {
    let menu_actions: Vec<ShortcutAction> = BINDINGS.iter().filter(|row| return row.menu).map(|row| return row.action).collect();
    assert_eq!(
        menu_actions,
        vec![
            ShortcutAction::MenuUp,
            ShortcutAction::MenuDown,
            ShortcutAction::MenuChoose,
            ShortcutAction::RowTrash,
            ShortcutAction::RowCopy,
            ShortcutAction::RowShow,
        ]
    );
}
