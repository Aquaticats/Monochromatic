//! What:
//!  The part of a tree that differs from an index, found by walking the tree beside the index's cache-tree: a directory that
//!  the cache-tree marks valid, with the tree name the tree has there, is equal by construction and is not read.
//! Why:
//!  A commit compares its index with the tree of `HEAD` several times, and the comparison read every directory of the tree
//!  (about 1,400 tree objects, tens of thousands of reads and file probes on the checkout of this repository) to find one
//!  changed file. The index already says which directories changed: Git invalidates the cache-tree along every changed path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { old, skipped } = unequalParts(odb, treeName, cacheTree, index); // undefined: read the whole tree instead
//! ```

#![cfg(unix)]

use crate::index_cache_tree::CacheTree;
use crate::repo_read_diff::{Change, diff_sorted};
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::{ObjectKind, name_length};
use crate::repo_read_odb::ObjectDatabase;
use crate::repo_read_tree::{FlatEntry, is_tree_mode, parse_tree};

/// The deepest tree nesting the walk follows, as `repo_read_tree.rs` follows it.
const TREE_DEPTH_LIMIT: usize = 256;

/// The most entries the tree side may hold before the reader declines, as `repo_read_tree.rs` holds them.
const FLAT_ENTRY_LIMIT: usize = 4_000_000;

/// What the walk found.
#[derive(Debug)]
pub struct UnequalParts {
    /// The entries of the tree outside the directories that were left unread, in path order.
    pub tree: Vec<FlatEntry>,
    /// The ranges of the index (start, end) that belong to the directories left unread, in index order.
    pub skipped: Vec<(usize, usize)>,
}

/// The state of one walk.
struct Walk<'a> {
    /// Where the tree objects are read from.
    database: &'a ObjectDatabase,
    /// The object format, which fixes the name length.
    format: ObjectFormat,
    /// The index paths, sorted.
    paths: &'a [&'a [u8]],
    /// The entries of the tree seen so far.
    tree: Vec<FlatEntry>,
    /// The index ranges left unread so far.
    skipped: Vec<(usize, usize)>,
}

/// The subdirectory `name` of a cache-tree node; the file keeps its subdirectories sorted by length and then bytes.
fn subtree_of<'a>(node: &'a CacheTree, name: &[u8]) -> Option<&'a CacheTree> {
    let at: usize = node.subtrees.binary_search_by(|(candidate, _)| { return (candidate.len(), candidate.as_slice()).cmp(&(name.len(), name)); }).ok()?;
    return Some(&node.subtrees[at].1);
}

/// The range of `range` whose paths start with `prefix`, which is one stretch because the paths are sorted.
fn below(paths: &[&[u8]], range: (usize, usize), prefix: &[u8]) -> (usize, usize) {
    let window: &[&[u8]] = &paths[range.0..range.1];
    let start: usize = window.partition_point(|path| { return *path < prefix; });
    let length: usize = window[start..].partition_point(|path| { return path.starts_with(prefix); });
    return (range.0 + start, range.0 + start + length);
}

impl Walk<'_> {
    /// Walk the directory `tree` (below `prefix`), whose index paths are `range`, and `node` (its cache-tree node when there is one).
    fn directory(&mut self, tree: &[u8], node: Option<&CacheTree>, prefix: &mut Vec<u8>, range: (usize, usize), depth: usize) -> Option<()> {
        if depth > TREE_DEPTH_LIMIT {
            return None;
        }
        if let Some(valid) = node.filter(|candidate| { return candidate.entry_count >= 0 && candidate.id.as_slice() == tree; }) {
            // A node whose count is not the number of index entries below the directory is not trusted; the whole tree is read.
            if usize::try_from(valid.entry_count).ok()? != range.1 - range.0 {
                return None;
            }
            self.skipped.push(range);
            return Some(());
        }
        let object = self.database.read(tree)?;
        if object.kind != ObjectKind::Tree {
            return None;
        }
        for entry in parse_tree(object.bytes.as_slice(), name_length(self.format))? {
            let length: usize = prefix.len();
            prefix.extend_from_slice(entry.name.as_slice());
            if is_tree_mode(entry.mode) {
                prefix.push(b'/');
                let inner: (usize, usize) = below(self.paths, range, prefix.as_slice());
                let child: Option<&CacheTree> = node.and_then(|parent| { return subtree_of(parent, entry.name.as_slice()); });
                self.directory(entry.id.as_slice(), child, prefix, inner, depth + 1)?;
            } else {
                self.tree.push(FlatEntry { path: prefix.clone(), mode: entry.mode, id: entry.id });
                if self.tree.len() > FLAT_ENTRY_LIMIT {
                    return None;
                }
            }
            prefix.truncate(length);
        }
        return Some(());
    }
}

