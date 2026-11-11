//! What:
//!  The service's Git for a push:
//! the real Git,
//! its working directory and environment,
//! the command builder,
//! the
//!  deadline-bound read-only query and the failure types of a push.
//! Why:
//!  The push,
//! its reconcile and the queries before them all start Git the same way,
//! so none re-enters the wrapper
//!  and every Git child receives the same environment.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const output = await nanoSpawn(gitPath, args, { cwd, env });
//! ```

use crate::bounded_push::STOP_GRACE;
use crate::push_deadline_advice::limit_clause;
use crate::bounded_query::{QueryOutput, query_window, run_bounded_query, run_bounded_query_awaiting};
use crate::js_text::trim_javascript;
/// Owned OS strings for Git arguments and the child environment.
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;
/// `Command` starts real Git.
use std::process::Command;

/// What:
///  Why a push or one of its Git queries could not finish.
/// Why:
///   Every variant is a failure the service reports,
/// and the commit stays saved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SubprocessError | TypeError | NodeJS.ErrnoException
/// ```
#[derive(Debug)]
pub enum SingleFlightError {
    /// Git could not be started,
    /// or failed in a way the incumbent does not model as an exit status.
    Start(std::io::Error),
    /// A Git query exited with a status the caller did not expect.
    Git {
        /// Arguments of the query,
        /// for the diagnostic.
        arguments: String,
        /// Exit code,
        /// or nothing when a signal ended the process.
        code: Option<i32>,
        /// Complete standard error text of the query.
        stderr: String,
    },
    /// A coordination file could not be read or published.
    Io(std::io::Error),
    /// A Git query showed no sign of work for its quiet window and was stopped.
    TimedOut {
        /// Arguments of the query,
        /// for the diagnostic.
        arguments: String,
        /// The quiet window it exceeded,
        /// in whole seconds.
        seconds: u64,
    },
    /// A command of a merge step was still running at its total ceiling and was stopped.
    Ceiling {
        /// Arguments of the command,
        /// for the diagnostic.
        arguments: String,
        /// The ceiling it passed,
        /// in whole seconds.
        seconds: u64,
    },
}

impl std::fmt::Display for SingleFlightError {
    /// Writes the diagnostic text of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Start(error) => write!(formatter, "git could not be started: {error}"),
            Self::Git {
                arguments,
                code,
                stderr,
            } => write!(
                formatter,
                "git {arguments} exited with {code:?}: {}",
                stderr.trim_end()
            ),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::TimedOut { arguments, seconds } => write!(
                formatter,
                "git {arguments} showed no sign of work for {seconds} seconds and was stopped; {}",
                limit_clause()
            ),
            Self::Ceiling { arguments, seconds } => write!(
                formatter,
                "git {arguments} was still running at its {seconds}-second ceiling and was stopped: one command of a merge step may \
                 not run for ever, so the merge it was computing is not made. Nothing was changed: the commit is kept, no reference \
                 moved and the worktree was not touched. Hint: a merge driver that keeps working without returning does this; make \
                 it return, or set {} higher",
                crate::push_deadline_and_patience_settings::MERGE_CEILING_VARIABLE
            ),
        };
    }
}

impl std::error::Error for SingleFlightError {}

/// What:
///  One push this invocation ran itself.
/// Why:
///   `output` is Git's interleaved output with one trailing line break removed,
/// as the incumbent's
///  `nano-spawn` `output` is;
/// a successful push's output is never recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushAttempt = { exitCode: number; output: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushAttempt {
    /// Push exit code,
    /// `0` on success.
    pub exit_code: i64,
    /// Complete interleaved push output.
    pub output: String,
}

/// What:
///  The real Git the service runs,
/// where,
/// and with which environment.
/// Why:
///   The service passes the complete environment in (the invocation's variables with the overlay applied),
///  and nothing is read from the process environment here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushGit = { gitPath: string; cwd: string; environment: NodeJS.ProcessEnv };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushGit {
    /// Absolute real Git executable.
    pub real_git: PathBuf,
    /// Directory the commit landed in.
    pub cwd: PathBuf,
    /// Complete environment every Git child receives:
    /// the invocation's variables with the overlay applied.
    pub environment: Vec<(OsString, OsString)>,
    /// Whether the pushes of this coordinator must upload LFS objects first (`lfs_bounded_push`);
    /// queries and fetches never do.
    pub lfs: bool,
}

/// What:
///  Build a real Git command with the given arguments,
/// directory and environment.
/// Why:
///   Every coordination query and push starts Git this way,
/// so none re-enters the wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nanoSpawn(gitPath, args, { cwd })
/// ```
pub fn git_command(git: &PushGit, arguments: &[OsString]) -> Command {
    let mut command = Command::new(&git.real_git);
    crate::hooks_off::add_arguments(&mut command, arguments, crate::hooks_off::StartOwner::Wrapper);
    let mut in_place = in_push_environment(command, git);
    // The push environment is built from nothing, so the marker is added after it: this start says it is Git's too.
    crate::started_as_git::mark_real_git_start(&mut in_place, crate::started_as_git::StartMode::Child);
    return in_place;
}

