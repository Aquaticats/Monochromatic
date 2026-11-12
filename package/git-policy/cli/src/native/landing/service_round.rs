//! What:
//!  The drain of several repositories in one round of the service's after-commit work,
//! side by side.
//! Why:
//!  Repositories share nothing:
//! each has its own work directory,
//! its own claims and its own pushes,
//! so one repository's slow
//!  push or review never needs to hold up another's.
//! The round's task hands each repository to a lane of tasks on the runtime and takes the answers
//!  back in the order the repositories were named (`parallel_runner_on_the_runtime_awaiting.rs`),
//! so the retry list is the one a drain of one after
//!  another gives,
//! and no worker waits on Git or on storage while an item runs.
//! No lock is taken (`service_lock_inventory_tests.rs`):
//! the handlers are shared read-only,
//! every claim is made
//!  by an atomic rename as before,
//! and the directories of one repository (a main checkout and its linked worktrees) share a lane
//!  and are drained one after another there,
//! because they share a work directory and its upkeep.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const retries = await drainDirectories({ directories, handlers, now, host });
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::claim_time::ClaimTime;
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::parallel_runner::Event;
use crate::parallel_runner::OrderedEvents;
use crate::parallel_runner_on_the_runtime::LinesToTheConsumer;
use crate::parallel_runner_on_the_runtime::OrderedRunError;
use crate::parallel_runner_on_the_runtime_awaiting::{OrderedAwaitingWork, OrderedRunOfAwaitingOwned, run_ordered_awaiting_on_the_runtime};
use crate::push_retry::{RetryChange, Retrying};
use crate::service_work::{CLAIM_LEASE, Drained, PUSH_KINDS, WorkHandlers, common_directory_of, drain_common_records_planned_awaiting, drain_planned_awaiting};
use crate::wait_ceilings::AFTER_COMMIT_STATE_CEILING;
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

/// The 90th percentile of `MemoryPeak` over the 28 commit-service units of this host that `systemctl --user show 'cli-git@*.service'
///  --property=MemoryPeak` reported on 2026-11-03 (the units' peaks since each one's last start;
/// the Git processes a service
///  starts are inside the unit):
/// 130,482,176 bytes.
/// The median was 33,000,000 bytes,
/// the 75th percentile 49,000,000 bytes,
/// and the
///  largest 1,073,766,400 bytes,
/// a unit that had run under the one-gibibyte cap this build no longer sets.
/// A unit serves one
///  worktree for its whole life,
/// so a unit's peak is an upper bound of what one repository's drain holds at once.
pub const MEASURED_REPOSITORY_PEAK: u64 = 130_482_176;

/// The bytes the drain of one repository holds at its peak:
/// `MEASURED_REPOSITORY_PEAK` rounded up to a power of two,
/// as
///  `CONVERSION_MEMORY` rounds its measurement.
/// The coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Parallelism caps and memory estimates").
pub const REPOSITORY_MEMORY: u64 = MEASURED_REPOSITORY_PEAK.next_power_of_two();

/// What:
///  One directory of a round,
/// with the pushes of it that are due again.
/// Why:
///   The service loop owns the retry plan;
/// it takes what is due for the directory out of the plan before the round and hands it to
///  the directory's worker inside the request,
/// as an owned value,
/// so no worker reads the plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Assignment = { directory: string; retrying: Retrying };
/// ```
#[derive(Clone, Debug)]
pub struct Assignment {
    /// The Git directory to drain.
    pub directory: PathBuf,
    /// The first pause and the pushes due again for this directory.
    pub retrying: Retrying,
}

/// Methods over an assignment.
impl Assignment {
    /// A directory with no plan:
    /// a push that could pass is reported as failed.
    pub fn plain(directory: PathBuf) -> Self {
        return Self { directory, retrying: Retrying::none() };
    }
}

