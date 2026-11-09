//! What:
//!  Pins the one lowercase hexadecimal spelling that a digest directory name and a redacted label share.
//! Why:
//!  A change of alphabet,
//!  case,
//!  or nibble order would silently rename every cache directory and every escaped path label.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Compare against the standard formatter and against a hand-written label.
//! ```

/// Brings the encoder under test into scope.
use super::*;
/// The display label that escapes invalid pathname bytes.
use crate::path_name_bytes::safe_component;

/// The shared alphabet agrees with the standard formatter on every byte,
///  so the label escapes and the digest names stay the same bytes.
#[test]
fn every_byte_matches_the_standard_lowercase_formatting() {
    for byte in u8::MIN..=u8::MAX {
        let mut spelled: String = String::new();
        push_lowercase_hex_byte(&mut spelled, byte);
        assert_eq!(spelled, format!("{byte:02x}"), "byte {byte}");
    }
}

/// A whole slice pairs its high nibble first,
///  in lowercase,
///  with no separator.
#[test]
fn slice_spelling_pairs_the_high_nibble_first() {
    assert_eq!(bytes_as_lowercase_hex(b""), "");
    assert_eq!(bytes_as_lowercase_hex(b"\x00\x0f\xa5\xff\x10"), "000fa5ff10");
}

/// The redacted label of a pathname with invalid UTF-8 spells those bytes with the shared digits.
#[test]
fn the_display_label_escapes_invalid_bytes_with_the_shared_digits() {
    assert_eq!(safe_component(b"\xff"), "\\xff");
    assert_eq!(safe_component(b"ok\xffz\xfe"), "ok\\xffz\\xfe");
}
