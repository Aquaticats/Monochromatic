//! What:
//!  The loop of the commit service's after-commit work as a task of the runtime,
//! and the two storage steps only it makes:
//! it waits for wakes and due retries,
//! runs a round of repository work for them,
//!  and keeps the service from exiting idle while a repository waits to be looked at again.
//! Why:
//!  A loop that is a thread holds one of the process's threads for the life of the service,
//! and everything it runs is awaited:
//!  its waits are the runtime's own (`service_queue_received_on_the_runtime.rs`),
//! its rounds are lanes of tasks (`service_round.rs`),
//! and the records of both are jobs of the blocking pool
//!  (`service_work_drain_pending_claims.rs`).
//! So the loop is a task as well.
//! The two steps that write a file are jobs of the pool here
//!  for the same reason the drain's storage steps are:
//! a worker that waited on a disk,
//! a network file system or a FUSE mount would hold a thread the runtime needs.
//!  The watch and the plan each have one owner at a time,
//! so a job takes its value and hands it back,
//! and nothing locks it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for await (const dir of wakesOrDueRetries) await drain(dir, handlers)
//! ```

use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::claim_time::ClaimTime;
use crate::push_retry::PushRetries;
use crate::service_claim_watch::ClaimWatch;
use crate::service_queue_received_on_the_runtime::{Received, receive_for_at_most_awaiting};
use crate::service_retry::RetrySchedule;
use crate::service_work::{GitHandlers, Wake};
use crate::service_work_run_round::run_round_awaiting;
use crate::stop_of_the_service_noticed_by_its_children::is_stopping;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;

/// What:
///  Write `git_dir` down as a repository to look at when this service starts again,
/// as a job of the process's blocking pool,
///  and give the watch back with the note in it.
/// Why:
///   The note is a file in the instance's journal,
/// so writing it on a worker would hold a thread the runtime needs while storage waits.
/// The watch has one
///  owner at a time,
/// so the job takes it and hands it back,
/// and nothing locks it.
/// A job the system refuses a thread for makes the write
///  on this task instead
/// (`run_a_job_or_in_turn`),
/// so a note is not lost to a refusal.
/// A watch kept in memory only writes nothing,
/// so it is not a job at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = await rememberAwaiting(watch, gitDir)
/// ```
async fn remember_awaiting(watch: ClaimWatch, git_dir: &Path) -> ClaimWatch {
    if watch.key().is_none() {
        let mut kept: ClaimWatch = watch;
        kept.remember(git_dir);
        return kept;
    }
    let unchanged: ClaimWatch = watch.clone();
    return match run_a_job_or_in_turn((watch, git_dir.to_path_buf()), remember_in_a_job).await {
        Ok(back) => back,
        // Nothing was written and the watch is as it was, so the next wake writes this directory down; the notes it
        //  already holds stand.
        Err(reason) => {
            crate::service_warn!("{} could not be written down for a restart: {reason}", git_dir.display());
            unchanged
        }
    };
}

/// `remember_awaiting`'s work,
/// as the job of the pool takes it.
fn remember_in_a_job((mut watch, git_dir): (ClaimWatch, PathBuf)) -> ClaimWatch {
    watch.remember(git_dir.as_path());
    return watch;
}

/// What:
///  Try again the notes that could not be written or read,
/// as a job of the process's blocking pool,
/// and give the watch back with the directories this made known.
/// Why:
///   The steps are a write and a reading of the journal's files,
/// so they are storage like the write above,
/// and the watch is the job's and comes back.
/// A watch kept in memory only has nothing to
///  try again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [watch, found] = await retryAwaiting(watch)
/// ```
async fn retry_awaiting(watch: ClaimWatch) -> (ClaimWatch, Vec<PathBuf>) {
    if watch.key().is_none() {
        let mut kept: ClaimWatch = watch;
        let found: Vec<PathBuf> = kept.retry();
        return (kept, found);
    }
    let unchanged: ClaimWatch = watch.clone();
    return match run_a_job_or_in_turn(watch, retry_in_a_job).await {
        Ok((back, found)) => (back, found),
        // Nothing was read or written, so the pause for this failure is not grown either: the next round of the schedule
        //  tries the same notes again.
        Err(reason) => {
            crate::service_warn!("the notes this service keeps for a restart could not be tried again: {reason}");
            (unchanged, Vec::new())
        }
    };
}

/// `retry_awaiting`'s work,
/// as the job of the pool takes it.
fn retry_in_a_job(mut watch: ClaimWatch) -> (ClaimWatch, Vec<PathBuf>) {
    let found: Vec<PathBuf> = watch.retry();
    return (watch, found);
}

