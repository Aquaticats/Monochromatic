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
//! The two reasons a start does not happen are the system's refusal
//! and the program being this tool's own shadow,
//! so the cause and the remedy are chosen by which one happened.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function describeStartFailure(failure: StartFailure): string
//! ```

/// Imports the owned operating-system text type.
use std::ffi::OsString;
/// Imports the error of a failed start.
use std::io::{
    Error,
    ErrorKind,
};
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
    /// Why the start did not happen.
    pub refusal: &'a StartRefusal,
    /// What the retrying did before giving up.
    pub retry: RetryReport,
    /// How many shadows sit above this one.
    pub depth: u32,
}

/// What:
/// `pub enum StartRefusal` says why a start did not happen:
/// the system answered the open or the start with an error,
/// or the program is this tool's own shadow,
/// or leads to one.
/// Why:
/// The two need different words.
/// A shadow's sentence carried inside an error of the kind "something else"
/// reached the arm for causes nothing has classified,
/// whose advice was to run the program by hand:
/// a shadow run by hand answers as a shadow and never shows the refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StartRefusal = { kind: 'system'; error: Error } | { kind: 'shadow'; sentence: string };
/// ```
#[derive(Debug)]
pub enum StartRefusal {
    /// The system's own answer to the open or the start.
    System {
        /// The error the system answered with,
        /// its kind whole.
        error: Error,
    },
    /// The program is this tool's own shadow,
    /// or a launcher record that leads to one.
    Shadow {
        /// The sentence naming the path and the shadow it is or leads to.
        sentence: String,
    },
}

/// What:
/// What a `StartRefusal` answers a caller that only wants an error to hand on.
/// Why:
/// `run_program` reports a start it could not make as an error and prints no message,
/// so it needs the refusal's contents as an error rather than as words.
impl StartRefusal {
    /// What:
    /// `pub fn into_error(self) -> Error` turns a refusal into the error
    /// a caller that reports failures as errors hands on.
    /// Why:
    /// `run_program` answers with an error and prints no message of its own,
    /// so the shadow's sentence becomes that error's text and keeps its words.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function intoError(refusal: StartRefusal): Error
    /// ```
    pub fn into_error(self) -> Error {
        return match self {
            StartRefusal::System {
                error,
            } => error,
            // What:     `Error::other(sentence)` builds an error of the kind `Other`, which carries no claim about the cause.
            // Why:      The sentence is the whole answer here, and no operating-system error describes it.
            StartRefusal::Shadow {
                sentence,
            } => Error::other(sentence),
        };
    }
}

/// What:
/// `const ENOEXEC: i32 = 8` is the number the kernel answers with when a file
/// is not a program it can start ("Exec format error").
/// Why:
/// `std::io::ErrorKind` has no variant of its own for it
/// (`Error::from_raw_os_error(8).kind()` is `Uncategorized`),
/// so its arm is chosen by the number.
/// The number is 8 on Linux (`asm-generic/errno-base.h`) and on the other Unix systems this crate is built for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ENOEXEC = 8;
/// ```
const ENOEXEC: i32 = 8;

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

/// What:
/// The cause and the remedy for a start the system refused because another process holds open for writing
/// a file this start needs.
/// Why:
/// `execve` refuses such a file (`ETXTBSY`),
/// and both the shadow mode and the launcher mode meet that refusal,
/// so the words live in one place and both say the same thing:
/// some process holds open for writing a file this start needs,
/// that file is the program itself or the interpreter named on its first line,
/// this tool cannot tell which process holds it or why,
/// and the same command works once the write has finished and left a whole program.
/// It never names a usual cause and never claims a replacement is in progress:
/// the refusal proves a write,
/// and nothing about who writes or why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function beingWrittenCauseAndRemedy(error: Error): [string, string]
/// ```
pub fn being_written_cause_and_remedy(error: &Error) -> (String, String) {
    let cause: String = format!(
        "the system refused to start it because a file this start needs is open for writing by some other process ({error}): the program itself, or the interpreter named on the program's first line (`#!`). mpm-shadow cannot tell which process holds that file, or why it holds it"
    );
    let remedy: String = "run the same command again once that process has finished with the file: nothing of this command ran, so nothing has to be undone, and the start succeeds as soon as the file is free and whole. If a write was cut short, the file is a broken program or interpreter, and starting it again will not work: install or upgrade it again, which writes a whole new copy into place".to_string();
    return (cause, remedy);
}

/// What:
/// The cause and the remedy for a start that did not happen,
/// chosen by which kind of refusal it was.
/// Why:
/// A shadow found where the real `mpm` should run is not a program fault,
/// so it must not share the words for a cause nothing has classified.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function causeAndRemedy(failure: StartFailure): [string, string]
/// ```
fn cause_and_remedy(failure: &StartFailure) -> (String, String) {
    return match failure.refusal {
        StartRefusal::Shadow {
            sentence,
        } => shadow_cause_and_remedy(sentence),
        StartRefusal::System {
            error,
        } => system_cause_and_remedy(failure, error),
    };
}

