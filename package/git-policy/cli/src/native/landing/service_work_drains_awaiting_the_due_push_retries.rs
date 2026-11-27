//! What:
//!  The pushes a round runs again because their retry is due,
//!  awaited by a caller that is itself a task of the runtime:
//!  one push per due record,
//!  a notice for the person on each way it can end,
//!  the plan the next try is kept in,
//!  and the programs a record held in memory given up with it.
//! Why:
//!   A push's retry is due when its own plan says so,
//!  and a record nothing will run keeps nothing,
//!  so each way a push ends gives up what its record held.
//!  They live beside the round that drains the queue,
//!  because that file is at the line bound the repository linter enforces (`rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // landing/serviceWorkDrainsAwaitingTheDuePushRetries.ts
//! export const runDuePushRetriesAwaiting = async (gitDir, handlers, candidates, replanning) => ...;
//! ```

use crate::push_failure_notice::PushOutcome;
use crate::push_retry::{
    Due,
    FIRST_PAUSE,
};
use crate::push_retry_cause_line::cause_or_silence;
use crate::push_retry_failure::{
    PROTECTED_BRANCH_TRIES,
    out_of_tries,
    out_of_tries_words,
};
use crate::service_work_handlers_replanning_and_acceptance::{
    PlannedRetry,
    Replanning,
    Verdict,
    WorkHandlers,
    is_acceptable,
    planned_retry,
};
use crate::service_work_storage_awaited::{
    clear_push_failures_before_awaiting,
    record_push_failure_awaiting,
    record_push_report_awaiting,
    record_push_skip_awaiting,
};
use std::path::Path;
use std::time::{
    Duration,
    Instant,
};

/// What:
///  Gives up the programs and transport settings the push `due` carries,
///  because that record can no longer run.
/// Why:
///   A due retry runs the plan's own item rather than a claimed record,
/// so it never reaches the settling a claimed push goes through,
/// and nothing else here would give the values up:
/// one of them can hold a password,
/// a plan queues the very same record id,
/// and a record nothing will run keeps nothing (`service_push_programs_held_in_memory`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const forgetTheProgramsOf = (due) => forget(due.gitDir, due.item.id());
/// ```
fn forget_the_programs_of(due: &Due) {
    crate::service_push_programs_held_in_memory::forget(
        due.git_dir.as_path(),
        due.item.id().as_str(),
    );
}

