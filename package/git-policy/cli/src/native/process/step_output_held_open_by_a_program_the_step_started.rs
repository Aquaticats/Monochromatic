//! What:
//!  What a person is told when a program the step started still holds one of its streams open after the step ended,
//!  and the error that carries those words and the bytes read before the drain ended.
//! Why:
//!  The read of a step's output forgives one loss and no other:
//!  a stream whose last writer is a program the step started,
//!  and only a sentence naming that program turns a lost tail into something a person can act on.
//!  `rust/max-lines` splits the read of a step's output by role,
//!  and the words are the role a caller and a person meet.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! throw new OutputHeldOpen({ waitingFor, holders, read });
//! isOutputHeldOpen(error);
//! ```

use crate::process_use::ProcessUser;
/// A name another program chose,
/// shown so no byte of it can act on the line it lands in.
use crate::shown_so_it_cannot_rewrite_the_line::shown_so_it_cannot_rewrite_the_line;
use std::time::Duration;

/// How long the read of a step's output may go on after the step itself has ended.
///  A pipe holds at most 64 KiB (`pipe-max-size`'s default),
/// which this process reads in one go,
/// so a second is thousands of times what draining a pipe nothing more is written to costs.
///  A program the step started that still holds the write end a second after the step ended is holding it,
///  not writing the step's output,
/// and the wait for the step must not last as long as that program lives.
/// Labelled unmeasured as a quiet window is (`SPEC.md`,
/// "The quiet windows are labelled unmeasured"):
/// no log holds how long a surviving writer takes to close a step's streams.
pub const OUTPUT_DRAIN_AFTER_EXIT: Duration = Duration::from_secs(1);

/// What:
///  The sentence for a step whose output a program it started still held open after the step ended.
/// Why:
///  The step is over and its output is what was read;
/// the person has to know that the step is not what holds the command,
///  and which program does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} ended, but a program it started still holds its output open (${holders}); ...`
/// ```
fn output_held_open_text(
    waiting_for: &str,
    holders: &[ProcessUser],
) -> String {
    let named: String = holder_named(holders);
    return format!(
        "{waiting_for} ended, but {named} still holds the step's output open, so the output this command read may be incomplete and \
         the wait for it ended after {} seconds of it. Check with `ps` what that program is and whether it is still needed, end it if \
         it is stuck, and run the command again; the step's own end is what the repository state shows.",
        OUTPUT_DRAIN_AFTER_EXIT.as_secs()
    );
}

/// What:
///  The sentence for a step that ended with its standard error cut short:
/// its exit status and its standard output were read whole,
///  and a program it started still holds its standard error open.
/// Why:
///  What the step printed to standard output is its answer,
/// so the answer is returned and the person is told that the standard error
///  read here stops where the program still holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${waitingFor} ended and its exit status and standard output were read whole, but ${named} still holds its standard error open ...`
/// ```
pub(super) fn error_cut_short_text(
    waiting_for: &str,
    holders: &[ProcessUser],
) -> String {
    let named: String = holder_named(holders);
    return format!(
        "{waiting_for} ended and its exit status and standard output were read whole, but {named} still holds its standard error open, so \
         the standard error read here stops {} seconds after the step ended. Check with `ps` what that program is and whether it is \
         still needed; the step's own end is what the repository state shows.",
        OUTPUT_DRAIN_AFTER_EXIT.as_secs()
    );
}

/// What:
///  The program that still holds a stream of the step,
/// named as the person knows it,
///  with its own name shown so that no byte of it can rewrite the sentence.
/// Why:
///  Both sentences about a held-open stream name the same program,
/// and a number alone tells the person nothing.
///  A program sets its own name to any bytes it likes (`/proc/<pid>/comm`),
///  and this is a child the step started,
///  so the name is that program's text and never this one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holders.length === 0 ? 'a program it started, which no longer shows in the process table' : `process ${holders[0].pid} (${shownSoItCannotRewriteTheLine(holders[0].command)})`;
/// ```
fn holder_named(holders: &[ProcessUser]) -> String {
    return match holders.first() {
        Some(holder) => match holder.command.as_ref() {
            // What: the program's own name, shown so that no byte of it can rewrite this sentence.
            // Why:  a line break or a terminal escape inside the name would change what the person reads about
            //  which program holds the step's stream.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return `process ${holder.pid} (${shownSoItCannotRewriteTheLine(command)})`;
            // ```
            Some(command) => {
                return format!(
                    "process {} ({})",
                    holder.pid,
                    shown_so_it_cannot_rewrite_the_line(command)
                );
            }
            None => return format!("process {}", holder.pid),
        },
        None => String::from("a program it started, which no longer shows in the process table"),
    };
}

/// What:
///  The error of a read whose stream a program the step started still held open,
/// with the bytes read before the drain ended and the programs that held it.
/// Why:
///   The caller decides what the loss costs:
/// a step whose standard output was cut short has no answer,
///  while one whose standard error was cut short still has its exit status and its standard output,
/// so the bytes
///  taken before the drain ended have to reach that decision.
#[derive(Debug)]
pub(super) struct OutputHeldOpen {
    /// The sentence that names the step,
    /// the program and what to do.
    pub(super) waiting_for: String,
    /// The programs that held the stream open when the drain ended.
    pub(super) holders: Vec<ProcessUser>,
    /// The bytes read of the stream before the drain ended.
    pub(super) read: Vec<u8>,
}

impl std::fmt::Display for OutputHeldOpen {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return formatter.write_str(
            output_held_open_text(self.waiting_for.as_str(), self.holders.as_slice()).as_str(),
        );
    }
}

impl std::error::Error for OutputHeldOpen {}

/// Whether `error` is the error of a step whose output a program it started still held open.
pub fn is_output_held_open(error: &std::io::Error) -> bool {
    return error.get_ref().is_some_and(is_the_held_open_error);
}

/// Whether `inner` is this module's held-open error,
/// for `is_output_held_open`.
fn is_the_held_open_error(inner: &(dyn std::error::Error + Send + Sync + 'static)) -> bool {
    return inner.is::<OutputHeldOpen>();
}

/// Controls of these sentences stay out of the release executable.
#[cfg(test)]
#[path = "step_output_held_open_by_a_program_the_step_started_tests.rs"]
mod tests;
