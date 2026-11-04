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
use crate::repo_read_hook::try_request;
/// The watched run that announces a step that takes long and stops it at the bound.
use crate::child_class::{ChildClass, class_of};
use crate::child_watch::{WatchTerms, output_watched_within};
use crate::wait_ceilings::{OBJECT_STEP_CEILING, PLUMBING_STEP_CEILING};
/// Private exclusive files for standard input.
use crate::private_storage::write_exclusive_scratch;
/// The repository scratch directory the input file is made in.
use crate::scratch_location::prepare_repository_scratch;
/// Starts that ride out a short shortage of processes.
use crate::start_retry::{START_RETRY_BUDGET, retry_refused_start_with};
/// Unique temporary names.
use crate::random_id::random_uuid;
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Command` builds a child process;
///  `Stdio` configures its streams.
use std::io::Write;
use std::process::{Command, Stdio};
/// A span of time, for the bounded rerun.
use std::time::Duration;

/// What:
///  The real Git executable and the variables every Git child receives.
/// Why:
///   Transaction code is handed this once and starts every Git through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitContext = { gitPath: string; overlay: [string, string][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitContext {
    /// Absolute real Git executable.
    pub real_git: PathBuf,
    /// Variables added to every child:
    ///  lock PID injection and the forward-target marker.
    pub overlay: Vec<(OsString, OsString)>,
    /// Global options of the invocation,
    ///  kept before every subcommand (`-C`,
    ///  `--git-dir`,
    ///  `-c`).
    pub global_prefix: Vec<OsString>,
    /// The repository's common Git directory,
    ///  where the input file of a request that carries input is made (`cli-git-scratch` inside it);
    ///  `None` where the repository is not yet known,
    ///  and a request with input then fails naming the defect.
    pub common_dir: Option<PathBuf>,
}

/// What:
///  One Git start:
///  where it runs,
///  its arguments after the global prefix,
///  and its
///       environment and input adjustments.
/// Why:
///   A builder keeps call sites short while every option stays explicit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitRequest = { cwd: string; args: string[]; indexPath?: string; objectDirectory?: string; environment?: Record<string, string>; unset?: string[]; input?: Uint8Array };
/// ```
#[derive(Clone, Debug, Default)]
pub struct GitRequest {
    /// Working directory.
    pub cwd: PathBuf,
    /// Arguments after the global prefix.
    pub arguments: Vec<OsString>,
    /// Whether the invocation's global prefix is left out (shadow and private-store commands).
    pub without_prefix: bool,
    /// Private index,
    ///  as `GIT_INDEX_FILE`.
    pub index: Option<PathBuf>,
    /// Object store receiving every written object,
    ///  as `GIT_OBJECT_DIRECTORY`.
    pub object_directory: Option<PathBuf>,
    /// Variables added after the overlay.
    pub environment: Vec<(OsString, OsString)>,
    /// Inherited variables removed before the additions apply.
    pub unset: Vec<OsString>,
    /// Bytes on standard input;
    ///  none means an empty input.
    pub input: Option<Vec<u8>>,
}

/// Builder methods for one request.
impl GitRequest {
    /// What:
    ///  A request running `arguments` in `cwd`.
    /// Why:
    ///   Most requests need nothing else.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { cwd, args }
    /// ```
    pub fn new<S: AsRef<OsStr>>(cwd: &Path, arguments: &[S]) -> Self {
        // `mut` allows collecting the arguments one at a time.
        let mut owned: Vec<OsString> = Vec::with_capacity(arguments.len());
        for argument in arguments {
            owned.push(argument.as_ref().to_os_string());
        }
        return Self {
            cwd: cwd.to_path_buf(),
            arguments: owned,
            ..Self::default()
        };
    }
}

/// What:
///  How one Git start ended and what it wrote.
/// Why:
///   Callers decide on the exit code and parse standard output as bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitOutput = { stdout: Uint8Array; stderr: string; exitCode: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitOutput {
    /// Exit code;
    ///  `None` when a signal ended Git.
    pub code: Option<i32>,
    /// Standard output bytes.
    pub stdout: Vec<u8>,
    /// Standard error bytes.
    pub stderr: Vec<u8>,
}

/// Methods over a finished Git start.
impl GitOutput {
    /// What:
    ///  Whether Git exited with status 0.
    /// Why:
    ///   Most callers only need success or failure.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// output.exitCode === 0
    /// ```
    pub fn succeeded(&self) -> bool {
        return self.code == Some(0);
    }

