//! What:
//!  Pure decision of the add-explicit policy:
//!  `git add` names the paths it stages.
//! Why:
//!  `git add .`,
//!  `git add -A` and `git add -u` sweep up every changed path,
//!  including
//!      ones the caller never looked at,
//!  so one commit stops describing one change.
//!  Git reads a pathspec by its own grammar,
//!  and a fixed list of spellings cannot name every
//!      form that reaches the whole tree,
//!  so the policy asks the repository what a
//!      spelling selects and rejects the ones that select everything.
//!  This file decides from the arguments and that one measurement;
//!  it reads no
//!      file and starts no process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // decideAddExplicit(question, { selected: 5, repository: 5, directory: 5 }) => 'monochromatic-git-policy-cli: git add rejects bulk-staging patterns (.) ...'
//! ```

/// What:
///  `use` brings names from sibling files into this file;
///  `super::` means "the parent
///       module",
///  where every sibling file of this crate is declared.
/// Why:
///   Which tokens stage in bulk comes from Git's own option table and pathspec grammar of `git add`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ADD_ESCAPE_HATCH, LITERAL_VARIABLE, caller_global_prefix, literal_pathspecs_enabled, parse_add_region } from './command_add.ts';
/// ```
use crate::command_add::{
    ADD_ESCAPE_HATCH, AddRegion, LITERAL_VARIABLE, caller_global_prefix, literal_pathspecs_enabled,
    names_no_literal_character, parse_add_region,
};
use crate::command_options::OptionError;
use crate::global_arguments::command_tokens;
/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   Arguments are compared as bytes;
///  only the rejection text renders them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
///  The finding code of an add-explicit rejection.
///  `&str` is borrowed text baked into the
///       program.
/// Why:
///   Callers identify the finding by this stable code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BULK_ADD_CODE = 'bulk-add-rejected';
/// ```
pub const BULK_ADD_CODE: &str = "bulk-add-rejected";

/// What:
///  What one bulk-staging selector is,
/// which fixes what the diagnostic can say it selected.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   `-A` and `-u` are known without asking the repository,
///  and a pathspec is not,
///  so the two say different things about the number of paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BulkSelector = 'every-changed-path' | 'every-changed-tracked-path' | { pathspec: { namesNoLiteralCharacter: boolean } };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkSelector {
    /// `-A`,
    ///  `--all` or `--no-ignore-removal`:
    ///  every changed path of the worktree.
    EveryChangedPath,
    /// `-u` or `--update`:
    ///  every changed path the index already holds.
    EveryChangedTrackedPath,
    /// One positional pathspec:
    ///  Git says which paths it selects,
    ///  and `names_no_literal_character` says whether it can tell one path from another.
    Pathspec {
        /// The spelling names no character of a name,
        ///  so it selects by reach alone.
        names_no_literal_character: bool,
    },
}

/// What:
///  One token of the caller's command that may stage in bulk.
///  `usize` is the region token index;
///  `OsString` is the token as the caller wrote it.
/// Why:
///   The index keeps the tokens in the order the caller wrote them,
///  and the spelling lets the diagnostic quote each one exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BulkToken = { token: number; spelling: string; selector: BulkSelector };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkToken {
    /// Region token index of the token.
    pub token: usize,
    /// The token as the caller wrote it.
    pub spelling: OsString,
    /// What the token selects.
    pub selector: BulkSelector,
}

/// What:
///  What the policy needs to know about one `git add` before it can decide.
///  `Vec<BulkToken>` is an owned list in argument order.
/// Why:
///   A command with no such token is let through without asking the repository anything,
///  and one with a pathspec needs the measurement this question names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddExplicitQuestion = { tokens: BulkToken[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddExplicitQuestion {
    /// Every token that may stage in bulk,
    ///  in argument order.
    pub tokens: Vec<BulkToken>,
}

