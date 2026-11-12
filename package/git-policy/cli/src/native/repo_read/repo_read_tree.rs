//! What:
//!  Trees and commits read in process:
//! the entries of a tree,
//! the flat list of a whole tree,
//! and the header fields
//!  and subject of a commit.
//! Why:
//!  The diff and listing readers compare a commit's tree with an index or another tree path by path,
//! and `show` prints a
//!  commit's subject;
//! both need only these parses,
//! and a commit with a header the parser does not know
//!  (a different message encoding,
//! a signature it must not interpret) returns `None` so Git answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries = parseTree(bytes, hashLength); const flat = flattenTree(odb, treeName);
//! ```

#![cfg(unix)]

use crate::index_byte_search::first_index_of_byte;
use crate::repo_read_object::{ObjectKind, name_length};
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_odb::ObjectDatabase;
use std::cell::RefCell;
use std::rc::Rc;

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
    /// The mode as stored (`0o100644`,
    /// `0o040000`,
    /// ...).
    pub mode: u32,
    /// The entry name,
    /// which holds no `/`.
    pub name: Vec<u8>,
    /// The object name.
    pub id: Vec<u8>,
}

/// One non-directory entry of a flattened tree with its full path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlatEntry {
    /// The path from the top of the tree,
    /// `/` separated.
    pub path: Vec<u8>,
    /// The mode.
    pub mode: u32,
    /// The object name.
    pub id: Vec<u8>,
}