/// What:
///  Run the pushes that are due again,
/// awaited by a caller that is itself a task of the runtime:
/// one push per due record,
/// a notice
///  for the person on each way it can end,
/// and the plan the next try is kept in.
/// Why:
///   A push starts Git,
/// so it is asked through the handlers' awaited face;
/// what it leaves for the person is a file,
///  so each record is a job of the pool;
/// and the words of a retry are planned by one function the synchronous face uses too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runDuePushRetriesAwaiting(gitDir, handlers, candidates, replanning)
/// ```
pub(crate) async fn run_due_push_retries_awaiting<H: WorkHandlers + Sync + ?Sized>(
    git_dir: &Path,
    handlers: &H,
    candidates: &[Due],
    replanning: &mut Replanning,
) {
    for due in candidates {
        if due.git_dir != git_dir {
            continue;
        }
        let branch: &str = due.item.branch_ref.as_str();
        let commit: &str = due.item.oid.as_str();
        if !is_acceptable(&due.item, git_dir) {
            crate::service_warn!(
                "the push of {branch} {} is not tried again: its repository or branch record is no longer acceptable",
                crate::service_timing::in_the_repository_at(due.item.repository.as_path())
            );
            replanning.forget(git_dir, branch).await;
            forget_the_programs_of(due);
            continue;
        }
        let pushing_since: std::time::SystemTime = std::time::SystemTime::now();
        let began: Instant = Instant::now();
        let verdict: Verdict = handlers
            .push_awaiting(std::slice::from_ref(&due.item))
            .await;
        match verdict {
            Verdict::Done => {
                clear_push_failures_before_awaiting(git_dir, branch, pushing_since).await;
                replanning.forget(git_dir, branch).await;
                // The push landed on this plan's last try of the branch, so this record can no longer run and keeps no value. No plan
                //  carries it any more: the file went just above.
                forget_the_programs_of(due);
                crate::service_log!(
                    "the push of {branch:?} {} landed on try {} push_ms={}",
                    crate::service_timing::in_the_repository_at(due.item.repository.as_path()),
                    due.attempt + 1,
                    began.elapsed().as_millis()
                );
            }
            Verdict::Delivered {
                commit: sent,
                said,
            } => {
                // This push landed on the branch's retry plan, so its account is a record the next command shows exactly as the
                //  round's own push leaves one: without it a server's line about a push the service made after a failure would
                //  reach nobody, and nothing makes an accepted push say it again. The step itself says in the journal what it could
                //  not record, with the words.
                if !said.is_empty() {
                    let _recorded: bool =
                        record_push_report_awaiting(git_dir, branch, sent.as_str(), said.as_str())
                            .await
                            .is_ok();
                }
                clear_push_failures_before_awaiting(git_dir, branch, pushing_since).await;
                replanning.forget(git_dir, branch).await;
                // The plan's last try landed the branch, so this record can no longer run and keeps no value.
                forget_the_programs_of(due);
                crate::service_log!(
                    "the push of {branch:?} {} landed on try {} push_ms={}",
                    crate::service_timing::in_the_repository_at(due.item.repository.as_path()),
                    due.attempt + 1,
                    began.elapsed().as_millis()
                );
            }
            Verdict::Retry(text) => {
                // What:
                //  A push the service has given every try it gives a protected-branch refusal is reported as failed,
                //  with what the server
                //  said and how long the tries covered.
                // Why:
                //  That refusal can clear by itself, so it is retried;
                // and a rule of the branch that refuses every push (an unsigned commit on a
                //  branch that requires signatures) refuses every try as well,
                // so the tries end and the branch waits for the person
                //  instead of for ever.
                //  The words are the same whether or not this round keeps a plan,
                // so a drain that reports a retryable failure as failed still names what the server said.
                if out_of_tries(text.as_str(), due.attempt + 1) {
                    let said: String = format!(
                        "{}\n{}",
                        out_of_tries_words(replanning.first.unwrap_or(FIRST_PAUSE)),
                        text
                    );
                    crate::service_warn!(
                        "the push of {branch:?} failed on try {} of {PROTECTED_BRANCH_TRIES} and needs you: {}",
                        due.attempt + 1,
                        cause_or_silence(text.as_str())
                    );
                    // The plan is the only durable record that this push is waiting for the person, so it goes only once the refusal
                    //  is written: a kill between the two would otherwise leave neither the plan nor the notice, nothing would try the
                    //  branch again, and the server's refusal would be said to nobody. A refusal the storage refused stays in the plan
                    //  for a later look, which asks the server again and says the same thing.
                    let recorded: Result<(), String> = record_push_failure_awaiting(
                        git_dir,
                        branch,
                        commit,
                        PushOutcome::Refused,
                        said.as_str(),
                    )
                    .await;
                    match recorded {
                        Ok(()) => {
                            replanning.forget(git_dir, branch).await;
                            // The person must act now, so the plan ends here and this record keeps no value of the command that asked for
                            //  its last try.
                            forget_the_programs_of(due);
                        }
                        Err(reason) => {
                            crate::service_error!(
                                "the failed push of {commit} could not be recorded for the person: {reason}"
                            );
                            // The refusal could not be written, so the plan stays: its next look asks the server again and says the same
                            //  thing. It is planned again so that the look comes after a pause and not at once.
                            if let Some(first) = replanning.first {
                                let _planned: PlannedRetry = planned_retry(
                                    git_dir,
                                    replanning,
                                    first,
                                    &due.item,
                                    due.attempt + 1,
                                    said.as_str(),
                                )
                                .await;
                            }
                        }
                    }
                    continue;
                }
                // The notice says what happens next, so it has to be what this round did: a plan kept, or none at all.
                let first_pause: Option<Duration> = replanning.first;
                let planned: Option<PlannedRetry> = match first_pause {
                    Some(pause) => Some(
                        planned_retry(
                            git_dir,
                            replanning,
                            pause,
                            &due.item,
                            due.attempt + 1,
                            text.as_str(),
                        )
                        .await,
                    ),
                    None => None,
                };
                let (said, outcome): (String, PushOutcome) = match planned {
                    // The framing promises a retry only when the plan the words speak of exists.
                    Some(kept_plan) => (
                        kept_plan.words,
                        if kept_plan.kept {
                            PushOutcome::Unfinished
                        } else {
                            PushOutcome::UnfinishedWithNoRetry
                        },
                    ),
                    None => (text, PushOutcome::UnfinishedWithNoRetry),
                };
                let keeps_a_plan: bool = first_pause.is_some();
                if let Err(reason) =
                    record_push_failure_awaiting(git_dir, branch, commit, outcome, said.as_str())
                        .await
                {
                    crate::service_error!(
                        "the failed push of {commit} {} could not be recorded for the person: {reason}",
                        crate::service_timing::in_the_repository_at(due.item.repository.as_path())
                    );
                }
                // The record goes before the plan's file, as it does for every way a push ends here: a service killed
                //  between the two leaves a plan a later one tries again, never a notice that promises a retry nothing
                //  schedules.
                if !keeps_a_plan {
                    replanning.forget(git_dir, branch).await;
                    // No plan carries this push, so nothing runs its record again and the values of the command that asked for this try
                    //  are given up with it. A kept plan queues this same record id, so that ending alone keeps them.
                    forget_the_programs_of(due);
                }
            }
            Verdict::Failed(text) => {
                crate::service_warn!(
                    "the push of {branch:?} {} failed on try {} and needs you: {}",
                    crate::service_timing::in_the_repository_at(due.item.repository.as_path()),
                    due.attempt + 1,
                    cause_or_silence(text.as_str())
                );
                // The record of a refusal the person must act on is the last word about this push, so it is written
                //  before the plan's file goes: a kill between the two then leaves the plan, and a later service tries
                //  the push again instead of the person waiting for a try nothing schedules.
                if let Err(reason) = record_push_failure_awaiting(
                    git_dir,
                    branch,
                    commit,
                    PushOutcome::Refused,
                    text.as_str(),
                )
                .await
                {
                    crate::service_error!(
                        "the failed push of {commit} {} could not be recorded for the person: {reason}",
                        crate::service_timing::in_the_repository_at(due.item.repository.as_path())
                    );
                }
                replanning.forget(git_dir, branch).await;
                // The person must act, so the plan ends here whether or not its words could be written, and a record no plan carries
                //  keeps no value.
                forget_the_programs_of(due);
            }
            Verdict::Skipped(text) => {
                if !text.is_empty()
                    && let Err(reason) =
                        record_push_skip_awaiting(git_dir, branch, commit, text.as_str()).await
                {
                    crate::service_error!(
                        "the skipped push of {commit} {} could not be recorded for the person: {reason}",
                        crate::service_timing::in_the_repository_at(due.item.repository.as_path())
                    );
                }
                // The skip's words go before the plan's file, as every other record of a push's end does here. Nothing ran and no plan
                //  carries the push, so the values of the command that asked for this try are given up with it.
                replanning.forget(git_dir, branch).await;
                forget_the_programs_of(due);
            }
        }
    }
    return;
}
