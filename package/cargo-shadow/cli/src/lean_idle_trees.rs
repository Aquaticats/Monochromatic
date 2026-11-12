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
//! A pass over thirty thousand trees is longer than the bound it runs under,
//! so the order and the cost are part of the mechanism:
//! the ledger of every tree is read first (one small read each,
//! and the whole guard list comes from it),
//! the trees whose ledger already shows they qualify are judged before the rest,
//! a tree whose workspace is still there is kept without walking it for its last write,
//! and every loop checks the clock between two items,
//! so a pass that reaches its bound stops at the next safe point and says where.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function collectIdleTrees(trees: Tree[], after: string | undefined, base: string, retired: Set<string>, policy: IdlePolicy, env: GuardEnvironment, now: number, dryRun: boolean, deadline: Deadline, log: string[]): TreeCollection
//! ```

/// The streaming log line.
use crate::lean_run_report::{emit, gib, log_line};
/// Paths and sets,
/// and the clock the progress lines keep.
use std::{collections::HashSet, path::{Path, PathBuf}, time::Instant};

/// The tree readers.
use crate::lean_build_trees::{Tree, TreeKind, Workspace, last_activity, workspace_of};
/// The proof of "in use" the pass takes before it judges.
use crate::lean_guard::GuardEnvironment;
/// Imports `missing` from `crate::lean_identity`:
/// an entry is gone only when its absence is confirmed.
use crate::lean_identity::{Deadline, missing};
/// Imports the names this file uses from `crate::lean_removal`.
use crate::lean_removal::{TRASH_DIRECTORY_NAME, empty_trash, modes_named};
/// The survey that reads the base and the order it gives.
use crate::lean_tree_survey::{Survey, judging_order, survey};
/// The removal of one qualified tree.
use crate::lean_tree_removal::collect_one;
/// Imports the names this file uses from `crate::lean_sizes`.
use crate::lean_sizes::tree_bytes;

/// How often the judging loop says where it is,
/// in seconds.
/// Why:
/// A pass over the whole base is the longest thing the tool does,
/// and a journal that shows nothing for an hour tells a person neither what it is judging nor whether it is moving.
/// Thirty seconds is short enough to answer "is it working" while a removal is in flight and long enough that the lines
/// stay readable beside one line per removal.
const PROGRESS_SECONDS: u64 = 30;

/// The idle periods of mechanism C.
#[derive(Clone, Copy, Debug)]
pub struct IdlePolicy {
    /// A tree whose workspace is gone (or whose toolchain is retired) is removed after this many seconds without a write.
    pub idle_seconds: i64,
    /// A tree nothing identifies is removed after this many seconds without a write.
    pub unidentified_idle_seconds: i64,
}

/// Why a tree was kept,
/// as the summary counts it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeepKind {
    /// Something that uses it is still there:
    /// its workspace,
    /// the source a lead names,
    /// or another tree's workspace inside it.
    InUse,
    /// It has not been left alone long enough for its case.
    NotIdleEnough,
}

/// What the collection did with one tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreeVerdict {
    /// Kept,
    /// for this reason and this kind of reason.
    Keep {
        /// Why the tree is kept.
        reason: String,
        /// Whether the keep is about something using it or about its age.
        kind: KeepKind,
    },
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
    /// It was claimed and the deadline stopped its deletion:
    /// what is left of it is in the trash and the next pass finishes it.
    Left {
        /// The reason it qualified.
        reason: String,
        /// Disk bytes its removal freed before the deadline.
        bytes: u64,
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
    /// in the order the pass judged them.
    pub outcomes: Vec<TreeOutcome>,
    /// How many of the trees after the recorded position the pass judged before the deadline stopped it.
    pub judged: usize,
    /// How many trees this pass was to judge:
    /// the list from the recorded position.
    pub given: usize,
    /// The last tree the pass judged,
    /// when it did not finish the list;
    /// the next pass continues after it.
    pub resume_after: Option<PathBuf>,
    /// The lines a person reads about how this pass went:
    /// what the survey cost,
    /// what the deadline stopped,
    /// and what an earlier pass's trash gave back.
    pub lines: Vec<String>,
    /// How many trees qualified from their ledger alone,
    /// which is how many the pass reaches first.
    pub candidates: usize,
    /// Disk bytes an earlier pass's claimed entries gave back.
    pub finished_bytes: u64,
}

