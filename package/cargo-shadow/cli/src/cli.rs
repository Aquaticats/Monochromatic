//! What:
//! The command-line grammar of the binary when it is started under its own name, `cargo-shadow` (or as the cargo
//! subcommand `cargo shadow`).
//! Why:
//! The shadow mode passes cargo's arguments through untouched, so clap owns only the catalog commands:
//! `install`, `update`, `verify`, plus `--help` and `--version`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cli.ts
//! program.name('cargo-shadow').version(pkg.version).command('install')...
//! ```

use clap::Parser;

/// What:
/// `#[derive(Parser)]` writes the argument parser from the attributes. The struct has no fields yet.
/// Why:
/// A shadow with no commands of its own still answers `--help` and `--version` by its own name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const program = new Command().name('cargo-shadow').version(version).description(about);
/// ```
#[derive(Parser)]
#[command(name = "cargo-shadow", version, about = "Shadows cargo.", long_about = None, arg_required_else_help = true)]
pub struct Cli {}

/// What:
/// `pub fn run_cli(arguments: Vec<String>) -> i32` parses the arguments and returns the exit code.
/// Why:
/// clap prints its own help and error text; the function only maps the outcome to the number the process exits with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runCli(args: string[]): number
/// ```
pub fn run_cli(arguments: Vec<String>) -> i32 {
    match Cli::try_parse_from(arguments) {
        Ok(_) => return 0,
        Err(error) => {
            let code: i32 = error.exit_code();
            let _ = error.print();
            return code;
        }
    }
}
