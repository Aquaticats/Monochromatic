//! What:
//! Mechanism C,
//! the collection of verified-idle trees in the host build base.
//! Why:
//! Worktrees created and discarded leave their build trees behind forever
//! (`doc/decision/cargo-build-dir-host-cache.md`,
//! "Throwaway worktree builds outlive their worktree").
//! A tree is removed only when all hold:
//! its workspace is gone (or it belongs to a retired toolchain,
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

/// The streaming log line.
use crate::lean_run_report::emit;
/// Paths and sets.
use std::{collections::HashSet, path::{Path, PathBuf}};

/// The tree readers.
use crate::lean_build_trees::{Tree, TreeKind, Workspace, last_activity, profile_dirs_of, workspace_of};
/// The ledger that names the workspace a tree was built for.
use crate::lean_keyed_folder::read_ledger;
/// The proof and the removal.
use crate::lean_guard::{Guard, GuardEnvironment, Guarded, guard_profile};
/// Imports the names this file uses from `crate::lean_identity`.
use crate::lean_identity::Deadline;
/// Imports the names this file uses from `crate::lean_proc_scan`.
use crate::lean_proc_scan::{ScanReport, scan_processes};
/// Imports the names this file uses from `crate::lean_removal`.
use crate::lean_removal::{Refusal, TRASH_DIRECTORY_NAME, claim_into_trash, delete_claimed, empty_trash, item_of, modes_named};
/// Imports the names this file uses from `crate::lean_sizes`.
use crate::lean_sizes::tree_bytes;

/// The idle periods of mechanism C.
#[derive(Clone, Copy, Debug)]
pub struct IdlePolicy {
    /// A tree whose workspace is gone (or whose toolchain is retired) is removed after this many seconds without a write.
    pub idle_seconds: i64,
    /// A tree nothing identifies is removed after this many seconds without a write.
    pub unidentified_idle_seconds: i64,
}

/// What the collection did with one tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreeVerdict {
    /// Kept,
    /// for this reason.
    Keep(
        /// Why the tree is kept.
        String,
    ),
    /// Not judged or not removable now,
    /// for this reason.
    Skip(
        /// Why the tree was not judged or not removed.
        String,
    ),
    /// Removed (or in a dry run,
    /// would be) with these disk bytes.
    Removed {
        /// The reason it qualified.
        reason: String,
        /// Disk bytes freed or that would be.
        bytes: u64,
    },
    /// Part of it is gone and part of it stayed,
    /// because a directory the walk reached could not be emptied.
    PartlyRemoved {
        /// The reason it qualified.
        reason: String,
        /// Disk bytes its removal freed.
        bytes: u64,
        /// One reason for each part that stayed.
        remaining: String,
    },
}

/// One tree and what happened to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeOutcome {
    /// The tree.
    pub tree: PathBuf,
    /// What was done.
    pub verdict: TreeVerdict,
}

/// What one pass over a list of trees did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TreeCollection {
    /// One outcome per tree that was judged,
    /// in the order given.
    pub outcomes: Vec<TreeOutcome>,
    /// How many of the trees it was given the pass judged before the deadline stopped it,
    /// which is the position the next run continues after.
    pub judged: usize,
}

/// What:
/// `fn qualifies(tree: &Tree, workspace: &Workspace, retired: &HashSet<String>, policy: &IdlePolicy, now: i64, activity: i64) -> Result<(String, i64), String>`
/// decides from the facts whether a tree qualifies:
/// `Ok((reason, idle period that applied))` when it does,
/// `Err(why not)` when it does not.
/// Why:
/// The decision is separate from the removal so it can be read and tested alone,
/// and the workspace it was read once for the whole pass is handed in:
/// reading it again here would repeat the most expensive part of the pass for every tree it judges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function qualifies(tree: Tree, workspace: Workspace, retired: Set<string>, policy: IdlePolicy, now: number, activity: number): { ok: string } | { no: string }
/// ```
fn qualifies(tree: &Tree, workspace: &Workspace, retired: &HashSet<String>, policy: &IdlePolicy, now: i64, activity: i64) -> Result<(String, i64), String> {
    let idle: i64 = now - activity;
    let toolchain_retired: bool = match &tree.kind {
        TreeKind::Keyed(name) => retired.contains(name),
        TreeKind::Legacy => false,
    };
    let (needed, why): (i64, String) = match workspace {
        _ if toolchain_retired => (policy.idle_seconds, String::from("its toolchain is retired")),
        Workspace::Known(root) if root.exists() => return Err(format!("its workspace {} exists", root.display())),
        Workspace::Known(root) => (policy.idle_seconds, format!("its workspace {} is gone", root.display())),
        Workspace::Lead(source) if source.exists() => return Err(format!("the source {} that its dependency information names exists", source.display())),
        Workspace::Lead(source) => (policy.idle_seconds, format!("the source {} that its dependency information names is gone (a lead, not proof)", source.display())),
        Workspace::Unidentified => (policy.unidentified_idle_seconds, String::from("nothing identifies its workspace")),
    };
    if idle <= needed {
        return Err(format!("idle for {idle} s, within the {needed} s period ({why})"));
    }
    return Ok((format!("{why}; idle for {idle} s, longer than {needed} s"), needed));
}

