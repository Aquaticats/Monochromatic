//! What:
//!  Trees and commits read in process: the entries of a tree, the flat list of a whole tree, and the header fields
//!  and subject of a commit.
//! Why:
//!  The diff and listing readers compare a commit's tree with an index or another tree path by path, and `show` prints a
//!  commit's subject; both need only these parses, and a commit with a header the parser does not know
//!  (a different message encoding, a signature it must not interpret) returns `None` so Git answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries = parseTree(bytes, hashLength); const flat = flattenTree(odb, treeName);
//! ```

#![cfg(unix)]

use super::repo_read_object::{ObjectKind, name_length};
use super::repo_read_layout::ObjectFormat;
use super::repo_read_odb::ObjectDatabase;

/// The deepest tree nesting the flattener follows.
const TREE_DEPTH_LIMIT: usize = 256;

/// The most entries a flat list may hold before the reader declines.
const FLAT_ENTRY_LIMIT: usize = 4_000_000;

/// The mode of a tree entry that is a directory.
pub const MODE_TREE: u32 = 0o040_000;

/// The mode of a gitlink (a submodule commit).
pub const MODE_GITLINK: u32 = 0o160_000;

/// One entry of a tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeEntry {
    /// The mode as stored (`0o100644`, `0o040000`, ...).
    pub mode: u32,
    /// The entry name, which holds no `/`.
    pub name: Vec<u8>,
    /// The object name.
    pub id: Vec<u8>,
}

/// One non-directory entry of a flattened tree with its full path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlatEntry {
    /// The path from the top of the tree, `/` separated.
    pub path: Vec<u8>,
    /// The mode.
    pub mode: u32,
    /// The object name.
    pub id: Vec<u8>,
}

/// The entries of a tree object's content, or `None` when any entry is malformed.
pub fn parse_tree(bytes: &[u8], hash_length: usize) -> Option<Vec<TreeEntry>> {
    let mut entries: Vec<TreeEntry> = Vec::new();
    let mut cursor: usize = 0;
    while cursor < bytes.len() {
        let space: usize = cursor + bytes[cursor..].iter().position(|byte| { return *byte == b' '; })?;
        let mode_text: &str = std::str::from_utf8(&bytes[cursor..space]).ok()?;
        // Git writes `40000` for a directory and no leading zero for any mode; other spellings are left to Git.
        if mode_text.is_empty() || mode_text.starts_with('0') {
            return None;
        }
        let mode: u32 = u32::from_str_radix(mode_text, 8).ok()?;
        // Git canonicalizes other spellings (`100664` reads as `100644`); only the canonical ones are read here.
        if !matches!(mode, 0o100_644 | 0o100_755 | 0o120_000 | 0o160_000 | MODE_TREE) {
            return None;
        }
        let nul: usize = space + 1 + bytes[space + 1..].iter().position(|byte| { return *byte == 0; })?;
        let name: Vec<u8> = bytes[space + 1..nul].to_vec();
        if name.is_empty() || name.contains(&b'/') {
            return None;
        }
        let id: Vec<u8> = bytes.get(nul + 1..nul + 1 + hash_length)?.to_vec();
        entries.push(TreeEntry { mode, name, id });
        cursor = nul + 1 + hash_length;
    }
    return Some(entries);
}

/// Whether the mode names a directory.
pub fn is_tree_mode(mode: u32) -> bool {
    return mode & 0o170_000 == MODE_TREE;
}

/// What:
///  Every non-directory entry below the tree `name`, with full paths, in the order Git walks them.
/// Why:
///   Tree order sorts a directory as if its name ended in `/`, which is the byte order of the full paths, so the list
///  is sorted by path and two lists merge by one pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flattenTree(odb: ObjectDatabase, tree: Uint8Array, format): FlatEntry[] | undefined;
/// ```
pub fn flatten_tree(database: &ObjectDatabase, tree: &[u8], format: ObjectFormat) -> Option<Vec<FlatEntry>> {
    let mut flat: Vec<FlatEntry> = Vec::new();
    walk(database, tree, format, &mut Vec::new(), 0, &mut flat)?;
    return Some(flat);
}

