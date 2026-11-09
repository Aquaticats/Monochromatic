//! What:
//!  Runs a blocking call on tokio's blocking pool and turns "the system gave no thread" into an error value.
//! Why:
//!  `tokio::task::spawn_blocking` (and so every `tokio::fs` call) panics,
//! not errors,
//! when the operating system refuses a thread
//!  and no pool thread exists.
//! The owner chose "Fail plainly" (`doc/planning/git-policy-cli-full-tokio.md`,
//! choice 11):
//! the
//!  call fails with a plain message and never runs inline on the calling task,
//! where no timer could end it.
//!
//! A refusal here proves less than it looks:
//! the pool pushes the job into its queue *before* it asks for a thread
//! (`tokio-1.54.1/src/runtime/blocking/pool.rs`,
//!  `spawn_task`:
//! the task is pushed at line 624 and the thread asked for at
//!  line 628),
//! so the closure this call gave up on stays queued and the pool's next thread
//!  runs it.
//! This runner therefore claims only that the step may still run
//!  (`BlockingJobError::MayStillRunAfterItsThreadWasRefused`).
//! The strong answer,
//! that the step will not run and changed nothing,
//!  belongs to the caller that takes the claim on the step's work before it
//! answers (`blocking_jobs_within_the_budget.rs`,
//! `the_claim_a_job_takes_before_running_it.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Node's libuv pool refuses nothing; here the refusal is a value:
//! const result = await runBlockingJob(() => readFileSync(path)); // Ok(value) | Err(MayStillRunAfterItsThreadWasRefused)
//! ```

/// Why a blocking job did not give its value.
#[derive(Debug, Eq, PartialEq)]
pub enum BlockingJobError {
    /// The system refused a thread and the pool had none to run the job on,
    /// and the job will do nothing:
    ///  only a caller that takes the claim on the job's work
    /// (`the_claim_a_job_takes_before_running_it.rs`)
    ///  before this answer can say so,
    /// and only such a caller produces this value
    /// (`blocking_jobs_within_the_budget.rs`,
    /// `a_refused_thread_start`).
    NoThreadToStart,
    /// The system refused a thread for the job's start,
    ///  and the pool had already queued the job before it asked,
    /// so the job may still run later on another thread:
    /// whether it ran is not known from here.
    ///  This is what a runner that takes no claim on the job's work can say,
    /// and nothing more (`run_blocking_job`).
    MayStillRunAfterItsThreadWasRefused,
    /// The system refused a thread for the job's start,
    ///  and the pool then ran the job on another thread:
    /// it has started,
    ///  and whether it finished is not known from here.
    StartedAfterItsThreadWasRefused,
    /// The job ran and panicked.
    Panicked,
    /// The runtime ended before the job did.
    Cancelled,
}

impl std::fmt::Display for BlockingJobError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            BlockingJobError::NoThreadToStart => write!(
                formatter,
                "the system refused a thread for a blocking step, and this call took the claim on that step's work before it answered, \
                 so the step will not run and changed nothing. Hint: check the process limit (`ulimit -u`) and the control group's \
                 `pids.max`, free some, and run the command again"
            ),
            BlockingJobError::MayStillRunAfterItsThreadWasRefused => write!(
                formatter,
                "the system refused a thread for a blocking step, and the pool had already queued that step before it asked for one, so \
                 the step may still run later; whether it ran is not known from here. Hint: check the process limit (`ulimit -u`) and \
                 the control group's `pids.max`, and check what the step was doing before running the command again"
            ),
            BlockingJobError::StartedAfterItsThreadWasRefused => write!(
                formatter,
                "the system refused a thread for a blocking step, and the pool then ran that step on another thread: it has \
                 started, and whether it finished is not known from here. Hint: check the process limit (`ulimit -u`) and the \
                 control group's `pids.max`, and check what the step was doing before running the command again"
            ),
            BlockingJobError::Panicked => write!(formatter, "a blocking step panicked; this is a defect of monochromatic-git-policy-cli"),
            BlockingJobError::Cancelled => write!(formatter, "a blocking step was cancelled because the runtime was ending"),
        };
    }
}

/// What:
///  Runs `job` on the blocking pool and waits for its value.
/// Why:
///  The pool keeps a call that can wait on storage off the workers that poll tasks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = await runBlockingJob(() => syncAll(file));
/// ```
pub async fn run_blocking_job<T, J>(job: J) -> Result<T, BlockingJobError>
where
    T: Send + 'static,
    J: FnOnce() -> T + Send + 'static,
{
    // `spawn_blocking` panics where no thread can start, and the pool has already queued the closure at that point
    //  (`tokio-1.54.1/src/runtime/blocking/pool.rs`, `spawn_task`), so a refused job is still queued to run later. Nothing here
    //  can stop it, so the answer says it may still run and claims no state of the step's work.
    let started = crate::panic_notice::while_panics_are_expected(job, tokio::task::spawn_blocking::<J, T>);
    let Ok(handle) = started else {
        return Err(BlockingJobError::MayStillRunAfterItsThreadWasRefused);
    };
    return match handle.await {
        Ok(value) => Ok(value),
        Err(joined) if joined.is_panic() => Err(BlockingJobError::Panicked),
        Err(_cancelled) => Err(BlockingJobError::Cancelled),
    };
}

/// Blocking-job controls stay out of the release executable.
#[cfg(test)]
#[path = "blocking_job_tests.rs"]
mod tests;
