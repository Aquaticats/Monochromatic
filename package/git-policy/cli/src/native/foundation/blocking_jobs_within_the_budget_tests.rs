//! The blocking budget:
//! a job's value comes back,
//! a panic gives its place back,
//! no more jobs run at once than the budget allows,
//! and a system that refuses every thread gives an error value.

use super::*;
use crate::lazy_runtime::run_on_runtime;
use std::sync::atomic::AtomicUsize;
use std::sync::{Condvar, Mutex};
use crate::parallel_test_support::ONE_PLACE;
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
    // The cap only lowers what the CPUs allow, so the control gives the process room for the bound it measures.
    let _room: usize = crate::parallel_test_support::room_for("the_budget_is_at_least_one_place_and_a_cap_holds_it_lower", BOUND_HELD_TO);
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
    // Two of the burst's jobs must really run at once, so the balance of places must be as wide as the number of places.
    let _room: usize = crate::parallel_test_support::room_for("no_more_jobs_run_at_once_than_the_budget_allows", BOUND_HELD_TO);
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

/// What:
///  A refused thread start says which of the two happened:
///  that the step will not run,
/// or that it started on another thread and may have done its work.
/// Why:
///  The helper is the only place where that is known,
/// because it takes the claim:
/// the reporter's claim proves the queued closure will do nothing,
///  and a claim the job took first proves the job started
/// (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
/// `spawn_task`,
///  which queues before it asks for a thread).
/// The runner underneath promises neither:
/// its refusal says the queued closure
/// may still run (`BlockingJobError::MayStillRunAfterItsThreadWasRefused`),
///  and the claim taken here is what turns that into a fact.
/// A message that says the step had not started is false where the job took
///  the claim first,
/// and for a conversion the worktree may already have been changed.
#[test]
fn a_refused_thread_start_says_whether_the_step_will_run_or_already_has() {
    // The reporter takes the claim first: the step will do nothing.
    let never: Arc<TheClaimOnAJobsWork> = TheClaimOnAJobsWork::new();
    let (bound, taken) = {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(ONE_PLACE));
        let taken = Arc::clone(&semaphore).try_acquire_owned().expect("the place is free");
        (semaphore, taken)
    };
    let held: Arc<crate::the_place_of_one_blocking_job::ThePlaceOfOneJob> = crate::the_place_of_one_blocking_job::ThePlaceOfOneJob::holding(taken);
    let will_not_run: BlockingJobError = a_refused_thread_start(BlockingJobError::MayStillRunAfterItsThreadWasRefused, &never, &held);
    assert_eq!(will_not_run, BlockingJobError::NoThreadToStart);
    let said: String = will_not_run.to_string();
    assert!(said.contains("will not run"), "the step cannot run, and the message says so: {said}");
    assert!(!said.contains("may still run later"), "nothing is left that may run the step: {said}");
    assert_eq!(bound.available_permits(), 1, "the place went back, because the job will never run");

    // The job took the claim first: it started on another thread, and what it did is not known from here.
    let would_run: Arc<TheClaimOnAJobsWork> = TheClaimOnAJobsWork::new();
    assert!(would_run.the_job_takes_it(), "the job is first");
    let (other_bound, other_taken) = {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(ONE_PLACE));
        let other_taken = Arc::clone(&semaphore).try_acquire_owned().expect("the place is free");
        (semaphore, other_taken)
    };
    let other: Arc<crate::the_place_of_one_blocking_job::ThePlaceOfOneJob> = crate::the_place_of_one_blocking_job::ThePlaceOfOneJob::holding(other_taken);
    let started: BlockingJobError = a_refused_thread_start(BlockingJobError::MayStillRunAfterItsThreadWasRefused, &would_run, &other);
    assert_eq!(started, BlockingJobError::StartedAfterItsThreadWasRefused);
    let started_text: String = started.to_string();
    assert!(started_text.contains("has started"), "the job started on another thread, and the message says so: {started_text}");
    assert!(!started_text.contains("will not run"), "the job runs, so nothing may promise it will not: {started_text}");
    assert_eq!(other_bound.available_permits(), 0, "the place stays with the job that holds it");
}

