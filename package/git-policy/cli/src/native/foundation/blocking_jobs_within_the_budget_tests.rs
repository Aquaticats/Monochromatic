//! The blocking budget:
//! a job's value comes back,
//! a panic gives its place back,
//! no more jobs run at once than the budget allows,
//! and a system that refuses every thread gives an error value.

use super::*;
use crate::lazy_runtime::run_on_runtime;
use std::sync::atomic::AtomicUsize;
use crate::parallel_test_support::ONE_PLACE;
use std::time::Duration;

/// How long a job of the bound's control holds its place,
/// long enough for the other jobs of the burst to arrive.
const HELD_MILLIS: u64 = 40;

/// How many jobs the bound's control starts at once,
/// three times the bound the control sets.
const BURST: usize = 6;

/// How many jobs hold themselves to at once in the bound's control.
const BOUND_HELD_TO: usize = 2;

/// The longest a control waits for a job that should not wait at all.
const PATIENCE: Duration = Duration::from_secs(30);

/// How many jobs are inside their work right now,
/// and the most that ever were at one instant.
struct Concurrency {
    /// How many are inside now.
    live: AtomicUsize,
    /// The most that were inside together.
    most: AtomicUsize,
}

impl Concurrency {
    /// A count where nothing is running yet.
    fn new() -> Arc<Self> {
        return Arc::new(Concurrency { live: AtomicUsize::new(0), most: AtomicUsize::new(0) });
    }

    /// Count one job in and keep the most.
    fn enter(&self) -> usize {
        let now: usize = self.live.fetch_add(1, Ordering::SeqCst) + 1;
        let _kept: usize = self.most.fetch_max(now, Ordering::SeqCst);
        return now;
    }

    /// Count one job out.
    fn leave(&self) {
        let _left: usize = self.live.fetch_sub(1, Ordering::SeqCst);
    }
}

/// A job that holds its place for `HELD_MILLIS` and says how many were inside with it.
fn hold_a_place(shared: Arc<Concurrency>) -> impl FnOnce() -> usize + Send + 'static {
    return move || -> usize {
        let with_me: usize = shared.enter();
        std::thread::sleep(Duration::from_millis(HELD_MILLIS));
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
/// however many are started.
#[test]
fn no_more_jobs_run_at_once_than_the_budget_allows() {
    hold_blocking_jobs_to(BOUND_HELD_TO);
    let shared: Arc<Concurrency> = Concurrency::new();
    let outcome = run_on_runtime(async {
        let started = tokio::join!(
            run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&shared))),
            run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&shared))),
            run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&shared))),
            run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&shared))),
            run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&shared))),
            run_blocking_job_within_the_budget(hold_a_place(Arc::clone(&shared))),
        );
        return [started.0, started.1, started.2, started.3, started.4, started.5];
    })
    .expect("the runtime");
    assert!(outcome.iter().all(Result::is_ok), "every job of the burst ran: {outcome:?}");
    let most: usize = shared.most.load(Ordering::SeqCst);
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
/// the jobs have not run when the refusal is reported,
/// each one says plainly that it had not started,
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
    assert_eq!(outcomes.0, Err(BlockingJobError::NoThreadToStart), "the first job had not run when the refusal was reported");
    assert_eq!(
        outcomes.1.expect("the place of a job refused a thread was never given back"),
        Err(BlockingJobError::NoThreadToStart),
        "the second job had not run either, so the first job's place was given back"
    );
    assert!(!ran.load(Ordering::SeqCst), "a job ran inline");
}

/// How many blocking threads the control of a refused job's place keeps in its own runtime,
/// more than the bound it holds the process to.
const POOL_THREADS_FOR_THE_CONTROL: usize = 8;

