//! What:
//!  Pure tree comparison and captured-delta replay over flattened trees: the changes between a base and a result tree,
//!  and the tree that results from replaying those changes onto a current tree, with the conflicts it cannot settle.
//! Why:
//!  "Replay of the captured change" (`doc/planning/cli-git-commits-never-fail.md`) builds the tree to publish as the
//!  three-way merge of base `B`, current `C` and result `T`. Step 1 builds it with explicit inputs only: it reads no
//!  ref, index or worktree and changes none, and its answer is compared with `git merge-tree --write-tree
//!  --merge-base=B C T` in `commit_replay_tests.rs`.
//!  The granularity is the path (mode and object name merged separately); there is no line merge and no rename
//!  detection, so a text edit on both sides of one path is a conflict here where Git may merge it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // function replay(base: Entry[], result: Entry[], current: Entry[]): { entries: Entry[]; conflicts: Conflict[] };
//! ```

// What:     `use std::collections::BTreeMap;` imports a key-to-value table that keeps its keys sorted.
//           Sibling: `HashMap`, which does not keep an order.
// Why:      The comparison walks paths in one fixed order and the output is sorted without a separate sort.
//
// In TS you'd write (pseudocode):
// ```ts
// // a Map whose keys are sorted when iterated
// ```
use std::collections::{BTreeMap, BTreeSet};

use crate::repo_read_tree::FlatEntry;

/// The mode and object name of one path.
type Cell = (u32, Vec<u8>);

/// What:
///  `struct Delta { ... }`: one changed path, with its old and new cell (`None` when absent on that side).
/// Why:
///  A captured delta is exactly the paths whose cell differs between base and result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Delta = { path: Uint8Array; old?: Cell; new?: Cell };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delta {
    /// The path from the top of the tree.
    pub path: Vec<u8>,
    /// The base side: mode and object name, or `None` for an added path.
    pub old: Option<(u32, Vec<u8>)>,
    /// The result side, or `None` for a deleted path.
    pub new: Option<(u32, Vec<u8>)>,
}

/// What:
///  `enum ConflictKind { ... }`: why a path could not be settled.
/// Why:
///  The caller records the conflict and keeps the captured inputs (the design's "a replay conflict keeps the inputs").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConflictKind = 'both-changed' | 'directory-file';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictKind {
    /// The current side and the result both changed the path, differently (including edit against delete and add
    /// against add).
    BothChanged,
    /// Base, current and result are three different gitlinks (submodule commits). Whether one descends from the others is an
    /// ancestry question only the submodule's repository can answer, so the caller falls back to Git or that repository.
    GitlinkNeedsAncestry,
    /// A path is a file on one side and has entries below it on the other.
    DirectoryFile,
}

/// A path that could not be settled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Conflict {
    /// The path.
    pub path: Vec<u8>,
    /// Why.
    pub kind: ConflictKind,
}

/// The outcome of a replay: the settled entries in path order, and the conflicts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replayed {
    /// Every settled path of the resulting tree; a conflicted path keeps the current side's cell, or is absent.
    pub entries: Vec<FlatEntry>,
    /// The paths that could not be settled.
    pub conflicts: Vec<Conflict>,
}

/// Indexes a flattened tree by path.
fn index(entries: &[FlatEntry]) -> BTreeMap<Vec<u8>, Cell> {
    let mut table: BTreeMap<Vec<u8>, Cell> = BTreeMap::new();
    for entry in entries {
        table.insert(entry.path.clone(), (entry.mode, entry.id.clone()));
    }
    return table;
}

/// What:
///  The changes from `base` to `result`, in path order. Neither input needs to be sorted.
/// Why:
///  This is the captured delta of a request, computed once at capture; it also serves as the pure tree comparison
///  that the live protocol will use instead of `git diff-tree`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compareTrees(base: FlatEntry[], result: FlatEntry[]): Delta[];
/// ```
pub fn compare_trees(base: &[FlatEntry], result: &[FlatEntry]) -> Vec<Delta> {
    let old: BTreeMap<Vec<u8>, Cell> = index(base);
    let new: BTreeMap<Vec<u8>, Cell> = index(result);
    let mut paths: Vec<&Vec<u8>> = old.keys().chain(new.keys()).collect();
    paths.sort();
    paths.dedup();
    let mut deltas: Vec<Delta> = Vec::new();
    for path in paths {
        let before: Option<&Cell> = old.get(path);
        let after: Option<&Cell> = new.get(path);
        if before != after {
            deltas.push(Delta { path: path.clone(), old: before.cloned(), new: after.cloned() });
        }
    }
    return deltas;
}

