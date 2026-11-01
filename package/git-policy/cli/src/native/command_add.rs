//! What:
//!  The complete `git add` option table of Git 2.56.0 and the bulk-staging facts read
//!       from it.
//! Why:
//!  The add-explicit policy rejects staging that names no explicit path.
//!  Git accepts
//!      `-A` inside any cluster (`-vA`),
//!  by abbreviation (`--al`) and through
//!      `--no-ignore-removal`;
//!  a list of exact tokens misses those.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseAddRegion(postSubcommandArgs): AddRegion  (throws OptionError)
//! ```

/// What:
///  Bring the tokenizer,
///  its table builder and its questions into this file.
/// Why:
///   This module only declares Git's table and interprets what the tokenizer found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseOptions, row } from './command_options.ts';
/// ```
use super::command_options::{
    Arity, DEFAULT_MODE, Occurrence, OptionError, OptionSpec, ParsedOptions, UNREAD, parse_options,
    row,
};
use super::command_options_query::{
    WrapperFlags, is_enabled, last_occurrence, positional_tokens, split_wrapper_flags,
};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// Wrapper-only flag that suppresses bulk-add enforcement for one invocation.
pub const ADD_ESCAPE_HATCH: &str = "--no-enforce-bulk-add";

/// `-A`,
///  `--all`.
pub const ALL: u16 = 1;
/// `--ignore-removal`,
///  the same variable as `--no-all` (builtin/add.c:252-259).
pub const IGNORE_REMOVAL: u16 = 2;
/// `-u`,
///  `--update`.
pub const UPDATE: u16 = 3;
/// `--resolved`:
///  add every conflict-resolved tracked file.
pub const RESOLVED: u16 = 4;

/// What:
///  Rows of `builtin_add_options` (builtin/add.c:261-292),
///  in source order.
///       `&[OptionSpec]` is a borrowed table baked into the program.
/// Why:
///   Only `--chmod`,
///  `--pathspec-from-file`,
///  `-U` and `--inter-hunk-context` take a
///       value;
///  every other token after an option is a pathspec.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const ADD_TABLE = [row(UNREAD, 'n', 'dry-run', 'none', true), /* ... */] as const;
/// ```
pub const ADD_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'n'), Some("dry-run"), Arity::None, true),
    row(UNREAD, Some(b'v'), Some("verbose"), Arity::None, true),
    row(UNREAD, Some(b'i'), Some("interactive"), Arity::None, true),
    row(UNREAD, Some(b'p'), Some("patch"), Arity::None, true),
    row(UNREAD, None, Some("auto-advance"), Arity::None, true),
    row(UNREAD, Some(b'U'), Some("unified"), Arity::Required, false),
    row(
        UNREAD,
        None,
        Some("inter-hunk-context"),
        Arity::Required,
        false,
    ),
    row(UNREAD, Some(b'e'), Some("edit"), Arity::None, true),
    row(UNREAD, Some(b'f'), Some("force"), Arity::None, true),
    row(UPDATE, Some(b'u'), Some("update"), Arity::None, true),
    row(UNREAD, None, Some("renormalize"), Arity::None, true),
    row(RESOLVED, None, Some("resolved"), Arity::None, true),
    row(UNREAD, Some(b'N'), Some("intent-to-add"), Arity::None, true),
    row(ALL, Some(b'A'), Some("all"), Arity::None, true),
    row(
        IGNORE_REMOVAL,
        None,
        Some("ignore-removal"),
        Arity::None,
        true,
    ),
    row(UNREAD, None, Some("refresh"), Arity::None, true),
    row(UNREAD, None, Some("ignore-errors"), Arity::None, true),
    row(UNREAD, None, Some("ignore-missing"), Arity::None, true),
    row(UNREAD, None, Some("sparse"), Arity::None, true),
    row(UNREAD, None, Some("chmod"), Arity::Required, true),
    row(UNREAD, None, Some("warn-embedded-repo"), Arity::None, true),
    row(
        UNREAD,
        None,
        Some("pathspec-from-file"),
        Arity::Required,
        true,
    ),
    row(UNREAD, None, Some("pathspec-file-nul"), Arity::None, true),
];

/// Pathspecs that match every changed path under the working directory or repository.
const BULK_PATHSPECS: &[&[u8]] = &[b".", b"./", b"*", b":/"];

/// What:
///  Why a token stages in bulk.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   The policy names the matched tokens;
///  the kind says which Git meaning matched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BulkKind = 'all-flag' | 'update-flag' | 'pathspec';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BulkKind {
    /// `-A`/`--all` is on after every option was applied.
    AllFlag,
    /// `-u`/`--update` is on after every option was applied.
    UpdateFlag,
    /// A positional pathspec equal to `.`,
    ///  `./`,
    ///  `*` or `:/`.
    Pathspec,
}

/// What:
///  One bulk-staging match.
///  `usize` is the list index type.
/// Why:
///   The token index lets the diagnostic quote the caller's exact spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BulkMatch = { kind: BulkKind; token: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BulkMatch {
    /// Which Git meaning matched.
    pub kind: BulkKind,
    /// Region token index of the token that turned the meaning on.
    pub token: usize,
}

