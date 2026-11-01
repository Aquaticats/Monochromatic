//! What:
//!  The shapes of a Git object read in process, object names as bytes and hexadecimal, and the zlib step.
//! Why:
//!  Loose objects and pack entries are zlib streams; every reader of this family shares one inflate call
//!  (a pure Rust decoder) and one definition of an object name, so a size or format mismatch is refused in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type RawObject = { kind: 'commit' | 'tree' | 'blob' | 'tag'; bytes: Uint8Array };
//! ```

use super::repo_read_layout::ObjectFormat;
use miniz_oxide::inflate::decompress_to_vec_zlib_with_limit;

/// The largest object this family inflates; anything bigger is left to Git.
pub const OBJECT_SIZE_LIMIT: usize = 256 << 20;

/// The four kinds of Git object.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectKind {
    /// A commit.
    Commit,
    /// A tree.
    Tree,
    /// A blob.
    Blob,
    /// An annotated tag.
    Tag,
}

impl ObjectKind {
    /// The word Git writes in an object header and `cat-file -t` prints.
    pub fn word(self) -> &'static str {
        return match self {
            ObjectKind::Commit => "commit",
            ObjectKind::Tree => "tree",
            ObjectKind::Blob => "blob",
            ObjectKind::Tag => "tag",
        };
    }

    /// The kind a loose-object header word names.
    pub fn from_word(word: &[u8]) -> Option<ObjectKind> {
        return match word {
            b"commit" => Some(ObjectKind::Commit),
            b"tree" => Some(ObjectKind::Tree),
            b"blob" => Some(ObjectKind::Blob),
            b"tag" => Some(ObjectKind::Tag),
            _ => None,
        };
    }
}

/// One object with its kind and its content, without the header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawObject {
    /// What the object is.
    pub kind: ObjectKind,
    /// The content bytes.
    pub bytes: Vec<u8>,
}

/// The length in bytes of an object name.
pub fn name_length(format: ObjectFormat) -> usize {
    return format.hex_length() / 2;
}

/// The lower-case hexadecimal spelling of an object name.
pub fn to_hex(name: &[u8]) -> String {
    let mut text: String = String::with_capacity(name.len() * 2);
    for byte in name {
        text.push(char::from(b"0123456789abcdef"[usize::from(byte >> 4)]));
        text.push(char::from(b"0123456789abcdef"[usize::from(byte & 15)]));
    }
    return text;
}

/// The raw object name a hexadecimal spelling of the right length stands for.
pub fn from_hex(text: &[u8], format: ObjectFormat) -> Option<Vec<u8>> {
    if text.len() != format.hex_length() {
        return None;
    }
    let digit = |byte: u8| -> Option<u8> {
        return match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        };
    };
    let mut name: Vec<u8> = Vec::with_capacity(text.len() / 2);
    for pair in text.chunks(2) {
        name.push(digit(pair[0])? << 4 | digit(pair[1])?);
    }
    return Some(name);
}

/// What:
///  Read a loose object file's bytes: the zlib stream of `<kind> <size>\0<content>`; the kind and content come back
///  only when the header's size equals the content length.
/// Why:
///   A loose file that disagrees with its own header is corrupt, and Git words that failure itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const object = parseLoose(await readFile(path)); // undefined: ask Git
/// ```
pub fn parse_loose(file_bytes: &[u8]) -> Option<RawObject> {
    let whole: Vec<u8> = decompress_to_vec_zlib_with_limit(file_bytes, OBJECT_SIZE_LIMIT + 32).ok()?;
    let nul: usize = whole.iter().position(|byte| { return *byte == 0; })?;
    let header: &[u8] = &whole[..nul];
    let space: usize = header.iter().position(|byte| { return *byte == b' '; })?;
    let kind: ObjectKind = ObjectKind::from_word(&header[..space])?;
    let size_text: &str = std::str::from_utf8(&header[space + 1..]).ok()?;
    if size_text.is_empty() || (size_text.len() > 1 && size_text.starts_with('0')) || !size_text.bytes().all(|byte| { return byte.is_ascii_digit(); }) {
        return None;
    }
    let size: usize = size_text.parse::<usize>().ok()?;
    let content: &[u8] = &whole[nul + 1..];
    if content.len() != size {
        return None;
    }
    return Some(RawObject { kind, bytes: content.to_vec() });
}

/// What:
///  Inflate a zlib stream whose result must be exactly `expected` bytes; trailing input is ignored.
/// Why:
///   Pack entries do not record their compressed length, so the caller hands a window that may run past the stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const out = inflateZlib(window, expected); // undefined when the stream is bad, short or too long
/// ```
pub fn inflate_exact(window: &[u8], expected: usize) -> Option<Vec<u8>> {
    if expected > OBJECT_SIZE_LIMIT {
        return None;
    }
    let inflated: Vec<u8> = decompress_to_vec_zlib_with_limit(window, expected).ok()?;
    if inflated.len() != expected {
        return None;
    }
    return Some(inflated);
}
