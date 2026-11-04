//! What:
//!  The drain of several repositories in one round of the service's after-commit work, side by side.
//! Why:
//!  Repositories share nothing: each has its own work directory, its own claims and its own pushes, so one repository's slow
//!  push or review never needs to hold up another's. The round's thread hands each repository to a worker and takes the answers
//!  back in the order the repositories were named (`parallel_runner.rs`), so the retry list is the one a drain of one after
//!  another gives. No lock is taken (`service_lock_inventory_tests.rs`): the handlers are shared read-only, every claim is made
//!  by an atomic rename as before, and the directories of one repository (a main checkout and its linked worktrees) share a lane
//!  and are drained one after another there, because they share a work directory and its upkeep.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const retries = await drainDirectories({ directories, handlers, now, host });
//! ```

use super::claim_time::ClaimTime;
use super::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use super::parallel_runner::{Event, OrderedRun, run_ordered};
use super::push_retry::{RetryChange, Retrying};
use super::service_work::{CLAIM_LEASE, Drained, WorkHandlers, common_directory_of, drain_common_records_planned, drain_planned};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The 90th percentile of `MemoryPeak` over the 28 commit-service units of this host that `systemctl --user show 'cli-git@*.service'
///  --property=MemoryPeak` reported on 2026-11-03 (the units' peaks since each one's last start; the Git processes a service
///  starts are inside the unit): 130,482,176 bytes. The median was 33,000,000 bytes, the 75th percentile 49,000,000 bytes, and the
///  largest 1,073,766,400 bytes, a unit that had run under the one-gibibyte cap this build no longer sets. A unit serves one
///  worktree for its whole life, so a unit's peak is an upper bound of what one repository's drain holds at once.
pub const MEASURED_REPOSITORY_PEAK: u64 = 130_482_176;

/// The bytes the drain of one repository holds at its peak: `MEASURED_REPOSITORY_PEAK` rounded up to a power of two, as
///  `CONVERSION_MEMORY` rounds its measurement. The coordinator's pick under the owner's delegation, open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`, "Parallelism caps and memory estimates").
pub const REPOSITORY_MEMORY: u64 = MEASURED_REPOSITORY_PEAK.next_power_of_two();

/// What:
///  One directory of a round, with the pushes of it that are due again.
/// Why:
///   The service loop owns the retry plan; it takes what is due for the directory out of the plan before the round and hands it to
///  the directory's worker inside the request, as an owned value, so no worker reads the plan.
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
    /// A directory with no plan: a push that could pass is reported as failed.
    pub fn plain(directory: PathBuf) -> Self {
        return Self { directory, retrying: Retrying::none() };
    }
}

/// What:
///  What a round gives back: when to look again at the directories that have a claim another instance holds, and the changes the
///  workers ask of the retry plan.
/// Why:
///   The waits keep the order the directories were named; the changes are in that order too, so applying them in sequence gives
///  the plan a drain of one directory after another would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RoundAnswer = { waits: [string, number][]; changes: RetryChange[] };
/// ```
#[derive(Debug, Default)]
pub struct RoundAnswer {
    /// For each directory that has such a claim, how long until its lease runs out, in the order the directories were named.
    pub waits: Vec<(PathBuf, Duration)>,
    /// What the workers ask of the retry plan.
    pub changes: Vec<RetryChange>,
}

/// What:
///  What a round needs to drain its repositories.
/// Why:
///   One value names the inputs, so the call reads as a list of facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RoundDrain<H> = { directories: string[]; handlers: H; now: ClaimTime; host: HostResources };
/// ```
pub struct RoundDrain<'a, H: WorkHandlers + Sync> {
    /// The Git directories named by the round's wakes, each once, in the order they were first named, each with what is due again.
    pub directories: &'a [Assignment],
    /// What runs each kind of work; shared read-only by every worker.
    pub handlers: &'a H,
    /// The reading of the clock the round started at.
    pub now: &'a ClaimTime,
    /// The CPUs and memory the budget is derived from.
    pub host: &'a HostResources,
}

/// What:
///  The lane of a directory: the common directory of its repository when it names one, else the directory itself.
/// Why:
///   The directories of one repository share a work directory and its upkeep, so they are drained one after another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lane = commonDirectoryOf(gitDir) ?? gitDir;
/// ```
fn lane_key(directory: &Path) -> PathBuf {
    return common_directory_of(directory).unwrap_or_else(|| { return directory.to_path_buf(); });
}

/// What:
///  Drain every directory, repositories side by side, and give, for each directory that has a claim another instance holds, how
///  long until that claim's lease runs out, in the order the directories were named.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await drainDirectories({ directories, handlers, now, host });
/// ```
pub fn drain_directories<H: WorkHandlers + Sync>(request: &RoundDrain<'_, H>) -> RoundAnswer {
    let RoundDrain { directories, handlers, now, host } = *request;
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    let mut lane_of: std::collections::HashMap<PathBuf, usize> = std::collections::HashMap::new();
    for (index, assignment) in directories.iter().enumerate() {
        match lane_of.get(&lane_key(assignment.directory.as_path())) {
            Some(&lane) => lanes[lane].push(index),
            None => {
                lane_of.insert(lane_key(assignment.directory.as_path()), lanes.len());
                lanes.push(vec![index]);
            }
        }
    }
    let workers: usize = worker_budget(&BudgetRequest { items: lanes.len(), per_item_memory: REPOSITORY_MEMORY, ceiling: None, cap: None, host });
    let mut answer: RoundAnswer = RoundAnswer::default();
    run_ordered(OrderedRun {
        items: directories,
        lanes: lanes.as_slice(),
        workers,
        work: &|_index: usize, assignment: &Assignment, _say: &mut dyn FnMut(&str)| {
            let own: Drained = drain_planned(assignment.directory.as_path(), handlers, now, &assignment.retrying);
            let common: Drained = drain_common_records_planned(assignment.directory.as_path(), handlers, now, CLAIM_LEASE, assignment.retrying.first);
            let wait: Option<Duration> = [own.wait, common.wait].into_iter().flatten().min();
            return (wait, own.changes.into_iter().chain(common.changes).collect::<Vec<RetryChange>>());
        },
        on_event: &mut |event: Event<'_, (Option<Duration>, Vec<RetryChange>)>| {
            if let Event::End(index, (wait, changes)) = event {
                if let Some(lease_wait) = wait {
                    answer.waits.push((directories[index].directory.clone(), lease_wait));
                }
                answer.changes.extend(changes);
            }
        },
    });
    return answer;
}

/// Round controls stay out of the release executable.
#[cfg(test)]
#[path = "service_round_tests.rs"]
mod tests;
