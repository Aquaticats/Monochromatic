//! Unit tests for the Unicode keymap builder,
//!  compiled with the same xkb library the clients use.

use smithay::input::keyboard::xkb;

use super::{base_keymap_text, spare_evdev, with_unicode_keys, MAX_SPARE_KEYS};

/// Compile keymap text and return the UTF-8 each evdev code types unshifted.
fn typed_by(text: &str, evdev: u32) -> Option<String> {
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let keymap = xkb::Keymap::new_from_string(&context, text.to_string(), xkb::KEYMAP_FORMAT_TEXT_V1, xkb::KEYMAP_COMPILE_NO_FLAGS)?;
    let state = xkb::State::new(&keymap);
    let utf8 = state.key_get_utf8(xkb::Keycode::new(evdev + 8));
    return if utf8.is_empty() { None } else { Some(utf8) }
}

/// The base keymap still types ordinary letters.
#[test]
fn the_base_keymap_types_a_letter() {
    let base = base_keymap_text();
    assert_eq!(base.as_ref().map(|text| return typed_by(text, 30)), Ok(Some("a".to_string())));
}

/// Accents,
///  Hangul,
///  kana,
///  and a character outside the BMP each type as themselves on their spare key.
#[test]
fn spare_keys_type_accents_cjk_hangul_and_emoji() {
    let characters = ['\u{f6}', '\u{304b}', '\u{d55c}', '\u{1F600}', '\u{20AC}', '\u{301}'];
    let base = base_keymap_text().unwrap_or_default();
    let keymap = with_unicode_keys(&base, &characters).unwrap_or_default();
    for (index, character) in characters.iter().enumerate() {
        assert_eq!(typed_by(&keymap, spare_evdev(index)), Some(character.to_string()), "character {index}");
    }
    assert_eq!(typed_by(&keymap, 30), Some("a".to_string()), "base keys survive");
}

/// More than `MAX_SPARE_KEYS` characters are refused.
#[test]
fn too_many_characters_are_refused() {
    let base = base_keymap_text().unwrap_or_default();
    let characters = vec!['x'; MAX_SPARE_KEYS + 1];
    assert!(with_unicode_keys(&base, &characters).is_err());
    assert!(with_unicode_keys(&base, &characters[..MAX_SPARE_KEYS]).is_ok());
}

/// A text with no keycodes section is refused instead of corrupted.
#[test]
fn a_malformed_base_is_refused() {
    assert!(with_unicode_keys("not a keymap", &['x']).is_err());
}
