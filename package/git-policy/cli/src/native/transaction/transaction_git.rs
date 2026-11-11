//! What:
//!  Running real Git for transaction,
//!  recovery and landing work:
//!  captured output,
//!  an
//!       optional private index and object store,
//!  extra or removed variables,
//!  and bytes on
//!       standard input.
//! Why:
//!  Every transaction step asks Git a question or writes an object;
//!  one runner keeps every
//!      such start an argument array (never a shell string) carrying the wrapper's child
//!      environment overlay (lock PID injection),
//!  as the incumbent's `runTransactionGit` does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { stdout } = await runTransactionGit({ gitPath, cwd, args, indexPath, input });
//! ```

/// Building a Git child from an argument array and an overlay.
use crate::forwarding::git_command;
/// Who a real-Git start runs for:
/// the wrapper's own work,
/// not the person's command.
use crate::hooks_off::StartOwner;
use crate::repo_read_hook::try_request;
/// The watched run that announces a step that takes long and stops it at the bound.
use crate::child_class::class_of;
use crate::child_watch::{StepBound, WatchTerms, output_watched_within_for_a_stop};
/// The request,
/// reply and failure types and the text of a command line,
/// re-exported so `crate::transaction_git::GitRequest`
///  and the other paths that name them keep working.
pub use crate::transaction_git_request_and_reply_types::{GitContext, GitFailure, GitOutput, GitRequest, command_text};
/// The rules for which steps may be run again or ended,
/// re-exported under the path they had in this file.
pub(crate) use crate::transaction_git_step_rerun_and_end_rules::{NO_REFLOG_SETTING, subcommand_of};
use crate::transaction_git_step_rerun_and_end_rules::{is_branch_write, may_be_rerun};
/// The bound of a step,
/// and the test-only shortened bounds of the current thread.
use crate::transaction_git_step_bounds::step_bound;
#[cfg(test)]
pub(crate) use crate::transaction_git_step_bounds::{EXTRA_WAIT_FOR_THIS_THREAD, STEP_CEILING_FOR_THIS_THREAD};
/// Private exclusive files for standard input.
use crate::private_storage::write_exclusive_scratch;
/// The repository scratch directory the input file is made in.
use crate::scratch_location::prepare_repository_scratch;
/// Starts that ride out a short shortage of processes.
use crate::start_retry::{RefusableStart, START_RETRY_BUDGET, retry_refused_start_with};
/// Unique temporary names.
use crate::random_id::random_uuid;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Command` builds a child process;
///  `Stdio` configures its streams.
use std::io::Write;
use std::process::{Command, Stdio};
/// A span of time,
/// for the bounded rerun.
use std::time::Duration;

/// What:
///  An already unlinked private file holding `bytes`,
///  opened for reading.
///  The file is made in the repository scratch directory of `common_dir`,
///  never in the system temporary directory.
/// Why:
///   Git may write output before it reads all its input;
///  feeding input from a file instead
///       of a pipe can never deadlock and needs no writer thread.
///  The name is removed at once,
///       so nothing is left behind even after a crash,
///  and a crash inside that moment leaves a file that the next preparation of the directory
///       sweeps once it is a day old.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // child.stdin.end(input);
/// ```
pub(crate) fn input_file(bytes: &[u8], common_dir: Option<&Path>) -> std::io::Result<std::fs::File> {
    let Some(common) = common_dir else {
        return Err(std::io::Error::other(
            "monochromatic-git-policy-cli has no repository scratch directory for the input of this Git command: the command was started \
             before the repository was known, which is a defect in monochromatic-git-policy-cli",
        ));
    };
    let directory: PathBuf = prepare_repository_scratch(common)?;
    let unique: String = match random_uuid() {
        Ok(id) => id,
        Err(error) => return Err(std::io::Error::other(error.0)),
    };
    let path: PathBuf = directory.join(format!("cli-git-input-{unique}"));
    write_exclusive_scratch(path.as_path(), bytes)?;
    let file: std::io::Result<std::fs::File> = std::fs::File::open(&path);
    // The open handle keeps the bytes readable after the name is gone. If the platform refuses
    // to remove an open file, the private file stays behind and the error is ignored.
    let _ = std::fs::remove_file(&path);
    let opened: std::fs::File = file?;
    return Ok(opened);
}