/// What:
///  The pathspec spellings Git must be asked about,
///  in argument order.
///  `Vec<OsString>` is an owned list of byte-preserving text.
/// Why:
///   The lookup reads the caller's own spellings,
///  never a rewritten form of them,
///  so its answer is the one the caller's `git add` will act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathspecSpellings = (question: AddExplicitQuestion) => question.tokens.filter((t) => typeof t.selector === 'object').map((t) => t.spelling);
/// ```
pub fn pathspec_spellings(question: &AddExplicitQuestion) -> Vec<OsString> {
    let mut spellings: Vec<OsString> = Vec::<OsString>::new();
    for found in &question.tokens {
        // `matches!` asks whether the selector is the pathspec variant without binding its field.
        if matches!(found.selector, BulkSelector::Pathspec { .. }) {
            // `.clone()` copies the spelling, which the caller keeps.
            spellings.push(found.spelling.clone());
        }
    }
    return spellings;
}

/// What:
///  Which reachable set a selection covers entirely.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   The diagnostic names the set the caller swept up,
///  and the set decides whether a pathspec that names something is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BulkScope = 'repository' | 'directory';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkScope {
    /// Every path the repository's worktree gives.
    Repository,
    /// Every path the directory the caller runs in gives,
    ///  which is the whole repository when that directory is its top.
    Directory,
}

/// What:
///  What Git answered about one command's pathspecs:
/// how many paths they select,
/// and how many the two reachable sets hold.
///  `usize` is the count type.
/// Why:
///   A selection is always a subset of the worktree's paths,
///  so two equal counts are two equal sets,
///  and the counts tell "everything" from "everything this directory holds"
///  without naming a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedSelection = { selected: number; repository: number; directory: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StagedSelection {
    /// How many paths the caller's pathspecs select.
    pub selected: usize,
    /// How many paths the repository's whole worktree gives.
    pub repository: usize,
    /// How many paths the directory the caller runs in gives.
    pub directory: usize,
}

/// What:
///  Which reachable set the caller's pathspecs cover entirely,
/// or nothing when they leave a path out.
///  `Option<BulkScope>` is "a set,
///  or nothing".
/// Why:
///   The policy refuses a reach-only spelling exactly when it sweeps up a whole set,
///  and the text says which set that was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scope = (s: StagedSelection) => s.selected === s.repository ? 'repository' : s.selected === s.directory ? 'directory' : undefined;
/// ```
pub fn scope_of(selection: &StagedSelection) -> Option<BulkScope> {
    // A repository with no path at all selects nothing, and nothing is not everything.
    if selection.selected == 0 {
        return None;
    }
    if selection.selected == selection.repository {
        return Some(BulkScope::Repository);
    }
    if selection.selected == selection.directory {
        return Some(BulkScope::Directory);
    }
    return None;
}

/// What:
///  What the policy must be told about the caller's own command,
///  or nothing when it stages nothing in bulk.
///  `Option<AddExplicitQuestion>` is "the question,
///  or nothing to ask".
/// Why:
///   Only `git add` is judged,
///  and only when Git accepts its options;
///  a region Git refuses is left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addExplicitQuestion(args: string[]): AddExplicitQuestion | undefined;
/// ```
pub fn add_explicit_question(arguments: &[OsString]) -> Option<AddExplicitQuestion> {
    // `let Some((a, b)) = ... else { ... };` unpacks the command word and the tokens after
    // it, or returns when Git runs no subcommand.
    let Some((word, region)) = command_tokens(arguments) else {
        // `None` is the "absent" case of `Option`.
        return None;
    };
    // `.as_encoded_bytes()` lends the raw bytes of the subcommand word.
    if word.as_encoded_bytes() != b"add" {
        return None;
    }
    // What the caller's own command makes of each spelling:
    // literal pathspec mode turns `**` and every `:` form into a plain file name,
    // so the reading of every spelling depends on the mode its caller set.
    let literal: bool = literal_pathspecs_enabled(
        caller_global_prefix(arguments),
        std::env::var_os(LITERAL_VARIABLE).as_deref(),
    );
    // `Result<A, B>` is "either success `A` or failure `B`"; `&[]` is an empty flag list.
    let parsed: Result<AddRegion, OptionError> = parse_add_region(region, &[]);
    // `let Ok(x) = ... else { ... };` unwraps the facts or returns for a refused region.
    let Ok(facts) = parsed else {
        return None;
    };
    let mut tokens: Vec<BulkToken> = Vec::<BulkToken>::new();
    // `if let Some(token) = ...` runs only when `-A` is on.
    if let Some(token) = facts.all_token {
        tokens.push(BulkToken {
            token,
            spelling: region[token].clone(),
            selector: BulkSelector::EveryChangedPath,
        });
    }
    if let Some(token) = facts.update_token {
        tokens.push(BulkToken {
            token,
            spelling: region[token].clone(),
            selector: BulkSelector::EveryChangedTrackedPath,
        });
    }
    for token in &facts.pathspec_tokens {
        // `.as_encoded_bytes()` lends the spelling's raw bytes; nothing is decoded or copied.
        let spelling: &[u8] = region[*token].as_encoded_bytes();
        tokens.push(BulkToken {
            token: *token,
            spelling: region[*token].clone(),
            selector: BulkSelector::Pathspec {
                names_no_literal_character: names_no_literal_character(spelling, literal),
            },
        });
    }
    // `sort_by_key` orders the list by the value the closure returns for each item.
    tokens.sort_by_key(token_of);
    if tokens.is_empty() {
        return None;
    }
    // `Some(x)` is the "present" case.
    return Some(AddExplicitQuestion { tokens });
}

