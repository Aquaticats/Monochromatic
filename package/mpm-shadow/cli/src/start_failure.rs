//! What:
//! The message for a real `mpm` that could not be started.
//! Why:
//! The person sees only this text:
//! it must name the command they ran,
//! the program tried,
//! the stage,
//! the cause in
//! plain words,
//! the remedy,
//! the exit status the shadow returns and what had completed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function describeStartFailure(failure: StartFailure): string
//! ```

/// Imports the owned operating-system text type.
use std::ffi::OsString;
/// Imports the error of a failed start.
use std::io::{Error, ErrorKind};
/// Imports the borrowed path type.
use std::path::Path;

/// The exit status of a start that failed before the real `mpm` ran,
/// which the message tells the reader to expect.
use crate::constants::EXEC_FAILED_EXIT_CODE;
/// The deepest nesting of shadow starts allowed,
/// named in the message about a nested start.
use crate::nesting::MAX_NESTING;
/// What the retry of a refused start did,
/// which the message reports.
use crate::start_retry::RetryReport;

/// Everything the message needs to know.
pub struct StartFailure<'a> {
    /// The arguments `mpm` was run with (after the program name).
    pub outer: &'a [OsString],
    /// The program the shadow tried to start as the real `mpm`.
    pub program: &'a Path,
    /// Why the start failed.
    pub error: &'a Error,
    /// What the retrying did before giving up.
    pub retry: RetryReport,
    /// How many shadows sit above this one.
    pub depth: u32,
}

/// Whether a character forces an argument to be quoted:
/// whitespace splits a word and a quote mark would end the quoting.
fn forces_quotes(character: char) -> bool {
    return character.is_whitespace() || character == '\'' || character == '"';
}

/// Characters after which an argument must be quoted to stay one word in the printed command line.
fn needs_quotes(argument: &str) -> bool {
    return argument.is_empty() || argument.chars().any(forces_quotes);
}

/// What:
/// The command line the person ran,
/// as they would type it:
/// `mpm`,
/// then each argument,
/// in single quotes when it holds a space or a quote.
/// Why:
/// The message must name the command,
/// and an argument with a space has to stay one argument to be recognizable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function commandLine(outer: string[]): string
/// ```
pub fn command_line(outer: &[OsString]) -> String {
    let mut text: String = String::from("mpm");
    for argument in outer {
        let lossy: String = argument.to_string_lossy().to_string();
        text.push(' ');
        if needs_quotes(&lossy) {
            text.push('\'');
            text.push_str(&lossy.replace('\'', "'\\''"));
            text.push('\'');
        } else {
            text.push_str(&lossy);
        }
    }
    return text;
}

/// The sentence about nesting,
/// empty at the top level.
fn nesting_note(depth: u32) -> String {
    if depth == 0 {
        return String::new();
    }
    return format!(
        "\n  nesting: this mpm-shadow was started by another mpm-shadow (nesting level {depth}); one shadow starting another is not the intended chain"
    );
}

/// The cause and the remedy for one kind of start failure,
/// in plain words.
fn cause_and_remedy(failure: &StartFailure) -> (String, String) {
    let program: String = failure.program.display().to_string();
    return match failure.error.kind() {
        ErrorKind::WouldBlock => {
            let waited: String = if failure.retry.refusals > 0 {
                format!(
                    " The start was refused {} times over {} ms of waiting (pauses that grow), then given up.",
                    failure.retry.refusals,
                    failure.retry.paused.as_millis()
                )
            } else {
                String::new()
            };
            (
                format!(
                    "the operating system refused to create another process ({}): a process or task limit was reached (the per-user `ulimit -u`, or the limit of the service, container or cgroup the command runs in).{waited}",
                    failure.error
                ),
                "wait for other work under the same user, service or container to finish, or raise the limit (`ulimit -u` for the user, `TasksMax` for a systemd unit, `--pids-limit` for a container), then run the command again; nothing was started, so there is nothing to clean up".to_string(),
            )
        }
        ErrorKind::PermissionDenied => (
            format!("the operating system denied permission to run the program ({})", failure.error),
            format!("give the program execute permission (`chmod +x {program}`) and make sure every directory on its path can be searched, then run the command again"),
        ),
        ErrorKind::NotFound => (
            format!("the program, or the interpreter named on its first line (`#!`), does not exist ({})", failure.error),
            "check that the program still exists and that the interpreter named on its first line is installed, then run the command again".to_string(),
        ),
        _ => (
            format!("the operating system could not start the program ({})", failure.error),
            format!("run `{program} --version` by hand to see the same error outside the shadow, fix what it reports, then run the command again"),
        ),
    };
}

/// What:
/// The message for a real `mpm` that could not be started:
/// the command that was run,
/// the program tried,
/// the stage,
/// the cause in plain words,
/// what to do,
/// the exit status and why,
/// and what had completed.
/// Why:
/// The person sees only this text and must be able to act on it without reading the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function describeStartFailure(failure: StartFailure): string
/// ```
pub fn describe_start_failure(failure: &StartFailure) -> String {
    let command: String = command_line(failure.outer);
    let (cause, remedy) = cause_and_remedy(failure);
    return format!(
        "mpm-shadow: the real `mpm` could not be started, so `{command}` did not run.\n  \
         command you ran: {command}\n  \
         program it tried to start: {program} (the first `mpm` on PATH that is not this shadow)\n  \
         stage: starting the real `mpm` to run that command{nesting}\n  \
         cause: {cause}\n  \
         what to do: {remedy}\n  \
         exit status: mpm-shadow exits with {code}, the shell convention for a program that was found but could not be started; the command you ran therefore reports {code}, not an exit status of mpm's own\n  \
         what completed: nothing of `{command}` ran, so nothing it would have installed or changed was; post-run cleanup did not run either, because no mpm command finished",
        program = failure.program.display(),
        nesting = nesting_note(failure.depth),
        code = EXEC_FAILED_EXIT_CODE,
    );
}

/// What:
/// The message for a shadow that refuses to start `program` because too many shadows already sit above it.
/// Why:
/// A chain of shadows,
/// each taking the next for the real `mpm`,
/// grows until the host runs out of processes;
/// stopping it at a bound turns that into a short,
/// explained failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function describeNestingRefusal(outer: string[], program: string, depth: number): string
/// ```
pub fn describe_nesting_refusal(outer: &[OsString], program: &Path, depth: u32) -> String {
    let command: String = command_line(outer);
    return format!(
        "mpm-shadow: refusing to start {program} for `{command}`: this mpm-shadow was started by another mpm-shadow (nesting level {depth}, limit {limit}).\n  \
         cause: a shadow found a shadow (or a launcher of one) on PATH and took it for the real `mpm`, so each started the next; a shadow that starts itself would go on until the host refuses new processes.\n  \
         what to do: put the real `mpm` on PATH after the shadow, or find which `mpm` on PATH is a shadow (`command -v -a mpm`), then run the command again.\n  \
         exit status: mpm-shadow exits with {code}; nothing of `{command}` ran in this process, and the post-run cleanup did not run.",
        program = program.display(),
        limit = MAX_NESTING,
        code = EXEC_FAILED_EXIT_CODE,
    );
}
