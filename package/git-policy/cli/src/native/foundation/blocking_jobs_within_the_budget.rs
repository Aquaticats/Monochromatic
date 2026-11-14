//! What:
//!  The most blocking jobs this process runs at once:
//! one bound for the whole process,
//! sized from the budget for parallel work and
//!  the limits on threads,
//! taken by each job that goes to the blocking pool and given back when that job ends.
//! Why:
//!  The owner's ruling is a runtime "sized to budget" with "Build size + job limit" (`doc/planning/git-policy-cli-full-tokio.md`,
//! "The owner's answers at 2026-11-07",
//! choice 2):
//! tokio cannot resize a runtime's workers after `build()`,
//! so the size
//!  at the build is the size for the life of the process,
//! and what can still follow "as many threads as the computer can spare at
//!  that moment" is how much work runs at once.
//! One object of the process holds that bound,
//! shared by the rounds,
//! the parallel runner and the
//!  durable syncs,
//! so the work of one of them cannot take the places the others need.
//! A permit is owned by the job itself,
//! so it is given back when the job ends,
//! never when the task that
//!  waits for it stops waiting.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Node's libuv pool has a size, not a permit per job; here the place is taken before the job and held by it:
//! const value = await runBlockingJobWithinTheBudget(() => copyTree(source, target)); // Ok(value) | Err(NoThreadToStart)
//! ```

use crate::blocking_job::{BlockingJobError, run_blocking_job};
use crate::the_claim_a_job_takes_before_running_it::TheClaimOnAJobsWork;
use crate::the_place_of_one_blocking_job::{APlaceGivenBackWhenItEnds, ThePlaceOfOneJob};
use crate::parallel_budget::HostResources;
use crate::runtime_limits::RuntimeLimits;
use crate::runtime_plan::budgeted_workers;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::Semaphore;

/// The places of the process,
/// made by the first call that needs one.
static BUDGET: OnceLock<TheBudget> = OnceLock::new();

/// The most places a process may hold itself to,
/// where the process asked for a cap before the budget was made;
/// `usize::MAX` for no cap.
static HELD_TO: AtomicUsize = AtomicUsize::new(usize::MAX);

/// How many blocking jobs may run at once,
/// and the places that bound them.
struct TheBudget {
    /// The places the work of the service takes and gives back.
    places: Arc<Semaphore>,
    /// How many places there are,
    /// kept so that a caller can be told the bound without taking one.
    allowed: usize,
    /// The places kept for the answers a client waits for,
    /// which the work of the service never takes.
    answering: Option<Arc<Semaphore>>,
}

/// What:
///  The budget of this process:
/// as many places as the budget for parallel work and the limits on threads leave,
/// and no more than the
///  cap the process asked for.
/// Why:
///  Made once,
/// so every part of the process draws on one bound (`doc/planning/git-policy-cli-full-tokio.md`,
/// "How the worker count is
///  chosen":
/// "the semaphore is one object of the process (not one per round)").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const budget = makeTheBudget(); // { places: new Semaphore(n), allowed: n }
/// ```
fn budget() -> &'static TheBudget {
    return BUDGET.get_or_init(decide_the_budget);
}

