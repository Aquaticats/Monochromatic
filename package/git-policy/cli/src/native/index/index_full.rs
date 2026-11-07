//! What:
//!  The whole content of an index file the in-process index update changes:
//! every entry with its stat data and flags,
//! and the
//!  extensions Git keeps (`TREE`,
//! `REUC`,
//! `UNTR`,
//! `FSMN`),
//! read from bytes (`read-cache.c`:
//! `do_read_index`).
//! Why:
//!  `repo_read_index.rs` keeps only path,
//! mode,
//! object name and stage,
//! which is enough to list an index but not to write
//!  one back.
//! The reader here returns `None` for a split index,
//! a sparse index,
//! a required extension it does not know,
//! a
//!  reserved flag bit or a size that does not add up,
//! so Git answers those.
//! `index_full_write.rs` writes it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const index: FullIndex | undefined = parseFullIndex(bytes, hashLength); // undefined: ask Git
//! ```

use crate::index_byte_search::first_index_of_byte;
use crate::index_cache_tree::CacheTree;
use crate::index_fsmonitor::{Fsmonitor, read_fsmonitor};
use crate::index_resolve_undo::ResolveUndo;
use crate::index_stat::{StatData, be32};
use crate::index_untracked::Untracked;
use crate::index_untracked_io::read_untracked;

/// The flag bit of an entry that Git was told to assume unchanged.
pub const ASSUME_VALID: u16 = 0x8000;

/// The two stage bits of an entry's flags.
pub const STAGE_MASK: u16 = 0x3000;

/// The flag bit that says an entry has extended flags.
pub const EXTENDED: u16 = 0x4000;

/// The extended flag of an entry only added to the index with `git add --intent-to-add`.
pub const INTENT_TO_ADD: u16 = 0x2000;

/// The extended flag of an entry outside the sparse checkout.
pub const SKIP_WORKTREE: u16 = 0x4000;

/// The bytes of the header and of the fixed fields before the object name.
const HEADER_LENGTH: usize = 12;

/// The bytes of the stat fields and mode in an entry,
/// before the object name.
const FIXED_LENGTH: usize = 40;

/// One entry of the index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FullEntry {
    /// The stat data recorded for the file.
    pub stat: StatData,
    /// The file mode with its type bits.
    pub mode: u32,
    /// The object name.
    pub id: Vec<u8>,
    /// The flag bits that are kept:
    /// assume-valid and the stage (the name length and the extended bit are worked out on write).
    pub flags: u16,
    /// The extended flags (`INTENT_TO_ADD`,
    /// `SKIP_WORKTREE`).
    pub extended: u16,
    /// The path.
    pub path: Vec<u8>,
    /// Whether fsmonitor vouches that the file is unchanged;
    /// only meaningful when the monitor is on.
    pub monitor_valid: bool,
}

impl FullEntry {
    /// The merge stage,
    /// 0 for a resolved path.
    pub fn stage(&self) -> u8 {
        return ((self.flags & STAGE_MASK) >> 12) as u8;
    }
}

/// The index file's content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FullIndex {
    /// The file format version as read (2,
    /// 3 or 4).
    pub version: u32,
    /// The entries by path and stage.
    pub entries: Vec<FullEntry>,
    /// The cache-tree,
    /// if the file has one.
    pub cache_tree: Option<CacheTree>,
    /// The resolve-undo record,
    /// if the file has one.
    pub resolve_undo: Option<ResolveUndo>,
    /// The untracked cache,
    /// if the file has one.
    pub untracked: Option<Untracked>,
    /// The fsmonitor extension,
    /// if the file has one.
    pub fsmonitor: Option<Fsmonitor>,
}

/// A big-endian `u16` at `at`.
fn be16(bytes: &[u8], at: usize) -> Option<u16> {
    let pair: &[u8] = bytes.get(at..at.checked_add(2)?)?;
    return Some(u16::from_be_bytes([pair[0], pair[1]]));
}

