//! What:
//!  The stat rewrite of a Git index: the bytes of an index file in, the bytes of the same index out, with the stat
//!  fields of each entry replaced by what the file on disk reports now.
//! Why:
//!  A reflink copy gives every file a new inode number and change time, so the first `git status` in the copied tree
//!  finds every entry stat-dirty and hashes every file (22.1 s measured against 0.17 s with the rewrite, and the clean
//!  filter of Git LFS runs on LFS files). The rewrite touches an entry only when the file's size and modification
//!  time equal the entry's, so a file that really changed stays dirty. Entry lengths never change, so the entries are
//!  edited in place. The accelerator extensions that record stat-derived state (`UNTR` untracked cache, `FSMN`
//!  fsmonitor token) or offsets into the entries (`EOIE`, `IEOT`) are dropped, because Git rebuilds them; a split
//!  index (`link`) or a sparse index (`sdir`) is left untouched. The checksum is recomputed with the hash that
//!  verifies the old one (SHA-1, else SHA-256); an index written with `index.skipHash` has an all-zero checksum and
//!  keeps one. Versions 2, 3 and 4 are read (`Documentation/gitformat-index.adoc`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = rewriteIndexStat(bytes, (path) => lstatFields(join(tree, path))); // { bytes, updated, total } | { skipped }
//! ```

use crate::subvolume_sha::{sha1, sha256};

/// The bytes of the header: signature, version and entry count.
const HEADER: usize = 12;

/// The bytes of the fixed stat fields of an entry, before the object name.
const STAT: usize = 40;

/// The extension signatures that are dropped: derived state that Git rebuilds.
const DROPPED: [&[u8; 4]; 4] = [b"UNTR", b"FSMN", b"EOIE", b"IEOT"];

/// The extension signatures that make the rewrite decline: they change what the entries mean.
const DECLINED: [&[u8; 4]; 2] = [b"link", b"sdir"];

/// What:
///  What a file on disk reports now, in the units an index entry stores.
///  `u32` is an unsigned 32-bit integer (siblings `u64` and `i64`); an index stores each stat field as 32 bits.
/// Why:
///   The caller fills it from `lstat`; the rewrite compares size and modification time and copies the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StatFields = { ctimeSeconds: number; ctimeNanos: number; mtimeSeconds: number; mtimeNanos: number; dev: number; ino: number; uid: number; gid: number; size: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatFields {
    /// Change time, seconds.
    pub ctime_seconds: u32,
    /// Change time, nanoseconds.
    pub ctime_nanos: u32,
    /// Modification time, seconds.
    pub mtime_seconds: u32,
    /// Modification time, nanoseconds.
    pub mtime_nanos: u32,
    /// Device, low 32 bits.
    pub dev: u32,
    /// Inode, low 32 bits.
    pub ino: u32,
    /// Owner.
    pub uid: u32,
    /// Group.
    pub gid: u32,
    /// Size, low 32 bits.
    pub size: u32,
}

/// What:
///  The result of one rewrite.
///  `Vec<u8>` is the owned bytes of the new file; `String` is the owned reason a rewrite was declined.
/// Why:
///   A declined index is left as copied, and the reason goes into the report, so the cost (a slow first status) is
///  explained.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexOutcome = { bytes: Uint8Array; updated: number; total: number } | { skipped: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexOutcome {
    /// The new file; `updated` of `total` entries got fresh stat fields.
    Rewritten {
        /// The whole new index file, checksum included.
        bytes: Vec<u8>,
        /// Entries whose stat fields were replaced.
        updated: usize,
        /// Entries in the index.
        total: usize,
    },
    /// The index was not changed, for this reason.
    Skipped(String),
}

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

/// A big-endian `u32` at `at`, or nothing past the end.
pub(crate) fn be32(bytes: &[u8], at: usize) -> Option<u32> {
    let quad: &[u8] = bytes.get(at..at.checked_add(4)?)?;
    return Some(u32::from_be_bytes([quad[0], quad[1], quad[2], quad[3]]));
}

/// A big-endian `u16` at `at`, or nothing past the end.
fn be16(bytes: &[u8], at: usize) -> Option<u16> {
    let pair: &[u8] = bytes.get(at..at.checked_add(2)?)?;
    return Some(u16::from_be_bytes([pair[0], pair[1]]));
}