/// What:
///  The budget this process makes on its first need:
/// the places the budget for parallel work and the limits leave,
/// no more than the cap the process asked for,
///  and one of them kept for the answers a client waits for.
/// Why:
///  A general place bounds a job only as long as the ceiling of the step that asked for it,
/// and a step that outlasts its ceiling keeps its place until its work ends,
/// which a filesystem call that stopped answering can outlast without end.
/// On `2026-11-13` a review of this branch held every general place with such a step and showed what follows:
/// a queued client's withdrawal was never decided, so the person was told it was not known whether the commit would
///  land, and a new commit's look answered `Unavailable`, which ends a commit.
/// So the answers a client waits for do not ask for a general place at all
/// (`run_an_answer_to_a_client`),
/// and the places sum to the process's bound, so a place is still a thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answering = allowed >= 3 ? PLACES_KEPT_TO_ANSWER_A_CLIENT : 0;
/// const budget = { places: new Semaphore(allowed - answering), allowed, answering: answering ? new Semaphore(answering) : undefined };
/// ```
fn decide_the_budget() -> TheBudget {
    let host: HostResources = HostResources::read();
    let limits: RuntimeLimits = RuntimeLimits::read();
    let allowed: usize = budgeted_workers(&host, &limits).min(HELD_TO.load(Ordering::SeqCst)).max(1);
    // One of the places is kept for the answers a client waits for, where there are enough to keep one from
    //  (`PLACES_KEPT_TO_ANSWER_A_CLIENT`).
    let answering: usize = if allowed >= 3 { PLACES_KEPT_TO_ANSWER_A_CLIENT } else { 0 };
    return TheBudget {
        places: Arc::new(Semaphore::new(allowed.saturating_sub(answering))),
        allowed,
        answering: (answering > 0).then(|| { return Arc::new(Semaphore::new(answering)); }),
    };
}

/// What:
///  How many of the process's places are kept for the answers a client waits for:
/// one.
/// Why:
///  A withdrawal a person waits for and the read a commit's answer waits for must get their turn however long the
/// jobs holding places take, and the work of the service can hold every place it may take.
/// One is the smallest number that does that:
/// the two asks are one read of the journal each,
/// they take turns at this place,
/// and a second place would be taken from the work a person's commit needs.
/// The coordinator's pick under the owner's delegation, open to the owner's veto.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PLACES_KEPT_TO_ANSWER_A_CLIENT = 1;
/// ```
pub const PLACES_KEPT_TO_ANSWER_A_CLIENT: usize = 1;

/// What:
///  How many places the service's own work may take at once:
/// the process's places less those kept for the answers a client waits for.
/// Why:
///  A control that holds every place of the service's work states how many jobs it must hold,
/// and the figure is the semaphore's own, not the whole budget: the answers keep their place either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placesForTheServicesOwnWork = () => budget().places.availablePermits() // while nothing runs
/// ```
pub fn places_for_the_services_own_work() -> usize {
    return budget().allowed.saturating_sub(budget().answering.as_ref().map_or(0, |_| { return PLACES_KEPT_TO_ANSWER_A_CLIENT; }));
}

/// What:
///  How many blocking jobs this process runs at once.
/// Why:
///  A control,
/// a log line and the shipped callers need the figure that is in force,
/// which is not the number of free places while
///  jobs run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// print(`at most ${budgetedBlockingJobs()} blocking jobs at once`);
/// ```
pub fn budgeted_blocking_jobs() -> usize {
    return budget().allowed;
}

/// What:
///  How many of the process's places a round's own slow work may take:
/// every place but those kept for the work a person waits for,
/// and at least one.
/// Why:
///   A round of the after-commit work removes trash entries side by side,
/// and each removal is a job of the blocking pool,
/// so a round that takes every place holds the steps that answer a client behind it:
/// the check of which instance a commit belongs to,
/// a withdrawal's decision,
/// and the note of a beat
/// all run there,
/// and the measurement of 2026-11-10 shows what that cost (`clients_keep_a_place_of_the_pool`,
/// `doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`).
/// The number is the places slow work itself may take, not one less than the whole budget:
/// a round that removed with more places than `slow_places` offers would queue behind its own lane for them,
/// and the promise that the places of a person's work stay free is kept by one figure, not two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const places = Math.max(1, budgetedBlockingJobs() - SLOW_WORK_RESERVE);
/// ```
pub fn places_a_round_may_take() -> usize {
    return budget().allowed.saturating_sub(SLOW_WORK_RESERVE).max(1);
}

/// What:
///  Holds every blocking job of this process to at most `cap` at once.
/// Why:
///  A process that runs many of its kind at once bounds its own parallel work the way it bounds its runtime
/// (`service/service_main.rs`,
/// `SERVICE_WORKER_CAP`),
/// and a control needs a bound small enough to see.
/// A budget that is already made is not resized,
/// so a call after the first job changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdBlockingJobsTo(2)
/// ```
pub fn hold_blocking_jobs_to(cap: usize) {
    HELD_TO.store(cap, Ordering::SeqCst);
}

