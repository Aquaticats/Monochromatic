//! What:
//!  Which `git stash` subcommand a region runs in Git 2.56.0,
//!  and where wrapper-only
//!       flags sit in it.
//! Why:
//!  The linked-worktree policy guards stash;
//!  its escape hatch must be found in option
//!      position of the subcommand that actually reads the arguments,
//!  so a message or a
//!      path that spells the hatch is forwarded untouched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseStashRegion(['push', '-m', '--no-enforce-worktree']).hasEscapeHatch === false
//! ```

use super::command_options::{
    OptionError, OptionErrorKind, OptionSpec, ParseMode, ParsedOptions, WrapperOccurrence,
    parse_options,
};
use super::command_options_query::{WrapperFlags, split_wrapper_flags};
use super::command_stash_table::{
    STASH_APPLY_TABLE, STASH_DROP_TABLE, STASH_EMPTY_TABLE, STASH_EXPORT_TABLE,
    STASH_KEEP_UNKNOWN_MODE, STASH_PLAIN_MODE, STASH_POP_TABLE, STASH_PUSH_TABLE, STASH_SAVE_TABLE,
    STASH_SHOW_TABLE, STASH_STOP_MODE, STASH_STORE_TABLE,
};
/// What:
///  Bring the shared hatch spelling,
///  the tokenizer,
///  its questions and the stash tables
///       into this file.
/// Why:
///   This module only dispatches as `cmd_stash` does and interprets the tokenizer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseOptions } from './command_options.ts';
/// ```
use super::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  The subcommand `cmd_stash` dispatches to (builtin/stash.c:2465-2479).
///  An `enum` is
///       a closed set of named alternatives.
/// Why:
///   `AssumedPush` is the form without a subcommand word,
///  which Git parses as `push`
///       with stricter flags (2515-2528).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StashSubcommand = 'apply' | 'clear' | /* ... */ | 'assumed-push';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StashSubcommand {
    /// `git stash apply`.
    Apply,
    /// `git stash clear`.
    Clear,
    /// `git stash drop`.
    Drop,
    /// `git stash pop`.
    Pop,
    /// `git stash branch`.
    Branch,
    /// `git stash list`.
    List,
    /// `git stash show`.
    Show,
    /// `git stash store`.
    Store,
    /// `git stash create`:
    ///  every argument is message text.
    Create,
    /// `git stash push`.
    Push,
    /// `git stash export`.
    Export,
    /// `git stash import`.
    Import,
    /// `git stash save`,
    ///  the legacy spelling of `push`.
    Save,
    /// `git stash` with no subcommand word.
    AssumedPush,
}

/// What:
///  Facts of one `git stash` region.
/// Why:
///   The shipped policy guards every stash form;
///  the subcommand is reported for its
///       diagnostics and the hatch for its bypass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StashRegion = { hasEscapeHatch: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StashRegion {
    /// The subcommand that reads the arguments.
    pub subcommand: StashSubcommand,
    /// Wrapper-only flags in option position;
    ///  `escape` is `--no-enforce-worktree`.
    pub wrapper: WrapperFlags,
}

/// Subcommand words in `cmd_stash` table order,
///  each with the subcommand it selects.
const SUBCOMMAND_WORDS: &[(&str, StashSubcommand)] = &[
    ("apply", StashSubcommand::Apply),
    ("clear", StashSubcommand::Clear),
    ("drop", StashSubcommand::Drop),
    ("pop", StashSubcommand::Pop),
    ("branch", StashSubcommand::Branch),
    ("list", StashSubcommand::List),
    ("show", StashSubcommand::Show),
    ("store", StashSubcommand::Store),
    ("create", StashSubcommand::Create),
    ("push", StashSubcommand::Push),
    ("export", StashSubcommand::Export),
    ("import", StashSubcommand::Import),
    ("save", StashSubcommand::Save),
];

