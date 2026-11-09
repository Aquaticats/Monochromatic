//! What:
//!  The entries of an index file read in process,
//! for versions 2,
//! 3 and 4.
//! Why:
//!  `ls-files --stage`,
//! `ls-files --unmerged` and the cached diffs only need each entry's path,
//! mode,
//! object name and
//!  stage.
//!  The reader returns `None` for a split index (`link`),
//! a sparse index (`sdir` or a directory entry),
//! an extension whose
//!  text Git words on standard error or dies on,
//! unsorted entries or a size that does not add up,
//! so Git answers those.
//!  It does not verify the trailing checksum;
//! Git does,
//! and a torn index cannot occur because writers rename a finished file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries: IndexEntry[] | undefined = parseIndex(bytes, objectFormat); // undefined: ask Git
//! ```

/// The `TREE` (cache-tree) extension,
/// which names the tree each valid directory of the index hashes to.
use crate::index_byte_search::first_index_of_byte;
use crate::index_cache_tree::CacheTree;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::name_length;

/// The bytes of the index header:
/// signature,
/// version,
/// entry count.
const HEADER_LENGTH: usize = 12;

/// The bytes of the fixed stat fields before the object name in an entry.
const STAT_LENGTH: usize = 40;

/// The extended-flags bit that marks an intent-to-add entry.
pub const INTENT_TO_ADD: u16 = 0x2000;

/// One index entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexEntry {
    /// The path,
    /// `/` separated,
    /// relative to the top of the worktree.
    pub path: Vec<u8>,
    /// The file mode with its type bits.
    pub mode: u32,
    /// The object name.
    pub id: Vec<u8>,
    /// The merge stage,
    /// 0 for a resolved path.
    pub stage: u8,
    /// The extended flags of a version 3 or 4 entry,
    /// 0 otherwise.
    pub extended: u16,
}

/// A big-endian `u16` at `at`.
fn be16(bytes: &[u8], at: usize) -> Option<u16> {
    let pair: &[u8] = bytes.get(at..at.checked_add(2)?)?;
    return Some(u16::from_be_bytes([pair[0], pair[1]]));
}

/// A big-endian `u32` at `at`.
fn be32(bytes: &[u8], at: usize) -> Option<u32> {
    let quad: &[u8] = bytes.get(at..at.checked_add(4)?)?;
    return Some(u32::from_be_bytes([quad[0], quad[1], quad[2], quad[3]]));
}

/// What:
///  Parse the whole file;
/// `None` when anything is outside the plain layouts listed above.
/// Why:
///   Every answer built on it is checked against Git by the differential controls,
/// including the standard error of the extension line Git prints for a signature it does not know.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseIndex(bytes: Uint8Array, format: ObjectFormat): IndexEntry[] | undefined;
/// ```
pub fn parse_index(bytes: &[u8], format: ObjectFormat) -> Option<Vec<IndexEntry>> {
    let parsed = parse_index_with_tree(bytes, format)?;
    return Some(parsed.entries);
}

/// The entries of an index file and its cache-tree extension.
#[derive(Clone, Debug)]
pub struct ParsedIndex {
    /// The entries,
    /// sorted by path and stage.
    pub entries: Vec<IndexEntry>,
    /// The `TREE` extension,
    /// or `None` when the file has none or it does not read as Git writes it.
    pub tree: Option<CacheTree>,
}

/// What:
///  Whether an `IEOT` body names exactly the entries this reader parsed.
/// Why:
///   Git reads the entries through this table when `index.threads` turns threaded loading on,
///  and `read_ieot_extension` (`read-cache.c`, Git 2.56.0) words a version other than 1
///  (`error: invalid IEOT version <n>`) and a table naming no block
///  (`error: invalid number of IEOT entries <n>`);
///  it then seeks each block by the table's own byte offsets,
///  so a table naming other bytes is a different listing, or no listing at all:
///  measured on Git 2.56.0, a first block one byte off ends the process without a word,
///  and a first block pointing into the header prints `fatal: unknown index entry format 0xc0110000`.
///   So the version must be 1, the table must name at least one block, and every block must begin where
///  this reader parsed its first entry and cover every entry between them.
///   A file with no `IEOT` extension is unaffected,
///  and a table read this way answers the entries this reader already has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function offsetTableIsSilent(body: Uint8Array, starts: number[]): boolean;
/// ```
fn offset_table_is_silent(body: &[u8], starts: &[u32]) -> Option<()> {
    if be32(body, 0)? != 1 {
        return None;
    }
    let size: usize = body.len().checked_sub(4)?;
    if size == 0 || size % 8 != 0 {
        return None;
    }
    let mut next: usize = 0;
    for block in 0..size / 8 {
        let begins: usize = usize::try_from(be32(body, 4 + block * 8)?).ok()?;
        let held: usize = usize::try_from(be32(body, 8 + block * 8)?).ok()?;
        if held == 0 || usize::try_from(*starts.get(next)?).ok()? != begins {
            return None;
        }
        next = next.checked_add(held)?;
    }
    if next != starts.len() {
        return None;
    }
    return Some(());
}

