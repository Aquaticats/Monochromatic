//! What:
//!  The complete `git clean` option table of Git 2.56.0 and the deletion facts read
//!       from it.
//! Why:
//!  The linked-worktree policy guards a clean that can delete files.
//!  Git's last
//!      `--dry-run`/`--no-dry-run` wins,
//!  in clusters (`-ndX`) and abbreviations (`--dry`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // cleanChangesWorktree(parseCleanRegion(['-ndX'])) === false
//! ```

use super::command_options::{
    Arity, DEFAULT_MODE, OptionError, OptionSpec, ParsedOptions, UNREAD, parse_options, row,
};
use super::command_options_query::{WrapperFlags, is_enabled, split_wrapper_flags};
/// What:
///  Bring the shared hatch spelling,
///  the tokenizer,
///  its table builder and its
///       questions into this file.
/// Why:
///   This module only declares Git's table and interprets what the tokenizer found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseOptions, row } from './command_options.ts';
/// ```
use super::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// `-n`,
///  `--dry-run`.
pub const DRY_RUN: u16 = 1;
/// `-i`,
///  `--interactive`.
pub const INTERACTIVE: u16 = 2;

/// What:
///  Rows of `options[]` in `cmd_clean` (builtin/clean.c:936-948),
///  in source order.
///       `&[OptionSpec]` is a borrowed table baked into the program;
///  `None` in the long
///       position means the option has only a letter.
/// Why:
///   `-e`/`--exclude` takes a value and is `PARSE_OPT_NONEG`;
///  `-d`,
///  `-x` and `-X` have
///       no long spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const CLEAN_TABLE = [row(UNREAD, 'q', 'quiet', 'none', true), /* ... */] as const;
/// ```
pub const CLEAN_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(DRY_RUN, Some(b'n'), Some("dry-run"), Arity::None, true),
    row(UNREAD, Some(b'f'), Some("force"), Arity::None, true),
    row(
        INTERACTIVE,
        Some(b'i'),
        Some("interactive"),
        Arity::None,
        true,
    ),
    row(UNREAD, Some(b'd'), None, Arity::None, true),
    row(UNREAD, Some(b'e'), Some("exclude"), Arity::Required, false),
    row(UNREAD, Some(b'x'), None, Arity::None, true),
    row(UNREAD, Some(b'X'), None, Arity::None, true),
];

/// What:
///  Final-state facts of one `git clean` region.
///  `bool` is true or false.
/// Why:
///   The policy decides from the final dry-run state;
///  `interactive` is reported for
///       its diagnostics.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CleanRegion = { dryRunActive: boolean; interactiveActive: boolean; hasEscapeHatch: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanRegion {
    /// `-n`/`--dry-run` is on after every option was applied.
    pub dry_run: bool,
    /// `-i`/`--interactive` is on after every option was applied.
    pub interactive: bool,
    /// Wrapper-only flags in option position;
    ///  `escape` is `--no-enforce-worktree`.
    pub wrapper: WrapperFlags,
}

/// What:
///  Parse the region after `clean`.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A region Git itself would refuse yields the refusal;
///  Git then deletes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCleanRegion(region: string[], wrapperFlags: string[]): CleanRegion; // throws
/// ```
pub fn parse_clean_region(
    region: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<CleanRegion, OptionError> {
    // The command's own escape hatch is flag 0; the caller's flags follow.
    let mut flags: Vec<&[u8]> = Vec::<&[u8]>::with_capacity(wrapper_flags.len() + 1);
    flags.push(WORKTREE_ENFORCEMENT_ESCAPE_HATCH.as_bytes());
    for flag in wrapper_flags {
        // `*flag` reads the `&[u8]` the loop variable points at.
        flags.push(*flag);
    }
    // `?` returns Git's refusal to our caller, or unwraps the tokenized region.
    let parsed: ParsedOptions = parse_options(region, CLEAN_TABLE, DEFAULT_MODE, flags.as_slice())?;
    // `Ok(x)` is the success case of `Result`.
    return Ok(CleanRegion {
        dry_run: is_enabled(&parsed, DRY_RUN),
        interactive: is_enabled(&parsed, INTERACTIVE),
        wrapper: split_wrapper_flags(parsed.wrapper.as_slice()),
    });
}

/// What:
///  Whether the clean can delete files.
///  `&CleanRegion` borrows the facts.
/// Why:
///   Every `unlink` and `rmdir` in Git's clean is skipped under dry run,
///  also after
///       the interactive menu chose what to delete (builtin/clean.c:191,
///  223,
///  267,
///       1050-1078).
/// Gotcha:
///  Divergence from the incumbent,
///  which treated every interactive clean as
///         deleting:
///  `git clean -i -n` deletes nothing in Git 2.56.0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanChangesWorktree(region: CleanRegion): boolean;
/// ```
pub fn clean_changes_worktree(region: &CleanRegion) -> bool {
    return !region.dry_run;
}

/// Facts in every accepted spelling,
///  and real-Git controls of the table and of dry runs.
#[cfg(test)]
#[path = "command_clean_tests.rs"]
mod tests;
