//! What:
//!  The shapes of a Git object read in process,
//! object names as bytes and hexadecimal,
//! and the zlib step.
//! Why:
//!  Loose objects and pack entries are zlib streams;
//! every reader of this family shares one inflate call
//!  (a pure Rust decoder) and one definition of an object name,
//! so a size or format mismatch is refused in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type RawObject = { kind: 'commit' | 'tree' | 'blob' | 'tag'; bytes: Uint8Array };
//! ```

use crate::index_byte_search::first_index_of_byte;
use crate::repo_read_layout::ObjectFormat;
use miniz_oxide::inflate::TINFLStatus;
use miniz_oxide::inflate::core::{DecompressorOxide, decompress, inflate_flags};

/// The largest object this family inflates;
/// anything bigger is left to Git.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a file,
/// chain or count past this bound makes the reader
///  decline and real Git answer,
/// so a Git start is added and no result changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
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

/// One object with its kind and its content,
/// without the header.
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
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(name);
}

/// The raw object name a hexadecimal spelling of the right length stands for.
pub fn from_hex(text: &[u8], format: ObjectFormat) -> Option<Vec<u8>> {
    if text.len() != format.hex_length() {
        return None;
    }
    let mut name: Vec<u8> = Vec::with_capacity(text.len() / 2);
    for pair in text.chunks(2) {
        name.push(hex_digit_value(pair[0])? << 4 | hex_digit_value(pair[1])?);
    }
    return Some(name);
}

/// Whether `byte` is an ASCII decimal digit.
fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// The value of one lower-case hexadecimal digit,
/// or nothing for any other byte.
fn hex_digit_value(byte: u8) -> Option<u8> {
    return match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    };
}

/// The most compressed bytes of one stream the inflater looks at;
/// a longer stream is left to Git.
///  A block that produces nothing still costs a table rebuild in the decoder (upstream issue 198 of the decoder crate),
///  so the cost of a hostile stream grows with its length,
/// not with its output;
/// real commits,
/// trees and small blobs
///  are far below the bound,
/// and Git inflates the rest.
pub const INFLATE_INPUT_LIMIT: usize = 512 << 10;

/// The smallest output buffer the inflater starts with,
/// so an empty input still grows.
const FIRST_OUTPUT_BYTES: usize = 64;

/// What:
///  Inflate the zlib stream at the start of `input` into at most `limit` bytes,
/// and report how many input bytes the stream
///  used,
/// its checksum included.
/// Why:
///   Git refuses a loose object with bytes after its stream (`garbage at end of loose object`),
/// so the loose reader needs
///  the length,
/// which the decoder's convenience functions do not return;
/// a pack window runs past its stream on purpose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { output, consumed } = inflateZlib(input, limit); // undefined: bad, short or longer than the limit
/// ```
pub fn inflate_zlib(whole_input: &[u8], limit: usize) -> Option<(Vec<u8>, usize)> {
    let input: &[u8] = &whole_input[..whole_input.len().min(INFLATE_INPUT_LIMIT)];
    let flags: u32 = inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER | inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
    let mut output: Vec<u8> = vec![0; input.len().saturating_mul(2).min(limit)];
    let mut state: Box<DecompressorOxide> = Box::<DecompressorOxide>::default();
    let mut produced: usize = 0;
    let mut used: usize = 0;
    loop {
        let (status, consumed, written) = decompress(&mut state, input.get(used..)?, &mut output, produced, flags);
        used = used.checked_add(consumed)?;
        produced = produced.checked_add(written)?;
        match status {
            TINFLStatus::Done => {
                output.truncate(produced);
                return Some((output, used));
            }
            TINFLStatus::HasMoreOutput => {
                if output.len() >= limit {
                    return None;
                }
                let grown: usize = output.len().saturating_mul(2).max(FIRST_OUTPUT_BYTES).min(limit);
                output.resize(grown, 0);
            }
            _ => return None,
        }
    }
}

/// What:
///  Read a loose object file's bytes:
/// the zlib stream of `<kind> <size>\0<content>`;
/// the kind and content come back
///  only when the header's size equals the content length.
/// Why:
///   A loose file that disagrees with its own header is corrupt,
/// and Git words that failure itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const object = parseLoose(await readFile(path)); // undefined: ask Git
/// ```
pub fn parse_loose(file_bytes: &[u8]) -> Option<RawObject> {
    let (whole, consumed): (Vec<u8>, usize) = inflate_zlib(file_bytes, OBJECT_SIZE_LIMIT + 32)?;
    // Bytes after the stream are Git's `garbage at end of loose object`.
    if consumed != file_bytes.len() {
        return None;
    }
    let nul: usize = first_index_of_byte(&whole, 0)?;
    let header: &[u8] = &whole[..nul];
    let space: usize = first_index_of_byte(header, b' ')?;
    let kind: ObjectKind = ObjectKind::from_word(&header[..space])?;
    let size_text: &str = std::str::from_utf8(&header[space + 1..]).ok()?;
    if size_text.is_empty() || (size_text.len() > 1 && size_text.starts_with('0')) || !size_text.bytes().all(is_digit_byte) {
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
///  Inflate a zlib stream whose result must be exactly `expected` bytes;
/// trailing input is ignored.
/// Why:
///   Pack entries do not record their compressed length,
/// so the caller hands a window that may run past the stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const out = inflateZlib(window, expected); // undefined when the stream is bad, short or too long
/// ```
pub fn inflate_exact(window: &[u8], expected: usize) -> Option<Vec<u8>> {
    if expected > OBJECT_SIZE_LIMIT {
        return None;
    }
    let (inflated, _): (Vec<u8>, usize) = inflate_zlib(window, expected)?;
    if inflated.len() != expected {
        return None;
    }
    return Some(inflated);
}

#[cfg(test)]
#[path = "repo_read_object_tests.rs"]
mod tests;