/// What:
///  Build the child command for one request.
/// Why:
///   The prefix,
///  the overlay,
///  the removals and the additions apply in a fixed order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawn(gitPath, args, { cwd, env: { ...withoutUnset(process.env), ...environment, GIT_INDEX_FILE, GIT_OBJECT_DIRECTORY } })
/// ```
pub(crate) fn build_command(context: &GitContext, request: &GitRequest) -> Command {
    // `full_arguments` decides which of the caller's global options reach this start, so a request whose
    // pathspecs are the wrapper's own runs without the caller's pathspec modes.
    let arguments: Vec<OsString> = request.full_arguments(context.global_prefix.as_slice());
    let mut command: Command = git_command(
        context.real_git.as_path(),
        arguments.as_slice(),
        context.overlay.as_slice(),
        StartOwner::Wrapper,
    );
    command.current_dir(request.cwd.as_path());
    for name in &request.unset {
        command.env_remove(name);
    }
    for (name, value) in &request.environment {
        command.env(name, value);
    }
    if let Some(index) = &request.index {
        command.env("GIT_INDEX_FILE", index);
    }
    if let Some(objects) = &request.object_directory {
        command.env("GIT_OBJECT_DIRECTORY", objects);
    }
    return command;
}

/// What:
///  Run one request and capture its output,
///  whatever its exit code.
///       `std::io::Result<GitOutput>` is "the outcome,
///  or why Git could not start".
/// Why:
///   Callers that treat a non-zero exit as an answer (a conflict,
///  a missing ref) read the
///       code themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ ..., allowFailure: true })
/// ```
pub fn run_git(context: &GitContext, request: &GitRequest) -> std::io::Result<GitOutput> {
    // An exact read-only command line the in-process readers know costs no Git process.
    if let Some(answered) = try_request(context, request) {
        return Ok(answered);
    }
    let label: String = crate::add_timing::git_step_label(request.arguments.as_slice());
    // A step that is starting takes the note of an earlier failure with it: only the step running last can be the reason a
    //  command stops (`git_step_failure.rs`).
    crate::git_step_failure::clear_failed_step();
    let entered: bool = crate::add_timing::begin_step(label.as_str());
    let output: std::io::Result<GitOutput> = run_git_pausing(context, request, START_RETRY_BUDGET, &mut std::thread::sleep);
    crate::add_timing::end_step(entered);
    return output;
}

/// What:
///  Whether Git failed because the system would not give it a process or a thread.
/// Why:
///   Git words these failures with the system's own text (`unable to create thread: Resource temporarily
///  unavailable`,
/// `cannot fork() for ...`),
/// and no other failure of a rerunnable command carries it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// output.exitCode !== 0 && output.stderr.includes('Resource temporarily unavailable')
/// ```
pub(crate) fn refused_by_the_system(output: &GitOutput) -> bool {
    return output.code != Some(0)
        && String::from_utf8_lossy(output.stderr.as_slice()).contains("Resource temporarily unavailable");
}