/// What:
///  Runs `job` on the blocking pool while one of the process's places is held.
/// Why:
///  The bound of the owner's "job limit" is this call:
/// a job that cannot take a place waits for one,
/// and a job that ends gives its
///  place back.
/// The place is taken before the pool is asked for a thread,
/// so a burst of work queues here and never starts threads the budget did not
///  count.
/// The place is held by this frame and not moved into the job,
/// and that is measured,
/// not a preference:
/// when the
///  system refuses a thread,
/// `spawn_blocking` panics after it has queued the job,
/// and the queued job keeps everything it captured for the
///  life of the process (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
/// `spawn_task`:
/// the task is pushed before the thread is
///  asked for,
/// and `Err(NoThreads)` leaves it queued with no thread to run it).
/// A place that travelled inside the job would therefore be lost for
///  good,
/// and a later job would wait for it with nothing to end that wait
/// ("nothing should ever wait forever").
/// The cost of holding it here is that a caller
///  who stops waiting gives its place back while the job it started may still run;
/// no caller of the shipped code stops waiting for one of these
///  jobs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = await budget.places.acquire(); try { return await runBlockingJob(job); } finally { place.release(); }
/// ```
pub async fn run_blocking_job_within_the_budget<T, J>(job: J) -> Result<T, BlockingJobError>
where
    T: Send + 'static,
    J: FnOnce() -> T + Send + 'static,
{
    // A control can wait on this ask instead of on a clock, so work that bypasses the bound cannot pass it.
    #[cfg(test)]
    let _asking: jobs_waiting_for_a_place::AJobWaits = jobs_waiting_for_a_place::note_that_a_job_waits();
    let place: tokio::sync::OwnedSemaphorePermit = match Arc::clone(&budget().places).acquire_owned().await {
        Ok(taken) => taken,
        // The semaphore lives as long as the process, so a closed one is a defect rather than a state: nothing is run.
        Err(_closed) => return Err(BlockingJobError::Cancelled),
    };
    // The place is this job's, so it is no longer one of the jobs waiting for one.
    #[cfg(test)]
    drop(_asking);
    return run_a_job_holding_the_place(place, job).await;
}

/// What:
///  `run_blocking_job_within_the_budget`'s work,
/// once its place of the bound is taken.
/// Why:
///   A caller that may not wait without an end takes the place under a ceiling of its own
/// (`run_a_job_once_awaited_no_longer_than`),
/// and the rest of the job is the same for both:
///  the claim,
/// the pool's thread and the place that goes back when the work ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = await runTheJobHoldingThePlace(place, job);
/// ```
async fn run_a_job_holding_the_place<T, J>(place: tokio::sync::OwnedSemaphorePermit, job: J) -> Result<T, BlockingJobError>
where
    T: Send + 'static,
    J: FnOnce() -> T + Send + 'static,
{
    // The place and the claim travel together: a job the pool queued can run after this frame stops waiting, and then
    //  it must still hold a place, while a job this frame gives up on must do nothing at all.
    let claimed: Arc<TheClaimOnAJobsWork> = TheClaimOnAJobsWork::new();
    let held: Arc<ThePlaceOfOneJob> = ThePlaceOfOneJob::holding(place);
    // A caller that stops waiting drops this, and the job the pool still holds queued must then do nothing.
    let mut call: TheCallOnAJob = TheCallOnAJob { claimed: Arc::clone(&claimed), held: Arc::clone(&held), settled: false };
    let (jobs_claim, jobs_place) = (Arc::clone(&claimed), Arc::clone(&held));
    let outcome = run_blocking_job(move || {
        // The claim comes first, before anything of the job's own work.
        if !jobs_claim.the_job_takes_it() {
            return None;
        }
        let _place: APlaceGivenBackWhenItEnds = APlaceGivenBackWhenItEnds::covering(jobs_place);
        return Some(job());
    })
    .await;
    return match outcome {
        Ok(Some(value)) => {
            // The job ran and gave its place back as its work ended.
            call.settled = true;
            Ok(value)
        }
        // The job found the claim taken, which only this frame's reporter does, and only after this call returned.
        Ok(None) => panic!("a blocking job found its own work given up on while the caller still waited for it"),
        Err(failed) => {
            // The place goes back here only when the job will never run: a job that took the claim holds it until its
            //  work ends, so the process's bound keeps covering every running job.
            let answered: BlockingJobError = a_refused_thread_start(failed, &claimed, &held);
            call.settled = true;
            Err(answered)
        }
    };
}

