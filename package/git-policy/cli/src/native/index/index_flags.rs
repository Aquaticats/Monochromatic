//! What:
//!  A read of the per-entry flags of an index file that decide whether the index may seed an explicit-path commit:
//!  whether any entry is marked assume-unchanged or skip-worktree,
//!  which those paths are,
//!  and which entries are marked intent-to-add.
//! Why:
//!  A private index copied from the real index and reset to the base keeps those marks,
//! and `git add --all -- <path>`
//!  then skips a marked file the commit named (assume-unchanged) or refuses it (skip-worktree),
//! where Git's own partial
//!  commit and the previous flow staged it.
//!  An intent-to-add entry stops nothing:
//!  it holds the empty blob's name and no stat data at all,
//!  so a copy that leaves it out loses nothing a commit needs,
//!  and a tree Git builds from an index that holds one leaves that path out,
//!  so the entry has to be left out rather than copied.
//!  Reporting the paths lets the caller leave exactly those entries out.
//!  Finding a mark needs the whole file,
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
//! const flags = readIndexFlags(await readFile(indexPath)); // 'plain' | { intentToAdd: paths } | 'marked' | 'unreadable'
//! ```

/// The byte search the extension walk uses.
use crate::index_byte_search::first_index_of_byte;
/// Git's varint,
/// which a version 4 entry writes its strip count in.
use crate::index_stat::decode_varint;

/// Bit of the 16-bit entry flags that marks assume-unchanged.
const ASSUME_VALID: u16 = 0x8000;

/// Bit of the 16-bit entry flags that says a second 16-bit flags field follows.
const EXTENDED: u16 = 0x4000;

/// Bit of the extended flags that marks skip-worktree.
const SKIP_WORKTREE: u16 = 0x4000;

/// Bit of the extended flags that marks intent-to-add.
/// Such an entry keeps the empty blob's name and is left out of any tree built from the index,
/// so a copy that carries one
///  drops a tracked file from the commit it seeds.
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexFlags {
    /// Every entry was read and none is marked assume-unchanged,
    /// skip-worktree or intent-to-add.
    Plain,
    /// Every entry was read,
    /// and the only entries marked are these intent-to-add ones,
    /// whose paths are given in index order.
    IntentToAdd(Vec<Vec<u8>>),
    /// At least one entry is marked assume-unchanged or skip-worktree,
    /// which a copy may not carry,
    /// with the paths those marks hide from Git's own staging.
    Marked(MarkedPaths),
    /// The file is not an index this reader fully understands (an unknown version,
    /// a split index,
    /// a layout that does
    /// not add up),
    /// so nothing is known about its marks.
    Unreadable,
}

/// What:
///  The paths an index marks so that Git's own staging of `-a` or of a named path leaves them alone.
/// Why:
///  `git commit -a` and `git commit -- <path>` skip a skip-worktree entry and,
/// for `-a`,
///  an assume-unchanged one,
///  measured against Git 2.56.0;
///  the wrapper's private index is seeded from the base and holds no mark,
///  so its staging took those files
///  and the commit recorded bytes Git would not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MarkedPaths = { skipWorktree: Uint8Array[]; assumeUnchanged: Uint8Array[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkedPaths {
    /// The paths whose entry is marked skip-worktree,
    /// in the order the index holds them.
    pub skip_worktree: Vec<Vec<u8>>,
    /// The paths whose entry is marked assume-unchanged,
    /// in the order the index holds them.
    pub assume_unchanged: Vec<Vec<u8>>,
}

