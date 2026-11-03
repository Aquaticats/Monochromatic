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
use super::service_work::{WorkHandlers, common_directory_of, drain, drain_common_records};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The bytes the drain of one repository holds at its peak, estimated: the items of a burst, the output of a push, and a review's
///  patch and answer, which `service_review.rs` bounds; about 64 MiB.
pub const REPOSITORY_MEMORY: u64 = 64 << 20;

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
    /// The Git directories named by the round's wakes, each once, in the order they were first named.
    pub directories: &'a [PathBuf],
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
/// const retries = await drainDirectories({ directories, handlers, now, host });
/// ```
pub fn drain_directories<H: WorkHandlers + Sync>(request: &RoundDrain<'_, H>) -> Vec<(PathBuf, Duration)> {
    let RoundDrain { directories, handlers, now, host } = *request;
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    let mut lane_of: std::collections::HashMap<PathBuf, usize> = std::collections::HashMap::new();
    for (index, directory) in directories.iter().enumerate() {
        match lane_of.get(&lane_key(directory.as_path())) {
            Some(&lane) => lanes[lane].push(index),
            None => {
                lane_of.insert(lane_key(directory.as_path()), lanes.len());
                lanes.push(vec![index]);
            }
        }
    }
    let _budget: usize = worker_budget(&BudgetRequest { items: lanes.len(), per_item_memory: REPOSITORY_MEMORY, ceiling: None, cap: None, host });
    let workers: usize = 1;
    let mut retries: Vec<(PathBuf, Duration)> = Vec::new();
    run_ordered(OrderedRun {
        items: directories,
        lanes: lanes.as_slice(),
        workers,
        work: &|_index: usize, directory: &PathBuf, _say: &mut dyn FnMut(&str)| {
            let waits: [Option<Duration>; 2] = [drain(directory.as_path(), handlers, now), drain_common_records(directory.as_path(), handlers, now)];
            return waits.into_iter().flatten().min();
        },
        on_event: &mut |event: Event<'_, Option<Duration>>| {
            if let Event::End(index, Some(wait)) = event {
                retries.push((directories[index].clone(), wait));
            }
        },
    });
    return retries;
}

/// Round controls stay out of the release executable.
#[cfg(test)]
#[path = "service_round_tests.rs"]
mod tests;
