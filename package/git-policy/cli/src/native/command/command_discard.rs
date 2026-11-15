//! What:
//!  Which regions of `git checkout`,
//! `git switch`,
//! `git restore`,
//! `git rm`,
//! `git mv`,
//! `git checkout-index`,
//!  `git read-tree` and the `--abort` forms of `merge`,
//! `rebase`,
//! `cherry-pick`,
//! `revert` and `am` can overwrite or
//!  delete uncommitted state.
//! Why:
//!  The linked-worktree policy confines such commands:
//! it rejects them in the main worktree and,
//! once the worktree
//!  holds uncommitted state,
//! in a linked worktree too.
//!  Every `git read-tree` form Git accepts writes the index or,
//! with `-u`,
//! worktree files,
//!  so each is read as what it writes.
//! Each region is read from the command's own option table through one tokenizer,
//! so a spelling
//!  Git accepts as `--force` is not missed and a value that looks like a flag is not mistaken for one.
//!  The tables are `command_checkout_table`,
//!  `command_restore_table`,
//!  `command_rm_table`,
//!  `command_mv_table`,
//!  `command_checkout_index_table` and the read-tree table below.
//! Gotcha:
//!  Where Git itself refuses the region the answer is the refusal;
//! the caller lets Git say so.
//!  An option no table holds is different:
//!  it is a word the guard cannot read,
//!  so the caller refuses the command and names it rather than guessing what it writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkoutDiscard(['--', 'a.txt']) === 'overwrites'; checkoutDiscard(['topic']) === 'moves-head'
//! ```

use crate::command_checkout_index_table::{
    CHECKOUT_INDEX_FORCE, CHECKOUT_INDEX_TABLE,
};
use crate::command_checkout_table::{CHECKOUT_TABLE, DISCARD_CHANGES, FORCE, PATCH, PATHSPEC_FROM_FILE, STAGE, SWITCH_TABLE};
use crate::command_mv_table::{MV_DRY_RUN, MV_FORCE, MV_TABLE};
use crate::command_options::{Arity, Boundary, DEFAULT_MODE, OptionError, OptionSpec, ParsedOptions, parse_options, row, UNREAD};
use crate::command_options_query::{is_enabled, is_stated, positional_tokens};
use crate::command_restore_table::{
    RESTORE_IGNORE_SKIP_WORKTREE_BITS, RESTORE_STAGED, RESTORE_TABLE, RESTORE_WORKTREE,
};
use crate::command_rm_table::{RM_CACHED, RM_DRY_RUN, RM_FORCE, RM_SPARSE, RM_TABLE};
use std::ffi::OsString;

/// The `--dry-run` row of `git read-tree`:
/// it writes neither the index nor the worktree.
pub const READ_TREE_DRY_RUN: u16 = 1;
/// The `--debug-unpack` row of `git read-tree`:
/// Git prints the unpacked entries and returns before the index is written.
pub const READ_TREE_DEBUG_UNPACK: u16 = 2;
/// The `-m` row of `git read-tree`:
/// the merge that may write worktree files and, with one tree, reads the index against it.
pub const READ_TREE_MERGE: u16 = 3;
/// The `--reset` row of `git read-tree`:
/// the one-way read that writes over the worktree entries in its way.
pub const READ_TREE_RESET: u16 = 4;
/// The `-u` row of `git read-tree`:
/// the option that makes any of them write worktree files at all.
pub const READ_TREE_UPDATE: u16 = 5;
/// The `--prefix` row of `git read-tree`:
/// the tree it reads lands below the prefix it names.
pub const READ_TREE_PREFIX: u16 = 6;

/// What:
///  What a region can do to uncommitted state.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A region that overwrites is judged by the worktree kind;
/// one that only moves `HEAD` loses nothing unless
///  local changes exist,
/// so it is judged by them alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Discard = 'harmless' | 'moves-head' | 'overwrites';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Discard {
    /// Nothing is overwritten or deleted.
    Harmless,
    /// `HEAD` moves;
    /// Git refuses to overwrite a local change,
    /// but a `--force` or a path form would.
    MovesHead,
    /// The region overwrites or deletes worktree files or staged entries by itself.
    Overwrites,
}