/// What:
/// The cause and the remedy for a program that is this tool's own shadow,
/// or leads to one:
/// what was found,
/// why starting it would loop,
/// and what repairs it.
/// Why:
/// The reader must be able to tell a race with an install from a broken link,
/// and the shadow itself cannot say which one happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shadowCauseAndRemedy(sentence: string): [string, string]
/// ```
fn shadow_cause_and_remedy(sentence: &str) -> (String, String) {
    let cause: String = format!(
        "{sentence}: the PATH lookup a moment earlier took that path for the real `mpm`, and the check this start makes on the open file found a shadow there, so the file at that path, or a launcher record beside it, was different when the lookup read it. An install, an upgrade, or a launcher writing its record in another session changes exactly those, and mpm-shadow cannot tell which of them changed, or which process changed it. Starting it would put a shadow where the real `mpm` has to run: a shadow looks for the real `mpm` again, and two shadows that each stood first for the other started each other without end when this was measured on 2026-11-07 (about 105 starts a second), so the guards that stop a shadow starting a shadow end such a run instead of letting the command through"
    );
    let remedy: String = format!(
        "run the command again: the lookup skips every shadow, so the next run reads the next `mpm` on PATH, or says that no real `mpm` is left there. If the same refusal returns, `command -v -a mpm` lists every `mpm` on PATH in the order the lookup reads them: the shadow's own directory belongs first and the real `mpm`'s after it, so a further `mpm` that leads to a shadow is one to remove, and a launcher whose record names a shadow is one to have written again by `mpm install` in the tree it belongs to. Nothing of the command ran, so nothing has to be undone"
    );
    return (cause, remedy);
}

/// What:
/// The cause and the remedy for one kind of error the system answered a start with.
/// Why:
/// Each arm names what the refusal proves and every step that can act on it,
/// and no arm names a step that shows nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function systemCauseAndRemedy(failure: StartFailure, error: Error): [string, string]
/// ```
fn system_cause_and_remedy(
    failure: &StartFailure,
    error: &Error,
) -> (String, String) {
    let program: String = failure.program.display().to_string();
    return match error.kind() {
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
                    "the operating system refused to create another process ({error}): a process or task limit was reached (the per-user `ulimit -u`, or the limit of the service, container or cgroup the command runs in).{waited}"
                ),
                "wait for other work under the same user, service or container to finish, or raise the limit (`ulimit -u` for the user, `TasksMax` for a systemd unit, `--pids-limit` for a container), then run the command again; nothing was started, so there is nothing to clean up".to_string(),
            )
        }
        ErrorKind::ExecutableFileBusy => being_written_cause_and_remedy(error),
        ErrorKind::PermissionDenied => (
            format!("the operating system denied permission to run the program ({error})"),
            format!("give the program execute permission (`chmod +x {program}`) and make sure every directory on its path can be searched, then run the command again"),
        ),
        ErrorKind::NotFound => (
            format!(
                "the program, or the interpreter named on its first line (`#!`), is not there ({error}). A program can be gone because an install or an upgrade moved it aside and has not yet named the new copy into place, so a start that lands between those two steps finds nothing; an interpreter can be missing because it is not installed"
            ),
            "run the same command again: an install or an upgrade that had moved the program aside has finished by then. If the program, or the interpreter its first line names, is still missing, install it again".to_string(),
        ),
        // What:     The file is not a program the kernel can start, and no shell shows that refusal.
        // Why:      `ETXTBSY` has a kind, this does not; the arm is chosen by the kernel's number.
        _ if error.raw_os_error() == Some(ENOEXEC) => (
            format!(
                "the file this start chose is not a program the kernel can start ({error}): it is empty or cut short, it holds a program for another operating system or processor, or it is not a program at all. Nothing was started, and a shell that is given this file runs it as a script or does nothing instead of reporting the refusal, so running it by hand shows something else"
            ),
            format!(
                "install or upgrade the program again, which writes a whole new copy into place, then run the command again; if `{program}` is not a program this tool installs, put the real program back in its place. Nothing of this command ran, so nothing has to be undone"
            ),
        ),
        _ => (
            format!(
                "the operating system could not start the program ({error}), and no arm of this message knows that refusal: the system's own words are the whole of what mpm-shadow can say about the cause"
            ),
            format!(
                "read those words, then run the same command again: nothing of this command ran, and a refusal caused by a shortage (of memory, or of open files) is over by then. When the same words come back they name what was refused and are the thing to act on: words about the program's own file mean a file that is damaged or not whole, which installing or upgrading that program again replaces; words about a limit mean work to wait for, or a limit to raise for this command. Running `{program} --version` by hand repeats the same words only in the first case: a shell may run a damaged file as a script instead of reporting the refusal, and a limit of the shadow's own process is not your shell's, so the hand run is a second reading, never the proof"
            ),
        ),
    };
}

/// What:
/// What the message says the program is,
/// which depends on how the start ended.
/// Why:
/// "The first `mpm` on PATH that is not this shadow" is exactly what the lookup chose
/// and exactly what the refused shadow is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function programNote(refusal: StartRefusal): string
/// ```
fn program_note(refusal: &StartRefusal) -> &'static str {
    return match refusal {
        StartRefusal::System {
            ..
        } => "the first `mpm` on PATH that is not this shadow",
        StartRefusal::Shadow {
            ..
        } => "the `mpm` the PATH lookup chose, which at the start was or led to a shadow",
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
         program it tried to start: {program} ({note})\n  \
         stage: starting the real `mpm` to run that command{nesting}\n  \
         cause: {cause}\n  \
         what to do: {remedy}\n  \
         exit status: mpm-shadow exits with {code}, the shell convention for a program that was found but could not be started; the command you ran therefore reports {code}, not an exit status of mpm's own\n  \
         what completed: nothing of `{command}` ran, so nothing it would have installed or changed was; post-run cleanup did not run either, because no mpm command finished",
        program = failure.program.display(),
        note = program_note(failure.refusal),
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
pub fn describe_nesting_refusal(
    outer: &[OsString],
    program: &Path,
    depth: u32,
) -> String {
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
