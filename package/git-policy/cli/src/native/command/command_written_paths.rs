//! What:
//!  Which tree paths one guarded form writes into the worktree,
//!  read from the form's own words.
//! Why:
//!  The guard has to know what a command would overwrite before it can save it:
//!  a status names the worktree's own changes,
//!  never the files Git did not put there,
//!  and an untracked or ignored file that stands at a path the form writes is lost
//!  whether or not anything tracked is dirty.
//!  A form that writes a tree writes every path of it,
//!  a form that names paths writes those of them the tree holds,
//!  and a restore without a source writes the index's own paths.
//!  Reading the words here keeps that question in one place,
//!  where the same option tables Git uses decide what the words mean.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writtenPaths('ResetHard', ['--hard', 'other']) === { kind: 'tree', revision: 'other' }
//! // writtenPaths('CheckoutPaths', ['other', '--', 'b.txt']) === { kind: 'matching-tree', revision: 'other', pathspecs: ['b.txt'] }
//! ```

use crate::command_checkout_table::{CHECKOUT_TABLE, FORCE, SWITCH_TABLE};
use crate::command_discard::{restores_index_only, spells_long};
/// Git's own option grammar, for where a region's options end and its paths begin.
use crate::command_options::{Boundary, DEFAULT_MODE, ParsedOptions, parse_options};
/// The token indices that are positionals rather than options.
use crate::command_options_query::{is_enabled, positional_tokens};
use crate::command_reset::{RESET_TABLE, ResetMode, parse_reset_region};
/// The forms the linked-worktree policy guards.
use crate::rule_linked_worktree::GuardedCommand;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;

/// What:
///  The paths one form writes, as the source of them.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A whole tree is written by a forced move of `HEAD`,
///  a named part of one by a path form,
///  and a restore without a source writes what the index holds;
///  each answer is a different query,
///  and the pathspecs alone decide which of those paths the form reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WrittenPaths = { kind: 'tree'; revision?: string } | { kind: 'matching-tree'; revision?: string; pathspecs: string[] }
///   | { kind: 'index'; pathspecs: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WrittenPaths {
    /// Every path of the revision's tree,
    ///  `HEAD` when the form names none.
    Tree {
        /// The revision the form names,
        ///  or nothing for `HEAD`.
        revision: Option<OsString>,
    },
    /// The paths of the revision's tree that the form's own pathspecs match.
    MatchingTree {
        /// The revision the form names,
        ///  or nothing for `HEAD`.
        revision: Option<OsString>,
        /// The paths the form names,
        ///  which Git matches against the tree.
        pathspecs: Vec<OsString>,
    },
    /// The paths the index holds,
    ///  which a restore without a source writes over.
    Index {
        /// The paths the form names,
        ///  and nothing for every path of the index.
        pathspecs: Vec<OsString>,
    },
}

impl WrittenPaths {
    /// What:
    ///  The word this answer is named by,
    ///  for a record of the question that was asked.
    /// Why:
    ///   A record of what the guard measured names the reading it made,
    ///  and the three readings are not the same question.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// written.kind === 'tree' | 'matching-tree' | 'index'
    /// ```
    pub fn kind(&self) -> &'static str {
        return match self {
            WrittenPaths::Tree { .. } => "tree",
            WrittenPaths::MatchingTree { .. } => "matching-tree",
            WrittenPaths::Index { .. } => "index",
        };
    }
}

/// What:
///  The tree paths one guarded form writes,
///  or nothing for a form that writes no file of a tree.
///  `&[OsString]` borrows the tokens after the subcommand.
/// Why:
///   A forced move of `HEAD` and a path form write files;
///  a `clean` deletes what Git's own dry run names,
///  a `rm` and a `mv` reach only paths their index already holds,
///  and a `read-tree` without `-u` writes no worktree file,
///  so none of those is read here.
///  A `reset --merge` or `reset --keep` over a file Git did not put there is refused by Git itself,
///  which deletes nothing,
///  so only a hard reset is read:
///  saving a copy for a command Git then refuses would move nothing and say nothing true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writtenPaths(command: GuardedCommand, region: string[]): WrittenPaths | undefined;
/// ```
pub fn written_paths(command: GuardedCommand, region: &[OsString]) -> Option<WrittenPaths> {
    match command {
        GuardedCommand::ResetHard => {
            // `--soft` writes nothing and is not guarded; `--merge` and `--keep` leave a path Git did not write alone.
            let parsed = parse_reset_region(region, &[]).ok()?;
            if parsed.mode != Some(ResetMode::Hard) {
                return None;
            }
            return Some(WrittenPaths::Tree { revision: crate::command_discard::revision_before_paths(region, RESET_TABLE) });
        }
        GuardedCommand::CheckoutPaths => return checkout_written_paths(region),
        GuardedCommand::SwitchForce => {
            return Some(WrittenPaths::Tree { revision: first_positional(region, SWITCH_TABLE) });
        }
        GuardedCommand::RestoreWorktree => return restore_written_paths(region),
        _ => return None,
    }
}

