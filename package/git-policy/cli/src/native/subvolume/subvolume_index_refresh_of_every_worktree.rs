//! What:
//!  The rewrite of the index of the checkout and of every linked worktree inside it, side by side within the host's budget.
//! Why:
//!  Each index is its own file under its own lock describing its own tree, so the rewrites are independent: they run on the
//!  ordered runner (`parallel_runner.rs`) and are reported in the order the checkout's index first, then each linked
//!  worktree's by path. The rewrite of one file is `refresh_index_file_with` in `subvolume_index_files.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const count = await refreshAllIndexes(root, lock, references, say);
//! ```

use crate::line_sink::LineSink;
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::parallel_runner::{Event, OrderedEvents, OrderedRun, OrderedWork, run_ordered};
use crate::subvolume_index_files::{IndexLock, RefreshOptions, linked_worktrees_inside, refresh_index_file_with, repository_hash_length};
use std::path::{Path, PathBuf};

/// The number of copies of an index file one rewrite holds at once: the bytes read, the parsed entries, and the bytes written.
///  The budget multiplies the largest index file by it to estimate what one rewrite holds.
///  Measured with `refresh_index_file` on throwaway repositories of this repository's tracked files, resident size read every 0.2 ms
///  in a fresh process (2026-11-03): an index of 2,786,564 bytes grew the process by 13,123,584 bytes (4.7 copies) and one of
///  1,066,894 bytes by 7,929,856 bytes (7.4 copies), which is 3.0 copies of the index plus about 4.7 MB that does not grow with
///  it. Four copies cover the growth for an index over 4.7 MB, the indexes this estimate exists for. Kept at 4 as the coordinator's pick under the owner's delegation, open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`, "Picked for these follow-ups", option A).
const INDEX_REWRITE_COPIES: u64 = 4;

/// What one rewrite of an index file is: the file, the tree it describes, and the trees its entries were copied from.
struct IndexTarget {
    /// The index file.
    index: PathBuf,
    /// The tree whose files the index describes.
    tree: PathBuf,
    /// The trees the entries were copied from (each is the reference tree joined with the worktree's place in the checkout).
    references: Vec<PathBuf>,
}

/// The rewrite of one index file, replaceable so a control can watch or delay it.
pub type RewriteOne<'a> = dyn Fn(&Path, &Path, &RefreshOptions<'_>) -> String + Sync + 'a;

/// What:
///  What a rewrite of every index of a checkout is told.
/// Why:
///   One value names the inputs: the checkout, how its indexes are locked, the trees they were copied from, the host the
///  budget is derived from, and the rewrite of one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllIndexes = { root: string; lock: IndexLock; references: string[]; host: HostResources; rewrite: RewriteOne };
/// ```
pub struct AllIndexes<'a> {
    /// The main checkout.
    pub root: &'a Path,
    /// How each index is protected while it is rewritten.
    pub lock: IndexLock<'a>,
    /// The trees the checkout was copied from; each linked worktree is looked up under the same relative path in them.
    pub references: &'a [PathBuf],
    /// The CPUs and memory the budget is derived from.
    pub host: &'a HostResources,
    /// The rewrite of one index file.
    pub rewrite: &'a RewriteOne<'a>,
}

/// What:
///  Rewrite the index of the checkout and of every linked worktree inside it, reporting each through `say`.
///  `&mut dyn LineSink` is a borrowed sink that receives one report line at a time.
///  `references` are the trees the checkout was copied from (empty trusts every entry); each linked worktree is looked up
///  under the same relative path in them.
/// Why:
///   The first `git status` of each of them is otherwise a full re-hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshAllIndexes(root: string, lock: IndexLock, references: string[], say: (line: string) => void): void;
/// ```
pub fn refresh_all_indexes(root: &Path, lock: IndexLock<'_>, references: &[PathBuf], say: &mut dyn LineSink) -> usize {
    return refresh_all_indexes_with(&AllIndexes { root, lock, references, host: &HostResources::read(), rewrite: &refresh_index_file_with }, say);
}