/// The entries of a tree object's content,
/// or `None` when any entry is malformed.
pub fn parse_tree(bytes: &[u8], hash_length: usize) -> Option<Vec<TreeEntry>> {
    let mut entries: Vec<TreeEntry> = Vec::new();
    let mut cursor: usize = 0;
    while cursor < bytes.len() {
        let space: usize = cursor + first_index_of_byte(&bytes[cursor..], b' ')?;
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
        let nul: usize = space + 1 + first_index_of_byte(&bytes[space + 1..], 0)?;
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
///  Every non-directory entry below the tree `name`,
/// with full paths,
/// in the order Git walks them.
/// Why:
///   Tree order sorts a directory as if its name ended in `/`,
/// which is the byte order of the full paths,
/// so the list
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

/// The entries of the tree object `name`,
/// or nothing when the object is missing,
/// is not a tree or does not parse.
fn tree_entries_named(database: &ObjectDatabase, name: &[u8], format: ObjectFormat) -> Option<Vec<TreeEntry>> {
    let object = database.read(name)?;
    if object.kind != ObjectKind::Tree {
        return None;
    }
    return parse_tree(object.bytes.as_slice(), name_length(format));
}

/// Whether `byte` is a newline.
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  The flat list of a tree,
/// shared with every other question this process asked about the same tree.
/// Why:
///   One commit asks about its base tree a dozen times,
/// through a database opened afresh by each question;
/// a tree's name
///  fixes its content,
/// so the list is kept by name and handed out again.
/// The list of a few trees is kept,
/// the oldest dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flattenTreeShared(odb, tree, format): readonly FlatEntry[] | undefined; // same list, one read per process
/// ```
pub fn flatten_tree_shared(database: &ObjectDatabase, tree: &[u8], format: ObjectFormat) -> Option<Rc<Vec<FlatEntry>>> {
    let key: Vec<u8> = tree.to_vec();
    // The thread's lists are taken out for the look and put back, because a `thread_local` can lend them only to a closure.
    let held: Vec<(Vec<u8>, Rc<Vec<FlatEntry>>)> = SHARED_LISTS.take();
    let mut kept: Option<Rc<Vec<FlatEntry>>> = None;
    for (name, list) in &held {
        if *name == key {
            kept = Some(Rc::clone(list));
            break;
        }
    }
    SHARED_LISTS.set(held);
    if let Some(found) = kept {
        return Some(found);
    }
    let listed: Rc<Vec<FlatEntry>> = Rc::new(flatten_tree(database, tree, format)?);
    let mut kept_lists: Vec<(Vec<u8>, Rc<Vec<FlatEntry>>)> = SHARED_LISTS.take();
    if kept_lists.len() >= SHARED_LIST_LIMIT {
        kept_lists.remove(0);
    }
    kept_lists.push((key, Rc::clone(&listed)));
    SHARED_LISTS.set(kept_lists);
    return Some(listed);
}

/// How many trees' lists a thread keeps for later questions.
const SHARED_LIST_LIMIT: usize = 3;

thread_local! {
    /// The lists `flatten_tree_shared` keeps,
    /// oldest first,
    /// by the tree's name.
    static SHARED_LISTS: RefCell<Vec<(Vec<u8>, Rc<Vec<FlatEntry>>)>> = const { RefCell::new(Vec::new()) };
}

/// What:
///  The entries that differ between two trees,
/// as two flat lists in path order:
/// the old side's entries and the new side's,
///  with every subtree both sides share left unread.
/// Why:
///   A listing of what a commit changed needs only the paths that differ,
/// and the repository's trees are mostly equal from
///  one commit to the next,
/// so equal subtree names end the walk there instead of reading every file of a large checkout.
///  A name that is a file on one side and a directory on the other appears as a file in one list and as paths below it in
///  the other,
/// which `diff_sorted` then declines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function flattenPair(odb, oldTree, newTree, format): [FlatEntry[], FlatEntry[]] | undefined;
/// ```
pub fn flatten_pair(database: &ObjectDatabase, old: &[u8], new: &[u8], format: ObjectFormat) -> Option<(Vec<FlatEntry>, Vec<FlatEntry>)> {
    let mut before: Vec<FlatEntry> = Vec::new();
    let mut after: Vec<FlatEntry> = Vec::new();
    walk_pair(database, (old, new), format, &mut Vec::new(), 0, (&mut before, &mut after))?;
    return Some((before, after));
}

/// The sort key of a tree entry:
/// a directory sorts as if its name ended in a slash.
fn tree_order_key(entry: &TreeEntry) -> Vec<u8> {
    let mut key: Vec<u8> = entry.name.clone();
    if is_tree_mode(entry.mode) {
        key.push(b'/');
    }
    return key;
}

/// Append one entry of one side whole:
/// a file as itself,
/// a directory as every file below it.
fn push_side(database: &ObjectDatabase, entry: &TreeEntry, format: ObjectFormat, prefix: &mut Vec<u8>, depth: usize, flat: &mut Vec<FlatEntry>) -> Option<()> {
    let length: usize = prefix.len();
    prefix.extend_from_slice(entry.name.as_slice());
    if is_tree_mode(entry.mode) {
        prefix.push(b'/');
        walk(database, entry.id.as_slice(), format, prefix, depth + 1, flat)?;
    } else {
        flat.push(FlatEntry { path: prefix.clone(), mode: entry.mode, id: entry.id.clone() });
    }
    prefix.truncate(length);
    return Some(());
}

/// Walk two trees together below `prefix`,
/// appending only what differs.
fn walk_pair(
    database: &ObjectDatabase,
    trees: (&[u8], &[u8]),
    format: ObjectFormat,
    prefix: &mut Vec<u8>,
    depth: usize,
    flats: (&mut Vec<FlatEntry>, &mut Vec<FlatEntry>),
) -> Option<()> {
    if depth > TREE_DEPTH_LIMIT {
        return None;
    }
    let (before, after) = flats;
    let old_entries: Vec<TreeEntry> = tree_entries_named(database, trees.0, format)?;
    let new_entries: Vec<TreeEntry> = tree_entries_named(database, trees.1, format)?;
    let (mut left, mut right): (usize, usize) = (0, 0);
    while left < old_entries.len() || right < new_entries.len() {
        let order = match (old_entries.get(left), new_entries.get(right)) {
            (Some(gone), Some(added)) => tree_order_key(gone).cmp(&tree_order_key(added)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, _) => std::cmp::Ordering::Greater,
        };
        match order {
            std::cmp::Ordering::Less => {
                push_side(database, &old_entries[left], format, prefix, depth, before)?;
                left += 1;
            }
            std::cmp::Ordering::Greater => {
                push_side(database, &new_entries[right], format, prefix, depth, after)?;
                right += 1;
            }
            std::cmp::Ordering::Equal => {
                let (was, now): (&TreeEntry, &TreeEntry) = (&old_entries[left], &new_entries[right]);
                if was.id != now.id || was.mode != now.mode {
                    if is_tree_mode(was.mode) {
                        let length: usize = prefix.len();
                        prefix.extend_from_slice(was.name.as_slice());
                        prefix.push(b'/');
                        walk_pair(database, (was.id.as_slice(), now.id.as_slice()), format, prefix, depth + 1, (&mut *before, &mut *after))?;
                        prefix.truncate(length);
                    } else {
                        push_side(database, was, format, prefix, depth, before)?;
                        push_side(database, now, format, prefix, depth, after)?;
                    }
                }
                left += 1;
                right += 1;
            }
        }
        if before.len() + after.len() > FLAT_ENTRY_LIMIT {
            return None;
        }
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
    /// The message:
    /// everything after the blank line that ends the header.
    pub message: Vec<u8>,
}

/// What:
///  The tree,
/// parents and message of a commit object's content;
///  `None` for a header with an `encoding` line (Git would re-encode the message) or a malformed one.
/// Why:
///   `show --format=%s` prints the message converted to UTF-8;
/// only a commit that needs no conversion is read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCommit(bytes: Uint8Array, format): CommitFields | undefined;
/// ```
pub fn parse_commit(bytes: &[u8], format: ObjectFormat) -> Option<CommitFields> {
    let mut blank_line_at: Option<usize> = None;
    for (position, pair) in bytes.windows(2).enumerate() {
        if pair == b"\n\n" {
            blank_line_at = Some(position);
            break;
        }
    }
    let split: usize = blank_line_at?;
    let header: &[u8] = &bytes[..split];
    let mut tree: Option<Vec<u8>> = None;
    let mut parents: Vec<Vec<u8>> = Vec::new();
    for (index, line) in header.split(is_newline).enumerate() {
        if let Some(text) = line.strip_prefix(b"tree ") {
            if index != 0 {
                return None;
            }
            tree = Some(crate::repo_read_object::from_hex(text, format)?);
        } else if let Some(text) = line.strip_prefix(b"parent ") {
            parents.push(crate::repo_read_object::from_hex(text, format)?);
        } else if line.starts_with(b"encoding ") {
            return None;
        }
    }
    return Some(CommitFields { tree: tree?, parents, message: bytes[split + 2..].to_vec() });
}

/// What:
///  Whether Git reads this byte at the end of a line as white space:
/// a space,
/// a tab,
/// a line break or a carriage return.
/// Why:
///   This is the set Git 2.56.0's `sane_ctype` table marks as space,
///  which its `is_blank_line` in `pretty.c` removes;
/// a form feed or a vertical tab is text to Git,
///  as Rust's own ASCII trim would not agree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endsWithGitWhitespace = (c: number) => c === 0x20 || c === 0x09 || c === 0x0a || c === 0x0d;
/// ```
fn git_takes_off_the_end(byte: u8) -> bool {
    return matches!(byte, b' ' | b'\t' | b'\n' | b'\r');
}

/// What:
///  The subject `%s` prints for a message:
/// the first paragraph with its lines joined by spaces.
/// Why:
///   Git skips leading blank lines,
/// stops at the first blank line,
/// trims each line's trailing white space and joins
///  with a single space,
/// where the white space is `git_takes_off_the_end`'s set and not everything Rust trims;
/// any message the plain rule does not cover
///  (a carriage return inside a line,
/// a NUL) returns `None`.
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
    for line in message.split(is_newline) {
        let end: usize = line.iter().rposition(|byte| { return !git_takes_off_the_end(*byte); }).map_or(0, |at| { return at + 1; });
        let trimmed: &[u8] = &line[..end];
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
        subject.extend_from_slice(trimmed);
    }
    return Some(subject);
}

#[cfg(test)]
#[path = "repo_read_tree_tests.rs"]
mod tests;