/// What:
///  The paths a `git checkout` form writes.
/// Why:
///   A forced checkout with no path switches the whole tree;
///  a path form writes only the paths it names,
///  read from the revision before them or from the index when it names none,
///  and `git checkout <words> -- <paths>` is the spelling that always means paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutWrittenPaths(region: string[]): WrittenPaths | undefined; // throws-free
/// ```
fn checkout_written_paths(region: &[OsString]) -> Option<WrittenPaths> {
    let parsed: ParsedOptions = parse_options(region, CHECKOUT_TABLE, DEFAULT_MODE, &[]).ok()?;
    let names: Vec<&OsString> = positional_tokens(&parsed, region.len())
        .into_iter()
        .filter_map(|index| region.get(index))
        .collect();
    // A region Git refuses names nothing, and `git checkout` refuses a path form that also forces a branch.
    if matches!(parsed.boundary, Boundary::DashDash(_)) {
        let (before, pathspecs): (Vec<&OsString>, Vec<OsString>) = split_at_dash_dash(region);
        let revision: Option<OsString> = before.first().map(|name| (*name).clone());
        return Some(WrittenPaths::MatchingTree { revision, pathspecs });
    }
    if names.len() > 1 {
        let revision: Option<OsString> = Some(names[0].clone());
        return Some(WrittenPaths::MatchingTree {
            revision,
            pathspecs: names[1..].iter().map(|name| (*name).clone()).collect(),
        });
    }
    if !is_enabled(&parsed, FORCE) {
        return None;
    }
    return Some(WrittenPaths::Tree { revision: names.first().map(|name| (*name).clone()) });
}

/// What:
///  The paths a `git restore` form writes over,
///  read from its `--source` and its own path list.
/// Why:
///  Without a source the index's content goes back over the worktree,
///  and with one the source tree's content does;
///  a form that restores only the index writes no worktree file at all.
///  The paths are the tokens after `--` when it is written,
///  and otherwise every token that is not an option or the value of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreWrittenPaths(region: string[]): WrittenPaths | undefined;
/// ```
fn restore_written_paths(region: &[OsString]) -> Option<WrittenPaths> {
    if restores_index_only(region) {
        return None;
    }
    let source: Option<OsString> = restore_source(region);
    let pathspecs: Vec<OsString> = restore_pathspecs(region);
    return Some(match source {
        Some(revision) => WrittenPaths::MatchingTree { revision: Some(revision), pathspecs },
        None => WrittenPaths::Index { pathspecs },
    });
}

/// What:
///  The revision a `git restore` form names as its source,
///  in any spelling Git's own table accepts.
/// Why:
///   `--source <tree>`, `--source=<tree>`, `-s <tree>` and `-s<tree>` are one option,
///  and a value that looks like a flag is that value (`restore --source -W -- f`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreSource(region: string[]): string | undefined;
/// ```
fn restore_source(region: &[OsString]) -> Option<OsString> {
    let mut at: usize = 0;
    while at < region.len() {
        let written: &[u8] = region[at].as_encoded_bytes();
        at += 1;
        if written == b"--" {
            return None;
        }
        if let Some(value) = long_option_value(written, "source") {
            return Some(value);
        }
        let long_without_value: bool =
            written.starts_with(b"--") && !written.contains(&b'=') && spells_long(written, "source", crate::command_discard::RESTORE_LONG_OPTIONS);
        if long_without_value {
            return region.get(at).cloned();
        }
        if written.starts_with(b"--") {
            if !written.contains(&b'=') && option_takes_a_value(written) {
                at += 1;
            }
            continue;
        }
        if written.len() > 1 && written[0] == b'-' {
            let cluster: &[u8] = &written[1..];
            let position: usize = match cluster.iter().position(|byte| *byte == b's') {
                Some(found) => found,
                None => {
                    continue;
                }
            };
            let value: &[u8] = &cluster[position + 1..];
            if !value.is_empty() {
                return Some(OsString::from(String::from_utf8_lossy(value).into_owned()));
            }
            return region.get(at).cloned();
        }
    }
    return None;
}

