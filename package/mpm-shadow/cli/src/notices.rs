//! What:
//! The one place this program writes a line about its own work to standard error:
//! the line a long wait prints,
//! the warning about a lock older than the tool that runs,
//! the lines naming what a run left in a folder,
//! and the cleanup summary after an mpm command.
//! Why:
//! A person reading a terminal or a log needs those lines,
//! and the work they describe either succeeded already or is still going:
//! none of them is the command's result,
//! so a stream that refuses the bytes must not end,
//! fail or change the command.
//! `eprintln!` does not answer that need:
//! it panics on every write error but a closed descriptor,
//! and this crate's release build sets `panic = "abort"`,
//! so a full disk or a pipe whose reader has gone turns one courtesy line into `SIGABRT`
//! (`0.2.7-alpha.20` did exactly that for the wait line;
//! `doc/second-pass-reads-of-the-alpha-releases-what-each-confirmed.md` holds the finding).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // A stderr write that ignores a broken stream.
//! export function say(line: string): void {
//!   try {
//!     process.stderr.write(`${line}\n`);
//!   } catch {
//!     // The line is a courtesy; a stream that refuses it changes nothing about the command.
//!   }
//! }
//! ```

/// Imports `Write` from `std::io`,
/// the trait that gives a stream its `write_fmt` method,
/// which the `writeln!` macro below calls.
/// Why:
/// `writeln!` needs the trait in scope to compile,
/// and the trait's presence is what lets this module write without `println!`'s panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No import: `process.stderr.write` is a method on the global object.
/// ```
use std::io::Write;

/// What:
/// `pub fn say(line: &str)` writes `line` and one newline to standard error
/// and drops whatever that write answered.
/// Why:
/// Every notice this program says about its own work goes through one writer,
/// so the rule "a stream that refuses the line does not change the command" is stated once,
/// and a caller cannot accidentally say a notice with a write that aborts the process.
/// The answer is dropped rather than reported because there is nowhere left to report it:
/// the only channel it would go to is the one that just refused the bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function say(line: string): void
/// ```
pub fn say(line: &str) {
    // What: `std::io::stderr()` borrows the process's standard error stream
    //  (the sibling `std::io::stdout()` is the other one).
    // Why: the notices belong on standard error, beside a command's failures,
    //  while the command's report stays on standard output for whatever parses it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const standardError = process.stderr;
    // ```
    let mut standard_error = std::io::stderr();
    // What: `writeln!` writes the line and a newline into that stream and answers an `io::Result<()>`,
    //  and `let _ =` drops the answer without looking at it.
    // Why: `eprintln!` does the same write and then panics on the answer,
    //  which this build cannot unwind (`panic = "abort"`), so a discarding write is what keeps a broken
    //  stream from ending the command.
    // Gotcha: `let _ =` discards silently, which is exactly the intent here and nowhere else in this crate.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try {
    //   standardError.write(`${line}\n`);
    // } catch {
    //   // Dropped: the stream is the only place the failure could be reported.
    // }
    // ```
    let _ = writeln!(standard_error, "{line}");
}
