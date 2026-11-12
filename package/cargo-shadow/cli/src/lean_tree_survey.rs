//! What:
//! The two reads a pass over the host build base makes before it judges a tree:
//! the ledger of every tree,
//! which is the guard against removing a tree that holds another tree's workspace,
//! and the newest time among each tree's own directories,
//! which orders the pass by age.
//! Why:
//! Both are part of the pass and neither of them removes anything,
//! so they are apart from the judging loop and from the removal it calls.
//! The guard is read for every tree of the base,
//! including the ones the pass will not reach:
//! a tree this pass judges may hold the workspace another one records.
//! The age is a hint and not a proof:
//! a write inside a unit's fingerprint directory changes none of the times it reads,
//! so a tree may look older than it is,
//! and the walk for its last write decides.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function survey(trees: Tree[], retired: Set<string>, deadline: Deadline): Survey
//! ```

/// Paths and sets.
use std::{collections::HashSet, path::PathBuf};

/// The trees,
/// their ledgers and the time of their directories.
use crate::lean_build_trees::{Tree, TreeKind, recent_write_hint};
/// The deadline and the confirmed absence of a workspace.
use crate::lean_identity::{Deadline, missing};
/// The ledger that names the workspace a tree was built for.
use crate::lean_keyed_folder::read_ledger;

/// What the ledger of every tree says,
/// read once before anything is judged.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Survey {
    /// One class per tree of the list,
    /// in the order it was given:
    /// true when the ledger alone shows the tree may qualify (a retired toolchain,
    /// or a workspace confirmed absent).
    pub candidates: Vec<bool>,
    /// Every ledgered workspace of the base,
    /// as pairs of the tree and the workspace it names.
    /// This is the guard list:
    /// a tree that holds one of these workspaces is never removed.
    pub roots: Vec<(PathBuf, PathBuf)>,
    /// One hint per tree,
    /// in the order it was given:
    /// the newest time among its root,
    /// its profile directories and their working directories,
    /// or nothing when the deadline stopped the survey before it read that tree's.
    /// It orders the candidates;
    /// it never decides anything.
    pub hints: Vec<Option<i64>>,
    /// How many trees the survey read.
    pub read: usize,
    /// Whether the deadline stopped the survey before the last tree,
    /// in which case the guard list is incomplete and nothing may be removed.
    pub cut: bool,
}

/// What:
/// `pub(crate) fn survey(trees: &[Tree], retired: &HashSet<String>, deadline: &Deadline) -> Survey` reads the ledger of every tree once,
/// and the newest time among each tree's directories.
/// Why:
/// Three things need it before a single tree is judged:
/// the guard against removing a tree that holds another tree's workspace must cover the whole base,
/// including the trees this pass does not reach;
/// the order the pass judges in puts the trees whose last write is oldest first,
/// so the space a pass frees comes before the pass has read every tree;
/// and the hint ends the question "is this tree old enough" for most trees without the walk that answers it exactly.
/// The hint is the newest time of a tree's own directories,
/// which a write anywhere in the tree does not necessarily change,
/// so it is a lower bound of the newest write:
/// a hint younger than the idle period proves the tree is not old enough,
/// and a hint older than it proves nothing,
/// so the walk decides.
/// Only the ledger and those few times are read for every tree,
/// never the dependency information.
/// Reading stops at the deadline:
/// a guard list that is short of one tree may not be used to remove anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function survey(trees: Tree[], retired: Set<string>, deadline: Deadline): Survey
/// ```
pub(crate) fn survey(trees: &[Tree], retired: &HashSet<String>, deadline: &Deadline) -> Survey {
    let mut found: Survey = Survey { candidates: vec![false; trees.len()], hints: vec![None; trees.len()], ..Survey::default() };
    for (index, tree) in trees.iter().enumerate() {
        if deadline.passed() {
            found.cut = true;
            break;
        }
        let toolchain_retired: bool = match &tree.kind {
            TreeKind::Keyed(name) => retired.contains(name),
            TreeKind::Legacy => false,
        };
        found.hints[index] = recent_write_hint(&tree.path, deadline);
        let Some(root) = read_ledger(&tree.path) else {
            found.candidates[index] = toolchain_retired;
            found.read += 1;
            continue;
        };
        found.candidates[index] = toolchain_retired || missing(&root);
        found.roots.push((tree.path.clone(), root));
        found.read += 1;
    }
    return found;
}

/// What:
/// `pub(crate) fn judging_order(trees: &[Tree], survey: &Survey) -> Vec<usize>` is the indexes of the trees in the order the pass judges them:
/// the oldest last write first,
/// and path order within one time.
/// Why:
/// A pass that is cut by its deadline judges only the front of this order,
/// so the order decides when the space comes out of the pass:
/// measured 2026-11-12 on this host,
/// a pass in path order judged 10,805 trees in 1,095 s and freed nothing,
/// because every tree it reached was younger than its idle period,
/// and the trees that would have gone were further down the same path order.
/// A tree whose hint the deadline stopped is judged after the ones with a time,
/// because the pass has no answer about it and the ones with an answer are the ones it can free now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judgingOrder = (trees: Tree[], survey: Survey): number[] => trees.map((_, index) => index).sort((a, b) => (survey.hints[a] ?? Infinity) - (survey.hints[b] ?? Infinity) || (trees[a].path < trees[b].path ? -1 : trees[a].path > trees[b].path ? 1 : 0));
/// ```
pub(crate) fn judging_order(trees: &[Tree], survey: &Survey) -> Vec<usize> {
    // The key is built once per tree and the sort compares keys, so no comparison reads a field of the survey:
    // a closure that captured them would be a sort the linter cannot see the state of.
    let mut keyed: Vec<(i64, PathBuf, usize)> = Vec::with_capacity(trees.len());
    for (index, tree) in trees.iter().enumerate() {
        keyed.push((survey.hints[index].unwrap_or(i64::MAX), tree.path.clone(), index));
    }
    keyed.sort();
    let mut order: Vec<usize> = Vec::with_capacity(keyed.len());
    for (_hint, _path, index) in keyed {
        order.push(index);
    }
    return order;
}
