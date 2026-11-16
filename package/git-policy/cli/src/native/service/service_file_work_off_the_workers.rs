//! What:
//!  The one door the commit service's tasks take to reach a file:
//!  a job of the blocking pool,
//!  awaited under a ceiling,
//!  never a synchronous call on a worker of the runtime.
//! Why:
//!  A worker of the runtime polls tasks,
//! and a task that spends its turn in a file waits there for the whole time the file takes:
//!  the acceptor that would greet the next client and the connections that would beat it need that same worker.
//! With one worker,
//! which the plan of this runtime permits (`runtime_plan.rs`),
//! one such call is every client of the service,
//! and with the shipped ceilings it outlasts the five seconds of silence a client allows
//! (`doc/planning/git-policy-cli-full-tokio.md`, a blocking call runs on the blocking pool and never on a worker).
//!  The door is this service's own spelling of that:
//! the job goes to the pool through `run_a_slow_job_once_awaited_no_longer_than`,
//! which takes one of the places kept for slow work, awaits the job with the runtime's timer, and gives the place back
//! when it ends,
//! so a job that does not return takes neither a worker nor the place the steps that answer a client need,
//!  and a job that gives no answer within its ceiling is left behind with one line in the journal.
//!  Every caller says what the step is and what the service does at the ceiling,
//! because "nothing should ever wait forever" asks for the answer as well as the bound
//! (`doc/planning/cli-git-no-unbounded-waits.md`).
//!  A source guard fails when a synchronous file call or a blocking receive is written in an `async` body of this
//! service's modules outside this door (`source_guard/blocking_code_off_the_runtime_tests.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = await offTheWorkers({ what: 'the read of the plan', ceiling, work: () => readThePlan() }); // answer ?? nothing
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_slow_job_once_awaited_no_longer_than};
use std::time::Duration;

/// What a name of a step of the service reads like,
/// as the journal and a person see it.
pub struct Step {
    /// What the step does, in the words a person reads.
    pub what: &'static str,
    /// How long it may take.
    pub ceiling: Duration,
}

/// What:
///  Runs `work` on the blocking pool and answers what it gave within `step.ceiling`,
///  and nothing when it gave no answer in time or never started.
/// Why:
///   The task that calls this waits on the runtime's own timer,
/// so the worker it was polled on is free to greet and to beat the clients that need it,
/// and a step that stopped answering is left behind rather than holding the service.
///  A step that gave no answer is said once,
/// with what the caller does then,
/// which is the caller's own sentence:
/// a bound nobody can act on is not a bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await offTheWorkers(step, () => readThePlan()); if (answer === undefined) { log(`${step.what} gave no answer`); }
/// ```
pub async fn off_the_workers<T, Work>(step: Step, work: Work) -> Option<T>
where
    T: Send + 'static,
    Work: FnOnce() -> T + Send + 'static,
{
    let answer: TheAnswerOfAJobAwaited<T> = run_a_slow_job_once_awaited_no_longer_than(step.ceiling, work).await;
    return match answer {
        TheAnswerOfAJobAwaited::Ran(value) => Some(value),
        TheAnswerOfAJobAwaited::NotRun => {
            crate::service_warn!(
                "{} did not start: the system gave it no thread of the blocking pool within {:?}, so this service goes on without it",
                step.what,
                step.ceiling
            );
            None
        }
        TheAnswerOfAJobAwaited::NoAnswer => {
            crate::service_warn!(
                "{} gave no answer within {:?} and is left behind, so this service goes on without it",
                step.what,
                step.ceiling
            );
            None
        }
    };
}

/// What:
///  The plan, the watch and the identity a service starts with when the door's restore step gave no answer at all.
/// Why:
///   A restore that gave no answer has not answered "empty": the folders are known before the restore is asked and stay
/// known after it,
///  so the state says "not restored yet" over the real folders and the after-commit loop tries them again,
///  which makes a saved push late and never forgotten.
///  An empty path here would be a value invented for a question nobody answered, and the loop would read the empty
///  path's "not found" as an empty store
///  (`service_server.rs`, `start_the_after_commit_work_or_exit`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = theStateOfARestoreThatGaveNoAnswer(plan, watch, instance); // { retries: unread(plan), watch: unread(watch, instance) }
/// ```
pub fn the_state_of_a_restore_that_gave_no_answer(
    plan_directory: &std::path::Path,
    watch_directory: &std::path::Path,
    instance: Option<&str>,
) -> (crate::push_retry::PushRetries, crate::service_claim_watch::ClaimWatch) {
    return (
        crate::push_retry::PushRetries::unread_from_environment(plan_directory),
        crate::service_claim_watch::ClaimWatch::unread(watch_directory, instance),
    );
}

/// Controls of the door stay out of the release executable.
#[cfg(test)]
#[path = "service_file_work_off_the_workers_tests.rs"]
mod tests;
