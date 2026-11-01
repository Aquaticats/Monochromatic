//! What:
//!  Facts about the arguments after `git commit`,
//!  read as Git 2.56.0 reads them.
//! Why:
//!  The commit-only transform,
//!  the dry-run check and the commit transaction all decide
//!      from these facts.
//!  Every fact is the final state after Git applied each option in
//!      order,
//!  so `--all --no-all` reports `all` off,
//!  exactly as `git commit` would act.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseCommitRegion(postSubcommandArgs): CommitRegion  (throws OptionError)
//! ```

/// What:
///  Bring the commit table,
///  the tokenizer and its questions into this file.
/// Why:
///   This module only interprets what the shared tokenizer found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { COMMIT_TABLE, ALL, ONLY /* ... */ } from './command_commit_table.ts';
/// import { parseOptions } from './command_options.ts';
/// ```
use super::command_commit_table::{
    ALL, ALLOW_EMPTY, AMEND, COMMIT_TABLE, DRY_RUN, FIXUP, INCLUDE, INTERACTIVE, LONG, NULL, ONLY,
    PATCH, PATHSPEC_FILE_NUL, PATHSPEC_FROM_FILE, PORCELAIN, SHORT,
};
use super::command_options::{
    DEFAULT_MODE, Occurrence, OptionError, OptionValue, ParsedOptions, parse_options,
};
use super::command_options_query::{
    WrapperFlags, is_enabled, last_occurrence, positional_tokens, split_wrapper_flags, value_bytes,
};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// Wrapper-only flag that suppresses `-o` injection for one commit invocation.
pub const COMMIT_ESCAPE_HATCH: &str = "--no-enforce-only";

/// What:
///  The form of a `--fixup` value (builtin/commit.c:1378-1411).
///  An `enum` is a closed
///       set of named alternatives.
/// Why:
///   `amend:` and `reword:` turn on `--allow-empty`;
///  `reword:` also turns on `--only`
///       itself and refuses paths and an explicit `-o`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FixupKind = 'plain' | 'amend' | 'reword' | 'unknown-suboption';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixupKind {
    /// `--fixup=<commit>`.
    Plain,
    /// `--fixup=amend:<commit>`.
    Amend,
    /// `--fixup=reword:<commit>`.
    Reword,
    /// Letters then `:` that are neither `amend` nor `reword`;
    ///  Git dies on these.
    UnknownSuboption,
}

/// What:
///  Final-state facts of one `git commit` region.
///  `Vec<usize>` is an owned list of
///       token indexes;
///  `Option<bool>` is "true,
///  false,
///  or never stated".
/// Why:
///   Indexes point into the caller's unchanged arguments,
///  so paths keep their bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitRegion = { all: boolean; only?: boolean; include: boolean; /* ... */ pathspecs: number[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitRegion {
    /// `-a`/`--all` is on.
    pub all: bool,
    /// `Some(true)` after `-o`/`--only`,
    ///  `Some(false)` after `--no-only`,
    ///  `None` if unstated.
    pub only: Option<bool>,
    /// `-i`/`--include` is on.
    pub include: bool,
    /// `--interactive` is on.
    pub interactive: bool,
    /// `-p`/`--patch` is on.
    pub patch: bool,
    /// `--dry-run`,
    ///  a status format,
    ///  or `-z` makes Git record no commit (commit.c:1422-1423).
    pub dry_run: bool,
    /// `--amend` is on.
    pub amend: bool,
    /// `--allow-empty` is on.
    pub allow_empty: bool,
    /// Form of the final `--fixup` value,
    ///  when one is set.
    pub fixup: Option<FixupKind>,
    /// Position of the final `--pathspec-from-file` value,
    ///  when one is set.
    pub pathspec_from_file: Option<OptionValue>,
    /// `--pathspec-file-nul` is on.
    pub pathspec_file_nul: bool,
    /// Region token indexes of positional pathspecs,
    ///  before and after `--`.
    pub pathspecs: Vec<usize>,
    /// Wrapper-only flags in option position;
    ///  `escape` is `--no-enforce-only`.
    pub wrapper: WrapperFlags,
}

/// What:
///  Classify a `--fixup` value by its leading letters.
///  `&[u8]` borrows the value bytes.
/// Why:
///   Git treats `<letters>:` as a suboption and anything else as a commit name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const match = /^([A-Za-z]+):/.exec(value); // then compare match[1]
/// ```
fn fixup_kind(value: &[u8]) -> FixupKind {
    let mut letters: usize = 0;
    // `is_ascii_alphabetic` matches C `isalpha` in Git's locale-independent ctype table.
    while letters < value.len() && value[letters].is_ascii_alphabetic() {
        letters += 1;
    }
    if letters == 0 || letters == value.len() || value[letters] != b':' {
        return FixupKind::Plain;
    }
    // `&value[..letters]` borrows the bytes before the colon.
    let suboption: &[u8] = &value[..letters];
    if suboption == b"amend" {
        return FixupKind::Amend;
    }
    if suboption == b"reword" {
        return FixupKind::Reword;
    }
    return FixupKind::UnknownSuboption;
}