/// What:
///  The rewrite of one index, run by a worker of the ordered runner.
/// Why:
///   The lock, the rewrite and the hash length are what each rewrite reads; the runner shares them among its worker threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RewriteEachIndex implements OrderedWork<IndexTarget, string> { run(i, target) { return rewrite(target.index, target.tree, { lock, references: target.references, hashLength }); } }
/// ```
struct RewriteEachIndex<'a> {
    /// The rewrite of one index file.
    rewrite: &'a RewriteOne<'a>,
    /// How each index is protected while it is rewritten.
    lock: IndexLock<'a>,
    /// The length of the repository's object hashes, when it could be read.
    hash_length: Option<usize>,
}

impl OrderedWork<IndexTarget, String> for RewriteEachIndex<'_> {
    fn run(&self, _index: usize, target: &IndexTarget, _say: &mut dyn LineSink) -> String {
        return (self.rewrite)(target.index.as_path(), target.tree.as_path(), &RefreshOptions { lock: self.lock, references: target.references.as_slice(), hash_length: self.hash_length });
    }
}

/// What:
///  Says one line per index as each index ends, in index order.
/// Why:
///   The runner reports on the calling thread only and in input order, so the sink and the target list need no lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SayEachIndex implements OrderedEvents<string> { onEvent(event) { if (event.end) say(`index ${targets[event.index].index}: ${event.line}`); } }
/// ```
struct SayEachIndex<'a> {
    /// Where the report lines go.
    say: &'a mut dyn LineSink,
    /// The indexes, in the order they are reported.
    targets: &'a [IndexTarget],
}

impl OrderedEvents<String> for SayEachIndex<'_> {
    fn on_event(&mut self, event: Event<'_, String>) {
        if let Event::End(index, line) = event {
            self.say.say(format!("index {}: {line}", self.targets[index].index.display()).as_str());
        }
    }
}

/// What:
///  `refresh_all_indexes` with the host and the rewrite of one file chosen by the caller; returns how many indexes it covered.
/// Why:
///   Each index is its own file under its own lock describing its own tree, so the rewrites run side by side within a budget
///  and are reported in the order the checkout's index first, then each linked worktree's by path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const count = await refreshAllIndexesWith({ root, lock, references, host, rewrite }, say);
/// ```
pub fn refresh_all_indexes_with(request: &AllIndexes<'_>, say: &mut dyn LineSink) -> usize {
    let AllIndexes { root, lock, references, host, rewrite } = *request;
    let hash_length: Option<usize> = repository_hash_length(root.join(".git").as_path());
    let mut targets: Vec<IndexTarget> = vec![IndexTarget { index: root.join(".git").join("index"), tree: root.to_path_buf(), references: references.to_vec() }];
    for linked in linked_worktrees_inside(root) {
        let relative: &Path = linked.worktree.strip_prefix(root).unwrap_or(Path::new(""));
        targets.push(IndexTarget {
            index: linked.admin_directory.join("index"),
            tree: linked.worktree.clone(),
            references: joined_below(references, relative),
        });
    }
    let mut largest: u64 = 0;
    for target in &targets {
        let size: u64 = match std::fs::metadata(&target.index) {
            Ok(found) => found.len(),
            Err(_) => 0,
        };
        largest = largest.max(size);
    }
    let workers: usize = worker_budget(&BudgetRequest { items: targets.len(), per_item_memory: largest.saturating_mul(INDEX_REWRITE_COPIES), ceiling: None, cap: None, host });
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    for index in 0..targets.len() {
        lanes.push(vec![index]);
    }
    run_ordered(OrderedRun {
        items: targets.as_slice(),
        lanes: lanes.as_slice(),
        workers,
        work: &RewriteEachIndex { rewrite, lock, hash_length },
        on_event: &mut SayEachIndex { say, targets: targets.as_slice() },
    });
    return targets.len();
}

/// Each reference tree path joined with `relative`, in order.
fn joined_below(references: &[PathBuf], relative: &Path) -> Vec<PathBuf> {
    let mut joined: Vec<PathBuf> = Vec::new();
    for reference in references {
        joined.push(reference.join(relative));
    }
    return joined;
}

/// The controls for indexes rewritten side by side stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_parallel_tests.rs"]
mod parallel_tests;
