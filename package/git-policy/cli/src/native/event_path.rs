//! What:
//!  A repository path as an event reports it:
//!  readable text,
//!  and the exact bytes
//!       as well when those bytes are not UTF-8.
//! Why:
//!  Git allows any bytes in a file name,
//!  and a JSON string can hold only text.
//!  An
//!      event keeps `path` as text with replacement characters,
//!  so every existing reader
//!      works unchanged,
//!  and adds an optional `pathBytes` field,
//!  base64 encoded,
//!  only for
//!      a name that is not UTF-8,
//!  so a reader that needs the file can still find it.
//!      This is option C of `doc/planning/cli-git-rust-open-decisions.md`,
//!  section
//!      "Non-UTF-8 paths in events and journals",
//!  which the owner adopted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const path = eventPath(bytes); line.path = path.text; if (path.exact) line.pathBytes = base64(path.exact);
//! ```

/// What:
///  The 64 characters of the standard base64 alphabet,
///  in value order.
///       `&[u8; 64]` is a fixed-length byte array baked into the program.
/// Why:
///   RFC 4648 section 4 is the alphabet every common decoder reads by default
///       (`Buffer.from(text, 'base64')`,
///  Python's `base64.b64decode`,
///  Go's `[]byte` JSON).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
/// ```
const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// What:
///  One path as events report it.
///  The fields are private,
///  so the only ways to build
///       one are the two constructors,
///  and `exact` is present exactly when the bytes
///       were not UTF-8.
///  `#[derive(...)]` generates cloning,
///  debug printing and `==`.
/// Why:
///   A reader must be able to rely on "no `pathBytes` means `path` is the exact name".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EventPath = { readonly text: string; readonly exact?: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventPath {
    /// The path as text;
    ///  bytes that are not UTF-8 are replaced by U+FFFD.
    text: String,
    /// The exact bytes,
    ///  kept only when they are not UTF-8.
    exact: Option<Vec<u8>>,
}

/// What:
///  `impl EventPath { ... }` attaches the constructors and the two reads.
/// Why:
///   The invariant between the two fields is kept in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class EventPath { static fromGitBytes(bytes) {} static display(text) {} }
/// ```
impl EventPath {
    /// What:
    ///  The event form of a pathname exactly as Git printed it.
    ///  `&[u8]` borrows the
    ///       bytes;
    ///  `String::from_utf8_lossy` replaces each invalid sequence by U+FFFD.
    /// Why:
    ///   A UTF-8 name is its own exact form and gets no extra field;
    ///  any other name
    ///       keeps its bytes,
    ///  because different names can share one replaced text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromGitBytes(bytes: Uint8Array): EventPath;
    /// ```
    pub fn from_git_bytes(bytes: &[u8]) -> EventPath {
        // `match` on the strict decoding: text keeps no copy of the bytes, anything else does.
        match std::str::from_utf8(bytes) {
            Ok(text) => {
                return EventPath {
                    text: String::from(text),
                    exact: None,
                };
            }
            Err(_) => {
                return EventPath {
                    text: String::from_utf8_lossy(bytes).into_owned(),
                    // `.to_vec()` copies the borrowed bytes into an owned list.
                    exact: Some(bytes.to_vec()),
                };
            }
        }
    }

    /// What:
    ///  A path whose event form is text only.
    ///  `&str` borrows the text.
    /// Why:
    ///   A constant name,
    ///  or the scanner's masked display of a name,
    ///  is already text;
    ///       the masked display must never carry the bytes it masks.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static display(text: string): EventPath;
    /// ```
    pub fn display(text: &str) -> EventPath {
        return EventPath {
            text: String::from(text),
            exact: None,
        };
    }

    /// What:
    ///  The readable text of the path.
    ///  `&str` borrows it from the record.
    /// Why:
    ///   Every event prints it as `path`,
    ///  whatever else it prints.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get text(): string;
    /// ```
    pub fn text(&self) -> &str {
        return self.text.as_str();
    }

    /// What:
    ///  The exact bytes,
    ///  only when they are not UTF-8.
    ///  `Option<&[u8]>` lends them or
    ///       says there are none.
    /// Why:
    ///   Rendering adds `pathBytes` exactly when this is present.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get exact(): Uint8Array | undefined;
    /// ```
    pub fn exact(&self) -> Option<&[u8]> {
        // `.as_deref()` lends the owned list inside the option as a borrowed slice.
        return self.exact.as_deref();
    }

    /// What:
    ///  The bytes of the name:
    ///  the exact ones when kept,
    ///  else the text's UTF-8 bytes.
    /// Why:
    ///   A list of paths that holds one name that is not UTF-8 lists the bytes of every
    ///       name,
    ///  so its entries line up with the readable list one for one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get bytes(): Uint8Array { return this.exact ?? new TextEncoder().encode(this.text); }
    /// ```
    pub fn bytes(&self) -> &[u8] {
        match &self.exact {
            Some(exact) => return exact.as_slice(),
            None => return self.text.as_bytes(),
        }
    }
}

