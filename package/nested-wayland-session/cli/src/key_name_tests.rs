//! Unit tests for the key-name lookup.

use super::{normalize, resolve, KEY_CODE_MAX};

/// Every function key from F1 to F24 resolves to the kernel's code for it.
#[test]
fn function_keys_f1_to_f24_resolve() {
    let expected = [
        59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 87, 88, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194,
    ];
    for (index, code) in expected.iter().enumerate() {
        assert_eq!(resolve(&format!("f{}", index + 1)), Some(*code), "F{}", index + 1);
        assert_eq!(resolve(&format!("F{}", index + 1)), Some(*code), "F{} upper", index + 1);
    }
    assert_eq!(resolve("f25"), None);
    assert_eq!(resolve("f0"), None);
}

/// Names in X11 keysym spelling and kernel spelling reach the same physical key.
#[test]
fn spellings_of_one_key_agree() {
    for spelling in ["pageup", "PageUp", "Page_Up", "page-up", "PGUP", "prior", "KEY_PAGEUP"] {
        assert_eq!(resolve(spelling), Some(104), "{spelling}");
    }
    assert_eq!(resolve("Scroll_Lock"), Some(70));
    assert_eq!(resolve("Num_Lock"), Some(69));
    assert_eq!(resolve("Caps_Lock"), Some(58));
    assert_eq!(resolve("Insert"), Some(110));
    assert_eq!(resolve("Delete"), Some(111));
    assert_eq!(resolve("Home"), Some(102));
    assert_eq!(resolve("End"), Some(107));
    assert_eq!(resolve("Pause"), Some(119));
    assert_eq!(resolve("Print"), Some(99));
}

/// Menu is the context-menu key (`KEY_COMPOSE`),
///  and the kernel's `KEY_MENU` keeps its own name.
#[test]
fn menu_is_the_context_menu_key() {
    assert_eq!(resolve("menu"), Some(127));
    assert_eq!(resolve("Menu"), Some(127));
    assert_eq!(resolve("compose"), Some(127));
    assert_eq!(resolve("menukb"), Some(139));
}

/// Every modifier resolves on both sides.
#[test]
fn modifiers_resolve_left_and_right() {
    assert_eq!(resolve("ctrl"), Some(29));
    assert_eq!(resolve("Control_L"), Some(29));
    assert_eq!(resolve("leftctrl"), Some(29));
    assert_eq!(resolve("Control_R"), Some(97));
    assert_eq!(resolve("rctrl"), Some(97));
    assert_eq!(resolve("shift"), Some(42));
    assert_eq!(resolve("Shift_L"), Some(42));
    assert_eq!(resolve("Shift_R"), Some(54));
    assert_eq!(resolve("alt"), Some(56));
    assert_eq!(resolve("Alt_L"), Some(56));
    assert_eq!(resolve("Alt_R"), Some(100));
    assert_eq!(resolve("AltGr"), Some(100));
    assert_eq!(resolve("meta"), Some(125));
    assert_eq!(resolve("Super_L"), Some(125));
    assert_eq!(resolve("Super_R"), Some(126));
    assert_eq!(resolve("win"), Some(125));
    assert_eq!(resolve("rwin"), Some(126));
}

/// The keypad,
///  media keys,
///  punctuation,
///  and digits resolve.
#[test]
fn keypad_media_punctuation_and_digits_resolve() {
    assert_eq!(resolve("kp0"), Some(82));
    assert_eq!(resolve("KP_7"), Some(71));
    assert_eq!(resolve("KP_Enter"), Some(96));
    assert_eq!(resolve("KP_Add"), Some(78));
    assert_eq!(resolve("KP_Subtract"), Some(74));
    assert_eq!(resolve("KP_Multiply"), Some(55));
    assert_eq!(resolve("KP_Divide"), Some(98));
    assert_eq!(resolve("KP_Decimal"), Some(83));
    assert_eq!(resolve("playpause"), Some(164));
    assert_eq!(resolve("XF86AudioPlay"), Some(164));
    assert_eq!(resolve("nextsong"), Some(163));
    assert_eq!(resolve("volumeup"), Some(115));
    assert_eq!(resolve("mute"), Some(113));
    assert_eq!(resolve("minus"), Some(12));
    assert_eq!(resolve("equal"), Some(13));
    assert_eq!(resolve("grave"), Some(41));
    assert_eq!(resolve("backslash"), Some(43));
    assert_eq!(resolve("comma"), Some(51));
    assert_eq!(resolve("period"), Some(52));
    assert_eq!(resolve("slash"), Some(53));
    assert_eq!(resolve("0"), Some(11));
    assert_eq!(resolve("9"), Some(10));
    assert_eq!(resolve("-"), Some(12));
}

/// The names that worked before still resolve to the same codes.
#[test]
fn earlier_names_keep_their_codes() {
    assert_eq!(resolve("enter"), Some(28));
    assert_eq!(resolve("return"), Some(28));
    assert_eq!(resolve("esc"), Some(1));
    assert_eq!(resolve("escape"), Some(1));
    assert_eq!(resolve("tab"), Some(15));
    assert_eq!(resolve("space"), Some(57));
    assert_eq!(resolve("backspace"), Some(14));
    assert_eq!(resolve("del"), Some(111));
    assert_eq!(resolve("up"), Some(103));
    assert_eq!(resolve("down"), Some(108));
    assert_eq!(resolve("left"), Some(105));
    assert_eq!(resolve("right"), Some(106));
    assert_eq!(resolve("pagedown"), Some(109));
    assert_eq!(resolve("A"), Some(30));
    assert_eq!(resolve("a"), Some(30));
}

/// `code:<n>` takes evdev numbers in decimal or hexadecimal and refuses the rest.
#[test]
fn raw_codes_are_evdev_numbers_in_range() {
    assert_eq!(resolve("code:68"), Some(68));
    assert_eq!(resolve("CODE:68"), Some(68));
    assert_eq!(resolve("code:0x44"), Some(68));
    assert_eq!(resolve("code:1"), Some(1));
    assert_eq!(resolve("code:767"), Some(KEY_CODE_MAX));
    assert_eq!(resolve("code:0"), None);
    assert_eq!(resolve("code:768"), None);
    assert_eq!(resolve("code:-1"), None);
    assert_eq!(resolve("code:"), None);
    assert_eq!(resolve("code:f5"), None);
}

/// Unknown and empty names resolve to nothing.
#[test]
fn unknown_names_do_not_resolve() {
    assert_eq!(resolve("bogus"), None);
    assert_eq!(resolve(""), None);
    assert_eq!(resolve("f5f"), None);
    assert_eq!(resolve("\u{20AC}"), None);
}

/// Normalizing lowercases and drops separators only.
#[test]
fn normalize_lowercases_and_drops_separators() {
    assert_eq!(normalize("Page_Up-X"), "pageupx");
}