/// The places slow work may take of the process's bound,
/// made by the first slow job that needs one.
static SLOW_PLACES: OnceLock<Arc<Semaphore>> = OnceLock::new();

/// How many of the process's places stay free of slow work:
/// 2.
/// Why:
///  A job queues in the process's bound in the order it asked,
/// so a queue of slow jobs stands between the work a person waits for and the place it needs:
/// measured `2026-11-11T23:05:00-05:00` on this host,
/// with the service's own four places,
/// a push written behind four worktrees' trashes of 500 entries each started 4.3 s after it was asked for and took
/// another 4.3 s,
/// while the same push with no slow work beside it lands in about 0.1 s
/// (`landing/service_work_items_side_by_side_tests.rs`,
/// the measurement for the record).
/// The pushes of a repository are never claimed by the slow lane,
/// so what held them was the pool and not the work:
/// a place kept free of slow work is what makes the lane's own promise true
/// (`landing/service_work_slow_lane.rs`).
/// Two and not one, because a place is not a lane:
/// the place a commit's landing needs and the place the ask that routes its request needs are two asks,
/// and one free place let either of them wait behind the other.
/// The service that answered a commit thirteen minutes late on `2026-11-12` held all four of its places in slow work
/// or in unbudgeted jobs of the pool (the journals and the analysis are in
/// `doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`),
/// and one free place would have been enough for neither the ask nor the landing.
/// The owner's rule about waits,
/// "nothing should ever wait forever or indefinitely",
/// read as the smallest number that leaves the path that begins a landing its own two places while slow work runs;
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
///
/// It is public so a control of a process the test cannot reach states the room its relation needs:
/// the places kept here, plus one place for each slow step the control holds at once
/// (`binary/binary_a_commit_lands_while_every_kind_of_work_is_held_tests.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SLOW_WORK_RESERVE = 2;
/// ```
pub const SLOW_WORK_RESERVE: usize = 2;

/// What:
///  The places slow work may take at once:
/// the process's places less `SLOW_WORK_RESERVE`,
/// and at least one.
/// Why:
///   Slow work is the work nobody waits for,
/// so it takes what the work a person waits for does not need.
/// A process held to one place offers slow work that one place,
/// because a lane of no places would never finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slowPlaces = () => slowPlacesOf(budget()); // a semaphore of allowed - 1
/// ```
fn slow_places() -> &'static Arc<Semaphore> {
    return SLOW_PLACES.get_or_init(the_places_slow_work_may_take);
}

/// The places slow work may take,
/// made by the first slow job that asks for one.
fn the_places_slow_work_may_take() -> Arc<Semaphore> {
    return Arc::new(Semaphore::new(places_a_round_may_take()));
}

