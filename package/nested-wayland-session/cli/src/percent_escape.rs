//! Percent-escaping of client-chosen bytes in the one-line answer grammars.
//!
//! What:
//!      The escape every answer format shares.
//! Why:
//!       The formats differ only in which bytes pass through unchanged,
//!  and a second copy of the escape would let one format's percent prefix,
//!  hexadecimal case,
//!  or digit order drift from the others.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function percentEscape(bytes: Buffer, isSafe: (byte: number) => boolean): string;
//! ```

/// Uppercase hexadecimal alphabet,
///  indexed by one nibble.
const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Escapes every byte `is_safe` refuses as `%` and two uppercase hexadecimal digits.
///
/// What:
///      `pub(crate) fn percent_escape(bytes: &[u8], is_safe: fn(u8) -> bool) -> String`.
///  Accepted bytes are copied,
///  and refused bytes,
///  including every non-ASCII byte,
///  are escaped one at a time.
/// Why:
///       The answer grammars separate fields with bytes a client can send,
///  so a client-chosen string may carry only the separators its format documents.
pub(crate) fn percent_escape(bytes: &[u8], is_safe: fn(u8) -> bool) -> String {
    let mut escaped: String = String::with_capacity(bytes.len());
    for byte in bytes {
        if is_safe(*byte) {
            escaped.push(char::from(*byte));
            continue;
        }
        escaped.push('%');
        escaped.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
        escaped.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
    }
    return escaped;
}

/// Pins the escape and the three formats that share it.
#[cfg(test)]
#[path = "percent_escape_tests.rs"]
mod tests;