/// What:
///  Run one request,
/// running it again within the budget when Git reports a refused process or thread
///  and the command is harmless to run twice.
///  After the budget the last output is returned as it was,
/// so the caller reports Git's own words.
/// Why:
///   The command is rebuilt for each run,
/// so its input file is fresh.
///  A start that already waited out its own budget is not waited for again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retryWhile(() => runGitOnce(request), (output) => refusedBySystem(output) && mayBeRerun(request));
/// ```
fn run_git_pausing(
    context: &GitContext,
    request: &GitRequest,
    budget: Duration,
    pause: &mut dyn FnMut(Duration),
) -> std::io::Result<GitOutput> {
    if !may_be_rerun(request) {
        return run_git_once(context, request);
    }
    let what: String = command_text(&request.arguments);
    let mut attempt: RefusableGitRun<'_> = RefusableGitRun { context, request, refused: None };
    let result: std::io::Result<GitOutput> = retry_refused_start_with(what.as_str(), budget, pause, &mut attempt);
    match (result, attempt.refused.take()) {
        (Ok(finished), _) => return Ok(finished),
        (Err(error), Some(last)) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(last),
        (Err(error), _) => return Err(error),
    }
}

/// What:
///  One run of a request that the retry may start again:
/// the run,
/// and the output Git gave when the system refused it a process.
/// Why:
///   The retry takes its start as a value;
/// a refusal is told to the retry as `WouldBlock` and the output that carried it is kept
///  here,
/// so that after the budget the caller can return Git's own last words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RefusableGitRun implements RefusableStart<GitOutput> { refused?: GitOutput; start() {} }
/// ```
struct RefusableGitRun<'a> {
    /// The Git program and the directories the run uses.
    context: &'a GitContext,
    /// What to run.
    request: &'a GitRequest,
    /// The last output Git gave when the system refused it a process.
    refused: Option<GitOutput>,
}

impl RefusableStart<GitOutput> for RefusableGitRun<'_> {
    fn start(&mut self) -> std::io::Result<GitOutput> {
        let output: GitOutput = match run_git_once(self.context, self.request) {
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
///  Where `run_git_once_with` asks for the watch terms of a start,
/// once for each start.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The production entry reads the terms of the request's class;
/// tests choose short ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface WatchTermsSource { terms(): WatchTerms }
/// ```
pub(crate) trait WatchTermsSource {
    /// The terms for the next start.
    fn terms(&self) -> WatchTerms;
}

/// The watch terms of the class a request belongs to,
/// asked again at each start.
pub(crate) struct TermsOfRequest<'a> {
    /// The request whose subcommand names the class.
    pub(crate) request: &'a GitRequest,
}

impl WatchTermsSource for TermsOfRequest<'_> {
    fn terms(&self) -> WatchTerms {
        return terms_for_request(self.request);
    }
}

/// What:
///  Run one request once and capture its output,
/// whatever its exit code.
/// Why:
///   The command and its input file are built here,
/// so every run starts from the request alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGitOnce({ gitPath, cwd, args })
/// ```
fn run_git_once(context: &GitContext, request: &GitRequest) -> std::io::Result<GitOutput> {
    return run_git_once_with(context, request, &TermsOfRequest { request });
}

/// What:
///  The watch terms of a request:
/// those of the class its subcommand belongs to.
/// Why:
///   A query step and a bulk step are stopped at different bounds,
/// so the choice is one function a test can pin.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// termsForClass(classOf(request.args))
/// ```
pub(crate) fn terms_for_request(request: &GitRequest) -> WatchTerms {
    return WatchTerms::for_class(class_of(request.arguments.as_slice()));
}

