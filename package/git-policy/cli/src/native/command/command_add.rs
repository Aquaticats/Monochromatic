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

/// What:
///  The characters Git's pathspec matcher reads as wildcards rather than as part of a name.
///  `&[u8]` is borrowed bytes baked into the program.
/// Why:
///  A spelling that holds none of them names one path,
///  so the policy lets it through and starts no process,
///  while every other spelling may reach further than the caller meant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GLOB_CHARACTERS = new Uint8Array([42, 63, 91, 93, 92]); // * ? [ ] \\
/// ```
const GLOB_CHARACTERS: &[u8] = b"*?[]\\";

/// What:
///  Whether one byte separates path segments.
///  `&u8` borrows one byte.
/// Why:
///  A named function keeps the pass free of a closure that captures nothing,
///  and every pass here splits on the same byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSeparator = (byte: number) => byte === 0x2f;
/// ```
fn is_separator(byte: &u8) -> bool {
    return *byte == b'/';
}

/// What:
///  Whether one byte closes the parentheses of longhand pathspec magic.
/// Why:
///  A named function keeps the pass free of a closure that captures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isCloseParenthesis = (byte: number) => byte === 0x29;
/// ```
fn is_close_parenthesis(byte: &u8) -> bool {
    return *byte == b')';
}

/// What:
///  Whether one byte separates two magic parameters.
/// Why:
///  A named function keeps the pass free of a closure that captures nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isComma = (byte: number) => byte === 0x2c;
/// ```
fn is_comma(byte: &u8) -> bool {
    return *byte == b',';
}

/// What:
///  Whether one positional pathspec names one path and nothing else:
/// no pathspec magic,
/// no glob character and no `.` or `..` segment.
/// Why:
///  `git add file.ts` and `git add dir/` stage what their names hold,
///  so the policy lets them through without asking Git what they select,
///  and keeps the guard free of a process for the commands people run most.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isOrdinaryNamedPath = (spelling: Uint8Array) => spelling[0] !== 0x3a && !spelling.some((b) => GLOB_CHARACTERS.includes(b)) && !spelling.split('/').some((s) => s === '.' || s === '..');
/// ```
pub fn is_ordinary_named_path(spelling: &[u8]) -> bool {
    // Magic is a leading colon; a colon anywhere else is part of a name.
    if spelling.first() == Some(&b':') {
        return false;
    }
    // `for byte in spelling` visits every byte, so one wildcard is enough to make it a pattern.
    for byte in spelling {
        if GLOB_CHARACTERS.contains(byte) {
            return false;
        }
    }
    // `split` gives each path segment: the working directory and its parent reach beyond one name.
    for segment in spelling.split(is_separator) {
        if segment == b"." || segment == b".." {
            return false;
        }
    }
    return true;
}

/// What:
///  What one pathspec spelling leaves Git's matcher,
/// read from its magic.
///  A `struct` is a record with named fields.
/// Why:
///  A spelling names no character of a name when its pattern is wildcards only,
///  so one reading answers "is this reach-only" for the shorthand forms,
///  the longhand forms and a caller that reads pathspecs literally.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reading = { pattern: Uint8Array; excludes: boolean; patternIsAName: boolean };
/// ```
struct Reading<'pattern> {
    /// The pattern Git matches after the magic.
    pattern: &'pattern [u8],
    /// The magic negates the pattern:
    ///  everything the pattern leaves is selected.
    excludes: bool,
    /// Pathspec magic makes the pattern a name,
    ///  so no character of it is a wildcard.
    pattern_is_a_name: bool,
}

/// What:
///  Read the magic of one pathspec spelling.
///  `Option<Reading<'_>>` is "the reading,
///  or nothing for a longhand form with no closing parenthesis".
/// Why:
///  Git reads pathspec magic only before the pattern,
///  and the wrapper must read the same grammar:
///  `:` starts it,
///  `/` anchors at the top of the worktree,
///  and `!` and `^` negate the pattern.
///  Git strips a leading colon even when no magic letter follows it,
///  which is why `:**` selects what `**` selects and not a file named `:**`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reading(spelling: Uint8Array, literal: boolean): Reading | undefined;
/// ```
fn reading(spelling: &[u8], literal: bool) -> Option<Reading<'_>> {
    // A caller that reads pathspecs literally has no magic at all: every character is a name's.
    if literal {
        return Some(Reading {
            pattern: spelling,
            excludes: false,
            pattern_is_a_name: true,
        });
    }
    if spelling.first() != Some(&b':') {
        return Some(Reading {
            pattern: spelling,
            excludes: false,
            pattern_is_a_name: false,
        });
    }
    // The longhand form names its magic in a parenthesised list; Git reads up to the first `)`.
    if spelling.get(1) == Some(&b'(') {
        let close: usize = spelling.iter().position(is_close_parenthesis)?;
        let mut excludes: bool = false;
        let mut pattern_is_a_name: bool = false;
        for parameter in spelling[2..close].split(is_comma) {
            excludes = excludes || parameter == b"exclude" || parameter == b"!";
            pattern_is_a_name = pattern_is_a_name || parameter == b"literal";
        }
        return Some(Reading {
            pattern: spelling.get(close + 1..).unwrap_or_default(),
            excludes,
            pattern_is_a_name,
        });
    }
    // The shorthand form is a run of `/`, `!` and `^`, then at most one colon, then the pattern.
    let mut position: usize = 1;
    let mut excludes: bool = false;
    while position < spelling.len() && matches!(spelling[position], b'/' | b'!' | b'^') {
        excludes = excludes || spelling[position] != b'/';
        position += 1;
    }
    if spelling.get(position) == Some(&b':') {
        position += 1;
    }
    return Some(Reading {
        pattern: spelling.get(position..).unwrap_or_default(),
        excludes,
        pattern_is_a_name: false,
    });
}