/// What:
///  The subcommand a word selects,
///  or nothing.
///  `&[u8]` borrows the word's bytes.
/// Why:
///   Git compares the whole word exactly (`parse_subcommand`,
///  parse-options.c:609-619).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subcommand = SUBCOMMAND_WORDS.find(([word]) => word === token)?.[1];
/// ```
fn subcommand_of(word: &[u8]) -> Option<StashSubcommand> {
    // `for (name, subcommand) in SUBCOMMAND_WORDS` unpacks each pair of the table.
    for (name, subcommand) in SUBCOMMAND_WORDS {
        if name.as_bytes() == word {
            // `*subcommand` copies the small enum value out of the borrowed table.
            return Some(*subcommand);
        }
    }
    return None;
}

/// What:
///  The table and flags a subcommand parses its arguments with.
///       `(&'static [OptionSpec], ParseMode)` is a pair of a baked-in table and its mode.
/// Why:
///   Each `*_stash` function calls `parse_options` with its own table and flags.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [table, mode] = GRAMMAR[subcommand];
/// ```
fn grammar_of(subcommand: StashSubcommand) -> (&'static [OptionSpec], ParseMode) {
    // `match` picks one arm per variant; the compiler refuses a forgotten variant.
    return match subcommand {
        StashSubcommand::Apply => (STASH_APPLY_TABLE, STASH_PLAIN_MODE),
        StashSubcommand::Clear => (STASH_EMPTY_TABLE, STASH_STOP_MODE),
        StashSubcommand::Drop => (STASH_DROP_TABLE, STASH_PLAIN_MODE),
        StashSubcommand::Pop => (STASH_POP_TABLE, STASH_PLAIN_MODE),
        StashSubcommand::Branch => (STASH_EMPTY_TABLE, STASH_PLAIN_MODE),
        StashSubcommand::List => (STASH_EMPTY_TABLE, STASH_KEEP_UNKNOWN_MODE),
        StashSubcommand::Show => (STASH_SHOW_TABLE, STASH_KEEP_UNKNOWN_MODE),
        StashSubcommand::Store => (STASH_STORE_TABLE, STASH_KEEP_UNKNOWN_MODE),
        // `create` joins every argument into the message without parsing options
        // (builtin/stash.c:1654-1655); the empty stopping grammar is never used for it.
        StashSubcommand::Create => (STASH_EMPTY_TABLE, STASH_STOP_MODE),
        StashSubcommand::Push => (STASH_PUSH_TABLE, STASH_PLAIN_MODE),
        StashSubcommand::Export => (STASH_EXPORT_TABLE, STASH_PLAIN_MODE),
        StashSubcommand::Import => (STASH_EMPTY_TABLE, STASH_PLAIN_MODE),
        StashSubcommand::Save => (STASH_SAVE_TABLE, STASH_PLAIN_MODE),
        StashSubcommand::AssumedPush => (STASH_PUSH_TABLE, STASH_STOP_MODE),
    };
}

/// What:
///  Refuse a first token that spells a subcommand word after one dash.
///       `Result<(), OptionError>` is "nothing on success,
///  or a refusal".
/// Why:
///   Before assuming `push`,
///  Git's top-level pass applies the typo check against the
///       subcommand words,
///  so `git stash -push` is an error (parse-options.c:622-640,
///       1067-1072,
///  1127-1131).
/// Gotcha:
///  That pass also refuses `-no-<x>` and shows usage for `--help` and for a cluster
///         starting with `h`.
///  The assumed-push pass that follows refuses those tokens with
///         the same kind at the same index,
///  so they need no check here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkTopLevelToken(token: string, index: number): void; // throws OptionError
/// ```
fn check_top_level_token(token: &[u8], index: usize) -> Result<(), OptionError> {
    // Git checks clusters of at least three letters: one dash, then three or more bytes.
    if token.len() < 4 || token[0] != b'-' || token[1] == b'-' {
        return Ok(());
    }
    // `&token[1..]` borrows the letters after the dash.
    let cluster: &[u8] = &token[1..];
    for (name, _) in SUBCOMMAND_WORDS {
        if name.as_bytes().starts_with(cluster) {
            return Err(OptionError {
                kind: OptionErrorKind::SingleDashLongOption,
                token: index,
            });
        }
    }
    return Ok(());
}

