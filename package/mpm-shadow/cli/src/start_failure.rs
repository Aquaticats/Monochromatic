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
use crate::constants::{
    EXEC_FAILED_EXIT_CODE,
    SHADOWED_NAME,
};
/// The reader that says which interpreter a program names,
/// and its answers.
use crate::named_interpreter::{
    NamedInterpreter,
    named_interpreter,
};
/// The deepest nesting of shadow starts allowed,
/// named in the message about a nested start.
use crate::nesting::MAX_NESTING;
/// Every `mpm` the PATH lookup saw,
/// and why it took none of the ones it passed over.
use crate::real_binary::{
    Candidate,
    Lookup,
    SkipReason,
};
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
    /// Every `mpm` the PATH lookup saw,
    /// which a refusal of a shadow has to name:
    /// the list itself is printed, so no sentence has to send the reader to a command a shell may reject.
    pub lookup: &'a Lookup,
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
/// The sentence naming the files a start of `program` needs before the program runs,
/// which is the program's own file and the interpreter its file names.
/// Why:
/// A compiled program has no first line,
/// so a sentence that says the interpreter is named there points the reader at a file the program does not
/// have;
/// the program's own bytes say which interpreter it names, and the answer is read where it is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function neededFiles(program: string): string
/// ```
fn needed_files(program: &Path) -> String {
    return match named_interpreter(program) {
        NamedInterpreter::Script {
            interpreter,
        } => format!(
            "the program itself, or the interpreter its first line names (`{}`)",
            interpreter.display()
        ),
        NamedInterpreter::DynamicLinker {
            linker,
        } => format!(
            "the program itself, or its dynamic linker (`{}`), which is the interpreter a compiled program names where a script has its `#!` line",
            linker.display()
        ),
        NamedInterpreter::NoInterpreter => "the program's own file, which is the only file that start needs: it is a compiled program and names no interpreter".to_string(),
        NamedInterpreter::Unknown => "the program itself, or the interpreter it names, which is the program after `#!` on a script's first line or a compiled program's dynamic linker".to_string(),
    };
}

/// What:
/// The cause and the remedy for a start the system refused because another process holds open for writing
/// a file this start needs,
/// for a caller that knows which program was started.
/// Why:
/// `execve` refuses such a file (`ETXTBSY`),
/// and both the shadow mode and the launcher mode meet that refusal,
/// so the words live in one place and both say the same thing:
/// some process holds open for writing a file this start needs,
/// that file is the program itself or the interpreter the program names,
/// this tool cannot tell which process holds it or why,
/// and the same command works once the write has finished and left a whole program.
/// It never names a usual cause and never claims a replacement is in progress:
/// the refusal proves a write,
/// and nothing about who writes or why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function beingWrittenCauseAndRemedyFor(program: string | undefined, error: Error): [string, string]
/// ```
pub fn being_written_cause_and_remedy_for(
    program: Option<&Path>,
    error: &Error,
) -> (String, String) {
    let needed: String = match program {
        Some(path) => needed_files(path),
        None => "the program itself, or the interpreter it names, which is the program after `#!` on a script's first line or a compiled program's dynamic linker".to_string(),
    };
    let cause: String = format!(
        "the system refused to start it because a file this start needs is open for writing by some other process ({error}): {needed}. mpm-shadow cannot tell which process holds that file, or why it holds it"
    );
    let remedy: String = "run the same command again once that process has finished with the file: nothing of this command ran, so nothing has to be undone, and the start succeeds as soon as the file is free and whole. If a write was cut short, the file is a broken program or interpreter, and starting it again will not work: install or upgrade it again, which writes a whole new copy into place".to_string();
    return (cause, remedy);
}

/// What:
/// The cause and the remedy for that same refusal,
/// for a caller that does not know which program was started.
/// Why:
/// The launcher mode's own start reports this refusal without passing its program,
/// and the words that name both kinds of interpreter are true without it too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function beingWrittenCauseAndRemedy(error: Error): [string, string]
/// ```
pub fn being_written_cause_and_remedy(error: &Error) -> (String, String) {
    return being_written_cause_and_remedy_for(None, error);
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
        "run the command again: the lookup skips every shadow, so the next run reads the next `mpm` on PATH, or reports that no real `mpm` is left there. If the same refusal returns, the `{SHADOWED_NAME}` commands this run found on PATH, listed in this message in the order the lookup read them, name what to change: the shadow's own directory belongs first and the real `mpm`'s after it, so a further `mpm` that leads to a shadow is one to remove, and a launcher whose record names a shadow is one to have written again by `mpm install` in the tree it belongs to. Nothing of the command ran, so nothing has to be undone"
    );
    return (cause, remedy);
}