/// What:
///  The alphabet character for the low six bits of `value`.
///  `u32` is an unsigned
///       32-bit integer;
///  `char::from` turns one ASCII byte into a character.
/// Why:
///   Each base64 character carries six bits of the input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sextet = (value: number) => ALPHABET[value & 63];
/// ```
fn sextet(value: u32) -> char {
    // `as usize` widens the masked value to an index; `& 63` keeps the low six bits.
    return char::from(BASE64_ALPHABET[(value & 63) as usize]);
}

/// What:
///  Encode bytes as standard base64 with `=` padding (RFC 4648 section 4).
/// Why:
///   `pathBytes` carries exact pathname bytes inside a JSON string.
///  Base64 is what
///       ripgrep's JSON output uses for a path that is not UTF-8 and what Go's JSON
///       encoding uses for bytes;
///  it is unmistakably not a name,
///  so no reader uses it as
///       one by accident.
///  Every three input bytes become four characters;
///  a final group
///       of one or two bytes is padded to four characters with `=`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function base64(bytes: Uint8Array): string { return Buffer.from(bytes).toString('base64'); }
/// ```
pub fn base64_standard(bytes: &[u8]) -> String {
    let mut encoded: String = String::new();
    // `.chunks(3)` lends the input three bytes at a time; the last group may be shorter.
    for group in bytes.chunks(3) {
        // `u32::from` widens each byte; `<<` shifts it into its place in the 24-bit group.
        let first: u32 = u32::from(group[0]) << 16;
        let second: u32 = match group.get(1) {
            Some(byte) => u32::from(*byte) << 8,
            None => 0,
        };
        let third: u32 = match group.get(2) {
            Some(byte) => u32::from(*byte),
            None => 0,
        };
        let joined: u32 = first | second | third;
        encoded.push(sextet(joined >> 18));
        encoded.push(sextet(joined >> 12));
        if group.len() > 1 {
            encoded.push(sextet(joined >> 6));
        } else {
            encoded.push('=');
        }
        if group.len() > 2 {
            encoded.push(sextet(joined));
        } else {
            encoded.push('=');
        }
    }
    return encoded;
}

/// What:
///  The value of one base64 alphabet character,
///  or nothing for any other byte.
///  `Option<u32>` is a value that may be absent.
/// Why:
///   Decoding reads one character at a time and must refuse everything outside the alphabet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sextetValue = (character: string) => ALPHABET.indexOf(character) // -1 when absent
/// ```
fn sextet_value(character: u8) -> Option<u32> {
    // `.position(..)` is the index of the first match; `as u32` widens the index.
    return BASE64_ALPHABET
        .iter()
        .position(|candidate| { return *candidate == character; })
        .map(|index| { return index as u32; });
}

/// What:
///  Decode standard padded base64 (RFC 4648 section 4) into bytes,
///  or nothing when the text is not exactly what `base64_standard` would print for some bytes.
/// Why:
///   A journal that holds the exact bytes of a name that is not UTF-8 is read back by the wrapper;
///  a text that
///       decodes in more than one way (stray padding, non-zero spare bits) is a damaged record,
///  not a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function base64Decode(text: string): Uint8Array | undefined { const bytes = Buffer.from(text, 'base64'); return bytes.toString('base64') === text ? bytes : undefined; }
/// ```
pub fn base64_decode_standard(text: &str) -> Option<Vec<u8>> {
    let characters: &[u8] = text.as_bytes();
    if characters.len() % 4 != 0 {
        return None;
    }
    let mut decoded: Vec<u8> = Vec::with_capacity(characters.len() / 4 * 3);
    // `.chunks(4)` lends the input four characters at a time.
    for group in characters.chunks(4) {
        let mut joined: u32 = 0;
        let mut data_characters: usize = 0;
        for character in group {
            joined <<= 6;
            if *character == b'=' {
                continue;
            }
            joined |= sextet_value(*character)?;
            data_characters += 1;
        }
        // Two data characters carry one byte, three carry two, four carry three; fewer carry none.
        if data_characters < 2 {
            return None;
        }
        decoded.push((joined >> 16) as u8);
        if data_characters > 2 {
            decoded.push((joined >> 8) as u8);
        }
        if data_characters > 3 {
            decoded.push(joined as u8);
        }
    }
    // Printing the bytes again must give the same text; padding in the wrong place or spare bits that are not zero
    // fail here.
    if base64_standard(decoded.as_slice()) != text {
        return None;
    }
    return Some(decoded);
}

/// Construction,
///  invariant and encoding controls stay out of the release executable.
#[cfg(test)]
#[path = "event_path_tests.rs"]
mod tests;