/// What:
/// `fn qualifies_apart_from_idleness(tree: &Tree, workspace: &Workspace, retired: &HashSet<String>, policy: &IdlePolicy) -> Result<(String, i64), (String, KeepKind)>`
/// decides from the workspace and the toolchain alone,
/// before anything about the tree's last write is read:
/// `Ok((reason, idle period that applies))` when only idleness is left to check,
/// `Err((why kept, kind))` when the tree is kept whatever its age.
/// Why:
/// The answer "its workspace is still there" is a `stat` of one path,
/// and the answer "it has written nothing for seven days" is a walk over every unit of the tree:
/// most trees of this host are kept because their workspace is alive,
/// so asking the cheap question first is what makes a pass over them short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function qualifiesApartFromIdleness(tree: Tree, workspace: Workspace, retired: Set<string>, policy: IdlePolicy): { ok: [string, number] } | { no: [string, KeepKind] }
/// ```
fn qualifies_apart_from_idleness(tree: &Tree, workspace: &Workspace, retired: &HashSet<String>, policy: &IdlePolicy) -> Result<(String, i64), (String, KeepKind)> {
    let toolchain_retired: bool = match &tree.kind {
        TreeKind::Keyed(name) => retired.contains(name),
        TreeKind::Legacy => false,
    };
    return match workspace {
        _ if toolchain_retired => Ok((String::from("its toolchain is retired"), policy.idle_seconds)),
        Workspace::Known(root) if !missing(root) => Err((format!("its workspace {} exists or cannot be shown to be gone", root.display()), KeepKind::InUse)),
        Workspace::Known(root) => Ok((format!("its workspace {} is gone", root.display()), policy.idle_seconds)),
        Workspace::Lead(source) if !missing(source) => Err((format!("the source {} that its dependency information names exists or cannot be shown to be gone", source.display()), KeepKind::InUse)),
        Workspace::Lead(source) => Ok((format!("the source {} that its dependency information names is gone (a lead, not proof)", source.display()), policy.idle_seconds)),
        Workspace::Unidentified => Ok((String::from("nothing identifies its workspace"), policy.unidentified_idle_seconds)),
    };
}

/// What:
/// `fn qualifies(activity: i64, idle: i64) -> Result<String, (String, KeepKind)>` says whether a tree left alone for `idle` seconds
/// has been idle long enough for `needed`.
/// Why:
/// The comparison and the two sentences it can answer sit in one place,
/// so the keep and the removal read alike.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const qualifies = (activity: number, now: number, needed: number, why: string): string | [string, KeepKind]
/// ```
fn qualifies(now: i64, activity: i64, needed: i64, why: &str) -> Result<String, (String, KeepKind)> {
    let idle: i64 = now - activity;
    if idle <= needed {
        return Err((format!("idle for {idle} s, within the {needed} s period ({why})"), KeepKind::NotIdleEnough));
    }
    return Ok(format!("{why}; idle for {idle} s, longer than {needed} s"));
}

