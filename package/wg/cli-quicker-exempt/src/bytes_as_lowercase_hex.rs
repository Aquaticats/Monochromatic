//! Lowercase hexadecimal spelling of arbitrary bytes.
//!
//! The bpffs pin key and the runtime watcher state directory both name their files this way,
//!  so one place owns the alphabet,
//!  the nibble order,
//! and the case.

/// Digits one byte is spelled with.
pub const DIGITS_PER_BYTE: usize = 2;
/// Lowercase hexadecimal alphabet,
///  indexed by one nibble.
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Spells `bytes` as lowercase hexadecimal digits,
///  high nibble first,
///  with no separator.
///
/// What:
///      `pub fn bytes_as_lowercase_hex(bytes: &[u8]) -> String`.
/// Why:
///       Every caller uses the result as one path component,
///  so the spelling must be injective over bytes,
///  hold no `/` and no `.`,
///  and stay identical wherever it is produced.
pub fn bytes_as_lowercase_hex(bytes: &[u8]) -> String {
    let mut spelled: String = String::with_capacity(bytes.len() * DIGITS_PER_BYTE);
    for byte in bytes {
        push_lowercase_hex_byte(&mut spelled, *byte);
    }
    return spelled;
}

/// Appends the two lowercase hexadecimal digits of `byte`,
///  high nibble first.
///
/// What:
///      `pub fn push_lowercase_hex_byte(out: &mut String, byte: u8)`.
/// Why:
///       A unit name escapes one byte at a time after a `\x` prefix,
///  and it must spell that byte exactly as the path components do.
pub fn push_lowercase_hex_byte(out: &mut String, byte: u8) {
    out.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
    out.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
}

/// Unit tests for the spelling and for the two call sites that share it.
#[cfg(test)]
#[path = "bytes_as_lowercase_hex_tests.rs"]
mod tests;
