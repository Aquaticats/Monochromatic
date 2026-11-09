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

/// What:
///  Holds the process's one place on blocking jobs on a task of the runtime,
///  and gives it back when a job of the work under test asks for a place.
/// Why:
///   A control can then show that the work it measures waits for a place,
///  which only a job of the blocking pool does;
///  work that ran on a worker of the runtime instead would start at once and never ask.
/// A clock would not do:
///  a holder that gave the place back after a fixed wait would hand it over before work that
/// bypasses the bound ever tried,
///  so that work would read `given_back` as true and pass.
/// The answer on `finished` is what the work under test asked:
///  true when a job of it was waiting for a place,
/// which is the proof that it reached the process's bound.
/// The place is known to be taken before this returns,
///  so the work that follows either waits for it or did not take one.
/// The bound is held for the life of the process (`hold_blocking_jobs_to`),
///  which is why this runs in a test process of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [givenBack, finished] = holdTheOnlyPlace(); const asked = finished.recv(); // true when the work asked
/// ```
pub(crate) fn hold_the_only_place() -> (std::sync::Arc<std::sync::atomic::AtomicBool>, std::sync::mpsc::Receiver<bool>) {
    use std::sync::atomic::{AtomicBool, Ordering};
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(ONE_PLACE);
    let given_back: std::sync::Arc<AtomicBool> = std::sync::Arc::new(AtomicBool::new(false));
    let (taken, has_taken) = std::sync::mpsc::channel::<()>();
    let (done, finished) = std::sync::mpsc::channel::<bool>();
    let flag: std::sync::Arc<AtomicBool> = std::sync::Arc::clone(&given_back);
    let holder = crate::lazy_runtime::spawn_on_runtime(async move {
        let _outcome = crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || {
            let _told = taken.send(());
            // The place goes back on an ask from the work under test, never merely because time passed.
            let asked: bool = crate::blocking_jobs_within_the_budget::jobs_waiting_for_a_place::wait_until_a_job_waits(PATIENCE);
            flag.store(true, Ordering::SeqCst);
            let _sent = done.send(asked);
        })
        .await;
    });
    assert!(holder.is_some(), "the runtime has workers, so a place of the blocking pool can be held");
    assert!(has_taken.recv_timeout(PATIENCE).is_ok(), "the job that holds the only place never started");
    return (given_back, finished);
}
