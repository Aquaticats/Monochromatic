//! What:
//!  Running real Git for a caller that is itself a task of the runtime:
//! the same command,
//! the same overlay,
//! the same watch and the same bounds,
//!  with every wait a wait of the runtime.
//! Why:
//!  The owner's ruling is "Full on tokio.
//! Multi-thread."
//! (`doc/planning/git-policy-cli-full-tokio.md`),
//! and a task cannot call the synchronous face:
//!  that face registers the pipes of a step that is ended at its ceiling by entering the runtime
//! (`lazy_runtime::run_on_runtime`),
//! which is refused from inside it,
//!  because waiting for the runtime from within it would never end
//! (`RuntimeError::Nested`).
//! So this file is the same runner with the entering taken out:
//!  the pipes are registered on the runtime the caller already runs on,
//! the child is started by the awaiting retry,
//! and its end is
//!  awaited through its pidfd.
//! The synchronous face (`transaction_git.rs`) stays for the callers that have not been converted and for the ones
//! that must not be,
//!  such as a reply to a client of the service,
//! and both faces read the same pieces:
//! `build_command`,
//! `input_file`,
//! `step_subject`,
//!  `terms_for_request` and `step_bound` are theirs together,
//! and the watched run itself is worded once
//! (`push/child_watch_step_bounds_and_errors.rs`,
//! `ending_of_a_watched_step`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { stdout } = await runTransactionGitOnTheRuntime({ gitPath, cwd, args, indexPath, input });
//! ```

/// The watched run for a caller that is a task of the runtime.
use crate::child_watch_output_watched_on_the_runtime::output_watched_within_for_a_stop_awaiting;
/// An in-process answer to an exact read-only command line.
use crate::repo_read_hook::try_request;
/// Starts that ride out a short shortage of processes,
/// awaited on the runtime's timer.
use crate::start_retry::{
    RefusableAwaitingStart,
    START_RETRY_BUDGET,
    retry_refused_awaiting_start,
};
/// Building a Git child from an argument array and an overlay,
/// the private input file and the subject a notice names.
use crate::transaction_git::{
    GitContext,
    GitFailure,
    GitOutput,
    GitRequest,
    RefusableGitRun,
    TermsOfRequest,
    WatchTermsSource,
    checked_from,
    command_text,
};
/// The step both faces start,
/// and what it answers once it ended.
use crate::transaction_git_step_about_to_start::{
    StepAboutToStart,
    answer_of_a_start,
    start_failure,
    step_about_to_start,
    step_finished,
    step_is_run_again,
};
/// The rules for which steps may be run again.
use crate::transaction_git_step_rerun_and_end_rules::may_be_rerun;
/// A span of time,
/// for the retry budget.
use std::time::Duration;

/// What:
///  Run one request and capture its output,
/// whatever its exit code,
/// awaiting every wait of it on the runtime.
/// Why:
///   The asynchronous face of `transaction_git::run_git`,
/// for a caller that is a task:
/// a step that is ended at its ceiling has
///  its pipes read on the runtime,
/// and the child's end is awaited through its pidfd.
///  The in-process answer and the timing marks are the synchronous face's,
/// so the two cannot differ in what they record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGitOnTheRuntime({ ..., allowFailure: true })
/// ```
pub async fn run_git_awaiting(
    context: &GitContext,
    request: &GitRequest,
) -> std::io::Result<GitOutput> {
    // An exact read-only command line the in-process readers know costs no Git process.
    if let Some(answered) = try_request(context, request) {
        return Ok(answered);
    }
    let label: String = crate::add_timing::git_step_label(request.arguments.as_slice());
    let entered: bool = crate::add_timing::begin_step(label.as_str());
    let output: std::io::Result<GitOutput> =
        run_git_pausing_awaiting(context, request, START_RETRY_BUDGET).await;
    crate::add_timing::end_step(entered);
    return output;
}

/// What:
///  Run one request,
/// running it again within the budget when Git reports a refused process or thread
///  and the command is harmless to run twice,
/// awaiting the pause on the runtime's timer.
/// Why:
///   The synchronous face's `run_git_pausing`,
/// with `retry_refused_awaiting_start` in place of a sleep that holds the worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retryWhile(() => runGitOnce(request), (output) => refusedBySystem(output) && mayBeRerun(request));
/// ```
async fn run_git_pausing_awaiting(
    context: &GitContext,
    request: &GitRequest,
    budget: Duration,
) -> std::io::Result<GitOutput> {
    if !may_be_rerun(request) {
        return run_git_once_awaiting(context, request).await;
    }
    let what: String = command_text(&request.arguments);
    let mut attempt: RefusableGitRun<'_> = RefusableGitRun {
        context,
        request,
        refused: None,
    };
    let result: std::io::Result<GitOutput> =
        retry_refused_awaiting_start(what.as_str(), budget, &mut attempt).await;
    return match (result, attempt.refused.take()) {
        (Ok(finished), _) => Ok(finished),
        (Err(error), Some(last)) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(last),
        (Err(error), _) => Err(error),
    };
}

