//! The blocking budget:
//! a job's value comes back,
//! a panic gives its place back,
//! no more jobs run at once than the budget allows,
//! and a system that refuses every thread gives an error value.

use super::*;
use crate::lazy_runtime::run_on_runtime;
use std::sync::atomic::AtomicUsize;
use std::sync::{Condvar, Mutex};
use std::time::Duration;

/// How many jobs the bound's control starts at once,
/// three times the bound the control sets.
const BURST: usize = 6;

/// How many jobs hold themselves to at once in the bound's control.
const BOUND_HELD_TO: usize = 2;

/// The longest a control waits for a job that should not wait at all.
const PATIENCE: Duration = Duration::from_secs(30);

/// What:
///  How long the bound's control looks for a job beyond the bound once the bound is reached.
/// Why:
///  A job the bound does not hold is inside as soon as the blocking pool gives it a thread,
/// and a job of the burst holds its place until this control has counted the burst,
///  so this is a liveness guard with room to spare rather than a window the claim rests on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await counted.waitForOneMore(1_000); // false when the bound held
/// ```
const BEYOND_THE_BOUND_WINDOW: Duration = Duration::from_millis(1000);

/// What:
///  How many jobs are inside their work right now,
///  the most that ever were of them at one instant,
///  and whether the control has finished counting the burst.
/// Why:
///  The bound has to be witnessed by construction:
///  a job that slept for a fixed time let the burst finish before the bound was reached,
/// which made the peak an arrival order of six jobs and not a measurement of the bound.
///  Here every job holds its place until the control has counted the burst,
/// so the peak is the whole burst's peak,
/// and the place of a job is given back because the control read the count,
/// never because time passed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Counted { enter(): number; leave(): void; waitFor(live: number, patience: Duration): boolean; most(): number }
/// ```
struct Concurrency {
    /// How many jobs are inside now,
    /// and whether the control has counted the burst.
    state: Mutex<Held>,
    /// Woken whenever a job enters,
    /// leaves,
    /// or the control has counted the burst.
    changed: Condvar,
    /// The most that were inside together.
    most: AtomicUsize,
}

/// What the jobs and the control share under the lock of a `Concurrency`.
struct Held {
    /// How many jobs are inside their work right now.
    live: usize,
    /// Whether the control has read the peak,
    /// so every job of the burst may finish.
    counted: bool,
}

impl Concurrency {
    /// A count where nothing is running yet and nothing has been counted.
    fn new() -> Arc<Self> {
        return Arc::new(Concurrency { state: Mutex::new(Held { live: 0, counted: false }), changed: Condvar::new(), most: AtomicUsize::new(0) });
    }

    /// Count one job in and keep the most that were inside together.
    fn enter(&self) -> usize {
        let mut held = self.state.lock().expect("lock");
        held.live += 1;
        let now: usize = held.live;
        self.most.fetch_max(now, Ordering::SeqCst);
        self.changed.notify_all();
        return now;
    }

    /// Count one job out.
    fn leave(&self) {
        let mut held = self.state.lock().expect("lock");
        held.live = held.live.saturating_sub(1);
        self.changed.notify_all();
    }

    /// Wait until `wanted` jobs are inside together;
    /// false when they did not all come within `patience`.
    fn wait_for_live(&self, wanted: usize, patience: Duration) -> bool {
        let locked = self.state.lock().expect("lock");
        let (released, waited) = self.changed.wait_timeout_while(locked, patience, |held: &mut Held| { return held.live < wanted; }).expect("wait");
        drop(released);
        return !waited.timed_out();
    }

    /// Hold the job until the control has counted the burst;
    /// false only when the control never counted,
    /// which is a control that failed before it could.
    fn wait_until_counted(&self, patience: Duration) -> bool {
        let locked = self.state.lock().expect("lock");
        let (released, waited) = self.changed.wait_timeout_while(locked, patience, |held: &mut Held| { return !held.counted; }).expect("wait");
        drop(released);
        return !waited.timed_out();
    }

    /// Let every job of the burst finish,
    /// once the peak has been read.
    fn count_the_burst(&self) {
        let mut held = self.state.lock().expect("lock");
        held.counted = true;
        self.changed.notify_all();
    }

    /// The most that were inside together.
    fn most(&self) -> usize {
        return self.most.load(Ordering::SeqCst);
    }
}

/// A job that holds its place until the control has counted the burst and says how many were inside with it.
fn hold_a_place(shared: Arc<Concurrency>) -> impl FnOnce() -> usize + Send + 'static {
    return move || -> usize {
        let with_me: usize = shared.enter();
        let _counted: bool = shared.wait_until_counted(PATIENCE);
        shared.leave();
        return with_me;
    };
}

