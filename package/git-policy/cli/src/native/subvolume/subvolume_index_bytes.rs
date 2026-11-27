//! What:
//!  Reading a Git index file's bytes:
//!  the header,
//!  the fixed stat fields of an entry,
//!  the walk over the entries,
//!  the extensions that follow them,
//!  and which hash verifies the checksum.
//! Why:
//!  Entry lengths never change,
//!  so the rewrite edits an entry in place,
//!  and it may only do that once the whole file is known to parse:
//!  the version decides how a path is stored,
//!  the hash length decides where the entries end,
//!  and an index whose checksum is all zero proves its hash length only by parsing to its end.
//!  Nothing here writes;
//!  a shape this reader does not know is named and the index is left as it is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries = walkEntries(bytes, hashLength);            // undefined when the layout is unknown
//! const length = checksumKind(bytes, knownLength);          // 20 or 32, and whether the checksum is all zero
//! ```

use crate::subvolume_sha::{
    sha1,
    sha256,
};

/// The bytes of the header:
/// signature,
/// version and entry count.
pub(crate) const HEADER: usize = 12;

/// The bytes of the fixed stat fields of an entry,
/// before the object name.
pub(crate) const STAT: usize = 40;

/// The extension signatures that are dropped:
/// derived state that Git rebuilds.
pub(super) const DROPPED: &[&[u8; 4]] = &[b"UNTR", b"FSMN", b"EOIE", b"IEOT"];

/// The extension signatures that make the rewrite decline:
/// they change what the entries mean.
pub(super) const DECLINED: &[&[u8; 4]] = &[b"link", b"sdir"];

/// One entry found by the walk.
pub(crate) struct Located {
    /// Offset of the entry's first byte.
    pub(crate) start: usize,
    /// The path bytes.
    pub(crate) path: Vec<u8>,
    /// The entry's flags.
    pub(crate) flags: u16,
    /// The entry's mode.
    pub(crate) mode: u32,
}

/// A big-endian `u32` at `at`,
/// or nothing past the end.
pub(crate) fn be32(
    bytes: &[u8],
    at: usize,
) -> Option<u32> {
    let quad: &[u8] = bytes.get(at..at.checked_add(4)?)?;
    return Some(u32::from_be_bytes([quad[0], quad[1], quad[2], quad[3]]));
}

/// A big-endian `u16` at `at`,
/// or nothing past the end.
fn be16(
    bytes: &[u8],
    at: usize,
) -> Option<u16> {
    let pair: &[u8] = bytes.get(at..at.checked_add(2)?)?;
    return Some(u16::from_be_bytes([pair[0], pair[1]]));
}

/// What:
///  Walk the entries of an index whose checksum is `hash_length` bytes;
/// return them and the offset where they end.
/// Why:
///   Entry layout depends on the hash length (the object name) and the version (v4 compresses the path against the
///  previous one,
/// v2 and v3 pad to 8 bytes).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walkEntries(bytes: Uint8Array, hashLength: number): { entries: Located[]; end: number } | undefined;
/// ```
pub(crate) fn walk_entries(
    bytes: &[u8],
    hash_length: usize,
) -> Option<(Vec<Located>, usize)> {
    let version: u32 = be32(bytes, 4)?;
    let count: usize = usize::try_from(be32(bytes, 8)?).ok()?;
    let limit: usize = bytes.len().checked_sub(hash_length)?;
    let mut found: Vec<Located> = Vec::with_capacity(count.min(1 << 20));
    let mut at: usize = HEADER;
    let mut previous: Vec<u8> = Vec::new();
    for _ in 0..count {
        let mode: u32 = be32(bytes, at.checked_add(24)?)?;
        let flags_at: usize = at.checked_add(STAT + hash_length)?;
        let flags: u16 = be16(bytes, flags_at)?;
        let mut name_at: usize = flags_at + 2;
        if flags & 0x4000 != 0 {
            if version < 3 {
                return None;
            }
            name_at += 2;
        }
        let path: Vec<u8>;
        let next: usize;
        if version == 4 {
            let mut cursor: usize = name_at;
            let mut byte: u8 = *bytes.get(cursor)?;
            cursor += 1;
            let mut strip: usize = usize::from(byte & 0x7f);
            while byte & 0x80 != 0 {
                byte = *bytes.get(cursor)?;
                cursor += 1;
                strip = strip.checked_add(1)?.checked_shl(7)? | usize::from(byte & 0x7f);
            }
            let keep: usize = previous.len().checked_sub(strip)?;
            let end: usize = cursor + first_nul_at(bytes.get(cursor..limit)?)?;
            let mut joined: Vec<u8> = previous[..keep].to_vec();
            joined.extend_from_slice(&bytes[cursor..end]);
            path = joined;
            next = end + 1;
        } else {
            let length: usize = first_nul_at(bytes.get(name_at..limit)?)?;
            path = bytes[name_at..name_at + length].to_vec();
            next = at.checked_add((name_at - at + length + 8) & !7)?;
        }
        if next > limit {
            return None;
        }
        previous = path.clone();
        found.push(Located {
            start: at,
            path,
            flags,
            mode,
        });
        at = next;
    }
    return Some((found, at));
}