/// What:
///  The revision a region names before its paths,
/// if it names one.
///  `Option<OsString>` is "the revision or nothing".
/// Why:
///   A forced checkout,
///  a forced switch and a hard reset write a tree into the worktree,
///  and
///  which tree decides whether an ignored file the worktree holds is replaced;
///  the tree is the first
///  positional token before `--`,
///  and the tokens after `--` are paths.
///  A region Git refuses
///  names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function revisionBeforePaths(region: string[], table: OptionSpec[]): string | undefined;
/// ```
pub fn revision_before_paths(region: &[OsString], table: &[OptionSpec]) -> Option<OsString> {
    // `let Ok(parsed) = ... else { return None; }` is the accepted region or nothing.
    let Ok(parsed) = parse_options(region, table, DEFAULT_MODE, &[]) else {
        return None;
    };
    // `.first()` is the first positional index before the boundary; `?` returns when there is none.
    let first: &usize = parsed.leading.first()?;
    return Some(region[*first].clone());
}

/// What:
///  The revision expression Git resolves for a shorthand that only `switch` and `checkout` read.
///  `Option<&'static [u8]>` is "the expression or nothing".
/// Why:
///   `git switch -` means the previous branch,
///  and no other Git command reads `-` as one:
///  `git ls-tree -`
///  fails,
///  so the tree a forced switch writes would be read as `HEAD` and the wrong tree protected.
///  `@{-N}` is resolved by Git wherever an object name is read,
///  so only `-` needs an expression of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function previousBranchExpression(revision: Buffer): Buffer | undefined;
/// ```
pub fn previous_branch_expression(revision: &[u8]) -> Option<&'static [u8]> {
    if revision == b"-" {
        return Some(b"@{-1}");
    }
    return None;
}

/// What:
///  Read a `git checkout` region.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   `--force`,
/// `--patch`,
/// `--ours`,
/// `--theirs`,
/// a path list (`--`,
/// `--pathspec-from-file`,
/// or a tree-ish followed
///  by paths) overwrite files;
/// a lone name moves `HEAD` or,
/// when it is a file,
/// overwrites it,
/// so it counts as a
///  `HEAD` move that is rejected once local changes exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutDiscard(region: string[]): Discard; // throws OptionError
/// ```
pub fn checkout_discard(region: &[OsString]) -> Result<Discard, OptionError> {
    let parsed: ParsedOptions = parse_options(region, CHECKOUT_TABLE, DEFAULT_MODE, &[])?;
    let names: usize = positional_tokens(&parsed, region.len()).len();
    if is_enabled(&parsed, FORCE)
        || is_enabled(&parsed, PATCH)
        || is_stated(&parsed, STAGE)
        || is_stated(&parsed, PATHSPEC_FROM_FILE)
        || matches!(parsed.boundary, Boundary::DashDash(_))
        || names > 1
    {
        return Ok(Discard::Overwrites);
    }
    if names == 1 {
        return Ok(Discard::MovesHead);
    }
    return Ok(Discard::Harmless);
}

/// What:
///  Read a `git switch` region.
/// Why:
///   `--force` and `--discard-changes` throw local changes away;
/// any named target moves `HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function switchDiscard(region: string[]): Discard;
/// ```
pub fn switch_discard(region: &[OsString]) -> Result<Discard, OptionError> {
    let parsed: ParsedOptions = parse_options(region, SWITCH_TABLE, DEFAULT_MODE, &[])?;
    if is_enabled(&parsed, FORCE) || is_enabled(&parsed, DISCARD_CHANGES) {
        return Ok(Discard::Overwrites);
    }
    if !positional_tokens(&parsed, region.len()).is_empty() {
        return Ok(Discard::MovesHead);
    }
    return Ok(Discard::Harmless);
}

/// What:
///  What one `git restore` region does,
///  read from its own table.
///  A `struct` is a record with named fields.
/// Why:
///   Every restore form writes worktree files,
///  the index,
///  or both,
///  so the only question left is which of the two,
///  and whether the person asked Git to write the files a `skip-worktree` mark names;
///  both answers come from the same parse,
///  so the flag reading and the scope reading cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RestoreReading = { discard: Discard; stagedOnly: boolean; ignoreSkipWorktreeBits: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreReading {
    /// What the region does to uncommitted state.
    pub discard: Discard,
    /// `--staged` without `--worktree`:
    ///  the index alone is written and the files stay.
    pub staged_only: bool,
    /// `--ignore-skip-worktree-bits`:
    ///  the person asked for the files the mark names,
    ///  so the mark hides no loss of theirs.
    pub ignore_skip_worktree_bits: bool,
}