/// What:
///  Give `command` the push's working directory and its exact environment.
/// Why:
///   Both builders of push Git commands run in the same place with the same environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { ...command, cwd: git.cwd, env: git.environment }
/// ```
fn in_push_environment(mut command: Command, git: &PushGit) -> Command {
    command.current_dir(&git.cwd).env_clear();
    for pair in &git.environment {
        command.env(&pair.0, &pair.1);
    }
    return command;
}

/// The arguments as one display string,
/// for diagnostics.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// args.join(' ')
/// ```
fn describe_arguments(arguments: &[OsString]) -> String {
    let mut shown: Vec<String> = Vec::new();
    for argument in arguments {
        shown.push(argument.to_string_lossy().into_owned());
    }
    return shown.join(" ");
}

/// What:
///  Run a read-only Git query and return its trimmed standard output.
///       `Result<A, B>` is either the output or a failure.
/// Why:
///   Any failed query is a coordination failure;
/// it is never read as an answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await nanoSpawn(gitPath, args, { cwd })).stdout.trim();
/// ```
pub fn git_output(git: &PushGit, arguments: &[OsString]) -> Result<String, SingleFlightError> {
    let output = run_query(git, arguments)?;
    return output_text(arguments, &output);
}

/// What:
///  The same read,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The reconcile and the push of the landing round read the object store as tasks,
/// and the synchronous face refuses a call from inside the runtime.
/// The answer is read and trimmed the same way,
/// so the two cannot differ in what they hand back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await query(args)).stdout.trim();
/// ```
pub async fn git_output_awaiting(git: &PushGit, arguments: &[OsString]) -> Result<String, SingleFlightError> {
    let output = run_query_awaiting(git, arguments).await?;
    return output_text(arguments, &output);
}

/// What:
///  The trimmed standard output of a query that exited zero,
/// or the failure that names the query otherwise.
/// Why:
///   Any failed query is a coordination failure,
/// and it is never read as an answer;
/// both faces of `git_output`
///  decide that here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (output.code !== 0) throw new GitError(args, output.code, output.stderr); return trimJavascript(output.stdout);
/// ```
fn output_text(arguments: &[OsString], output: &QueryOutput) -> Result<String, SingleFlightError> {
    if output.code != Some(0) {
        return Err(SingleFlightError::Git {
            arguments: describe_arguments(arguments),
            code: output.code,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    let text = String::from_utf8_lossy(&output.stdout);
    return Ok(trim_javascript(&text).to_owned());
}

/// What:
///  Run a read-only Git query that is ended only when it shows no sign of work,
/// and capture its streams separately.
/// Why:
///   A query that never returns would hold the caller,
/// and the service's push behind it,
/// for good.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await nanoSpawn(gitPath, args, { cwd, signal: quietFor(windowMs) })
/// ```
pub fn run_query(git: &PushGit, arguments: &[OsString]) -> Result<QueryOutput, SingleFlightError> {
    // An exact read-only command line the in-process readers know costs no Git process.
    if let Some((code, stdout, stderr)) = crate::repo_read_hook::try_complete(arguments, git.cwd.as_path(), crate::git_system_config::with_real_git(git.environment.as_slice(), git.real_git.as_path()).as_slice()) {
        return Ok(QueryOutput { code, stdout, stderr });
    }
    return run_bounded_query(
        git_command(git, arguments),
        describe_arguments(arguments).as_str(),
        query_window(&git.environment),
        STOP_GRACE,
        git.environment.as_slice(),
    );
}

/// What:
///  The same query,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The synchronous face enters the runtime for the child's wait and is refused from inside it,
/// and the landing round
///  runs its items as tasks (`foundation/parallel_runner_on_the_runtime_awaiting.rs`).
/// The in-process answer,
/// the command line,
/// the quiet window and the grace are the synchronous face's,
///  so the two cannot differ in what they ask or wait for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await nanoSpawn(gitPath, args, { cwd, signal: quietFor(windowMs) })
/// ```
pub async fn run_query_awaiting(git: &PushGit, arguments: &[OsString]) -> Result<QueryOutput, SingleFlightError> {
    // An exact read-only command line the in-process readers know costs no Git process.
    if let Some((code, stdout, stderr)) = crate::repo_read_hook::try_complete(arguments, git.cwd.as_path(), crate::git_system_config::with_real_git(git.environment.as_slice(), git.real_git.as_path()).as_slice()) {
        return Ok(QueryOutput { code, stdout, stderr });
    }
    return run_bounded_query_awaiting(
        git_command(git, arguments),
        describe_arguments(arguments).as_str(),
        query_window(&git.environment),
        STOP_GRACE,
        git.environment.as_slice(),
    )
    .await;
}

/// Query behavior is controlled in the test module.
#[cfg(test)]
#[path = "auto_push_single_flight_tests.rs"]
mod tests;
