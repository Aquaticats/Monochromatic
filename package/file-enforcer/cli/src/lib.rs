//! What:
//!  The `file-enforcer` command line as a library:
//! parse arguments,
//! find the config,
//! hand the request to the engine.
//! Why:
//!  `main.rs` stays three lines,
//! and the binary's behavior is testable by calling `run_command_line`.
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

/// The `rust-catalog` rule kind,
/// over the dependency-catalog library.
pub mod kind_rust_catalog;

/// The `browserslist` rule kind,
/// over the in-repo browserslist crate.
pub mod kind_browserslist;

/// Product directory discovery for the JetBrains rule.
pub mod jetbrains_discovery;

/// The JSON half of the Harper policy.
pub mod harper_json;

/// The XML half of the Harper policy.
pub mod harper_xml;

/// Entry edits for the Harper policy.
pub mod harper_xml_edit;

/// The `jetbrains-harper` rule kind,
/// over the in-repo XML edit crate.
pub mod kind_jetbrains_harper;

/// The registry the command line and its server use.
pub mod kinds;

/// Scratch trees and run helpers for the rule-kind tests.
#[cfg(test)]
mod test_support;

/// Synthetic JetBrains settings files for the Harper tests.
#[cfg(test)]
mod harper_fixtures;

/// Imports `Cli` from `args`.
use args::Cli;
/// Imports `Parser` from `clap`.
use clap::Parser;
/// Imports `EventSink` from `monochromatic_file_enforcer::event`.
use monochromatic_file_enforcer::event::EventSink;
/// Imports `LinePrinter`,
/// `event_for_clap_error` from `output`.
use output::{LinePrinter, event_for_clap_error};
/// Imports `OsString` from `std::ffi`.
use std::ffi::OsString;

/// What:
///  Parse the arguments and run the command;
/// return the exit code.
///  `&[OsString]` is a borrowed list of operating-system strings (TS:
/// `string[]`,
/// but not necessarily UTF-8).
/// Why:
///  Help,
/// version and parse errors are printed as JSON lines here,
/// before any command runs.
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