/// A job's value comes back from the blocking pool.
#[test]
fn a_job_gives_its_value() {
    let value: usize = run_on_runtime(run_blocking_job_within_the_budget(|| { return 41 + 1; })).expect("the runtime").expect("the job");
    assert_eq!(value, 42);
}

/// The budget is the process's own and says how many places it holds.
#[test]
fn the_budget_is_at_least_one_place_and_a_cap_holds_it_lower() {
    hold_blocking_jobs_to(BOUND_HELD_TO);
    assert_eq!(budgeted_blocking_jobs(), BOUND_HELD_TO, "the cap the process asked for is the bound in force");
}

/// No more jobs run at once than the budget allows,
/// however many are started,
/// and the budget is reached:
///  every job of the burst is inside its work and held there,
/// so the peak below is the peak of the whole burst and not its arrival order.
#[test]
fn no_more_jobs_run_at_once_than_the_budget_allows() {
    hold_blocking_jobs_to(BOUND_HELD_TO);
    let shared: Arc<Concurrency> = Concurrency::new();
    let burst_shared: Arc<Concurrency> = Arc::clone(&shared);
    // The burst runs on a thread of its own: the control has to count it while it runs, and a count taken after it is the
    //  order six jobs arrived in rather than the bound.
    let burst = std::thread::spawn(move || {
        return run_on_runtime(async move {
            let started = tokio::join!(
                run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&burst_shared))),
                run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&burst_shared))),
                run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&burst_shared))),
                run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&burst_shared))),
                run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&burst_shared))),
                run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&burst_shared))),
            );
            return [started.0, started.1, started.2, started.3, started.4, started.5];
        })
        .expect("the runtime");
    });
    // The bound is reached: the burst is three times it, so every place is taken by a job that holds it.
    assert!(shared.wait_for_live(BOUND_HELD_TO, PATIENCE), "the burst of {BURST} jobs never reached the budget of {BOUND_HELD_TO} places");
    // And it is not passed: only this control lets a job out of its work, so a job inside beyond the bound is still inside
    //  when the window ends.
    let beyond: bool = shared.wait_for_live(BOUND_HELD_TO + 1, BEYOND_THE_BOUND_WINDOW);
    let most: usize = shared.most();
    shared.count_the_burst();
    let outcome = burst.join().expect("the burst's thread");
    assert!(outcome.iter().all(Result::is_ok), "every job of the burst ran: {outcome:?}");
    assert!(!beyond, "the burst of {BURST} jobs held more than the budget of {BOUND_HELD_TO} places at once");
    assert_eq!(most, BOUND_HELD_TO, "the burst of {BURST} jobs held {most} places at once, and the budget is {BOUND_HELD_TO}");
}

/// A job that panics gives its place back,
/// so the next job does not wait for a place that nothing holds.
#[test]
fn a_job_that_panics_gives_its_place_back() {
    hold_blocking_jobs_to(1);
    let (refused, after) = run_on_runtime(async {
        let refused = run_blocking_job_within_the_budget(|| -> u8 { panic!("the job panicked") }).await;
        let after = tokio::time::timeout(PATIENCE, run_blocking_job_within_the_budget(|| { return 5_u8; })).await;
        return (refused, after);
    })
    .expect("the runtime");
    assert_eq!(refused, Err(BlockingJobError::Panicked));
    assert_eq!(after.expect("the place of a job that panicked was never given back"), Ok(5));
}

/// Where the system refuses every thread,
/// the jobs do not run,
/// each one says plainly that it did not run,
/// and the place of a refused job is given back so the next job is refused too instead of waiting for it.
/// A place held inside the job would be lost with it:
/// the pool queues the job before it asks for a thread,
/// and a refused
///  thread leaves it queued with nothing to run it.
#[test]
fn where_no_thread_can_start_the_jobs_fail_plainly_and_give_their_places_back() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "where_no_thread_can_start_the_jobs_fail_plainly_and_give_their_places_back") {
        return;
    }
    hold_blocking_jobs_to(1);
    let ran: Arc<std::sync::atomic::AtomicBool> = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag: Arc<std::sync::atomic::AtomicBool> = Arc::clone(&ran);
    let outcomes = run_on_runtime(async {
        let first = run_blocking_job_within_the_budget(move || { flag.store(true, Ordering::SeqCst); }).await;
        let second = tokio::time::timeout(PATIENCE, run_blocking_job_within_the_budget(|| { return 5_u8; })).await;
        return (first, second);
    })
    .expect("a current-thread runtime stands in where no thread can start");
    assert_eq!(outcomes.0, Err(BlockingJobError::NoThreadToStart), "the first job did not run");
    assert_eq!(
        outcomes.1.expect("the place of a job refused a thread was never given back"),
        Err(BlockingJobError::NoThreadToStart),
        "the second job ran although no thread can start"
    );
    assert!(!ran.load(Ordering::SeqCst), "a job ran inline");
}