/// What:
///  What a round gives back:
/// when to look again at the directories that have a claim another instance holds,
/// and the changes the
///  workers ask of the retry plan.
/// Why:
///   The waits keep the order the directories were named;
/// the changes are in that order too,
/// so applying them in sequence gives
///  the plan a drain of one directory after another would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RoundAnswer = { waits: [string, number][]; changes: RetryChange[] };
/// ```
#[derive(Debug, Default)]
pub struct RoundAnswer {
    /// For each directory that has such a claim,
    /// how long until its lease runs out,
    /// in the order the directories were named.
    pub waits: Vec<(PathBuf, Duration)>,
    /// What the workers ask of the retry plan.
    pub changes: Vec<RetryChange>,
}

/// What:
///  What a round needs to drain its repositories.
/// Why:
///   One value names the inputs,
/// so the call reads as a list of facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RoundDrain<H> = { directories: string[]; handlers: H; now: ClaimTime; host: HostResources };
/// ```
pub struct RoundDrain<'a, H: WorkHandlers + Sync + Send + 'static> {
    /// The Git directories named by the round's wakes,
    /// each once,
    /// in the order they were first named,
    /// each with what is due again.
    pub directories: &'a [Assignment],
    /// What runs each kind of work;
    /// shared read-only by every lane,
    /// and owned here because the lanes are tasks that outlive this frame.
    pub handlers: Arc<H>,
    /// The reading of the clock the round started at.
    pub now: &'a ClaimTime,
    /// The CPUs and memory the budget is derived from.
    pub host: &'a HostResources,
}

/// What:
///  The lane of a directory:
/// the common directory of its repository when it names one,
/// else the directory itself.
/// Why:
///   The directories of one repository share a work directory and its upkeep,
/// so they are drained one after another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lane = commonDirectoryOf(gitDir) ?? gitDir;
/// ```
fn lane_key(directory: &Path) -> PathBuf {
    return match common_directory_of(directory) {
        Some(common) => common,
        None => directory.to_path_buf(),
    };
}

/// What:
///  The lane of each directory of the round,
/// read as a job of the blocking pool and awaited no longer than the after-commit work's own state.
/// Why:
///   `lane_key` reads the `commondir` file of each directory,
/// which is storage,
/// and the round is awaited by a task of the runtime:
///  a filesystem that stopped answering would hold a worker the runtime needs,
/// and a timer cannot end a read on that worker (finding 5
///  of the review of `b3d00b3be4`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keys = await laneKeysAwaiting(directories);
/// ```
async fn lane_keys_awaiting(directories: &[Assignment]) -> TheAnswerOfAJobAwaited<Vec<PathBuf>> {
    let mut asked: Vec<PathBuf> = Vec::with_capacity(directories.len());
    for assignment in directories {
        asked.push(assignment.directory.clone());
    }
    return run_a_job_awaited_no_longer_than(AFTER_COMMIT_STATE_CEILING, asked, lane_keys_in_a_job).await;
}

/// `lane_keys_awaiting`'s work,
/// as the job of the pool takes it.
fn lane_keys_in_a_job(directories: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut keys: Vec<PathBuf> = Vec::with_capacity(directories.len());
    for directory in &directories {
        keys.push(lane_key(directory.as_path()));
    }
    return keys;
}

/// What:
///  The round's directories in one lane,
/// which is every directory one after another,
/// and nothing at all for no directory.
/// Why:
///  A lane key that could not be read may not put two directories of one repository side by side,
/// and one lane never does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lanes = oneLaneOf(directories.length);
/// ```
fn one_lane_of(count: usize) -> Vec<Vec<usize>> {
    if count == 0 {
        return Vec::new();
    }
    return vec![(0..count).collect()];
}

/// What:
///  The lanes of the round:
/// one lane per repository,
/// each holding its directories in the order they were named.
/// Why:
///   Two directories of one repository share a work directory and its upkeep,
/// so they are drained one after another;
///  directories of different repositories share nothing and run side by side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lanes = lanesOf(keys);
/// ```
fn lanes_of(keys: &[PathBuf]) -> Vec<Vec<usize>> {
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    let mut lane_of: HashMap<PathBuf, usize> = HashMap::new();
    for (index, key) in keys.iter().enumerate() {
        match lane_of.get(key) {
            Some(&lane) => lanes[lane].push(index),
            None => {
                lane_of.insert(key.clone(), lanes.len());
                lanes.push(vec![index]);
            }
        }
    }
    return lanes;
}

