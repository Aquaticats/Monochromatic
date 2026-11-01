//! Tests for the key hint badge state and texts (D106).
//!
//! What: the state tests drive `KeyHints` with injected time; the text tests read `badge_text` for held sets.
//! Why: D106 fixes the 300 ms hold, the narrowing as more modifiers are held, and the disappearance on release.

use super::*;

/// What: `ctrl` is the held set with Control only.
const CTRL: HeldModifiers = HeldModifiers { ctrl: true, meta: false, alt: false, shift: false };
/// What: `ctrl_shift` is the held set with Control and Shift.
const CTRL_SHIFT: HeldModifiers = HeldModifiers { ctrl: true, meta: false, alt: false, shift: true };
/// What: `alt` is the held set with Alt only.
const ALT: HeldModifiers = HeldModifiers { ctrl: false, meta: false, alt: true, shift: false };
/// What: `alt_shift` is the held set with Alt and Shift.
const ALT_SHIFT: HeldModifiers = HeldModifiers { ctrl: false, meta: false, alt: true, shift: true };
/// What: `ctrl_alt` is the held set with Control and Alt.
const CTRL_ALT: HeldModifiers = HeldModifiers { ctrl: true, meta: false, alt: true, shift: false };
/// What: `none` is the empty held set.
const NONE: HeldModifiers = HeldModifiers { ctrl: false, meta: false, alt: false, shift: false };
/// What: `command` is the held set with Command only, as the Mac map reads it.
const COMMAND: HeldModifiers = HeldModifiers { ctrl: false, meta: true, alt: false, shift: false };

#[test]
fn badges_are_hidden_until_the_modifiers_have_been_held_for_300_ms() {
    let held = KeyHints::default().on_modifiers(CTRL, 1000);
    assert_eq!(held.on_tick(1299).visible(), None);
    assert_eq!(held.on_tick(1300).visible(), Some(CTRL));
    assert_eq!(held.on_tick(5000).visible(), Some(CTRL));
}

#[test]
fn a_quick_chord_never_shows_badges() {
    let pressed = KeyHints::default().on_modifiers(CTRL, 0);
    let released = pressed.on_tick(120).on_modifiers(NONE, 150);
    assert_eq!(released.on_tick(900).visible(), None);
}

#[test]
fn adding_a_modifier_before_the_badges_show_restarts_the_hold() {
    let hints = KeyHints::default().on_modifiers(CTRL, 0).on_modifiers(CTRL_SHIFT, 200);
    assert_eq!(hints.on_tick(450).visible(), None);
    assert_eq!(hints.on_tick(500).visible(), Some(CTRL_SHIFT));
}

#[test]
fn shown_badges_narrow_at_once_when_more_modifiers_are_held() {
    let shown = KeyHints::default().on_modifiers(CTRL, 0).on_tick(300);
    assert_eq!(shown.visible(), Some(CTRL));
    assert_eq!(shown.on_modifiers(CTRL_SHIFT, 400).visible(), Some(CTRL_SHIFT));
}

#[test]
fn releasing_one_required_modifier_keeps_the_rest_and_releasing_all_hides_and_resets() {
    let shown = KeyHints::default().on_modifiers(CTRL_SHIFT, 0).on_tick(300);
    let one_released = shown.on_modifiers(CTRL, 500);
    assert_eq!(one_released.visible(), Some(CTRL));
    let all_released = one_released.on_modifiers(NONE, 600);
    assert_eq!(all_released.visible(), None);
    let held_again = all_released.on_modifiers(CTRL, 700);
    assert_eq!(held_again.on_tick(900).visible(), None, "the hold time starts over after a full release");
    assert_eq!(held_again.on_tick(1000).visible(), Some(CTRL));
}

#[test]
fn an_unchanged_held_set_keeps_its_start_time() {
    let hints = KeyHints::default().on_modifiers(CTRL, 0).on_modifiers(CTRL, 250);
    assert_eq!(hints.on_tick(300).visible(), Some(CTRL));
}

#[test]
fn wait_ms_counts_down_to_the_hold_time_and_ends_once_shown() {
    let hints = KeyHints::default().on_modifiers(CTRL, 100);
    assert_eq!(KeyHints::default().wait_ms(0), None);
    assert_eq!(hints.wait_ms(100), Some(300));
    assert_eq!(hints.wait_ms(250), Some(150));
    assert_eq!(hints.wait_ms(900), Some(0));
    assert_eq!(hints.on_tick(400).wait_ms(400), None);
}