/// What:
///  Runs `job` on the blocking pool while one of the places for slow work is held,
/// so the places the work a person waits for needs stay free of it.
/// Why:
///   A step nobody waits for may take minutes,
/// and every place it holds is a place a push or a landing waits for:
/// this is where the lane's work asks,
/// and the reserve is what keeps its queue out of the round's way.
/// The place for slow work is this frame's and is given back when this call returns;
/// a job the pool queued and this frame stopped waiting for therefore runs with no place for slow work,
/// which is one slow job more than the lane offers and never more blocking work than the process allows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const slow = await slowPlaces.acquire(); try { return await runBlockingJobWithinTheBudget(job); } finally { slow.release(); }
/// ```
pub async fn run_a_slow_job_within_the_budget<T, J>(job: J) -> Result<T, BlockingJobError>
where
    T: Send + 'static,
    J: FnOnce() -> T + Send + 'static,
{
    let slow: tokio::sync::OwnedSemaphorePermit = match Arc::clone(slow_places()).acquire_owned().await {
        Ok(taken) => taken,
        // The semaphore lives as long as the process, so a closed one is a defect rather than a state: nothing is run.
        Err(_closed) => return Err(BlockingJobError::Cancelled),
    };
    let answer: Result<T, BlockingJobError> = run_blocking_job_within_the_budget(job).await;
    drop(slow);
    return answer;
}

/// What:
///  Run `work(state)` as a job of the blocking pool with a place for slow work held,
/// or run it in turn here where the system refused a thread.
/// Why:
///   The slow kinds' steps are the same two cases `run_a_job_or_in_turn` covers,
/// and the reserve is what the places of both take.
/// The work in turn is this frame's,
/// so the place for slow work stays held for as long as it runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await runASlowJobWithinTheBudget(() => work(state)); } catch (refused) { if (!refused.noThreadToStart) raise(refused); return work(state); }
/// ```
pub async fn run_a_slow_job_or_in_turn<S: Clone + Send + 'static, T: Send + 'static>(state: S, work: fn(S) -> T) -> Result<T, BlockingJobError> {
    let slow: tokio::sync::OwnedSemaphorePermit = match Arc::clone(slow_places()).acquire_owned().await {
        Ok(taken) => taken,
        Err(_closed) => return Err(BlockingJobError::Cancelled),
    };
    // The place for slow work is held on this frame, so the work done in turn below it is covered too.
    let answer: Result<T, BlockingJobError> = run_a_job_or_in_turn(state, work).await;
    drop(slow);
    return answer;
}

/// What:
///  Run `work(state)` as a job of the blocking pool,
/// or run it in turn here where the system refused a thread,
/// with a place of the process's bound held for as long as that work runs.
/// Why:
///   The owner's rule is that a thread is only a way to do the work sooner
/// (`scoped_start.rs`),
/// so the work is done on this
///  task where no thread can start and the command loses nothing.
/// Only a refusal falls back:
/// `NoThreadToStart` is the one answer
///  that promises the step will not run and changed nothing
/// (`a_refused_thread_start`),
/// while a busy pool queues the job and the wait for it
///  is the caller's.
/// The work is the same blocking work the job would have done,
/// so it takes a place again before it starts:
/// the job gave its place back before this call learned it would never run,
/// and a fallback without a place would let a second job take the free one and run beside it,
/// so the process
///  would run more blocking work at once than its bound allows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await runBlockingJobWithinTheBudget(() => work(state)); } catch (refused) { if (!refused.noThreadToStart) raise(refused); const place = await budget.places.acquire(); try { return work(state); } finally { place.release(); } }
/// ```
pub async fn run_a_job_or_in_turn<S: Clone + Send + 'static, T: Send + 'static>(state: S, work: fn(S) -> T) -> Result<T, BlockingJobError> {
    let theirs: S = state.clone();
    let outcome: Result<T, BlockingJobError> = run_blocking_job_within_the_budget(move || { return work(theirs); }).await;
    return match outcome {
        Ok(value) => Ok(value),
        // The job will not run and changed nothing, so the same work runs here, and its place is held until it ends.
        Err(BlockingJobError::NoThreadToStart) => match Arc::clone(&budget().places).acquire_owned().await {
            // The place is this frame's and it is dropped after the work, never before it.
            Ok(_place) => Ok(work(state)),
            // The semaphore lives as long as the process, so a closed one is a defect: nothing is run.
            Err(_closed) => Err(BlockingJobError::Cancelled),
        },
        Err(another) => Err(another),
    };
}