/// What:
///  Walk the entries of an index whose checksum is `hash_length` bytes; return them and the offset where they end.
/// Why:
///   Entry layout depends on the hash length (the object name) and the version (v4 compresses the path against the
///  previous one, v2 and v3 pad to 8 bytes).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walkEntries(bytes: Uint8Array, hashLength: number): { entries: Located[]; end: number } | undefined;
/// ```
pub(crate) fn walk_entries(bytes: &[u8], hash_length: usize) -> Option<(Vec<Located>, usize)> {
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
            let end: usize = cursor + bytes.get(cursor..limit)?.iter().position(|stored: &u8| { return *stored == 0; })?;
            let mut joined: Vec<u8> = previous[..keep].to_vec();
            joined.extend_from_slice(&bytes[cursor..end]);
            path = joined;
            next = end + 1;
        } else {
            let length: usize = bytes.get(name_at..limit)?.iter().position(|stored: &u8| { return *stored == 0; })?;
            path = bytes[name_at..name_at + length].to_vec();
            next = at.checked_add((name_at - at + length + 8) & !7)?;
        }
        if next > limit {
            return None;
        }
        previous = path.clone();
        found.push(Located { start: at, path, flags, mode });
        at = next;
    }
    return Some((found, at));
}

/// What:
///  Whether the extensions that follow the entries of an index whose checksum is `hash_length` bytes end exactly where
///  the checksum begins.
/// Why:
///   An index with an all-zero checksum proves nothing about its hash length, so a length is accepted only when the
///  whole file parses under it; a wrong length almost always leaves a cut-off extension.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function extensionsFit(bytes: Uint8Array, entriesEnd: number, hashLength: number): boolean;
/// ```
fn extensions_fit(bytes: &[u8], entries_end: usize, hash_length: usize) -> bool {
    let Some(limit) = bytes.len().checked_sub(hash_length) else { return false };
    let mut at: usize = entries_end;
    while at < limit {
        let Some(size) = be32(bytes, at + 4) else { return false };
        let Some(end) = at.checked_add(8).and_then(|header_end: usize| { return header_end.checked_add(usize::try_from(size).ok()?); }) else { return false };
        if end > limit {
            return false;
        }
        at = end;
    }
    return at == limit;
}

/// What:
///  Which hash verifies the checksum, as a hash length: 20 for SHA-1, 32 for SHA-256, and for an all-zero checksum
///  (`index.skipHash`) the one length under which the whole file parses.
///  `known` is the repository's hash length when the caller read it from the repository's configuration.
/// Why:
///   A checksum that verifies proves its length. An all-zero checksum proves nothing, so the repository's object
///  format decides when it is known; when it is not, a file that parses under both lengths is ambiguous and is
///  declined, because a wrong guess drops or keeps the wrong bytes of every entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checksumLength(bytes: Uint8Array, known?: number): { length: number; zero: boolean } | { declined: string };
/// ```
fn checksum_kind(bytes: &[u8], known: Option<usize>) -> Result<(usize, bool), String> {
    let candidates: Vec<usize> = match known {
        Some(length) => vec![length],
        None => vec![20, 32],
    };
    for length in &candidates {
        let verifies: bool = match length {
            20 => bytes.len() >= HEADER + 20 && sha1(&bytes[..bytes.len() - 20]) == bytes[bytes.len() - 20..],
            32 => bytes.len() >= HEADER + 32 && sha256(&bytes[..bytes.len() - 32]) == bytes[bytes.len() - 32..],
            _ => false,
        };
        if verifies {
            return Ok((*length, false));
        }
    }
    let fitting: Vec<usize> = candidates
        .iter()
        .copied()
        .filter(|length: &usize| {
            let zero: bool = bytes.len() >= HEADER + length && bytes[bytes.len() - length..].iter().all(|byte: &u8| { return *byte == 0; });
            return zero && walk_entries(bytes, *length).is_some_and(|(_entries, end): (Vec<Located>, usize)| { return extensions_fit(bytes, end, *length); });
        })
        .collect();
    return match fitting.as_slice() {
        [length] => Ok((*length, true)),
        [] if known.is_some() => Err(String::from("the checksum does not match the repository's hash and is not all zero")),
        [] => Err(String::from("the checksum matches neither SHA-1 nor SHA-256, and it is not an all-zero checksum of a file that parses to its end")),
        _ => Err(String::from("the checksum is all zero and the file parses as both SHA-1 and SHA-256, and the repository's object format is not known")),
    };
}

