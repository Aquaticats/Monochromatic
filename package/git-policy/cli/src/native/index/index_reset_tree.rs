//! What:
//!  The tree questions an in-process path reset asks:
//! a commit's root tree,
//! the entries of that tree at or below given
//!  paths,
//! and the paths that differ between two commits' trees.
//! Why:
//!  `reset <commit> -- <paths>` reads only those paths of the commit,
//! and the landing's follow-up lists what changed between
//!  the branch before and after;
//! walking down the named paths reads a few small trees instead of listing a whole checkout,
//!  and equal subtrees end the comparison of two trees at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tree = rootTreeOf(db, commit); const entries = entriesAt(db, tree, names); const paths = changedPaths(db, oldTree, newTree);
//! ```

use crate::index_byte_search::is_slash;
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::{ObjectKind, from_hex, name_length};
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{FlatEntry, TREE_DEPTH_LIMIT, TreeEntry, flatten_pair, flatten_tree, is_tree_mode, parse_commit, parse_tree};
use std::collections::BTreeMap;

/// The root tree of the commit named by `hex`,
/// or of the tree it names.
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


/// Order two entries by path,
/// as the sorted list of the index needs.
fn by_path(left: &FlatEntry, right: &FlatEntry) -> std::cmp::Ordering {
    return left.path.cmp(&right.path);
}

/// Whether two neighbours in a sorted list name the same path.
fn same_path(later: &mut FlatEntry, earlier: &mut FlatEntry) -> bool {
    return later.path == earlier.path;
}

/// The first path component of `name` at `offset`, and where that component's children begin.
/// Why:
///  One walk of a tree answers every name below it, so a component is read where it starts
/// and the rest of the name is read in the tree that component names.
fn component_at(name: &[u8], offset: usize) -> (&[u8], usize) {
    if offset >= name.len() {
        return (&[], name.len());
    }
    let rest: &[u8] = &name[offset..];
    let Some(at) = rest.iter().position(is_slash) else {
        return (rest, name.len());
    };
    return (&rest[..at], offset + at + 1);
}

/// The files at or below every name in `names`, read from one tree:
/// each tree object is read once, not once for every name that passes through it.
/// Why:
///  A reset of a commit that changed `N` files names `N` paths, and walking the root and a directory
/// for each of them reads the same tree objects `N` times; the walk here groups the names by their
/// next component, so a landing of 20,000 changed files reads 201 trees rather than 40,000.
/// `offset` is where the component below `prefix` starts in each name, and every name in `names`
/// shares the bytes before it.
fn entries_at_many(
    database: &ObjectDatabase,
    tree: &[u8],
    names: &[Vec<u8>],
    offset: usize,
    format: ObjectFormat,
    prefix: &mut Vec<u8>,
    found: &mut Vec<FlatEntry>,
    depth: usize,
) -> Option<()> {
    if names.is_empty() {
        return Some(());
    }
    if depth > TREE_DEPTH_LIMIT {
        return None;
    }
    let entries: Vec<TreeEntry> = entries_of(database, tree, format)?;
    let by_name: BTreeMap<&[u8], &TreeEntry> = entries.iter().map(|entry| return (entry.name.as_slice(), entry)).collect();
    let mut position: usize = 0;
    while position < names.len() {
        let (component, children): (&[u8], usize) = component_at(names[position].as_slice(), offset);
        let mut group_end: usize = position + 1;
        while group_end < names.len() && component_at(names[group_end].as_slice(), offset).0 == component {
            group_end += 1;
        }
        // An exact name sorts before every name below it, so the first of a group says whether the group names the
        // entry itself or only paths below it.
        let names_below: bool = children < names[position].len();
        if let Some(entry) = by_name.get(component) {
            let mark: usize = prefix.len();
            prefix.extend_from_slice(component);
            if is_tree_mode(entry.mode) {
                if names_below {
                    prefix.push(b'/');
                    entries_at_many(database, entry.id.as_slice(), &names[position..group_end], children, format, prefix, found, depth + 1)?;
                } else {
                    // The name is the directory itself: every path below it is selected.
                    let below: Vec<FlatEntry> = flatten_tree(database, entry.id.as_slice(), format)?;
                    for below_entry in below {
                        let mut path: Vec<u8> = prefix.clone();
                        path.push(b'/');
                        path.extend_from_slice(below_entry.path.as_slice());
                        found.push(FlatEntry { path, mode: below_entry.mode, id: below_entry.id });
                    }
                }
            } else if !names_below {
                found.push(FlatEntry { path: prefix.clone(), mode: entry.mode, id: entry.id.clone() });
            }
            prefix.truncate(mark);
        }
        position = group_end;
    }
    return Some(());
}

/// The tree's files at or below each of `names`,
/// sorted by path without repeats.
/// Why:
///  The names are sorted and deduplicated once, so the walk can group them by component
/// without keeping a map of every path.
///  A name that ends in a slash names the empty path below a directory, which selects nothing,
/// as the component walk found no entry for it; it is dropped here rather than read as its directory.
pub fn tree_entries_at(database: &ObjectDatabase, root: &[u8], names: &[Vec<u8>], format: ObjectFormat) -> Option<Vec<FlatEntry>> {
    let mut ordered: Vec<Vec<u8>> = names.to_vec();
    ordered.retain(|name| return name.last() != Some(&b'/'));
    ordered.sort_unstable();
    ordered.dedup();
    let mut found: Vec<FlatEntry> = Vec::new();
    let mut prefix: Vec<u8> = Vec::new();
    entries_at_many(database, root, ordered.as_slice(), 0, format, &mut prefix, &mut found, 0)?;
    found.sort_by(by_path);
    found.dedup_by(same_path);
    return Some(found);
}

/// The paths that differ between the trees (`diff-tree -r --name-only --no-renames`),
/// sorted without repeats;
/// `None` for the
/// old tree stands for the empty tree.
pub fn changed_paths(database: &ObjectDatabase, old: Option<&[u8]>, new: &[u8], format: ObjectFormat) -> Option<Vec<Vec<u8>>> {
    let (before, after): (Vec<FlatEntry>, Vec<FlatEntry>) = match old {
        Some(tree) => flatten_pair(database, tree, new, format)?,
        None => (Vec::new(), flatten_tree(database, new, format)?),
    };
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for entry in before.into_iter().chain(after) {
        paths.push(entry.path);
    }
    paths.sort();
    paths.dedup();
    return Some(paths);
}

#[cfg(test)]
#[path = "index_reset_tree_tests.rs"]
mod tests;
