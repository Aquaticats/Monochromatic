//! What:
//!  What became of one Git step of a landing's own side reference:
//!  the step ran and answered,
//!  the wait for it ended without an answer,
//!  or the step was ended and the sentence it was ended with is kept for the landing that answers.
//! Why:
//!  A step the wrapper never ends is not ended for being slow
//!  (`transaction_git_step_bounds.rs`,
//!  `is_side_ref_write`),
//!  so the wait for a write can end before the write does,
//!  and that ending says the result is not known,
//!  never that the write failed
//!  (`the_write_of_a_side_reference.rs`).
//!  The sentence of a step that a stop or a ceiling ended is read by the landing's own Git steps,
//!  which answer with it (`service_land_git_steps_and_import_stream.rs`),
//!  so it is kept here beside the answer that keeps it.
//!  It sits in a file of its own because the write of a side reference was over the limit the repository linter
//!  enforces (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer: StepAnswer = answering(git, request);
//! ```

use crate::child_watch::{
    is_ceiling,
    is_ended_by_stop,
    is_not_known,
};
use crate::landing_state_carried_by_its_task::{
    LandingState,
    on_the_landing_state,
};
use crate::service_protocol::FailureCode;
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
use crate::transaction_git_awaiting::run_git_awaiting;

/// Keeps the sentence of a step that was ended at its ceiling,
/// for the landing that answers it.
pub(crate) fn keep_the_ceiling_step(
    state: &LandingState,
    sentence: String,
) {
    *state.ceiling_step.borrow_mut() = Some(sentence);
}

/// Takes the sentence of the step that was ended at its ceiling out of the landing's state.
fn take_the_ceiling_step(
    state: &LandingState,
    (): (),
) -> Option<String> {
    return state.ceiling_step.take();
}

/// The sentence of the step that was ended at its ceiling,
/// once;
/// the landing keeps it for the line the clients are told (the state `ceiling_step` of `landing_state_carried_by_its_task.rs`).
pub(crate) fn take_ceiling_step() -> Option<String> {
    return on_the_landing_state((), take_the_ceiling_step);
}

/// What became of one run of a Git step,
/// for a writer that tells a wait with no answer apart from an ending.
/// Why:
///  A step the wrapper never ends is not ended for being slow
///  (`transaction_git_step_bounds.rs`,
///  `is_side_ref_write`),
///  so the wait for it can end before the step does,
///  and that ending says the result is not known,
/// never that the step failed.
///  A write may be made once more when its answer is not known;
///  a write Git refused may not,
/// and a step a stop or a ceiling ended keeps the code the landing answers with.
///  The step runner of the landing reads it,
/// so a write that asks the reference itself tells the two apart
///  (`service_land_git_steps_and_import_stream.rs`,
///  `run`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StepAnswer = { answered: GitOutput } | 'left-running' | { ended: FailureCode }
/// ```
pub(crate) enum StepAnswer {
    /// Git ran and answered.
    Answered(GitOutput),
    /// The command stopped waiting for the step,
    /// which is left running,
    /// so its result is not known.
    LeftRunning,
    /// The step was ended,
    /// or Git could not be run,
    /// and the code says which.
    Ended(FailureCode),
}

/// What:
///  Runs one Git step and says what its answer was,
/// keeping the sentence of a step that was ended.
/// Why:
///  `run` answers a failure code and nothing more,
///  and a writer that may be made again needs the difference between a wait that ended without an answer and a write Git
///  refused (`StepAnswer`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function answering(git, request): StepAnswer { try { return { answered: await runGit(git, request) } } catch (error) { if (isNotKnown(error)) return 'left-running'; if (isCeiling(error) || isEndedByStop(error)) { rememberCeilingStep(error.message); return { ended: 'StepTooSlow' } } return { ended: 'Unavailable' } } }
/// ```
pub(crate) fn answering(
    git: &GitContext,
    request: GitRequest,
) -> StepAnswer {
    return match run_git(git, &request) {
        Ok(output) => StepAnswer::Answered(output),
        Err(error) if is_not_known(&error) => StepAnswer::LeftRunning,
        Err(error) => {
            // A step ended at its ceiling, or by the service's stop, is told apart from every other failure, and its sentence is kept
            //  for the answer; the commit it belongs to is kept on a side ref and not made (`land_batch_timed`).
            if is_ceiling(&error) || is_ended_by_stop(&error) {
                on_the_landing_state(error.to_string(), keep_the_ceiling_step);
                return StepAnswer::Ended(FailureCode::StepTooSlow);
            }
            StepAnswer::Ended(FailureCode::Unavailable)
        }
    };
}

/// What:
///  `answering` with the step's Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
///  and a side reference is written from the landing once the landing is a task;
/// the three answers and the sentence kept for a step that was ended
///  are the synchronous face's,
/// so a commit one face keeps is kept by the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function answeringAwaiting(git, request): StepAnswer { try { return { answered: await runGitAwaiting(git, request) } } catch (error) { if (isNotKnown(error)) return 'left-running'; if (isCeiling(error) || isEndedByStop(error)) { rememberCeilingStep(error.message); return { ended: 'StepTooSlow' } } return { ended: 'Unavailable' } } }
/// ```
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller yet: the landing's body calls the synchronous face until it is a task (doc/planning/git-policy-cli-full-tokio.md)"
    )
)]
pub(crate) async fn answering_awaiting(
    git: &GitContext,
    request: GitRequest,
) -> StepAnswer {
    return match run_git_awaiting(git, &request).await {
        Ok(output) => StepAnswer::Answered(output),
        Err(error) if is_not_known(&error) => StepAnswer::LeftRunning,
        Err(error) => {
            if is_ceiling(&error) || is_ended_by_stop(&error) {
                on_the_landing_state(error.to_string(), keep_the_ceiling_step);
                return StepAnswer::Ended(FailureCode::StepTooSlow);
            }
            StepAnswer::Ended(FailureCode::Unavailable)
        }
    };
}
