//! What:
//!  A Git step of a transaction at the moment before it starts:
//!  its child command,
//! the words that name it,
//! the terms of its watch,
//! and the records that live as long as it runs,
//!  with what both faces do once the child has ended.
//! Why:
//!   The synchronous face (`transaction_git.rs`) and the awaited face (`transaction_git_awaiting.rs`) start the same
//!  step and record the same thing about it;
//! only the call that starts it,
//!  which waits for the pipes here and on the runtime there,
//! differs.
//!  Written once, the command,
//! the input file,
//! the subject,
//! the ledger name,
//! the bound,
//!  the stopping rule and the timing mark cannot drift between the faces,
//!  which is what the debt ledger of 2026-11-25 measured them doing.
//! The child command, the input file it may read and the sentence a failed step is named with are made here too,
//! and `transaction_git.rs` re-exports them under the paths they had there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const step = stepAboutToStart({ context, request, terms }); // { command, subject, ledgerStep, bound, ... }
//! ```

/// The kind of step a request's subcommand names,
/// for the sentence that names the step.
use crate::child_class::class_of;
/// The bound of one step,
/// and the terms of the watch that holds the step to it.
use crate::child_watch::{
    StepBound,
    WatchTerms,
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
/// The repository scratch directory the input file is made in.
use crate::scratch_location::prepare_repository_scratch;
/// The record a step that is never ended keeps for the clients that queue behind it,
/// and the function that takes it.
use crate::service_busy_step_record_on_this_thread::{
    StepGuard,
    enter_step,
};
/// The Git child's request and answer,
/// and the text of a command line.
use crate::transaction_git_request_and_reply_types::{
    GitContext,
    GitOutput,
    GitRequest,
    command_text,
};
/// How long the step is waited for and what happens at the bound.
use crate::transaction_git_step_bounds::step_bound;
/// Whether the step may be re-run and whether a stop lets it finish.
use crate::transaction_git_step_rerun_and_end_rules::{
    is_branch_write,
    may_be_rerun,
};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// The child's streams,
/// and the write of a redo's notice to standard error.
use std::io::Write;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Command,
    Stdio,
};
/// When the step began,
/// for the ledger of the steps a command ran.
use std::time::Instant;

/// What:
///  One Git step ready to start,
///  holding everything both faces need before the start and after it.
/// Why:
///   The command and its records must be made before the child starts,
/// and the timing mark must be written after it ended;
///  keeping them in one value is what lets each face hold only the start itself.
///  The record of an unbounded step is this value's own field on purpose:
///  it ends when the step's last use of the value does,
///  which is after the child has ended and the step is written down.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StepAboutToStart = { command: Command; subject: string; ledgerStep: string; terms: WatchTerms; bound: StepBound; stoppable: boolean; aStopLetsItFinish: boolean; began: number };
/// ```
pub(crate) struct StepAboutToStart {
    /// The child command,
    /// with its input file and its standard input already set.
    pub(crate) command: Command,
    /// What a notice calls the step:
    /// the command line and the kind of step.
    pub(crate) subject: String,
    /// The name the step is written down under once it ended.
    pub(crate) ledger_step: String,
    /// How quiet the step may be and how long it is waited for.
    pub(crate) terms: WatchTerms,
    /// What ends the step.
    pub(crate) bound: StepBound,
    /// Whether the step may be run again when it shows no sign of work.
    pub(crate) stoppable: bool,
    /// Whether a stop of the service lets this step finish.
    pub(crate) a_stop_lets_it_finish: bool,
    /// When the step was asked for,
    /// which the ledger measures it from.
    pub(crate) began: Instant,
    /// The record of a step that is never ended,
    /// kept until this value goes.
    _record: Option<StepGuard>,
}