/// What a scan of every entry of one index file found.
struct Marks {
    /// The paths marked skip-worktree.
    skip_worktree: Vec<Vec<u8>>,
    /// The paths marked assume-unchanged.
    assume_unchanged: Vec<Vec<u8>>,
    /// The paths whose entry is marked intent-to-add,
    /// in the order the index holds them.
    intent_to_add: Vec<Vec<u8>>,
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
///  A version 4 entry's name,
///  which stores only the part that differs from the entry before it,
///  and the bytes the entry after it starts at.
/// Why:
///  Only this layout hides a name behind a strip count,
/// and the path of an intent-to-add entry is the one thing the caller needs from the entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// compressedName(bytesAfterFlags, earlier) // [name, bytesUsed] | undefined
/// ```
fn compressed_name(bytes_after_flags: &[u8], earlier: &[u8]) -> Option<(Vec<u8>, usize)> {
    let (strip_count, rest) = decode_varint(bytes_after_flags)?;
    let strip: usize = usize::try_from(strip_count).ok()?;
    let terminator: usize = first_index_of_byte(rest, 0)?;
    let kept: usize = earlier.len().checked_sub(strip)?;
    let mut name: Vec<u8> = earlier.get(..kept)?.to_vec();
    name.extend_from_slice(rest.get(..terminator)?);
    let used: usize = bytes_after_flags.len().checked_sub(rest.len())?.checked_add(terminator)?.checked_add(1)?;
    return Some((name, used));
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
/// scanEntries(bytes, nameLength) // marks, or undefined when the layout is not understood
/// ```
fn scan_entries(bytes: &[u8], name_length: usize) -> Option<Marks> {
    let end: usize = bytes.len().checked_sub(name_length)?;
    if bytes.get(..4)? != b"DIRC" {
        return None;
    }
    let version: u32 = be32(bytes, 4)?;
    if !(2..=4).contains(&version) {
        return None;
    }
    let count: u32 = be32(bytes, 8)?;
    let mut marks: Marks = Marks { skip_worktree: Vec::new(), assume_unchanged: Vec::new(), intent_to_add: Vec::new() };
    let mut at: usize = HEADER_LENGTH;
    // The name of the entry before, which a version 4 entry is written against.
    let mut earlier: Vec<u8> = Vec::new();
    for _entry in 0..count {
        let flags: u16 = be16(bytes, at.checked_add(STAT_LENGTH)?.checked_add(name_length)?)?;
        // The bytes of the entry before its name: the stat fields, the object name, the 16-bit flags and, when it
        // carries them, the extended flags. The padding below is counted from the entry's own start, so every one of
        // those bytes is part of it.
        let mut head: usize = STAT_LENGTH + name_length + 2;
        let assume_unchanged: bool = flags & ASSUME_VALID != 0;
        let mut extended: u16 = 0;
        if flags & EXTENDED != 0 {
            if version < 3 {
                return None;
            }
            extended = be16(bytes, at.checked_add(head)?)?;
            head += 2;
        }
        let skip_worktree: bool = extended & SKIP_WORKTREE != 0;
        let wants_name: bool = extended & INTENT_TO_ADD != 0;
        let name_start: usize = at.checked_add(head)?;
        if version == 4 {
            let (name, used): (Vec<u8>, usize) = compressed_name(bytes.get(name_start..end)?, earlier.as_slice())?;
            at = name_start.checked_add(used)?;
            if wants_name {
                marks.intent_to_add.push(name.clone());
            }
            if skip_worktree {
                marks.skip_worktree.push(name.clone());
            }
            if assume_unchanged {
                marks.assume_unchanged.push(name.clone());
            }
            earlier = name;
        } else {
            let terminator: usize = first_index_of_byte(bytes.get(name_start..end)?, 0)?;
            let name: Vec<u8> = bytes.get(name_start..name_start.checked_add(terminator)?)?.to_vec();
            if wants_name {
                marks.intent_to_add.push(name.clone());
            }
            if skip_worktree {
                marks.skip_worktree.push(name.clone());
            }
            if assume_unchanged {
                marks.assume_unchanged.push(name);
            }
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
    return Some(marks);
}

/// What:
///  The marks an index file carries:
/// whether an entry is assume-unchanged or skip-worktree,
/// and which entries are intent-to-add.
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
            Some(marks) if !marks.skip_worktree.is_empty() || !marks.assume_unchanged.is_empty() => {
                return IndexFlags::Marked(MarkedPaths { skip_worktree: marks.skip_worktree, assume_unchanged: marks.assume_unchanged });
            }
            Some(marks) if !marks.intent_to_add.is_empty() => return IndexFlags::IntentToAdd(marks.intent_to_add),
            Some(_) => return IndexFlags::Plain,
            None => {}
        }
    }
    return IndexFlags::Unreadable;
}

/// Reader controls stay out of the release executable.
#[cfg(test)]
#[path = "index_flags_tests.rs"]
mod tests;