/// Append the entries of `tree` below `prefix` to `flat`.
fn walk(
    database: &ObjectDatabase,
    tree: &[u8],
    format: ObjectFormat,
    prefix: &mut Vec<u8>,
    depth: usize,
    flat: &mut Vec<FlatEntry>,
) -> Option<()> {
    if depth > TREE_DEPTH_LIMIT {
        return None;
    }
    let object = database.read(tree)?;
    if object.kind != ObjectKind::Tree {
        return None;
    }
    for entry in parse_tree(object.bytes.as_slice(), name_length(format))? {
        let length: usize = prefix.len();
        prefix.extend_from_slice(entry.name.as_slice());
        if is_tree_mode(entry.mode) {
            prefix.push(b'/');
            walk(database, entry.id.as_slice(), format, prefix, depth + 1, flat)?;
        } else {
            flat.push(FlatEntry { path: prefix.clone(), mode: entry.mode, id: entry.id });
            if flat.len() > FLAT_ENTRY_LIMIT {
                return None;
            }
        }
        prefix.truncate(length);
    }
    return Some(());
}

/// The fields of a commit the readers use.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitFields {
    /// The root tree.
    pub tree: Vec<u8>,
    /// The parents in order.
    pub parents: Vec<Vec<u8>>,
    /// The message: everything after the blank line that ends the header.
    pub message: Vec<u8>,
}

/// What:
///  The tree, parents and message of a commit object's content;
///  `None` for a header with an `encoding` line (Git would re-encode the message) or a malformed one.
/// Why:
///   `show --format=%s` prints the message converted to UTF-8; only a commit that needs no conversion is read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCommit(bytes: Uint8Array, format): CommitFields | undefined;
/// ```
pub fn parse_commit(bytes: &[u8], format: ObjectFormat) -> Option<CommitFields> {
    let split: usize = bytes.windows(2).position(|pair| { return pair == b"\n\n"; })?;
    let header: &[u8] = &bytes[..split];
    let mut tree: Option<Vec<u8>> = None;
    let mut parents: Vec<Vec<u8>> = Vec::new();
    for (index, line) in header.split(|byte| { return *byte == b'\n'; }).enumerate() {
        if let Some(text) = line.strip_prefix(b"tree ") {
            if index != 0 {
                return None;
            }
            tree = Some(super::repo_read_object::from_hex(text, format)?);
        } else if let Some(text) = line.strip_prefix(b"parent ") {
            parents.push(super::repo_read_object::from_hex(text, format)?);
        } else if line.starts_with(b"encoding ") {
            return None;
        }
    }
    return Some(CommitFields { tree: tree?, parents, message: bytes[split + 2..].to_vec() });
}

/// What:
///  The subject `%s` prints for a message: the first paragraph with its lines joined by spaces.
/// Why:
///   Git skips leading blank lines, stops at the first blank line, trims each line's trailing white space and joins
///  with a single space; any message the plain rule does not cover (a carriage return) returns `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subject = message.replace(/^\n+/, '').split(/\n\s*\n/)[0].split('\n').map((l) => l.trimEnd()).join(' ');
/// ```
pub fn subject_of(message: &[u8]) -> Option<Vec<u8>> {
    if message.contains(&b'\r') || message.contains(&0) {
        return None;
    }
    let mut subject: Vec<u8> = Vec::new();
    let mut started: bool = false;
    for line in message.split(|byte| { return *byte == b'\n'; }) {
        let trimmed: &[u8] = line.trim_ascii_end();
        let blank: bool = trimmed.is_empty();
        if blank && !started {
            continue;
        }
        if blank {
            break;
        }
        if started {
            subject.push(b' ');
        }
        started = true;
        subject.extend_from_slice(line.trim_ascii_end());
    }
    return Some(subject);
}

#[cfg(test)]
#[path = "repo_read_tree_tests.rs"]
mod tests;