/// What:
///  Whether the options leave Git with a status format,
///  which forces a dry run.
/// Why:
///   `--short`,
///  `--porcelain` and `--long` write one shared variable,
///  so only the last
///       of them counts and a negated form clears it (commit.c:1752-1761).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let format = false; for (const o of occurrences) if (isFormat(o.id)) format = !o.negated;
/// ```
fn has_status_format(parsed: &ParsedOptions) -> bool {
    let mut format: bool = false;
    // `for occurrence in &parsed.occurrences` borrows each record in the order Git applied it.
    for occurrence in &parsed.occurrences {
        if occurrence.id == SHORT || occurrence.id == PORCELAIN || occurrence.id == LONG {
            format = !occurrence.negated;
        }
    }
    return format;
}

/// What:
///  The value of an option's final use,
///  unless that use negated it.
/// Why:
///   `--no-pathspec-from-file` after a value clears it,
///  as Git's `OPTION_FILENAME` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = last === undefined || last.negated ? undefined : last.value;
/// ```
fn final_value(parsed: &ParsedOptions, id: u16) -> Option<OptionValue> {
    let last: Option<Occurrence> = last_occurrence(parsed, id);
    // `if let Some(found) = last` runs only when the option was written.
    if let Some(found) = last {
        if found.negated {
            // `None` is the "absent" case of `Option`.
            return None;
        }
        return found.value;
    }
    return None;
}

/// What:
///  Parse the region after `commit`.
///  `wrapper_flags` lists the caller's other
///       wrapper-only spellings;
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A region Git itself would refuse yields the refusal,
///  never a guessed fact set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCommitRegion(region: string[], wrapperFlags: string[]): CommitRegion; // throws
/// ```
pub fn parse_commit_region(
    region: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<CommitRegion, OptionError> {
    // The command's own escape hatch is flag 0; the caller's flags follow.
    let mut flags: Vec<&[u8]> = Vec::<&[u8]>::with_capacity(wrapper_flags.len() + 1);
    flags.push(COMMIT_ESCAPE_HATCH.as_bytes());
    for flag in wrapper_flags {
        // `*flag` reads the `&[u8]` the loop variable points at.
        flags.push(*flag);
    }
    // `?` returns Git's refusal to our caller, or unwraps the tokenized region.
    let parsed: ParsedOptions =
        parse_options(region, COMMIT_TABLE, DEFAULT_MODE, flags.as_slice())?;
    // `mut` lets the default "never stated" be replaced when the option was written.
    let mut only: Option<bool> = None;
    if let Some(stated) = last_occurrence(&parsed, ONLY) {
        // `Some(x)` is the "present" case of `Option`.
        only = Some(!stated.negated);
    }
    let mut fixup: Option<FixupKind> = None;
    if let Some(value) = final_value(&parsed, FIXUP) {
        fixup = Some(fixup_kind(value_bytes(region, value)));
    }
    // `Ok(x)` is the success case of `Result`.
    return Ok(CommitRegion {
        all: is_enabled(&parsed, ALL),
        only,
        include: is_enabled(&parsed, INCLUDE),
        interactive: is_enabled(&parsed, INTERACTIVE),
        patch: is_enabled(&parsed, PATCH),
        dry_run: is_enabled(&parsed, DRY_RUN)
            || has_status_format(&parsed)
            || is_enabled(&parsed, NULL),
        amend: is_enabled(&parsed, AMEND),
        allow_empty: is_enabled(&parsed, ALLOW_EMPTY),
        fixup,
        pathspec_from_file: final_value(&parsed, PATHSPEC_FROM_FILE),
        pathspec_file_nul: is_enabled(&parsed, PATHSPEC_FILE_NUL),
        pathspecs: positional_tokens(&parsed, region.len()),
        wrapper: split_wrapper_flags(parsed.wrapper.as_slice()),
    });
}

/// Ported incumbent cases and the readings where Git 2.56.0 differs from the incumbent.
#[cfg(test)]
#[path = "command_commit_tests.rs"]
mod tests;

/// Real Git 2.56.0 controls for the table and for each divergence.
#[cfg(test)]
#[path = "command_commit_git_tests.rs"]
mod git_tests;
