//! What:
//!  A read of the per-entry flags of an index file that decide whether the index may seed an explicit-path commit:
//!  whether any entry is marked assume-unchanged,
//! skip-worktree or intent-to-add.
//! Why:
//!  A private index copied from the real index and reset to the base keeps those marks,
//! and `git add --all -- <path>`
//!  then skips a marked file the commit named (assume-unchanged) or refuses it (skip-worktree),
//! where Git's own partial
//!  commit and the previous flow staged it;
//! an intent-to-add entry of a tracked empty file survives the reset and is then
//!  left out of the tree,
//! so the commit loses that file.
//! Finding a mark needs the whole file,
//! and a Git start for it would add a
//!  process to every commit,
//! so the entries are read here;
//! anything this reader does not fully understand answers
//!  "unreadable",
//! and the caller then builds the index the slower,
//! always-correct way.
//!  The layout is the one in Git's `gitformat-index` documentation:
//! a 12-byte header,
//! entries (versions 2 to 4),
//! then
//!  extensions and a trailing checksum.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const flags = readIndexFlags(await readFile(indexPath)); // 'plain' | 'marked' | 'unreadable'
//! ```

use crate::index_byte_search::first_index_of_byte;

/// Bit of the 16-bit entry flags that marks assume-unchanged.
const ASSUME_VALID: u16 = 0x8000;

/// Bit of the 16-bit entry flags that says a second 16-bit flags field follows.
const EXTENDED: u16 = 0x4000;

/// Bit of the extended flags that marks skip-worktree.
const SKIP_WORKTREE: u16 = 0x4000;

/// Bit of the extended flags that marks intent-to-add.
/// Such an entry keeps the empty blob's name and is left out of any tree built from the index,
/// so a copy that a one-tree
///  `read-tree --reset` leaves in place drops a tracked empty file from the commit.
const INTENT_TO_ADD: u16 = 0x2000;

/// Bytes of the index header:
/// signature,
/// version and entry count.
const HEADER_LENGTH: usize = 12;

/// Bytes of an entry before its object name:
/// ten 32-bit stat fields.
const STAT_LENGTH: usize = 40;

/// Object name lengths tried in turn:
/// SHA-1 and SHA-256.
const OBJECT_NAME_LENGTHS: [usize; 2] = [20, 32];

/// The extension that makes the entries live partly in another file,
/// which this reader does not follow.
const SPLIT_INDEX_EXTENSION: &[u8; 4] = b"link";

/// What a read of an index file found.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexFlags {
    /// Every entry was read and none is marked assume-unchanged,
    /// skip-worktree or intent-to-add.
    Plain,
    /// At least one entry is marked assume-unchanged,
    /// skip-worktree or intent-to-add.
    Marked,
    /// The file is not an index this reader fully understands (an unknown version,
    /// a split index,
    /// a layout that does
    /// not add up),
    /// so nothing is known about its marks.
    Unreadable,
}

/// Big-endian 16-bit value at `at`.
fn be16(bytes: &[u8], at: usize) -> Option<u16> {
    let pair: &[u8] = bytes.get(at..at.checked_add(2)?)?;
    return Some(u16::from_be_bytes([pair[0], pair[1]]));
}

/// Big-endian 32-bit value at `at`.
fn be32(bytes: &[u8], at: usize) -> Option<u32> {
    let quad: &[u8] = bytes.get(at..at.checked_add(4)?)?;
    return Some(u32::from_be_bytes([quad[0], quad[1], quad[2], quad[3]]));
}

/// What:
///  Reads every entry and extension of an index whose object names are `name_length` bytes long.
/// Why:
///   A wrong name length misplaces the entries,
/// so the layout stops adding up and the read answers nothing;
///  a right one consumes the file exactly up to its checksum.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scanEntries(bytes, nameLength) // true when any entry is marked; undefined when the layout is not understood
/// ```
fn scan_entries(bytes: &[u8], name_length: usize) -> Option<bool> {
    let end: usize = bytes.len().checked_sub(name_length)?;
    if bytes.get(..4)? != b"DIRC" {
        return None;
    }
    let version: u32 = be32(bytes, 4)?;
    if !(2..=4).contains(&version) {
        return None;
    }
    let count: u32 = be32(bytes, 8)?;
    let mut marked: bool = false;
    let mut at: usize = HEADER_LENGTH;
    for _entry in 0..count {
        let flags: u16 = be16(bytes, at.checked_add(STAT_LENGTH)?.checked_add(name_length)?)?;
        let mut head: usize = STAT_LENGTH + name_length + 2;
        if flags & ASSUME_VALID != 0 {
            marked = true;
        }
        if flags & EXTENDED != 0 {
            if version < 3 {
                return None;
            }
            if be16(bytes, at.checked_add(head)?)? & (SKIP_WORKTREE | INTENT_TO_ADD) != 0 {
                marked = true;
            }
            head += 2;
        }
        let name_start: usize = at.checked_add(head)?;
        if version == 4 {
            // The strip count is a varint; its last byte has no continuation bit.
            let mut cursor: usize = name_start;
            while bytes.get(cursor)? & 0x80 != 0 {
                cursor += 1;
            }
            cursor += 1;
            let terminator: usize = first_index_of_byte(bytes.get(cursor..end)?, 0)?;
            at = cursor + terminator + 1;
        } else {
            let terminator: usize = first_index_of_byte(bytes.get(name_start..end)?, 0)?;
            // An entry is padded with one to eight NULs to a multiple of eight bytes from its own start.
            at = at.checked_add((head + terminator + 8) & !7)?;
        }
        if at > end {
            return None;
        }
    }
    while at < end {
        let signature: &[u8] = bytes.get(at..at.checked_add(4)?)?;
        if !signature.iter().all(u8::is_ascii_alphanumeric) || signature == SPLIT_INDEX_EXTENSION {
            return None;
        }
        let size: usize = usize::try_from(be32(bytes, at + 4)?).ok()?;
        at = at.checked_add(8)?.checked_add(size)?;
    }
    if at != end {
        return None;
    }
    return Some(marked);
}

/// What:
///  Whether an index file has an entry marked assume-unchanged or skip-worktree.
/// Why:
///   Both object name lengths are tried because the reader is not told the repository's object format;
///  a layout that adds up for neither is unreadable,
/// never "plain".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readIndexFlags(bytes)
/// ```
pub fn read_index_flags(bytes: &[u8]) -> IndexFlags {
    for name_length in OBJECT_NAME_LENGTHS {
        match scan_entries(bytes, name_length) {
            Some(true) => return IndexFlags::Marked,
            Some(false) => return IndexFlags::Plain,
            None => {}
        }
    }
    return IndexFlags::Unreadable;
}

/// Reader controls stay out of the release executable.
#[cfg(test)]
#[path = "index_flags_tests.rs"]
mod tests;
