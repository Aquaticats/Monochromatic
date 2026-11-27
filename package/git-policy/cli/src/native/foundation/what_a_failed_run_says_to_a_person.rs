//! What:
//!  Why a run of independent items on the runtime did not finish every item,
//!  in the words a person reads:
//!  what was not run,
//!  what is not known from here,
//!  and what to check and do before the command is run again.
//! Why:
//!  `parallel_runner_on_the_runtime.rs` runs the jobs and reports them in input order;
//!  the answer it gives when it cannot finish is here,
//! so the file that runs jobs carries one job,
//!  and the words of a failure stay beside each other.
//!  The awaiting form of the same run answers with the same words,
//! and a refused blocking job is turned into them in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type OrderedRunError = { noThreadToStart: true } | { panicked: true } | { cancelled: true } | { noRuntime: RuntimeError };
//! // each carries the sentence a person reads, naming what was not run
//! ```

/// The error a blocking job answers with,
/// which is turned into the words below.
use crate::blocking_job::BlockingJobError;
/// Why the runtime could not run the jobs,
/// carried by the variant that has no other reason.
use crate::lazy_runtime::RuntimeError;

/// Why an ordered run on the runtime did not finish every item.
#[derive(Debug)]
pub enum OrderedRunError {
    /// The system refused a thread for a job,
    ///  so the items after it in its lane were not run,
    /// and the item it held will not run either.
    ///  This is what a job that takes the claim on the item's work before
    /// it answers can say (`blocking_jobs_within_the_budget.rs`).
    NoThreadToStart,
    /// The system refused a thread for a job,
    ///  and the pool then ran it on another thread,
    /// so the items after it in its lane were not run,
    ///  and whether the item it held finished is not known from here.
    StartedAfterItsThreadWasRefused,
    /// The system refused a thread for a job,
    ///  and the pool had already queued it,
    /// so it may still run later;
    /// the items after it in its lane were not run,
    ///  and whether it ran is not known from here.
    MayStillRunAfterItsThreadWasRefused,
    /// A job panicked,
    /// so the items after it in its lane were not run.
    Panicked,
    /// The runtime ended before the run did.
    Cancelled,
    /// There was no runtime to run the jobs on,
    /// or the call came from inside it.
    NoRuntime(RuntimeError),
}

impl std::fmt::Display for OrderedRunError {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self {
            OrderedRunError::NoThreadToStart => write!(
                formatter,
                "the system refused a thread for a step of this run, so the items after it in its lane were not run and the \
                 item it held will not run. Hint: check the process limit (`ulimit -u`) and the control group's `pids.max`, \
                 free some, and run the command again"
            ),
            OrderedRunError::StartedAfterItsThreadWasRefused => write!(
                formatter,
                "the system refused a thread for a step of this run, and the pool then ran that step on another thread: the \
                 items after it in its lane were not run, and whether the item it held finished is not known from here. Hint: \
                 check the process limit (`ulimit -u`) and the control group's `pids.max`, and check what that worktree looks \
                 like before running the command again"
            ),
            OrderedRunError::MayStillRunAfterItsThreadWasRefused => write!(
                formatter,
                "the system refused a thread for a step of this run, and the pool had already queued that step before it asked for one, \
                 so the step may still run later; the items after it in its lane were not run, and whether it ran is not known from \
                 here. Hint: check the process limit (`ulimit -u`) and the control group's `pids.max`, and check what that worktree \
                 looks like before running the command again"
            ),
            OrderedRunError::Panicked => write!(
                formatter,
                "a step of this run panicked, so the items after it in its lane were not run"
            ),
            OrderedRunError::Cancelled => write!(
                formatter,
                "a step of this run was cancelled because the runtime was ending"
            ),
            OrderedRunError::NoRuntime(reason) => write!(formatter, "{reason}"),
        };
    }
}

impl From<BlockingJobError> for OrderedRunError {
    fn from(error: BlockingJobError) -> Self {
        return match error {
            BlockingJobError::NoThreadToStart => OrderedRunError::NoThreadToStart,
            BlockingJobError::MayStillRunAfterItsThreadWasRefused => {
                OrderedRunError::MayStillRunAfterItsThreadWasRefused
            }
            BlockingJobError::StartedAfterItsThreadWasRefused => {
                OrderedRunError::StartedAfterItsThreadWasRefused
            }
            BlockingJobError::Panicked => OrderedRunError::Panicked,
            BlockingJobError::Cancelled => OrderedRunError::Cancelled,
        };
    }
}
