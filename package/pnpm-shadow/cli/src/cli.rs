//! What:
//! The command-line grammar of the binary when it is started under its own name,
//! `pnpm-shadow`.
//! Why:
//! The shadow mode passes pnpm's arguments through untouched,
//! so clap owns only this:
//! `--help` and `--version`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cli.ts
//! program.name('pnpm-shadow').version(pkg.version).description('...');
//! ```

/// What:
/// `use clap::Parser;` imports the trait that gives a clap-derived struct its `try_parse_from` function.
/// Why:
/// The struct below declares the grammar;
/// this trait turns text into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command } from 'commander';
/// ```
use clap::Parser;

/// What:
/// `#[derive(Parser)]` is a code generator attached to the struct:
/// it writes the argument parser from the
/// attributes.
/// `#[command(...)]` sets the program's name,
/// takes its version from `Cargo.toml`,
/// and shows help when no
/// argument is given.
/// The struct has no fields because the program takes no other arguments.
/// Why:
/// A shadow with no commands of its own still answers `--help` and `--version` by its own name,
/// so a person can tell
/// what it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const program = new Command().name('pnpm-shadow').version(version).description(about);
/// ```
#[derive(Parser)]
#[command(
    name = "pnpm-shadow",
    version,
    about = "Shadows pnpm. Started under the name `pnpm` it warns about package.json dependency versions that are not catalog: or workspace:*, then runs the real pnpm unchanged and never blocks it. Started under its own name it only answers --help and --version.",
    // clap also turns the struct's doc comment into help text, and its long form would win over `about` in `--help`;
    // `long_about = None` keeps the comment above out of the help text.
    long_about = None,
    arg_required_else_help = true
)]
pub struct Cli {}

/// What:
/// `pub fn run_cli(arguments: Vec<String>) -> i32` parses the arguments and returns the exit code.
/// Why:
/// clap prints its own help and error text;
/// the function only maps the outcome to the number the process exits with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runCli(args: string[]): number { try { program.parse(args); return 0; } catch (e) { return e.exitCode; } }
/// ```
pub fn run_cli(arguments: Vec<String>) -> i32 {
    // `Cli::try_parse_from` returns `Ok(Cli)` or `Err(error)`; `--help` and `--version` come back as an `Err` whose
    // exit code is 0.
    match Cli::try_parse_from(arguments) {
        Ok(_) => return 0,
        Err(error) => {
            let code: i32 = error.exit_code();
            // `let _ =` discards the print result: a closed pipe must not turn into a panic.
            let _ = error.print();
            return code;
        }
    }
}