/// What:
///  The drain of one directory:
/// the records of the directory itself and those of the common directory of a bare repository,
/// awaited
///  by whoever asks for them.
/// Why:
///   The lane of the round and the control that drains the same directories one after another
/// (`service_round_tests.rs`)
/// both run this one body,
/// so a difference between them is the runner's and never the drain's.
/// It claims the pushes alone (`PUSH_KINDS`):
/// the kinds that can take minutes are claimed by the slow lane,
/// which the loop asks beside this round
///  (`service_work_slow_lane.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [wait, changes] = await drainOne(handlers, now, assignment)
/// ```
async fn drain_one<H: WorkHandlers + Sync>(handlers: &H, now: &ClaimTime, assignment: &Assignment) -> (Option<Duration>, Vec<RetryChange>) {
    let own: Drained = drain_planned_awaiting(assignment.directory.as_path(), handlers, now, CLAIM_LEASE, &assignment.retrying, &PUSH_KINDS).await;
    let common: Drained = drain_common_records_planned_awaiting(assignment.directory.as_path(), handlers, now, CLAIM_LEASE, &assignment.retrying, &PUSH_KINDS).await;
    let wait: Option<Duration> = [own.wait, common.wait].into_iter().flatten().min();
    return (wait, own.changes.into_iter().chain(common.changes).collect::<Vec<RetryChange>>());
}

/// What:
///  One lane's item work,
/// held by the runner as a value it may move between workers.
/// Why:
///   Every step the drain runs reads its child through the runtime,
/// so a lane is a task:
/// a job of the blocking pool cannot await anything,
/// and the faces that
///  enter the runtime refuse a call from inside it.
/// The handlers are shared by the lanes as one reference,
/// which a task may carry because the handlers are `Sync`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DrainEachDirectory implements OrderedAwaitingWork<Assignment, [Duration | undefined, RetryChange[]]> { async run(i, assignment) { return drainOne(handlers, now, assignment); } }
/// ```
struct DrainEachDirectory<H> {
    /// What the drain does with each record it finds.
    handlers: Arc<H>,
    /// The time the drain judges leases and retries by.
    now: ClaimTime,
}

impl<H: WorkHandlers + Sync + Send> OrderedAwaitingWork<Assignment, (Option<Duration>, Vec<RetryChange>)> for DrainEachDirectory<H> {
    fn run<'a>(
        &'a self,
        _index: usize,
        assignment: &'a Assignment,
        _say: LinesToTheConsumer<(Option<Duration>, Vec<RetryChange>)>,
    ) -> Pin<Box<dyn Future<Output = (Option<Duration>, Vec<RetryChange>)> + Send + 'a>> {
        return Box::pin(async move { return drain_one(self.handlers.as_ref(), &self.now, assignment).await; });
    }
}

/// What:
///  Collects,
/// in the order the directories were named,
/// the wait of each claim another instance holds and every retry change.
/// Why:
///   The runner reports on the calling thread only and in input order,
/// so the answer needs no lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CollectWaitsAndChanges implements OrderedEvents<[Duration | undefined, RetryChange[]]> { answer: RoundAnswer }
/// ```
struct CollectWaitsAndChanges<'a> {
    /// The directories,
    /// in the order they were named.
    directories: &'a [Assignment],
    /// The answer being built.
    answer: &'a mut RoundAnswer,
}

impl OrderedEvents<(Option<Duration>, Vec<RetryChange>)> for CollectWaitsAndChanges<'_> {
    fn on_event(&mut self, event: Event<'_, (Option<Duration>, Vec<RetryChange>)>) {
        if let Event::End(index, (wait, changes)) = event {
            if let Some(lease_wait) = wait {
                self.answer.waits.push((self.directories[index].directory.clone(), lease_wait));
            }
            self.answer.changes.extend(changes);
        }
    }
}

