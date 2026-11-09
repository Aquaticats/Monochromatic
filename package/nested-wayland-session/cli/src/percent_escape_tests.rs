//! Pins the one percent escape that the job text,
//! the launcher URI,
//! and the dropped-file URI share.
//!
//! What:
//!      The escape against the standard formatter,
//!  and each format's own documented example.
//! Why:
//!       A change of percent prefix,
//!  hexadecimal case,
//!  or digit order would change every answer a client reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(encodeText("Done; 100 %")).toBe("Done%3B%20100%20%25");
//! ```

/// Brings the escape under test into scope.
use super::*;
/// The launcher answer's URI format.
use crate::launcher_format::encode_uri;
/// The dropped-file URI format.
use crate::uri_list::file_uri;
/// The job and notification text format.
use crate::wire_text::encode_text;
/// Pinned URI examples are paths.
use std::path::Path;

/// Accepts every byte,
///  so the caller can show that accepted bytes are copied unchanged.
fn accept_every_byte(_byte: u8) -> bool {
    return true;
}

/// Refuses every byte,
///  so the caller can compare the escape with the standard formatter byte by byte.
fn refuse_every_byte(_byte: u8) -> bool {
    return false;
}

/// The shared escape agrees with the standard uppercase formatting on every byte.
#[test]
fn every_byte_matches_the_standard_uppercase_formatting() {
    for byte in u8::MIN..=u8::MAX {
        assert_eq!(percent_escape(&[byte], refuse_every_byte), format!("%{byte:02X}"), "byte {byte}");
    }
}

/// Accepted bytes are copied and refused bytes are always escaped.
#[test]
fn accepted_bytes_are_copied_unchanged() {
    assert_eq!(percent_escape(b"aZ0-._~", accept_every_byte), "aZ0-._~");
    assert_eq!(percent_escape(b"", refuse_every_byte), "");
    assert_eq!(percent_escape(b" /", refuse_every_byte), "%20%2F");
}

/// The job text,
///  the launcher URI,
///  and the dropped-file URI all spell their escapes with the shared digits.
#[test]
fn the_three_formats_escape_their_separators_with_the_shared_digits() {
    assert_eq!(encode_text("Done; 100 %"), "Done%3B%20100%20%25");
    assert_eq!(encode_uri("application://a b.desktop"), "application://a%20b.desktop");
    assert_eq!(file_uri(Path::new("/music/a b#1.mp3")), "file:///music/a%20b%231.mp3");
}
