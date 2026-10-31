//! What:
//!  The `file-enforcer` command line as a library: parse arguments, find the config, hand the request to the engine.
//! Why:
//!  `main.rs` stays three lines, and the binary's behavior is testable by calling `run_command_line`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export async function runCommandLine(args: string[]): Promise<number>
//! ```

/// The clap grammar.
pub mod args;

/// JSON-lines output.
pub mod output;

/// Config discovery.
pub mod discover;

/// Command dispatch.
pub mod commands;

use args::Cli;
use clap::Parser;
use monochromatic_file_enforcer::event::EventSink;
use output::{LinePrinter, event_for_clap_error};
use std::ffi::OsString;

/// What:
///  Parse the arguments and run the command; return the exit code.
///  `&[OsString]` is a borrowed list of operating-system strings (TS: `string[]`, but not necessarily UTF-8).
/// Why:
///  Help, version and parse errors are printed as JSON lines here, before any command runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function runCommandLine(args: string[]): Promise<number>
/// ```
pub fn run_command_line(arguments: &[OsString]) -> i32 {
    let mut printer = LinePrinter::default();
    let cli = match Cli::try_parse_from(arguments) {
        Ok(cli) => cli,
        Err(error) => {
            let (event, code) = event_for_clap_error(&error);
            printer.emit(event);
            return code;
        }
    };
    return commands::dispatch(&cli, &mut printer);
}
