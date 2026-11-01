//! Keymaps that carry arbitrary Unicode characters,
//!  so `type` can deliver text the way an input method does:
//!  a key event whose keysym is the character.
//!
//! A client such as winit turns a key event into `KeyEvent.text` through the keymap it was sent,
//!  so a keymap with the keysym `U00F6` on a spare key makes that key type `ö`.
//!  Wayland has no other way to put an arbitrary character into a plain key event,
//!  and tools such as `wtype` use the same route.

/// Smithay compositor types.
use smithay::input::keyboard::xkb;

/// First evdev code used for a spare Unicode key.
///
/// What:
///      `pub const FIRST_SPARE_EVDEV: u32 = 0x300;`.
///  One above `KEY_MAX`,
///  so no real key can share it.
/// Why:
///       A spare key must not shadow a key a test can press.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FIRST_SPARE_EVDEV = 0x300;
/// ```
pub const FIRST_SPARE_EVDEV: u32 = 0x300;

/// Offset between an evdev code and the xkb keycode the keymap names.
///
/// What:
///      `const XKB_OFFSET: u32 = 8;`.
/// Why:
///       xkb keycodes are evdev codes plus 8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const XKB_OFFSET = 8;
/// ```
const XKB_OFFSET: u32 = 8;

/// Most characters one keymap carries.
///
/// What:
///      `pub const MAX_SPARE_KEYS: usize = 64;`.
///  Key names are `<X00>` to `<X3F>`.
/// Why:
///       A longer text switches keymaps between chunks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_SPARE_KEYS = 64;
/// ```
pub const MAX_SPARE_KEYS: usize = 64;

/// The compositor's default keymap as xkb text.
///
/// What:
///      `pub fn base_keymap_text() -> Result<String, String>`.
///  Compiled from the same defaults (and `XKB_DEFAULT_*` environment) the seat's keyboard uses.
/// Why:
///       The spare keys are added to the layout the client already has,
///  so ordinary keys keep working in the same keymap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function baseKeymapText(): string { ... }
/// ```
pub fn base_keymap_text() -> Result<String, String> {
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let keymap = xkb::Keymap::new_from_names(&context, "", "", "", "", None, xkb::KEYMAP_COMPILE_NO_FLAGS)
        .ok_or_else(|| return "compiling the default keymap failed".to_string())?;
    return Ok(keymap.get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1))
}

/// Evdev code of the spare key that types the character at an index.
///
/// What:
///      `pub fn spare_evdev(index: usize) -> u32`.
/// Why:
///       The typing loop and the keymap builder must agree on which key is which.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spareEvdev = (index: number) => FIRST_SPARE_EVDEV + index;
/// ```
#[must_use]
pub fn spare_evdev(index: usize) -> u32 {
    return FIRST_SPARE_EVDEV + u32::try_from(index).unwrap_or(0)
}

/// Insert text just after the first line break following a marker.
///
/// What:
///      `fn insert_after_header(keymap: &str, marker: &str, addition: &str) -> Result<String, String>`.
/// Why:
///       Both the keycodes and the symbols section take their new lines right after the section header.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function insertAfterHeader(keymap: string, marker: string, addition: string): string { ... }
/// ```
fn insert_after_header(keymap: &str, marker: &str, addition: &str) -> Result<String, String> {
    let header = keymap.find(marker).ok_or_else(|| return format!("the keymap has no {marker} section"))?;
    let line_end = keymap[header..]
        .find('\n')
        .ok_or_else(|| return format!("the keymap {marker} header has no line break"))?;
    let at = header + line_end + 1;
    return Ok(format!("{}{}{}", &keymap[..at], addition, &keymap[at..]))
}

/// Add spare keys that type the given characters to a keymap's text.
///
/// What:
///      `pub fn with_unicode_keys(base: &str, characters: &[char]) -> Result<String, String>`.
///  Character `n` is typed by evdev `spare_evdev(n)`.
///  At most `MAX_SPARE_KEYS` characters.
/// Why:
///       `type Björk` sends `ö` as a key whose keysym is U00F6,
///  which the client reads as the text `ö`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withUnicodeKeys(base: string, characters: string[]): string { ... }
/// ```
///
/// @example
/// ```ts
/// withUnicodeKeys(baseKeymap, ["ö"]); // adds `<X00> = 776;` and `key <X00> { [ U00F6 ] };`
/// ```
pub fn with_unicode_keys(base: &str, characters: &[char]) -> Result<String, String> {
    if characters.len() > MAX_SPARE_KEYS {
        return Err(format!("a keymap carries at most {MAX_SPARE_KEYS} spare keys"))
    }
    let mut keycodes = String::new();
    let mut symbols = String::new();
    for (index, character) in characters.iter().enumerate() {
        let keycode = spare_evdev(index) + XKB_OFFSET;
        keycodes.push_str(&format!("\t<X{index:02X}> = {keycode};\n"));
        symbols.push_str(&format!("\tkey <X{index:02X}> {{ [ U{:04X} ] }};\n", u32::from(*character)));
    }
    let with_codes = insert_after_header(base, "xkb_keycodes", &keycodes)?;
    let with_symbols = insert_after_header(&with_codes, "xkb_symbols", &symbols)?;
    return Ok(raise_maximum(&with_symbols, spare_evdev(MAX_SPARE_KEYS) + XKB_OFFSET))
}

/// Raise a keycodes section's `maximum = N;` line to at least a value.
///
/// What:
///      `fn raise_maximum(keymap: &str, minimum_maximum: u32) -> String`.
///  Leaves the text alone when it has no such line or already allows the value.
/// Why:
///       Some xkb readers drop keycodes above the declared maximum.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function raiseMaximum(keymap: string, atLeast: number): string { ... }
/// ```
fn raise_maximum(keymap: &str, minimum_maximum: u32) -> String {
    let mut out = String::with_capacity(keymap.len());
    for line in keymap.split_inclusive('\n') {
        let current = line.trim().strip_prefix("maximum = ").and_then(|rest| return rest.trim_end_matches(';').parse::<u32>().ok());
        match current {
            Some(value) if value < minimum_maximum => out.push_str(&format!("\tmaximum = {minimum_maximum};\n")),
            _ => out.push_str(line),
        }
    }
    return out
}

/// What:
///      `#[cfg(test)] #[path = "unicode_keymap_tests.rs"] mod tests;`.
/// Why:
///       Keep the keymap tests beside the builder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./unicode_keymap_tests";
/// ```
#[cfg(test)]
#[path = "unicode_keymap_tests.rs"]
mod tests;
