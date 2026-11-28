//! What:
//!  The door over a diagnostic line's words:
//!  a debug record said only when `MONOCHROMATIC_VERBOSE` is exactly `true`,
//!  and a warning said unless `MONOCHROMATIC_WARN` is exactly `false`,
//!  each handed to the thread that writes it.
//! Why:
//!  The incumbent prints debug records only when `MONOCHROMATIC_VERBOSE=true` and warnings
//!      unless `MONOCHROMATIC_WARN=false` (`package/logger/core/src/sink/console.ts`),
//!  and its
//!      spec requires that diagnostics never corrupt a JSONL stream:
//!  each diagnostic is one
//!      complete line written in one call (`SPEC.md`,
//!  "Debug logging").
//!  The switches,
//!  the line's shape and the record of the lines this thread wrote are
//!      `the_words_of_a_diagnostic_line_and_the_lines_this_thread_wrote.rs`,
//!  which the queue's own notice about a stopped reader is written from as well,
//!  and the write is `diagnostic_lines_off_the_calling_thread.rs`,
//!  which says a line on a thread of its own so a reader that stopped draining never holds
//!      the command that said it.
//!  This file chooses the level and asks the one write;
//!  it holds no words of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const l = tagged({ tag: 'monochromatic-git-policy-cli' }); l.debug(message); l.warn(message);
//! ```

/// The one write a diagnostic line takes,
///  which the queue's own notice takes as well.
use crate::diagnostic_lines_off_the_calling_thread::write_line;
/// The current time in `toISOString` form.
use crate::iso_time::now_iso;
/// The record of the lines this thread wrote,
///  which a control reads through this file's name and no release path asks for.
#[cfg(test)]
pub(crate) use crate::the_words_of_a_diagnostic_line_and_the_lines_this_thread_wrote::take_lines_written;
/// The words one diagnostic line is made of,
///  which this file publishes under its own name:
///  a reader of the release reaches a line's shape and its two switches here,
///  as it did before the words moved below the queue.
pub use crate::the_words_of_a_diagnostic_line_and_the_lines_this_thread_wrote::{
    diagnostic_line,
    verbose_enabled,
    warnings_suppressed,
};

/// What:
///  Print a debug diagnostic when verbose diagnostics are on.
///       `tag` names the function or module boundary,
///  as the incumbent's tagged loggers do.
/// Why:
///   Decisions such as "owner alive,
///  skipping" are invisible otherwise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tagged({ tag }).debug(message);
/// ```
pub fn debug(
    tag: &str,
    message: &str,
) {
    if verbose_enabled() {
        write_line(diagnostic_line("debug", now_iso().as_str(), tag, message).as_str());
    }
}

/// What:
///  Print a warning unless warnings are suppressed.
/// Why:
///   Conditions the user should know about that do not fail the command,
///  such as a
///       landed-capture record that could not be written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tagged({ tag }).warn(message);
/// ```
pub fn warn(
    tag: &str,
    message: &str,
) {
    if !warnings_suppressed() {
        write_line(diagnostic_line("warn", now_iso().as_str(), tag, message).as_str());
    }
}

/// The door's own words stay out of the release executable.
#[cfg(test)]
#[path = "diagnostic_log_tests.rs"]
mod tests;
