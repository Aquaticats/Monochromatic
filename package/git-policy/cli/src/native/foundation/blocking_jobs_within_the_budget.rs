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
            if claimed.the_reporter_takes_it() {
                held.give_it_back();
            }
            call.settled = true;
            Err(failed)
        }
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

/// Blocking-budget controls stay out of the release executable.
#[cfg(test)]
#[path = "blocking_jobs_within_the_budget_tests.rs"]
mod tests;