/// What:
///  Drain every directory,
/// repositories side by side as lanes of tasks,
/// and give,
/// for each directory that has a claim another instance holds,
/// how
///  long until that claim's lease runs out,
/// and the changes the workers ask of the retry plan,
/// both in the order the directories were named.
/// Also,
/// when a lane stopped before its last directory,
/// why,
/// because a
///  caller that is told nothing reads the silence as a refresh that covered every directory.
/// Why:
///   See the module description.
/// A lane that ends early runs no item after the one it stopped in,
/// and the claims those items hold keep their
///  lease,
/// so the next round of the work loop finds them again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [answer, notFinished] = await drainDirectoriesAwaiting({ directories, handlers, now, host });
/// ```
pub async fn drain_directories_awaiting<H: WorkHandlers + Sync + Send + 'static>(request: &RoundDrain<'_, H>) -> (RoundAnswer, Option<OrderedRunError>) {
    let directories: &[Assignment] = request.directories;
    let now: &ClaimTime = request.now;
    let host: &HostResources = request.host;
    // The lane of each directory is read off the pool, and a read that gives no answer puts every directory in one lane: the
    //  directories of one repository must never be drained side by side, and one lane is always that.
    let lanes: Vec<Vec<usize>> = match lane_keys_awaiting(directories).await {
        TheAnswerOfAJobAwaited::Ran(keys) => lanes_of(keys.as_slice()),
        TheAnswerOfAJobAwaited::NotRun => {
            crate::service_warn!("the common directory of each repository of this round was not read, because the system gave no thread for it, so the round drains its directories one after another");
            one_lane_of(directories.len())
        }
        TheAnswerOfAJobAwaited::NoAnswer => {
            crate::service_warn!("the common directory of each repository of this round gave no answer, so the round drains its directories one after another");
            one_lane_of(directories.len())
        }
    };
    let workers: usize = worker_budget(&BudgetRequest { items: lanes.len(), per_item_memory: REPOSITORY_MEMORY, ceiling: None, cap: None, host });
    let mut answer: RoundAnswer = RoundAnswer::default();
    let finished: Result<(), OrderedRunError> = run_ordered_awaiting_on_the_runtime(OrderedRunOfAwaitingOwned {
        items: Arc::from(directories.to_vec()),
        lanes: Arc::from(lanes),
        workers,
        work: Arc::new(DrainEachDirectory { handlers: Arc::clone(&request.handlers), now: now.clone() }),
        on_event: &mut CollectWaitsAndChanges { directories, answer: &mut answer },
    })
    .await;
    return (answer, finished.err());
}

/// What:
///  Drain every directory from a caller outside the runtime,
/// entering it once.
/// Why:
///   The service's work loop runs on its own thread and keeps this face;
/// the body is the awaited one,
/// so a round cannot
///  drain differently depending on which face asked it.
/// A round that stopped before its last directory says so in the service log,
/// and every
///  directory it did drain answers as the awaited face reported it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [answer, stopped] = runOnRuntime(drainDirectoriesAwaiting(request)); if (stopped) warn(stopped)
/// ```
pub fn drain_directories<H: WorkHandlers + Sync + Send + 'static>(request: &RoundDrain<'_, H>) -> RoundAnswer {
    return match crate::lazy_runtime::run_on_runtime(drain_directories_awaiting(request)) {
        Ok((answer, stopped)) => {
            if let Some(reason) = stopped {
                crate::service_warn!("the round of the service's work did not finish: {reason}");
            }
            answer
        }
        Err(refusal) => {
            crate::service_warn!("the round of the service's work was not run: {refusal}");
            RoundAnswer::default()
        }
    };
}

/// Round controls stay out of the release executable.
#[cfg(test)]
#[path = "service_round_tests.rs"]
mod tests;
