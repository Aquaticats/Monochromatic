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
use std::io::Error;
/// Imports the borrowed path type.
use std::path::Path;

/// The exit status of a start that failed before the real `mpm` ran,
/// which the message tells the reader to expect.
use crate::constants::{
    EXEC_FAILED_EXIT_CODE,
    SHADOWED_NAME,
};
/// The deepest nesting of shadow starts allowed,
/// named in the message about a nested start.
use crate::nesting::MAX_NESTING;
/// Every `mpm` the PATH lookup saw,
/// which a refusal of a shadow has to name.
use crate::real_binary::Lookup;
/// What the retry of a refused start did,
/// which the message reports.
use crate::start_retry::RetryReport;

/// What:
/// The cause and the remedy of a start that did not happen,
/// one arm per kind of refusal,
/// and the two functions that choose them.
/// Why:
/// The arms are the words of the message,
/// and they change for their own reasons (a new refusal,
/// a new step that works);
/// keeping them beside the assembly would make one file of prose and structure.
mod cause_and_remedy;

/// What:
/// The `mpm` commands the PATH lookup read,
/// with why it took or passed over each,
/// and the message for a PATH that holds no real `mpm`.
/// Why:
/// Both messages that name those commands print the same list,
/// and that list is the shadow's own answer to a command a shell may not have.
mod path_commands;

/// What:
/// `pub use self::cause_and_remedy::{...}` hands the arms' two public sentences out under the name callers already use.
/// Why:
/// The launcher mode reports the being-written refusal with them,
/// and it names them through this module.
pub use self::cause_and_remedy::{
    being_written_cause_and_remedy,
    being_written_cause_and_remedy_for,
};
/// What:
/// `pub use self::path_commands::describe_missing_real_mpm;` hands out the message for a PATH with no real `mpm`.
/// Why:
/// `shim.rs` prints it,
/// and it names this module for the words of every message about a start.
pub use self::path_commands::describe_missing_real_mpm;

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
    /// the list itself is printed,
    /// so no sentence has to send the reader to a command a shell may reject.
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
    let (cause, remedy) = self::cause_and_remedy::cause_and_remedy(failure);
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
        lines.push(self::path_commands::commands_on_path(failure.lookup));
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
        self::path_commands::commands_on_path(lookup),
        format!(
            "  what to do: put the real `mpm` on PATH after the shadow, or remove the `mpm` from the `{SHADOWED_NAME}` commands this run found on PATH that is a shadow, then run the command again."
        ),
        format!(
            "  exit status: mpm-shadow exits with {EXEC_FAILED_EXIT_CODE}; nothing of `{command}` ran in this process, and the post-run cleanup did not run."
        ),
    ];
    return lines.join("\n");
}
