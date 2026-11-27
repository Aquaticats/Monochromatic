//! What:
//! The list of workspaces the trees of the base record,
//! and the two questions a pass asks of it:
//! whether a tree holds another tree's workspace,
//! and whether the record a tree holds is what keeps another tree from being removed.
//! Why:
//! A record goes with the tree that holds it,
//! so a tree cannot be removed while it is another tree's guard.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function ledgerRoots(trees: Tree[], deadline: Deadline): [string, string][]
//! function recordsAWorkspaceInside(workspace: Workspace, trees: Tree[][], own: string): string | undefined
//! function holdsOtherWorkspace(tree: string, roots: [string, string][]): string | undefined
//! ```
/// Imports the tree and what identifies the workspace it was built for.
use crate::lean_build_trees::{
    Tree,
    Workspace,
};
/// Imports the deadline that stops the reading and the check that an entry's absence is confirmed.
use crate::lean_identity::{
    Deadline,
    missing,
};
/// Imports the ledger that names the workspace a tree was built for.
use crate::lean_keyed_folder::read_ledger;
/// Imports the borrowed and owned path types the guard list is made of.
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `pub fn ledger_roots(trees: &[Tree], deadline: &Deadline) -> Vec<(PathBuf, PathBuf)>` is every tree among them whose ledger names
/// a workspace,
/// with that workspace.
/// Why:
/// The guard against removing a tree that holds another tree's workspace must cover the whole base.
/// A pass that continues after a recorded position judges only the trees after it,
/// while the trees before it still record workspaces that a tree it judges may hold.
/// Only the ledger is read,
/// never the dependency information,
/// so a guard costs one small file read where identifying an unlabelled tree costs a walk over its dependency files.
/// Reading stops at the deadline:
/// a guard list that grew after it passed would guard trees the pass no longer judges,
/// which it does not need,
/// because a pass past its deadline judges nothing and so removes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const ledgerRoots = (trees: Tree[], deadline: Deadline): [string, string][] => trees.flatMap((tree) => { if (deadline.passed()) return []; const root = readLedger(tree.path); return root === undefined ? [] : [[tree.path, root]]; });
/// ```
pub fn ledger_roots(
    trees: &[Tree],
    deadline: &Deadline,
) -> Vec<(PathBuf, PathBuf)> {
    let mut roots: Vec<(PathBuf, PathBuf)> = Vec::new();
    for tree in trees {
        if deadline.passed() {
            break;
        }
        if let Some(root) = read_ledger(&tree.path) {
            roots.push((tree.path.clone(), root));
        }
    }
    return roots;
}

/// What:
/// `pub(super) fn records_a_workspace_inside(workspace: &Workspace, trees: &[&[Tree]], own: &Path) -> Option<PathBuf>` is the tree
/// of the base,
/// other than the one given,
/// that the recorded workspace of the tree given lies inside.
/// Why:
/// Such a record is the only thing that keeps the other tree from being removed,
/// so a tree that holds it cannot go before the workspace inside that other tree is gone:
/// the record goes with the tree,
/// and the next pass would have nothing to guard with and would delete the live workspace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordsWorkspaceInside = (workspace: Workspace, trees: Tree[][], own: string): string | undefined
/// ```
pub(super) fn records_a_workspace_inside(
    workspace: &Workspace,
    trees: &[&[Tree]],
    own: &Path,
) -> Option<PathBuf> {
    let Workspace::Known(root) = workspace else {
        return None;
    };
    // A workspace whose absence is confirmed holds nothing, so its record guards nothing.
    // An answer other than absence (a directory this user may not search) may hide the workspace, so the record stays a guard.
    if missing(root) {
        return None;
    }
    for list in trees {
        for tree in *list {
            if tree.path != own && root.starts_with(&tree.path) {
                return Some(tree.path.clone());
            }
        }
    }
    return None;
}

/// What:
/// `pub(super) fn holds_other_workspace(tree: &Path, roots: &[(PathBuf, PathBuf)]) -> Option<PathBuf>` finds another tree whose recorded
/// workspace lies inside this tree.
/// Why:
/// A tree built from sources inside another tree is removed first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsOtherWorkspace = (tree: string, roots: [string, string][]): string | undefined
/// ```
pub(super) fn holds_other_workspace(
    tree: &Path,
    roots: &[(PathBuf, PathBuf)],
) -> Option<PathBuf> {
    for (other, root) in roots {
        if other != tree && root.starts_with(tree) {
            return Some(other.clone());
        }
    }
    return None;
}
