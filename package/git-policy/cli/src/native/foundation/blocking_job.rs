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
//! In TS you'd write (pseudocode):
//! ```ts
//! // Node's libuv pool refuses nothing; here the refusal is a value:
//! const result = await runBlockingJob(() => readFileSync(path)); // Ok(value) | Err(NoThreadToStart)
//! ```

/// Why a blocking job did not give its value.
#[derive(Debug, Eq, PartialEq)]
pub enum BlockingJobError {
    /// The system refused a thread and the pool had none to run the job on then.
    ///  The pool queues a job before it asks for a thread,
    /// so the job has not run *yet*:
    /// nothing here says it never will,
    ///  and callers that promise a file is unchanged must take a claim first
    /// (`the_claim_a_job_takes_before_running_it.rs`).
    NoThreadToStart,
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
                "the system refused a thread for a blocking step, so it had not started when that was reported; the pool had \
                 already queued it, so it may still run later. Hint: check the process limit (`ulimit -u`) and the control \
                 group's `pids.max`, free some, and run the command again"
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
    //  (`tokio-1.54.1/src/runtime/blocking/pool.rs`, `spawn_task`), so a refused job is still queued to run later.
    let started = crate::panic_notice::while_panics_are_expected(job, tokio::task::spawn_blocking::<J, T>);
    let Ok(handle) = started else {
        return Err(BlockingJobError::NoThreadToStart);
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