/// Whether `byte` is zero.
fn is_zero_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// The position of the first NUL byte of `bytes`,
/// or `None` when there is none.
fn first_nul_at(bytes: &[u8]) -> Option<usize> {
    for (position, stored) in bytes.iter().enumerate() {
        if *stored == 0 {
            return Some(position);
        }
    }
    return None;
}

/// Whether `signature` is one of the four-byte extension names in `names`.
pub(super) fn is_listed(
    names: &[&[u8; 4]],
    signature: &[u8],
) -> bool {
    for name in names {
        if signature == name.as_slice() {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether the extensions that follow the entries of an index whose checksum is `hash_length` bytes end exactly where
///  the checksum begins.
/// Why:
///   An index with an all-zero checksum proves nothing about its hash length,
/// so a length is accepted only when the
///  whole file parses under it;
/// a wrong length almost always leaves a cut-off extension.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function extensionsFit(bytes: Uint8Array, entriesEnd: number, hashLength: number): boolean;
/// ```
fn extensions_fit(
    bytes: &[u8],
    entries_end: usize,
    hash_length: usize,
) -> bool {
    let Some(limit) = bytes.len().checked_sub(hash_length) else {
        return false;
    };
    let mut at: usize = entries_end;
    while at < limit {
        let Some(size) = be32(bytes, at + 4) else {
            return false;
        };
        let Some(header_end) = at.checked_add(8) else {
            return false;
        };
        let Ok(extension_size) = usize::try_from(size) else {
            return false;
        };
        let Some(end) = header_end.checked_add(extension_size) else {
            return false;
        };
        if end > limit {
            return false;
        }
        at = end;
    }
    return at == limit;
}

/// What:
///  Which hash verifies the checksum,
/// as a hash length:
/// 20 for SHA-1,
/// 32 for SHA-256,
/// and for an all-zero checksum
///  (`index.skipHash`) the one length under which the whole file parses.
///  `known` is the repository's hash length when the caller read it from the repository's configuration.
/// Why:
///   A checksum that verifies proves its length.
/// An all-zero checksum proves nothing,
/// so the repository's object
///  format decides when it is known;
/// when it is not,
/// a file that parses under both lengths is ambiguous and is
///  declined,
/// because a wrong guess drops or keeps the wrong bytes of every entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checksumLength(bytes: Uint8Array, known?: number): { length: number; zero: boolean } | { declined: string };
/// ```
pub(crate) fn checksum_kind(
    bytes: &[u8],
    known: Option<usize>,
) -> Result<(usize, bool), String> {
    let candidates: Vec<usize> = match known {
        Some(length) => vec![length],
        None => vec![20, 32],
    };
    for length in &candidates {
        let verifies: bool = match length {
            20 => {
                bytes.len() >= HEADER + 20
                    && sha1(&bytes[..bytes.len() - 20]) == bytes[bytes.len() - 20..]
            }
            32 => {
                bytes.len() >= HEADER + 32
                    && sha256(&bytes[..bytes.len() - 32]) == bytes[bytes.len() - 32..]
            }
            _ => false,
        };
        if verifies {
            return Ok((*length, false));
        }
    }
    let mut fitting: Vec<usize> = Vec::new();
    for length in candidates.iter().copied() {
        let zero: bool = bytes.len() >= HEADER + length
            && bytes[bytes.len() - length..].iter().all(is_zero_byte);
        if !zero {
            continue;
        }
        if let Some((_entries, end)) = walk_entries(bytes, length)
            && extensions_fit(bytes, end, length)
        {
            fitting.push(length);
        }
    }
    return match fitting.as_slice() {
        [length] => Ok((*length, true)),
        [] if known.is_some() => Err(String::from(
            "the checksum does not match the repository's hash and is not all zero",
        )),
        [] => Err(String::from(
            "the checksum matches neither SHA-1 nor SHA-256, and it is not an all-zero checksum of a file that parses to its end",
        )),
        _ => Err(String::from(
            "the checksum is all zero and the file parses as both SHA-1 and SHA-256, and the repository's object format is not known",
        )),
    };
}
