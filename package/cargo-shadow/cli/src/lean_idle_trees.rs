//! What:
//! Mechanism C,
//! the collection of verified-idle trees in the host build base.
//! Why:
//! Worktrees created and discarded leave their build trees behind forever
//! (`doc/decision/cargo-build-dir-host-cache.md`,
//! "Throwaway worktree builds outlive their worktree").
//! A tree is removed only when all hold:
//! its workspace is gone (confirmed absent,
//! or it belongs to a retired toolchain,
//! or nothing identifies its workspace and it has been idle
//! longer),
//! it has been idle longer than the period for its case,
//! no other tree's workspace lies inside it,
//! every profile directory yields its build locks without waiting,
//! a scan of every process finds none using anything under it,
//! and its identity is the one judged.
//! The tree root is renamed into the trash while the locks are held,
//! the locks are released,
//! and the trash is deleted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function collectIdleTrees(base: string, policy: IdlePolicy, env: GuardEnvironment, now: number, dryRun: boolean): TreeOutcome[]
//! ```
//!
//! The collection's parts are one file each:
//! the idle periods and what a pass says it did (`idle_policy_and_verdicts.rs`),
//! the decision that a tree qualifies (`tree_qualification.rs`),
//! the records that guard other trees (`workspace_guards.rs`),
//! and the removal of one qualified tree (`qualified_tree_removal.rs`).

/// The streaming log line.
use crate::lean_run_report::emit;
/// Paths and sets.
use std::{
    collections::HashSet,
    path::{
        Path,
        PathBuf,
    },
};

/// The tree readers.
use crate::lean_build_trees::{
    Tree,
    Workspace,
    last_activity,
    workspace_of,
};
/// The environment the guards read.
use crate::lean_guard::GuardEnvironment;
/// The run's deadline.
use crate::lean_identity::Deadline;
/// Imports the names this file uses from `crate::lean_removal`.
use crate::lean_removal::{
    TRASH_DIRECTORY_NAME,
    allocated_bytes_of,
    empty_trash,
    modes_named,
};
/// Imports the names this file uses from `crate::lean_sizes`.
use crate::lean_sizes::tree_bytes;

/// The idle periods of mechanism C and what a pass says it did with each tree.
mod idle_policy_and_verdicts;
/// The removal of one qualified tree under the protocol.
mod qualified_tree_removal;
/// The decision that one tree qualifies for removal.
mod tree_qualification;
/// The records that guard another tree from being removed.
mod workspace_guards;

