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
use crate::command_options::{
    Arity, DEFAULT_MODE, Occurrence, OptionError, OptionSpec, ParsedOptions, UNREAD, parse_options,
    row,
};
use crate::command_options_query::{
    WrapperFlags, is_enabled, last_occurrence, positional_tokens, split_wrapper_flags,
};
/// `OsStr` is borrowed operating-system text of raw bytes,
/// which an environment value is.
use std::ffi::OsStr;
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

/// Pathspecs Git resolves to the whole worktree,
///       or to every changed path below the working directory.
/// Why:
///   Pathspec grammar has magic and globs,
/// so a fixed list cannot be complete;
///  it names every
///       spelling Git 2.56.0 resolves to the whole tree that a person is likely to write.
///  `:(top)/*`
///       selects the same set but is absent because Git 2.56.0 fails in its own `prep_exclude`
/// for it,
/// so it never reaches this list.
const BULK_PATHSPECS: &[&[u8]] = &[
    // The working directory, absolute and with a trailing slash.
    b".",
    b"./",
    // A bare star matches across `/`, so it reaches every depth.
    b"*",
    // The top of the worktree with no pattern, then with a pattern the default glob reads across `/`.
    b":/",
    b":/*",
    b":/**",
    // Pathspec magic with no pattern names the place it anchors at; the top is the whole repository.
    b":(top)",
    // A bare double star, then the same pattern under explicit glob magic.
    b"**",
    b":(glob)**",
];

/// The variable that makes Git read every pathspec of the caller's own command literally.
pub const LITERAL_VARIABLE: &str = "GIT_LITERAL_PATHSPECS";

/// The global options that set that mode,
/// in the order the caller wrote them.
const LITERAL_OPTIONS: [&[u8]; 2] = [b"--literal-pathspecs", b"--no-literal-pathspecs"];

/// What:
///  The spellings that still stage the whole working directory when pathspec magic is literal.
/// Why:
///   Literal mode turns `*`,
/// `**` and every `:` form into a plain name,
///  but a pathspec that names a directory takes everything below it in any mode,
///  and `.` and `./` name the working directory itself.
///  Measured against Git 2.56.0:
/// `git --literal-pathspecs add -- .` stages every file of the tree.
const WORKTREE_ROOT_DIRECTORY_PATHSPECS: &[&[u8]] = &[b".", b"./"];

/// What:
///  Whether one positional token stages in bulk for a caller whose pathspec mode is `literal`.
/// Why:
///  The policy judges what a token selects,
/// so the same spelling is bulk for one caller and one literal file name for another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stagesInBulk = (spelling: Uint8Array, literal: boolean) => BULK_PATHSPECS.some((bulk) => equal(bulk, spelling)) && (!literal || WORKTREE_ROOT_DIRECTORY_PATHSPECS.some((dir) => equal(dir, spelling)));
/// ```
pub fn stages_in_bulk(spelling: &[u8], literal: bool) -> bool {
    if !BULK_PATHSPECS.contains(&spelling) {
        return false;
    }
    return !literal || WORKTREE_ROOT_DIRECTORY_PATHSPECS.contains(&spelling);
}

/// What:
///  Whether the caller's own command reads pathspecs literally:
/// the last of `--literal-pathspecs` and `--no-literal-pathspecs` among its global options,
/// and otherwise the variable's own boolean reading.
/// Why:
///  Git applies both,
/// and the policy must read the same mode Git will,
/// or it rejects a literal file name or lets a whole-tree spelling through.
///  A value Git would refuse to read as a boolean counts as unset,
/// so a spelling this reading does not model keeps the rejection instead of lifting it.
///   Only the tokens Git reads as options count:
///  in `git -C --literal-pathspecs add -- '**'` that word is the directory `-C` changes to,
///  and Git sets no mode at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const literalPathspecs = (globalPrefix: string[], variable?: string) => last(optionTokens(globalPrefix).filter((o) => LITERAL_OPTIONS.includes(o))) === '--literal-pathspecs' || (variable !== undefined && parseBool(variable) === true);
/// ```
pub fn literal_pathspecs_enabled(global_prefix: &[OsString], variable: Option<&OsStr>) -> bool {
    let mut from_options: Option<bool> = None;
    for option in crate::global_arguments::option_tokens(global_prefix) {
        let spelling: &[u8] = option.as_encoded_bytes();
        if spelling == LITERAL_OPTIONS[0] {
            from_options = Some(true);
        } else if spelling == LITERAL_OPTIONS[1] {
            from_options = Some(false);
        }
    }
    if let Some(literal) = from_options {
        return literal;
    }
    return variable.is_some_and(read_as_true);
}

/// What:
///  Whether one variable value is one of Git's true boolean spellings,
/// read by the configuration reader that already models that grammar.
/// Why:
///  A named function keeps the check out of a closure the linter cannot keep.
fn read_as_true(value: &OsStr) -> bool {
    return crate::repo_read_config_text::parse_bool(Some(value.as_encoded_bytes())) == Some(true);
}

/// What:
///  The caller's global options,
/// the tokens before the subcommand word.
/// Why:
///  The pathspec mode is set there and nowhere else;
/// taking the split from `global_layout` keeps this reading equal to the one every other rule uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const callerGlobalPrefix = (args: string[]) => args.slice(0, globalLayout(args).prefixLen);
/// ```
pub fn caller_global_prefix(arguments: &[OsString]) -> &[OsString] {
    let prefix_len: usize = crate::global_arguments::global_layout(arguments).prefix_len;
    return arguments.get(..prefix_len).unwrap_or_default();
}

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
    /// A positional pathspec naming the whole worktree,
    ///  or every changed path below the working directory:
    ///  the `BULK_PATHSPECS` spellings.
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
///  The bulk matches a caller's own pathspec mode leaves standing:
/// every flag match,
/// and a pathspec match only while its spelling still selects the whole worktree.
/// Why:
///  The policy rejects on this list,
/// and what a spelling selects depends on the mode its caller set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bulkMatchesUnder = (region, matches, literal) => matches.filter((m) => m.kind !== 'pathspec' || stagesInBulk(region[m.token], literal));
/// ```
pub fn bulk_matches_under(region: &[OsString], matches: &[BulkMatch], literal: bool) -> Vec<BulkMatch> {
    let mut kept: Vec<BulkMatch> = Vec::with_capacity(matches.len());
    for found in matches {
        if found.kind != BulkKind::Pathspec || stages_in_bulk(region[found.token].as_encoded_bytes(), literal) {
            kept.push(*found);
        }
    }
    return kept;
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
