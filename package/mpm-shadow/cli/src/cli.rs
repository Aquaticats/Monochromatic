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
use clap::{Arg, ArgAction, Command};

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
use crate::toolchain::commands::{dispatch, subcommands as toolchain_subcommands, Verb, VERBS as TOOLCHAIN_VERBS};
/// Imports the environment view and the toolchain roots.
use crate::toolchain::scope::{EnvMap, toolchain_roots};
/// Imports the owned path type.
use std::path::PathBuf;
/// Imports the toolchain prune command.
use crate::toolchain::prune_cargo::run_prune_command;

/// What:
/// The root verb `prune`:
/// the only top-level verb whose arguments this module owns.
/// Why:
/// The grammar,
/// `--help` and the help test read the same declaration,
/// so the verb name and its visible aliases have one owner.
pub const PRUNE: Verb = Verb {
    name: "prune",
    aliases: &[],
    build: prune_command,
};

/// Describes the `prune` subcommand:
/// its options and its help text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pruneCommand(): Command
/// ```
fn prune_command(verb: &Verb) -> Command {
    return verb
        .command()
        .about("Delete the superseded cached versions of this repository's own crates from cargo's registry")
        .arg(
            Arg::new("toolchain")
                .long("toolchain")
                .action(ArgAction::SetTrue)
                .help(
                    "Instead, clean the toolchain root: delete the cargo tool versions no known lock names, \
                     and the release-archive versions of this repository's tools that are older than the newest finished one",
                ),
        )
        .arg(
            Arg::new("global")
                .long("global")
                .action(ArgAction::SetTrue)
                .requires("toolchain")
                .help("With --toolchain: prune the machine-global toolchain root"),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .action(ArgAction::SetTrue)
                .requires("toolchain")
                .help("With --toolchain: list what would be deleted and delete nothing"),
        )
        .arg(
            Arg::new("lock")
                .long("lock")
                .action(ArgAction::Append)
                .requires("toolchain")
                .help("With --toolchain: another toolchain.lock.json whose versions must be kept (repeatable), for example a clone git does not list"),
        );
}

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
    // What:     `Command::new("mpm-shadow")` starts the root description; each method call returns the
    //           updated description (method chaining, like a TS builder).
    // Why:      `--help` and `--version` come for free from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return new Command('mpm-shadow').version(pkg.version).addCommand(prune);
    // ```
    return Command::new("mpm-shadow")
        .about("A shadow of mpm that owns the toolchain plan and lock with pnpm's commands (install, update, outdated, list, bin): invoked as `mpm`, a command line whose first word is one of these verbs or their pnpm aliases (`mpm i -g`) runs that command here, and every other command line runs the real mpm, after which the superseded cached versions of this repository's own crates are deleted. Invoked as mpm-shadow it runs the same commands.")
        // `env!("CARGO_PKG_VERSION")` is a compile-time macro (the `!`) that pastes in the version from
        // `Cargo.toml`; `subcommand_required(true)` makes a bare `mpm-shadow` print the help instead.
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        // Both lines build the surface from declarations, so the grammar cannot hold a verb the
        // help test does not read.
        .subcommand((PRUNE.build)(&PRUNE))
        .subcommands(toolchain_subcommands());
}

/// What:
/// Every top-level subcommand `--help` lists,
/// in render order:
/// `prune` followed by the toolchain verbs.
/// Why:
/// `build_command` and the help test read this one surface,
/// so a verb registered in the grammar is always asserted against the rendered help.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function declaredVerbs(): Verb[]
/// ```
pub fn declared_verbs() -> Vec<&'static Verb> {
    // The root verb comes first in the rendered help, then the toolchain verbs.
    let mut verbs: Vec<&'static Verb> = vec![&PRUNE];
    verbs.extend(TOOLCHAIN_VERBS);
    return verbs;
}

/// What:
/// `pub fn names_a_declared_verb(word: &str) -> bool` declares a public function that answers whether `word`
///           (a borrowed text,
/// `&str`) is the name or an alias of one of the verbs `declared_verbs` lists.
/// Why:
/// `main.rs` runs a command line typed to `mpm` as this tool's own command when its first word is one of these
///           pnpm verbs (owner,
/// 2026-11-03:
/// "I want pnpm style verbs.
/// We talked about this!"),
/// and the verb
///           table stays the one owner of those words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function namesADeclaredVerb(word: string): boolean {
///   return declaredVerbs().some((verb) => verb.name === word || verb.aliases.includes(word));
/// }
/// ```
pub fn names_a_declared_verb(word: &str) -> bool {
    for verb in declared_verbs() {
        // What:     `verb.aliases.contains(&word)` asks whether the alias list holds `word`; `&word` lends the
        //           text to the search without giving it away.
        // Why:      An alias such as `i` is as much pnpm's verb as the name `install`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (verb.name === word || verb.aliases.includes(word)) return true;
        // ```
        if verb.name == word || verb.aliases.contains(&word) {
            return true;
        }
    }
    return false;
}

/// What:
/// `pub fn run_cli(arguments: Vec<String>) -> i32` declares a public function taking the command-line texts
///           and returning the exit code.
/// Why:
/// `main.rs` calls it for every start that is not under the name `mpm`,
/// and for a start under `mpm` whose first word is one of pnpm's verbs (`names_a_declared_verb`).
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
    if let Some(("prune", prune_matches)) = matches.subcommand() {
        if prune_matches.get_flag("toolchain") {
            return run_prune_command(prune_matches);
        }
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
    let roots: Vec<PathBuf> = toolchain_roots(&EnvMap::from_process());
    let deleted = prune_stale_own_crates(&directory, &roots);
    for path in &deleted {
        println!("deleted stale {}", path.display());
    }
    if deleted.is_empty() {
        println!("nothing stale under {}", directory.display());
    }
    return 0;
}
