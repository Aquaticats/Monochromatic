//! What:
//! The `mpm-shadow` command itself (the non-shadow mode):
//! `--help`,
//! `--version` and the `prune` subcommand,
//!           with the grammar owned by `clap`.
//! Why:
//! A person can run the prune by hand,
//! for example to clear copies that were left before the shadow was
//!           installed.
//! The builder API is used instead of derive macros so every line is an ordinary method call.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cli.ts, with a library such as commander:
//! program.name('mpm-shadow').command('prune');
//! ```

/// What:
/// `use clap::Command;` imports the type that describes a command line,
/// from the `clap` crate (a third-party
///           library,
/// like an npm package).
/// Why:
/// We describe the grammar with it below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command } from 'commander';
/// ```
use clap::Command;

/// Imports the process-environment module (CARGO_HOME,
/// HOME).
use std::env;

/// Imports `cargo_home` from `crate::cargo_home`.
use crate::cargo_home::cargo_home;
/// Imports `USAGE_EXIT_CODE` from `crate::constants`.
use crate::constants::USAGE_EXIT_CODE;
/// Imports `prune_stale_own_crates` from `crate::prune`.
use crate::prune::prune_stale_own_crates;
/// Imports the toolchain verbs' grammar and dispatch.
use crate::toolchain::commands::{dispatch, subcommands as toolchain_subcommands};

/// What:
/// `pub fn build_command() -> Command` declares a public function returning the grammar description.
/// Why:
/// Both the real run and the tests need the grammar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function buildCommand(): Command
/// ```
pub fn build_command() -> Command {
    // What:     `Command::new("prune").about(...)` starts a subcommand description; each method call returns the
    //           updated description (method chaining, like a TS builder).
    // Why:      `--help` for the subcommand comes from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const prune = new Command('prune').description('...');
    // ```
    let prune_command: Command = Command::new("prune")
        .about("Delete the superseded cached versions of this repository's own crates from cargo's registry");

    // What:     `env!("CARGO_PKG_VERSION")` is a compile-time macro (the `!`) that pastes in the version from
    //           `Cargo.toml`. `subcommand_required(true)` makes a bare `mpm-shadow` print the help instead.
    // Why:      `--help` and `--version` come for free from this description.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return new Command('mpm-shadow').version(pkg.version).addCommand(prune);
    // ```
    return Command::new("mpm-shadow")
        .about("Owns the toolchain plan and lock (lock, sync, status, update, bin-paths) and shadows mpm: invoked as mpm it runs the real mpm, then deletes the superseded cached versions of this repository's own crates.")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(prune_command)
        .subcommands(toolchain_subcommands());
}

/// What:
/// `pub fn run_cli(arguments: Vec<String>) -> i32` declares a public function taking the command-line texts
///           and returning the exit code.
/// Why:
/// `main.rs` calls it for every start that is not under the name `mpm`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runCli(args: string[]): number
/// ```
pub fn run_cli(arguments: Vec<String>) -> i32 {
    // `get_matches_from` parses the arguments; on `--help`, `--version` or a bad command line, clap prints and exits.
    let matches = build_command().get_matches_from(arguments);

    // What:     `if let Some(("prune", _)) = matches.subcommand()` runs the block only when the user typed `prune`;
    //           `subcommand()` returns an `Option` of a (name, matches) pair, the pattern requires the name `"prune"`
    //           and ignores the second half with `_`.
    // Why:      `prune` is the only subcommand.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (matches.subcommand?.name === 'prune') { ... }
    // ```
    if let Some(("prune", _)) = matches.subcommand() {
        return prune_now();
    }
    // Every other verb is a toolchain verb; `dispatch` answers `None` for a name it does not own.
    if let Some((name, verb_matches)) = matches.subcommand()
        && let Some(exit_code) = dispatch(name, verb_matches)
    {
        return exit_code;
    }
    return USAGE_EXIT_CODE;
}

/// What:
/// `fn prune_now() -> i32` declares a private function returning the exit code.
/// Why:
/// It holds the `prune` subcommand's work,
/// so `run_cli` stays flat.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pruneNow(): number
/// ```
fn prune_now() -> i32 {
    let Some(directory) = cargo_home(env::var("CARGO_HOME").ok(), env::var("HOME").ok()) else {
        eprintln!("mpm-shadow: neither CARGO_HOME nor HOME is set, so cargo's home cannot be found");
        return 1;
    };
    let deleted = prune_stale_own_crates(&directory);
    for path in &deleted {
        println!("deleted stale {}", path.display());
    }
    if deleted.is_empty() {
        println!("nothing stale under {}", directory.display());
    }
    return 0;
}