/// Re-exports the idle periods and what a pass says it did with each tree.
pub use self::idle_policy_and_verdicts::{
    IdlePolicy,
    TreeCollection,
    TreeOutcome,
    TreeVerdict,
};
/// Imports the removal of one qualified tree.
use self::qualified_tree_removal::collect_one;
/// Imports the decision that a tree qualifies.
use self::tree_qualification::qualifies;
/// Re-exports the guard list the records of the trees make.
pub use self::workspace_guards::ledger_roots;
/// Imports the record questions a pass asks of the guard list.
use self::workspace_guards::{
    holds_other_workspace,
    records_a_workspace_inside,
};
/// What:
/// `pub fn collect_idle_trees(...) -> TreeCollection`
/// judges every tree it is given and removes the ones that qualify,
/// holding the workspaces the trees it is not given record as guards.
/// Why:
/// It is mechanism C.
/// A dry run reports what would be removed with its size and removes nothing.
/// The deadline stops the judging,
/// and the count of judged trees is where the next run continues from.
/// The trees before the position this pass continues from are still trees of the base,
/// and a tree this pass judges may hold one of their workspaces:
/// the guard covers them too,
/// so resuming never loses the guard a pass from the front had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function collectIdleTrees(trees: Tree[], earlier: Tree[], base: string, retired: Set<string>, policy: IdlePolicy, env: GuardEnvironment, now: number, dryRun: boolean, deadline: Deadline, log: string[]): TreeCollection
/// ```
pub fn collect_idle_trees(
    trees: &[Tree],
    earlier: &[Tree],
    base: &Path,
    retired: &HashSet<String>,
    policy: &IdlePolicy,
    environment: &GuardEnvironment,
    now: i64,
    dry_run: bool,
    deadline: &Deadline,
    log: &mut Vec<String>,
) -> TreeCollection {
    let mut collection: TreeCollection = TreeCollection::default();
    // The workspaces are read once for the whole list, before the deadline applies to the judging,
    // so the run says how long that took: it is a fixed cost of every pass, however small the bound is.
    let reading_began: std::time::Instant = std::time::Instant::now();
    // The trees this pass does not judge are guards all the same, so their records are read first:
    // a resumed pass starts in the middle of the list, and a tree before that position may record a workspace
    // inside a tree this pass would otherwise remove.
    let mut roots: Vec<(PathBuf, PathBuf)> = ledger_roots(earlier, deadline);
    // Read once and kept: the same answer decides whether a tree holds another tree's workspace,
    // and whether the tree itself qualifies.
    let mut workspaces: Vec<Workspace> = Vec::with_capacity(trees.len());
    for tree in trees {
        let found: Workspace = workspace_of(&tree.path);
        if let Workspace::Known(root) = &found {
            roots.push((tree.path.clone(), root.clone()));
        }
        workspaces.push(found);
    }
    if !trees.is_empty() || !earlier.is_empty() {
        emit(
            log,
            format!(
                "read the recorded workspace of {} trees in {} s",
                trees.len() + earlier.len(),
                reading_began.elapsed().as_secs()
            ),
        );
    }
    let trash: PathBuf = base.join(TRASH_DIRECTORY_NAME);
    if !dry_run {
        let sweep = empty_trash(&trash);
        if !sweep.modes_added.is_empty() {
            emit(
                log,
                format!(
                    "finished an earlier run's deletion in {} ({} disk bytes){};",
                    trash.display(),
                    sweep.bytes,
                    modes_named(&sweep.modes_added)
                ),
            );
        }
        for failure in sweep.failures {
            emit(
                log,
                format!(
                    "could not finish an earlier run's deletion in {}: {failure}",
                    trash.display()
                ),
            );
        }
    }
    for (index, tree) in trees.iter().enumerate() {
        if deadline.passed() {
            break;
        }
        let Some(activity) = last_activity(&tree.path, deadline) else {
            // Its idleness was not read, so nothing was decided about it: the next run judges it first.
            break;
        };
        collection.judged += 1;
        let (reason, needed): (String, i64) =
            match qualifies(tree, &workspaces[index], retired, policy, now, activity) {
                Ok(found) => found,
                Err(why) => {
                    collection.outcomes.push(TreeOutcome {
                        tree: tree.path.clone(),
                        verdict: TreeVerdict::Keep(why),
                    });
                    continue;
                }
            };
        if let Some(inside) = holds_other_workspace(&tree.path, &roots) {
            collection.outcomes.push(TreeOutcome {
                tree: tree.path.clone(),
                verdict: TreeVerdict::Keep(format!(
                    "the workspace of {} lies inside it",
                    inside.display()
                )),
            });
            continue;
        }
        // The record this tree holds may be what keeps another tree from being removed, so it stays while it does.
        if let Some(guarded) =
            records_a_workspace_inside(&workspaces[index], &[trees, earlier], &tree.path)
        {
            collection.outcomes.push(TreeOutcome {
                tree: tree.path.clone(),
                verdict: TreeVerdict::Keep(format!(
                    "the workspace it records lies inside {}, whose guard that record is",
                    guarded.display()
                )),
            });
            continue;
        }
        if dry_run {
            let bytes: u64 = tree_bytes(&tree.path, deadline, allocated_bytes_of).bytes;
            emit(
                log,
                format!(
                    "would remove tree {} ({bytes} disk bytes): {reason}",
                    tree.path.display()
                ),
            );
            collection.outcomes.push(TreeOutcome {
                tree: tree.path.clone(),
                verdict: TreeVerdict::Removed {
                    reason,
                    bytes,
                },
            });
            continue;
        }
        let verdict: TreeVerdict = collect_one(
            tree,
            &reason,
            &trash,
            environment,
            now,
            needed,
            deadline,
            log,
        );
        collection.outcomes.push(TreeOutcome {
            tree: tree.path.clone(),
            verdict,
        });
    }
    let _ = std::fs::remove_dir(&trash);
    return collection;
}

/// The tests of the idle-tree collection.
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "lean_idle_trees_tests.rs"]
mod lean_idle_trees_tests;
