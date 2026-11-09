//! What:
//!  The rewrite of the index of the checkout and of every linked worktree inside it,
//! side by side within the host's budget.
//! Why:
//!  Each index is its own file under its own lock describing its own tree,
//! so the rewrites are independent:
//! they run on the
//!  awaited ordered runner (`parallel_runner_on_the_runtime_awaiting.rs`) and are reported in the order
//! the checkout's index first,
//! then each linked
//!  worktree's by path.
//! Each rewrite is one job of the process's blocking pool,
//! awaited by a task of the runtime,
//!  so the process's one bound on blocking jobs covers it.
//! The rewrite of one file is `refresh_index_file_with` in `subvolume_index_files.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const count = await refreshAllIndexes(root, lock, references, say);
//! ```

use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::lazy_runtime::run_on_runtime;
use crate::line_sink::LineSink;
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::parallel_runner::{Event, OrderedEvents};
use crate::parallel_runner_on_the_runtime::LinesToTheConsumer;
use crate::parallel_runner_on_the_runtime_awaiting::{OrderedAwaitingWork, OrderedRunOfAwaitingOwned, run_ordered_awaiting_on_the_runtime};
use crate::subvolume_error::MigrateError;
use crate::subvolume_index_files::{IndexLock, RefreshOptions, linked_worktrees_inside, refresh_index_file_with, repository_hash_length};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

/// The number of copies of an index file one rewrite holds at once:
/// the bytes read,
/// the parsed entries,
/// and the bytes written.
///  The budget multiplies the largest index file by it to estimate what one rewrite holds.
///  Measured with `refresh_index_file` on throwaway repositories of this repository's tracked files,
/// resident size read every 0.2 ms
///  in a fresh process (2026-11-03):
/// an index of 2,786,564 bytes grew the process by 13,123,584 bytes (4.7 copies) and one of
///  1,066,894 bytes by 7,929,856 bytes (7.4 copies),
/// which is 3.0 copies of the index plus about 4.7 MB that does not grow with
///  it.
/// Four copies cover the growth for an index over 4.7 MB,
/// the indexes this estimate exists for.
/// Kept at 4 as the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// option A).
const INDEX_REWRITE_COPIES: u64 = 4;

/// What one rewrite of an index file is:
/// the file,
/// the tree it describes,
/// and the trees its entries were copied from.
struct IndexTarget {
    /// The index file.
    index: PathBuf,
    /// The tree whose files the index describes.
    tree: PathBuf,
    /// The trees the entries were copied from (each is the reference tree joined with the worktree's place in the checkout).
    references: Vec<PathBuf>,
}