/// What:
///  The error a refused start is,
///  from what the claim on the job's work decided.
/// Why:
///  This is the only place that knows which of two things happened,
/// because it takes the claim:
///  a reporter's claim proves the queued closure will do nothing,
/// and a claim the job took first proves the job started on another thread
/// (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
/// `spawn_task`,
///  which queues the job before it asks for a thread).
/// A message that says the step had not started is false in the second case,
///  and for a conversion the worktree may already have been changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = aRefusedThreadStart(failed, claim, place); // NeverRuns | StartedAnyway
/// ```
fn a_refused_thread_start(failed: BlockingJobError, claimed: &TheClaimOnAJobsWork, held: &ThePlaceOfOneJob) -> BlockingJobError {
    if claimed.the_reporter_takes_it() {
        held.give_it_back();
        return match failed {
            // The runner's own refusal promises nothing about the closure the pool queued; the claim this call has just
            //  taken is what makes this answer sure.
            BlockingJobError::MayStillRunAfterItsThreadWasRefused => BlockingJobError::NoThreadToStart,
            other => other,
        };
    }
    // The job took the claim, so it runs or has run, and this answer says only what is known.
    return match failed {
        BlockingJobError::NoThreadToStart | BlockingJobError::MayStillRunAfterItsThreadWasRefused => BlockingJobError::StartedAfterItsThreadWasRefused,
        other => other,
    };
}

/// What:
///  What a job of the pool answered,
///  when the caller may not wait for it without an end:
/// the value it gave,
/// nothing at all,
/// or nothing this caller can read.
/// Why:
///   A step whose answer is a value of the caller's own state can be asked again when nothing ran,
/// and must be treated as
///  possibly done when the job started on another thread or outlasted its ceiling;
/// a caller that took
///  `StartedAfterItsThreadWasRefused` for "nothing ran" left a promised retry with no wake
/// (finding 1 of the review of `b3d00b3be4`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AnswerOfAJob<T> = { ran: T } | 'notRun' | 'noAnswer';
/// ```
#[derive(Debug)]
pub enum TheAnswerOfAJobAwaited<T> {
    /// The job ran and gave this.
    Ran(T),
    /// The system gave no thread,
    /// so the job will do nothing and nothing changed.
    NotRun,
    /// The job started on another thread and its value cannot be read here,
    /// or it outlasted its ceiling:
    ///  it may have done its work.
    NoAnswer,
}

/// What:
///  Run `work(state)` as a job of the blocking pool,
/// with a place of the process's bound held while it runs,
/// and await it no longer than `ceiling`.
/// Why:
///   A caller that polls the runtime may not run a step that can wait on storage itself:
/// a filesystem that stopped answering would hold a worker the runtime needs,
/// and a timer around work on that worker cannot end it.
/// The ceiling bounds the caller's wait,
/// not the job:
/// a job that outlasts it is
///  left behind with its place held until its work ends,
/// as `blocking_jobs_within_the_budget.rs` holds every place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await runAJobAwaitedNoLongerThan(ceiling, state, work);
/// ```
pub async fn run_a_job_awaited_no_longer_than<S: Send + 'static, T: Send + 'static>(ceiling: Duration, state: S, work: fn(S) -> T) -> TheAnswerOfAJobAwaited<T> {
    #[cfg(test)]
    if let Some(answering) = a_controls_answer::the_answer_a_control_states() {
        // The work runs, as it does on the pool, so a control sees the step's own changes; only the answer is the control's.
        let _ran: T = work(state);
        return match answering {
            a_controls_answer::AStatesAnswer::NotRun => TheAnswerOfAJobAwaited::NotRun,
            a_controls_answer::AStatesAnswer::NoAnswer => TheAnswerOfAJobAwaited::NoAnswer,
        };
    }
    return run_a_job_once_awaited_no_longer_than(ceiling, move || { return work(state); }).await;
}