/// What:
///  One awaited run of a request that the retry may start again:
///  the shared start value (`RefusableGitRun`),
/// started on the runtime.
/// Why:
///   The value and the rule of one start's answer are the synchronous face's,
/// and only the run itself is awaited here,
///  so a refusal the retry waits out is read the same way on both faces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RefusableGitRun implements RefusableAwaitingStart<GitOutput> { refused?: GitOutput; async start() {} }
/// ```
impl RefusableAwaitingStart<GitOutput> for RefusableGitRun<'_> {
    async fn start(&mut self) -> std::io::Result<GitOutput> {
        // What:     `run_git_once_awaiting(...).await` runs the request on the runtime and answers the output Git gave,
        //           or the failure that ended the run.
        // Why:      This is the only difference from the synchronous start: the same decision, awaited.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const finished = await runGitOnceAwaiting(context, request);
        // ```
        let finished: GitOutput = match run_git_once_awaiting(self.context, self.request).await {
            Ok(finished) => finished,
            // What:     `start_failure(error)` makes a refused start the retry waits out and hands any other failure on.
            // Why:      The synchronous start reads its failure the same way.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // Err: return fail(startFailure(error));
            // ```
            Err(error) => return Err(start_failure(error)),
        };
        // What:     `answer_of_a_start(...)` keeps a refused run's output and asks the retry to try again.
        // Why:      The rule is the synchronous face's, and it is written once.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return answerOfAStart(this.refused, finished);
        // ```
        return answer_of_a_start(&mut self.refused, finished);
    }
}

/// What:
///  Run one request once and capture its output,
/// whatever its exit code.
/// Why:
///   The command and its input file are built for each run,
/// so every run starts from the request alone,
/// as in the
///  synchronous face.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGitOnceOnTheRuntime({ gitPath, cwd, args })
/// ```
async fn run_git_once_awaiting(
    context: &GitContext,
    request: &GitRequest,
) -> std::io::Result<GitOutput> {
    return run_git_once_with_awaiting(
        context,
        request,
        &TermsOfRequest {
            request,
        },
    )
    .await;
}

/// What:
///  `run_git_once_awaiting` where a step that shows no sign of work for the quiet window is ended and run
///  once more,
/// and one that may not be run twice is only waited for.
/// Why:
///   The synchronous face's `run_git_once_with`:
/// the owner's rule "End Git,
/// hold hooks (Recommended)" is the same,
/// and
///  the redo is limited to the steps that are safe to run twice (`may_be_rerun`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await runOnce(terms.terms()); } catch (quiet) { if (!mayBeRerun(request)) throw quiet; return await runOnce(terms.terms()); }
/// ```
async fn run_git_once_with_awaiting<Terms: WatchTermsSource + Sync>(
    context: &GitContext,
    request: &GitRequest,
    terms: &Terms,
) -> std::io::Result<GitOutput> {
    let first: std::io::Result<GitOutput> =
        run_git_watched_awaiting(context, request, terms.terms()).await;
    return match first {
        Err(error)
            if error.kind() == std::io::ErrorKind::TimedOut
                && may_be_rerun(request)
                && !crate::child_watch::is_ceiling(&error) =>
        {
            step_is_run_again(request);
            run_git_watched_awaiting(context, request, terms.terms()).await
        }
        other => other,
    };
}

/// What:
///  Start one request once under a watch and capture its output,
/// awaiting its end on the runtime.
/// Why:
///   The synchronous face's `run_git_watched` with the awaited watched run in place of the blocking one;
/// the
///  step's ledger entry,
/// the step file the clients read and the bound are the same,
/// so the two faces record the same steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runWatchedOnTheRuntime({ gitPath, cwd, args, terms, stoppable: mayBeRerun(request) })
/// ```
async fn run_git_watched_awaiting(
    context: &GitContext,
    request: &GitRequest,
    terms: crate::child_watch::WatchTerms,
) -> std::io::Result<GitOutput> {
    // What:     `step_about_to_start(...)` makes the command, the words, the terms, the bound and the record of the step,
    //           exactly as the synchronous face does.
    // Why:      The two faces must start the same step, and this is the one place that decides what it is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const step = stepAboutToStart({ context, request, terms });
    // ```
    let mut step: StepAboutToStart = step_about_to_start(context, request, terms)?;
    // What:     `output_watched_within_for_a_stop_awaiting(...)` starts the step and awaits its end on the runtime,
    //           reading its pipes as a task and ending it at its ceiling or when a stop asks.
    // Why:      This is the whole difference from the synchronous face, which waits for the same step on its own thread.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const watched = await outputWatchedWithinForAStopAwaiting(step.command, step.subject, step.ledgerStep, step.terms, step.stoppable, step.bound, step.aStopLetsItFinish);
    // ```
    let watched: std::io::Result<std::process::Output> = output_watched_within_for_a_stop_awaiting(
        &mut step.command,
        step.subject.as_str(),
        step.ledger_step.as_str(),
        step.terms,
        step.stoppable,
        step.bound,
        step.a_stop_lets_it_finish,
    )
    .await;
    // What:     `step_finished(...)` writes the step into the ledger and turns the child's output into the answer a caller reads.
    // Why:      The ledger entry and the answer are the synchronous face's, and the `?` hands a start failure on the same way.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return stepFinished(step.ledgerStep, request, step.began, watched);
    // ```
    return Ok(step_finished(
        step.ledger_step.as_str(),
        request,
        step.began,
        watched?,
    ));
}

/// What:
///  Run one request and require success,
/// awaiting every wait of it;
///  a start failure or a non-zero exit is a failure naming the command and quoting Git's standard error.
/// Why:
///   The asynchronous face of `transaction_git::run_git_checked`,
/// worded by the same `checked_from`,
/// so a task and a
///  synchronous caller word a failure the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGitOnTheRuntime({ gitPath, cwd, args }) // throws CommitTransactionGitError
/// ```
pub async fn run_git_checked_awaiting(
    context: &GitContext,
    request: &GitRequest,
) -> Result<GitOutput, GitFailure> {
    return checked_from(
        run_git_awaiting(context, request).await,
        request.arguments.as_slice(),
    );
}

/// Awaiting-run controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_git_awaiting_tests.rs"]
mod tests;