/// What:
///  The path segments a pattern leaves once `.` and `..` are applied,
/// each as the bytes the pattern wrote.
///  `Vec<&[u8]>` is an owned list of borrowed slices.
/// Why:
///  `sub/..` reaches the working directory and names nothing on the way,
///  so the segments that remain are what a person would call the name in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resolvedSegments = (pattern: Uint8Array) => pattern.split('/').reduce((kept, s) => s === '..' ? kept.slice(0, -1) : s === '.' || s === '' ? kept : [...kept, s], []);
/// ```
fn resolved_segments(pattern: &[u8]) -> Vec<&[u8]> {
    let mut kept: Vec<&[u8]> = Vec::<&[u8]>::new();
    for segment in pattern.split(is_separator) {
        if segment.is_empty() || segment == b"." {
            continue;
        }
        if segment == b".." {
            // The parent segment is left out of the name, exactly as a shell resolves it.
            kept.pop();
            continue;
        }
        kept.push(segment);
    }
    return kept;
}

/// What:
///  Whether one pathspec spelling names no character of a name,
///  so it selects by reach alone and everything it stages is whatever it reaches.
/// Why:
///  A spelling that names a character of a name,
/// such as `*.txt`,
///  stages everything only because the tree happens to fit that name,
///  and refusing it would block a caller who named what they meant.
///  A spelling that names nothing cannot tell one path from another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesNoLiteralCharacter = (spelling, literal) => { const read = reading(spelling, literal); if (!read) return false; if (read.excludes) return true; const segments = resolvedSegments(read.pattern); if (read.patternIsAName) return segments.length === 0; return segments.every((s) => [...s].every((b) => GLOB_CHARACTERS.includes(b))); };
/// ```
pub fn names_no_literal_character(spelling: &[u8], literal: bool) -> bool {
    let Some(read) = reading(spelling, literal) else {
        return false;
    };
    // A negated spelling selects everything its pattern leaves, whatever that pattern names.
    if read.excludes {
        return true;
    }
    let segments: Vec<&[u8]> = resolved_segments(read.pattern);
    // A name's own characters are no wildcards, so only a spelling that resolves to a directory is reach-only.
    if read.pattern_is_a_name {
        return segments.is_empty();
    }
    for segment in segments {
        for byte in segment {
            if !GLOB_CHARACTERS.contains(byte) {
                return false;
            }
        }
    }
    return true;
}

/// The variable that makes Git read every pathspec of the caller's own command literally.
pub const LITERAL_VARIABLE: &str = "GIT_LITERAL_PATHSPECS";

/// The global options that set that mode,
/// in the order the caller wrote them.
const LITERAL_OPTIONS: [&[u8]; 2] = [b"--literal-pathspecs", b"--no-literal-pathspecs"];

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
///  The facts one `git add` region gives about staging in bulk.
///  `Option<usize>` is "the token index,
///  or nothing".
/// Why:
///  `-A` and `-u` stage every changed path whatever the tree holds,
///  so they are rejected without asking Git anything;
///  a pathspec stages what Git says it selects,
///  which only the repository can answer.
///  Every list is in argument order,
///  so the diagnostic quotes the caller's own tokens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AddRegion = { allToken?: number; updateToken?: number; pathspecTokens: number[]; resolved: boolean; wrapper: WrapperFlags };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddRegion {
    /// The token that left `-A`,
    ///  `--all` or `--no-ignore-removal` on.
    pub all_token: Option<usize>,
    /// The token that left `-u` or `--update` on.
    pub update_token: Option<usize>,
    /// Token index of each positional pathspec that is no ordinary named path;
    ///  Git must be asked what it selects.
    pub pathspec_tokens: Vec<usize>,
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
///  Parse the region after `add` for what it says about staging in bulk.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A region Git itself would refuse yields the refusal,
///  never a guessed fact set,
///  and the caller's pathspec mode changes only what a spelling names,
///  which `names_no_literal_character` reads where that matters.
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
    let mut pathspec_tokens: Vec<usize> = Vec::<usize>::new();
    for token in positional_tokens(&parsed, region.len()) {
        // `.as_encoded_bytes()` lends the raw bytes; nothing is decoded or copied.
        // An ordinary name stages what it names, so the policy starts no process for it.
        if is_ordinary_named_path(region[token].as_encoded_bytes()) {
            continue;
        }
        pathspec_tokens.push(token);
    }
    // `Ok(x)` is the success case of `Result`.
    return Ok(AddRegion {
        all_token: all_token(&parsed),
        update_token: update_token(&parsed),
        pathspec_tokens,
        resolved: is_enabled(&parsed, RESOLVED),
        wrapper: split_wrapper_flags(parsed.wrapper.as_slice()),
    });
}

/// Bulk forms in every accepted spelling,
///  and real-Git controls of the table and of the paths that stage everything.
#[cfg(test)]
#[path = "command_add_tests.rs"]
mod tests;
