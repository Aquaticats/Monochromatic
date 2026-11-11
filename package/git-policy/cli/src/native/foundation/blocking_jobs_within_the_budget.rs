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
    /// The places a job takes and gives back.
    places: Arc<Semaphore>,
    /// How many places there are,
    /// kept so that a caller can be told the bound without taking one.
    allowed: usize,
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

/// The budget this process makes on its first need:
/// the places the budget for parallel work and the limits leave,
/// and no more than the cap the process asked
///  for.
fn decide_the_budget() -> TheBudget {
    let host: HostResources = HostResources::read();
    let limits: RuntimeLimits = RuntimeLimits::read();
    let allowed: usize = budgeted_workers(&host, &limits).min(HELD_TO.load(Ordering::SeqCst)).max(1);
    return TheBudget { places: Arc::new(Semaphore::new(allowed)), allowed };
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
    let job = run_blocking_job_within_the_budget(move || { return work(state); });
    return match tokio::time::timeout(ceiling, job).await {
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
