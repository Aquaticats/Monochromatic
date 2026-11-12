//! What:
//!  Keeping the nested Git repositories a clean would delete out of an interactive clean's menu,
//!  by running that clean with the one force at which Git keeps every nested repository.
//! Why:
//!  An interactive clean deletes nothing until its person answers a menu,
//!  and that answer comes after Git has started,
//!  so nothing the wrapper runs beforehand can ask what the person will choose.
//!  Moving a repository aside before the menu takes it out of the choices,
//!  and a person who then quits has had their worktree changed by a command that deleted nothing.
//!  Git's own protection at one force keeps every nested repository where it is
//!  (`builtin/clean.c`,
//! `if (force > 1) rm_flags = 0`),
//!  and `--no-force -f` is how a clean typed with two or more forces runs as though it were typed with one:
//!  the flag resets Git's count,
//!  so the menu offers exactly what the clean deletes and the repository is never moved.
//!  The person is told which repository is kept and how to delete it deliberately,
//!  and the clean as typed (without `-i`) saves it first and then deletes it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const capped = keepNestedRepositoriesOutOfAnInteractiveMenu(['clean', '-i', '-ffdX'], facts);
//! // capped.arguments === ['clean', '-i', '-ffdX', '--no-force', '-f']
//! ```

/// The table and the readings of `git clean`,
///  and the two sentences a failed reading is told apart by.
use crate::command_clean::{CLEAN_TABLE, UNREADABLE_CLEAN_DELETION, clean_dry_run_arguments, parse_clean_region};
/// Which nested Git repositories that reading says the clean would delete.
use crate::command_clean_nested_repositories::nested_repositories_the_clean_deletes;
/// Git's own option grammar,
///  for where the clean's options end.
use crate::command_options::{Boundary, DEFAULT_MODE, ParsedOptions, parse_options};
/// Where the subcommand starts in the invocation's arguments.
use crate::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
/// The facts interface this transform asks.
use crate::repository_facts::RepositoryFacts;
/// The token copier of every transform that adds tokens.
use crate::rule_argument_rewrite::insert_tokens;
/// The directory the paths Git prints are below.
use crate::worktree_identity::worktree_root;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// The tokens that make a clean run with exactly one force,
///  whatever it was typed with.
/// Why:
///   Git counts `-f` and resets the count on `--no-force`
///  (`parse-options.c`,
///  `OPTION_COUNTUP`),
///  so the pair lands on one force whatever the person's spelling was,
///  including a cluster such as `-iffdX`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const ONE_FORCE = ['--no-force', '-f'] as const;
/// ```
pub const ONE_FORCE: &[&str] = &["--no-force", "-f"];

/// What:
///  What an interactive clean needs from this transform.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A clean that keeps everything Git keeps by itself needs nothing,
///  a clean whose repositories were found is run with one force and told to the person,
///  and a clean whose reading never answered must stop rather than guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CapDecision = { kind: 'unchanged' } | { kind: 'capped'; arguments; repositories } | { kind: 'stopped'; message };
/// ```
pub enum CapDecision {
    /// Nothing of this clean's deletion reaches a nested repository.
    Unchanged,
    /// The clean runs with one force,
    ///  and these repositories stay where they are.
    Capped {
        /// The invocation with `--no-force -f` at the end of the clean's options.
        arguments: Vec<OsString>,
        /// The repositories the menu no longer offers,
        ///  as Git named the paths they are inside.
        repositories: Vec<Vec<u8>>,
    },
    /// Git never answered what the clean would delete,
    ///  so the command stops with the sentence saying so.
    Stopped {
        /// What the person reads instead of the menu.
        message: String,
    },
}

/// What:
///  One interactive clean this transform may have to cap:
///  the arguments that read what it would delete,
///  and the index where the cap goes.
/// Why:
///   The reading is the clean as typed,
///  and the cap goes after the clean's own options so the person's spelling still decides everything else,
///  while their later `-f` tokens can no longer raise the count past one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InteractiveClean = { query: string[]; insertion: number };
/// ```
struct InteractiveClean {
    /// The dry-run arguments of the clean as typed.
    query: Vec<OsString>,
    /// The index in the invocation's arguments where `--no-force -f` is written.
    insertion: usize,
}