/// What:
///  Runs the after-commit work of each repository named,
/// one repository at a time,
/// folding wakes that arrive while it
///  works into the next round,
/// and looks again at a repository whose claims another instance held when their lease is over,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   This is the body of `work_loop`
/// (`service_server_work_loop.rs`),
/// which is the face a caller outside the runtime enters once,
/// and the body of the task the service
///  starts.
/// Draining claims everything pending,
/// so a burst of wakes for one repository is one pass.
/// The instance that held a claim
///  may have died,
/// and nothing but this loop would then wake the work:
/// it waits for a wake no longer than the first lease still
///  running,
/// and the service counts itself busy meanwhile,
/// so it does not exit idle with work claimed by a dead holder.
/// A
///  service that has just started has had no wake yet,
/// so the loop looks at every repository an earlier service of this instance
///  was woken for (`ClaimWatch`),
/// after the schedule's slack,
/// and the claims or pending work it finds are run or waited for as
///  after any wake:
/// a kill is followed by the work being run with no commit needed.
/// The same
///  holds for a push the service stopped:
/// the loop owns the plan of its new tries (`PushRetries`),
/// each round hands the workers
///  what is due and applies what they ask,
/// and the wake for the earliest plan is kept in the same schedule as the lease wakes,
/// so
///  the service neither forgets the plan nor exits idle while a try is pending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const dir of wakesOrDueRetries) await drain(dir, handlers)
/// ```
pub async fn work_loop_awaiting(
    receiver: &mut UnboundedReceiver<Wake>,
    handlers: &GitHandlers,
    busy: &AtomicUsize,
    mut push_retries: PushRetries,
    mut watch: ClaimWatch,
) {
    let mut retries: RetrySchedule = RetrySchedule::default();
    let mut counted_waiting: bool = false;
    // A plan that is already there when the loop starts (one rebuilt from disk) is woken for like one made by a round.
    let started: Instant = Instant::now();
    let mut restored: Vec<(PathBuf, Duration)> = Vec::new();
    for directory in push_retries.directories() {
        if let Some(wait) = push_retries.next_wait(directory.as_path(), started) {
            restored.push((directory, wait));
        }
    }
    // A repository an earlier service was woken for may hold pending work or a claim its holder left: it is looked at without a
    //  wake, at once. A push plan that names the same repository does not stand in for the look: the plan is due when the push's
    //  own pause is over, which may be long after a claim has run out its lease, and the round that follows the look asks for the
    //  plan's wake again.
    for directory in watch.directories() {
        restored.push((directory, Duration::ZERO));
    }
    // Notes that could not be read or written are tried again after a pause, and the service stays up until they are.
    let mut watch_scheduled: bool = false;
    if let Some(key) = watch.key()
        && watch.needs_retry()
    {
        restored.push((key, watch.pause(push_retries.first())));
        watch_scheduled = true;
    }
    retries.replace(&[], restored, started);
    if !retries.is_empty() {
        busy.fetch_add(1, Ordering::SeqCst);
        counted_waiting = true;
    }
    loop {
        let first: Option<Wake> = match receive_for_at_most_awaiting(receiver, retries.next_wait(Instant::now())).await {
            Received::Item(wake) => Some(wake),
            Received::Timeout => None,
            Received::Closed => return,
        };
        // After a stop is recorded no round starts: the claims this task holds run out and the next service runs them again, and
        //  the wakes that still arrive are read and dropped until the shutdown ends the process.
        if is_stopping() {
            while !matches!(receive_for_at_most_awaiting(receiver, None).await, Received::Closed) {}
            return;
        }
        busy.fetch_add(1, Ordering::SeqCst);
        // The wake that ended the wait, then every wake already queued behind it.
        let mut wakes: Vec<Wake> = first.into_iter().collect();
        while let Ok(queued_wake) = receiver.try_recv() {
            wakes.push(queued_wake);
        }
        let mut watch_due: Vec<PathBuf> = Vec::new();
        let mut due: Vec<PathBuf> = Vec::new();
        for directory in retries.take_due(Instant::now()) {
            if watch.key().as_ref() == Some(&directory) {
                watch_due.push(directory);
            } else {
                due.push(directory);
            }
        }
        for directory in due {
            wakes.push((directory, None));
        }
        if !watch_due.is_empty() {
            // The repositories a retried read of the notes found are looked at as if they had been woken for.
            watch_scheduled = false;
            let (back, found) = retry_awaiting(watch).await;
            watch = back;
            for directory in found {
                wakes.push((directory, None));
            }
        }
        let mut ran: Vec<PathBuf> = Vec::new();
        for (directory, _) in &wakes {
            ran.push(directory.clone());
        }
        // Written down before the first claim of a repository, so that a service killed while it holds the claim is followed by one
        //  that knows where to look. This task does it, never the one that lands commits.
        for directory in &ran {
            watch = remember_awaiting(watch, directory.as_path()).await;
        }
        let (back, asked): (PushRetries, Vec<(PathBuf, Duration)>) = run_round_awaiting(wakes, handlers, push_retries, &ClaimTime::read()).await;
        push_retries = back;
        // A directory the round named for its plan alone (the common directory of a bare repository) replaces its old entry too.
        for (directory, _) in &asked {
            ran.push(directory.clone());
        }
        retries.replace(ran.as_slice(), asked, Instant::now());
        if !watch_scheduled
            && let Some(key) = watch.key()
            && watch.needs_retry()
        {
            retries.replace(&[], vec![(key, watch.pause(push_retries.first()))], Instant::now());
            watch_scheduled = true;
        }
        // A repository waiting to be looked at again keeps the service from exiting idle, as running work does.
        if !retries.is_empty() && !counted_waiting {
            busy.fetch_add(1, Ordering::SeqCst);
            counted_waiting = true;
        } else if retries.is_empty() && counted_waiting {
            busy.fetch_sub(1, Ordering::SeqCst);
            counted_waiting = false;
        }
        busy.fetch_sub(1, Ordering::SeqCst);
    }
}