/// What:
///  Rewrite the stat fields of an index file; see the module description for the rules.
///  `lookup` is called with an entry's path and returns what `lstat` reports now, or nothing for a missing file.
///  `&mut dyn FnMut(&[u8]) -> Option<StatFields>` is a borrowed callback that may keep state (sibling `Fn` cannot).
/// Why:
///   Keeping the file system out of this function makes every branch testable with built bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rewriteIndexStat(bytes: Uint8Array, lookup: (path: Uint8Array) => StatFields | undefined): IndexOutcome;
/// ```
pub fn rewrite_index_stat(bytes: &[u8], lookup: &mut dyn FnMut(&[u8]) -> Option<StatFields>) -> IndexOutcome {
    return rewrite_index_stat_with(bytes, &RewriteContext::default(), lookup, &mut |_path: &[u8], _stored: &StoredStat| { return true; });
}

/// What:
///  What the caller knows about the index beyond its bytes.
///  `Option<usize>` is a value that may be absent.
/// Why:
///   The repository's hash length decides how an all-zero checksum is read, and the index file's own modification time
///  decides which entries are racy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RewriteContext = { hashLength?: number; indexMtime?: [number, number] };
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RewriteContext {
    /// The repository's hash length in bytes (20 or 32) when it was read from its configuration.
    pub hash_length: Option<usize>,
    /// The modification time of the index file the bytes came from: seconds and nanoseconds.
    pub index_mtime: Option<(u32, u32)>,
}

/// What:
///  The stat fields an entry holds, as stored.
/// Why:
///   The caller compares them with the file the index was written for, so an entry is refreshed only when that file is
///  the one Git recorded and not one that changed since.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StoredStat = { ctimeSeconds: number; ctimeNanos: number; mtimeSeconds: number; mtimeNanos: number; ino: number; uid: number; gid: number; size: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoredStat {
    /// Change time, seconds.
    pub ctime_seconds: u32,
    /// Change time, nanoseconds (0 when Git was built without nanosecond support).
    pub ctime_nanos: u32,
    /// Modification time, seconds.
    pub mtime_seconds: u32,
    /// Modification time, nanoseconds (0 when Git was built without nanosecond support).
    pub mtime_nanos: u32,
    /// Inode, low 32 bits.
    pub ino: u32,
    /// Owner.
    pub uid: u32,
    /// Group.
    pub gid: u32,
    /// Size, low 32 bits.
    pub size: u32,
}

