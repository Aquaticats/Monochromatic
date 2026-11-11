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
//! Each step here has a face a task of the runtime may ask,
//! because the loop of the after-commit work is
//!  a task (`service_server_work_loop_awaiting.rs`):
//! the awaited face awaits the round's lanes and puts the plan's own writes on the
//!  blocking pool,
//! while the synchronous face is what a thread outside the runtime asks.
//! Both faces plan a round,
//! name its directories and
//!  read its answer through the same functions,
//! so a difference between them is the runner's and never the round's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const asked = await runPlannedRound(host, directories, handlers, retries, now);
//! ```

use crate::service_work_handlers_replanning_and_acceptance::*;
use crate::service_work::GitHandlers;
use crate::claim_time::ClaimTime;
use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::wait_ceilings::AFTER_COMMIT_STATE_CEILING;
use crate::parallel_runner_on_the_runtime::OrderedRunError;
use crate::push_retry::{PushRetries, RetryChange, Retrying};
use crate::parallel_budget::HostResources;
use crate::service_review_key::ReviewKey;
use crate::service_round::{Assignment, RoundAnswer, RoundDrain, drain_directories, drain_directories_awaiting};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// What:
///  Apply what the round's workers asked of the retry plan,
/// as a job of the process's blocking pool,
/// awaited no longer than the journal's state write,
///  and give the plan back to its owner whichever way the job ended.
/// Why:
///   A change writes or removes a plan's own file in the instance's journal,
/// so it is storage,
/// and a task of the runtime never does it itself:
///  a filesystem that stopped answering would hold a worker the runtime needs,
/// and a timer cannot end a write on that worker.
/// The plan has one owner at a time,
/// so the job takes it and hands it back,
/// and nothing locks it.
/// A job the system gives no thread for,
/// or one that starts on another thread,
/// is answered by keeping the round's own changes in the list,
/// which is what the loop's next wake reads.
/// The plan's files are the workers' own:
/// each wrote the plan it promised before the person was told a retry
///  comes,
/// so a round that writes nothing loses no promise,
/// and the round after it writes the files again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// retries = await applyAwaiting(retries, changes)
/// ```
async fn apply_awaiting(retries: PushRetries, changes: Vec<RetryChange>) -> PushRetries {
    let (kept, asked): (PushRetries, Vec<RetryChange>) = (retries.clone(), changes.clone());
    return match run_a_job_awaited_no_longer_than(AFTER_COMMIT_STATE_CEILING, (retries, changes), apply_in_a_job).await {
        TheAnswerOfAJobAwaited::Ran(back) => back,
        // The plan of this round is kept here and no file is written from a task of the runtime; the workers' own plans
        //  stand on disk, the forgets it asked for are owed to the round after this one, and that round writes them.
        TheAnswerOfAJobAwaited::NotRun => {
            crate::service_warn!("the plan of this round's pushes was not written, because the system gave no thread for it; the round keeps it and the next round writes it");
            apply_without_writing(kept, asked)
        }
        // The job started on another thread or gave no answer within its ceiling, so what it wrote is not known; the list
        //  keeps the round's changes, which is what the loop's wake for these directories reads, and names each branch whose
        //  forget it cannot prove so that a later round writes it.
        TheAnswerOfAJobAwaited::NoAnswer => {
            crate::service_warn!("the plan of this round's pushes gave no answer, so whether the journal holds it is not known; the round keeps it and the next round writes it");
            apply_without_writing(kept, asked)
        }
    };
}

/// What:
///  `changes` applied to `retries` without writing the journal.
/// Why:
///  A task of the runtime may not write storage,
/// and the worker of each plan already wrote the file it promised;
/// this keeps the list the next wake reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// retries = applyWithoutWriting(retries, changes)
/// ```
fn apply_without_writing(mut retries: PushRetries, changes: Vec<RetryChange>) -> PushRetries {
    retries.apply_without_writing(changes);
    return retries;
}

/// `apply_awaiting`'s work,
/// as the job of the pool takes it.
fn apply_in_a_job((mut retries, changes): (PushRetries, Vec<RetryChange>)) -> PushRetries {
    retries.apply(changes);
    return retries;
}

/// What:
///  The directories a round drains,
/// each once,
/// in the order they were first named,
/// and the handlers of that round with
///  the review credentials its wakes carried.
/// Why:
///   Both faces of a round name their directories and their handlers this way,
/// and the credential is held only for the reviews of
///  the round that brought it:
/// a service that restarts before the round runs has none,
/// and its reviews record the plain "no
///  key" outcome.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [directories, round] = roundOfWakes(wakes, handlers);
/// ```
fn round_of_wakes(wakes: Vec<Wake>, handlers: &GitHandlers) -> (Vec<PathBuf>, Arc<GitHandlers>) {
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
    return (directories, Arc::new(handlers.for_round(keys)));
}

/// What:
///  Hand each directory what the plan has due for it now,
/// as the owned value its worker takes.
/// Why:
///   The plan is the caller's and is changed here on the caller's thread only,
/// so the workers share nothing and no lock is
///  taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const assignments = planTheRound(directories, retries, takenAt);
/// ```
fn plan_the_round(directories: &[PathBuf], retries: &mut PushRetries, taken_at: Instant) -> Vec<Assignment> {
    let first: Duration = retries.first();
    let mut assignments: Vec<Assignment> = Vec::new();
    for directory in directories {
        assignments.push(Assignment {
            directory: directory.clone(),
            retrying: Retrying { first: Some(first), due: retries.take_due(directory.as_path(), taken_at), plans: retries.plans().map(Path::to_path_buf) },
        });
    }
    return assignments;
}