/// What:
///  Run `work(state)` as a job of the blocking pool with a place of slow work and a place of the process's bound held,
///  awaiting each of them no longer than `ceiling`.
/// Why:
///  A step of slow work that can wait on storage with no end must be bounded like any other, and it takes the places a
/// slow step takes: the place of slow work that leaves the reserve free, and a place of the process's bound, so a place
/// stays a thread.
/// A place that no place comes for within the ceiling proves the work never started, and a work that outlasts the
/// ceiling after it started may have done its work; the two are told apart here as everywhere else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await runASlowJobOnceAwaitedNoLongerThan(ceiling, state, work); // { ran } | noPlace | noAnswer
/// ```
pub async fn run_a_slow_job_once_awaited_no_longer_than<S: Send + 'static, T: Send + 'static>(ceiling: Duration, state: S, work: fn(S) -> T) -> TheAnswerOfAJobAwaited<T> {
    let slow: tokio::sync::OwnedSemaphorePermit = match tokio::time::timeout(ceiling, Arc::clone(slow_places()).acquire_owned()).await {
        Ok(Ok(taken)) => taken,
        // No place came within the ceiling, so the job never started and nothing it would have changed was changed.
        Ok(Err(_closed)) => return TheAnswerOfAJobAwaited::NotRun,
        Err(_elapsed) => return TheAnswerOfAJobAwaited::NotRun,
    };
    let answer: TheAnswerOfAJobAwaited<T> = run_a_job_once_awaited_no_longer_than(ceiling, move || { return work(state); }).await;
    drop(slow);
    return answer;
}

/// What:
///  Run `work(state)` as a job of the blocking pool with one of the places kept for the answers a client waits for,
///  awaiting the place no longer than `ceiling` and then the job no longer than another `ceiling`.
/// Why:
///  A withdrawal a person waits for and the read a commit's answer waits for must get their turn however long the
/// jobs holding places take,
/// and every place of the service's own work can be held by a step that outlasted its ceiling.
/// So these two asks never wait for one of those places:
/// they have one kept for them (`PLACES_KEPT_TO_ANSWER_A_CLIENT`),
/// and they wait for it no longer than `ceiling`,
///  so an answer that cannot start is said rather than waited for.
/// The places of the two classes sum to the process's bound,
/// so a place of this class is a thread the service's own work cannot be holding.
/// A process with fewer than three places keeps none for this class,
/// and the ask here is the general one,
/// with the same ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await runAnAnswerToAClient(ceiling, state, work); // { ran } | noPlace | noAnswer
/// ```
pub async fn run_an_answer_to_a_client<S: Send + 'static, T: Send + 'static>(ceiling: Duration, state: S, work: fn(S) -> T) -> TheAnswerOfAJobAwaited<T> {
    let Some(answering) = budget().answering.as_ref() else {
        // A process of two places or fewer has no place to keep for this class, so the answer takes a general one.
        return run_a_job_awaited_no_longer_than(ceiling, state, work).await;
    };
    let place: tokio::sync::OwnedSemaphorePermit = match tokio::time::timeout(ceiling, Arc::clone(answering).acquire_owned()).await {
        Ok(Ok(taken)) => taken,
        // No place came within the ceiling, so the job never started and nothing it would have changed was changed.
        Ok(Err(_)) | Err(_) => return TheAnswerOfAJobAwaited::NotRun,
    };
    return match tokio::time::timeout(ceiling, run_a_job_holding_the_place(place, move || { return work(state); })).await {
        Ok(Ok(value)) => TheAnswerOfAJobAwaited::Ran(value),
        // Only a refusal that promised the job will not run leaves the state as it was.
        Ok(Err(BlockingJobError::NoThreadToStart)) => TheAnswerOfAJobAwaited::NotRun,
        Ok(Err(_started_anyway)) => TheAnswerOfAJobAwaited::NoAnswer,
        Err(_outlasted_its_ceiling) => TheAnswerOfAJobAwaited::NoAnswer,
    };
}