#[test]
fn control_held_shows_the_plain_control_chords() {
    let platform = KeyboardPlatform::WindowsLinux;
    assert_eq!(badge_text(HintTarget::Search, platform, Some(CTRL)), "F");
    assert_eq!(badge_text(HintTarget::FolderTrigger, platform, Some(CTRL)), "O");
    assert_eq!(badge_text(HintTarget::DeckTitle, platform, Some(CTRL)), "G");
    assert_eq!(badge_text(HintTarget::Previous, platform, Some(CTRL)), "Left");
    assert_eq!(badge_text(HintTarget::Next, platform, Some(CTRL)), "Right");
    assert_eq!(badge_text(HintTarget::ModeCycle, platform, Some(CTRL)), "M");
}

#[test]
fn control_held_hides_the_chords_that_need_shift() {
    let platform = KeyboardPlatform::WindowsLinux;
    assert_eq!(badge_text(HintTarget::OpenFolder, platform, Some(CTRL)), "");
    assert_eq!(badge_text(HintTarget::ModeRepeat, platform, Some(CTRL)), "");
    assert_eq!(badge_text(HintTarget::DeckFolder, platform, Some(CTRL)), "");
}

#[test]
fn adding_shift_swaps_the_plain_badges_for_the_shift_chords() {
    let platform = KeyboardPlatform::WindowsLinux;
    assert_eq!(badge_text(HintTarget::Search, platform, Some(CTRL_SHIFT)), "");
    assert_eq!(badge_text(HintTarget::FolderTrigger, platform, Some(CTRL_SHIFT)), "");
    assert_eq!(badge_text(HintTarget::OpenFolder, platform, Some(CTRL_SHIFT)), "O");
    assert_eq!(badge_text(HintTarget::DeckFolder, platform, Some(CTRL_SHIFT)), "G");
    assert_eq!(badge_text(HintTarget::ModeRepeat, platform, Some(CTRL_SHIFT)), "1");
    assert_eq!(badge_text(HintTarget::ModeInOrder, platform, Some(CTRL_SHIFT)), "2");
    assert_eq!(badge_text(HintTarget::ModeShuffleFolder, platform, Some(CTRL_SHIFT)), "3");
    assert_eq!(badge_text(HintTarget::ModeShuffleAll, platform, Some(CTRL_SHIFT)), "4");
    assert_eq!(badge_text(HintTarget::EditorReset, platform, Some(CTRL_SHIFT)), "Backspace");
}

#[test]
fn alt_held_shows_the_seek_and_focus_chords_and_shift_keeps_the_seek_ends() {
    let platform = KeyboardPlatform::WindowsLinux;
    assert_eq!(badge_text(HintTarget::SeekBack, platform, Some(ALT)), "Left");
    assert_eq!(badge_text(HintTarget::SeekForward, platform, Some(ALT)), "Right");
    assert_eq!(badge_text(HintTarget::FolderList, platform, Some(ALT)), "1");
    assert_eq!(badge_text(HintTarget::TrackList, platform, Some(ALT)), "2");
    assert_eq!(badge_text(HintTarget::SeekBack, platform, Some(ALT_SHIFT)), "Left");
    assert_eq!(badge_text(HintTarget::SeekForward, platform, Some(ALT_SHIFT)), "Right");
    assert_eq!(badge_text(HintTarget::FolderList, platform, Some(ALT_SHIFT)), "");
}

#[test]
fn control_and_alt_held_show_the_settings_chord() {
    let platform = KeyboardPlatform::WindowsLinux;
    assert_eq!(badge_text(HintTarget::Settings, platform, Some(CTRL_ALT)), "S");
    assert_eq!(badge_text(HintTarget::Settings, platform, Some(CTRL)), "");
}

#[test]
fn the_mac_map_reads_command_as_the_primary_modifier() {
    let platform = KeyboardPlatform::Mac;
    assert_eq!(badge_text(HintTarget::Search, platform, Some(COMMAND)), "F");
    assert_eq!(badge_text(HintTarget::Settings, platform, Some(COMMAND)), ",");
    assert_eq!(badge_text(HintTarget::Search, platform, Some(CTRL)), "");
}

#[test]
fn no_visible_set_means_no_badge_for_any_control() {
    for target in HintTarget::ALL {
        assert_eq!(badge_text(target, KeyboardPlatform::WindowsLinux, None), "", "{target:?}");
    }
}

#[test]
fn every_control_has_a_chord_on_both_maps() {
    for platform in KeyboardPlatform::ALL {
        for target in HintTarget::ALL {
            let has_chord = target.actions().iter().any(|action| return binding_for(*action, platform).is_some());
            assert!(has_chord, "{target:?} on {platform:?}");
        }
    }
}
