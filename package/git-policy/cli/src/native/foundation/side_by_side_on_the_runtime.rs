//! What:
//!  Run independent items as jobs of the process's blocking pool,
//! all of them beside each other or one beside the calling task's own work,
//!  and give their answers back in the order the items were given.
//! Why:
//!  The scoped threads of `side_by_side_threads.rs` lend the caller's borrows for the length of one
//!  thread scope,
//! which a job of the blocking pool cannot do:
//! a job owns what it reads.
//! The slices of a copy,
//! the workers of a removal and the two manifest walks of the final check are waits on storage,
//! so they belong on the blocking pool and never on a worker that polls tasks
//! (`doc/planning/git-policy-cli-full-tokio.md`,
//! "Blocking files,
//! fsync and the runtime's own shutdown").
//! Each item travels as owned state,
//! and every job takes a place of the process's one bound on blocking jobs
//! (`blocking_jobs_within_the_budget.rs`),
//! so a fan-out of many items cannot run more work at once than the process allows.
//! A thread the system refuses is not a failure of the item:
//! `run_a_job_or_in_turn` runs that work on the awaiting task instead,
//! as `start_or_run_with` runs it on a scoped thread.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answers = await Promise.allSettled(states.map((state) => runBlockingJobWithinTheBudget(() => job(state))));
//! const [answer, here] = await Promise.all([runBlockingJobWithinTheBudget(() => job(state)), callerJob(callerState)]);
//! ```

use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;

/// No answer yet:
/// what a slot of the run holds until its item has answered.
/// Why:
///   The answers are collected by index,
/// so a slot must name the state before any arrives,
/// and a named function stands where a closure would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noAnswerYet = () => undefined;
/// ```
fn no_answer_yet<T>() -> Option<Result<T, BlockingJobError>> {
    return None;
}

/// What:
///  Run `job` once for each state as one job of the blocking pool,
/// every one of them beside the others,
/// and return the answers in the order of the states.
/// Why:
///  The items of a copy,
/// a removal and a final check are independent,
/// so they run side by side and the person waits
///  for the slowest of them.
/// A refused thread is that item's work done in turn,
/// and a panic of one item is an error in its place,
///  as a panicked scoped thread was:
/// what to make of it is the caller's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answers = await Promise.allSettled(states.map((state) => runBlockingJobWithinTheBudget(() => job(state))));
/// ```
pub async fn run_all_side_by_side_on_the_runtime<S, T>(states: Vec<S>, job: fn(S) -> T) -> Vec<Result<T, BlockingJobError>>
where
    S: Clone + Send + 'static,
    T: Send + 'static,
{
    let count: usize = states.len();
    let mut started: tokio::task::JoinSet<(usize, Result<T, BlockingJobError>)> = tokio::task::JoinSet::new();
    for (index, state) in states.into_iter().enumerate() {
        started.spawn(async move { return (index, run_a_job_or_in_turn(state, job).await); });
    }
    let mut answers: Vec<Option<Result<T, BlockingJobError>>> = std::iter::repeat_with(no_answer_yet).take(count).collect();
    while let Some(joined) = started.join_next().await {
        // An item's task that panicked outside the job it awaited answered nothing, and its slot says so below.
        if let Ok((index, answer)) = joined {
            answers[index] = Some(answer);
        }
    }
    let mut outcomes: Vec<Result<T, BlockingJobError>> = Vec::new();
    for answer in answers {
        outcomes.push(answer.unwrap_or(Err(BlockingJobError::Panicked)));
    }
    return outcomes;
}

/// The job of `run_beside_the_caller_on_the_runtime`:
/// the state it reads and the function that reads it.
/// Why:
///   The two halves of one call travel as one value,
/// as the scoped form's are,
/// so a caller names them in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AJob = [state, (state) => answer];
/// ```
pub type AJob<S, T> = (S, fn(S) -> T);

/// What:
///  Run `job(state)` as a job of the blocking pool while `caller_work(caller_state)` runs on the
///  calling task,
/// and return both answers,
///  the job's first.
/// Why:
///   Two independent reads (the source's manifest walk beside the stage's) take as long as the slower
///  one.
/// The calling task's function runs after the job has started,
/// so a thread the system refuses leaves that job's work
///  done in turn before the caller's begins,
/// as the scoped form does.
/// The caller's function is not `Send`,
/// which is why the two are joined here and never spawned:
/// the runtime drives this one future,
///  and the caller's borrow stays on its own task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [answer, here] = await Promise.all([runBlockingJobWithinTheBudget(() => job(state)), callerWork(callerState)]);
/// ```
pub async fn run_beside_the_caller_on_the_runtime<S, T, C, U>(job: AJob<S, T>, caller: AJob<C, U>) -> (Result<T, BlockingJobError>, U)
where
    S: Clone + Send + 'static,
    T: Send + 'static,
{
    let (state, work): AJob<S, T> = job;
    let (caller_state, caller_work): AJob<C, U> = caller;
    let (answer, here): (Result<T, BlockingJobError>, U) = tokio::join!(
        run_a_job_or_in_turn(state, work),
        async move { return caller_work(caller_state); },
    );
    return (answer, here);
}

/// Side-by-side controls on the runtime stay out of the release executable.
#[cfg(test)]
#[path = "side_by_side_on_the_runtime_tests.rs"]
mod tests;
