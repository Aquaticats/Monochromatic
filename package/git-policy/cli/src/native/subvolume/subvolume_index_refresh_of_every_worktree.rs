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

use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::lazy_runtime::run_on_runtime;
use crate::line_sink::LineSink;
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::parallel_runner::{Event, OrderedEvents};
use crate::parallel_runner_on_the_runtime::LinesToTheConsumer;
use crate::parallel_runner_on_the_runtime_awaiting::{OrderedAwaitingWork, OrderedRunOfAwaitingOwned, run_ordered_awaiting_on_the_runtime};
use crate::the_claim_a_job_takes_before_running_it::TheClaimOnAJobsWork;
use crate::subvolume_error::MigrateError;
use crate::subvolume_index_files::{IndexLock, RefreshOptions, linked_worktrees_inside, refresh_index_file_with, repository_hash_length};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    /// What the platform of the command that asked for this run says about its writes reaching the disk.
    /// Each rewrite is a job of the blocking pool,
    /// whose thread is not the caller's,
    /// so a caller that keeps its writes in memory says so here and each job installs it.
    pub flushes_to_disk: bool,
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
pub fn refresh_all_indexes(root: &Path, lock: IndexLock, references: &[PathBuf], flushes_to_disk: bool, say: &mut dyn LineSink) -> Result<usize, MigrateError> {
    let request: AllIndexes = AllIndexes {
        root: root.to_path_buf(),
        lock,
        references: references.to_vec(),
        host: HostResources::read(),
        flushes_to_disk,
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
    /// Whether the writes of a rewrite reach the disk;
    /// each job installs this answer on its own thread,
    /// which is not the thread that asked for the run.
    flushes_to_disk: bool,
    /// How many rewrites gave no result,
    /// so a phase that covered fewer indexes than it counted says so instead of marking itself done.
    gave_no_result: Arc<RewritesThatGaveNoResult>,
}

/// What:
///  How many rewrites gave no result,
/// by the kind of end,
/// because the two kinds promise different things about the person's files.
/// Why:
///   A job that never started left its index alone.
/// A job that ended without a result may have installed its index before it ended,
/// and nothing here can tell which,
///  so the report keeps that uncertainty instead of promising an unchanged file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RewritesThatGaveNoResult = { neverStarted: number; endedWithoutAResult: number };
/// ```
#[derive(Default)]
struct RewritesThatGaveNoResult {
    /// The jobs the process's bound refused a thread for,
    /// whose indexes are as they were.
    never_started: AtomicUsize,
    /// The jobs that panicked or were cancelled,
    /// whose indexes may or may not have been installed.
    ended_without_a_result: AtomicUsize,
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
            // The claim decides, once, between this job doing the work and the report giving up on it: the pool
            //  queues a job before it asks for a thread, so a job it could not start may still run later.
            let claim: Arc<TheClaimOnAJobsWork> = TheClaimOnAJobsWork::new();
            let jobs_claim: Arc<TheClaimOnAJobsWork> = Arc::clone(&claim);
            let flushes_to_disk: bool = self.flushes_to_disk;
            let rewritten = run_blocking_job_within_the_budget(move || {
                // The platform's answer about the disk travels with the job: this thread is not the caller's, so a call that
                //  keeps its writes in memory must say so here for the index it installs as well.
                let _flushes: crate::subvolume_file_write_steps::WritesKeptInMemory =
                    crate::subvolume_file_write_steps::WritesKeptInMemory::for_flush_policy(flushes_to_disk);
                // The claim comes first, before anything reads or writes the index.
                if !jobs_claim.the_job_takes_it() {
                    return None;
                }
                return Some((rewrite)(index.as_path(), tree.as_path(), &RefreshOptions { lock, references: references.as_slice(), hash_length }));
            })
            .await;
            return match rewritten {
                Ok(Some(line)) => line,
                // The report had already given up on this job, so the work was not done and the index is as it was.
                Ok(None) => String::from("not rewritten, because the report gave up on this work before it ran"),
                Err(refused) => match refused {
                    BlockingJobError::NoThreadToStart if claim.the_reporter_takes_it() => {
                        self.gave_no_result.never_started.fetch_add(1, Ordering::SeqCst);
                        format!("not rewritten ({refused})")
                    }
                    // The kinds of end that leave the index unknown: the job started on another thread after the report gave
                    //  up on it (or took this item's own claim first), it panicked, or it was cut off, or the runner took no
                    //  claim on the queued closure and nothing here can say it will not run.
                    BlockingJobError::MayStillRunAfterItsThreadWasRefused | BlockingJobError::NoThreadToStart | BlockingJobError::StartedAfterItsThreadWasRefused | BlockingJobError::Panicked | BlockingJobError::Cancelled => {
                        self.gave_no_result.ended_without_a_result.fetch_add(1, Ordering::SeqCst);
                        format!("ended without a known result, so this index may have been written or may be as it was ({refused})")
                    }
                },
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
///  Everything one run reads before its rewrites start:
/// the indexes with the trees they describe,
/// the length of the repository's object hashes,
/// and the size of the largest index.
/// Why:
///  The preparation runs as one job of the blocking pool
/// (`prepare_the_run`),
///  so it owns what it reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedRun = { targets: IndexTarget[]; hashLength?: number; largest: number };
/// ```
struct PreparedRun {
    /// The indexes and the trees they describe,
    /// in the order they are reported.
    targets: Vec<IndexTarget>,
    /// The length of the repository's object hashes,
    /// when it could be read.
    hash_length: Option<usize>,
    /// The largest index of the run,
    /// in bytes,
    /// which the budget divides the free memory by.
    largest: u64,
}

/// What:
///  Reads what one run needs before its rewrites start.
/// Why:
///  These are the only storage reads of a run that do not happen in an item's job,
/// and the first of them is the repository's configuration:
/// a named pipe there blocks the open until a writer comes,
/// which must happen on the blocking pool and never on the thread that polls the runtime
/// (`refresh_all_indexes_awaiting`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareTheRun(root: string, references: string[]): PreparedRun;
/// ```
fn prepare_the_run(root: &Path, references: &[PathBuf]) -> PreparedRun {
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
    return PreparedRun { targets, hash_length, largest };
}

/// What:
///  `refresh_all_indexes` on the runtime:
/// the storage reads that come first are one job of the blocking pool,
/// every index is a task of the awaited ordered runner,
/// and each rewrite is one job of the blocking pool inside its task.
/// Why:
///   A caller that is itself asynchronous cannot enter the runtime,
/// so it awaits this face;
/// the lines and the count are the synchronous face's.
/// The preparation is a job of the pool rather than a read on the awaiting task,
/// because the first of those reads is the repository's configuration with a synchronous `std::fs::read`
/// (`repository_hash_length`),
/// and a named pipe where that file is blocks the open until a writer comes:
/// on the runtime's own thread the future would never reach an await,
/// no timeout could expire,
/// and every other task of that runtime would stop with it
/// ("nothing should ever wait forever").
/// On the pool a thread waits there instead,
/// and the caller's timeout expires.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function refreshAllIndexesAwaiting({ root, lock, references, host, rewrite }, say): Promise<number>;
/// ```
pub async fn refresh_all_indexes_awaiting(request: &AllIndexes, say: &mut dyn LineSink) -> Result<usize, MigrateError> {
    let AllIndexes { root, lock, references, host, flushes_to_disk, rewrite } = request;
    let (root_owned, references_owned): (PathBuf, Vec<PathBuf>) = (root.clone(), references.clone());
    let prepared: PreparedRun = match run_blocking_job_within_the_budget(move || { return prepare_the_run(root_owned.as_path(), references_owned.as_slice()); }).await {
        Ok(prepared) => prepared,
        // Nothing was read and nothing was rewritten, and the runtime's own reason is why.
        Err(refused) => return Err(MigrateError::Runtime(refused.to_string())),
    };
    let PreparedRun { targets, hash_length, largest } = prepared;
    let workers: usize = worker_budget(&BudgetRequest { items: targets.len(), per_item_memory: largest.saturating_mul(INDEX_REWRITE_COPIES), ceiling: None, cap: None, host });
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    for index in 0..targets.len() {
        lanes.push(vec![index]);
    }
    // The items are shared, so the consumer still knows the path of each index it reports.
    let items: Arc<[IndexTarget]> = Arc::from(targets);
    let count: usize = items.len();
    let gave_no_result: Arc<RewritesThatGaveNoResult> = Arc::new(RewritesThatGaveNoResult::default());
    let reported = run_ordered_awaiting_on_the_runtime(OrderedRunOfAwaitingOwned {
        items: Arc::clone(&items),
        lanes: Arc::from(lanes),
        workers,
        work: Arc::new(RewriteEachIndex { rewrite: Arc::clone(rewrite), lock: lock.clone(), hash_length, flushes_to_disk: *flushes_to_disk, gave_no_result: Arc::clone(&gave_no_result) }),
        on_event: &mut SayEachIndex { say, targets: items.as_ref() },
    })
    .await;
    // The lines were said by the consumer as each index ended; what ended the run is the answer of the call.
    if let Err(stopped) = reported {
        return Err(MigrateError::Runtime(stopped.to_string()));
    }
    // A rewrite that gave no result is not an index that needed nothing: the caller marks its phase done, so it must be
    //  told, and each kind of end says only what it can vouch for.
    let never_started: usize = gave_no_result.never_started.load(Ordering::SeqCst);
    let ended_without_a_result: usize = gave_no_result.ended_without_a_result.load(Ordering::SeqCst);
    if never_started == 0 && ended_without_a_result == 0 {
        return Ok(count);
    }
    let mut why: Vec<String> = Vec::new();
    if never_started > 0 {
        // Only a job whose work the report claimed before it could start is promised unchanged.
        why.push(format!("{never_started} of {count} index file(s) were not rewritten, because the work of each one was given up on before it could start, so those stay as they were"));
    }
    if ended_without_a_result > 0 {
        why.push(format!("{ended_without_a_result} of {count} index file(s) ended without a known result, so whether those were rewritten is not known; run the command again to rewrite them"));
    }
    return Err(MigrateError::Runtime(format!("{}.", why.join(". "))));
}

/// Controls for the claim a rewrite takes stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_index_refresh_of_every_worktree_tests.rs"]
mod tests;

/// Each reference tree path joined with `relative`,
/// in order.
fn joined_below(references: &[PathBuf], relative: &Path) -> Vec<PathBuf> {
    let mut joined: Vec<PathBuf> = Vec::new();
    for reference in references {
        joined.push(reference.join(relative));
    }
    return joined;
}
