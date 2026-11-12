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
//! The in-process answer is theirs as well,
//! but only this face reads it as a job of the blocking pool with a ceiling of its own
//!  (`in_process_answer_awaiting`):
//! its readers walk the file system, and a worker of the runtime inside that walk would hold the tasks that
//!  would end its wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { stdout } = await runTransactionGitOnTheRuntime({ gitPath, cwd, args, indexPath, input });
//! ```

/// Building a Git child from an argument array and an overlay,
/// the private input file and the subject a notice names.
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, TermsOfRequest, WatchTermsSource, build_command, checked_from, command_text, input_file, refused_by_the_system, step_subject};
/// The rules for which steps may be run again or ended.
use crate::transaction_git_step_rerun_and_end_rules::{is_branch_write, may_be_rerun};
/// The bound of a step and the terms a request's class gives it.
use crate::transaction_git_step_bounds::step_bound;
/// An in-process answer to an exact read-only command line.
use crate::repo_read_hook::try_request;
/// The place the in-process read takes when one of the bound's is free,
/// and the answer that says it did not run.
use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_if_a_place_is_free_within};
/// The longest that read may take before Git answers the same command line.
use crate::wait_ceilings::IN_PROCESS_ANSWER_CEILING;
/// The watched run for a caller that is a task of the runtime.
use crate::child_watch_output_watched_on_the_runtime::output_watched_within_for_a_stop_awaiting;
/// The bound a step that is never ended is left running at,
/// and the value a dial of the watch reads.
use crate::child_watch::StepBound;
/// Starts that ride out a short shortage of processes,
/// awaited on the runtime's timer.
use crate::start_retry::{RefusableAwaitingStart, START_RETRY_BUDGET, retry_refused_awaiting_start};
/// The command the child is started from and its streams,
/// and the write of a redo's notice to standard error.
use std::io::Write;
use std::process::{Command, Stdio};
/// A span of time,
/// for the retry budget.
use std::time::Duration;

/// What:
///  The in-process answer to a request,
/// read as a job of the process's blocking pool when one of its places is free,
/// and awaited no longer than `IN_PROCESS_ANSWER_CEILING`,
/// or nothing at all when the readers have no answer or no place was free.
/// Why:
///   The readers walk the file system,
/// including the real path of the repository's own directory,
/// which waits in the kernel on a store that stopped answering,
/// and a worker of the runtime inside that read cannot be ended by any timer:
/// no ceiling can fire while the future is not polled
/// (the review of `259f83cbab`, the route's fallback repeating its own layout reading in process,
/// on the worker, outside every ceiling).
/// So the read runs on a thread of the pool,
/// and its absence is the same nothing the readers answer with when they decline a command line:
/// Git starts, and the step's own watch and ceiling end a start that does not answer.
///   The answer is a saving rather than the only way to it,
/// so the job also waits for no place:
/// a round holding the bound would otherwise delay a commit for a step the commit does not need,
/// and a busy pool is answered by Git in the time a Git start takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = await inProcessAnswerAwaiting(context, request); // undefined: start Git
/// ```
async fn in_process_answer_awaiting(context: &GitContext, request: &GitRequest) -> Option<GitOutput> {
    // The job owns what it reads, so nothing of the request is borrowed across the pool's thread.
    let question: (GitContext, GitRequest) = (context.clone(), request.clone());
    return match run_a_job_if_a_place_is_free_within(IN_PROCESS_ANSWER_CEILING, move || { return try_request(&question.0, &question.1); }).await {
        TheAnswerOfAJobAwaited::Ran(answered) => answered,
        // No place was free, or the job outlasted its ceiling and may still be inside a read nobody can end: Git answers.
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => None,
    };
}

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
///  The in-process answer is asked first, as on the synchronous face,
/// but as a job of the blocking pool and with a ceiling of its own
/// (`in_process_answer_awaiting`),
/// and the timing marks are the synchronous face's,
/// so the two cannot differ in what they record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGitOnTheRuntime({ ..., allowFailure: true })
/// ```
pub async fn run_git_awaiting(context: &GitContext, request: &GitRequest) -> std::io::Result<GitOutput> {
    // An exact read-only command line the in-process readers know costs no Git process.
    if let Some(answered) = in_process_answer_awaiting(context, request).await {
        return Ok(answered);
    }
    let label: String = crate::add_timing::git_step_label(request.arguments.as_slice());
    let entered: bool = crate::add_timing::begin_step(label.as_str());
    let output: std::io::Result<GitOutput> = run_git_pausing_awaiting(context, request, START_RETRY_BUDGET).await;
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
async fn run_git_pausing_awaiting(context: &GitContext, request: &GitRequest, budget: Duration) -> std::io::Result<GitOutput> {
    if !may_be_rerun(request) {
        return run_git_once_awaiting(context, request).await;
    }
    let what: String = command_text(&request.arguments);
    let mut attempt: RefusableGitRunAwaiting<'_> = RefusableGitRunAwaiting { context, request, refused: None };
    let result: std::io::Result<GitOutput> = retry_refused_awaiting_start(what.as_str(), budget, &mut attempt).await;
    return match (result, attempt.refused.take()) {
        (Ok(finished), _) => Ok(finished),
        (Err(error), Some(last)) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(last),
        (Err(error), _) => Err(error),
    };
}

/// What:
///  One awaited run of a request that the retry may start again:
/// the run,
/// and the output Git gave when the system refused it a process.
/// Why:
///   The same value the synchronous face's `RefusableGitRun` is,
/// for the awaiting retry:
/// a refusal is told to the retry as `WouldBlock`
///  and the output that carried it is kept here,
/// so that after the budget the caller returns Git's own last words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RefusableGitRun implements RefusableAwaitingStart<GitOutput> { refused?: GitOutput; async start() {} }
/// ```
struct RefusableGitRunAwaiting<'a> {
    /// The Git program and the directories the run uses.
    context: &'a GitContext,
    /// What to run.
    request: &'a GitRequest,
    /// The last output Git gave when the system refused it a process.
    refused: Option<GitOutput>,
}

