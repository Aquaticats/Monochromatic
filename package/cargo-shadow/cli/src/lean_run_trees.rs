//! What:
//! The idle-trees mechanism of `cargo-shadow lean run`:
//! what a pass over the host build base's trees found,
//! and the summary it reports.
//! Why:
//! One pass over thirty thousand trees is the longest mechanism of the run,
//! so what it counts and where it stopped are written and tested here,
//! apart from the run's order and its other steps.
//! The order is part of the mechanism:
//! the pass lists the base,
//! reads the ledger of every tree once,
//! judges the trees whose ledger already shows they qualify before the rest,
//! and continues after the tree the last pass stopped at.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function treeReport(base: string, after: string | undefined, options: RunOptions, deadline: Deadline, env: GuardEnvironment, pinned: Set<string>): MechanismReport
//! ```

/// Sets and paths.
use std::{collections::HashSet, path::Path};

/// The trees of the host build base.
use crate::lean_build_trees::{Tree, list_trees};
/// The guard's environment.
use crate::lean_guard::GuardEnvironment;
/// The run's deadline and the clock the idle periods are measured against.
use crate::lean_identity::{Deadline, now_seconds};
/// The tree collection and its verdicts.
use crate::lean_idle_trees::{IdlePolicy, KeepKind, TreeVerdict, collect_idle_trees};
/// The run's options and the report of one mechanism.
use crate::lean_run::{MechanismReport, RunOptions};
/// The figures the summary line shows.
use crate::lean_run_report::gib;
/// The idle periods of the trees.
use crate::lean_policy::{TREE_IDLE_SECONDS, UNIDENTIFIED_TREE_IDLE_SECONDS};
/// The retirement of the dated nightlies a tree's folder names.
use crate::lean_run::retired_toolchain_names;

/// What:
/// `pub struct TreeTally` is what a pass over the trees found,
/// by count,
/// so the summary line is written and tested in one place.
/// Why:
/// A tree that was only partly removed is neither removed nor skipped,
/// and a summary that counted it as either would say something untrue about it;
/// the same holds for a tree whose deletion the deadline stopped with the rest of it claimed into the trash.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TreeTally {
    /// Trees the pass judged.
    pub judged: usize,
    /// Trees it was given.
    pub given: usize,
    /// Trees removed whole.
    pub removed: usize,
    /// Disk bytes they freed.
    pub bytes: u64,
    /// Trees of which part stayed.
    pub partly: usize,
    /// Disk bytes their partial removals freed.
    pub partly_bytes: u64,
    /// Trees whose deletion the deadline stopped,
    /// with the rest of each in the trash.
    pub left: usize,
    /// Disk bytes those deletions freed before the deadline.
    pub left_bytes: u64,
    /// Trees kept.
    pub kept: usize,
    /// Of the kept trees,
    /// the ones kept because something that uses them is still there.
    pub kept_in_use: usize,
    /// Of the kept trees,
    /// the ones kept because they have not been left alone long enough.
    pub kept_not_idle: usize,
    /// Trees skipped for a reason other than the deadline.
    pub skipped: usize,
    /// Disk bytes an earlier pass's claimed entries gave back.
    pub finished_bytes: u64,
}

/// What:
/// `pub fn line(&self, verb: &str) -> String` is the idle-trees summary line.
/// Why:
/// It says how far the pass got out of what it was given,
/// and it counts a partly removed tree as partly removed and a deletion the deadline stopped as left for the next pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (tally: TreeTally, verb: string): string => `idle trees: ${tally.judged} of ${tally.given} trees judged; ${verb} ${tally.removed} (${gib(tally.bytes)} by disk blocks); partly removed ${tally.partly} (${gib(tally.partlyBytes)} by disk blocks freed, what stayed is named); left for the next pass ${tally.left} (${gib(tally.leftBytes)} by disk blocks freed so far); kept ${tally.kept}; skipped ${tally.skipped}`;
/// ```
impl TreeTally {
    /// The summary line,
    /// with the verb the run earns (`removed` or `would remove`).
    pub fn line(&self, verb: &str) -> String {
        return format!(
            "idle trees: {} of {} trees judged; {verb} {} ({} by disk blocks); partly removed {} ({} by disk blocks freed, what stayed is named); left for the next pass {} ({} by disk blocks freed so far); kept {}; skipped {}",
            self.judged,
            self.given,
            self.removed,
            gib(self.bytes),
            self.partly,
            gib(self.partly_bytes),
            self.left,
            gib(self.left_bytes),
            self.kept,
            self.skipped
        );
    }
}

/// What:
/// `pub(crate) fn trees_in_order(base: &Path, deadline: &Deadline) -> Option<Vec<Tree>>` lists the base's trees in one order,
/// so a position in that order names the same place on the next run,
/// and answers nothing when the deadline passed while listing.
/// Why:
/// The record of where a pass stopped is a path,
/// and it is only a position if the list it indexes is ordered the same way every time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const treesInOrder = (base: string, deadline: Deadline): Tree[] | undefined => listTrees(base, deadline)?.sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
/// ```
pub(crate) fn trees_in_order(base: &Path, deadline: &Deadline) -> Option<Vec<Tree>> {
    let mut trees: Vec<Tree> = list_trees(base, deadline)?;
    trees.sort_by(by_path);
    return Some(trees);
}

