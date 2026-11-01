//! What:
//!  Start real Git with the caller's exact arguments,
//!  streams and environment.
//! Why:
//!  The wrapper adds policy around Git;
//!  it must never change what Git receives
//!      or what the caller observes from Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // spawn(gitPath, args, { stdio: 'inherit', env }) and exit with the child's status.
//! ```

/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Arguments are forwarded byte for byte;
///  they are never decoded or re-quoted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // process.argv entries, but without any UTF-8 decoding.
/// ```
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::Path;
/// Starts that ride out a short shortage of processes.
#[cfg(unix)]
use super::start_retry::exec_retrying;
/// Starts that ride out a short shortage of processes.
use super::start_retry::RetryStart;
/// `Command` builds a child process from a program and an argument list,
///  with no shell.
use std::process::Command;

/// What:
///  How a Git child ended.
///       `i32` is a signed 32-bit integer,
///  the type of exit codes and signal numbers
///       (siblings `u8`,
///  `i64`).
/// Why:
///   A child killed by a signal has no exit code;
///  callers need both cases to
///       report Git's result faithfully.
///  `i32` is what the operating system returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildOutcome = { exited: number } | { signaled: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildOutcome {
    /// The child called `exit` with this code.
    Exited(i32),
    /// The child was terminated by this signal number.
    Signaled(i32),
}

/// What:
///  Build the child command:
///  program,
///  unchanged arguments,
///  added variables.
///       `&[OsString]` borrows the argument list;
///  `&[(OsString, OsString)]` borrows
///       name/value pairs.
/// Why:
///   One builder guarantees every real-Git start,
///  forwarded or queried,
///  uses an
///       argument array (no shell string) and inherits the caller's environment plus
///       only the overlay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitCommand(realGit: string, args: string[], overlay: [string, string][]): Command;
/// ```
pub fn git_command(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> Command {
    // `mut` allows the builder to be configured step by step.
    let mut command: Command = Command::new(real_git);
    // Traditional hooks never run through the wrapper; see `hooks_off`.
    super::hooks_off::disable_traditional_hooks(&mut command, arguments);
    // `.args` appends each argument as its own array element, bytes unchanged.
    command.args(arguments);
    // `for (name, value) in overlay` destructures each borrowed pair.
    for (name, value) in overlay {
        command.env(name, value);
    }
    return command;
}

/// What:
///  Translate a child's end into this process's exit code.
/// Why:
///   Git 2.56.0 `run-command.c` reports a child killed by signal N as `128 + N`,
///       which is also what a POSIX shell reports;
///  an exit code passes through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function exitCode(outcome: ChildOutcome): number;
/// ```
pub fn exit_code(outcome: ChildOutcome) -> i32 {
    // What: `match` picks by variant and binds the number inside it.
    // Why:  The two cases are computed differently and both must be handled.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return 'exited' in outcome ? outcome.exited : 128 + outcome.signaled;
    // ```
    match outcome {
        ChildOutcome::Exited(code) => return code,
        ChildOutcome::Signaled(signal) => return 128 + signal,
    }
}

/// What:
///  Read a finished child's status into a `ChildOutcome`.
///       `#[cfg(unix)]` on the inner block compiles it only on Unix-like systems.
/// Why:
///   Unix reports either an exit code or a terminating signal,
///  never both.
///  Other
///       systems report only an exit code.
///  One function serves both,
///  so the part every
///       system shares is the part every test run exercises;
///  a status that is neither
///       cannot be reported faithfully and counts as a general failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function outcomeOf(status: { code: number | null; signal: number | null }): ChildOutcome;
/// ```
pub(crate) fn outcome_of(status: std::process::ExitStatus) -> ChildOutcome {
    // `if let Some(code) = ...` runs only when the child exited normally.
    if let Some(code) = status.code() {
        return ChildOutcome::Exited(code);
    }
    #[cfg(unix)]
    {
        // The trait adds `.signal()` to exit statuses on Unix.
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return ChildOutcome::Signaled(signal);
        }
    }
    return ChildOutcome::Exited(1);
}

/// What:
///  Run real Git as a child with inherited stdin,
///  stdout and stderr,
///  and wait.
///       `std::io::Result<T>` is `Result<T, std::io::Error>`.
/// Why:
///   Callers that must do work after Git returns (post-commit policies,
///  status
///       notes) need the wrapper process to survive Git.
///  The child shares the
///       caller's terminal and streams directly;
///  nothing is buffered or copied.
///       This path installs no signal handlers:
///  a signal sent to the wrapper alone
///       does not reach Git,
///  exactly as with the TypeScript wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runRealGit(realGit, args, overlay): Promise<ChildOutcome>;
/// ```
pub fn run_real_git(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> std::io::Result<ChildOutcome> {
    // What: `.status()` starts the child with inherited streams and waits for it;
    //       a trailing `?` returns a start failure to our caller.
    // Why:  Inherited streams are the default for `.status()`, so Git reads the
    //       caller's stdin and writes the caller's stdout and stderr itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const status = await spawn(realGit, args, { stdio: 'inherit' });
    // ```
    let status: std::process::ExitStatus = git_command(real_git, arguments, overlay).status_retrying()?;
    // `Ok(...)` is the success variant carrying how the child ended.
    return Ok(outcome_of(status));
}

/// What:
///  Become real Git;
///  returns only when Git could not be started.
///       The returned `std::io::Error` says why the operating system refused.
///       `#[cfg(unix)]` and `#[cfg(not(unix))]` each compile one of the two inner blocks.
/// Why:
///   On Unix this process is replaced by Git,
///  which is exact forwarding:
///  same process
///       ID,
///  same streams,
///  same signals,
///  same exit status,
///  and no wrapper process left
///       between the caller and Git.
///  Windows cannot replace a process image;
///  the nearest
///       faithful behaviour there is to wait for Git and exit with its status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Unix has no Node equivalent: process.execve(realGit, args, env) replaces the process image.
/// // Elsewhere: process.exit(exitCode(await runRealGit(realGit, args, overlay)));
/// ```
pub fn replace_process_with_real_git(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> std::io::Error {
    #[cfg(unix)]
    {
        // `exec_retrying` replaces this process like `.exec()` and tries again when the start is refused.
        return exec_retrying(&mut git_command(real_git, arguments, overlay));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        // `match` unpacks the run: success exits this process with Git's code.
        match run_real_git(real_git, arguments, overlay) {
            Ok(outcome) => std::process::exit(exit_code(outcome)),
            Err(error) => return error,
        }
    }
}

/// Real-Git process controls stay out of the release executable.
#[cfg(test)]
#[path = "forwarding_tests.rs"]
mod tests;
