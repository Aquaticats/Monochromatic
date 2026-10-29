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
use super::forwarding::git_command;
/// Private exclusive files for standard input.
use super::private_storage::write_exclusive_scratch;
/// Unique temporary names.
use super::random_id::random_uuid;
/// `OsStr`/`OsString` are borrowed/owned operating-system text of raw bytes.
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Command` builds a child process;
///  `Stdio` configures its streams.
use std::process::{Command, Stdio};

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
/// Why:
///   Git may write output before it reads all its input;
///  feeding input from a file instead
///       of a pipe can never deadlock and needs no writer thread.
///  The name is removed at once,
///       so nothing is left behind even after a crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // child.stdin.end(input);
/// ```
fn input_file(bytes: &[u8]) -> std::io::Result<std::fs::File> {
    let unique: String = match random_uuid() {
        Ok(id) => id,
        Err(error) => return Err(std::io::Error::other(error.0)),
    };
    let path: PathBuf = std::env::temp_dir().join(format!("cli-git-input-{unique}"));
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
    let mut command: Command = build_command(context, request);
    match &request.input {
        Some(bytes) => {
            command.stdin(Stdio::from(input_file(bytes.as_slice())?));
        }
        None => {
            command.stdin(Stdio::null());
        }
    }
    let output: std::process::Output = command.output()?;
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
    let output: GitOutput = match run_git(context, request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(GitFailure(format!(
                "{} could not start: {error}",
                command_text(request.arguments.as_slice())
            )));
        }
    };
    if !output.succeeded() {
        return Err(GitFailure(format!(
            "{} failed: {}",
            command_text(request.arguments.as_slice()),
            output.error_text()
        )));
    }
    return Ok(output);
}

/// Real-Git runner controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_git_tests.rs"]
mod tests;