/// What:
/// `pub(crate) fn by_path(left: &Tree, right: &Tree) -> std::cmp::Ordering` is the order a recorded position indexes.
/// Why:
/// Two runs must walk the trees in the same order for a key to name a place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byPath = (left: Tree, right: Tree): number => left.path < right.path ? -1 : left.path > right.path ? 1 : 0;
/// ```
fn by_path(left: &Tree, right: &Tree) -> std::cmp::Ordering {
    return left.path.cmp(&right.path);
}

/// What:
/// `pub(crate) fn tree_report(base: &Path, after: Option<&Path>, options: &RunOptions, deadline: &Deadline, environment: &GuardEnvironment, pinned: &HashSet<String>) -> MechanismReport`
/// runs mechanism C from the recorded position and returns its summary lines and where it stopped.
/// Why:
/// It reports removed trees by count and size,
/// a tree that was only partly removed as such,
/// a deletion the deadline stopped as left in the trash,
/// the trees the deadline left for the next run,
/// every kept tree by the kind of reason it was kept for,
/// and every skipped tree by its reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeReport(base: string, after: string | undefined, options: RunOptions, deadline: Deadline, env: GuardEnvironment, pinned: Set<string>): MechanismReport
/// ```
pub(crate) fn tree_report(base: &Path, after: Option<&Path>, options: &RunOptions, deadline: &Deadline, environment: &GuardEnvironment, pinned: &HashSet<String>) -> MechanismReport {
    let Some(trees) = trees_in_order(base, deadline) else {
        return MechanismReport { lines: vec![String::from("idle trees: the deadline passed while listing the base, so nothing was judged")], resume_after: None };
    };
    let retired: HashSet<String> = retired_toolchain_names(&trees, pinned);
    let policy: IdlePolicy = IdlePolicy { idle_seconds: TREE_IDLE_SECONDS, unidentified_idle_seconds: UNIDENTIFIED_TREE_IDLE_SECONDS };
    let mut log: Vec<String> = Vec::new();
    let collection = collect_idle_trees(&trees, after, base, &retired, &policy, environment, now_seconds(), options.dry_run, deadline, &mut log);
    let mut tally: TreeTally = TreeTally { judged: collection.judged, given: collection.given, finished_bytes: collection.finished_bytes, ..TreeTally::default() };
    let mut skips: Vec<String> = Vec::new();
    let mut leftovers: Vec<String> = Vec::new();
    for outcome in &collection.outcomes {
        match &outcome.verdict {
            TreeVerdict::Removed { bytes: freed, .. } => {
                tally.removed += 1;
                tally.bytes += freed;
            }
            TreeVerdict::PartlyRemoved { bytes: freed, remaining, .. } => {
                tally.partly += 1;
                tally.partly_bytes += freed;
                leftovers.push(format!("  partly removed {}: {remaining}", outcome.tree.display()));
            }
            TreeVerdict::Left { bytes: freed, .. } => {
                tally.left += 1;
                tally.left_bytes += freed;
                leftovers.push(format!("  left {} in the trash with the rest of its deletion; the next pass finishes it", outcome.tree.display()));
            }
            TreeVerdict::Keep { kind, .. } => {
                tally.kept += 1;
                match kind {
                    KeepKind::InUse => tally.kept_in_use += 1,
                    KeepKind::NotIdleEnough => tally.kept_not_idle += 1,
                }
            }
            TreeVerdict::Skip(why) => {
                tally.skipped += 1;
                skips.push(format!("  skipped {}: {why}", outcome.tree.display()));
            }
        }
    }
    let verb: &str = if options.dry_run { "would remove" } else { "removed" };
    let mut lines: Vec<String> = vec![tally.line(verb)];
    lines.extend(collection.lines.iter().cloned());
    if tally.finished_bytes > 0 {
        lines.push(format!("  finished an earlier pass's claimed deletions: {} by disk blocks", gib(tally.finished_bytes)));
    }
    if tally.judged > 0 && tally.removed == 0 && tally.left == 0 && tally.partly == 0 {
        lines.push(format!(
            "  nothing was {}: {} trees were kept because something that uses them is still there and {} because they have not been left alone long enough",
            if options.dry_run { "found to remove" } else { "removed" },
            tally.kept_in_use,
            tally.kept_not_idle
        ));
    }
    lines.extend(leftovers.into_iter().take(25));
    lines.extend(skips.into_iter().take(25));
    let unfinished: bool = collection.judged < collection.given;
    if unfinished {
        let at: String = match &collection.resume_after {
            Some(last) => format!(", at {}", last.display()),
            None => String::new(),
        };
        lines.push(format!("  the deadline passed after {} of {} trees{at}; the next run continues after it", collection.judged, collection.given));
    }
    return MechanismReport { lines, resume_after: collection.resume_after };
}