impl RefusableAwaitingStart<GitOutput> for RefusableGitRunAwaiting<'_> {
    async fn start(&mut self) -> std::io::Result<GitOutput> {
        let output: GitOutput = match run_git_once_awaiting(self.context, self.request).await {
            Ok(finished) => finished,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Err(std::io::Error::other(error)),
            Err(error) => return Err(error),
        };
        if refused_by_the_system(&output) {
            self.refused = Some(output);
            return Err(std::io::Error::from(std::io::ErrorKind::WouldBlock));
        }
        return Ok(output);
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
async fn run_git_once_awaiting(context: &GitContext, request: &GitRequest) -> std::io::Result<GitOutput> {
    return run_git_once_with_awaiting(context, request, &TermsOfRequest { request }).await;
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
async fn run_git_once_with_awaiting<Terms: WatchTermsSource + Sync>(context: &GitContext, request: &GitRequest, terms: &Terms) -> std::io::Result<GitOutput> {
    let first: std::io::Result<GitOutput> = run_git_watched_awaiting(context, request, terms.terms()).await;
    return match first {
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut && may_be_rerun(request) && !crate::child_watch::is_ceiling(&error) => {
            // Terminal output error: a notice is advisory; a failed write must not stop the commit.
            let _written = writeln!(
                std::io::stderr(),
                "monochromatic-git-policy-cli: {} showed no sign of work and was ended; running it again",
                step_subject(request)
            );
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
async fn run_git_watched_awaiting(context: &GitContext, request: &GitRequest, terms: crate::child_watch::WatchTerms) -> std::io::Result<GitOutput> {
    let mut command: Command = build_command(context, request);
    match &request.input {
        Some(bytes) => {
            command.stdin(Stdio::from(input_file(bytes.as_slice(), context.common_dir.as_deref())?));
        }
        None => {
            command.stdin(Stdio::null());
        }
    }
    // The class label travels in the subject, so the notices and the timeout sentence name the kind of step.
    let subject: String = step_subject(request);
    let ledger_step: String = crate::add_timing::git_step_label(request.arguments.as_slice());
    let bound: StepBound = step_bound(request);
    // A write that is never ended is recorded for the clients that queue behind it (`service_busy.rs`); the record ends with the step.
    let _recorded = if matches!(bound, StepBound::Unbounded) {
        crate::service_busy_step_record_on_this_thread::enter_step(format!("{} in {}", command_text(&request.arguments), request.cwd.display()))
    } else {
        None
    };
    // The step is written down whatever its outcome: a step that failed or was ended spent its time too.
    let began: std::time::Instant = std::time::Instant::now();
    // A stop of the service lets the branch write finish (owner, 2026-11-08, "B: let a branch write finish (Recommended)").
    let a_stop_lets_it_finish: bool = matches!(bound, StepBound::Unbounded) && is_branch_write(request);
    let watched = output_watched_within_for_a_stop_awaiting(&mut command, subject.as_str(), ledger_step.as_str(), terms, may_be_rerun(request), bound, a_stop_lets_it_finish).await;
    crate::transaction_step_durations::note_step(ledger_step.as_str(), began, began.elapsed());
    let output: std::process::Output = watched?;
    return Ok(GitOutput {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    });
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
pub async fn run_git_checked_awaiting(context: &GitContext, request: &GitRequest) -> Result<GitOutput, GitFailure> {
    return checked_from(run_git_awaiting(context, request).await, request.arguments.as_slice());
}

/// Awaiting-run controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_git_awaiting_tests.rs"]
mod tests;