/// What:
///  `parse_index`,
/// keeping the cache-tree extension.
/// Why:
///   A directory the cache-tree marks valid has a tree name that the index entries below it hash to,
/// so a comparison of the index
///  with a tree can leave that directory unread when the tree has the same name there (`repo_read_tree_cached.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseIndexWithTree(bytes: Uint8Array, format: ObjectFormat): { entries: IndexEntry[]; tree?: CacheTree } | undefined;
/// ```
pub fn parse_index_with_tree(bytes: &[u8], format: ObjectFormat) -> Option<ParsedIndex> {
    let hash_length: usize = name_length(format);
    let end: usize = bytes.len().checked_sub(hash_length)?;
    if bytes.get(..4)? != b"DIRC" {
        return None;
    }
    let version: u32 = be32(bytes, 4)?;
    if !(2..=4).contains(&version) {
        return None;
    }
    let count: usize = usize::try_from(be32(bytes, 8)?).ok()?;
    let mut entries: Vec<IndexEntry> = Vec::with_capacity(count.min(1 << 20));
    // Where each entry begins, which is what an offset table (`IEOT`) names.
    //  Kept for every file, because which extensions follow the entries is known only after they are read.
    let mut starts: Vec<u32> = Vec::with_capacity(count.min(1 << 20));
    let mut at: usize = HEADER_LENGTH;
    let mut previous: Vec<u8> = Vec::new();
    for _ in 0..count {
        starts.push(u32::try_from(at).ok()?);
        let mode: u32 = be32(bytes, at.checked_add(24)?)?;
        let id_at: usize = at.checked_add(STAT_LENGTH)?;
        let id: Vec<u8> = bytes.get(id_at..id_at.checked_add(hash_length)?)?.to_vec();
        let flags_at: usize = id_at + hash_length;
        let flags: u16 = be16(bytes, flags_at)?;
        let mut head: usize = STAT_LENGTH + hash_length + 2;
        let mut extended: u16 = 0;
        if flags & 0x4000 != 0 {
            if version < 3 {
                return None;
            }
            extended = be16(bytes, flags_at + 2)?;
            head += 2;
        }
        let name_start: usize = at.checked_add(head)?;
        let path: Vec<u8>;
        if version == 4 {
            let mut cursor: usize = name_start;
            let mut byte: u8 = *bytes.get(cursor)?;
            cursor += 1;
            let mut strip: usize = usize::from(byte & 0x7f);
            while byte & 0x80 != 0 {
                byte = *bytes.get(cursor)?;
                cursor += 1;
                strip = strip.checked_add(1)?.checked_shl(7)? | usize::from(byte & 0x7f);
            }
            let keep: usize = previous.len().checked_sub(strip)?;
            let terminator: usize = first_index_of_byte(bytes.get(cursor..end)?, 0)?;
            let mut joined: Vec<u8> = previous[..keep].to_vec();
            joined.extend_from_slice(&bytes[cursor..cursor + terminator]);
            path = joined;
            at = cursor + terminator + 1;
        } else {
            let terminator: usize = first_index_of_byte(bytes.get(name_start..end)?, 0)?;
            path = bytes[name_start..name_start + terminator].to_vec();
            at = at.checked_add((head + terminator + 8) & !7)?;
        }
        // Only the intent-to-add and skip-worktree bits are defined; Git canonicalizes other modes.
        let defined: bool = extended & !(INTENT_TO_ADD | 0x4000) == 0;
        let canonical: bool = matches!(mode, 0o100_644 | 0o100_755 | 0o120_000 | 0o160_000);
        if at > end || path.is_empty() || !defined || !canonical {
            return None;
        }
        previous.clone_from(&path);
        let stage: u8 = ((flags >> 12) & 3) as u8;
        entries.push(IndexEntry { path, mode, id, stage, extended });
    }
    for pair in entries.windows(2) {
        if (pair[0].path.as_slice(), pair[0].stage) >= (pair[1].path.as_slice(), pair[1].stage) {
            return None;
        }
    }
    let mut tree: Option<CacheTree> = None;
    while at < end {
        let signature: &[u8] = bytes.get(at..at.checked_add(4)?)?;
        let optional: bool = signature[0].is_ascii_uppercase();
        if !signature.iter().all(u8::is_ascii_alphanumeric) || !optional || signature == b"link" || signature == b"sdir" {
            return None;
        }
        let size: usize = usize::try_from(be32(bytes, at + 4)?).ok()?;
        let body_at: usize = at.checked_add(8)?;
        let body: Option<&[u8]> = bytes.get(body_at..body_at.checked_add(size)?);
        match signature {
            // An extension that does not read as Git writes it is dropped by Git too, so the comparison goes without it.
            b"TREE" => {
                tree = match body {
                    Some(found) => CacheTree::read(found, hash_length),
                    None => None,
                };
            }
            // Git words a `REUC` or an `FSMN` body it cannot read on standard error and answers anyway, so both must read here.
            b"REUC" => {
                crate::index_resolve_undo::ResolveUndo::read(body?, hash_length)?;
            }
            b"FSMN" => {
                crate::index_fsmonitor::read_fsmonitor(body?, entries.len())?;
            }
            // Git reads these without a word on any body it is handed, measured on Git 2.56.0 with each body `abcd`.
            b"UNTR" | b"EOIE" => {}
            // Git reads the entries through this table when threaded loading is on,
            //  so only a table naming exactly the entries read here is answered.
            b"IEOT" => {
                offset_table_is_silent(body?, starts.as_slice())?;
            }
            // An extension this reader does not know is one Git words (`ignoring <signature> extension`) or dies on.
            _ => return None,
        }
        at = body_at.checked_add(size)?;
    }
    if at != end {
        return None;
    }
    return Some(ParsedIndex { entries, tree });
}

#[cfg(test)]
#[path = "repo_read_index_tests.rs"]
mod tests;