/// What:
/// `const E2BIG: i32 = 7;` is the number the kernel answers a start with when the argument list is longer than
/// the limit in force (`asm-generic/errno-base.h`).
/// Why:
/// `std::io::ErrorKind` has no stable variant for it,
/// so its arm is chosen by the number,
/// as `ENOEXEC`'s is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const E2BIG = 7;
/// ```
#[cfg(target_os = "linux")]
const E2BIG: i32 = 7;

/// What:
/// `const ENAMETOOLONG: i32 = 36;` is the kernel's number for a path component longer than the file system
/// allows (`asm-generic/errno.h`).
/// Why:
/// The number is the only handle on this refusal, for the same reason as `E2BIG`'s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ENAMETOOLONG = 36;
/// ```
#[cfg(target_os = "linux")]
const ENAMETOOLONG: i32 = 36;

/// What:
/// `const ELOOP: i32 = 40;` is the kernel's number for a path with too many levels of symbolic links, which a
/// link leading back to itself makes (`asm-generic/errno.h`).
/// Why:
/// The number is the only handle on this refusal, for the same reason as `E2BIG`'s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ELOOP = 40;
/// ```
#[cfg(target_os = "linux")]
const ELOOP: i32 = 40;

/// What:
/// `fn uncurable(error: &Error) -> Option<&'static str>` answers the sentence for a refusal that running the same
/// command again cannot cure,
/// or nothing when repeating it can work.
/// `&'static str` is a borrowed text that lives as long as the program (sibling:
/// `String`, which owns its bytes and must be freed).
/// Why:
/// The last arm used to send every reader to run the same command again,
/// which for a too-long argument list or a link that loops is advice that cannot work;
/// the arm now says which refusals those are,
/// and this is the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function uncurable(error: Error): string | undefined
/// ```
fn uncurable(error: &Error) -> Option<&'static str> {
    #[cfg(target_os = "linux")]
    {
        return match error.raw_os_error() {
            Some(E2BIG) => Some(
                "Running the same command again cannot succeed: the kernel refused this argument list for being longer than the limit in force, and it refuses the same argument list the same way. Shorten the command line, or pass the arguments in a file or a variable the program reads, and run that",
            ),
            Some(ENAMETOOLONG) => Some(
                "Running the same command again cannot succeed: one part of the path to the program is longer than the file system allows, and the same path is refused the same way. Move the program to a shorter path and run the command again",
            ),
            Some(ELOOP) => Some(
                "Running the same command again cannot succeed: a symbolic link on the way to the program leads back to itself, and the same path loops the same way. Read each step of the program's path (`readlink` names where each link goes), repair the link that loops, and run the command again",
            ),
            _ => None,
        };
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = error;
        return None;
    }
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
        ErrorKind::ExecutableFileBusy => {
            being_written_cause_and_remedy_for(Some(failure.program), error)
        }
        ErrorKind::PermissionDenied => (
            format!("the operating system denied permission to run the program ({error})"),
            format!(
                "give the program execute permission (`chmod +x {program}`) and make sure every directory on its path can be searched, then run the command again"
            ),
        ),
        ErrorKind::NotFound => missing_cause_and_remedy(failure.program, error),
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
            last_arm_remedy(error),
        ),
    };
}

/// What:
/// The cause and the remedy for a start the system answered with `ENOENT`,
/// which is the program's absence or the absence of the interpreter it names.
/// Why:
/// `execve` answers one error for both,
/// and the program's own file says which interpreter it would need:
/// when that file is still there and the interpreter is not,
/// the cause can name the missing file instead of listing two possibilities.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingCauseAndRemedy(program: string, error: Error): [string, string]
/// ```
fn missing_cause_and_remedy(
    program: &Path,
    error: &Error,
) -> (String, String) {
    let display: String = program.display().to_string();
    return match named_interpreter(program) {
        NamedInterpreter::Script {
            interpreter,
        } if !interpreter.exists() => (
            format!(
                "`{display}` is there, but the interpreter its first line names (`{}`) is not ({error}): a script is started by that program, and this start finds nothing to start it with",
                interpreter.display()
            ),
            format!(
                "install the interpreter its first line names (`{}`), or install or upgrade the program again, which writes the whole file and its first line into place, then run the command again",
                interpreter.display()
            ),
        ),
        NamedInterpreter::DynamicLinker {
            linker,
        } if !linker.exists() => (
            format!(
                "`{display}` is there, but the dynamic linker it names (`{}`) is not ({error}): a compiled program cannot start without it, and this start finds nothing to start the program with",
                linker.display()
            ),
            format!(
                "install or upgrade the package that provides `{}`, then run the command again; when that linker belongs to the program's own install, installing or upgrading the program puts it back",
                linker.display()
            ),
        ),
        _ => (
            format!(
                "the program, or the interpreter it names, is not there ({error}). A program can be gone because an install or an upgrade moved it aside and has not yet named the new copy into place, so a start that lands between those two steps finds nothing; the interpreter a program names is the program after `#!` on a script's first line, or a compiled program's dynamic linker, and it can be missing because it is not installed"
            ),
            "run the same command again: an install or an upgrade that had moved the program aside has finished by then. If the program, or the interpreter it names, is still missing, install it again".to_string(),
        ),
    };
}