/// What:
///  Make one Git step from its request:
/// the command,
/// its input,
/// the words that name it,
/// its watch terms and its bound.
/// Why:
///   Everything here is a decision about the step rather than about waiting for it,
/// so both faces make it once,
///  here,
/// and neither can start a differently named or differently bounded step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stepAboutToStart({ context, request, terms }): StepAboutToStart; // throws when the input file cannot be made
/// ```
pub(crate) fn step_about_to_start(
    context: &GitContext,
    request: &GitRequest,
    terms: WatchTerms,
) -> std::io::Result<StepAboutToStart> {
    // What:     `build_command(...)` makes the child command Git is started from, without starting it.
    // Why:      The input file and the streams below are set on it before anything runs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const command = buildCommand(context, request);
    // ```
    let mut command: Command = build_command(context, request);
    // What:     `match &request.input` reads the optional bytes a step is fed on standard input.
    // Why:      A step Git must read a request from gets a file; every other step gets the null device,
    //           so a step that waits on a terminal can never be reached by accident.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (request.input !== undefined) command.stdin(Stdio.from(inputFile(request.input, context.commonDir))); else command.stdin(Stdio.null());
    // ```
    match &request.input {
        Some(bytes) => {
            // What:     `Stdio::from(file)` hands the file the child reads its request from as its standard input.
            // Why:      Git reads a large request from a file rather than an argument list, and the file is this step's own.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // command.stdin(Stdio.from(inputFile(bytes, context.commonDir)));
            // ```
            command.stdin(Stdio::from(input_file(
                bytes.as_slice(),
                context.common_dir.as_deref(),
            )?));
        }
        None => {
            // What:     `Stdio::null()` is the null device in place of a terminal.
            // Why:      A step must never wait for input a person is not being asked for.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // command.stdin(Stdio.null());
            // ```
            command.stdin(Stdio::null());
        }
    }
    // What:     `step_subject(request)` is the command line followed by the kind of step, as every notice words it.
    // Why:      The subject travels into the watched run, so a notice and the timeout sentence name the same step.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const subject = stepSubject(request);
    // ```
    let subject: String = step_subject(request);
    // What:     `git_step_label(...)` is the short name the timing ledger records the step under.
    // Why:      The ledger is written from here, so its name is decided before the step starts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ledgerStep = gitStepLabel(request.arguments);
    // ```
    let ledger_step: String = crate::add_timing::git_step_label(request.arguments.as_slice());
    // What:     `step_bound(request)` is what ends the step: a ceiling, a stop, or nothing.
    // Why:      Both faces pass the same bound, so a step cannot be waited for longer on one of them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bound = stepBound(request);
    // ```
    let bound: StepBound = step_bound(request);
    // What:     `enter_step(...)` records a step that is never ended for the clients that queue behind it,
    //           and answers the guard that removes the record when it goes; `None` records nothing.
    // Why:      A write that is never ended is what makes a client wait, so only such a step is announced,
    //           and the record ends with the step.
    // Gotcha:   The guard is held in the value this function answers, so the record lives as long as the step does;
    //           dropping the value early would remove the announcement while the child still runs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const record = bound.kind === 'unbounded' ? enterStep(`${commandText(request.arguments)} in ${request.cwd}`) : undefined;
    // ```
    let record: Option<StepGuard> = if matches!(bound, StepBound::Unbounded) {
        enter_step(format!(
            "{} in {}",
            command_text(&request.arguments),
            request.cwd.display()
        ))
    } else {
        None
    };
    // What:     `Instant::now()` marks the moment the step is asked for, and the ledger measures the step from it.
    // Why:      A step that failed or was ended spent its time too, so the mark is taken whatever its outcome.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const began = Date.now();
    // ```
    let began: Instant = Instant::now();
    // What:     `is_branch_write(request)` says whether this step writes a branch,
    //           which is the one class a stop lets finish (owner, 2026-11-08, "B: let a branch write finish (Recommended)").
    // Why:      A cut branch write is a state recovery would have to read, and the owner chose to let it end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const aStopLetsItFinish = bound.kind === 'unbounded' && isBranchWrite(request);
    // ```
    let a_stop_lets_it_finish: bool =
        matches!(bound, StepBound::Unbounded) && is_branch_write(request);
    // What:     The value those decisions make, with the record's guard kept inside it.
    // Why:      One value carries every face needs, so a face cannot forget one of them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { command, subject, ledgerStep, terms, bound, stoppable: mayBeRerun(request), aStopLetsItFinish, began };
    // ```
    return Ok(StepAboutToStart {
        command,
        subject,
        ledger_step,
        terms,
        bound,
        stoppable: may_be_rerun(request),
        a_stop_lets_it_finish,
        began,
        _record: record,
    });
}

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
///  What the wrapper answers once a step has ended:
/// the output Git gave,
/// with the step written down in the ledger.
/// Why:
///   The ledger entry and the shape of the answer are the same on both faces,
/// so a step that ended is written down the same way whichever face waited for it.
///  The record is written whatever the outcome,
/// which is why the caller passes the ended child's output and not a result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stepFinished(ledgerStep, request, began, output): GitOutput { noteStepAsked(ledgerStep, request.arguments, began, now() - began); return { code: output.status.code, stdout: output.stdout, stderr: output.stderr }; }
/// ```
pub(crate) fn step_finished(
    ledger_step: &str,
    request: &GitRequest,
    began: Instant,
    output: std::process::Output,
) -> GitOutput {
    // What:     `note_step_asked(...)` writes this step into the ledger with the moment it began and how long it took.
    // Why:      The timing of every step is what a slow command is diagnosed from, and both faces record it here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // noteStepAsked(ledgerStep, request.arguments, began, now() - began);
    // ```
    crate::transaction_step_durations::note_step_asked(
        ledger_step,
        request.arguments.as_slice(),
        began,
        began.elapsed(),
    );
    // What:     `GitOutput { .. }` is the answer a step's caller reads: the exit status as an `Option<i32>`
    //           (`None` when a signal ended the child) and the two captured streams.
    // Why:      The transaction decides what a step means from these three pieces alone, so nothing of the child is kept.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { code: output.status.code, stdout: output.stdout, stderr: output.stderr };
    // ```
    return GitOutput {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    };
}