    /// What:
    ///  Standard error as trimmed,
    ///  lossily decoded text for diagnostics.
    /// Why:
    ///   Failure messages quote what Git said.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// output.stderr.trim()
    /// ```
    pub fn error_text(&self) -> String {
        return String::from_utf8_lossy(self.stderr.as_slice())
            .trim()
            .to_string();
    }
}

/// What:
///  Why a Git start failed.
/// Why:
///   A start that could not happen and a Git that exited unsuccessfully are reported with
///       the command named,
///  as `CommitTransactionGitError`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CommitTransactionGitError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitFailure(pub String);

/// What:
///  The failure message as display text.
/// Why:
///   Diagnostics print it unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.message
/// ```
impl std::fmt::Display for GitFailure {
    /// Writes the message.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.0.as_str());
    }
}

/// What:
///  The arguments of a request as display text,
///  `git <arguments>`.
/// Why:
///   Failure messages name the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `git ${args.join(' ')}`
/// ```
pub fn command_text(arguments: &[OsString]) -> String {
    let mut text: String = String::from("git");
    for argument in arguments {
        text.push(' ');
        text.push_str(argument.to_string_lossy().as_ref());
    }
    return text;
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
fn input_file(bytes: &[u8], common_dir: Option<&Path>) -> std::io::Result<std::fs::File> {
    let Some(common) = common_dir else {
        return Err(std::io::Error::other(
            "cli-git has no repository scratch directory for the input of this Git command: the command was started \
             before the repository was known, which is a defect in cli-git",
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
fn build_command(context: &GitContext, request: &GitRequest) -> Command {
    // `mut` allows assembling the argument list in order.
    let mut arguments: Vec<OsString> = Vec::new();
    if !request.without_prefix {
        arguments.extend(context.global_prefix.iter().cloned());
    }
    arguments.extend(request.arguments.iter().cloned());
    let mut command: Command = git_command(
        context.real_git.as_path(),
        arguments.as_slice(),
        context.overlay.as_slice(),
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
    return crate::add_timing::step(label.as_str(), || { return run_git_pausing(context, request, START_RETRY_BUDGET, &mut |wait: Duration| { std::thread::sleep(wait); }); });
}

/// Commands whose second run after a refusal is harmless: they read, or they write content-addressed objects,
///  or they set the same value again.
const RERUNNABLE: [&str; 17] = [
    "rev-parse",
    "symbolic-ref",
    "for-each-ref",
    "ls-files",
    "diff",
    "diff-index",
    "diff-tree",
    "cat-file",
    "show",
    "hash-object",
    "write-tree",
    "ls-remote",
    "merge-tree",
    "init",
    "pack-objects",
    "index-pack",
    "version",
];

/// What:
///  The subcommand a request runs and where it sits among the arguments,
///  past the global options (`-c <pair>`, `-C <directory>` and the `--option` forms).
/// Why:
///   The rerun allow-list is decided by the subcommand and, for a few, by its options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const index = args.findIndex(...); // first token that is not a global option or its value
/// ```
pub(crate) fn subcommand_of(arguments: &[OsString]) -> Option<(usize, String)> {
    let mut index: usize = 0;
    while index < arguments.len() {
        let text: String = arguments[index].to_string_lossy().into_owned();
        if text == "-c" || text == "-C" {
            index += 2;
            continue;
        }
        if text.starts_with('-') {
            index += 1;
            continue;
        }
        return Some((index, text));
    }
    return None;
}

/// What:
///  Whether running the request a second time after the system refused it is harmless.
/// Why:
///   Git reports a refused process or thread as a failure after it started,
///  so only commands that read, write content-addressed objects, or change a private index the same way twice
///  may be run again;
///  reference updates, commits and every other writer are not,
///  and recovery already completes or rolls those back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// RERUNNABLE.includes(subcommand) || (subcommand === 'config' && reads(args)) || ...
/// ```
fn may_be_rerun(request: &GitRequest) -> bool {
    let Some((position, name)) = subcommand_of(request.arguments.as_slice()) else {
        return false;
    };
    let rest: Vec<String> = request.arguments[position + 1..]
        .iter()
        .map(|argument| { return argument.to_string_lossy().into_owned(); })
        .collect();
    match name.as_str() {
        "config" => return rest.iter().any(|option| {
            return matches!(option.as_str(), "--get" | "--get-all" | "--get-regexp" | "--list" | "-l");
        }),
        "remote" => return rest.first().is_some_and(|word| { return word == "get-url"; }),
        "add" | "reset" | "read-tree" | "update-index" => return request.index.is_some(),
        other => return RERUNNABLE.contains(&other),
    }
}

/// What:
///  Whether Git failed because the system would not give it a process or a thread.
/// Why:
///   Git words these failures with the system's own text (`unable to create thread: Resource temporarily
///  unavailable`, `cannot fork() for ...`), and no other failure of a rerunnable command carries it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// output.exitCode !== 0 && output.stderr.includes('Resource temporarily unavailable')
/// ```
fn refused_by_the_system(output: &GitOutput) -> bool {
    return output.code != Some(0)
        && String::from_utf8_lossy(output.stderr.as_slice()).contains("Resource temporarily unavailable");
}

/// What:
///  Run one request, running it again within the budget when Git reports a refused process or thread
///  and the command is harmless to run twice.
///  After the budget the last output is returned as it was, so the caller reports Git's own words.
/// Why:
///   The command is rebuilt for each run, so its input file is fresh.
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
    let mut refused: Option<GitOutput> = None;
    let result: std::io::Result<GitOutput> = retry_refused_start_with(what.as_str(), budget, pause, &mut || {
        let output: GitOutput = match run_git_once(context, request) {
            Ok(finished) => finished,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Err(std::io::Error::other(error)),
            Err(error) => return Err(error),
        };
        if refused_by_the_system(&output) {
            refused = Some(output);
            return Err(std::io::Error::from(std::io::ErrorKind::WouldBlock));
        }
        return Ok(output);
    });
    match (result, refused.take()) {
        (Ok(finished), _) => return Ok(finished),
        (Err(error), Some(last)) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(last),
        (Err(error), _) => return Err(error),
    }
}

/// What:
///  Run one request once and capture its output, whatever its exit code.
/// Why:
///   The command and its input file are built here, so every run starts from the request alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGitOnce({ gitPath, cwd, args })
/// ```
fn run_git_once(context: &GitContext, request: &GitRequest) -> std::io::Result<GitOutput> {
    return run_git_once_with(context, request, &|| { return terms_for_request(request); });
}

/// What:
///  The watch terms of a request: those of the class its subcommand belongs to.
/// Why:
///   A query step and a bulk step are stopped at different bounds, so the choice is one function a test can pin.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// termsForClass(classOf(request.args))
/// ```
pub(crate) fn terms_for_request(request: &GitRequest) -> WatchTerms {
    return WatchTerms::for_class(class_of(request.arguments.as_slice()));
}

/// What:
///  `run_git_once` with the notice cadence, the quiet window and the notice sink of its wait made by `terms`, which is asked
///  for again at each start.
///  A step that shows no sign of work for the window is ended and run once more; a step that may not be run twice is only
///  waited for.
/// Why:
///   The production entry reads the terms from the environment;
///  tests choose short ones and read the notices from a buffer.
///  The owner's rule: "End Git, hold hooks (Recommended)": a Git child with no progress is ended and its step redone, and a
///  person's hook is never ended (none runs below a step the wrapper starts, `hooks_off.rs`). Redoing is limited to the steps
///  that are safe to run twice (`may_be_rerun`); a step that is quiet again in its redo is reported with the sentence of
///  `watched_timeout_text`. That the redo happens once is the coordinator's reading of "redone" (audit, pick 4), open to veto.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await runOnce(terms()); } catch (quiet) { if (!mayBeRerun(request)) throw quiet; return await runOnce(terms()); }
/// ```
fn run_git_once_with(
    context: &GitContext,
    request: &GitRequest,
    terms: &dyn Fn() -> WatchTerms,
) -> std::io::Result<GitOutput> {
    let first: std::io::Result<GitOutput> = run_git_watched(context, request, terms());
    return match first {
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut && may_be_rerun(request) && !crate::child_watch::is_ceiling(&error) => {
            // Terminal output error: a notice is advisory; a failed write must not stop the commit.
            let _written = writeln!(
                std::io::stderr(),
                "cli-git: {} showed no sign of work and was ended; running it again",
                step_subject(request)
            );
            run_git_watched(context, request, terms())
        }
        other => other,
    };
}

/// How a request names itself in notices and in the sentence of a step that was ended: the command and the kind of step.
fn step_subject(request: &GitRequest) -> String {
    return format!(
        "{}, {}",
        command_text(&request.arguments),
        class_of(request.arguments.as_slice()).label()
    );
}

/// The subcommands whose step changes the person's own repository state in place (a reference, the real index, the worktree or the
///  configuration), so that ending one in the middle of its write could cost data and running it a second time is not known to be
///  harmless. A request of one of these that `may_be_rerun` refuses has no ceiling.
///  Left out on purpose, and so still ended at the ceiling of their class: the readers that `RERUNNABLE` does not name (`rev-list`,
///  `merge-base`, `log`, `ls-tree`, `status`, `var`, `diff-files`, `check-attr`, `check-ignore`, `stripspace`, `fmt-merge-msg`),
///  which change nothing, and the writers of unreferenced objects (`commit-tree`, `fast-import`, `mktree`), whose second run
///  writes the same content-addressed bytes and whose partial result is a loose object or a temporary pack that no reference names.
///  Citation: the owner's answer of 2026-11-04, "Never cut a branch write (Recommended)", which reads "Steps that can safely run
///  twice keep their ceilings; the branch write is never ended" (`doc/planning/cli-git-no-unbounded-waits.md`, "Picks").
const WRITES_IN_PLACE: [&str; 13] = [
    "update-ref",
    "reset",
    "read-tree",
    "update-index",
    "add",
    "checkout-index",
    "clean",
    "stash",
    "merge",
    "checkout",
    "switch",
    "restore",
    "config",
];

/// What:
///  Whether the request's step writes the person's repository state in place and may not be run twice, which is the step the
///  wrapper never ends.
/// Why:
///   The branch write is the owner's named case; the others are the same kind of step by the same reasoning, listed in
///  `WRITES_IN_PLACE` with what was left out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// !mayBeRerun(request) && WRITES_IN_PLACE.includes(subcommandOf(request))
/// ```
pub(crate) fn is_write_in_place(request: &GitRequest) -> bool {
    if may_be_rerun(request) {
        return false;
    }
    return subcommand_of(request.arguments.as_slice()).is_some_and(|(_, name)| { return WRITES_IN_PLACE.contains(&name.as_str()); });
}

/// What:
///  The longest the request's step runs before it is ended, whatever it shows: the ceiling of its class, or none for a write in
///  place that is not safe to run twice.
/// Why:
///   The owner's rule is that nothing waits for ever (`wait_ceilings.rs`), but a write in the middle of its work is ended only
///  by the person (owner, 2026-11-04: "Never cut a branch write (Recommended)"): a landing that is stuck in one keeps the service
///  busy, the clients that wait behind it are told what it is stuck in and are withdrawn at their own ceiling
///  (`service_busy.rs`). In a unit test the ceiling of the steps that have one can be shortened for the thread that runs the
///  control; a step with none stays without.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stepCeiling(request): number | undefined { return isWriteInPlace(request) ? undefined : classOf(request).ceiling }
/// ```
fn step_ceiling(request: &GitRequest) -> Option<Duration> {
    if is_write_in_place(request) {
        return None;
    }
    #[cfg(test)]
    if let Some(shortened) = STEP_CEILING_FOR_THIS_THREAD.with(std::cell::Cell::get) {
        return Some(shortened);
    }
    return Some(match class_of(request.arguments.as_slice()) {
        ChildClass::Plumbing => PLUMBING_STEP_CEILING,
        ChildClass::Object => OBJECT_STEP_CEILING,
    });
}

#[cfg(test)]
thread_local! {
    /// A shortened ceiling for the steps the current thread starts, so a control ends a step in a moment.
    pub(crate) static STEP_CEILING_FOR_THIS_THREAD: std::cell::Cell<Option<Duration>> = const { std::cell::Cell::new(None) };
}

/// What:
///  Start one request once under a watch and capture its output, whatever its exit code.
/// Why:
///   The command and its input file are built here, so every run starts from the request alone.
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
    let ceiling: Option<Duration> = step_ceiling(request);
    // A write that is never ended is recorded for the clients that queue behind it (`service_busy.rs`); the record ends with the step.
    let _recorded = if ceiling.is_none() {
        crate::service_busy::enter_step(format!("{} in {}", command_text(&request.arguments), request.cwd.display()))
    } else {
        None
    };
    let output: std::process::Output =
        output_watched_within(&mut command, subject.as_str(), ledger_step.as_str(), terms, may_be_rerun(request), ceiling)?;
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
///  success needs a start and a zero exit, and each failure names the command.
/// Why:
///   A step stopped at its bound did start, and its own text already says which command and why;
///  every other start failure and every non-zero exit is named here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkedFrom(outcome: GitOutput | Error, args: string[]): GitOutput; // throws CommitTransactionGitError
/// ```
fn checked_from(outcome: std::io::Result<GitOutput>, arguments: &[OsString]) -> Result<GitOutput, GitFailure> {
    let output: GitOutput = match outcome {
        Ok(finished) => finished,
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => {
            return Err(GitFailure(error.to_string()));
        }
        Err(error) => {
            return Err(GitFailure(format!("{} could not start: {error}", command_text(arguments))));
        }
    };
    if !output.succeeded() {
        return Err(GitFailure(format!("{} failed: {}", command_text(arguments), output.error_text())));
    }
    return Ok(output);
}

/// Real-Git runner controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_git_tests.rs"]
mod tests;