/// What:
///  Cap an interactive clean's force count when its menu would offer a nested Git repository,
///  and answer what it found.
///  `&[OsString]` borrows the invocation's arguments,
///  already free of wrapper controls.
/// Why:
///   This is the one reading an interactive clean gets,
///  because its menu cannot be asked before Git deletes;
///  a repository Git would keep at one force is one no menu may offer,
///  and the clean still runs every other thing the person asked for.
///  A clean Git refuses deletes nothing and needs no cap,
///  and a reading that never answered stops the command,
///  because a repository it could not see may be one the menu would have offered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keepNestedRepositoriesOutOfAnInteractiveMenu(args, facts): CapDecision;
/// ```
pub fn keep_nested_repositories_out_of_an_interactive_menu(
    arguments: &[OsString],
    facts: &mut dyn RepositoryFacts,
) -> CapDecision {
    let Some(clean) = plan_interactive_clean(arguments) else {
        return CapDecision::Unchanged;
    };
    let deleted: Vec<Vec<u8>> = match facts.clean_deleted_paths(clean.query) {
        Ok(answer) => answer,
        // Git refuses this clean itself, so it deletes nothing and Git's own refusal is the answer the person gets.
        Err(message) if message.starts_with(UNREADABLE_CLEAN_DELETION) => return CapDecision::Unchanged,
        // A reading that never answered, or one the wrapper ended, could hide a repository the menu would offer.
        Err(message) => return CapDecision::Stopped { message },
    };
    let root: std::path::PathBuf = match facts.location() {
        Ok(location) => match worktree_root(&location.identity) {
            Some(found) => found.to_path_buf(),
            None => return CapDecision::Unchanged,
        },
        Err(_) => return CapDecision::Unchanged,
    };
    let repositories: Vec<Vec<u8>> = match nested_repositories_the_clean_deletes(deleted.as_slice(), root.as_path()) {
        Ok(found) => found,
        Err(message) => return CapDecision::Stopped { message },
    };
    if repositories.is_empty() {
        return CapDecision::Unchanged;
    }
    return CapDecision::Capped {
        arguments: insert_tokens(arguments, clean.insertion, ONE_FORCE),
        repositories,
    };
}

/// What:
///  Whether this invocation is an interactive clean whose second force removes nested repositories,
///  and where its options end.
/// Why:
///   Every forwarded command reaches this transform,
///  so the reading stops at the first token that is not a `clean` subcommand;
///  `-i` is what makes the menu,
///  `-d` is what lets the clean descend into a repository,
///  an ignored mode is what lets it delete an ignored one,
///  and one force is Git's own protection of every nested repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planInteractiveClean(args): InteractiveClean | undefined;
/// ```
fn plan_interactive_clean(arguments: &[OsString]) -> Option<InteractiveClean> {
    let layout: GlobalLayout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command {
        return None;
    }
    if arguments.get(layout.prefix_len)?.as_encoded_bytes() != b"clean" {
        return None;
    }
    let region: &[OsString] = &arguments[layout.prefix_len + 1..];
    // A region Git refuses runs nothing, and one force keeps every nested repository by Git's own rule.
    let parsed: ParsedOptions = parse_options(region, CLEAN_TABLE, DEFAULT_MODE, &[]).ok()?;
    let facts = parse_clean_region(region, &[]).ok()?;
    if !facts.interactive || !facts.descends || !facts.deletes_ignored || facts.forces < 2 {
        return None;
    }
    return Some(InteractiveClean {
        query: clean_dry_run_arguments(&arguments[..layout.prefix_len], region)?,
        insertion: layout.prefix_len + 1 + option_region_end(&parsed, region.len()),
    });
}

/// What:
///  The index where the clean's options end,
///  which is where a flag added for Git must go so no path is read as an option.
/// Why:
///   Everything after `--`,
///  `--end-of-options` or the first non-option token is a path to Git,
///  and a flag written there would name a path instead of changing the clean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function optionRegionEnd(parsed, length): number;
/// ```
fn option_region_end(parsed: &ParsedOptions, length: usize) -> usize {
    // `.min(...)` is the smaller index; a token that ends the options earlier wins.
    let mut end: usize = match parsed.boundary {
        Boundary::End => length,
        Boundary::DashDash(at) | Boundary::EndOfOptions(at) | Boundary::NonOption(at) => at,
    };
    if let Some(first) = parsed.leading.first() {
        end = end.min(*first);
    }
    return end;
}

/// The plan,
///  its rewrite and every clean it leaves alone.
#[cfg(test)]
#[path = "rule_clean_interactive_nested_repositories_tests.rs"]
mod tests;