/// What:
///  One start's failure as the retry reads it:
///  a system that refused a process is told to the retry as "try again",
///  and every other failure is handed on as it came.
/// Why:
///   Both faces start a step through their own query,
/// and the test for a refusal is the same on both,
///  so the mapping is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startFailure = (error) => error.kind === 'WouldBlock' ? new Error(String(error), { retryable: true }) : error;
/// ```
pub(crate) fn start_failure(error: std::io::Error) -> std::io::Error {
    // What:     `match error.kind()` reads the classification a `std::io::Error` carries, not its message.
    // Why:      The retry decides from the kind alone, and the message staying the same is what lets Git's words travel.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (error.kind === 'WouldBlock') return ioErrorOther(error);
    // return error;
    // ```
    return match error.kind() {
        // What:     `std::io::Error::other(error)` wraps the refusal in an error the retry does not read as a refusal.
        // Why:      A refused start of the child's own `spawn` is the retry's business; its refusal is not.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return new Error(String(error));
        // ```
        std::io::ErrorKind::WouldBlock => std::io::Error::other(error),
        // What:     Every other failure is handed on unchanged.
        // Why:      A missing Git or a broken pipe is the caller's answer, not a reason to try again.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return error;
        // ```
        _ => error,
    };
}

/// What:
///  What one start answers the retry:
///  the output Git gave,
///  or a refusal the retry waits out,
///  with the output that carried it kept for the caller.
/// Why:
///   A run that the system refused is not a result but a pause,
/// and Git's own last words must survive the budget;
///  both faces keep them the same way,
///  so the rule is written once here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answerOfAStart = (refused, finished) => refusedByTheSystem(finished) ? (refused = finished, retryable()) : finished;
/// ```
pub(crate) fn answer_of_a_start(
    refused: &mut Option<GitOutput>,
    finished: GitOutput,
) -> std::io::Result<GitOutput> {
    // What:     `refused_by_the_system(&finished)` asks whether Git's own text says the system refused it a process.
    // Why:      Only that failure is worth waiting out; every other one is the step's answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (refusedByTheSystem(finished)) { refused.value = finished; return retryable(); }
    // ```
    if refused_by_the_system(&finished) {
        // What:     `*refused = Some(finished)` keeps this run's output where the caller reads it after the budget.
        // Why:      Git's own words are the report of a refusal that outlasted the budget.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // refused.value = finished;
        // ```
        *refused = Some(finished);
        // What:     `Err(std::io::Error::from(WouldBlock))` tells the retry that this start may be tried again.
        // Why:      The retry pauses and asks again; the output above is what the caller ends with.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return retryable();
        // ```
        return Err(std::io::Error::from(std::io::ErrorKind::WouldBlock));
    }
    // What:     `Ok(finished)` hands the run's own output on.
    // Why:      Git answered, so the retry is done and the caller reads this output.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return finished;
    // ```
    return Ok(finished);
}

/// What:
///  Says that a step showed no sign of work,
///  was ended,
///  and is being run again.
/// Why:
///   Both faces end a quiet step and run it once more,
/// and the person is owed the same sentence whichever face waited;
///  it goes to standard error by hand rather than through the diagnostic queue,
///  and a failed write there must not stop the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.stderr.write(`monochromatic-git-policy-cli: ${stepSubject(request)} showed no sign of work and was ended; running it again\n`);
/// ```
pub(crate) fn step_is_run_again(request: &GitRequest) {
    // Terminal output error: a notice is advisory; a failed write must not stop the commit.
    let _written = writeln!(
        std::io::stderr(),
        "monochromatic-git-policy-cli: {} showed no sign of work and was ended; running it again",
        step_subject(request)
    );
}
