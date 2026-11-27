//! What:
//! The cause and the remedy of a start that did not happen,
//! chosen by which refusal it was:
//! a file another process is writing,
//! a permission,
//! a program or interpreter that is gone,
//! a file the kernel will not start,
//! a refusal no arm knows,
//! or this tool's own shadow standing where the real `mpm` should run.
//! Why:
//! Every sentence here is the whole of what a person has to act on,
//! and each arm changes for its own reason,
//! so they live together and away from the message that places them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function causeAndRemedy(failure: StartFailure): [string, string]
//! ```

/// Imports the error a refused start carries,
/// and the named kinds the arms are chosen by.
use std::io::{
    Error,
    ErrorKind,
};
/// Imports the borrowed path type,
/// because two arms read the program's own file.
use std::path::Path;

/// Imports the name the shadow answers to,
/// which the refused-shadow remedy counts.
use crate::constants::SHADOWED_NAME;
/// Imports the reader that says which interpreter a program names,
/// and its answers.
use crate::named_interpreter::{
    NamedInterpreter,
    named_interpreter,
};

/// Imports the failure the arms describe and its two kinds of refusal.
use super::{
    StartFailure,
    StartRefusal,
};

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

/// What:
/// The sentence naming the files a start of `program` needs before the program runs,
/// which is the program's own file and the interpreter its file names.
/// Why:
/// A compiled program has no first line,
/// so a sentence that says the interpreter is named there points the reader at a file the program does not
/// have;
/// the program's own bytes say which interpreter it names,
/// and the answer is read where it is written.
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
pub(super) fn cause_and_remedy(failure: &StartFailure) -> (String, String) {
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
/// The number is the only handle on this refusal,
/// for the same reason as `E2BIG`'s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ENAMETOOLONG = 36;
/// ```
#[cfg(target_os = "linux")]
const ENAMETOOLONG: i32 = 36;

/// What:
/// `const ELOOP: i32 = 40;` is the kernel's number for a path with too many levels of symbolic links,
/// which a
/// link leading back to itself makes (`asm-generic/errno.h`).
/// Why:
/// The number is the only handle on this refusal,
/// for the same reason as `E2BIG`'s.
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
/// `String`,
/// which owns its bytes and must be freed).
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