/// What:
///  Read a `git restore` region.
/// Why:
///   Every form writes worktree files,
/// the index,
/// or both;
/// only a request for usage text writes nothing.
///  The tokenizer finds that request itself,
///  and it knows which words are values:
///  `git restore -s -h -- a.txt` restores the branch `-h`,
///  whose word a scan for a help flag took for the request and let the edit go
///  (the seventh review, reproduced here).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// restoreReading(['-s', '-h', '--', 'a.txt']) === { discard: 'overwrites', stagedOnly: false, ignoreSkipWorktreeBits: false };
/// ```
pub fn restore_reading(region: &[OsString]) -> Result<RestoreReading, OptionError> {
    let parsed: ParsedOptions = parse_options(region, RESTORE_TABLE, DEFAULT_MODE, &[])?;
    let staged: bool = is_enabled(&parsed, RESTORE_STAGED);
    let worktree: bool = is_enabled(&parsed, RESTORE_WORKTREE);
    return Ok(RestoreReading {
        discard: Discard::Overwrites,
        staged_only: staged && !worktree,
        ignore_skip_worktree_bits: is_enabled(&parsed, RESTORE_IGNORE_SKIP_WORKTREE_BITS),
    });
}

/// What:
///  What one `git rm` region does,
///  read from its own table.
/// Why:
///   A removal deletes an edited file when it carries a force,
///  and it also deletes the edit a `skip-worktree` mark hides when it carries `--sparse` without
///  `--cached`
///  (`git rm --sparse s.txt` exits 0 and the edited `s.txt` is gone,
///  while `git rm s.txt` refuses and keeps it,
///  measured on this host with native Git 2.56.0, 2026-11-14),
///  so a caller that guards the forced form needs the second answer as well.
///  `--dry-run` removes nothing at all.
///  Both answers come from the same parse,
///  so a word of one is never read as a value by the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoveReading = { discard: Discard; reachesASkippedEdit: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoveReading {
    /// What the region does to uncommitted state.
    pub discard: Discard,
    /// `--sparse` without `--cached`:
    ///  Git reaches the entries its sparse rules hide,
    ///  and the edit a `skip-worktree` mark names is one of them.
    pub reaches_a_skipped_edit: bool,
}

/// What:
///  Read a `git rm` region.
/// Why:
///   An unforced removal refuses a modified file itself,
/// so only a force discards by the person's instruction,
/// and a dry run discards nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeReading(['-f', '--pathspec-from-file', '--no-force']).discard === 'overwrites';
/// ```
pub fn remove_reading(region: &[OsString]) -> Result<RemoveReading, OptionError> {
    let parsed: ParsedOptions = parse_options(region, RM_TABLE, DEFAULT_MODE, &[])?;
    let forced: bool = counted_force_is_on(&parsed, RM_FORCE);
    let dry_run: bool = is_enabled(&parsed, RM_DRY_RUN);
    let sparse: bool = is_enabled(&parsed, RM_SPARSE);
    let cached: bool = is_enabled(&parsed, RM_CACHED);
    return Ok(RemoveReading {
        discard: if forced && !dry_run { Discard::Overwrites } else { Discard::Harmless },
        reaches_a_skipped_edit: sparse && !cached,
    });
}

/// What:
///  Read a `git mv` region.
/// Why:
///   A move refuses to write over an existing destination unless it carries a force,
///  and `--dry-run` moves nothing:
///  `git mv -f -n a.txt b.txt` reports the move and leaves both files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// moveDiscard(['-f', 'a.txt', 'b.txt']) === 'overwrites'; moveDiscard(['-f', '-n', 'a.txt', 'b.txt']) === 'harmless';
/// ```
pub fn move_reading(region: &[OsString]) -> Result<Discard, OptionError> {
    let parsed: ParsedOptions = parse_options(region, MV_TABLE, DEFAULT_MODE, &[])?;
    if counted_force_is_on(&parsed, MV_FORCE) && !is_enabled(&parsed, MV_DRY_RUN) {
        return Ok(Discard::Overwrites);
    }
    return Ok(Discard::Harmless);
}

/// What:
///  Read a `git checkout-index` region.
/// Why:
///   Without a force Git skips a file that already exists,
///  so the force is the whole question:
///  `git checkout-index -a` writes the files the worktree does not hold and leaves the ones it does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkoutIndexDiscard(['-f', '-a']) === 'overwrites'; checkoutIndexDiscard(['-a']) === 'harmless';
/// ```
pub fn checkout_index_reading(region: &[OsString]) -> Result<Discard, OptionError> {
    let parsed: ParsedOptions = parse_options(region, CHECKOUT_INDEX_TABLE, DEFAULT_MODE, &[])?;
    if counted_force_is_on(&parsed, CHECKOUT_INDEX_FORCE) {
        return Ok(Discard::Overwrites);
    }
    return Ok(Discard::Harmless);
}