/// One entry at `at`:
/// the entry and the offset after it.
/// `previous` is the path before it (version 4 only).
fn read_entry(bytes: &[u8], at: usize, end: usize, version: u32, hash_length: usize, previous: &[u8]) -> Option<(FullEntry, usize)> {
    let stat: StatData = StatData {
        ctime_sec: be32(bytes, at)?,
        ctime_nsec: be32(bytes, at + 4)?,
        mtime_sec: be32(bytes, at + 8)?,
        mtime_nsec: be32(bytes, at + 12)?,
        dev: be32(bytes, at + 16)?,
        ino: be32(bytes, at + 20)?,
        uid: be32(bytes, at + 28)?,
        gid: be32(bytes, at + 32)?,
        size: be32(bytes, at + 36)?,
    };
    let mode: u32 = be32(bytes, at + 24)?;
    let id_at: usize = at + FIXED_LENGTH;
    let id: Vec<u8> = bytes.get(id_at..id_at + hash_length)?.to_vec();
    let flags_at: usize = id_at + hash_length;
    let raw_flags: u16 = be16(bytes, flags_at)?;
    let mut head: usize = FIXED_LENGTH + hash_length + 2;
    let mut extended: u16 = 0;
    if raw_flags & EXTENDED != 0 {
        if version < 3 {
            return None;
        }
        extended = be16(bytes, flags_at + 2)?;
        head += 2;
    }
    let name_start: usize = at + head;
    let (path, next): (Vec<u8>, usize) = if version == 4 {
        let (strip, after) = crate::index_stat::decode_varint(bytes.get(name_start..end)?)?;
        let keep: usize = previous.len().checked_sub(usize::try_from(strip).ok()?)?;
        let cursor: usize = end - after.len();
        let terminator: usize = first_index_of_byte(after, 0)?;
        let mut joined: Vec<u8> = previous[..keep].to_vec();
        joined.extend_from_slice(&after[..terminator]);
        (joined, cursor + terminator + 1)
    } else {
        let terminator: usize = first_index_of_byte(bytes.get(name_start..end)?, 0)?;
        (bytes[name_start..name_start + terminator].to_vec(), at.checked_add((head + terminator + 8) & !7)?)
    };
    // Only the intent-to-add and skip-worktree bits are defined, and Git canonicalizes other modes.
    let defined: bool = extended & !(INTENT_TO_ADD | SKIP_WORKTREE) == 0;
    let canonical: bool = matches!(mode, 0o100_644 | 0o100_755 | 0o120_000 | 0o160_000);
    if next > end || path.is_empty() || !defined || !canonical {
        return None;
    }
    let entry: FullEntry = FullEntry { stat, mode, id, flags: raw_flags & (ASSUME_VALID | STAGE_MASK), extended, path, monitor_valid: false };
    return Some((entry, next));
}

/// The index of `bytes`,
/// or `None` when it is outside the plain layouts listed above.
/// The checksum is not verified;
/// Git
/// does,
/// and writers install a finished file by rename.
pub fn parse_full_index(bytes: &[u8], hash_length: usize) -> Option<FullIndex> {
    let end: usize = bytes.len().checked_sub(hash_length)?;
    if bytes.get(..4)? != b"DIRC" {
        return None;
    }
    let version: u32 = be32(bytes, 4)?;
    if !(2..=4).contains(&version) {
        return None;
    }
    let count: usize = usize::try_from(be32(bytes, 8)?).ok()?;
    let mut entries: Vec<FullEntry> = Vec::with_capacity(count.min(1 << 20));
    let mut at: usize = HEADER_LENGTH;
    for _ in 0..count {
        let previous: &[u8] = match entries.last() {
            Some(entry) => entry.path.as_slice(),
            None => &[],
        };
        let (entry, next) = read_entry(bytes, at, end, version, hash_length, previous)?;
        entries.push(entry);
        at = next;
    }
    for pair in entries.windows(2) {
        if (pair[0].path.as_slice(), pair[0].stage()) >= (pair[1].path.as_slice(), pair[1].stage()) {
            return None;
        }
    }
    let mut index: FullIndex = FullIndex { version, entries, cache_tree: None, resolve_undo: None, untracked: None, fsmonitor: None };
    while at < end {
        let signature: &[u8] = bytes.get(at..at + 4)?;
        let size: usize = usize::try_from(be32(bytes, at + 4)?).ok()?;
        let data: &[u8] = bytes.get(at + 8..(at + 8).checked_add(size)?)?;
        if !signature.iter().all(u8::is_ascii_alphanumeric) || !signature[0].is_ascii_uppercase() {
            return None;
        }
        match signature {
            b"TREE" => index.cache_tree = Some(CacheTree::read(data, hash_length)?),
            b"REUC" => index.resolve_undo = Some(ResolveUndo::read(data, hash_length)?),
            b"UNTR" => index.untracked = Some(read_untracked(data, hash_length)?),
            b"FSMN" => index.fsmonitor = Some(read_fsmonitor(data, index.entries.len())?),
            // Git writes neither of these unless asked to, and an optional extension it does not know is not kept.
            b"EOIE" | b"IEOT" => {}
            _ => {}
        }
        at += 8 + size;
    }
    if at != end {
        return None;
    }
    return Some(index);
}

#[cfg(test)]
#[path = "index_full_tests.rs"]
mod tests;
