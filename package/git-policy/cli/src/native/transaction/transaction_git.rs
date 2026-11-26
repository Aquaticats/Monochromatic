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

/// The watched run that announces a step that takes long and stops it at the bound.
use crate::child_class::class_of;
/// The bound of a step,
/// which the controls of the steps' bounds still ask.
#[cfg(test)]
use crate::child_watch::StepBound;
use crate::child_watch::{
    WatchTerms,
    output_watched_within_for_a_stop,
};
/// Building a Git child from an argument array and an overlay.
use crate::forwarding::git_command;
/// Who a real-Git start runs for:
/// the wrapper's own work,
/// not the person's command.
use crate::hooks_off::StartOwner;
/// Private exclusive files for standard input.
use crate::private_storage::write_exclusive_scratch;
/// Unique temporary names.
use crate::random_id::random_uuid;
use crate::repo_read_hook::try_request;
/// The repository scratch directory the input file is made in.
use crate::scratch_location::prepare_repository_scratch;
/// Starts that ride out a short shortage of processes.
use crate::start_retry::{
    RefusableStart,
    START_RETRY_BUDGET,
    retry_refused_start_with,
};
/// The request,
/// reply and failure types and the text of a command line,
/// re-exported so `crate::transaction_git::GitRequest`
///  and the other paths that name them keep working.
pub use crate::transaction_git_request_and_reply_types::{
    GitContext,
    GitFailure,
    GitOutput,
    GitRequest,
    command_text,
};
/// The step both faces start,
/// what it answers once it ended,
/// and how one start's answer is read.
use crate::transaction_git_step_about_to_start::{
    StepAboutToStart,
    answer_of_a_start,
    start_failure,
    step_about_to_start,
    step_finished,
    step_is_run_again,
};
/// The bound a step is given,
/// which the step's own preparation reads and the controls of the bounds still ask.
#[cfg(test)]
use crate::transaction_git_step_bounds::step_bound;
#[cfg(test)]
pub(crate) use crate::transaction_git_step_bounds::{
    EXTRA_WAIT_FOR_THIS_THREAD,
    STEP_CEILING_FOR_THIS_THREAD,
};
use crate::transaction_git_step_rerun_and_end_rules::may_be_rerun;
/// The rules for which steps may be run again or ended,
/// re-exported under the path they had in this file.
pub(crate) use crate::transaction_git_step_rerun_and_end_rules::{
    NO_REFLOG_SETTING,
    subcommand_of,
};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// `Command` builds a child process;
///  `Stdio` configures its streams.
/// `Command` builds a child process.
use std::process::Command;
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
pub(crate) fn input_file(
    bytes: &[u8],
    common_dir: Option<&Path>,
) -> std::io::Result<std::fs::File> {
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
pub(crate) fn build_command(
    context: &GitContext,
    request: &GitRequest,
) -> Command {
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
    if request.end_with_the_starting_thread {
        end_with_the_starting_thread(&mut command);
    }
    return command;
}

/// What:
///  Ask the kernel to end this child when the thread that starts it ends,
/// which is the thread that ends with this process.
/// Why:
///  A Git child an add leaves unjoined has nobody waiting for it;
///  without this it would outlive the command until it next writes to a pipe no reader holds,
///  and a read that never writes again would outlive it for the whole step ceiling.
///  `PR_SET_PDEATHSIG` is set in the child between `fork` and `exec`,
///  so it is there before Git runs and is not inherited by Git's own children.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// command.beforeExec(() => prctl(PR_SET_PDEATHSIG, SIGKILL));
/// ```
#[cfg(unix)]
fn end_with_the_starting_thread(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    // SAFETY: the closure runs in the child between `fork` and `exec`, where only async-signal-safe
    // calls are allowed; `prctl` is one, and it reads no memory the child shares with this process.
    let _ = unsafe {
        command.pre_exec(|| {
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            return Ok(());
        })
    };
}

/// What:
///  The parent-death signal has no counterpart here,
/// so nothing is asked for.
/// Why:
///  The field is set only by the ladder's early size read,
///  and this build never runs that read where the signal is unavailable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no equivalent outside Unix
/// ```
#[cfg(not(unix))]
fn end_with_the_starting_thread(_command: &mut Command) {}

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
pub fn run_git(
    context: &GitContext,
    request: &GitRequest,
) -> std::io::Result<GitOutput> {
    // An exact read-only command line the in-process readers know costs no Git process.
    if let Some(answered) = try_request(context, request) {
        return Ok(answered);
    }
    let label: String = crate::add_timing::git_step_label(request.arguments.as_slice());
    // A step that is starting takes the note of an earlier failure with it: only the step running last can be the reason a
    //  command stops (`git_step_failure.rs`). A run beside another step of the command is not that step, so it leaves the
    //  note where it found it (`GitRequest::beside_another_step`).
    if !request.beside_another_step {
        crate::git_step_failure::clear_failed_step();
    }
    let entered: bool = crate::add_timing::begin_step(label.as_str());
    let output: std::io::Result<GitOutput> = run_git_pausing(
        context,
        request,
        START_RETRY_BUDGET,
        &mut std::thread::sleep,
    );
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
        && String::from_utf8_lossy(output.stderr.as_slice())
            .contains("Resource temporarily unavailable");
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
    let mut attempt: RefusableGitRun<'_> = RefusableGitRun {
        context,
        request,
        refused: None,
    };
    let result: std::io::Result<GitOutput> =
        retry_refused_start_with(what.as_str(), budget, pause, &mut attempt);
    match (result, attempt.refused.take()) {
        (Ok(finished), _) => return Ok(finished),
        (Err(error), Some(last)) if error.kind() == std::io::ErrorKind::WouldBlock => {
            return Ok(last);
        }
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
///  Both faces start this one value:
/// the synchronous face below,
/// and the awaited face (`transaction_git_awaiting.rs`),
///  so neither can keep a different part of a refused run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RefusableGitRun implements RefusableStart<GitOutput> { refused?: GitOutput; start() {} }
/// ```
pub(crate) struct RefusableGitRun<'a> {
    /// The Git program and the directories the run uses.
    pub(crate) context: &'a GitContext,
    /// What to run.
    pub(crate) request: &'a GitRequest,
    /// The last output Git gave when the system refused it a process.
    pub(crate) refused: Option<GitOutput>,
}

impl RefusableStart<GitOutput> for RefusableGitRun<'_> {
    fn start(&mut self) -> std::io::Result<GitOutput> {
        // What:     `match run_git_once(...)` opens the run's answer: the output Git gave, or the failure that ended it.
        // Why:      The two are read differently, and a failure is turned into the retry's own shape before it travels.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const finished = attempt(() => runGitOnce(context, request));
        // ```
        let finished: GitOutput = match run_git_once(self.context, self.request) {
            Ok(finished) => finished,
            // What:     `start_failure(error)` makes a refusal the retry waits out, and hands any other failure on.
            // Why:      The awaited face reads this start the same way, and the rule is `start_failure`'s.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // Err: return fail(startFailure(error));
            // ```
            Err(error) => return Err(start_failure(error)),
        };
        // What:     `answer_of_a_start(...)` keeps a refused run's output and tells the retry to ask again.
        // Why:      The rule of one start's answer is shared with the awaited face.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return answerOfAStart(this.refused, finished);
        // ```
        return answer_of_a_start(&mut self.refused, finished);
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
fn run_git_once(
    context: &GitContext,
    request: &GitRequest,
) -> std::io::Result<GitOutput> {
    return run_git_once_with(
        context,
        request,
        &TermsOfRequest {
            request,
        },
    );
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
        Err(error)
            if error.kind() == std::io::ErrorKind::TimedOut
                && may_be_rerun(request)
                && !crate::child_watch::is_ceiling(&error) =>
        {
            step_is_run_again(request);
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
fn run_git_watched(
    context: &GitContext,
    request: &GitRequest,
    terms: WatchTerms,
) -> std::io::Result<GitOutput> {
    // What:     `step_about_to_start(...)` makes everything the step needs before it starts: the child command, its input,
    //           the words that name it, its watch terms, its bound and its record.
    // Why:      The awaited face makes the same value, so a step cannot be started with a different bound or a different name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const step = stepAboutToStart({ context, request, terms });
    // ```
    let mut step: StepAboutToStart = step_about_to_start(context, request, terms)?;
    // What:     `output_watched_within_for_a_stop(...)` starts the step on this thread and reads its output,
    //           ending it at its ceiling or when a stop asks;
    //           it answers the child's `Output`, or the failure that kept the child from starting.
    // Why:      This is the whole difference between the two faces: the awaited one waits for the same step on the runtime.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const watched = outputWatchedWithinForAStop(step.command, step.subject, step.ledgerStep, step.terms, step.stoppable, step.bound, step.aStopLetsItFinish, request.startedChild);
    // ```
    let watched: std::io::Result<std::process::Output> = output_watched_within_for_a_stop(
        &mut step.command,
        step.subject.as_str(),
        step.ledger_step.as_str(),
        step.terms,
        step.stoppable,
        step.bound,
        step.a_stop_lets_it_finish,
        request.started_child.as_ref(),
    );
    // What:     `step_finished(&step, request, watched?)` writes the step into the ledger and turns the child's output into
    //           the answer a caller reads; the `?` hands a start failure on and skips the ledger, as the synchronous face did.
    // Why:      The ledger entry and the answer are the awaited face's too, and they are written in one place.
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
pub(crate) fn checked_from(
    outcome: std::io::Result<GitOutput>,
    arguments: &[OsString],
) -> Result<GitOutput, GitFailure> {
    let step: String = crate::add_timing::git_step_label(arguments);
    let output: GitOutput = match outcome {
        Ok(finished) => finished,
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {
            crate::git_step_failure::note_failed_step(step.as_str(), None);
            return Err(GitFailure(error.to_string()));
        }
        Err(error) => {
            crate::git_step_failure::note_failed_step(step.as_str(), None);
            return Err(GitFailure(format!(
                "{} could not start: {error}",
                command_text(arguments)
            )));
        }
    };
    if !output.succeeded() {
        crate::git_step_failure::note_failed_step(step.as_str(), output.code);
        return Err(GitFailure(format!(
            "{} failed: {}",
            command_text(arguments),
            output.error_text()
        )));
    }
    return Ok(output);
}

/// Real-Git runner controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_git_tests.rs"]
mod tests;
