//! What:
//! The command-line surface of the toolchain half:
//! the `clap` description of `install` (alias `i`),
//! `update`,
//! `outdated`,
//! `list` (alias `ls`),
//! `bin`
//!           and the other verbs,
//! named after the pnpm commands they mirror,
//! and the function that runs one of them and turns its result into an exit code.
//! Why:
//! `cli.rs` keeps the older `prune` command and delegates every toolchain verb here,
//! so the toolchain code
//!           and its grammar live together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commands.ts
//! export function subcommands(): Command[]
//! export function dispatch(name: string, matches: ParsedArgs): number | undefined
//! ```

/// Imports the `clap` types for reading a parsed command line and starting a command.
use clap::{
    ArgMatches,
    Command,
};

/// Imports what this module uses from `super::scope`.
use super::scope::{
    EnvMap,
    Scope,
    resolve_scope,
};

/// Re-exports the verb declarations from `super::verb_grammar`,
/// so `cli.rs` and the tests keep reading the grammar through this module.
pub use super::verb_grammar::{
    VERBS,
    Verb,
};

/// The `bin` verb.
mod the_bin_verb;
/// The `install` verb.
mod the_install_verb;
/// The `list` verb.
mod the_list_verb;
/// The `update` and `outdated` verbs.
mod the_update_and_outdated_verbs;
/// The `verify-archive` verb.
mod the_verify_archive_verb;
/// The words every verb prints,
/// and the directory the root search starts in.
mod what_every_verb_prints;

/// Imports the `bin` verb.
use self::the_bin_verb::bin_verb;
/// Imports the `install` verb.
use self::the_install_verb::install_verb;
/// Imports the `list` verb.
use self::the_list_verb::list_verb;
/// Imports the `update` and `outdated` verbs.
use self::the_update_and_outdated_verbs::update_verbs;
/// Imports the `verify-archive` verb.
use self::the_verify_archive_verb::verify_archive_verb;
/// Re-exports the warning lines a command prints on standard error.
pub use self::what_every_verb_prints::older_lock_warning_lines;
/// Imports the one error printer every verb answers with,
/// and the starting directory.
use self::what_every_verb_prints::{
    report,
    start_directory,
};

/// What:
/// The toolchain subcommands for `mpm-shadow --help`,
/// built from the `VERBS` declaration.
/// Why:
/// Each verb is a plain `clap` command;
/// the grammar is data the tests can inspect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function subcommands(): Command[]
/// ```
pub fn subcommands() -> Vec<Command> {
    let mut commands: Vec<Command> = Vec::new();
    for verb in VERBS {
        // `verb.build` is the declared argument builder; calling it through the declaration
        // keeps the parsed name and the argument grammar in one place.
        commands.push((verb.build)(verb));
    }
    return commands;
}

/// What:
/// Runs the toolchain verb `name` with the arguments `matches`,
/// or returns `None` when `name` is not one of
///           this module's verbs.
/// Why:
/// `cli.rs` asks this first and falls back to its own `prune`.
/// The scope is resolved before the verb is chosen,
/// so a tree whose plan cannot be found is reported the same way for every verb,
/// including a name this module does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function dispatch(name: string, matches: ParsedArgs): number | undefined
/// ```
pub fn dispatch(
    name: &str,
    matches: &ArgMatches,
) -> Option<i32> {
    let env: EnvMap = EnvMap::from_process();
    let global: bool = matches.get_flag("global");
    let scope: Scope = match resolve_scope(global, &start_directory(), &env) {
        Ok(found) => found,
        Err(error) => return Some(report(&error)),
    };
    return match name {
        "install" => Some(install_verb(matches, &scope, &env)),
        "update" | "outdated" => Some(update_verbs(name, matches, &scope, &env)),
        "verify-archive" => Some(verify_archive_verb(matches, &scope)),
        "list" => Some(list_verb(&scope, &env)),
        "bin" => Some(bin_verb(matches, &scope, &env)),
        _ => None,
    };
}
