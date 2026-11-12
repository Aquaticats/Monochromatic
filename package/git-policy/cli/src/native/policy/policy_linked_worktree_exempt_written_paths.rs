//! What:
//!  The paths one guarded form writes into the worktree,
//!  read from the form's own words.
//! Why:
//!  A command that would overwrite or delete an untracked file is refused,
//!  and only the form's own grammar says which paths it writes:
//!  a hard reset writes a whole tree,
//!  a forced checkout with paths writes the paths it names,
//!  a restore without a source writes the index's content,
//!  and a move writes its destination.
//!  Reading them here keeps that question in one place,
//!  where Git's own option tables decide what the words mean;
//!  the paths are then matched against the worktree's untracked entries by Git
//!  (`repository_facts_paths_the_form_writes`).
//!  A form whose region Git would refuse writes nothing,
//!  and a `reset --merge` or `--keep` refuses to replace an untracked file itself,
//!  so neither has an untracked entry to refuse over here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writtenPaths('ResetHard', ['--hard', 'other'], root) === { kind: 'tree', revision: 'other' }
//! ```

/// Git's own option grammar for each form whose written paths are read.
use crate::command_checkout_table::{CHECKOUT_TABLE, SWITCH_TABLE};
/// The reset mode and revision reader,
///  whose own table says where a revision sits.
use crate::command_discard::revision_before_paths;
/// Git's option parser,
///  its refusal,
///  and where its options end.
use crate::command_options::{Boundary, DEFAULT_MODE, OptionError, ParsedOptions, parse_options};
/// The positional tokens of a parsed region.
use crate::command_options_query::positional_tokens;
/// The reset grammar.
use crate::command_reset::{RESET_TABLE, ResetMode, parse_reset_region};
/// The forms the linked-worktree policy guards.
use crate::rule_linked_worktree::GuardedCommand;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The paths one form writes,
///  as the source of them.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A whole tree or a part of one is one query over Git's own tree,
///  the index's content is another over its own listing,
///  and a move names its destination as a path,
///  so each answer is a different reading of the worktree.
///  `prefix` moves every written path,
///  as `read-tree --prefix` and `checkout-index --prefix` do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Written = { kind: 'tree'; revision?: string; pathspecs: string[]; prefix: string }
///   | { kind: 'index'; pathspecs: string[]; prefix: string }
///   | { kind: 'paths'; paths: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Written {
    /// Every path of the revision's tree,
    ///  or the paths of it the pathspecs match when there are any.
    Tree {
        /// The revision the form names,
        ///  or nothing for `HEAD`.
        revision: Option<OsString>,
        /// The paths the form names,
        ///  which Git matches against the tree;
        ///  empty for the whole of it.
        pathspecs: Vec<OsString>,
        /// The directory every written path goes under,
        ///  or nothing.
        prefix: Vec<u8>,
    },
    /// Every path the index holds,
    ///  or the paths of it the pathspecs match when there are any.
    Index {
        /// The paths the form names;
        ///  empty for every path of the index.
        pathspecs: Vec<OsString>,
        /// The directory every written path goes under,
        ///  or nothing.
        prefix: Vec<u8>,
    },
    /// The literal paths the form writes.
    Paths {
        /// Each path as the form spelled the destination.
        paths: Vec<Vec<u8>>,
    },
}

/// What:
///  The paths one guarded form writes,
///  or nothing for a form that writes no untracked entry.
///  `&[OsString]` borrows the tokens after the subcommand,
///  and `&Path` the worktree top a move's destination is read from.
/// Why:
///   A clean's own dry run says what it deletes,
///  a stash and `rm` reach only what the index already holds,
///  and a `reset --merge` or `--keep` refuses an untracked file in its way itself;
///  every other form names or implies the paths a refusal must look at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writtenPaths(command: GuardedCommand, region: string[], root: string): Written | undefined; // throws
/// ```
pub(crate) fn written_paths(
    command: GuardedCommand,
    region: &[OsString],
    worktree_root: &Path,
) -> Result<Option<Written>, String> {
    return match command {
        GuardedCommand::ResetHard => reset_paths(region),
        GuardedCommand::CheckoutPaths => checkout_paths(region),
        GuardedCommand::SwitchForce => switch_paths(region),
        _ => crate::policy_linked_worktree_exempt_written_paths_other_forms::written_paths_of_other_forms(
            command,
            region,
            worktree_root,
        ),
    };
}

/// What:
///  Git's own reading of a region,
///  or the sentence a form whose words could not be read is refused with.
/// Why:
///   A region the classifier accepted is read again here,
///  so a refusal is the answer to words this second reading cannot place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readWords(parsed, what: string): T; // throws
/// ```
pub(super) fn read_words<T>(parsed: Result<T, OptionError>, what: &str) -> Result<T, String> {
    return match parsed {
        Ok(value) => Ok(value),
        Err(error) => Err(format!("the {what} could not be read: {error}")),
    };
}