/// What:
///  Facts of one `git add` region.
///  `Vec<BulkMatch>` is an owned list in argument order.
/// Why:
///   The add-explicit policy rejects when the list is non-empty and no hatch is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddRegion = { bulkMatches: string[]; hasEscapeHatch: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddRegion {
    /// Bulk-staging matches in argument order.
    pub bulk_matches: Vec<BulkMatch>,
    /// `--resolved` is on.
    ///  Not a bulk match in the shipped policy;
    ///  reported for its owner.
    pub resolved: bool,
    /// Wrapper-only flags in option position;
    ///  `escape` is `--no-enforce-bulk-add`.
    pub wrapper: WrapperFlags,
}

/// What:
///  The token that left `-A` on,
///  or nothing.
///  `Option<usize>` is "an index or nothing".
/// Why:
///   `--all` and `--ignore-removal` write one variable in opposite senses,
///  so they are
///       applied together in argument order (`ignore_removal_cb`,
///  add.c:252-259).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let on; for (const o of occurrences) { if (o.id === ALL) on = o.negated ? undefined : o.token; /* ... */ }
/// ```
fn all_token(parsed: &ParsedOptions) -> Option<usize> {
    let mut on: Option<usize> = None;
    // `for occurrence in &parsed.occurrences` borrows each record in the order Git applied it.
    for occurrence in &parsed.occurrences {
        // `--all` turns it on; `--no-all` turns it off.
        let all_sets: bool = occurrence.id == ALL && !occurrence.negated;
        // `--no-ignore-removal` turns it on; `--ignore-removal` turns it off.
        let removal_sets: bool = occurrence.id == IGNORE_REMOVAL && occurrence.negated;
        if all_sets || removal_sets {
            // `Some(x)` is the "present" case of `Option`.
            on = Some(occurrence.token);
        } else if occurrence.id == ALL || occurrence.id == IGNORE_REMOVAL {
            // `None` is the "absent" case of `Option`.
            on = None;
        }
    }
    return on;
}

/// What:
///  The token that left `-u` on,
///  or nothing.
/// Why:
///   `--update --no-update` leaves it off,
///  as Git's `OPT_BOOL` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const token = last !== undefined && !last.negated ? last.token : undefined;
/// ```
fn update_token(parsed: &ParsedOptions) -> Option<usize> {
    let last: Option<Occurrence> = last_occurrence(parsed, UPDATE);
    // `if let Some(found) = last` runs only when the option was written.
    if let Some(found) = last {
        if found.negated {
            return None;
        }
        return Some(found.token);
    }
    return None;
}

/// What:
///  The sort key of a match:
///  its token index.
///  `&BulkMatch` borrows one match.
/// Why:
///   Matches are reported in the order the caller wrote them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// matches.sort((a, b) => a.token - b.token);
/// ```
fn match_token(found: &BulkMatch) -> usize {
    return found.token;
}

/// What:
///  Parse the region after `add`.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A region Git itself would refuse yields the refusal,
///  never a guessed fact set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseAddRegion(region: string[], wrapperFlags: string[]): AddRegion; // throws
/// ```
pub fn parse_add_region(
    region: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<AddRegion, OptionError> {
    // The command's own escape hatch is flag 0; the caller's flags follow.
    let mut flags: Vec<&[u8]> = Vec::<&[u8]>::with_capacity(wrapper_flags.len() + 1);
    flags.push(ADD_ESCAPE_HATCH.as_bytes());
    for flag in wrapper_flags {
        // `*flag` reads the `&[u8]` the loop variable points at.
        flags.push(*flag);
    }
    // `?` returns Git's refusal to our caller, or unwraps the tokenized region.
    let parsed: ParsedOptions = parse_options(region, ADD_TABLE, DEFAULT_MODE, flags.as_slice())?;
    let mut bulk_matches: Vec<BulkMatch> = Vec::<BulkMatch>::new();
    // `if let Some(token) = ...` runs only when `-A` is on.
    if let Some(token) = all_token(&parsed) {
        bulk_matches.push(BulkMatch {
            kind: BulkKind::AllFlag,
            token,
        });
    }
    if let Some(token) = update_token(&parsed) {
        bulk_matches.push(BulkMatch {
            kind: BulkKind::UpdateFlag,
            token,
        });
    }
    for token in positional_tokens(&parsed, region.len()) {
        // `.as_encoded_bytes()` lends the raw bytes; nothing is decoded or copied.
        if BULK_PATHSPECS.contains(&region[token].as_encoded_bytes()) {
            bulk_matches.push(BulkMatch {
                kind: BulkKind::Pathspec,
                token,
            });
        }
    }
    // `sort_by_key` orders the list by the value `match_token` returns for each item.
    bulk_matches.sort_by_key(match_token);
    // `Ok(x)` is the success case of `Result`.
    return Ok(AddRegion {
        bulk_matches,
        resolved: is_enabled(&parsed, RESOLVED),
        wrapper: split_wrapper_flags(parsed.wrapper.as_slice()),
    });
}

/// Bulk matches in every accepted spelling,
///  and real-Git controls of the table.
#[cfg(test)]
#[path = "command_add_tests.rs"]
mod tests;