/// What:
///  `run_git_once` with the notice cadence,
/// the quiet window and the notice sink of its wait made by `terms`,
/// which is asked
///  for again at each start.
///  A step that shows no sign of work for the window is ended and run once more;
/// a step that may not be run twice is only
///  waited for.
/// Why:
///   The production entry reads the terms from the environment;
///  tests choose short ones and read the notices from a buffer.
///  The owner's rule:
/// "End Git,
/// hold hooks (Recommended)":
/// a Git child with no progress is ended and its step redone,
/// and a
///  person's hook is never ended (none runs below a step the wrapper starts,
/// `hooks_off.rs`).
/// Redoing is limited to the steps
///  that are safe to run twice (`may_be_rerun`);
/// a step that is quiet again in its redo is reported with the sentence of
///  `watched_timeout_text`.
/// That the redo happens once is the coordinator's reading of "redone" (audit,
/// pick 4),
/// open to veto.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await runOnce(terms()); } catch (quiet) { if (!mayBeRerun(request)) throw quiet; return await runOnce(terms()); }
/// ```
fn run_git_once_with(
    context: &GitContext,
    request: &GitRequest,
    terms: &dyn WatchTermsSource,
) -> std::io::Result<GitOutput> {
    let first: std::io::Result<GitOutput> = run_git_watched(context, request, terms.terms());
    return match first {
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut && may_be_rerun(request) && !crate::child_watch::is_ceiling(&error) => {
            // Terminal output error: a notice is advisory; a failed write must not stop the commit.
            let _written = writeln!(
                std::io::stderr(),
                "monochromatic-git-policy-cli: {} showed no sign of work and was ended; running it again",
                step_subject(request)
            );
            run_git_watched(context, request, terms.terms())
        }
        other => other,
    };
}

/// How a request names itself in notices and in the sentence of a step that was ended:
/// the command and the kind of step.
pub(crate) fn step_subject(request: &GitRequest) -> String {
    return format!(
        "{}, {}",
        command_text(&request.arguments),
        class_of(request.arguments.as_slice()).label()
    );
}

/// What:
///  Start one request once under a watch and capture its output,
/// whatever its exit code.
/// Why:
///   The command and its input file are built here,
/// so every run starts from the request alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runWatched({ gitPath, cwd, args, terms, stoppable: mayBeRerun(request) })
/// ```
fn run_git_watched(context: &GitContext, request: &GitRequest, terms: WatchTerms) -> std::io::Result<GitOutput> {
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
    let watched = output_watched_within_for_a_stop(&mut command, subject.as_str(), ledger_step.as_str(), terms, may_be_rerun(request), bound, a_stop_lets_it_finish);
    crate::transaction_step_durations::note_step(ledger_step.as_str(), began, began.elapsed());
    let output: std::process::Output = watched?;
    return Ok(GitOutput {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    });
}

/// What:
///  Run one request and require success;
///  a start failure or a non-zero exit is a
///       failure naming the command and quoting Git's standard error.
/// Why:
///   Most transaction steps cannot continue without Git's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, args }) // throws CommitTransactionGitError
/// ```
pub fn run_git_checked(
    context: &GitContext,
    request: &GitRequest,
) -> Result<GitOutput, GitFailure> {
    return checked_from(run_git(context, request), request.arguments.as_slice());
}

/// What:
///  Turn the outcome of a run into the checked result:
///  success needs a start and a zero exit,
/// and each failure names the command.
/// Why:
///   A step stopped at its bound did start,
/// and its own text already says which command and why;
///  every other start failure and every non-zero exit is named here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkedFrom(outcome: GitOutput | Error, args: string[]): GitOutput; // throws CommitTransactionGitError
/// ```
pub(crate) fn checked_from(outcome: std::io::Result<GitOutput>, arguments: &[OsString]) -> Result<GitOutput, GitFailure> {
    let step: String = crate::add_timing::git_step_label(arguments);
    let output: GitOutput = match outcome {
        Ok(finished) => finished,
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {
            crate::git_step_failure::note_failed_step(step.as_str(), None);
            return Err(GitFailure(error.to_string()));
        }
        Err(error) => {
            crate::git_step_failure::note_failed_step(step.as_str(), None);
            return Err(GitFailure(format!("{} could not start: {error}", command_text(arguments))));
        }
    };
    if !output.succeeded() {
        crate::git_step_failure::note_failed_step(step.as_str(), output.code);
        return Err(GitFailure(format!("{} failed: {}", command_text(arguments), output.error_text())));
    }
    return Ok(output);
}

/// Real-Git runner controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_git_tests.rs"]
mod tests;