/// What one job of the fallback control reads:
/// how many times its work has run.
/// Why:
///   The control proves the work ran once,
/// in turn,
/// where no thread could start it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const countTheRun = (state: AtomicUsize) => state.fetchAdd(1) + 1;
/// ```
fn count_the_run(state: Arc<AtomicUsize>) -> usize {
    return state.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
}

/// Where the system refuses every thread the work of the job runs in turn on the awaiting task,
/// once,
/// and its value is the job's.
/// The body runs in a child that is refused every thread.
#[test]
fn a_job_refused_a_thread_runs_in_turn() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "a_job_refused_a_thread_runs_in_turn") {
        return;
    }
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
    let runs: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let value = runtime.block_on(run_a_job_or_in_turn(Arc::clone(&runs), count_the_run));
    assert_eq!(value, Ok(1), "the work gave its value from the task that awaited it");
    assert_eq!(runs.load(std::sync::atomic::Ordering::SeqCst), 1, "the work ran once, in turn");
}

/// What one fallback of the bound's control does:
/// tells the control it is inside its work,
/// then holds that work open until the control lets it end.
/// Why:
///  The control reads the process's places while this work is open,
/// and a value it returned at once would leave nothing to read.
///  Both waits are bounded,
/// so a control that catches the defect still ends.
/// The release travels in a lock because a fallback's state is cloned for the job it may never run,
/// and a receiver is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tellAndWait = ([inside, release]: [Sender, Mutex<Receiver>]) => { inside.send(); return release.lock().recvTimeout(PATIENCE).isOk(); };
/// ```
fn tell_and_wait_inside((inside, release): (std::sync::mpsc::Sender<()>, Arc<Mutex<std::sync::mpsc::Receiver<()>>>)) -> bool {
    let _told = inside.send(());
    let waited = release.lock().expect("lock").recv_timeout(PATIENCE);
    return waited.is_ok();
}

/// What:
///  Where the system refuses every thread,
///  the work of the fallback runs while a place of the process's bound is held for it.
/// Why:
///  The fallback does the same blocking work the job would have done,
/// so the place the job gave back before the start must be held again for the work itself;
/// otherwise a second job takes that place and the process runs more blocking work at once than its bound allows
/// (`foundation/blocking_jobs_within_the_budget.rs`,
/// `run_a_job_or_in_turn`).
///  The place is read from the bound itself,
/// and the same read shows the place free before the work starts,
/// so a control that could never see a place held cannot pass this one.
/// The body runs in a child that is refused every thread.
#[test]
fn a_fallback_that_runs_in_turn_holds_a_place_of_the_bound() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "a_fallback_that_runs_in_turn_holds_a_place_of_the_bound") {
        return;
    }
    hold_blocking_jobs_to(ONE_PLACE);
    // The runtime is built while a thread may start,
    // because the workers the control reads the bound beside are threads too.
    crate::no_threads_child_test_support::let_threads_start_again().expect("the process limit rises");
    let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().expect("a runtime of two workers");
    crate::no_threads_child_test_support::refuse_threads_again().expect("the process limit falls");
    assert_eq!(budget().places.available_permits(), ONE_PLACE, "the control can see the process's one place while no work runs");
    let (inside, has_inside) = std::sync::mpsc::channel::<()>();
    let (release, waiting) = std::sync::mpsc::channel::<()>();
    let ran = runtime.spawn(async move { return run_a_job_or_in_turn((inside, Arc::new(Mutex::new(waiting))), tell_and_wait_inside).await; });
    assert!(has_inside.recv_timeout(PATIENCE).is_ok(), "the fallback never reached its work, so the system did not refuse its thread");
    let free: usize = budget().places.available_permits();
    let _let_go = release.send(());
    let ended = runtime.block_on(async { return tokio::time::timeout(PATIENCE, ran).await; });
    std::mem::forget(runtime);
    assert_eq!(ended.expect("the fallback ended").expect("the fallback's task"), Ok(true), "the work ended when the control let it go");
    assert_eq!(
        free, 0,
        "the fallback ran its blocking work with the process's place free, so a second job took that place and both ran at once"
    );
}