/// What:
///  Whether the last spelling of a counted force option leaves it on.
///  `&ParsedOptions` borrows the tokenized region,
///  `u16` names the option's row.
/// Why:
///  `-f`/`--force` is `OPTION_COUNTUP` in every command that reads it here,
///  so `--no-force` sets the count back to zero and a later `-f` raises it again:
///  `git rm -f --no-force` is an unforced removal and `git rm --no-force -f` a forced one
///  (`OPT__FORCE`, `parse-options.h:569`; measured on this host with native Git 2.56.0, 2026-11-15).
///  A scan that counted the positive spellings alone read the first as forced,
///  and a scan that stopped at the first `-f` read the second as unforced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const forced = countup(parsed, RM_FORCE) > 0;
/// ```
fn counted_force_is_on(parsed: &ParsedOptions, id: u16) -> bool {
    let mut count: usize = 0;
    for occurrence in &parsed.occurrences {
        if occurrence.id != id {
            continue;
        }
        if occurrence.negated {
            count = 0;
        } else {
            count += 1;
        }
    }
    return count > 0;
}

/// What:
///  Read `git read-tree`'s own options,
///  so a spelling Git accepts is never read as an unknown option and a value that looks like a flag is never
///  read as one.
/// Why:
///   `-m`,
///  `-u`,
///  `-i` and `-n` have no long spelling,
///  `--no-sparse-checkout` is one option whose name begins with `no-`,
///  `--prefix`,
///  `--index-output` and `--exclude-per-directory` refuse the `--no-` form,
///  and `--prefix`,
///  `--index-output`,
///  `--super-prefix`,
///  `--exclude-per-directory` and `--recurse-submodules` take values
///  (`read-tree.c`,
///  `read_tree_options`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const READ_TREE_TABLE = [row(UNREAD, 'm', undefined, 'none', true), /* ... */] as const;
/// ```
pub const READ_TREE_TABLE: &[OptionSpec] = &[
    row(UNREAD, None, Some("super-prefix"), Arity::Required, true),
    row(UNREAD, None, Some("index-output"), Arity::Required, false),
    row(UNREAD, None, Some("empty"), Arity::None, true),
    row(UNREAD, Some(b'v'), Some("verbose"), Arity::None, true),
    row(READ_TREE_MERGE, Some(b'm'), None, Arity::None, true),
    row(UNREAD, None, Some("trivial"), Arity::None, true),
    row(UNREAD, None, Some("aggressive"), Arity::None, true),
    row(READ_TREE_RESET, None, Some("reset"), Arity::None, true),
    row(READ_TREE_PREFIX, None, Some("prefix"), Arity::Required, false),
    row(READ_TREE_UPDATE, Some(b'u'), None, Arity::None, true),
    row(UNREAD, None, Some("exclude-per-directory"), Arity::Required, false),
    row(UNREAD, Some(b'i'), None, Arity::None, true),
    row(READ_TREE_DRY_RUN, Some(b'n'), Some("dry-run"), Arity::None, true),
    row(UNREAD, None, Some("no-sparse-checkout"), Arity::None, true),
    row(READ_TREE_DEBUG_UNPACK, None, Some("debug-unpack"), Arity::None, true),
    row(UNREAD, None, Some("recurse-submodules"), Arity::Optional, true),
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
];

/// What:
///  Read a `git read-tree` region.
/// Why:
///   Every form Git accepts writes either the index or,
///  with `-u`,
///  worktree files,
///  and `-u` writes a file the worktree holds with the content of the tree,
///  so `git read-tree -m -u HEAD` reverts a staged edit and deletes a staged new file;
///  `-n` and `--dry-run` write nothing,
///  and neither does `--debug-unpack`,
///  which returns before the index is written (`read-tree.c`);
///  a region Git refuses is the refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTreeDiscard(region: string[]): Discard; // throws OptionError
/// ```
pub fn read_tree_discard(region: &[OsString]) -> Result<Discard, OptionError> {
    let parsed: ParsedOptions = parse_options(region, READ_TREE_TABLE, DEFAULT_MODE, &[])?;
    if is_enabled(&parsed, READ_TREE_DRY_RUN) || is_enabled(&parsed, READ_TREE_DEBUG_UNPACK) {
        return Ok(Discard::Harmless);
    }
    return Ok(Discard::Overwrites);
}