/// What:
/// The remedy for a refusal no arm of the message knows.
/// Why:
/// The arm names no step that cannot show the error:
/// a hand run of the program cannot produce a refusal of the shadow's own start,
/// and for the refusals repeating the command cannot cure,
/// the remedy says so instead of naming the same command again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lastArmRemedy(error: Error): string
/// ```
fn last_arm_remedy(error: &Error) -> String {
    let act_on: &str = "words about the program's own file mean a file that is damaged or not whole, which installing or upgrading that program again replaces; words about a limit mean work to wait for, or a limit to raise for this command";
    return match uncurable(error) {
        Some(sentence) => format!(
            "read those words: nothing of this command ran, so nothing has to be undone, and no step of this message shows the refusal again. {sentence}. The system's own words still say what to act on: {act_on}"
        ),
        None => format!(
            "read those words, then run the same command again: nothing of this command ran, and a refusal caused by a shortage (of memory, or of open files) is over by then. The system's own words still say what to act on: {act_on}"
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
    let mut lines: Vec<String> = vec![
        format!("mpm-shadow: the real `mpm` could not be started, so `{command}` did not run."),
        format!("  command you ran: {command}"),
        format!(
            "  program it tried to start: {} ({})",
            failure.program.display(),
            program_note(failure.refusal)
        ),
        format!(
            "  stage: starting the real `mpm` to run that command{}",
            nesting_note(failure.depth)
        ),
        format!("  cause: {cause}"),
    ];
    // What:     A refusal of a shadow is the one arm whose remedy names the `mpm` commands the lookup read.
    // Why:      The list is printed here, so no sentence has to send the reader to a command a shell may reject.
    if matches!(failure.refusal, StartRefusal::Shadow { .. }) {
        lines.push(commands_on_path(failure.lookup));
    }
    lines.push(format!("  what to do: {remedy}"));
    lines.push(format!(
        "  exit status: mpm-shadow exits with {EXEC_FAILED_EXIT_CODE}, the shell convention for a program that was found but could not be started; the command you ran therefore reports {EXEC_FAILED_EXIT_CODE}, not an exit status of mpm's own"
    ));
    lines.push(format!(
        "  what completed: nothing of `{command}` ran, so nothing it would have installed or changed was; post-run cleanup did not run either, because no mpm command finished"
    ));
    return lines.join("\n");
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
    lookup: &Lookup,
) -> String {
    let command: String = command_line(outer);
    let lines: Vec<String> = vec![
        format!(
            "mpm-shadow: refusing to start {} for `{command}`: this mpm-shadow was started by another mpm-shadow (nesting level {depth}, limit {limit}).",
            program.display(),
            limit = MAX_NESTING,
        ),
        "  cause: a shadow found a shadow (or a launcher of one) on PATH and took it for the real `mpm`, so each started the next; a shadow that starts itself would go on until the host refuses new processes.".to_string(),
        commands_on_path(lookup),
        format!(
            "  what to do: put the real `mpm` on PATH after the shadow, or remove the `mpm` from the `{SHADOWED_NAME}` commands this run found on PATH that is a shadow, then run the command again."
        ),
        format!(
            "  exit status: mpm-shadow exits with {EXEC_FAILED_EXIT_CODE}; nothing of `{command}` ran in this process, and the post-run cleanup did not run."
        ),
    ];
    return lines.join("\n");
}

/// What:
/// The line that introduces the `mpm` commands a lookup read,
/// and one line per candidate under it,
/// each with the reason the lookup took it or passed it over.
/// Why:
/// Every message that tells a person to sort out their `mpm` commands must show which ones were seen:
/// a file without its execute bit and a directory of that name need different repairs,
/// and neither can be guessed from "no real `mpm` found".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandsOnPath(lookup: Lookup): string
/// ```
fn commands_on_path(lookup: &Lookup) -> String {
    let mut lines: Vec<String> = vec![format!(
        "  the `{SHADOWED_NAME}` commands this run found on PATH, in the order the lookup read them:"
    )];
    if lookup.candidates.is_empty() {
        lines.push(format!(
            "    none: no directory on PATH holds a file named `{SHADOWED_NAME}`"
        ));
        return lines.join("\n");
    }
    for candidate in &lookup.candidates {
        lines.push(format!(
            "    {}: {}",
            candidate.path.display(),
            candidate_note(candidate)
        ));
    }
    return lines.join("\n");
}

/// What:
/// What one candidate's line says about it:
/// that the lookup chose it,
/// or why it passed it over.
/// Why:
/// The chosen line has to be recognizable as the program the message already names,
/// and each skipped line has to carry its own repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function candidateNote(candidate: Candidate): string
/// ```
fn candidate_note(candidate: &Candidate) -> String {
    return match &candidate.skipped_because {
        None => "chosen by the lookup".to_string(),
        Some(reason) => skipped_note(reason),
    };
}

/// What:
/// The words that name why the lookup passed one `mpm` over.
/// Why:
/// A person acts on the reason,
/// so it has to say what the file is rather than repeat that it was skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skippedNote(reason: SkipReason): string
/// ```
fn skipped_note(reason: &SkipReason) -> String {
    return match reason {
        SkipReason::ThisShadow => "skipped: it is this shadow".to_string(),
        SkipReason::LauncherOfThisShadow => {
            "skipped: a launcher whose record names this shadow".to_string()
        }
        SkipReason::Shadow {
            identity,
        } => format!("skipped: a shadow ({identity})"),
        SkipReason::Directory => "skipped: a directory, not a program".to_string(),
        SkipReason::NoExecuteBit {
            mode,
        } => format!("skipped: a file with no execute bit (mode {mode:04o})"),
        SkipReason::BrokenLink {
            target,
        } => format!(
            "skipped: a symbolic link that leads to no file (it points at {})",
            target.display()
        ),
        SkipReason::NotARegularFile => "skipped: not a regular file".to_string(),
    };
}

/// What:
/// The message for a lookup that found no real `mpm` at all:
/// every candidate it saw with the reason it was passed over,
/// and the remedy that fits what was found.
/// Why:
/// The message used to say that `mpm` is not installed, that every `mpm` is a shadow, or that an install moved it
/// aside,
/// and a file without its execute bit is none of the three:
/// naming the file and its mode is the only sentence that leads to `chmod`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function describeMissingRealMpm(lookup: Lookup): string
/// ```
pub fn describe_missing_real_mpm(lookup: &Lookup) -> String {
    let opening: String = if lookup.candidates.is_empty() {
        format!(
            "mpm-shadow: no real {SHADOWED_NAME} found on PATH: nothing named `{SHADOWED_NAME}` is in any directory on PATH"
        )
    } else {
        format!(
            "mpm-shadow: no real {SHADOWED_NAME} found on PATH: every `{SHADOWED_NAME}` the lookup saw was skipped, and the reason stands beside each"
        )
    };
    return format!(
        "{opening}\n{commands}\nmpm-shadow: what to do: {remedy}",
        commands = commands_on_path(lookup),
        remedy = missing_remedy(lookup),
    );
}

/// What:
/// The remedy for a lookup that found no real `mpm`,
/// built from what the candidates are.
/// Why:
/// A file that lost its execute bit is repaired by `chmod`,
/// a directory of that name by replacing or dropping it,
/// and a link that leads nowhere by pointing it somewhere;
/// only when none of those is there is installing the real `mpm` the whole answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingRemedy(lookup: Lookup): string
/// ```
fn missing_remedy(lookup: &Lookup) -> String {
    let install: String = format!(
        "install the real `{SHADOWED_NAME}` (this shadow starts it and never provides it), or put its directory on PATH after this shadow's"
    );
    let mut steps: Vec<String> = Vec::new();
    for candidate in &lookup.candidates {
        let path: String = candidate.path.display().to_string();
        match &candidate.skipped_because {
            Some(SkipReason::NoExecuteBit {
                ..
            }) => steps.push(format!("make `{path}` runnable (`chmod +x {path}`)")),
            Some(SkipReason::Directory) => steps.push(format!(
                "replace the directory `{path}` with the real `{SHADOWED_NAME}`, or take its directory off PATH"
            )),
            Some(SkipReason::BrokenLink {
                ..
            }) => steps.push(format!("point the link `{path}` at the real `{SHADOWED_NAME}`")),
            _ => {}
        }
    }
    if steps.is_empty() {
        return format!("{install}, then run the command again");
    }
    return format!(
        "{}, then run the command again. Installing the real `{SHADOWED_NAME}` (this shadow starts it and never provides it), or putting its directory on PATH after this shadow's, also ends this",
        steps.join("; ")
    );
}
