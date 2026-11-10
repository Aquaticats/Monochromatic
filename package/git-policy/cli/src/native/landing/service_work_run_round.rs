//! What:
//!  Runs one round of the service's work over several directories:
//! hands each directory its due retries,
//! drains them side by side,
//!  and returns the repositories to look at again with the wait for each.
//! Why:
//!  `service_round.rs` drains the directories of a round;
//! this file calls it with the work of `service_work.rs`,
//! so
//!  the two do not use each other in a circle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const asked = await runPlannedRound(host, directories, handlers, retries, now);
//! ```

use crate::service_work_handlers_replanning_and_acceptance::*;
use crate::service_work::GitHandlers;
use crate::claim_time::ClaimTime;
use crate::push_retry::{PushRetries, RetryChange, Retrying};
use crate::parallel_budget::HostResources;
use crate::service_review_key::ReviewKey;
use crate::service_round::{Assignment, RoundAnswer, RoundDrain, drain_directories};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
/// What:
///  Runs one round of wakes:
/// holds each wake's review credential in memory for the round,
/// drains each named repository once,
///  repositories side by side,
/// and drops the credentials when the round ends.
/// Gives,
/// for each repository that has a claim
///  another instance holds,
/// how long until that claim's lease runs out,
/// in the order the repositories were named.
/// Why:
///   The credential is held only for the reviews of the round that brought it;
/// a service that restarts before the round
///  runs has none,
/// and its reviews record the plain "no key" outcome.
/// The service looks at such a repository again when the
///  time is over,
/// because the holder may have died and nothing else would wake the work.
/// Repositories share nothing,
/// so the
///  round drains them side by side (`service_round.rs`);
/// the credentials are passed to the workers as values in a copy of the
///  handlers made for this round,
/// so nothing is shared mutably and no lock is taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const round = handlers.forRound(keysOf(wakes)); await Promise.all(directories.map((dir) => drain(dir, round)));
/// ```
pub fn run_round(wakes: Vec<Wake>, handlers: &GitHandlers, retries: &mut PushRetries, now: &ClaimTime) -> Vec<(PathBuf, Duration)> {
    return run_round_on(&HostResources::read(), wakes, handlers, retries, now);
}

/// `run_round` with the host's resources chosen by the caller,
/// so a control states a host of any size.
pub fn run_round_on(host: &HostResources, wakes: Vec<Wake>, handlers: &GitHandlers, retries: &mut PushRetries, now: &ClaimTime) -> Vec<(PathBuf, Duration)> {
    let mut directories: Vec<PathBuf> = Vec::new();
    let mut keys: std::collections::HashMap<PathBuf, ReviewKey> = std::collections::HashMap::new();
    for (directory, review_key) in wakes {
        if let Some(key) = review_key {
            keys.insert(directory.clone(), key);
        }
        if !directories.contains(&directory) {
            directories.push(directory);
        }
    }
    let round: Arc<GitHandlers> = Arc::new(handlers.for_round(keys));
    return run_planned_round(host, directories.as_slice(), &round, retries, now);
}

/// What:
///  Runs one round over `directories` with the retry plan:
/// takes what is due for each directory out of the plan,
/// hands it to
///  that directory's worker as an owned value,
/// applies the changes the workers ask once they are done,
/// and gives for each
///  directory when to look at it again,
/// whichever is first of its leases and its plan.
/// Why:
///   The plan is owned by the caller and changed here on the caller's thread only,
/// so the workers share nothing and no lock is
///  taken.
/// A directory whose drain planned a push under another directory (a common directory of a bare repository) is named
///  too,
/// so that its wake is not lost.
/// The handlers are shared as one value the round's tasks may carry,
/// because a task outlives the frame that
///  starts it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const due = dirs.map((d) => retries.takeDue(d)); const answers = await Promise.all(dirs.map((d, i) => drain(d, due[i]))); retries.apply(answers.flatMap(changes));
/// ```
pub fn run_planned_round<H: WorkHandlers + Sync + Send + 'static>(host: &HostResources, directories: &[PathBuf], handlers: &Arc<H>, retries: &mut PushRetries, now: &ClaimTime) -> Vec<(PathBuf, Duration)> {
    // A child a stop left behind and the kernel has since released is collected here, so the service keeps no zombie for a
    //  process it abandoned.
    let _collected: usize = crate::stop_of_a_push_and_sweep_of_its_locks::reap_abandoned_children();
    let taken_at: Instant = Instant::now();
    let first: Duration = retries.first();
    let mut assignments: Vec<Assignment> = Vec::new();
    for directory in directories {
        assignments.push(Assignment {
            directory: directory.clone(),
            retrying: Retrying { first: Some(first), due: retries.take_due(directory.as_path(), taken_at), plans: retries.plans().map(Path::to_path_buf) },
        });
    }
    let answered: RoundAnswer = drain_directories(&RoundDrain { directories: assignments.as_slice(), handlers: Arc::clone(handlers), now, host });
    let mut named: Vec<PathBuf> = directories.to_vec();
    for change in &answered.changes {
        if let RetryChange::Plan { git_dir, .. } = change
            && !named.contains(git_dir)
        {
            named.push(git_dir.clone());
        }
    }
    retries.apply(answered.changes);
    let applied_at: Instant = Instant::now();
    let mut asked: Vec<(PathBuf, Duration)> = Vec::new();
    for directory in named {
        // The first lease wait the round reported for this directory.
        let mut lease: Option<Duration> = None;
        for (named_directory, wait) in &answered.waits {
            if *named_directory == directory {
                lease = Some(*wait);
                break;
            }
        }
        let planned: Option<Duration> = retries.next_wait(directory.as_path(), applied_at);
        if let Some(wait) = [lease, planned].into_iter().flatten().min() {
            asked.push((directory, wait));
        }
    }
    return asked;
}