/// What:
///  The value a long option carries inside its own token (`--name=value`),
///  by any abbreviation Git accepts for `restore`.
/// Why:
///   Git accepts an unambiguous prefix of every long option,
///  and `--so`/`--sour` are `--source`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function longOptionValue(token: string, name: string): string | undefined; // the part after '='
/// ```
fn long_option_value(written: &[u8], name: &str) -> Option<OsString> {
    if !spells_long(written, name, crate::command_discard::RESTORE_LONG_OPTIONS) {
        return None;
    }
    let equals: usize = written.iter().position(|byte| *byte == b'=')?;
    return Some(OsString::from(String::from_utf8_lossy(&written[equals + 1..]).into_owned()));
}

/// What:
///  Whether one long option of `git restore` takes the next token as its value.
/// Why:
///   `--source` and `--conflict` do,
///  and every other long option of this command does not,
///  so the token after one of them is a value and never a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const optionTakesAValue = (token) => ['source', 'conflict'].some((name) => spellsLong(token, name));
/// ```
fn option_takes_a_value(written: &[u8]) -> bool {
    let names: [&str; 2] = ["source", "conflict"];
    return names.iter().any(|name| spells_long(written, name, crate::command_discard::RESTORE_LONG_OPTIONS));
}

/// What:
///  The paths a `git restore` form names,
///  which are the tokens after `--` when it is written and the plain words of the region otherwise.
/// Why:
///   Everything after `--` is a path to Git,
///  and before it every token that is not an option and not the value of one is a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restorePathspecs(region: string[]): string[];
/// ```
fn restore_pathspecs(region: &[OsString]) -> Vec<OsString> {
    let mut paths: Vec<OsString> = Vec::<OsString>::new();
    let mut at: usize = 0;
    while at < region.len() {
        let written: &[u8] = region[at].as_encoded_bytes();
        at += 1;
        if written == b"--" {
            paths.extend(region[at..].iter().cloned());
            return paths;
        }
        if written.starts_with(b"--") {
            if !written.contains(&b'=') && option_takes_a_value(written) {
                at += 1;
            }
            continue;
        }
        if written.len() > 1 && written[0] == b'-' {
            // A short cluster that ends with `s` takes the next token as its value.
            if written[1..].ends_with(b"s") {
                at += 1;
            }
            continue;
        }
        paths.push(region[at - 1].clone());
    }
    return paths;
}

/// What:
///  The region split at the first `--`:
///  the words before it and the paths after it.
/// Why:
///   A path after `--` is never an option,
///  and only the words before it can name a revision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitAtDashDash(region: string[]): [string[], string[]];
/// ```
fn split_at_dash_dash(region: &[OsString]) -> (Vec<&OsString>, Vec<OsString>) {
    let mut before: Vec<&OsString> = Vec::<&OsString>::new();
    for (index, token) in region.iter().enumerate() {
        if token.as_encoded_bytes() == b"--" {
            return (before, region[index + 1..].to_vec());
        }
        before.push(token);
    }
    return (before, Vec::<OsString>::new());
}

/// What:
///  The first positional word of a region,
///  which a `switch` reads as the branch or commit it moves to.
/// Why:
///   `git switch -f <target>` writes the target's whole tree,
///  and every option before it is not part of the target's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstPositional(region: string[], table: OptionSpec[]): string | undefined;
/// ```
fn first_positional(region: &[OsString], table: &[crate::command_options::OptionSpec]) -> Option<OsString> {
    let parsed: ParsedOptions = parse_options(region, table, DEFAULT_MODE, &[]).ok()?;
    let index: &usize = parsed.leading.first()?;
    return region.get(*index).cloned();
}

/// Reading the forms' words into the paths they write,
/// beside real-Git controls of each spelling.
#[cfg(test)]
#[path = "command_written_paths_tests.rs"]
mod tests;
