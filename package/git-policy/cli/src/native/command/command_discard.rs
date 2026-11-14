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
//! Each region is read as Git 2.56.0 reads it,
//! so a spelling
//!  Git accepts as `--force` is not missed and a value that looks like a flag is not mistaken for one.
//! Gotcha:
//!  Where Git itself refuses the region the answer is the refusal;
//! the caller lets Git say so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkoutDiscard(['--', 'a.txt']) === 'overwrites'; checkoutDiscard(['topic']) === 'moves-head'
//! ```

use crate::command_checkout_table::{
    CHECKOUT_TABLE, DISCARD_CHANGES, FORCE, PATCH, PATHSPEC_FROM_FILE, STAGE, SWITCH_TABLE,
};
use crate::command_options::{Arity, Boundary, DEFAULT_MODE, OptionError, OptionSpec, ParsedOptions, parse_options, row, UNREAD};
use crate::command_options_query::{is_enabled, is_stated, positional_tokens};
use std::ffi::OsString;

/// The `--dry-run` row of `git read-tree`:
/// it writes neither the index nor the worktree.
const DRY_RUN: u16 = 1;
/// The `--debug-unpack` row of `git read-tree`:
/// Git prints the unpacked entries and returns before the index is written.
const DEBUG_UNPACK: u16 = 2;

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
/// function switchDiscard(region: string[]): Discard; // throws OptionError
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

/// The tokens a flag scan reads:
/// everything before the first `--`.
///  A value after `--` is a path,
/// never an option.
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

/// Whether a byte separates an option's name from its inline value.
fn is_equals(byte: &u8) -> bool {
    return *byte == b'=';
}

/// What:
///  Whether a token spells the long option `full` (written with its leading dashes) by any abbreviation of at
///  least `minimum` characters,
/// with or without an `=value`.
/// Why:
///   Git accepts every unambiguous prefix of a long option;
/// `minimum` is the shortest prefix that no sibling
///  option of the command shares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function spellsLong(token: string, full: string, minimum: number): boolean;
/// ```
fn spells_long(token: &[u8], full: &str, minimum: usize) -> bool {
    if !token.starts_with(b"--") {
        return false;
    }
    let name: &[u8] = token.split(is_equals).next().unwrap_or_default();
    return name.len() >= minimum && full.as_bytes().starts_with(name);
}

/// What:
///  Whether a token is a cluster of short flags that contains `letter`.
/// Why:
///   `-rf` and `-fr` are `-r -f` in every command read by a flag scan,
/// none of whose short flags takes a value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hasShort(token: string, letter: string): boolean;
/// ```
fn has_short(token: &[u8], letter: u8) -> bool {
    return token.len() > 1 && token[0] == b'-' && token[1] != b'-' && token[1..].contains(&letter);
}

/// What:
///  Read a `git restore` region.
/// Why:
///   Every form writes worktree files,
/// the index,
/// or both;
/// only a request for usage text writes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreDiscard(region: string[]): Discard;
/// ```
pub fn restore_discard(region: &[OsString]) -> Discard {
    for token in options_of(region) {
        if token == b"-h" || spells_long(token, "--help", 4) {
            return Discard::Harmless;
        }
    }
    return Discard::Overwrites;
}

/// What:
///  Whether a `git restore` region touches the index only.
/// Why:
///   `--staged` alone unstages and leaves the files,
/// so the guidance differs from a restore of file content;
///  `--worktree` beside it brings the files back in.
/// `--st` and `--w` are the shortest unambiguous prefixes.
/// A token that
///  is the value of `--source`,
/// `-s` or `--conflict` is that value and never a flag (`restore --source -W --staged` names
///  a revision `-W` and restores the index only).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoresIndexOnly(region: string[]): boolean;
/// ```
pub fn restores_index_only(region: &[OsString]) -> bool {
    let tokens: Vec<&[u8]> = options_of(region);
    let mut staged: bool = false;
    let mut worktree: bool = false;
    let mut at: usize = 0;
    while at < tokens.len() {
        let token: &[u8] = tokens[at];
        at += 1;
        if token.starts_with(b"--") {
            if spells_long(token, "--staged", 4) {
                staged = true;
            }
            if spells_long(token, "--worktree", 3) {
                worktree = true;
            }
            // `--source <tree>` and `--conflict <style>` take the next token as their value, whatever it looks like.
            if !token.contains(&b'=') && (spells_long(token, "--source", 4) || spells_long(token, "--conflict", 3)) {
                at += 1;
            }
            continue;
        }
        if token.len() > 1 && token[0] == b'-' {
            // A short cluster is read letter by letter; `s` takes the rest of the token as its value, or the next token
            // when it ends the cluster, so nothing after it in the same token is a flag.
            for (position, letter) in token[1..].iter().enumerate() {
                match *letter {
                    b'S' => staged = true,
                    b'W' => worktree = true,
                    b's' => {
                        if position + 2 == token.len() {
                            at += 1;
                        }
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    return staged && !worktree;
}

/// What:
///  Read a `git rm`,
/// `git mv` or `git checkout-index` region.
/// Why:
///   Each refuses to overwrite or delete a modified file unless forced;
/// `-f` and `--force` are the only way
///  these commands discard.
/// `--fo` is the shortest prefix of `--force` in all three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forceDiscard(region: string[]): Discard;
/// ```
pub fn force_discard(region: &[OsString]) -> Discard {
    for token in options_of(region) {
        if has_short(token, b'f') || spells_long(token, "--force", 4) {
            return Discard::Overwrites;
        }
    }
    return Discard::Harmless;
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
    row(UNREAD, Some(b'm'), None, Arity::None, true),
    row(UNREAD, None, Some("trivial"), Arity::None, true),
    row(UNREAD, None, Some("aggressive"), Arity::None, true),
    row(UNREAD, None, Some("reset"), Arity::None, true),
    row(UNREAD, None, Some("prefix"), Arity::Required, false),
    row(UNREAD, Some(b'u'), None, Arity::None, true),
    row(UNREAD, None, Some("exclude-per-directory"), Arity::Required, false),
    row(UNREAD, Some(b'i'), None, Arity::None, true),
    row(DRY_RUN, Some(b'n'), Some("dry-run"), Arity::None, true),
    row(UNREAD, None, Some("no-sparse-checkout"), Arity::None, true),
    row(DEBUG_UNPACK, None, Some("debug-unpack"), Arity::None, true),
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
    if is_enabled(&parsed, DRY_RUN) || is_enabled(&parsed, DEBUG_UNPACK) {
        return Ok(Discard::Harmless);
    }
    return Ok(Discard::Overwrites);
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
        if spells_long(token, "--abort", 4) {
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
        if spells_long(token, "--skip", 4) {
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