/// What:
///  A three-way choice of one value: the side that changed from `base` wins; both changing alike is that change;
///  both changing differently is `Err`.
///  `Result<T, ()>` is success with a value or failure with nothing to say (TS: throw).
/// Why:
///  The same rule settles the mode, the object name and the presence of a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pick<T>(base: T, current: T, result: T): T; // throws on a real conflict
/// ```
fn pick<T: Clone + PartialEq>(base: &T, current: &T, result: &T) -> Result<T, ()> {
    if result == base {
        return Ok(current.clone());
    }
    if current == base || current == result {
        return Ok(result.clone());
    }
    return Err(());
}

/// The merged cell of one path, or `Err` when the sides conflict. Present on all three sides: mode and object name are
/// settled separately. Otherwise the whole cell is settled as one value.
fn merge_cell(base: Option<&Cell>, current: Option<&Cell>, result: Option<&Cell>) -> Result<Option<Cell>, ()> {
    if let (Some(b), Some(c), Some(r)) = (base, current, result) {
        // Mode and object name are settled apart only among regular files (the executable bit is a mode change on one
        // file); a change of file type (regular, symlink, gitlink) on either side settles the whole cell as one value.
        // Both descendants agreeing on a type different from the base is a conversion both asked for and merges as usual; the
        // descendants disagreeing on the type settles the whole cell.
        let descendants_agree: bool = (c.0 & 0o170_000) == (r.0 & 0o170_000);
        if !descendants_agree {
            return pick(&Some(b.clone()), &Some(c.clone()), &Some(r.clone()));
        }
        let mode: u32 = pick(&b.0, &c.0, &r.0)?;
        let id: Vec<u8> = pick(&b.1, &c.1, &r.1)?;
        return Ok(Some((mode, id)));
    }
    let b: Option<Cell> = base.cloned();
    let c: Option<Cell> = current.cloned();
    let r: Option<Cell> = result.cloned();
    return pick(&b, &c, &r);
}

/// What:
///  Replays the change from `base` to `result` onto `current`: for each path, the side that changed from the base wins,
///  both sides changing alike is that change, both changing differently is a conflict; then a path that is a file and
///  has entries below it is a directory/file conflict.
/// Why:
///  The captured delta is applied to the actual tail, never reinterpreted: `base` is the logical head at capture,
///  `result` the captured tree and `current` the tree to publish on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replayDelta(base: FlatEntry[], result: FlatEntry[], current: FlatEntry[]): Replayed;
/// ```
pub fn replay_delta(base: &[FlatEntry], result: &[FlatEntry], current: &[FlatEntry]) -> Replayed {
    let b: BTreeMap<Vec<u8>, Cell> = index(base);
    let r: BTreeMap<Vec<u8>, Cell> = index(result);
    let c: BTreeMap<Vec<u8>, Cell> = index(current);
    let mut paths: Vec<&Vec<u8>> = b.keys().chain(r.keys()).chain(c.keys()).collect();
    paths.sort();
    paths.dedup();
    let mut settled: BTreeMap<Vec<u8>, Cell> = BTreeMap::new();
    let mut conflicts: Vec<Conflict> = Vec::new();
    // The paths already reported, ordered, so "already a conflict" is one lookup and not a scan of the list per leaf.
    let mut reported: BTreeSet<Vec<u8>> = BTreeSet::new();
    for path in paths {
        match merge_cell(b.get(path), c.get(path), r.get(path)) {
            Ok(Some(cell)) => {
                settled.insert(path.clone(), cell);
            }
            Ok(None) => {}
            Err(()) => {
                let gitlinks: bool = [b.get(path), c.get(path), r.get(path)].iter().all(|cell| { return cell.is_some_and(|entry| { return entry.0 == 0o160_000; }); });
                let kind: ConflictKind = if gitlinks { ConflictKind::GitlinkNeedsAncestry } else { ConflictKind::BothChanged };
                conflicts.push(Conflict { path: path.clone(), kind });
                reported.insert(path.clone());
                if let Some(cell) = c.get(path) {
                    settled.insert(path.clone(), cell.clone());
                }
            }
        }
    }
    // A path that is a leaf and also a directory prefix of another path cannot be one tree.
    let leaves: Vec<Vec<u8>> = settled.keys().cloned().collect();
    for leaf in &leaves {
        let mut prefix: Vec<u8> = leaf.clone();
        prefix.push(b'/');
        // The smallest key at or after `leaf/` starts with it exactly when some path lies below `leaf` (one ordered lookup, not a scan).
        let has_children: bool = settled.range(prefix.clone()..).next().is_some_and(|(key, _)| { return key.starts_with(&prefix); });
        if has_children && !reported.contains(leaf) {
            conflicts.push(Conflict { path: leaf.clone(), kind: ConflictKind::DirectoryFile });
        }
    }
    conflicts.sort_by(|left, right| { return left.path.cmp(&right.path); });
    let entries: Vec<FlatEntry> = settled.into_iter().map(|(path, cell)| { return FlatEntry { path, mode: cell.0, id: cell.1 }; }).collect();
    return Replayed { entries, conflicts };
}
