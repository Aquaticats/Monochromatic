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
use crate::command_discard::{
    READ_TREE_DEBUG_UNPACK, READ_TREE_DRY_RUN, READ_TREE_MERGE, READ_TREE_PREFIX, READ_TREE_RESET, READ_TREE_TABLE,
    READ_TREE_UPDATE, restores_index_only, spells_long,
};
/// Git's own option grammar, for where a region's options end and its paths begin.
use crate::command_options::{Boundary, DEFAULT_MODE, ParsedOptions, parse_options};
/// The token indices that are positionals rather than options,
///  and the exact bytes of an option's value.
use crate::command_options_query::{is_enabled, positional_tokens, value_bytes};
use crate::command_reset::{RESET_TABLE, ResetMode, parse_reset_region};
/// The forms the linked-worktree policy guards.
use crate::rule_linked_worktree::GuardedCommand;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;

/// What:
///  Which of the worktree's entries at a written path Git writes over:
///  every one, or only the ones its ignore rules cover.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A forced move of `HEAD` replaces or deletes any entry in its way,
///  untracked or ignored alike;
///  a `reset --merge` or `reset --keep` refuses the untracked one and writes over the ignored one
///  (measured on this host with native Git 2.56.0:
///  `Untracked working tree file 'b.txt' would be overwritten` and stop,
///  against an ignored `b.txt` replaced with exit 0),
///  so asking about the untracked entries there would copy a file Git keeps
///  and leave a notice that says the opposite.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WrittenEntries = 'every-entry' | 'only-ignored';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WrittenEntries {
    /// Every entry at the path, whether Git's rules cover it or not.
    EveryEntry,
    /// Only the entries the worktree's ignore rules cover.
    OnlyIgnored,
}

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
/// type WrittenPaths = { kind: 'tree'; revision?: string; entries: WrittenEntries }
///   | { kind: 'matching-tree'; revision?: string; pathspecs: string[] }
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
        /// Which entries at those paths Git writes over.
        entries: WrittenEntries,
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
    /// The paths of every tree a `git read-tree -u` names,
    ///  each below the prefix the form writes them under.
    ReadTree {
        /// The trees the form names,
        ///  whose merged paths it writes into the worktree.
        revisions: Vec<OsString>,
        /// The directory the trees land in,
        ///  when the form writes one with `--prefix`.
        prefix: Option<OsString>,
        /// Which entries at those paths Git writes over.
        entries: WrittenEntries,
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
            WrittenPaths::Tree { entries: WrittenEntries::EveryEntry, .. } => "tree",
            WrittenPaths::Tree { entries: WrittenEntries::OnlyIgnored, .. } => "tree-of-ignored-entries",
            WrittenPaths::MatchingTree { .. } => "matching-tree",
            WrittenPaths::Index { .. } => "index",
            WrittenPaths::ReadTree { entries: WrittenEntries::EveryEntry, .. } => "read-tree-trees",
            WrittenPaths::ReadTree { entries: WrittenEntries::OnlyIgnored, .. } => "read-tree-trees-of-ignored-entries",
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
///  and a `rm` and a `mv` reach only paths their index already holds,
///  so none of those is read here.
///  A hard reset writes over every entry at the tree's paths;
///  `reset --merge` and `reset --keep` write the tree too,
///  but refuse an untracked entry and write over an ignored one,
///  so their reading asks about the ignored entries alone.
///  A `read-tree -u` writes the paths of every tree it names,
///  below the directory `--prefix` puts them in,
///  and which entries it writes over depends on the mode it carries:
///  `--reset` writes over an untracked file as well,
///  while `-m` alone refuses one and writes over an ignored one
///  (`SPEC.md`, the discarding-commands section, holds both measurements).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writtenPaths(command: GuardedCommand, region: string[]): WrittenPaths | undefined;
/// ```
pub fn written_paths(command: GuardedCommand, region: &[OsString]) -> Option<WrittenPaths> {
    match command {
        GuardedCommand::ResetHard => {
            // `--soft` and `--mixed` write no worktree file; `--merge` and `--keep` write the tree but refuse an
            //  untracked entry in its way, so only the entries their ignore rules cover are asked about.
            let parsed = parse_reset_region(region, &[]).ok()?;
            let revision: Option<OsString> = crate::command_discard::revision_before_paths(region, RESET_TABLE);
            return match parsed.mode {
                Some(ResetMode::Hard) => Some(WrittenPaths::Tree { revision, entries: WrittenEntries::EveryEntry }),
                Some(ResetMode::Merge) | Some(ResetMode::Keep) => {
                    Some(WrittenPaths::Tree { revision, entries: WrittenEntries::OnlyIgnored })
                }
                _ => None,
            };
        }
        GuardedCommand::CheckoutPaths => return checkout_written_paths(region),
        GuardedCommand::SwitchForce => {
            return Some(WrittenPaths::Tree {
                revision: first_positional(region, SWITCH_TABLE),
                entries: WrittenEntries::EveryEntry,
            });
        }
        GuardedCommand::RestoreWorktree => return restore_written_paths(region),
        GuardedCommand::ReadTree => return read_tree_written_paths(region),
        _ => return None,
    }
}

/// What:
///  The paths a `git read-tree` form writes into the worktree,
///  which are the paths of every tree it names,
///  each below the directory its `--prefix` puts it in.
/// Why:
///   Git writes no worktree file without `-u`,
///  and refuses `-u` unless the form also carries `-m`,
///  `--reset` or `--prefix`,
///  so only those forms write anything here.
///  Every path a merge can write comes from one of the trees it names,
///  so asking each of them is the whole set.
///  Measured on this host with native Git 2.56.0:
///  `--reset -u` writes over an untracked file at such a path and exits 0,
///  while `-m -u` and `--prefix` refuse an untracked one in Git's own words
///  and write over the one their ignore rules cover,
///  which is why only the reset form asks about the untracked entries too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTreeWrittenPaths(region: string[]): WrittenPaths | undefined;
/// ```
fn read_tree_written_paths(region: &[OsString]) -> Option<WrittenPaths> {
    let parsed: ParsedOptions = parse_options(region, READ_TREE_TABLE, DEFAULT_MODE, &[]).ok()?;
    // A dry run and `--debug-unpack` write no worktree file at all.
    if !is_enabled(&parsed, READ_TREE_UPDATE)
        || is_enabled(&parsed, READ_TREE_DRY_RUN)
        || is_enabled(&parsed, READ_TREE_DEBUG_UNPACK)
    {
        return None;
    }
    let reset: bool = is_enabled(&parsed, READ_TREE_RESET);
    // A region Git refuses writes nothing; Git says so itself.
    if !reset && !is_enabled(&parsed, READ_TREE_MERGE) && !is_enabled(&parsed, READ_TREE_PREFIX) {
        return None;
    }
    let mut revisions: Vec<OsString> = Vec::<OsString>::new();
    for index in positional_tokens(&parsed, region.len()) {
        if let Some(token) = region.get(index) {
            revisions.push(token.clone());
        }
    }
    if revisions.is_empty() {
        return None;
    }
    let mut prefix: Option<OsString> = None;
    for occurrence in &parsed.occurrences {
        if occurrence.id == READ_TREE_PREFIX
            && let Some(value) = occurrence.value
        {
            prefix = Some(OsString::from(String::from_utf8_lossy(value_bytes(region, value)).into_owned()));
        }
    }
    return Some(WrittenPaths::ReadTree {
        revisions,
        prefix,
        entries: if reset { WrittenEntries::EveryEntry } else { WrittenEntries::OnlyIgnored },
    });
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
        .filter_map(|index| return region.get(index))
        .collect();
    // Everything after `--` is a path, and the one word before it that is no option is the revision Git reads.
    if let Boundary::DashDash(boundary) = parsed.boundary {
        let pathspecs: Vec<OsString> = region.get(boundary + 1..).unwrap_or(&[]).to_vec();
        let mut revision: Option<OsString> = None;
        for index in positional_tokens(&parsed, region.len()) {
            if index < boundary {
                revision = region.get(index).cloned();
                break;
            }
        }
        return Some(WrittenPaths::MatchingTree { revision, pathspecs });
    }
    if names.len() > 1 {
        let revision: Option<OsString> = Some(names[0].clone());
        return Some(WrittenPaths::MatchingTree {
            revision,
            pathspecs: names[1..].iter().map(|name| return (*name).clone()).collect(),
        });
    }
    if !is_enabled(&parsed, FORCE) {
        return None;
    }
    return Some(WrittenPaths::Tree {
        revision: names.first().map(|name| return (*name).clone()),
        entries: WrittenEntries::EveryEntry,
    });
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
            let position: usize = match cluster.iter().position(|byte| return *byte == b's') {
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
    let equals: usize = written.iter().position(|byte| return *byte == b'=')?;
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
    return names.iter().any(|name| return spells_long(written, name, crate::command_discard::RESTORE_LONG_OPTIONS));
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
