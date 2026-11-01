//! Key names a person can say to the `key` command,
//!  resolved to Linux evdev codes.
//!
//! A name is one of:
//!  a raw code `code:<n>` (evdev numbering,
//!  decimal or `0x` hexadecimal,
//!  1 to 767),
//!  a spoken alias (`ctrl`,
//!  `win`,
//!  `menu`,
//!  `altgr`,
//!  `pgdn`,
//!  `playpause`),
//!  any `KEY_*` name from the kernel header (`f13`,
//!  `kp7`,
//!  `scrolllock`),
//!  or a single character.
//!  Case,
//!  underscores,
//!  and hyphens do not matter,
//!  so `Page_Up`,
//!  `PAGEUP`,
//!  and `page-up` are one key.

/// Sibling modules of this crate.
use crate::{key_table::EVDEV_KEYS, keymap::char_to_key};

/// Highest evdev key code the kernel header defines (`KEY_MAX`).
///
/// What:
///      `pub const KEY_CODE_MAX: u32 = 0x2ff;`.
/// Why:
///       `code:<n>` refuses anything above it,
///  because no keyboard can send it and the xkb keycode would leave the evdev range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KEY_CODE_MAX = 0x2ff;
/// ```
pub const KEY_CODE_MAX: u32 = 0x2ff;

/// Spoken aliases that differ from the kernel names,
///  already normalized (lowercase,
///  no `_` or `-`).
///
/// What:
///      `const ALIASES: &[(&str, u32)]`.
///  Modifiers resolve to the left key unless named right.
///  `menu` is the context-menu key a keyboard sends (`KEY_COMPOSE`,
///  keysym Menu),
///  not `KEY_MENU` (keysym XF86MenuKB),
///  which is `menukb`.
///  `print` is `KEY_SYSRQ`,
///  the key a keyboard sends for Print Screen.
///  `plus` is the `=` key:
///  hold shift yourself for a typed plus.
/// Why:
///       People say `ctrl`,
///  `win`,
///  `pgdn`,
///  and `menu`,
///  and the kernel header says
///           `LEFTCTRL`,
///  `LEFTMETA`,
///  `PAGEDOWN`,
///  and `COMPOSE`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ALIASES: readonly (readonly [string, number])[] = [["ctrl", 29], ["menu", 127], ...];
/// ```
#[rustfmt::skip]
const ALIASES: &[(&str, u32)] = &[
    ("ctrl", 29), ("control", 29), ("lctrl", 29), ("ctrll", 29), ("lcontrol", 29), ("controll", 29),
    ("rctrl", 97), ("ctrlr", 97), ("rcontrol", 97), ("controlr", 97),
    ("shift", 42), ("lshift", 42), ("shiftl", 42), ("rshift", 54), ("shiftr", 54),
    ("alt", 56), ("lalt", 56), ("altl", 56), ("ralt", 100), ("altr", 100), ("altgr", 100),
    ("isolevel3shift", 100),
    ("meta", 125), ("super", 125), ("win", 125), ("windows", 125), ("logo", 125), ("cmd", 125),
    ("command", 125), ("lmeta", 125), ("lsuper", 125), ("lwin", 125), ("metal", 125), ("superl", 125),
    ("rmeta", 126), ("rsuper", 126), ("rwin", 126), ("metar", 126), ("superr", 126),
    ("enter", 28), ("return", 28), ("escape", 1), ("bksp", 14), ("del", 111), ("ins", 110),
    ("pgup", 104), ("prior", 104), ("pgdn", 109), ("next", 109),
    ("menu", 127), ("compose", 127), ("contextmenu", 127), ("apps", 127), ("menukb", 139),
    ("print", 99), ("printscreen", 99), ("prtsc", 99), ("prtscr", 99), ("plus", 13),
    ("bracketleft", 26), ("bracketright", 27), ("period", 52),
    ("arrowup", 103), ("arrowdown", 108), ("arrowleft", 105), ("arrowright", 106),
    ("kpadd", 78), ("kpsubtract", 74), ("kpmultiply", 55), ("kpdivide", 98), ("kpdecimal", 83),
    ("numpadenter", 96), ("numpadadd", 78), ("numpadsubtract", 74), ("numpadmultiply", 55),
    ("numpaddivide", 98), ("numpaddecimal", 83),
    ("playpause", 164), ("audioplay", 164), ("mediaplay", 164), ("xf86audioplay", 164),
    ("audiopause", 201), ("mediapause", 201), ("xf86audiopause", 201),
    ("audiostop", 166), ("mediastop", 166), ("xf86audiostop", 166),
    ("audionext", 163), ("medianext", 163), ("xf86audionext", 163),
    ("audioprev", 165), ("mediaprev", 165), ("xf86audioprev", 165), ("mediaprevious", 165),
    ("audioraisevolume", 115), ("xf86audioraisevolume", 115), ("audiolowervolume", 114),
    ("xf86audiolowervolume", 114), ("audiomute", 113), ("xf86audiomute", 113),
];