/// What:
///  The entries of the tree `tree` that lie outside the directories the cache-tree proves equal, and the index ranges of those
///  directories; `None` when the walk cannot be trusted (an unreadable tree, a count that does not match the index).
/// Why:
///   Leaving an equal directory out of both sides leaves their difference as it was, so the caller compares what remains the way
///  it compared the whole lists. The index paths must be sorted and free of unmerged and intent-to-add entries, which the caller
///  checks, because a cache-tree says nothing of an entry it left out of the tree it hashed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unequalParts(odb, tree, format, cache, paths): { tree: FlatEntry[]; skipped: [number, number][] } | undefined;
/// ```
pub fn unequal_parts(database: &ObjectDatabase, tree: &[u8], format: ObjectFormat, cache: &CacheTree, paths: &[&[u8]]) -> Option<UnequalParts> {
    // Git reads the tree it compares with and fails when it is gone, so the root is read before any shortcut: a cache-tree that
    //  names it proves the directories below equal, not that the object is there.
    let root = database.read(tree)?;
    if root.kind != ObjectKind::Tree {
        return None;
    }
    let mut walk: Walk<'_> = Walk { database, format, paths, tree: Vec::new(), skipped: Vec::new() };
    walk.directory(tree, Some(cache), &mut Vec::new(), (0, paths.len()), 0)?;
    walk.skipped.sort_unstable();
    return Some(UnequalParts { tree: walk.tree, skipped: walk.skipped });
}

/// What:
///  The items of `all` outside the `skipped` ranges, in order.
/// Why:
///   The index side of the comparison drops the same directories the tree side left unread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function outside<T>(all: T[], skipped: [number, number][]): T[];
/// ```
pub fn outside<'a, Item>(all: &'a [Item], skipped: &[(usize, usize)]) -> Vec<&'a Item> {
    let mut kept: Vec<&Item> = Vec::new();
    let mut at: usize = 0;
    for (start, end) in skipped {
        kept.extend(all[at..*start].iter());
        at = *end;
    }
    kept.extend(all[at..].iter());
    return kept;
}

/// What:
///  The changes from `tree` to `index` found without reading the directories the cache-tree proves equal.
/// Why:
///   The index side drops the same directories the tree side left unread, so `diff_sorted` compares only what can differ and
///  gives the changes the full lists give; `None` when the walk or the comparison declines, and the caller then reads the whole tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function changesWithCache(odb, tree, format, cache, index: FlatEntry[]): Change[] | undefined;
/// ```
pub fn changes_with_cache(database: &ObjectDatabase, tree: &[u8], format: ObjectFormat, cache: &CacheTree, index: &[FlatEntry]) -> Option<Vec<Change>> {
    let paths: Vec<&[u8]> = index.iter().map(|entry| { return entry.path.as_slice(); }).collect();
    let parts: UnequalParts = unequal_parts(database, tree, format, cache, paths.as_slice())?;
    let kept: Vec<FlatEntry> = outside(index, parts.skipped.as_slice()).into_iter().cloned().collect();
    return diff_sorted(parts.tree.as_slice(), kept.as_slice());
}

/// Controls for the walk against real Git's trees and indexes stay out of the release executable.
#[cfg(test)]
#[path = "repo_read_tree_cached_tests.rs"]
mod tests;