/// What:
///  The region token index of one bulk token,
///  which is the order the caller wrote them in.
///  `&BulkToken` borrows one token.
/// Why:
///   A named function keeps the sort free of a closure that captures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tokenOf = (found: BulkToken) => found.token;
/// ```
fn token_of(found: &BulkToken) -> usize {
    return found.token;
}

/// What:
///  The sentence that says what the refused command sweeps up.
///  `Option<&StagedSelection>` is "the measurement,
///  or nothing when it could not be made".
/// Why:
///   The person needs to know how much the command would have staged,
///  and a flag's own reach is known without any measurement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sweptUp(question: AddExplicitQuestion, selection?: StagedSelection): string;
/// ```
fn swept_up(question: &AddExplicitQuestion, selection: Option<&StagedSelection>) -> String {
    if let Some(measured) = selection {
        match scope_of(measured) {
            Some(BulkScope::Repository) => {
                return format!("It selects all {} paths of this repository.", measured.selected);
            }
            Some(BulkScope::Directory) => {
                return format!("It selects all {} paths of this directory.", measured.selected);
            }
            // A selection that leaves a path out says nothing about what a flag reaches.
            None => {}
        }
    }
    let mut names_every_path: bool = false;
    for found in &question.tokens {
        // `match` unpacks the selector; the pathspec arm is no flag.
        match found.selector {
            BulkSelector::EveryChangedPath => names_every_path = true,
            BulkSelector::EveryChangedTrackedPath | BulkSelector::Pathspec { .. } => {}
        }
    }
    if names_every_path {
        return String::from("It selects every changed path in this repository.");
    }
    return String::from("It selects every changed tracked path in this repository.");
}