/// What:
/// `fn records_a_workspace_inside(workspace: &Workspace, trees: &[Tree], own: &Path) -> Option<PathBuf>` is the tree
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
/// const recordsWorkspaceInside = (workspace: Workspace, trees: Tree[], own: string): string | undefined
/// ```
fn records_a_workspace_inside(workspace: &Workspace, trees: &[Tree], own: &Path) -> Option<PathBuf> {
    let Workspace::Known(root) = workspace else { return None };
    // A workspace whose absence is confirmed holds nothing, so its record guards nothing.
    // An answer other than absence (a directory this user may not search) may hide the workspace, so the record stays a guard.
    if missing(root) {
        return None;
    }
    for tree in trees {
        if tree.path != own && root.starts_with(&tree.path) {
            return Some(tree.path.clone());
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

/// What a pass is doing,
/// for the lines it says while it runs.
struct Standing {
    /// When the judging began,
    /// so the line can say how much of the bound is left.
    began: Instant,
    /// The last time a progress line was said.
    last_said: Instant,
    /// Trees judged so far.
    judged: usize,
    /// Trees this pass will judge.
    given: usize,
    /// Trees removed so far.
    removed: usize,
    /// Disk bytes freed so far.
    bytes: u64,
    /// Trees kept so far.
    kept: usize,
    /// Trees skipped so far.
    skipped: usize,
}

/// What:
/// `fn progress(standing: &mut Standing, deadline: &Deadline)` says where the pass is when enough time has passed for another line.
/// Why:
/// The journal is the only place a person can see a pass of an hour working,
/// and a line every half minute is cheap beside the removal it sits next to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function progress(standing: Standing, deadline: Deadline): void
/// ```
fn progress(standing: &mut Standing, deadline: &Deadline) {
    if standing.last_said.elapsed().as_secs() < PROGRESS_SECONDS {
        return;
    }
    standing.last_said = Instant::now();
    log_line(&format!(
        "idle trees: judged {} of {} in {} s; removed {} ({} by disk blocks); kept {}; skipped {}; the bound ends in {} s",
        standing.judged,
        standing.given,
        standing.began.elapsed().as_secs(),
        standing.removed,
        gib(standing.bytes),
        standing.kept,
        standing.skipped,
        deadline.remaining().as_secs()
    ));
}

/// What:
/// `pub fn collect_idle_trees(...) -> TreeCollection`
/// surveys the base,
/// judges the trees from the recorded position in the order the survey gives,
/// and removes the ones that qualify,
/// holding the workspaces every tree records as guards.
/// Why:
/// It is mechanism C.
/// A dry run reports what would be removed with its size and removes nothing.
/// The deadline stops the judging,
/// and the tree it stopped at is where the next pass continues.
/// A survey the deadline cut judges nothing at all,
/// because the guard list and the order both come from the whole list:
/// a pass that removed on a half-read base could remove a tree whose base holds a live workspace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function collectIdleTrees(trees: Tree[], after: string | undefined, base: string, retired: Set<string>, policy: IdlePolicy, env: GuardEnvironment, now: number, dryRun: boolean, deadline: Deadline, log: string[]): TreeCollection
/// ```
pub fn collect_idle_trees(
    trees: &[Tree],
    after: Option<&Path>,
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
    // The survey is a fixed cost of every pass, however small the bound is, so the pass says what it cost.
    let surveying: Instant = Instant::now();
    let surveyed: Survey = survey(trees, retired, deadline);
    collection.lines.push(format!("  read the ledger of {} of {} trees in {} s", surveyed.read, trees.len(), surveying.elapsed().as_secs()));
    let mut candidates: usize = 0;
    for candidate in &surveyed.candidates {
        if *candidate {
            candidates += 1;
        }
    }
    collection.candidates = candidates;
    if surveyed.cut {
        collection.lines.push(format!("  the deadline passed while reading the ledger of {} of {} trees, so this pass judged nothing: a removal needs the whole base's records as its guard", surveyed.read, trees.len()));
        collection.given = trees.len();
        collection.resume_after = after.map(Path::to_path_buf);
        return collection;
    }
    // The age order is not a position: a tree that was kept has been written to since, or was too young, so it moves
    // further down the order on the next pass. The recorded position bounds the window a pass judges, so a pass that is
    // cut always leaves ground behind it and the next one continues after it instead of reading the same front again.
    let order: Vec<usize> = judging_order(trees, &surveyed);
    let mut to_judge: Vec<usize> = Vec::with_capacity(order.len());
    for index in order {
        let in_window: bool = match after {
            Some(recorded) => trees[index].path.as_path() > recorded,
            None => true,
        };
        if in_window {
            to_judge.push(index);
        }
    }
    collection.given = to_judge.len();
    if let Some(recorded) = after {
        collection.lines.insert(0, format!("  this pass started after {}, the tree the last pass stopped at", recorded.display()));
    }
    if candidates > 0 {
        collection.lines.push(format!("  {candidates} trees are named by their ledger as already qualified: a retired toolchain, or a workspace that is gone"));
    }
    let trash: PathBuf = base.join(TRASH_DIRECTORY_NAME);
    if !dry_run {
        let sweep = empty_trash(&trash, deadline);
        collection.finished_bytes = sweep.bytes;
        if !sweep.modes_added.is_empty() {
            emit(log, format!("finished an earlier run's deletion in {} ({} disk bytes){};", trash.display(), sweep.bytes, modes_named(&sweep.modes_added)));
        } else if sweep.bytes > 0 {
            emit(log, format!("finished an earlier run's deletion in {} ({} disk bytes);", trash.display(), sweep.bytes));
        }
        for failure in &sweep.failures {
            emit(log, format!("could not finish an earlier run's deletion in {}: {failure}", trash.display()));
        }
        if sweep.cut {
            collection.lines.push(format!("  the deadline passed while finishing an earlier pass's deletion in {}; the next pass finishes it", trash.display()));
        }
    }
    let mut standing: Standing = Standing { began: Instant::now(), last_said: Instant::now(), judged: 0, given: collection.given, removed: 0, bytes: 0, kept: 0, skipped: 0 };
    // The last tree of this pass by path, which is where a cut pass continues.
    let mut last_of_the_rest: Option<PathBuf> = None;
    for index in &to_judge {
        if deadline.passed() {
            break;
        }
        progress(&mut standing, deadline);
        let tree: &Tree = &trees[*index];
        last_of_the_rest = match last_of_the_rest {
            Some(current) if current >= tree.path => Some(current),
            _ => Some(tree.path.clone()),
        };
        let Some(workspace) = workspace_of(&tree.path, deadline) else {
            // Its workspace was not read, so nothing was decided about it: the next pass judges it first.
            break;
        };
        let (why_qualified, needed): (String, i64) = match qualifies_apart_from_idleness(tree, &workspace, retired, policy) {
            Ok(answer) => answer,
            Err((why, kind)) => {
                standing.judged += 1;
                standing.kept += 1;
                collection.judged += 1;
                collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Keep { reason: why, kind } });
                continue;
            }
        };
        // The hint is the newest time among the tree's own directories, so it is a lower bound of its newest write:
        // a hint younger than the period proves the tree is not old enough, and the walk is not needed to know it.
        if let Some(hint) = surveyed.hints[*index]
            && now - hint <= needed
        {
            standing.judged += 1;
            standing.kept += 1;
            collection.judged += 1;
            let told: String = format!("the newest write among its own directories was {} s ago, within the {needed} s period ({why_qualified})", now - hint);
            collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Keep { reason: told, kind: KeepKind::NotIdleEnough } });
            continue;
        }
        if let Some(inside) = holds_other_workspace(&tree.path, &surveyed.roots) {
            standing.judged += 1;
            standing.kept += 1;
            collection.judged += 1;
            collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Keep { reason: format!("the workspace of {} lies inside it", inside.display()), kind: KeepKind::InUse } });
            continue;
        }
        // The record this tree holds may be what keeps another tree from being removed, so it stays while it does.
        if let Some(guarded) = records_a_workspace_inside(&workspace, trees, &tree.path) {
            standing.judged += 1;
            standing.kept += 1;
            collection.judged += 1;
            let told: String = format!("the workspace it records lies inside {}, whose guard that record is", guarded.display());
            collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Keep { reason: told, kind: KeepKind::InUse } });
            continue;
        }
        let Some(activity) = last_activity(&tree.path, deadline) else {
            break;
        };
        let reason: String = match qualifies(now, activity, needed, &why_qualified) {
            Ok(answer) => answer,
            Err((why, kind)) => {
                standing.judged += 1;
                standing.kept += 1;
                collection.judged += 1;
                collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Keep { reason: why, kind } });
                continue;
            }
        };
        if dry_run {
            let measured = tree_bytes(&tree.path, deadline);
            if !measured.complete {
                // The size walk stopped at the deadline: nothing was said about this tree, so the next pass judges it first.
                break;
            }
            standing.judged += 1;
            collection.judged += 1;
            emit(log, format!("would remove tree {} ({} disk bytes): {reason}", tree.path.display(), measured.bytes));
            standing.removed += 1;
            standing.bytes += measured.bytes;
            collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Removed { reason, bytes: measured.bytes } });
            continue;
        }
        standing.judged += 1;
        collection.judged += 1;
        let verdict: TreeVerdict = collect_one(tree, &reason, &trash, environment, now, needed, deadline, log);
        match &verdict {
            TreeVerdict::Removed { bytes, .. } => {
                standing.removed += 1;
                standing.bytes += bytes;
            }
            TreeVerdict::Left { bytes, .. } => {
                standing.removed += 1;
                standing.bytes += bytes;
            }
            TreeVerdict::Skip(_) => standing.skipped += 1,
            TreeVerdict::Keep { .. } | TreeVerdict::PartlyRemoved { .. } => {}
        }
        collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict });
    }
    let _ = std::fs::remove_dir(&trash);
    // A pass that judged nothing keeps the position it started at, unless the list held nothing after it.
    collection.resume_after = if collection.judged < collection.given {
        match last_of_the_rest {
            Some(last) => Some(last),
            None => after.map(Path::to_path_buf),
        }
    } else {
        None
    };
    if collection.judged == 0 && collection.given > 0 {
        collection.lines.push(format!("  no tree was judged; the position is kept, so the next pass starts where this one did"));
    }
    return collection;
}
