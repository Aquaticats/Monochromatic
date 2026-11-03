//! What:
//!  The tree questions an in-process path reset asks: a commit's root tree, the entries of that tree at or below given
//!  paths, and the paths that differ between two commits' trees.
//! Why:
//!  `reset <commit> -- <paths>` reads only those paths of the commit, and the landing's follow-up lists what changed between
//!  the branch before and after; walking down the named paths reads a few small trees instead of listing a whole checkout,
//!  and equal subtrees end the comparison of two trees at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tree = rootTreeOf(db, commit); const entries = entriesAt(db, tree, names); const paths = changedPaths(db, oldTree, newTree);
//! ```

use super::repo_read_layout::ObjectFormat;
use super::repo_read_object::{ObjectKind, from_hex, name_length};
use super::repo_read_odb::ObjectDatabase;
use super::repo_read_tree::{FlatEntry, TreeEntry, flatten_pair, flatten_tree, is_tree_mode, parse_commit, parse_tree};

/// The root tree of the commit named by `hex`, or of the tree it names.
pub fn root_tree(database: &ObjectDatabase, hex: &str, format: ObjectFormat) -> Option<Vec<u8>> {
    let name: Vec<u8> = from_hex(hex.as_bytes(), format)?;
    let object = database.read(name.as_slice())?;
    return match object.kind {
        ObjectKind::Tree => Some(name),
        ObjectKind::Commit => Some(parse_commit(object.bytes.as_slice(), format)?.tree),
        ObjectKind::Blob | ObjectKind::Tag => None,
    };
}

/// The entries of the tree `tree`.
fn entries_of(database: &ObjectDatabase, tree: &[u8], format: ObjectFormat) -> Option<Vec<TreeEntry>> {
    let object = database.read(tree)?;
    if object.kind != ObjectKind::Tree {
        return None;
    }
    return parse_tree(object.bytes.as_slice(), name_length(format));
}

/// The files of the tree at `name` or below it, with their paths from the top: nothing when the name is absent, the one entry
/// when it is a file.
fn entries_at(database: &ObjectDatabase, root: &[u8], name: &[u8], format: ObjectFormat) -> Option<Vec<FlatEntry>> {
    let components: Vec<&[u8]> = name.split(|byte| { return *byte == b'/'; }).collect();
    let mut tree: Vec<u8> = root.to_vec();
    for (position, component) in components.iter().enumerate() {
        let entries: Vec<TreeEntry> = entries_of(database, tree.as_slice(), format)?;
        let Some(found) = entries.into_iter().find(|entry| { return entry.name.as_slice() == *component; }) else {
            return Some(Vec::new());
        };
        let last: bool = position + 1 == components.len();
        if !last {
            if !is_tree_mode(found.mode) {
                return Some(Vec::new());
            }
            tree = found.id;
            continue;
        }
        if !is_tree_mode(found.mode) {
            return Some(vec![FlatEntry { path: name.to_vec(), mode: found.mode, id: found.id }]);
        }
        let below: Vec<FlatEntry> = flatten_tree(database, found.id.as_slice(), format)?;
        return Some(below.into_iter().map(|entry| {
            let mut path: Vec<u8> = name.to_vec();
            path.push(b'/');
            path.extend_from_slice(entry.path.as_slice());
            return FlatEntry { path, mode: entry.mode, id: entry.id };
        }).collect());
    }
    return Some(Vec::new());
}

/// The tree's files at or below each of `names`, sorted by path without repeats.
pub fn tree_entries_at(database: &ObjectDatabase, root: &[u8], names: &[Vec<u8>], format: ObjectFormat) -> Option<Vec<FlatEntry>> {
    let mut found: Vec<FlatEntry> = Vec::new();
    for name in names {
        found.extend(entries_at(database, root, name.as_slice(), format)?);
    }
    found.sort_by(|left, right| { return left.path.cmp(&right.path); });
    found.dedup_by(|later, earlier| { return later.path == earlier.path; });
    return Some(found);
}

/// The paths that differ between the trees (`diff-tree -r --name-only --no-renames`), sorted without repeats; `None` for the
/// old tree stands for the empty tree.
pub fn changed_paths(database: &ObjectDatabase, old: Option<&[u8]>, new: &[u8], format: ObjectFormat) -> Option<Vec<Vec<u8>>> {
    let (before, after): (Vec<FlatEntry>, Vec<FlatEntry>) = match old {
        Some(tree) => flatten_pair(database, tree, new, format)?,
        None => (Vec::new(), flatten_tree(database, new, format)?),
    };
    let mut paths: Vec<Vec<u8>> = before.into_iter().chain(after).map(|entry| { return entry.path; }).collect();
    paths.sort();
    paths.dedup();
    return Some(paths);
}

#[cfg(test)]
#[path = "index_reset_tree_tests.rs"]
mod tests;
