//! What:
//!  The loop of the commit service's after-commit work as a task of the runtime,
//! and the storage steps only it makes:
//! it waits for wakes and due retries,
//! looks at the repositories a record waits in even when no wake came,
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
//! The steps that read or write a file are jobs of the pool here
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

use super::service_server_work_loop_awaiting_storage_steps::{
    look_awaiting,
    remember_awaiting,
    restore_the_plan,
    retry_awaiting,
};
use crate::claim_time::ClaimTime;
use crate::push_retry::PushRetries;
use crate::service_claim_watch::{
    ClaimWatch,
    LOOK_AGAIN_AFTER,
};
use crate::service_queue_received_on_the_runtime::{
    Received,
    receive_for_at_most_awaiting,
};
use crate::service_retry::RetrySchedule;
use crate::service_work::{
    GitHandlers,
    Wake,
};
use crate::service_work_run_round::run_round_awaiting;
use crate::service_work_slow_lane::SlowLane;
use crate::stop_of_the_service_noticed_by_its_children::is_stopping;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
use std::time::{
    Duration,
    Instant,
};
use tokio::sync::mpsc::UnboundedReceiver;

/// What:
///  Keep the watch and the plan in the schedule for what they still owe.
/// Why:
///   Both are a directory the loop must come back to by itself,
/// and both are taken out of the schedule when they are due,
/// so a
///  round that ran nothing must put them back as a round that ran something does,
/// or a note that could not be written and a plan
///  that could not be read would wait for the next wake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepWhatIsOwed(retries, watch, plan) // re-add the store's and the plan's own directories after their pauses
/// ```
fn keep_what_the_watch_and_the_plan_owe(
    retries: &mut RetrySchedule,
    watch: &ClaimWatch,
    plan: &PushRetries,
    watch_scheduled: &mut bool,
    plan_scheduled: &mut bool,
) {
    if !*watch_scheduled
        && let Some(key) = watch.key()
        && watch.needs_retry()
    {
        retries.replace(&[], vec![(key, watch.pause(plan.first()))], Instant::now());
        *watch_scheduled = true;
    }
    if !*plan_scheduled
        && plan.needs_restore()
        && let Some(directory) = plan.plans()
    {
        retries.replace(
            &[],
            vec![(directory.to_path_buf(), plan.restore_pause())],
            Instant::now(),
        );
        *plan_scheduled = true;
    }
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
/// A wake may also never come at all,
/// while the service runs and the record it would have run waits:
/// so the wait below never
///  outlasts `LOOK_AGAIN_AFTER`,
/// and each time it ends there the loop asks its journal's notes which repositories hold a record in
///  `pending`
/// (`look_awaiting`).
/// That look keeps nothing alive:
/// a service with nothing to do still ends at its idle bound,
/// and a
///  look that found nothing runs no round,
/// so the clock alone never makes the service work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const dir of wakesOrDueRetries) { lane.ask(dir); await drainPushes(dir) }
/// ```
pub async fn work_loop_awaiting(
    receiver: &mut UnboundedReceiver<Wake>,
    handlers: &GitHandlers,
    busy: &Arc<AtomicUsize>,
    mut push_retries: PushRetries,
    mut watch: ClaimWatch,
) {
    // The work that can take minutes runs beside this loop, so no step of it stands between a landing and its push
    //  (`service_work_slow_lane.rs`). The lane counts itself busy, so the service does not exit while it holds a claim.
    let slow: SlowLane = SlowLane::start(handlers, Arc::clone(busy));
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
    // A plan a start could not read is read again after a pause, and the service stays up until it is: the pushes an earlier
    //  service promised to try are in its files, and nothing else would look for them.
    let mut plan_scheduled: bool = false;
    if push_retries.needs_restore()
        && let Some(directory) = push_retries.plans()
    {
        restored.push((directory.to_path_buf(), push_retries.restore_pause()));
        plan_scheduled = true;
    }
    retries.replace(&[], restored, started);
    if !retries.is_empty() {
        busy.fetch_add(1, Ordering::SeqCst);
        counted_waiting = true;
    }
    // What: When the next look at the notes is due.
    // Why:  A record that exists and is due is found by this service whether or not anything woke it, so the wait below
    //       never outlasts this moment, and the look beside it asks which repositories hold a record no claim took
    //       (`service_claim_watch.rs`, `LOOK_AGAIN_AFTER`). The look is not work of its own: it keeps nothing alive, so a
    //       service with nothing to do still ends at its idle bound.
    let mut look_at: Instant = Instant::now() + LOOK_AGAIN_AFTER;
    loop {
        let now: Instant = Instant::now();
        let until_look: Duration = look_at.saturating_duration_since(now);
        let until_a_wake: Option<Duration> = match retries.next_wait(now) {
            Some(due) => Some(due.min(until_look)),
            None => Some(until_look),
        };
        let first: Option<Wake> = match receive_for_at_most_awaiting(receiver, until_a_wake).await {
            Received::Item(wake) => Some(wake),
            Received::Timeout => None,
            Received::Closed => return,
        };
        // After a stop is recorded no round starts: the claims this task holds run out and the next service runs them again, and
        //  the wakes that still arrive are read and dropped until the shutdown ends the process.
        if is_stopping() {
            while !matches!(
                receive_for_at_most_awaiting(receiver, None).await,
                Received::Closed
            ) {}
            return;
        }
        busy.fetch_add(1, Ordering::SeqCst);
        // The wake that ended the wait, then every wake already queued behind it.
        let mut wakes: Vec<Wake> = first.into_iter().collect();
        while let Ok(queued_wake) = receiver.try_recv() {
            wakes.push(queued_wake);
        }
        let mut watch_due: Vec<PathBuf> = Vec::new();
        let mut plan_due: bool = false;
        let mut due: Vec<PathBuf> = Vec::new();
        for directory in retries.take_due(Instant::now()) {
            if watch.key().as_ref() == Some(&directory) {
                watch_due.push(directory);
            } else if push_retries.plans() == Some(directory.as_path()) {
                // The plan's own directory is never a repository to drain, whatever the list has already read, so its wake is
                //  taken here and a read that has nothing left to do says so by finding nothing.
                plan_due = true;
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
        if plan_due {
            // The pushes a second read of the plan found are looked at as if their branches were due.
            plan_scheduled = false;
            let (back, found): (PushRetries, Vec<PathBuf>) = restore_the_plan(push_retries).await;
            push_retries = back;
            for directory in found {
                wakes.push((directory, None));
            }
        }
        // What: The look at the notes, when its bound is over, and every repository whose records wait.
        // Why:  A record written and never announced stays in `pending` until something looks, and a running service is the
        //       only thing that can look.
        if Instant::now() >= look_at {
            let (back, waiting): (ClaimWatch, Vec<PathBuf>) = look_awaiting(watch).await;
            watch = back;
            for directory in waiting {
                // A wake already named this repository, and its round takes the same records: one drain, not two.
                let mut named_already: bool = false;
                for (named, _) in &wakes {
                    if named == &directory {
                        named_already = true;
                    }
                }
                if !named_already {
                    wakes.push((directory, None));
                }
            }
            // A look that leaves the store owing a read is asked for less often until its pause runs out, so a journal that
            //  stopped answering cannot fill the blocking pool with looks of it.
            let pause: Duration = match watch.needs_retry() {
                true => watch.pause(push_retries.first()).max(LOOK_AGAIN_AFTER),
                false => LOOK_AGAIN_AFTER,
            };
            look_at = Instant::now() + pause;
        }
        let mut ran: Vec<PathBuf> = Vec::new();
        for (directory, _) in &wakes {
            ran.push(directory.clone());
        }
        if !wakes.is_empty() {
            // Written down before the first claim of a repository, so that a service killed while it holds the claim is followed by one
            //  that knows where to look. This task does it, never the one that lands commits.
            for directory in &ran {
                watch = remember_awaiting(watch, directory.as_path()).await;
            }
            // The same repositories are looked at for the kinds that can take minutes, beside the round below: the round claims the
            //  pushes alone, so a person's push is taken however long a trash, a prewarm, an upkeep or a review then runs.
            for wake in &wakes {
                slow.ask(wake.clone());
            }
            let (back, asked): (PushRetries, Vec<(PathBuf, Duration)>) =
                run_round_awaiting(wakes, handlers, push_retries, &ClaimTime::read()).await;
            push_retries = back;
            // A directory the round named for its plan alone (the common directory of a bare repository) replaces its old entry too.
            for (directory, _) in &asked {
                ran.push(directory.clone());
            }
            retries.replace(ran.as_slice(), asked, Instant::now());
            // Whether the watch and the plan still owe a read is asked after the round, which is what changed them.
            keep_what_the_watch_and_the_plan_owe(
                &mut retries,
                &watch,
                &push_retries,
                &mut watch_scheduled,
                &mut plan_scheduled,
            );
        } else {
            // Nothing to drain: only the look's own bound went by, so what the watch and the plan still owe is scheduled here
            //  exactly as a round schedules it, and no round is run for a clock that moved.
            keep_what_the_watch_and_the_plan_owe(
                &mut retries,
                &watch,
                &push_retries,
                &mut watch_scheduled,
                &mut plan_scheduled,
            );
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

/// Controls for the notes the loop keeps for a restart stay out of the release executable.
#[cfg(test)]
#[path = "service_server_work_loop_awaiting_tests.rs"]
mod tests;