/// The places this control gives the process,
/// so the reserve leaves one of them to the work a person waits for.
const TWO_PLACES: usize = 2;

/// What:
///  How long the control lets a slow job that must not start keep trying,
///  so a run that hands it the reserved place fails rather than hangs.
const SLOW_JOB_PATIENCE: Duration = Duration::from_secs(2);

/// What:
///  The place the gate keeps back stays free while slow work holds the rest,
///  so a push's own step runs beside a queue of slow jobs instead of behind it.
/// Why:
///  On 2026-11-11 at 10:38 the installed `0.6.1-alpha.10` had two reviews and four trashes claimed and running while a
/// fresh commit's push sat pending from 10:29, and `main` was pushed by hand.
/// The pushes were never claimed by the slow work;
/// their steps waited for a place of the process's one bound on blocking jobs,
/// which the slow work held.
/// Measured for the record with the service's four places and four worktrees' trashes of 500 entries each,
/// the push's wait fell from a queue of slow work's whole length to the push's own cost
/// (`landing/service_work_items_side_by_side_tests.rs`).
/// The control states the two things a reserve must do:
/// the work a person waits for runs while slow work holds every other place,
/// and slow work never takes the reserved one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdSlow(); expect(await plainJob()).toBe(7); expect(secondSlowStarted).toBe(false)
/// ```
#[test]
fn a_reserved_place_keeps_the_work_a_person_waits_for_out_of_a_queue_of_slow_jobs() {
    hold_blocking_jobs_to(TWO_PLACES);
    let (inside, has_inside) = std::sync::mpsc::channel::<()>();
    let (release, waiting) = std::sync::mpsc::channel::<()>();
    let slow = crate::lazy_runtime::spawn_on_runtime(async move {
        let _held: Result<(), BlockingJobError> = run_a_slow_job_within_the_budget(move || {
            let _told = inside.send(());
            let _ = waiting.recv();
        })
        .await;
    });
    assert!(slow.is_some(), "the runtime has a worker, so a slow job can take its place");
    assert!(has_inside.recv_timeout(PATIENCE).is_ok(), "the slow job never took its place");
    // The work a person waits for: one blocking job, which a push's own read of its child is.
    let began: std::time::Instant = std::time::Instant::now();
    let answered: Result<u8, BlockingJobError> = run_on_runtime(async { return run_blocking_job_within_the_budget(|| 7_u8).await; }).expect("the runtime");
    let waited: Duration = began.elapsed();
    assert_eq!(answered, Ok(7), "the work a person waits for ran");
    assert!(waited < PATIENCE, "the work a person waits for ran beside the slow job, in {waited:?}");
    // A second slow job must wait for the place the gate keeps, so slow work never takes the reserve.
    let started: Arc<std::sync::atomic::AtomicBool> = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let told: Arc<std::sync::atomic::AtomicBool> = Arc::clone(&started);
    let second = crate::lazy_runtime::spawn_on_runtime(async move {
        let _held: Result<(), BlockingJobError> = run_a_slow_job_within_the_budget(move || {
            told.store(true, Ordering::SeqCst);
        })
        .await;
    });
    assert!(second.is_some(), "the runtime has a worker, so a second slow job can ask");
    std::thread::sleep(SLOW_JOB_PATIENCE);
    let took_the_reserve: bool = started.load(Ordering::SeqCst);
    let _let_go = release.send(());
    assert!(!took_the_reserve, "a second slow job took the place the gate keeps for the work a person waits for");
}