/// Lowercase a key name and drop `_` and `-`.
///
/// What:
///      `pub fn normalize(name: &str) -> String`.
/// Why:
///       `Page_Up`,
///  `page-up`,
///  and `PAGEUP` must be the same lookup key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const normalize = (name: string) => name.toLowerCase().replace(/[_-]/g, "");
/// ```
pub fn normalize(name: &str) -> String {
    return name
        .chars()
        .filter(|character| return *character != '_' && *character != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

/// Parse the number of a `code:<n>` key.
///
/// What:
///      `fn raw_code(text: &str) -> Option<u32>`.
///  Decimal or `0x` hexadecimal;
///  `None` for 0,
///  for anything above `KEY_CODE_MAX`,
///  and for non-numbers.
/// Why:
///       A key the name tables lack is still pressable by its evdev number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rawCode(text: string): number | undefined { ... }
/// ```
fn raw_code(text: &str) -> Option<u32> {
    let parsed = match text.strip_prefix("0x") {
        Some(hexadecimal) => u32::from_str_radix(hexadecimal, 16).ok(),
        None => text.parse::<u32>().ok(),
    };
    return parsed.filter(|code| return (1..=KEY_CODE_MAX).contains(code))
}

/// Resolve one key name to its evdev code.
///
/// What:
///      `pub fn resolve(name: &str) -> Option<u32>`.
///  Order:
///  `code:<n>`,
///  then the kernel's `KEY_` spelling with the prefix removed,
///  then the aliases,
///  then the kernel names,
///  then a single character on the US layout (its Shift is not added,
///  as before).
/// Why:
///       One lookup behind `key NAME`,
///  so every spelling a person might type reaches a physical key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolve(name: string): number | undefined { ... }
/// ```
///
/// @example
/// ```ts
/// resolve("F10");         // => 68
/// resolve("Menu");        // => 127
/// resolve("code:68");     // => 68
/// resolve("KP_Enter");    // => 96
/// ```
pub fn resolve(name: &str) -> Option<u32> {
    if let Some(number) = name.to_lowercase().strip_prefix("code:") {
        return raw_code(number)
    }
    if let Some(kernel_name) = name.to_lowercase().strip_prefix("key_") {
        return resolve(kernel_name)
    }
    let normalized = normalize(name);
    if let Some((_, code)) = ALIASES.iter().find(|(alias, _)| return *alias == normalized) {
        return Some(*code)
    }
    if let Some((_, code)) = EVDEV_KEYS.iter().find(|(kernel, _)| return *kernel == normalized) {
        return Some(*code)
    }
    let mut characters = name.chars();
    let first = characters.next()?;
    if characters.next().is_some() {
        return None
    }
    return char_to_key(first).map(|(code, _shift)| return code)
}

/// What:
///      `#[cfg(test)] #[path = "key_name_tests.rs"] mod tests;`.
/// Why:
///       Keep the name-table tests beside the lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./key_name_tests";
/// ```
#[cfg(test)]
#[path = "key_name_tests.rs"]
mod tests;
