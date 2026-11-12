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
//! the ledger of every tree is read first (one small read each, and the whole guard list comes from it),
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
/// The ledger that names the workspace a tree was built for.
use crate::lean_keyed_folder::read_ledger;
/// The proof and the removal.
use crate::lean_guard::{Guard, GuardEnvironment, Guarded, guard_profile};
/// Imports `missing` from `crate::lean_identity`:
/// an entry is gone only when its absence is confirmed.
use crate::lean_identity::{Deadline, missing};
/// Imports the names this file uses from `crate::lean_proc_scan`.
use crate::lean_proc_scan::scan_processes;
/// Imports the names this file uses from `crate::lean_removal`.
use crate::lean_removal::{Refusal, TRASH_DIRECTORY_NAME, claim_into_trash, delete_claimed, empty_trash, item_of, modes_named};
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

/// What the ledger of every tree says,
/// read once before anything is judged.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Survey {
    /// One class per tree of the list,
    /// in the order it was given:
    /// true when the ledger alone shows the tree may qualify (a retired toolchain, or a workspace confirmed absent).
    pub candidates: Vec<bool>,
    /// Every ledgered workspace of the base,
    /// as pairs of the tree and the workspace it names.
    /// This is the guard list:
    /// a tree that holds one of these workspaces is never removed.
    pub roots: Vec<(PathBuf, PathBuf)>,
    /// How many trees the survey read.
    pub read: usize,
    /// Whether the deadline stopped the survey before the last tree,
    /// in which case the guard list is incomplete and nothing may be removed.
    pub cut: bool,
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
/// `pub fn survey(trees: &[Tree], retired: &HashSet<String>, deadline: &Deadline) -> Survey` reads the ledger of every tree once.
/// Why:
/// Two things need it before a single tree is judged:
/// the guard against removing a tree that holds another tree's workspace must cover the whole base,
/// including the trees this pass does not reach,
/// and the order the pass judges in puts the trees whose ledger already shows they qualify first,
/// so the space a pass frees comes before the pass has read every tree.
/// Only the ledger is read here,
/// never the dependency information,
/// so a survey costs one small file read and one `stat` per tree,
/// and a tree whose ledger names a workspace that is gone is a candidate without a walk.
/// Reading stops at the deadline:
/// a guard list that is short of one tree may not be used to remove anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function survey(trees: Tree[], retired: Set<string>, deadline: Deadline): Survey
/// ```
pub fn survey(trees: &[Tree], retired: &HashSet<String>, deadline: &Deadline) -> Survey {
    let mut found: Survey = Survey { candidates: vec![false; trees.len()], ..Survey::default() };
    for (index, tree) in trees.iter().enumerate() {
        if deadline.passed() {
            found.cut = true;
            break;
        }
        let toolchain_retired: bool = match &tree.kind {
            TreeKind::Keyed(name) => retired.contains(name),
            TreeKind::Legacy => false,
        };
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
/// `fn judging_order(trees: &[Tree], candidates: &[bool]) -> Vec<usize>` is the indexes of the trees in the order the pass judges
/// them:
/// the candidates first,
/// and each group in path order.
/// Why:
/// A pass that is cut by its deadline judges only the front of this order,
/// so putting the trees that already qualify there is what makes the space come out of the pass while it runs instead of
/// at the end of a survey of every tree.
/// The order is a function of the trees and their ledgers alone,
/// so the position the last pass recorded names the same place on the next one as long as those ledgers say the same thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judgingOrder = (trees: Tree[], candidates: boolean[]): number[] => { const at = trees.map((_, index) => index).sort((a, b) => trees[a].path < trees[b].path ? -1 : trees[a].path > trees[b].path ? 1 : 0); return [...at.filter((index) => candidates[index]), ...at.filter((index) => !candidates[index])]; };
/// ```
fn judging_order(trees: &[Tree], candidates: &[bool]) -> Vec<usize> {
    let mut by_path: Vec<usize> = (0..trees.len()).collect();
    by_path.sort_by(|left, right| { return trees[*left].path.cmp(&trees[*right].path); });
    let mut order: Vec<usize> = Vec::with_capacity(trees.len());
    for index in &by_path {
        if candidates[*index] {
            order.push(*index);
        }
    }
    for index in &by_path {
        if !candidates[*index] {
            order.push(*index);
        }
    }
    return order;
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

/// What:
/// `fn collect_one(tree, reason, trash, environment, now, needed, deadline, log) -> TreeVerdict`
/// removes one qualified tree under the protocol.
/// Why:
/// Unlinking what a directory holds needs write and search on it,
/// so a directory this user owns and may not write is made writable first and the removal's proof line says so;
/// a directory of another user stays as it is and is named.
/// A tree that could not be emptied whole is reported as partly removed,
/// never as skipped,
/// because part of it is gone,
/// and a deletion the deadline stopped is reported as left in the trash,
/// because that is what a person has to know to trust the numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collectOne(tree: Tree, reason: string, trash: string, env: GuardEnvironment, now: number, needed: number, deadline: Deadline, log: string[]): TreeVerdict
/// ```
fn collect_one(tree: &Tree, reason: &str, trash: &Path, environment: &GuardEnvironment, now: i64, needed: i64, deadline: &Deadline, log: &mut Vec<String>) -> TreeVerdict {
    let Some(profiles) = crate::lean_build_trees::profile_dirs_of(&tree.path, deadline) else {
        return TreeVerdict::Skip(String::from("the deadline passed while looking for its profile directories"));
    };
    let mut held: Vec<Guarded> = Vec::new();
    for profile in profiles {
        match guard_profile(&profile, environment, deadline) {
            Guard::Ready(ready) => held.push(ready),
            Guard::Skip(why) => return TreeVerdict::Skip(format!("{}: {why}", profile.display())),
        }
    }
    match last_activity(&tree.path, deadline) {
        Some(activity) if now - activity > needed => {}
        Some(_) => return TreeVerdict::Skip(String::from("a write happened while the locks were being taken")),
        None => return TreeVerdict::Skip(String::from("the deadline passed while reading the tree")),
    }
    let Some(scan) = scan_processes(&environment.proc_root, &[tree.path.as_path()], &[], environment.own_pid, deadline) else {
        return TreeVerdict::Skip(String::from("the deadline passed while scanning the processes, so nothing was removed"));
    };
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
    return match delete_claimed(&claimed, item.identity, deadline) {
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
        Err(Refusal::PartlyRemoved { deleted, .. }) if deleted.cut => {
            emit(log, format!(
                "left tree {} in {} (device {} inode {}): {reason}; {} disk bytes are gone and the deadline stopped the rest, which the next pass finishes; proof: {}",
                tree.path.display(),
                claimed.display(),
                item.identity.device,
                item.identity.inode,
                deleted.bytes,
                proofs.join(" | ")
            ));
            TreeVerdict::Left { reason: reason.to_string(), bytes: deleted.bytes }
        }
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
    let found: Survey = survey(trees, retired, deadline);
    collection.lines.push(format!("  read the ledger of {} of {} trees in {} s", found.read, trees.len(), surveying.elapsed().as_secs()));
    let candidates: usize = found.candidates.iter().filter(|one| { return **one; }).count();
    collection.candidates = candidates;
    if found.cut {
        collection.lines.push(format!("  the deadline passed while reading the ledger of {} of {} trees, so this pass judged nothing: a removal needs the whole base's records as its guard", found.read, trees.len()));
        return collection;
    }
    // The order is judged from the recorded position: the candidates after it first, then the rest after it.
    // Comparing paths inside each group keeps the position a single key a person can read,
    // and the pass that reaches the end of both groups leaves no position, so the next one starts at the front.
    let order: Vec<usize> = judging_order(trees, &found.candidates);
    let mut to_judge: Vec<usize> = Vec::with_capacity(order.len());
    for index in order {
        if after.is_none_or(|recorded| trees[index].path.as_path() > recorded) {
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
    for index in &to_judge {
        if deadline.passed() {
            break;
        }
        progress(&mut standing, deadline);
        let tree: &Tree = &trees[*index];
        let Some(workspace) = workspace_of(&tree.path, deadline) else {
            // Its workspace was not read, so nothing was decided about it: the next pass judges it first.
            break;
        };
        let (reason, needed): (String, i64) = match qualifies_apart_from_idleness(tree, &workspace, retired, policy) {
            Ok(found) => found,
            Err((why, kind)) => {
                standing.judged += 1;
                standing.kept += 1;
                collection.judged += 1;
                collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Keep { reason: why, kind } });
                continue;
            }
        };
        if let Some(inside) = holds_other_workspace(&tree.path, &found.roots) {
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
            let reason: String = format!("the workspace it records lies inside {}, whose guard that record is", guarded.display());
            collection.outcomes.push(TreeOutcome { tree: tree.path.clone(), verdict: TreeVerdict::Keep { reason, kind: KeepKind::InUse } });
            continue;
        }
        let Some(activity) = last_activity(&tree.path, deadline) else {
            break;
        };
        let reason: String = match qualifies(now, activity, needed, &reason) {
            Ok(found) => found,
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
    collection.resume_after = if collection.judged > 0 && collection.judged < collection.given {
        Some(collection.outcomes[collection.judged - 1].tree.clone())
    } else if collection.judged == 0 && collection.given > 0 {
        after.map(Path::to_path_buf)
    } else {
        None
    };
    if collection.judged == 0 && collection.given > 0 {
        collection.lines.push(format!("  no tree was judged; the position is kept, so the next pass starts where this one did"));
    }
    return collection;
}
