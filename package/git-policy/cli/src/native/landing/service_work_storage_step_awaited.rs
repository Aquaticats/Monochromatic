//! What:
//!  How one storage step of a drain is run:
//! as a job of the process's blocking pool,
//! awaited no longer than the ceiling the caller states,
//! with the words a step that got no place and a step that outlasted it are refused with,
//! and the ceiling this process's claim steps take,
//! which a control may state for one round.
//! Why:
//!  Every step of the drain reads or renames files and may wait on a disk,
//! so none of them runs on a thread of the runtime;
//! the steps themselves are declared beside this file
//! (`service_work_storage_claims_awaited.rs` and `service_work_storage_awaited.rs`)
//! and each asks this one place to run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = await aStorageStep(ceiling, 'the settling of the records this round ran', 'no record is settled', job);
//! ```

use crate::blocking_jobs_within_the_budget::{
    TheAnswerOfAJobAwaited,
    run_a_job_once_awaited_no_longer_than,
};
use crate::wait_ceilings::AFTER_COMMIT_STATE_CEILING;
use std::time::Duration;

/// What:
///  Say that a storage step of the drain was refused,
/// and what that leaves for the next round.
/// Why:
///   A step that changed nothing and a step that may still be finishing are both said plainly
/// (`a_storage_step_awaited_no_longer_than`),
/// and a caller told nothing would read the silence as a drain that did all of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = (said) => serviceWarn(`${said}; the records it would have moved keep their lease`)
/// ```
pub(crate) fn refused(said: &str) {
    crate::service_warn!(
        "{said}; the records it would have moved keep their lease, so the next round finds them"
    );
}

/// What:
///  Run one storage step of a drain as a job of the process's blocking pool,
/// awaiting the place and the job no longer than `ceiling`,
/// and give its answer or the words it is refused with,
/// which name the step and what it left undone.
/// Why:
///   A place of the process's bound is held by the job that took it until that job ends,
/// and a job whose read outlasted its ceiling is left behind holding the place
/// (`blocking_jobs_within_the_budget.rs`).
/// A step that awaited a place without an end would then wait for a place nothing gives back:
/// the round would never end,
/// so no later push or upkeep round ran and the service never exited idle
/// (finding 4 of the review of `5686c7bb29`,
/// "nothing should ever wait forever or indefinitely").
/// The ceiling is the caller's so that a control can reach a step's bound in a moment,
/// as `restored_within` does for the journal's read;
/// every step the drain runs passes the ceiling the after-commit work's other state steps take.
/// Each answer is said as what it is:
/// no place within the ceiling proves the step did not run and changed nothing,
/// while a step that outlasted its own ceiling may still be finishing what it began.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await aStorageStep(ceiling, 'the settling of the records this round ran', 'no record is settled', job)
/// ```
pub(crate) async fn a_storage_step_awaited_no_longer_than<T: Send + 'static>(
    ceiling: Duration,
    step: &str,
    left: &str,
    job: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    return match run_a_job_once_awaited_no_longer_than(ceiling, job).await {
        TheAnswerOfAJobAwaited::Ran(value) => Ok(value),
        TheAnswerOfAJobAwaited::NotRun => Err(format!(
            "{step} did not run, because no place of the process's bound and no thread came within {ceiling:?}, so {left}"
        )),
        TheAnswerOfAJobAwaited::NoAnswer => Err(format!(
            "{step} gave no answer within {ceiling:?}, so {left}, and what it began may still be being written"
        )),
    };
}

/// The ceiling a control stated for the claim steps of this process,
/// in milliseconds,
/// or nothing when it stated none.
/// The release executable keeps nothing of it.
#[cfg(test)]
static A_CLAIM_CEILING_FOR_A_CONTROL: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// What:
///  State the ceiling the next claim steps of this process take,
///  so a control reaches the bound a round keeps in a moment instead of the sixty seconds a shipped step waits.
/// Why:
///   A round whose claim step got no place must ask to be looked at again,
/// and only a control can reach that answer without waiting the shipped ceiling out;
/// `claim_all_within_awaiting` states it for one caller instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdTheClaimStepsTo(20); // the next claim steps take 20 ms
/// ```
#[cfg(test)]
pub(crate) fn hold_the_claim_steps_to(ceiling: Duration) {
    A_CLAIM_CEILING_FOR_A_CONTROL.store(
        u64::try_from(ceiling.as_millis()).unwrap_or(u64::MAX),
        std::sync::atomic::Ordering::SeqCst,
    );
}

/// The ceiling this process's claim steps take:
/// the one a control stated,
/// or `AFTER_COMMIT_STATE_CEILING`,
/// which is what the after-commit work's own state steps take.
pub(crate) fn the_claim_steps_ceiling() -> Duration {
    #[cfg(test)]
    {
        let stated: u64 = A_CLAIM_CEILING_FOR_A_CONTROL.load(std::sync::atomic::Ordering::SeqCst);
        if stated > 0 {
            return Duration::from_millis(stated);
        }
    }
    return AFTER_COMMIT_STATE_CEILING;
}