/// What:
/// `pub fn ledger_roots(trees: &[Tree]) -> Vec<(PathBuf, PathBuf)>` is every tree among them whose ledger names a workspace,
/// with that workspace.
/// Why:
/// The guard against removing a tree that holds another tree's workspace must cover the whole base.
/// A pass that continues after a recorded position judges only the trees after it,
/// while the trees before it still record workspaces that a tree it judges may hold.
/// Only the ledger is read,
/// never the dependency information,
/// so a guard costs one small file read where identifying an unlabelled tree costs a walk over its dependency files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const ledgerRoots = (trees: Tree[]): [string, string][] => trees.flatMap((tree) => { const root = readLedger(tree.path); return root === undefined ? [] : [[tree.path, root]]; });
/// ```
pub fn ledger_roots(trees: &[Tree]) -> Vec<(PathBuf, PathBuf)> {
    let mut roots: Vec<(PathBuf, PathBuf)> = Vec::new();
    for tree in trees {
        if let Some(root) = read_ledger(&tree.path) {
            roots.push((tree.path.clone(), root));
        }
    }
    return roots;
}

/// What:
/// `fn records_a_workspace_inside(workspace: &Workspace, trees: &[&[Tree]], own: &Path) -> Option<PathBuf>` is the tree of the
/// base,
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
fn records_a_workspace_inside(workspace: &Workspace, trees: &[&[Tree]], own: &Path) -> Option<PathBuf> {
    let Workspace::Known(root) = workspace else { return None };
    // A workspace that is gone holds nothing, so its record guards nothing.
    if !root.exists() {
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
/// `fn holds_other_workspace(tree: &Path, roots: &[(PathBuf, PathBuf)]) -> Option<PathBuf>` finds another tree whose recorded
/// workspace lies inside this tree.
/// Why:
/// A tree built from sources inside another tree is removed first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsOtherWorkspace = (tree: string, roots: [string, string][]): string | undefined
/// ```
fn holds_other_workspace(tree: &Path, roots: &[(PathBuf, PathBuf)]) -> Option<PathBuf> {
    for (other, root) in roots {
        if other != tree && root.starts_with(tree) {
            return Some(other.clone());
        }
    }
    return None;
}

/// What:
/// `fn collect_one(tree, reason, trash, environment, now, needed, deadline, log) -> TreeVerdict`
/// removes one qualified tree under the protocol.
/// Why:
/// Unlinking what a directory holds needs write and search on it,
/// so a directory this user owns and may not write is made writable first and the removal's proof line says so;
/// a directory of another user stays as it is and is named.
/// A tree that could not be emptied whole is reported as partly removed,
/// never as skipped,
/// because part of it is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collectOne(tree: Tree, reason: string, trash: string, env: GuardEnvironment, now: number, needed: number, deadline: Deadline, log: string[]): TreeVerdict
/// ```
fn collect_one(tree: &Tree, reason: &str, trash: &Path, environment: &GuardEnvironment, now: i64, needed: i64, deadline: &Deadline, log: &mut Vec<String>) -> TreeVerdict {
    let mut held: Vec<Guarded> = Vec::new();
    for profile in profile_dirs_of(&tree.path) {
        match guard_profile(&profile, environment) {
            Guard::Ready(ready) => held.push(ready),
            Guard::Skip(why) => return TreeVerdict::Skip(format!("{}: {why}", profile.display())),
        }
    }
    match last_activity(&tree.path, deadline) {
        Some(activity) if now - activity > needed => {}
        Some(_) => return TreeVerdict::Skip(String::from("a write happened while the locks were being taken")),
        None => return TreeVerdict::Skip(String::from("the deadline passed while reading the tree")),
    }
    let scan: ScanReport = scan_processes(&environment.proc_root, &[tree.path.as_path()], &[], environment.own_pid);
    if let Some(used) = scan.blocks(&tree.path) {
        return TreeVerdict::Skip(format!("pid {} uses {} as {}", used.pid, used.path.display(), used.how));
    }
    let mut proofs: Vec<String> = Vec::new();
    for guarded in &held {
        proofs.push(guarded.proof.clone());
    }
    proofs.push(scan.describe());
    let item = match item_of(&tree.path) {
        Ok(found) => found,
        Err(error) => return TreeVerdict::Skip(error.to_string()),
    };
    let claimed: PathBuf = match claim_into_trash(&item, trash, 0) {
        Ok(target) => target,
        Err(refusal) => return TreeVerdict::Skip(refusal.to_string()),
    };
    drop(held);
    return match delete_claimed(&claimed, item.identity) {
        Ok(deleted) => {
            emit(log, format!(
                "removed tree {} (device {} inode {}): {reason}; proof: {}{}",
                tree.path.display(),
                item.identity.device,
                item.identity.inode,
                proofs.join(" | "),
                modes_named(&deleted.modes_added)
            ));
            TreeVerdict::Removed { reason: reason.to_string(), bytes: deleted.bytes }
        }
        Err(Refusal::Gone(_)) => TreeVerdict::Skip(String::from("another owner removed the claimed entry")),
        Err(Refusal::PartlyRemoved { deleted, .. }) => {
            emit(log, format!(
                "partly removed tree {} (device {} inode {}): {reason}; {} disk bytes are gone and {} part(s) stay{}; each stayed because: {}; proof: {}",
                tree.path.display(),
                item.identity.device,
                item.identity.inode,
                deleted.bytes,
                deleted.remaining.len(),
                modes_named(&deleted.modes_added),
                deleted.remaining.join("; "),
                proofs.join(" | ")
            ));
            TreeVerdict::PartlyRemoved { reason: reason.to_string(), bytes: deleted.bytes, remaining: deleted.remaining.join("; ") }
        }
        Err(refusal) => TreeVerdict::Skip(refusal.to_string()),
    };
}

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
    let mut roots: Vec<(PathBuf, PathBuf)> = ledger_roots(earlier);
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
        emit(log, format!("read the recorded workspace of {} trees in {} s", trees.len() + earlier.len(), reading_began.elapsed().as_secs()));
    }
    let trash: PathBuf = base.join(TRASH_DIRECTORY_NAME);
    if !dry_run {
        let sweep = empty_trash(&trash);
        if !sweep.modes_added.is_empty() {
            emit(log, format!("finished an earlier run's deletion in {} ({} disk bytes){};", trash.display(), sweep.bytes, modes_named(&sweep.modes_added)));
        }
        for failure in sweep.failures {
            emit(log, format!("could not finish an earlier run's deletion in {}: {failure}", trash.display()));
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
        let (reason, needed): (String, i64) = match qualifies(tree, &workspaces[index], retired, policy, now, activity) {
            Ok(found) => found,
            Err(why) => {
                collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Keep(why) });
                continue;
            }
        };
        if let Some(inside) = holds_other_workspace(&tree.path, &roots) {
            collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Keep(format!("the workspace of {} lies inside it", inside.display())) });
            continue;
        }
        // The record this tree holds may be what keeps another tree from being removed, so it stays while it does.
        if let Some(guarded) = records_a_workspace_inside(&workspaces[index], &[trees, earlier], &tree.path) {
            collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Keep(format!("the workspace it records lies inside {}, whose guard that record is", guarded.display())) });
            continue;
        }
        if dry_run {
            let bytes: u64 = tree_bytes(&tree.path, deadline).bytes;
            emit(log, format!("would remove tree {} ({bytes} disk bytes): {reason}", tree.path.display()));
            collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Removed { reason, bytes } });
            continue;
        }
        let verdict: TreeVerdict = collect_one(tree, &reason, &trash, environment, now, needed, deadline, log);
        collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict });
    }
    let _ = std::fs::remove_dir(&trash);
    return collection;
}