/// What:
///  Parse the region after `stash`.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   Leading wrapper flags are skipped first,
///  so `git stash --no-enforce-worktree list`
///       still dispatches to `list`;
///  the rest follows the selected subcommand's grammar.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseStashRegion(region: string[], wrapperFlags: string[]): StashRegion; // throws
/// ```
pub fn parse_stash_region(
    region: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<StashRegion, OptionError> {
    // The command's own escape hatch is flag 0; the caller's flags follow.
    let mut flags: Vec<&[u8]> = Vec::<&[u8]>::with_capacity(wrapper_flags.len() + 1);
    flags.push(WORKTREE_ENFORCEMENT_ESCAPE_HATCH.as_bytes());
    for flag in wrapper_flags {
        // `*flag` reads the `&[u8]` the loop variable points at.
        flags.push(*flag);
    }
    let mut wrapper: Vec<WrapperOccurrence> = Vec::<WrapperOccurrence>::new();
    // Skip wrapper flags written before the subcommand word.
    let mut first: usize = 0;
    while first < region.len() {
        // `.as_encoded_bytes()` lends the raw bytes; nothing is decoded or copied.
        let token: &[u8] = region[first].as_encoded_bytes();
        // Index of the wrapper flag this token spells; the list length means "none".
        let mut flag: usize = 0;
        while flag < flags.len() && flags[flag] != token {
            flag += 1;
        }
        if flag == flags.len() {
            break;
        }
        wrapper.push(WrapperOccurrence { flag, token: first });
        first += 1;
    }
    let mut subcommand: StashSubcommand = StashSubcommand::AssumedPush;
    // Index of the first token the subcommand's own parser reads.
    let mut start: usize = first;
    if first < region.len() {
        let word: &[u8] = region[first].as_encoded_bytes();
        if let Some(selected) = subcommand_of(word) {
            subcommand = selected;
            start = first + 1;
        } else {
            // `?` returns the refusal to our caller, or continues on success.
            check_top_level_token(word, first)?;
        }
    }
    if subcommand != StashSubcommand::Create {
        let (table, mode): (&[OptionSpec], ParseMode) = grammar_of(subcommand);
        // What: `match` on a `Result` unpacks the success value or handles the failure.
        // Why:  The refusal's token index is relative to the subcommand's arguments; it is
        //       shifted so it points into the whole region.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { parsed = parseOptions(...); } catch (e) { e.token += start; throw e; }
        // ```
        let parsed: ParsedOptions =
            match parse_options(&region[start..], table, mode, flags.as_slice()) {
                Ok(accepted) => accepted,
                Err(error) => {
                    return Err(OptionError {
                        kind: error.kind,
                        token: error.token + start,
                    });
                }
            };
        for occurrence in &parsed.wrapper {
            wrapper.push(WrapperOccurrence {
                flag: occurrence.flag,
                token: occurrence.token + start,
            });
        }
    }
    // `Ok(x)` is the success case of `Result`.
    return Ok(StashRegion {
        subcommand,
        wrapper: split_wrapper_flags(wrapper.as_slice()),
    });
}

/// What:
///  Whether the subcommand can change the worktree, the index or the stash list.
/// Why:
///   `list`, `show` and `create` only read: `create` writes a commit object and stores nothing, so an
///  inspection is not blocked like `drop`.
///   Every other subcommand, and the form with no subcommand word, writes at least one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stashChangesState(subcommand: StashSubcommand): boolean;
/// ```
pub fn stash_changes_state(subcommand: StashSubcommand) -> bool {
    return !matches!(
        subcommand,
        StashSubcommand::List | StashSubcommand::Show | StashSubcommand::Create
    );
}

/// Dispatch,
///  per-subcommand value positions,
///  and real-Git controls of the tables.
#[cfg(test)]
#[path = "command_stash_tests.rs"]
mod tests;
