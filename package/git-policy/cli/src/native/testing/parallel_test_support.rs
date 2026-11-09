//! What:
//!  A point where several threads must all arrive,
//! for controls that prove work really overlapped.
//! Why:
//!  A bare barrier hangs a run that is serial;
//! this one gives up after a stated patience and says so,
//! so a serial run fails
//!  the control instead of stalling it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const point = new Rendezvous(4, 5_000);
//! const everyoneCame = await point.arrive();
//! ```

use std::sync::{Condvar, Mutex};
use std::time::Duration;

/// How long an arrival waits for the others when the control expects them to come.
pub(crate) const PATIENCE: Duration = Duration::from_secs(10);

/// How long the control for a serial run waits at a point it can never pass.
pub(crate) const SERIAL_PATIENCE: Duration = Duration::from_millis(200);

/// A point where a number of threads must all arrive:
/// it can be passed only if they overlap in time.
pub(crate) struct Rendezvous {
    /// How many threads have arrived.
    arrived: Mutex<usize>,
    /// Woken when one arrives.
    changed: Condvar,
    /// How many must arrive.
    target: usize,
    /// How long an arrival waits for the others.
    patience: Duration,
}

impl Rendezvous {
    /// A point for `target` threads.
    pub(crate) fn new(target: usize) -> Rendezvous {
        return Rendezvous { arrived: Mutex::new(0), changed: Condvar::new(), target, patience: PATIENCE };
    }

    /// A point for `target` threads whose arrivals give up after `patience`.
    pub(crate) fn impatient(target: usize, patience: Duration) -> Rendezvous {
        return Rendezvous { patience, ..Rendezvous::new(target) };
    }

    /// Arrive and wait for the others;
    /// false when they did not all come within the patience,
    /// which is what a serial run sees.
    pub(crate) fn arrive(&self) -> bool {
        let mut count = self.arrived.lock().expect("lock");
        *count += 1;
        self.changed.notify_all();
        return self.wait_while_short(count);
    }

    /// Wait without arriving until the target number have arrived;
    /// false after the patience.
    pub(crate) fn wait_for_target(&self) -> bool {
        let count = self.arrived.lock().expect("lock");
        return self.wait_while_short(count);
    }

    /// Count an arrival without waiting for the others.
    pub(crate) fn arrive_without_waiting(&self) {
        let mut count = self.arrived.lock().expect("lock");
        *count += 1;
        self.changed.notify_all();
    }

    /// Wait on a held count until it reaches the target;
    /// false when the patience ran out first.
    fn wait_while_short(&self, count: std::sync::MutexGuard<'_, usize>) -> bool {
        let (guard, timeout) = self.changed.wait_timeout_while(count, self.patience, |seen: &mut usize| { return *seen < self.target; }).expect("wait");
        drop(guard);
        return !timeout.timed_out();
    }
}

/// How many places of the process's bound on blocking jobs the place control holds:
/// the one.
pub(crate) const ONE_PLACE: usize = 1;

/// How long the job of the place control holds the only place,
/// so the work it measures has time to reach its own job.
/// A second is far more than an item needs to start when it takes no place at all
/// (the plant of `worktree/worktree_convert_parallel_tests.rs` starts at once),
///  and it bounds the one way these controls can pass without proving anything:
/// work that reached its own job later than this would find the place given back.
pub(crate) const HELD_BY_ANOTHER_JOB: Duration = Duration::from_millis(1000);

/// What:
///  Holds the process's one place on blocking jobs on a task of the runtime,
///  and gives it back after `HELD_BY_ANOTHER_JOB`.
/// Why:
///   A control can then show that the work it measures waits for a place,
///  which only a job of the blocking pool does;
///  work that ran on a worker of the runtime instead would start at once.
/// The place is known to be taken before this returns,
///  so the work that follows either waits for it or did not take one.
/// The bound is held for the life of the process (`hold_blocking_jobs_to`),
///  which is why this runs in a test process of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [givenBack, finished] = holdTheOnlyPlace();
/// ```
pub(crate) fn hold_the_only_place() -> (std::sync::Arc<std::sync::atomic::AtomicBool>, std::sync::mpsc::Receiver<()>) {
    use std::sync::atomic::{AtomicBool, Ordering};
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(ONE_PLACE);
    let given_back: std::sync::Arc<AtomicBool> = std::sync::Arc::new(AtomicBool::new(false));
    let (taken, has_taken) = std::sync::mpsc::channel::<()>();
    let (done, finished) = std::sync::mpsc::channel::<()>();
    let flag: std::sync::Arc<AtomicBool> = std::sync::Arc::clone(&given_back);
    let holder = crate::lazy_runtime::spawn_on_runtime(async move {
        let _outcome = crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || {
            let _told = taken.send(());
            std::thread::sleep(HELD_BY_ANOTHER_JOB);
            flag.store(true, Ordering::SeqCst);
            let _sent = done.send(());
        })
        .await;
    });
    assert!(holder.is_some(), "the runtime has workers, so a place of the blocking pool can be held");
    assert!(has_taken.recv_timeout(PATIENCE).is_ok(), "the job that holds the only place never started");
    return (given_back, finished);
}
