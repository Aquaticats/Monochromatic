//! What:
//!  Controls for the conversions of exact path bytes.
//! Why:
//!  A name that is not UTF-8 must reach Git byte for byte,
//!  and a message must show the closest readable text.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(Buffer.from(osString)).toEqual(bytes);
//! ```

/// The conversions under test.
use super::{os_from_path_bytes, path_message_text};

/// Unix keeps every byte of a name, invalid UTF-8 included.
#[cfg(unix)]
#[test]
fn an_operating_system_string_keeps_every_byte() {
    use std::os::unix::ffi::OsStrExt;
    let every: Vec<u8> = (1..=255).collect();
    assert_eq!(os_from_path_bytes(every.as_slice()).as_bytes(), every.as_slice());
    assert_eq!(os_from_path_bytes(b"caf\xe9.txt").as_bytes(), b"caf\xe9.txt");
    assert_eq!(os_from_path_bytes(b"").as_bytes(), b"");
}

/// A message shows a UTF-8 name as it is and replaces each invalid sequence.
#[test]
fn a_message_shows_replacement_characters_for_invalid_bytes() {
    assert_eq!(path_message_text("doc/café.md".as_bytes()), "doc/café.md");
    assert_eq!(path_message_text(b"caf\xe9.txt"), "caf\u{fffd}.txt");
    assert_eq!(path_message_text(b""), "");
}
