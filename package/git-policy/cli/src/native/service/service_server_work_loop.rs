//! What:
//!  The loop of the commit service's work thread: it waits for wakes and due retries, runs a round of repository work for them,
//!  and keeps the service from exiting idle while a repository waits to be looked at again.
//! Why:
//!  The loop that lands commits (`serve` in `service_server.rs`) and this loop run side by side; separate files keep each
//!  one readable, and this one does not use the other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for await (const dir of wakesOrDueRetries) await drain(dir, handlers)
//! ```

pub use crate::service_server_socket_binding::{Bind, INHERITED_DESCRIPTOR, bind, socket_passed_in};
use crate::claim_time::ClaimTime;
use crate::service_retry::RetrySchedule;
use crate::service_claim_watch::ClaimWatch;
use crate::push_retry::PushRetries;
use crate::service_work::{GitHandlers, Wake};
use crate::service_work_run_round::run_round;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::path::PathBuf;
use std::time::{Duration, Instant};
/// What:
///  Runs the after-commit work of each repository named, one repository at a time, folding wakes that arrive while it
///  works into the next round, and looks again at a repository whose claims another instance held when their lease is over.
/// Why:
///   Draining claims everything pending, so a burst of wakes for one repository is one pass. The instance that held a claim
///  may have died, and nothing but this loop would then wake the work: it waits for a wake no longer than the first lease still
///  running, and the service counts itself busy meanwhile, so it does not exit idle with work claimed by a dead holder. A
///  service that has just started has had no wake yet, so the loop looks at every repository an earlier service of this instance
///  was woken for (`ClaimWatch`), after the schedule's slack, and the claims or pending work it finds are run or waited for as
///  after any wake: a kill is followed by the work being run with no commit needed. The same
///  holds for a push the service stopped: the loop owns the plan of its new tries (`PushRetries`), each round hands the workers
///  what is due and applies what they ask, and the wake for the earliest plan is kept in the same schedule as the lease wakes, so
///  the service neither forgets the plan nor exits idle while a try is pending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const dir of wakesOrDueRetries) await drain(dir, handlers)
/// ```
pub fn work_loop(receiver: &Receiver<Wake>, handlers: &GitHandlers, busy: &AtomicUsize, mut push_retries: PushRetries, mut watch: ClaimWatch) {
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
        let first: Option<Wake> = match retries.next_wait(Instant::now()) {
            None => match receiver.recv() {
                Ok(wake) => Some(wake),
                Err(_) => return,
            },
            Some(wait) => match receiver.recv_timeout(wait) {
                Ok(wake) => Some(wake),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => return,
            },
        };
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
            for directory in watch.retry() {
                wakes.push((directory, None));
            }
        }
        let mut ran: Vec<PathBuf> = Vec::new();
        for (directory, _) in &wakes {
            ran.push(directory.clone());
        }
        // Written down before the first claim of a repository, so that a service killed while it holds the claim is followed by one
        //  that knows where to look. This thread does it, never the one that lands commits.
        for directory in &ran {
            watch.remember(directory.as_path());
        }
        let asked: Vec<(PathBuf, Duration)> = run_round(wakes, handlers, &mut push_retries, &ClaimTime::read());
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