/// What:
///  Every positional word of a parsed region,
///  in the order Git reads them.
/// Why:
///   A checkout's revision and paths are two of these,
///  and a switch's target is the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wordsAt = (parsed, region) => positionalTokens(parsed, region.length).map((at) => region[at]);
/// ```
pub(super) fn words_at(parsed: &ParsedOptions, region: &[OsString]) -> Vec<OsString> {
    let mut words: Vec<OsString> = Vec::<OsString>::new();
    for index in positional_tokens(parsed, region.len()) {
        words.push(region[index].clone());
    }
    return words;
}

/// What:
///  The first positional word of a parsed region,
///  or nothing when it has none.
/// Why:
///   A forced switch and a forced checkout name their revision there,
///  and a form that names none writes `HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstWord = (parsed, region) => positionalTokens(parsed, region.length).at(0);
/// ```
pub(super) fn first_word(parsed: &ParsedOptions, region: &[OsString]) -> Option<OsString> {
    let positions: Vec<usize> = positional_tokens(parsed, region.len());
    let index: &usize = positions.first()?;
    return Some(region[*index].clone());
}

/// What:
///  The tree a hard reset writes,
///  and nothing for `--merge` or `--keep`,
///  which refuse to replace an untracked file themselves.
/// Why:
///   `--hard` replaces or deletes any entry at the tree's paths;
///  `--merge` and `--keep` stop at an untracked one,
///  so only an ignored file is at risk there and that reading is asked separately.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resetPaths(region: string[]): Written | undefined; // throws
/// ```
fn reset_paths(region: &[OsString]) -> Result<Option<Written>, String> {
    let parsed = read_words(parse_reset_region(region, &[]), "reset")?;
    return match parsed.mode {
        Some(ResetMode::Hard) => Ok(Some(Written::Tree {
            revision: revision_before_paths(region, RESET_TABLE),
            pathspecs: Vec::<OsString>::new(),
            prefix: Vec::<u8>::new(),
        })),
        _ => Ok(None),
    };
}

/// What:
///  The paths a `git checkout` form writes:
///  a whole tree for a forced move with no path,
///  and the paths it names otherwise.
/// Why:
///   `git checkout -- <paths>` writes the index's content over those paths,
///  `git checkout <rev> -- <paths>` writes the revision's,
///  and a forced form with no path writes the whole tree it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutPaths(region: string[]): Written | undefined; // throws
/// ```
fn checkout_paths(region: &[OsString]) -> Result<Option<Written>, String> {
    let parsed: ParsedOptions = read_words(parse_options(region, CHECKOUT_TABLE, DEFAULT_MODE, &[]), "checkout")?;
    if let Boundary::DashDash(boundary) = parsed.boundary {
        let pathspecs: Vec<OsString> = region.get(boundary + 1..).unwrap_or(&[]).to_vec();
        let mut named: Option<OsString> = None;
        for index in positional_tokens(&parsed, region.len()) {
            if index < boundary {
                named = Some(region[index].clone());
                break;
            }
        }
        return Ok(Some(match named {
            Some(revision) => Written::Tree {
                revision: Some(revision),
                pathspecs,
                prefix: Vec::<u8>::new(),
            },
            None => Written::Index {
                pathspecs,
                prefix: Vec::<u8>::new(),
            },
        }));
    }
    let words: Vec<OsString> = words_at(&parsed, region);
    if words.len() > 1 {
        return Ok(Some(Written::Tree {
            revision: Some(words[0].clone()),
            pathspecs: words[1..].to_vec(),
            prefix: Vec::<u8>::new(),
        }));
    }
    return Ok(Some(Written::Tree {
        revision: words.first().cloned(),
        pathspecs: Vec::<OsString>::new(),
        prefix: Vec::<u8>::new(),
    }));
}

/// What:
///  The whole tree a forced `switch` writes.
/// Why:
///   `git switch --force <target>` writes the target's tree over the worktree,
///  and a switch that keeps a branch's files moves only `HEAD`,
///  which is never the guarded form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function switchPaths(region: string[]): Written | undefined; // throws
/// ```
fn switch_paths(region: &[OsString]) -> Result<Option<Written>, String> {
    let parsed: ParsedOptions = read_words(parse_options(region, SWITCH_TABLE, DEFAULT_MODE, &[]), "switch")?;
    return Ok(Some(Written::Tree {
        revision: first_word(&parsed, region),
        pathspecs: Vec::<OsString>::new(),
        prefix: Vec::<u8>::new(),
    }));
}

/// The forms' words read into the paths they write,
///  with a control for each spelling this module decides.
#[cfg(test)]
#[path = "policy_linked_worktree_exempt_written_paths_tests.rs"]
mod tests;