/// What:
///  The directories a round named:
/// those it drained,
/// then those a drain planned a push under (the common directory of a bare
///  repository),
/// so that their wake is not lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = namedOf(directories, answer.changes);
/// ```
fn named_of(directories: &[PathBuf], changes: &[RetryChange]) -> Vec<PathBuf> {
    let mut named: Vec<PathBuf> = directories.to_vec();
    for change in changes {
        if let RetryChange::Plan { git_dir, .. } = change
            && !named.contains(git_dir)
        {
            named.push(git_dir.clone());
        }
    }
    return named;
}

/// What:
///  When to look at each directory again,
/// whichever is first of its leases and its plan.
/// Why:
///   The round's waits name the repositories whose claims another instance holds,
/// and the plan names those a push is due for;
///  the earliest of the two is the one wake that serves both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// askedOf(named, answer.waits, retries, appliedAt)
/// ```
fn asked_of(named: Vec<PathBuf>, waits: &[(PathBuf, Duration)], retries: &PushRetries, applied_at: Instant) -> Vec<(PathBuf, Duration)> {
    let mut asked: Vec<(PathBuf, Duration)> = Vec::new();
    for directory in named {
        // The first lease wait the round reported for this directory.
        let mut lease: Option<Duration> = None;
        for (named_directory, wait) in waits {
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
    let (directories, round): (Vec<PathBuf>, Arc<GitHandlers>) = round_of_wakes(wakes, handlers);
    return run_planned_round(host, directories.as_slice(), &round, retries, now);
}

/// What:
///  `run_round` awaited by a caller that is itself a task of the runtime:
/// it gives the plan back with the repositories to look at again,
///  because the plan has one owner at a time and that owner is now the round's caller.
/// Why:
///   The loop of the after-commit work is a task,
/// and a task may not enter the runtime a synchronous face enters;
/// the plan it
///  owns travels into the round's awaited face and comes back with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [retries, asked] = await runRoundAwaiting(wakes, handlers, retries, now);
/// ```
pub async fn run_round_awaiting(wakes: Vec<Wake>, handlers: &GitHandlers, retries: PushRetries, now: &ClaimTime) -> (PushRetries, Vec<(PathBuf, Duration)>) {
    return run_round_awaiting_on(&HostResources::read(), wakes, handlers, retries, now).await;
}

/// `run_round_awaiting` with the host's resources chosen by the caller,
/// so a control states a host of any size.
pub async fn run_round_awaiting_on(
    host: &HostResources,
    wakes: Vec<Wake>,
    handlers: &GitHandlers,
    retries: PushRetries,
    now: &ClaimTime,
) -> (PushRetries, Vec<(PathBuf, Duration)>) {
    let (directories, round): (Vec<PathBuf>, Arc<GitHandlers>) = round_of_wakes(wakes, handlers);
    return run_planned_round_awaiting(host, directories.as_slice(), &round, retries, now).await;
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
    let assignments: Vec<Assignment> = plan_the_round(directories, retries, taken_at);
    let answered: RoundAnswer = drain_directories(&RoundDrain { directories: assignments.as_slice(), handlers: Arc::clone(handlers), now, host });
    let named: Vec<PathBuf> = named_of(directories, answered.changes.as_slice());
    retries.apply(answered.changes);
    let applied_at: Instant = Instant::now();
    let asked: Vec<(PathBuf, Duration)> = asked_of(named, answered.waits.as_slice(), retries, applied_at);
    return asked;
}

/// What:
///  `run_planned_round` awaited by a caller that is itself a task of the runtime:
/// the lanes are awaited rather than entered,
/// the plan's own writes are jobs of the blocking pool,
///  and the plan comes back to its owner with the repositories to look at again.
/// Why:
///   A task may not enter the runtime,
/// so it awaits the round's lanes and the plan's writes;
/// a lane that stopped before its last
///  directory is said here,
/// as the synchronous face says it,
/// because a caller told nothing reads the silence as a refresh that
///  covered every directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [answer, stopped] = await drainDirectoriesAwaiting(...); retries = await applyAwaiting(retries, answer.changes); return [retries, askedOf(...)];
/// ```
pub async fn run_planned_round_awaiting<H: WorkHandlers + Sync + Send + 'static>(
    host: &HostResources,
    directories: &[PathBuf],
    handlers: &Arc<H>,
    mut retries: PushRetries,
    now: &ClaimTime,
) -> (PushRetries, Vec<(PathBuf, Duration)>) {
    // A child a stop left behind and the kernel has since released is collected here, so the service keeps no zombie for a
    //  process it abandoned.
    let _collected: usize = crate::stop_of_a_push_and_sweep_of_its_locks::reap_abandoned_children();
    let taken_at: Instant = Instant::now();
    let assignments: Vec<Assignment> = plan_the_round(directories, &mut retries, taken_at);
    let (answered, stopped): (RoundAnswer, Option<OrderedRunError>) =
        drain_directories_awaiting(&RoundDrain { directories: assignments.as_slice(), handlers: Arc::clone(handlers), now, host }).await;
    if let Some(reason) = stopped {
        crate::service_warn!("the round of the service's work did not finish: {reason}");
    }
    let named: Vec<PathBuf> = named_of(directories, answered.changes.as_slice());
    retries = apply_awaiting(retries, answered.changes).await;
    let applied_at: Instant = Instant::now();
    let asked: Vec<(PathBuf, Duration)> = asked_of(named, answered.waits.as_slice(), &retries, applied_at);
    return (retries, asked);
}

/// Controls for the plan of the pushes waiting for a new try stay out of the release executable.
#[cfg(test)]
#[path = "service_work_run_round_tests.rs"]
mod tests;