/// What:
///  Run `job` as a job of the blocking pool,
/// with a place of the process's bound held while it runs,
/// awaiting the place no longer than `ceiling` and then the job no longer than another `ceiling`.
/// Why:
///   A place is held by the job that took it until that job ends,
/// so a job whose read outlasted its own ceiling leaves the place held with no end in sight
/// (`blocking_jobs_within_the_budget.rs`).
/// A caller that waited for a place without one would then wait for a place nothing gives back,
/// which is the unbounded wait the owner forbids:
///  a service that never leaves its round runs no later push,
/// no upkeep round and never exits idle
/// (finding 4 of the review of `5686c7bb29`).
/// The two waits are told apart because they mean different things:
/// no place within the ceiling proves the job never started and changed nothing,
/// while a job that outlasted its own ceiling may have done its work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await runAJobOnceAwaitedNoLongerThan(ceiling, job);
/// ```
pub async fn run_a_job_once_awaited_no_longer_than<T: Send + 'static, J: FnOnce() -> T + Send + 'static>(ceiling: Duration, job: J) -> TheAnswerOfAJobAwaited<T> {
    // A control can wait on this ask instead of on a clock, so work that bypasses the bound cannot pass it.
    #[cfg(test)]
    let _asking: jobs_waiting_for_a_place::AJobWaits = jobs_waiting_for_a_place::note_that_a_job_waits();
    let place: tokio::sync::OwnedSemaphorePermit = match tokio::time::timeout(ceiling, Arc::clone(&budget().places).acquire_owned()).await {
        Ok(Ok(taken)) => taken,
        // No place came within the ceiling, so the job never started and nothing it would have changed was changed.
        Ok(Err(_)) | Err(_) => return TheAnswerOfAJobAwaited::NotRun,
    };
    #[cfg(test)]
    drop(_asking);
    return match tokio::time::timeout(ceiling, run_a_job_holding_the_place(place, job)).await {
        Ok(Ok(value)) => TheAnswerOfAJobAwaited::Ran(value),
        // Only a refusal that promised the job will not run leaves the state as it was.
        Ok(Err(BlockingJobError::NoThreadToStart)) => TheAnswerOfAJobAwaited::NotRun,
        Ok(Err(_started_anyway)) => TheAnswerOfAJobAwaited::NoAnswer,
        Err(_outlasted_its_ceiling) => TheAnswerOfAJobAwaited::NoAnswer,
    };
}

/// What:
///  The claim and the place of one job,
///  until the call that made them settles them.
/// Why:
///  A caller that stops waiting drops this value,
/// and the pool may still hold the job's closure queued
/// (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
/// `spawn_task`),
///  so the claim is taken here to make that closure do nothing;
/// the place goes back only then,
///  because a job that took the claim first holds it until its work ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // on drop, before the call settled: if (claim.theReporterTakesIt()) place.giveItBack();
/// ```
struct TheCallOnAJob {
    /// The claim on the job's work.
    claimed: Arc<TheClaimOnAJobsWork>,
    /// The place the job holds.
    held: Arc<ThePlaceOfOneJob>,
    /// Whether this call has already decided who gives the place back.
    settled: bool,
}

impl Drop for TheCallOnAJob {
    fn drop(&mut self) {
        if self.settled {
            return;
        }
        // The caller stopped waiting: a job the pool has not started must do nothing, and its place goes back here
        //  unless that job took the claim, in which case it gives the place back when its work ends.
        if self.claimed.the_reporter_takes_it() {
            self.held.give_it_back();
        }
    }
}

#[cfg(test)]
#[path = "jobs_waiting_for_a_place.rs"]
pub(crate) mod jobs_waiting_for_a_place;

/// The answer a control states for the jobs its own thread asks for stays out of the release executable.
#[cfg(test)]
#[path = "the_answer_a_control_states.rs"]
pub(crate) mod a_controls_answer;

/// Blocking-budget controls stay out of the release executable.
#[cfg(test)]
#[path = "blocking_jobs_within_the_budget_tests.rs"]
mod tests;
