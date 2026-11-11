//! What:
//! The idle-trees mechanism of `cargo-shadow lean run`:
//! what a pass over the host build base's trees found,
//! and the summary it reports.
//! Why:
//! One pass over thirty thousand trees is the longest mechanism of the run,
//! so what it counts and where it stopped are written and tested here,
//! apart from the run's order and its other steps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function treeReport(trees: Tree[], start: number, options: RunOptions, deadline: Deadline, env: GuardEnvironment, base: string, pinned: Set<string>): MechanismReport
//! ```

/// Sets and paths.
use std::{collections::HashSet, path::{Path, PathBuf}};

/// The trees of the host build base.
use crate::lean_build_trees::{Tree, list_trees};
/// The guard's environment.
use crate::lean_guard::GuardEnvironment;
/// The run's deadline and the clock the idle periods are measured against.
use crate::lean_identity::{Deadline, now_seconds};
/// The tree collection and its verdicts.
use crate::lean_idle_trees::{IdlePolicy, TreeVerdict, collect_idle_trees};
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
/// and a summary that counted it as either would say something untrue about it.
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
    /// Trees kept.
    pub kept: usize,
    /// Trees skipped for a reason other than the deadline.
    pub skipped: usize,
}

/// What:
/// `pub fn line(&self, verb: &str) -> String` is the idle-trees summary line.
/// Why:
/// It says how far the pass got out of what it was given,
/// and it counts a partly removed tree as partly removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (tally: TreeTally, verb: string): string => `idle trees: ${tally.judged} of ${tally.given} trees judged; ${verb} ${tally.removed} (${gib(tally.bytes)} by disk blocks); partly removed ${tally.partly} (${gib(tally.partlyBytes)} by disk blocks freed, what stayed is named); kept ${tally.kept}; skipped ${tally.skipped}`;
/// ```
impl TreeTally {
    /// The summary line,
    /// with the verb the run earns (`removed` or `would remove`).
    pub fn line(&self, verb: &str) -> String {
        return format!(
            "idle trees: {} of {} trees judged; {verb} {} ({} by disk blocks); partly removed {} ({} by disk blocks freed, what stayed is named); kept {}; skipped {}",
            self.judged,
            self.given,
            self.removed,
            gib(self.bytes),
            self.partly,
            gib(self.partly_bytes),
            self.kept,
            self.skipped
        );
    }
}

/// What:
/// `pub(crate) fn trees_in_order(base: &Path) -> Vec<Tree>` lists the base's trees in one order,
/// so a position in that order names the same place on the next run.
/// Why:
/// The record of where a pass stopped is a path,
/// and it is only a position if the list it indexes is ordered the same way every time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const treesInOrder = (base: string): Tree[] => listTrees(base).sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
/// ```
pub(crate) fn trees_in_order(base: &Path) -> Vec<Tree> {
    let mut trees: Vec<Tree> = list_trees(base);
    trees.sort_by(by_path);
    return trees;
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
/// `pub(crate) fn tree_report(trees: &[Tree], start: usize, options: &RunOptions, deadline: &Deadline, environment: &GuardEnvironment, base: &Path, pinned: &HashSet<String>) -> MechanismReport`
/// runs mechanism C from the recorded position and returns its summary lines and where it stopped.
/// Why:
/// It reports removed trees by count and size,
/// a tree that was only partly removed as such,
/// the trees the deadline left for the next run,
/// and every kept or skipped tree by its reason.
/// The trees before the recorded position go in as guards rather than as work:
/// they are not judged again, and the trees this pass does judge cannot be removed while one of them records a workspace
/// inside them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeReport(trees: Tree[], start: number, options: RunOptions, deadline: Deadline, env: GuardEnvironment, base: string, pinned: Set<string>): MechanismReport
/// ```
pub(crate) fn tree_report(trees: &[Tree], start: usize, options: &RunOptions, deadline: &Deadline, environment: &GuardEnvironment, base: &Path, pinned: &HashSet<String>) -> MechanismReport {
    let retired: HashSet<String> = retired_toolchain_names(trees, pinned);
    let policy: IdlePolicy = IdlePolicy { idle_seconds: TREE_IDLE_SECONDS, unidentified_idle_seconds: UNIDENTIFIED_TREE_IDLE_SECONDS };
    let earlier: &[Tree] = &trees[..start];
    let chosen: &[Tree] = &trees[start..];
    let mut log: Vec<String> = Vec::new();
    let collection = collect_idle_trees(chosen, earlier, base, &retired, &policy, environment, now_seconds(), options.dry_run, deadline, &mut log);
    let mut tally: TreeTally = TreeTally { judged: collection.judged, given: chosen.len(), ..TreeTally::default() };
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
            TreeVerdict::Keep(_) => tally.kept += 1,
            TreeVerdict::Skip(why) => {
                tally.skipped += 1;
                skips.push(format!("  skipped {}: {why}", outcome.tree.display()));
            }
        }
    }
    let verb: &str = if options.dry_run { "would remove" } else { "removed" };
    let mut lines: Vec<String> = vec![tally.line(verb)];
    if start > 0 {
        lines.push(format!("  this pass started after {}, the tree the last run stopped at", trees[start - 1].path.display()));
    }

    // A pass that judged nothing keeps the position it started at, so a run cut before its first tree loses no ground.
    let judged_last: Option<&Tree> = if collection.judged > 0 { Some(&chosen[collection.judged - 1]) } else { None };
    let unfinished: bool = collection.judged < chosen.len();
    let resume_after: Option<PathBuf> = match (unfinished, judged_last) {
        (true, Some(last)) => Some(last.path.clone()),
        (true, None) if start > 0 => Some(trees[start - 1].path.clone()),
        _ => None,
    };
    if unfinished && let Some(last) = judged_last {
        lines.push(format!("  the deadline passed after {} of {} trees, at {}; the next run continues after it", collection.judged, chosen.len(), last.path.display()));
    }
    lines.extend(leftovers.into_iter().take(25));
    lines.extend(skips.into_iter().take(25));
    return MechanismReport { lines, resume_after };
}