/// The shortest spelling of `--abort` Git accepts:
/// `--ab`.
/// Why:
///   Every command that reads `--abort` (`merge`,
/// `rebase`,
/// `cherry-pick`,
/// `revert` and `am`) accepts that prefix,
///  and no one of them names another long option starting with `ab`,
///  so the two-figure prefix after the dashes is where their tables stop being ambiguous
///  (measured on this host with Git 2.56.0:
///  each takes `--ab` and refuses `--a` as ambiguous, 2026-11-12).
pub const SHORTEST_ABORT: usize = 4;

/// The shortest spelling of `--skip` Git accepts:
/// `--sk`.
/// Why:
///   The four commands that read `--skip` (`rebase`,
/// `cherry-pick`,
/// `revert` and `am`) accept that prefix and refuse `--s` as ambiguous,
///  with the options they each carry beside it,
///  so the two-figure prefix after the dashes is where their tables stop being ambiguous
///  (measured on this host with native Git 2.56.0, 2026-11-13).
pub const SHORTEST_SKIP: usize = 4;

/// What:
///  Whether a token spells the long option `wanted` by any abbreviation of at least `shortest` characters,
/// with or without an `=value`.
/// Why:
///   The five commands that read `--abort` all take `--ab` and refuse anything shorter,
///  so the reading is that shortest spelling and not a per-command sibling table.
///  This is the one reading that consumes no value on purpose:
///  its question is whether a word is there at all,
///  so it cannot miss the option it guards,
///  and a word that is another option's value costs a refusal of a command Git runs,
///  which is the safe direction
///  (`git rebase --onto --abort` names the revision `--abort`;
///  `SPEC.md`, "Discarding commands", records the limit).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spellsLongFrom(token: string, wanted: string, shortest: number): boolean;
/// ```
fn spells_long_from(token: &[u8], wanted: &str, shortest: usize) -> bool {
    if !token.starts_with(b"--") {
        return false;
    }
    let name: &[u8] = token.split(|byte| return *byte == b'=').next().unwrap_or_default();
    return name.len() >= shortest && wanted.as_bytes().starts_with(&name[b"--".len()..]);
}

/// What:
///  The tokens a presence scan reads:
///  everything before the first `--`.
///  A value after `--` is a path,
/// never an option.
/// Why:
///  Only the abort and skip readings use this scan;
///  every other reading asks its command's option table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wordsBeforeAPathList = (region) => region.slice(0, region.indexOf('--'));
/// ```
fn options_of(region: &[OsString]) -> Vec<&[u8]> {
    let mut tokens: Vec<&[u8]> = Vec::<&[u8]>::new();
    for token in region {
        let bytes: &[u8] = token.as_encoded_bytes();
        if bytes == b"--" {
            break;
        }
        tokens.push(bytes);
    }
    return tokens;
}

/// What:
///  Read a region of `merge`,
/// `rebase`,
/// `cherry-pick`,
/// `revert` or `am` for `--abort`.
/// Why:
///   Aborting restores the pre-operation index and worktree,
/// discarding everything done since,
/// including the
///  hand edits that resolved a conflict.
/// `--ab` is the shortest prefix no sibling option shares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function abortDiscard(region: string[]): Discard;
/// ```
pub fn abort_discard(region: &[OsString]) -> Discard {
    for token in options_of(region) {
        if spells_long_from(token, "abort", SHORTEST_ABORT) {
            return Discard::Overwrites;
        }
    }
    return Discard::Harmless;
}

/// What:
///  Read a region of `rebase`,
///  `cherry-pick`,
///  `revert` or `am` for `--skip`.
/// Why:
///   `--skip` drops the commit or patch the operation stopped at and goes on,
///  and the reset it makes over the worktree throws away the hand resolution of that conflict
///  (measured with Git 2.56.0 for every form).
///  `--sk` is the shortest prefix no sibling option shares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skipDiscard(region: string[]): Discard;
/// ```
pub fn skip_discard(region: &[OsString]) -> Discard {
    for token in options_of(region) {
        if spells_long_from(token, "skip", SHORTEST_SKIP) {
            return Discard::Overwrites;
        }
    }
    return Discard::Harmless;
}

/// Each reading with Git's own refusals,
/// abbreviations,
/// clusters and values that look like flags.
#[cfg(test)]
#[path = "command_discard_tests.rs"]
mod tests;
