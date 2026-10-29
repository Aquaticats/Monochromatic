//! US-QWERTY character/key-name to Linux evdev keycode tables.
//!
//! Synthetic keyboard input is expressed as evdev keycodes (the values in
//! `linux/input-event-codes.h`).
//!  The compositor's keyboard uses the default US xkb
//! layout,
//!  so mapping a character to the keycode-plus-shift that produces it under that
//! layout is a fixed table.
//!  This is display-independent and unit-tested directly.

/// Evdev keycode of the left Shift modifier (`KEY_LEFTSHIFT`).
///
/// What:
///      `pub const LEFT_SHIFT: u32 = 42;`.
///  Unsigned 32-bit;
///  the raw evdev code.
/// Why:
///       Typing an uppercase or shifted character wraps the key tap in a Shift
///           press/release,
///  and the input layer needs this code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LEFT_SHIFT = 42;
/// ```
pub const LEFT_SHIFT: u32 = 42;

/// Map a character to `(evdev_keycode, needs_shift)` under the US layout.
///
/// What:
///      `pub fn char_to_key(character: char) -> Option<(u32, bool)>`.
///  Returns the
///           keycode and whether Shift must be held,
///  or `None` for characters not on a
///           US keyboard (which the caller skips).
/// Why:
///       `type <text>` turns each character into a key tap;
///  this is the lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function charToKey(character: string): [number, boolean] | undefined { ... }
/// ```
///
/// @example
/// ```ts
/// charToKey("A"); // => [30, true]  (KEY_A held with Shift)
/// charToKey("!"); // => [2, true]   (KEY_1 held with Shift)
/// ```
pub fn char_to_key(character: char) -> Option<(u32, bool)> {
    // What:     `if character.is_ascii_alphabetic() { ... }`. True for A-Z and a-z.
    // Why:      Letters share one code per letter; case only sets the Shift flag.
    if character.is_ascii_alphabetic() {
        // What:     `let code = letter_code(character.to_ascii_lowercase())?;`. Lowercase
        //           the letter, look up its code; `?` returns `None` if somehow absent.
        // Why:      The keycode is the same for `a` and `A`.
        let code = letter_code(character.to_ascii_lowercase())?;

        // What:     `Some((code, character.is_ascii_uppercase()))`. Shift is needed exactly
        //           when the character is uppercase. Tail expression.
        // Why:      Return the code plus whether to hold Shift.
        return Some((code, character.is_ascii_uppercase()));
    }

    // What:     `match character { ... }`. Map each non-letter character to its
    //           `(code, shift)`. Digits and space are unshifted; the shifted symbols on
    //           the number row and punctuation keys set `true`. Anything else is `None`.
    // Why:      Cover the printable US-keyboard characters a test is likely to type.
    match character {
        ' ' => return Some((57, false)),
        '\t' => return Some((15, false)),
        '\n' => return Some((28, false)),
        '1' => return Some((2, false)),
        '!' => return Some((2, true)),
        '2' => return Some((3, false)),
        '@' => return Some((3, true)),
        '3' => return Some((4, false)),
        '#' => return Some((4, true)),
        '4' => return Some((5, false)),
        '$' => return Some((5, true)),
        '5' => return Some((6, false)),
        '%' => return Some((6, true)),
        '6' => return Some((7, false)),
        '^' => return Some((7, true)),
        '7' => return Some((8, false)),
        '&' => return Some((8, true)),
        '8' => return Some((9, false)),
        '*' => return Some((9, true)),
        '9' => return Some((10, false)),
        '(' => return Some((10, true)),
        '0' => return Some((11, false)),
        ')' => return Some((11, true)),
        '-' => return Some((12, false)),
        '_' => return Some((12, true)),
        '=' => return Some((13, false)),
        '+' => return Some((13, true)),
        '[' => return Some((26, false)),
        '{' => return Some((26, true)),
        ']' => return Some((27, false)),
        '}' => return Some((27, true)),
        '\\' => return Some((43, false)),
        '|' => return Some((43, true)),
        ';' => return Some((39, false)),
        ':' => return Some((39, true)),
        '\'' => return Some((40, false)),
        '"' => return Some((40, true)),
        '`' => return Some((41, false)),
        '~' => return Some((41, true)),
        ',' => return Some((51, false)),
        '<' => return Some((51, true)),
        '.' => return Some((52, false)),
        '>' => return Some((52, true)),
        '/' => return Some((53, false)),
        '?' => return Some((53, true)),
        _ => return None,
    }
}

/// Map a lowercase ASCII letter to its evdev keycode.
///
/// What:
///      `fn letter_code(letter: char) -> Option<u32>`.
///  Private helper covering
///           `a`..=`z`.
/// Why:
///       Keep the 26-entry letter table out of `char_to_key`'s symbol match.
fn letter_code(letter: char) -> Option<u32> {
    // What:     `match letter { 'a' => Some(30), ... }`. The QWERTY letter-to-code table.
    // Why:      One authoritative place for letter keycodes.
    match letter {
        'a' => return Some(30),
        'b' => return Some(48),
        'c' => return Some(46),
        'd' => return Some(32),
        'e' => return Some(18),
        'f' => return Some(33),
        'g' => return Some(34),
        'h' => return Some(35),
        'i' => return Some(23),
        'j' => return Some(36),
        'k' => return Some(37),
        'l' => return Some(38),
        'm' => return Some(50),
        'n' => return Some(49),
        'o' => return Some(24),
        'p' => return Some(25),
        'q' => return Some(16),
        'r' => return Some(19),
        's' => return Some(31),
        't' => return Some(20),
        'u' => return Some(22),
        'v' => return Some(47),
        'w' => return Some(17),
        'x' => return Some(45),
        'y' => return Some(21),
        'z' => return Some(44),
        _ => return None,
    }
}

/// Map a key name (`enter`,
///  `f10`,
///  `code:68`,
///  or a single character) to its evdev keycode.
///
/// What:
///      `pub fn named_key(name: &str) -> Option<u32>`.
///  Delegates to `key_name::resolve`,
///  which knows every kernel key name,
///  the spoken aliases,
///  and the raw `code:<n>` form.
///  `None` for unknown names.
/// Why:
///       The `key <name>` command presses named keys and single characters alike;
///  the tables live in `key_name` and `key_table`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedKey(name: string): number | undefined { ... }
/// ```
///
/// @example
/// ```ts
/// namedKey("enter"); // => 28
/// namedKey("f10");   // => 68
/// ```
pub fn named_key(name: &str) -> Option<u32> {
    return crate::key_name::resolve(name)
}

/// What:
///      `#[cfg(test)] #[path = "keymap_tests.rs"] mod tests;`.
///  Declares the keymap
///           unit test module from the sibling file.
/// Why:
///       Keep the keycode-table tests beside the tables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./keymap_tests";
/// ```
#[cfg(test)]
#[path = "keymap_tests.rs"]
mod tests;
