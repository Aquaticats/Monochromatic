//! What:
//!  The complete `git reset` option table of Git 2.56.0 and the mode read from it.
//! Why:
//!  The linked-worktree policy guards resets that rewrite worktree files:
//!  `--hard`,
//!      `--merge` and `--keep`.
//!  The five mode options write one variable,
//!  so only the
//!      last one counts,
//!  in any abbreviation Git accepts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // resetChangesWorktree(parseResetRegion(['--har', 'HEAD~1'])) === true
//! ```

use super::command_options::{
    Arity, DEFAULT_MODE, OptionError, OptionSpec, ParsedOptions, UNREAD, parse_options, row,
};
use super::command_options_query::{WrapperFlags, split_wrapper_flags};
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

/// `--mixed`.
pub const MIXED: u16 = 1;
/// `--soft`.
pub const SOFT: u16 = 2;
/// `--hard`.
pub const HARD: u16 = 3;
/// `--merge`.
pub const MERGE: u16 = 4;
/// `--keep`.
pub const KEEP: u16 = 5;

/// What:
///  Rows of `options[]` in `cmd_reset` (builtin/reset.c:350-382),
///  in source order.
///       `&[OptionSpec]` is a borrowed table baked into the program.
/// Why:
///   The five modes are `PARSE_OPT_NONEG`;
///  `--recurse-submodules` takes an optional
///       value,
///  so a token after it is a revision or path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const RESET_TABLE = [row(UNREAD, 'q', 'quiet', 'none', true), /* ... */] as const;
/// ```
pub const RESET_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(UNREAD, None, Some("no-refresh"), Arity::None, true),
    row(MIXED, None, Some("mixed"), Arity::None, false),
    row(SOFT, None, Some("soft"), Arity::None, false),
    row(HARD, None, Some("hard"), Arity::None, false),
    row(MERGE, None, Some("merge"), Arity::None, false),
    row(KEEP, None, Some("keep"), Arity::None, false),
    row(
        UNREAD,
        None,
        Some("recurse-submodules"),
        Arity::Optional,
        true,
    ),
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
    row(UNREAD, Some(b'N'), Some("intent-to-add"), Arity::None, true),
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
///  The reset mode Git ends up with.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   `Hard`,
///  `Merge` and `Keep` rewrite worktree files;
///  `Mixed` and `Soft` do not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ResetMode = 'mixed' | 'soft' | 'hard' | 'merge' | 'keep';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResetMode {
    /// Reset `HEAD` and the index.
    Mixed,
    /// Reset only `HEAD`.
    Soft,
    /// Reset `HEAD`,
    ///  the index and the working tree.
    Hard,
    /// Reset `HEAD`,
    ///  the index and the working tree,
    ///  keeping unmerged local changes.
    Merge,
    /// Reset `HEAD` but keep local changes.
    Keep,
}

/// What:
///  Facts of one `git reset` region.
///  `Option<ResetMode>` is "a mode or none stated".
/// Why:
///   With no mode option Git uses mixed,
///  which leaves worktree files alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ResetRegion = { hasDestructiveMode: boolean; hasEscapeHatch: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResetRegion {
    /// The last mode option written,
    ///  when any was.
    pub mode: Option<ResetMode>,
    /// Wrapper-only flags in option position;
    ///  `escape` is `--no-enforce-worktree`.
    pub wrapper: WrapperFlags,
}

/// What:
///  The mode a table identifier stands for,
///  or nothing for other rows.
/// Why:
///   One loop over the occurrences can then keep the last mode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mode = { [MIXED]: 'mixed', [SOFT]: 'soft', /* ... */ }[id];
/// ```
fn mode_of(id: u16) -> Option<ResetMode> {
    if id == MIXED {
        // `Some(x)` is the "present" case of `Option`.
        return Some(ResetMode::Mixed);
    }
    if id == SOFT {
        return Some(ResetMode::Soft);
    }
    if id == HARD {
        return Some(ResetMode::Hard);
    }
    if id == MERGE {
        return Some(ResetMode::Merge);
    }
    if id == KEEP {
        return Some(ResetMode::Keep);
    }
    // `None` is the "absent" case of `Option`.
    return None;
}

/// What:
///  Parse the region after `reset`.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   A region Git itself would refuse yields the refusal;
///  Git then resets nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseResetRegion(region: string[], wrapperFlags: string[]): ResetRegion; // throws
/// ```
pub fn parse_reset_region(
    region: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<ResetRegion, OptionError> {
    // The command's own escape hatch is flag 0; the caller's flags follow.
    let mut flags: Vec<&[u8]> = Vec::<&[u8]>::with_capacity(wrapper_flags.len() + 1);
    flags.push(WORKTREE_ENFORCEMENT_ESCAPE_HATCH.as_bytes());
    for flag in wrapper_flags {
        // `*flag` reads the `&[u8]` the loop variable points at.
        flags.push(*flag);
    }
    // `?` returns Git's refusal to our caller, or unwraps the tokenized region.
    let parsed: ParsedOptions = parse_options(region, RESET_TABLE, DEFAULT_MODE, flags.as_slice())?;
    let mut mode: Option<ResetMode> = None;
    // `for occurrence in &parsed.occurrences` borrows each record in the order Git applied it.
    for occurrence in &parsed.occurrences {
        let stated: Option<ResetMode> = mode_of(occurrence.id);
        if stated.is_some() {
            mode = stated;
        }
    }
    // `Ok(x)` is the success case of `Result`.
    return Ok(ResetRegion {
        mode,
        wrapper: split_wrapper_flags(parsed.wrapper.as_slice()),
    });
}

/// What:
///  Whether the reset can rewrite worktree files.
///  `&ResetRegion` borrows the facts.
/// Why:
///   Only these three modes set `opts.update`,
///  which lets Git write worktree files
///       (`reset_index`,
///  builtin/reset.c:79-95).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resetChangesWorktree(region: ResetRegion): boolean;
/// ```
pub fn reset_changes_worktree(region: &ResetRegion) -> bool {
    return region.mode == Some(ResetMode::Hard)
        || region.mode == Some(ResetMode::Merge)
        || region.mode == Some(ResetMode::Keep);
}

/// Modes in every accepted spelling,
///  and real-Git controls of the table and of last-wins.
#[cfg(test)]
#[path = "command_reset_tests.rs"]
mod tests;