/// What:
///  Decide from the question and the measurement:
///  the rejection text,
///  or nothing to let the command
///       through.
///  `Option<&StagedSelection>` is the measurement when one was made.
/// Why:
///   A flag stages every changed path whatever the repository holds,
///  while a pathspec is refused only when Git found it selects everything
///  and its spelling names no character of a name.
///  The text quotes the caller's own tokens,
///  so `-vA` is named as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideAddExplicit(question: AddExplicitQuestion, selection?: StagedSelection): string | undefined;
/// ```
pub fn decide_add_explicit(
    question: &AddExplicitQuestion,
    selection: Option<&StagedSelection>,
) -> Option<String> {
    // What the pathspecs cover, which is what decides a pathspec spelling.
    let covered: Option<BulkScope> = selection.and_then(scope_of);
    // `Vec<String>` is an owned list of owned text; `mut` allows pushing.
    let mut named: Vec<String> = Vec::<String>::new();
    // `Option<usize>` is "the index of the token named last, or nothing".
    let mut named_last: Option<usize> = None;
    // `for found in &question.tokens` borrows each token in argument order.
    for found in &question.tokens {
        // A pathspec that names something stages everything only because the tree fits that name.
        let bulk: bool = match found.selector {
            BulkSelector::EveryChangedPath | BulkSelector::EveryChangedTrackedPath => true,
            BulkSelector::Pathspec {
                names_no_literal_character,
            } => names_no_literal_character && covered.is_some(),
        };
        // One cluster such as `-Au` matches twice; it is named once.
        if bulk && named_last != Some(found.token) {
            // `.to_string_lossy()` renders a token as text; `.into_owned()` makes it owned.
            named.push(found.spelling.to_string_lossy().into_owned());
        }
        named_last = Some(found.token);
    }
    if named.is_empty() {
        return None;
    }
    // `format!` builds the text; `.join(", ")` puts the separators between the tokens.
    return Some(format!(
        "monochromatic-git-policy-cli: git add rejects bulk-staging patterns ({}) because they sweep up paths the \
         caller did not intend to stage, leaving the index in a state that does not match a \
         single logical change. {} Name the paths explicitly, or pass {ADD_ESCAPE_HATCH} to bypass \
         for this invocation.",
        named.join(", "),
        swept_up(question, selection)
    ));
}

/// What:
///  The `ls-files` arguments that list every path the caller's own spellings select,
///  with the caller's pathspec mode exactly as its global options and environment set it.
///  `Vec<OsString>` is an owned argument list.
/// Why:
///  `--cached` and `--others` together are the paths `git add` can stage,
///  `--exclude-standard` leaves out the ignored ones it would skip,
///  and `-z` keeps every path's bytes exact.
///  The list is what the caller's own command acts on,
///  so its mode must not be rewritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const selectionQueryArguments = (prefix: string[], pathspecs: string[]) => [...prefix, 'ls-files', '-z', '--cached', '--others', '--exclude-standard', '--', ...pathspecs];
/// ```
pub fn selection_query_arguments(global_prefix: &[OsString], pathspecs: &[OsString]) -> Vec<OsString> {
    // `.to_vec()` copies the borrowed prefix into a list this function owns.
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for token in ["ls-files", "-z", "--cached", "--others", "--exclude-standard", "--"] {
        arguments.push(OsString::from(token));
    }
    // `.iter().cloned()` visits the spellings and copies each one onto the list.
    arguments.extend(pathspecs.iter().cloned());
    return arguments;
}

/// What:
///  The pathspec that names the top of the worktree,
///  whatever directory the command runs in.
/// Why:
///  The reference set is every path the worktree gives;
///  `--full-name` then prints each one from that top level,
///  which is the form the caller's directory prefix compares with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TOP_OF_THE_WORKTREE = ':/';
/// ```
pub const TOP_OF_THE_WORKTREE: &str = ":/";

/// What:
///  The `ls-files` arguments that list every path the worktree gives.
/// Why:
///  The wrapper wrote this pathspec,
///  so it must not be read as a name by a caller who set literal pathspecs:
///  `--no-literal-pathspecs` after the caller's options is the last one Git reads,
///  and it wins over the environment variable too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wholeWorktreeQueryArguments = (prefix: string[]) => [...prefix, '--no-literal-pathspecs', 'ls-files', '-z', '--cached', '--others', '--exclude-standard', '--full-name', '--', TOP_OF_THE_WORKTREE];
/// ```
pub fn whole_worktree_query_arguments(global_prefix: &[OsString]) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = global_prefix.to_vec();
    for token in [
        "--no-literal-pathspecs",
        "ls-files",
        "-z",
        "--cached",
        "--others",
        "--exclude-standard",
        "--full-name",
        "--",
        TOP_OF_THE_WORKTREE,
    ] {
        arguments.push(OsString::from(token));
    }
    return arguments;
}

/// What:
///  The question,
///  the decision and the exact text.
#[cfg(test)]
#[path = "rule_add_explicit_tests.rs"]
mod tests;