/// What:
///  Rewrite the stat fields of an index file; see the module description for the rules.
///  `trusted` is asked, with an entry's path and stored fields, whether the file the entry was recorded for is still
///  the one the copy was made from.
///  An entry whose modification time is not earlier than the index file's own (racy in Git's terms) keeps its fields,
///  so Git hashes it.
/// Why:
///   Replacing the change time and inode of an entry that Git could not trust, or of a file that changed after Git
///  looked, would make Git believe the stored blob is current.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rewriteIndexStatWith(bytes: Uint8Array, context: RewriteContext, lookup: Lookup, trusted: (path: Uint8Array, stored: StoredStat) => boolean): IndexOutcome;
/// ```
pub fn rewrite_index_stat_with(
    bytes: &[u8],
    context: &RewriteContext,
    lookup: &mut dyn FnMut(&[u8]) -> Option<StatFields>,
    trusted: &mut dyn FnMut(&[u8], &StoredStat) -> bool,
) -> IndexOutcome {
    if bytes.len() < HEADER || &bytes[..4] != b"DIRC" {
        return IndexOutcome::Skipped(String::from("the file is not an index (no DIRC signature)"));
    }
    let version: u32 = be32(bytes, 4).unwrap_or(0);
    if !(2..=4).contains(&version) {
        return IndexOutcome::Skipped(format!("index version {version} is not 2, 3 or 4"));
    }
    let (hash_length, zero_checksum): (usize, bool) = match checksum_kind(bytes, context.hash_length) {
        Ok(found) => found,
        Err(reason) => return IndexOutcome::Skipped(reason),
    };
    let Some((entries, entries_end)) = walk_entries(bytes, hash_length) else {
        return IndexOutcome::Skipped(String::from("the entries do not have the documented layout"));
    };
    let limit: usize = bytes.len() - hash_length;
    // The extensions after the entries: keep each unless it is derived state; decline on one that changes the meaning.
    let mut kept: Vec<u8> = Vec::new();
    let mut at: usize = entries_end;
    while at < limit {
        let (Some(size), Some(signature)) = (be32(bytes, at + 4), bytes.get(at..at + 4)) else {
            return IndexOutcome::Skipped(String::from("an extension header is cut off"));
        };
        let end: usize = at + 8 + usize::try_from(size).unwrap_or(usize::MAX);
        if end > limit {
            return IndexOutcome::Skipped(String::from("an extension runs past the end of the file"));
        }
        if DECLINED.iter().any(|name: &&[u8; 4]| { return signature == name.as_slice(); }) {
            return IndexOutcome::Skipped(format!("the index uses the {} extension (split or sparse index)", String::from_utf8_lossy(signature)));
        }
        if !DROPPED.iter().any(|name: &&[u8; 4]| { return signature == name.as_slice(); }) {
            kept.extend_from_slice(&bytes[at..end]);
        }
        at = end;
    }
    let mut output: Vec<u8> = bytes[..entries_end].to_vec();
    let mut updated: usize = 0;
    for entry in &entries {
        let regular: bool = matches!(entry.mode & 0o170_000, 0o100_000 | 0o120_000);
        let resolved: bool = (entry.flags >> 12) & 3 == 0;
        let stored_size: u32 = be32(bytes, entry.start + 36).unwrap_or(0);
        if !regular || !resolved || stored_size == 0 {
            continue;
        }
        let stored_seconds: u32 = be32(bytes, entry.start + 8).unwrap_or(0);
        let stored_nanos: u32 = be32(bytes, entry.start + 12).unwrap_or(0);
        // Git's racy test: an entry modified in the same second as the index file, or later, cannot be trusted by its
        // stat fields alone. Its fields stay as recorded, which no longer match the copy, so Git hashes the file.
        if context.index_mtime.is_some_and(|(index_seconds, _nanos): (u32, u32)| { return stored_seconds >= index_seconds; }) {
            continue;
        }
        let Some(now) = lookup(entry.path.as_slice()) else { continue };
        // An index written by a Git built without nanosecond support stores 0 there.
        let same_time: bool = now.mtime_seconds == stored_seconds && (stored_nanos == 0 || stored_nanos == now.mtime_nanos);
        if now.size != stored_size || !same_time {
            continue;
        }
        let recorded: StoredStat = StoredStat {
            ctime_seconds: be32(bytes, entry.start).unwrap_or(0),
            ctime_nanos: be32(bytes, entry.start + 4).unwrap_or(0),
            mtime_seconds: stored_seconds,
            mtime_nanos: stored_nanos,
            ino: be32(bytes, entry.start + 20).unwrap_or(0),
            uid: be32(bytes, entry.start + 28).unwrap_or(0),
            gid: be32(bytes, entry.start + 32).unwrap_or(0),
            size: stored_size,
        };
        if !trusted(entry.path.as_slice(), &recorded) {
            continue;
        }
        let fields: [(usize, u32); 6] = [(0, now.ctime_seconds), (4, now.ctime_nanos), (16, now.dev), (20, now.ino), (28, now.uid), (32, now.gid)];
        for (offset, value) in fields {
            output[entry.start + offset..entry.start + offset + 4].copy_from_slice(&value.to_be_bytes());
        }
        updated += 1;
    }
    output.extend_from_slice(kept.as_slice());
    if zero_checksum {
        output.extend(std::iter::repeat_n(0u8, hash_length));
    } else if hash_length == 20 {
        let digest: [u8; 20] = sha1(output.as_slice());
        output.extend_from_slice(&digest);
    } else {
        let digest: [u8; 32] = sha256(output.as_slice());
        output.extend_from_slice(&digest);
    }
    return IndexOutcome::Rewritten { bytes: output, updated, total: entries.len() };
}

/// Index rewrite controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_tests.rs"]
mod tests;