/// What:
///  A job the pool queued after a refused thread does not run once its frame has given up on it,
///  so no blocking job of this process ever runs without a place of the bound.
/// Why:
///  The pool queues a job before it asks for a thread (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
/// `spawn_task`),
/// and the frame gives the place back when it stops waiting for the job.
/// A queued job that then ran would do its work with no place at all,
/// so the bound of the process would no
/// longer cover every running blocking job:
///  `a_report_that_says_an_index_could_not_be_started_...` measured four rewrites running that way.
/// A place that was never given back would instead leave later jobs waiting for ever,
///  so what decides is the claim:
/// the job takes it before its first effect,
/// and the frame takes it when it gives up.
/// The control runs in a child whose process limit is one,
/// because only a refused thread start queues a job this way.
#[test]
fn a_job_the_pool_queued_after_a_refusal_does_not_run() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "a_job_the_pool_queued_after_a_refusal_does_not_run") {
        return;
    }
    hold_blocking_jobs_to(BOUND_HELD_TO);
    // A runtime with a pool deep enough for the jobs below, built while a thread may start.
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises");
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(POOL_THREADS_FOR_THE_CONTROL)
        .enable_all()
        .build()
        .expect("a runtime with a deep pool");
    // Every thread is refused: the first job is queued, and its frame is told it did not start.
    crate::no_threads_child_test_support::refuse_threads_again().expect("the process limit falls");
    let ran: Arc<std::sync::atomic::AtomicBool> = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let queued = Arc::clone(&ran);
    let refused = runtime.block_on(async move { return run_blocking_job_within_the_budget(move || { queued.store(true, Ordering::SeqCst); }).await; });
    assert_eq!(refused, Err(BlockingJobError::NoThreadToStart), "no thread could start the first job");
    assert!(!ran.load(Ordering::SeqCst), "the first job had not run when the refusal was reported");
    // A thread may start again, and the pool's first new thread runs what it queued before anything else.
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises again");
    let kicked = runtime.block_on(async { return run_blocking_job_within_the_budget(|| ()).await; });
    assert!(kicked.is_ok(), "the job that starts the pool's first thread ran: {kicked:?}");
    assert!(
        !ran.load(Ordering::SeqCst),
        "the job the report gave up on ran afterwards with no place of the bound held for it, so the bound no longer covers every running blocking job"
    );
    let _kept: tokio::runtime::Runtime = runtime;
}

/// How long the control of a cancelled job lets the pool run after the caller stopped waiting.
const AFTER_THE_CANCEL: Duration = Duration::from_secs(2);

/// What:
///  A job whose caller stopped waiting does not run afterwards.
/// Why:
///  Dropping the future of `run_blocking_job_within_the_budget` leaves the closure queued in the pool
/// (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
/// `spawn_task`),
/// and the queue's next thread runs it.
/// A caller that timed out has told nobody that the work would happen,
/// so a job it gave up on must do nothing:
///  its progress and its result have no reader left,
/// and for a conversion the worktree would change after the caller reported a timeout
/// ("never discard user data",
/// "nothing should ever wait forever").
/// The control keeps the pool's only thread inside an unrelated job,
/// so the measured job is queued when the timeout drops it,
///  and then lets the pool run.
#[test]
fn a_job_whose_caller_stopped_waiting_does_not_run_afterwards() {
    crate::lazy_runtime::cap_workers(ONE_PLACE);
    hold_blocking_jobs_to(BOUND_HELD_TO);
    let (holding, held) = std::sync::mpsc::channel::<()>();
    let (release, waiting) = std::sync::mpsc::channel::<()>();
    let busy = crate::lazy_runtime::spawn_on_runtime(async move {
        let _outcome = run_blocking_job_within_the_budget(move || {
            let _told = holding.send(());
            let _ = waiting.recv();
        })
        .await;
    });
    assert!(busy.is_some(), "the runtime has a worker, so a place of the blocking pool can be held");
    assert!(held.recv_timeout(PATIENCE).is_ok(), "the job that keeps the pool's thread never started");
    // The place this job takes is free, so its closure is queued behind the only pool thread.
    let ran: Arc<std::sync::atomic::AtomicBool> = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let did_run = Arc::clone(&ran);
    let stopped = run_on_runtime(async move {
        let timed = tokio::time::timeout(Duration::from_millis(200), run_blocking_job_within_the_budget(move || { did_run.store(true, Ordering::SeqCst); })).await;
        return timed;
    })
    .expect("the runtime");
    assert!(stopped.is_err(), "the caller stopped waiting for a job whose closure the pool had not started");
    // The pool's thread is free again, and its queue runs what the caller gave up on.
    let _let_go = release.send(());
    let pool_ran = run_on_runtime(async { return run_blocking_job_within_the_budget(|| ()).await; }).expect("the runtime");
    assert!(pool_ran.is_ok(), "the job that drains the pool's queue ran: {pool_ran:?}");
    let waited: std::time::Instant = std::time::Instant::now();
    while ran.load(Ordering::SeqCst) && waited.elapsed() < AFTER_THE_CANCEL {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        !ran.load(Ordering::SeqCst),
        "the job whose caller stopped waiting ran afterwards, so a step can act after its caller reported a timeout"
    );
}
