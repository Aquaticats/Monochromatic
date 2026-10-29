// What:  Unit tests for the US-layout keycode tables.
// Why:   The tables are the ground truth for synthetic typing; a wrong code would silently
//        type the wrong character, so a few anchor cases guard the mapping.
//
// In TS you'd write (pseudocode):
// ```ts
// describe("keymap", () => { /* cases below */ });
// ```

use super::{char_to_key, named_key, LEFT_SHIFT};

#[test]
fn letters_share_a_code_and_case_sets_shift() {
    // KEY_A is evdev 30; case only flips the shift flag.
    assert_eq!(char_to_key('a'), Some((30, false)));
    assert_eq!(char_to_key('A'), Some((30, true)));
    assert_eq!(char_to_key('z'), Some((44, false)));
    assert_eq!(char_to_key('Z'), Some((44, true)));
}

#[test]
fn digits_space_and_shifted_symbols() {
    assert_eq!(char_to_key('1'), Some((2, false)));
    assert_eq!(char_to_key('!'), Some((2, true)));
    assert_eq!(char_to_key('0'), Some((11, false)));
    assert_eq!(char_to_key(')'), Some((11, true)));
    assert_eq!(char_to_key(' '), Some((57, false)));
    assert_eq!(char_to_key('/'), Some((53, false)));
    assert_eq!(char_to_key('?'), Some((53, true)));
}

#[test]
fn characters_off_the_us_layout_are_none() {
    // A non-US-keyboard character has no mapping and is skipped by the caller.
    assert_eq!(char_to_key('\u{20AC}'), None); // euro sign
    assert_eq!(char_to_key('\u{1F600}'), None); // emoji
}

#[test]
fn named_keys_and_single_characters_resolve() {
    assert_eq!(named_key("enter"), Some(28));
    assert_eq!(named_key("return"), Some(28));
    assert_eq!(named_key("space"), Some(57));
    assert_eq!(named_key("escape"), Some(1));
    // A single-character name maps through char_to_key (shift flag dropped).
    assert_eq!(named_key("a"), Some(30));
    assert_eq!(named_key("A"), Some(30));
    // Unknown multi-character names and the empty name have no code.
    assert_eq!(named_key("bogus"), None);
    assert_eq!(named_key(""), None);
}

/// Modifier names use the Linux evdev codes so shortcut probes reach the hosted client's real seat.
#[test]
fn named_modifiers_resolve_to_native_keycodes() {
    // Some is the expected native code, not a text character or a host-global input operation.
    assert_eq!(named_key("ctrl"), Some(29));
    assert_eq!(named_key("shift"), Some(42));
    assert_eq!(named_key("alt"), Some(56));
    assert_eq!(named_key("meta"), Some(125));
}

#[test]
fn left_shift_constant_is_the_evdev_code() {
    assert_eq!(LEFT_SHIFT, 42);
}