/// The rewrite of one index file,
/// replaceable so a control can watch or delay it.
///  It is shared rather than lent because a job of the blocking pool must own what it reads,
/// and it must be `Send` so the job that runs it may travel to another thread.
pub type RewriteOne = dyn Fn(&Path, &Path, &RefreshOptions<'_>) -> String + Send + Sync;

/// What:
///  What a rewrite of every index of a checkout is told.
/// Why:
///   One value names the inputs:
/// the checkout,
/// how its indexes are locked,
/// the trees they were copied from,
/// the host the
///  budget is derived from,
/// and the rewrite of one file.
///  Every field is owned so the value can outlive the call:
/// the items of the run are jobs of the blocking pool,
/// which must be `'static`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AllIndexes = { root: string; lock: IndexLock; references: string[]; host: HostResources; rewrite: RewriteOne };
/// ```
pub struct AllIndexes {
    /// The main checkout.
    pub root: PathBuf,
    /// How each index is protected while it is rewritten.
    pub lock: IndexLock,
    /// The trees the checkout was copied from;
    /// each linked worktree is looked up under the same relative path in them.
    pub references: Vec<PathBuf>,
    /// The CPUs and memory the budget is derived from.
    pub host: HostResources,
    /// The rewrite of one index file.
    pub rewrite: Arc<RewriteOne>,
}

/// What:
///  Rewrite the index of the checkout and of every linked worktree inside it,
/// reporting each through `say`.
///  `&mut dyn LineSink` is a borrowed sink that receives one report line at a time.
///  `references` are the trees the checkout was copied from (empty trusts every entry);
/// each linked worktree is looked up
///  under the same relative path in them.
/// Why:
///   The first `git status` of each of them is otherwise a full re-hash.
/// The callers of this face are synchronous (the migration that has just swapped a checkout);
///  so it enters the process's runtime once and waits there;
/// from inside the runtime it refuses,
///  and `refresh_all_indexes_awaiting` is the face such a caller uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refreshAllIndexes(root: string, lock: IndexLock, references: string[], say: (line: string) => void): void;
/// ```
pub fn refresh_all_indexes(root: &Path, lock: IndexLock, references: &[PathBuf], say: &mut dyn LineSink) -> Result<usize, MigrateError> {
    let request: AllIndexes = AllIndexes {
        root: root.to_path_buf(),
        lock,
        references: references.to_vec(),
        host: HostResources::read(),
        rewrite: Arc::new(refresh_index_file_with),
    };
    return refresh_all_indexes_with(&request, say);
}

/// What:
///  The rewrite of one index,
/// run by a task of the awaited ordered runner.
/// Why:
///   The lock,
/// the rewrite and the hash length are what each rewrite reads;
/// the runner shares them with its tasks,
///  and each job takes its own share of them before it goes to the pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RewriteEachIndex implements OrderedAwaitingWork<IndexTarget, string> { async run(i, target) { return await runBlockingJobWithinTheBudget(() => rewrite(target.index, target.tree, { lock, references: target.references, hashLength })); } }
/// ```
struct RewriteEachIndex {
    /// The rewrite of one index file.
    rewrite: Arc<RewriteOne>,
    /// How each index is protected while it is rewritten.
    lock: IndexLock,
    /// The length of the repository's object hashes,
    /// when it could be read.
    hash_length: Option<usize>,
    /// How many rewrites never ran,
    /// so a phase that covered fewer indexes than it counted says so instead of marking itself done.
    not_rewritten: Arc<AtomicUsize>,
}

impl OrderedAwaitingWork<IndexTarget, String> for RewriteEachIndex {
    fn run<'a>(&'a self, _index: usize, target: &'a IndexTarget, _say: LinesToTheConsumer<String>) -> Pin<Box<dyn Future<Output = String> + Send + 'a>> {
        return Box::pin(async move {
            let rewrite: Arc<RewriteOne> = Arc::clone(&self.rewrite);
            let lock: IndexLock = self.lock.clone();
            let hash_length: Option<usize> = self.hash_length;
            let index: PathBuf = target.index.clone();
            let tree: PathBuf = target.tree.clone();
            let references: Vec<PathBuf> = target.references.clone();
            let rewritten = run_blocking_job_within_the_budget(move || {
                return (rewrite)(index.as_path(), tree.as_path(), &RefreshOptions { lock, references: references.as_slice(), hash_length });
            })
            .await;
            return match rewritten {
                Ok(line) => line,
                // Whether the index was published is not known here: the step may have ended before it began,
                //  or after it installed the file, and this line says which kind of end this was.
                Err(refused) => {
                    self.not_rewritten.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    format!("not rewritten ({refused})")
                }
            };
        });
    }
}

/// What:
///  Says one line per index as each index ends,
/// in index order.
/// Why:
///   The runner reports on the calling task only and in input order,
/// so the sink and the target list need no lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SayEachIndex implements OrderedEvents<string> { onEvent(event) { if (event.end) say(`index ${targets[event.index].index}: ${event.line}`); } }
/// ```
struct SayEachIndex<'a> {
    /// Where the report lines go.
    say: &'a mut dyn LineSink,
    /// The indexes,
    /// in the order they are reported.
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
///  `refresh_all_indexes` with the host and the rewrite of one file chosen by the caller;
/// returns how many indexes it covered.
/// Why:
///   Each index is its own file under its own lock describing its own tree,
/// so the rewrites run side by side within a budget
///  and are reported in the order the checkout's index first,
/// then each linked worktree's by path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const count = refreshAllIndexesWith({ root, lock, references, host, rewrite }, say);
/// ```
pub fn refresh_all_indexes_with(request: &AllIndexes, say: &mut dyn LineSink) -> Result<usize, MigrateError> {
    return match run_on_runtime(refresh_all_indexes_awaiting(request, say)) {
        Ok(count) => count,
        Err(refusal) => Err(MigrateError::Runtime(refusal.to_string())),
    };
}

/// What:
///  `refresh_all_indexes` on the runtime:
/// every index is a task of the awaited ordered runner,
/// and each rewrite is one job of the blocking pool inside its task.
/// Why:
///   A caller that is itself asynchronous cannot enter the runtime,
/// so it awaits this face;
/// the lines and the count are the synchronous face's.
/// One limit,
/// said rather than hidden:
/// the storage reads this face makes before the run (the hash length,
/// the linked worktrees and the size of each index) run on the task that awaits it,
/// not on the blocking pool,
/// because a job of the pool must own what it reads.
/// They are a handful of short reads once per migration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function refreshAllIndexesAwaiting({ root, lock, references, host, rewrite }, say): Promise<number>;
/// ```
pub async fn refresh_all_indexes_awaiting(request: &AllIndexes, say: &mut dyn LineSink) -> Result<usize, MigrateError> {
    let AllIndexes { root, lock, references, host, rewrite } = request;
    let hash_length: Option<usize> = repository_hash_length(root.join(".git").as_path());
    let mut targets: Vec<IndexTarget> = vec![IndexTarget { index: root.join(".git").join("index"), tree: root.clone(), references: references.clone() }];
    for linked in linked_worktrees_inside(root.as_path()) {
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
    // The items are shared, so the consumer still knows the path of each index it reports.
    let items: Arc<[IndexTarget]> = Arc::from(targets);
    let count: usize = items.len();
    let not_rewritten: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let reported = run_ordered_awaiting_on_the_runtime(OrderedRunOfAwaitingOwned {
        items: Arc::clone(&items),
        lanes: Arc::from(lanes),
        workers,
        work: Arc::new(RewriteEachIndex { rewrite: Arc::clone(rewrite), lock: lock.clone(), hash_length, not_rewritten: Arc::clone(&not_rewritten) }),
        on_event: &mut SayEachIndex { say, targets: items.as_ref() },
    })
    .await;
    // The lines were said by the consumer as each index ended; what ended the run is the answer of the call.
    if let Err(stopped) = reported {
        return Err(MigrateError::Runtime(stopped.to_string()));
    }
    // A rewrite that never ran is not an index that needed nothing: the caller marks its phase done, so it must be told.
    let missed: usize = not_rewritten.load(std::sync::atomic::Ordering::SeqCst);
    if missed > 0 {
        return Err(MigrateError::Runtime(format!("{missed} of {count} index file(s) were not rewritten, because the work of each one could not be started; the indexes stay as they were")));
    }
    return Ok(count);
}

/// Each reference tree path joined with `relative`,
/// in order.
fn joined_below(references: &[PathBuf], relative: &Path) -> Vec<PathBuf> {
    let mut joined: Vec<PathBuf> = Vec::new();
    for reference in references {
        joined.push(reference.join(relative));
    }
    return joined;
}
