//! What:
//!  Which regions of `git checkout`, `git switch`, `git restore`, `git rm`, `git mv`, `git checkout-index`,
//!  `git read-tree` and the `--abort` forms of `merge`, `rebase`, `cherry-pick`, `revert` and `am` can overwrite or
//!  delete uncommitted state.
//! Why:
//!  The linked-worktree policy confines such commands: it rejects them in the main worktree and, once the worktree
//!  holds uncommitted state, in a linked worktree too. Each region is read as Git 2.56.0 reads it, so a spelling
//!  Git accepts as `--force` is not missed and a value that looks like a flag is not mistaken for one.
//! Gotcha:
//!  Where Git itself refuses the region the answer is the refusal; the caller lets Git say so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkoutDiscard(['--', 'a.txt']) === 'overwrites'; checkoutDiscard(['topic']) === 'moves-head'
//! ```

use super::command_checkout_table::{
    CHECKOUT_TABLE, DISCARD_CHANGES, FORCE, PATCH, PATHSPEC_FROM_FILE, STAGE, SWITCH_TABLE,
};
use super::command_options::{Boundary, DEFAULT_MODE, OptionError, ParsedOptions, parse_options};
use super::command_options_query::{is_enabled, is_stated, positional_tokens};
use std::ffi::OsString;

/// What:
///  What a region can do to uncommitted state.
///  An `enum` is a closed set of named alternatives.
/// Why:
///   A region that overwrites is judged by the worktree kind; one that only moves `HEAD` loses nothing unless
///  local changes exist, so it is judged by them alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Discard = 'harmless' | 'moves-head' | 'overwrites';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Discard {
    /// Nothing is overwritten or deleted.
    Harmless,
    /// `HEAD` moves; Git refuses to overwrite a local change, but a `--force` or a path form would.
    MovesHead,
    /// The region overwrites or deletes worktree files or staged entries by itself.
    Overwrites,
}

/// What:
///  Read a `git checkout` region.
///  `Result<A, B>` is "either success `A` or failure `B`".
/// Why:
///   `--force`, `--patch`, `--ours`, `--theirs`, a path list (`--`, `--pathspec-from-file`, or a tree-ish followed
///  by paths) overwrite files; a lone name moves `HEAD` or, when it is a file, overwrites it, so it counts as a
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
///   `--force` and `--discard-changes` throw local changes away; any named target moves `HEAD`.
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

/// The tokens a flag scan reads: everything before the first `--`.
///  A value after `--` is a path, never an option.
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
///  Whether a token spells the long option `full` (written with its leading dashes) by any abbreviation of at
///  least `minimum` characters, with or without an `=value`.
/// Why:
///   Git accepts every unambiguous prefix of a long option; `minimum` is the shortest prefix that no sibling
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
    let name: &[u8] = token.split(|byte| { return *byte == b'='; }).next().unwrap_or_default();
    return name.len() >= minimum && full.as_bytes().starts_with(name);
}

/// What:
///  Whether a token is a cluster of short flags that contains `letter`.
/// Why:
///   `-rf` and `-fr` are `-r -f` in every command read by a flag scan, none of whose short flags takes a value.
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
///   Every form writes worktree files, the index, or both; only a request for usage text writes nothing.
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
///   `--staged` alone unstages and leaves the files, so the guidance differs from a restore of file content;
///  `--worktree` beside it brings the files back in. `--st` and `--w` are the shortest unambiguous prefixes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoresIndexOnly(region: string[]): boolean;
/// ```
pub fn restores_index_only(region: &[OsString]) -> bool {
    let mut staged: bool = false;
    let mut worktree: bool = false;
    for token in options_of(region) {
        if has_short(token, b'S') || spells_long(token, "--staged", 4) {
            staged = true;
        }
        if has_short(token, b'W') || spells_long(token, "--worktree", 3) {
            worktree = true;
        }
    }
    return staged && !worktree;
}

/// What:
///  Read a `git rm`, `git mv` or `git checkout-index` region.
/// Why:
///   Each refuses to overwrite or delete a modified file unless forced; `-f` and `--force` are the only way
///  these commands discard. `--fo` is the shortest prefix of `--force` in all three.
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
///  Read a `git read-tree` region.
/// Why:
///   `--reset` replaces the index and, with `-u`, the worktree, dropping every unmerged and local entry.
///   `--res` is the shortest prefix that `--recurse-submodules` does not share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTreeDiscard(region: string[]): Discard;
/// ```
pub fn read_tree_discard(region: &[OsString]) -> Discard {
    for token in options_of(region) {
        if spells_long(token, "--reset", 5) {
            return Discard::Overwrites;
        }
    }
    return Discard::Harmless;
}

/// What:
///  Read a region of `merge`, `rebase`, `cherry-pick`, `revert` or `am` for `--abort`.
/// Why:
///   Aborting restores the pre-operation index and worktree, discarding everything done since, including the
///  hand edits that resolved a conflict. `--ab` is the shortest prefix no sibling option shares.
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

/// Each reading with Git's own refusals, abbreviations, clusters and values that look like flags.
#[cfg(test)]
#[path = "command_discard_tests.rs"]
mod tests;
