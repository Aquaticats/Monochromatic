//! What:     The `search-shadow` command itself (the non-shadow mode): `--help`, `--version` and the `link` subcommand,
//!           with the grammar owned by `clap`.
//! Why:      Users need a way to create the shadow links and to read what the tool does. The builder API is used
//!           instead of derive macros so every line is an ordinary method call.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cli.ts, with a library such as commander:
//! program.name('search-shadow').command('link').argument('<directory>');
//! ```

/// What:     `use clap::{Arg, Command};` imports two types from the `clap` crate (a third-party library, like an npm
///           package): `Command` describes a command line and `Arg` describes one argument.
/// Why:      We describe the grammar with them below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Argument, Command } from 'commander';
/// ```
use clap::{Arg, Command};

/// Imports the process-environment module (to find this executable).
use std::env;

/// Imports `Path` (a borrowed read-only path) and `PathBuf` (the owned one).
use std::path::{Path, PathBuf};

use crate::constants::{SHADOWED_NAMES, USAGE_EXIT_CODE};
use crate::link::link_shadows;

/// What:     `pub fn build_command() -> Command` declares a public function returning a `Command` description.
/// Why:      Both the real run and the tests need the grammar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function buildCommand(): Command { ... }
/// ```
pub fn build_command() -> Command {
    // What:     `Command::new("search-shadow")` starts the description; each `.about(...)`, `.subcommand(...)` call
    //           returns the updated description (method chaining, like a TS builder). `env!("CARGO_PKG_VERSION")` is a
    //           compile-time macro (the `!`) that pastes in the version from `Cargo.toml`.
    // Why:      `--help` and `--version` come for free from this description.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return new Command('search-shadow').description('...').version(pkg.version).addCommand(link);
    // ```
    let shadowed: String = SHADOWED_NAMES.join(", ");

    // What:     `Box::leak(text.into_boxed_str())` gives the text up to the program for good and returns a `&'static str`.
    // Why:      clap's `version` wants text that lives for the whole program (an owned `String` needs clap's `string`
    //           feature, which this crate does not enable). The command is built once per process, so the leak is one
    //           short string.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const versionText = `${pkg.version}, a shadow of ${shadowed}`;
    // ```
    let version_text: &'static str = Box::leak(format!("{}, a shadow of {shadowed}", env!("CARGO_PKG_VERSION")).into_boxed_str());
    let link_command: Command = Command::new("link")
        .about("Create a shadow link for every shadowed tool whose real binary is on PATH in DIRECTORY; put DIRECTORY first on PATH to activate them")
        .arg(
            Arg::new("directory")
                .required(true)
                .help("Directory that will hold the links, for example .cache/search-shadow-bin"),
        );

    return Command::new("search-shadow")
        .about(format!(
            "search-shadow is a shadow: it stands in for other tools, it is not a tool of its own. It shadows {shadowed}. Started under one of those names it runs the real tool at the lowest best-effort disk priority, refuses wide rg, grep and find walks, and prints one extra line after the tool's --version and one extra section after its --help. Started as search-shadow it offers the link command."
        ))
        .version(version_text)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(link_command);
}

/// What:     `pub fn run_cli(arguments: Vec<String>) -> i32` declares a public function that takes the owned list of
///           command-line texts and returns the exit code.
/// Why:      `main.rs` calls it for every invocation that is not under a shadowed name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runCli(args: string[]): number { ... }
/// ```
pub fn run_cli(arguments: Vec<String>) -> i32 {
    // `get_matches_from` parses the arguments; on `--help`, `--version` or a bad command line, clap prints and exits.
    let matches = build_command().get_matches_from(arguments);

    // What:     `if let Some(("link", link_matches)) = matches.subcommand()` runs the block only when the user typed the
    //           `link` subcommand; `subcommand()` returns an `Option` of a (name, matches) pair, and the pattern
    //           requires the name to be exactly `"link"`.
    // Why:      `link` is the only subcommand.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (matches.subcommand?.name === 'link') { ... }
    // ```
    //           (The second condition after `&&` reads the argument as a `String`; `::<String>` names the type to read.)
    if let Some(("link", link_matches)) = matches.subcommand()
        && let Some(directory) = link_matches.get_one::<String>("directory")
    {
        return link_into(directory);
    }
    return USAGE_EXIT_CODE;
}

/// What:     `fn link_into(directory: &str) -> i32` declares a private function taking a borrowed text and returning
///           the exit code.
/// Why:      It holds the `link` subcommand's work, so `run_cli` stays flat.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function linkInto(directory: string): number { ... }
/// ```
fn link_into(directory: &str) -> i32 {
    // What:     `env::current_exe()` returns the path of the running binary as a `Result`; `.is_err()` asks whether it
    //           failed and `.unwrap_or_default()` takes the path out (an empty path only in the failure case, which
    //           the check above already returned from).
    // Why:      The links must point at this very binary.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const executable = process.execPath;
    // ```
    let executable_result = env::current_exe();
    if executable_result.is_err() {
        eprintln!("search-shadow: could not find its own executable path");
        return 1;
    }
    let executable: PathBuf = executable_result.unwrap_or_default();

    // What:     `match result { Ok(created) => ..., Err(error) => ... }` picks the branch for the success or the failure
    //           case of a `Result` and names the value inside each. `println!` prints a line to standard output (a
    //           macro, hence `!`).
    // Why:      Print each created link on success, or explain the failure and stop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { const created = linkShadows(...); for (const link of created) console.log(...); return 0; }
    // catch (error) { console.error(...); return 1; }
    // ```
    // `env::var("PATH").unwrap_or_default()` reads PATH, or an empty text when it is unset (then nothing is found, so nothing is linked).
    let path_variable: String = env::var("PATH").unwrap_or_default();
    match link_shadows(Path::new(directory), &executable, &path_variable) {
        Ok(report) => {
            for link in &report.linked {
                println!("linked {} -> {}", link.display(), executable.display());
            }
            for link in &report.removed {
                println!("removed stale {} (its real tool is gone)", link.display());
            }
            if !report.skipped.is_empty() {
                println!("skipped, no real tool on PATH: {}", report.skipped.join(", "));
            }
            return 0;
        }
        Err(error) => {
            eprintln!("search-shadow: could not link into {directory}: {error}");
            return 1;
        }
    }
}
