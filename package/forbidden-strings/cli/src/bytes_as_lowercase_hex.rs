//! What:
//!  Lowercase hexadecimal spelling of bytes.
//! Why:
//!  The content-digest directory name and the redacted label of a pathname with invalid UTF-8 both spell bytes this way,
//!  so one place owns the alphabet,
//!  the nibble order,
//!  and the case.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Two lowercase digits per byte, high nibble first, no separator.
//! ```

/// Digits one byte is spelled with.
const DIGITS_PER_BYTE: usize = 2;
/// Lowercase hexadecimal alphabet,
///  indexed by one nibble.
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Appends the two lowercase hexadecimal digits of `byte`,
///  high nibble first.
pub(crate) fn push_lowercase_hex_byte(out: &mut String, byte: u8) {
    out.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
    out.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
}

/// Spells `bytes` as lowercase hexadecimal digits,
///  high nibble first,
///  with no separator.
pub(crate) fn bytes_as_lowercase_hex(bytes: &[u8]) -> String {
    let mut spelled: String = String::with_capacity(bytes.len() * DIGITS_PER_BYTE);
    for byte in bytes {
        push_lowercase_hex_byte(&mut spelled, *byte);
    }
    return spelled;
}

/// Pins the spelling and its two callers.
#[cfg(test)]
#[path = "bytes_as_lowercase_hex_tests.rs"]
mod tests;
