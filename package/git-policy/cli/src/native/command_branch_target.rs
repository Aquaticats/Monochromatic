//! What:
//!  Explicit creation and the remote-guess candidate of tokenized `git checkout` and
//!       `git switch` regions in Git 2.56.0.
//! Why:
//!  `git checkout topic` creates a local `topic` when exactly one remote has that
//!      branch.
//!  Which argument is the candidate,
//!  and whether Git guesses at all,
//!  follows
//!      `checkout_main` and `parse_branchname_arg` (builtin/checkout.c:1414-1532,
//!      1986-2021).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // checkoutTarget(parsed, region): number | undefined
//! ```

/// What:
///  Bring the shared checkout and switch option identifiers,
///  the tokenizer result
///       types and the tokenizer questions into this file.
/// Why:
///   Both commands use the same identifiers for the same meanings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DETACH, GUESS /* ... */ } from './command_checkout_table.ts';
/// ```
use super::command_checkout_table::{
    DETACH, GUESS, NEW_BRANCH, NEW_BRANCH_FORCE, ORPHAN, OVERLAY, PATCH, STAGE, TRACK,
};
use super::command_options::{Boundary, ParsedOptions};
use super::command_options_query::{is_enabled, is_stated, last_occurrence, positional_tokens};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  Whether the options create a branch by themselves.
/// Why:
///   `-b`,
///  `-B`,
///  `-c`,
///  `-C` and `--orphan` name the new branch.
///  `--track` and
///       `--no-track` both leave the tracking mode specified,
///  and Git then derives a new
///       branch name from the argument (checkout.c:1986-1997).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const creates = newBranch || newBranchForce || orphan || trackStated;
/// ```
pub(crate) fn creates_explicitly(parsed: &ParsedOptions) -> bool {
    return is_enabled(parsed, NEW_BRANCH)
        || is_enabled(parsed, NEW_BRANCH_FORCE)
        || is_enabled(parsed, ORPHAN)
        || is_stated(parsed, TRACK);
}

/// What:
///  Whether Git may still create a branch by guessing a remote branch.
/// Why:
///   `--no-guess` and `--patch` switch guessing off (checkout.c:2014-2018).
///  With
///       `--detach`,
///  a stage option or an overlay option Git dies before creating anything
///       (1686-1714,
///  550-552).
///  A pathspec file does not stop guessing:
///  when the file is
///       empty Git still switches branches (2058-2101).
/// Gotcha:
///  Divergence from the incumbent,
///  which treated `--pathspec-from-file` as ending
///         the guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const allowed = guess && !patch && !detach && !stage && !overlay;
/// ```
pub(crate) fn guess_allowed(parsed: &ParsedOptions) -> bool {
    // `--guess` is on by default; the last `--guess`/`--no-guess` decides.
    let mut guess: bool = true;
    // `if let Some(stated) = ...` runs only when the option was written.
    if let Some(stated) = last_occurrence(parsed, GUESS) {
        guess = !stated.negated;
    }
    return guess
        && !is_enabled(parsed, PATCH)
        && !is_enabled(parsed, DETACH)
        && !is_stated(parsed, STAGE)
        && !is_stated(parsed, OVERLAY);
}

/// What:
///  Whether a name contains a glob character Git treats as a pattern.
///  `&[u8]` borrows
///       the name bytes;
///  `u8` is one byte.
/// Why:
///   Without `--`,
///  `git checkout` does not guess for such a name (checkout.c:1512-1513;
///       the characters are `*`,
///  `?`,
///  `[` and backslash,
///  ctype.c:12).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wildcard = /[*?[\\]/.test(name);
/// ```
fn has_wildcard(name: &[u8]) -> bool {
    // `for byte in name` borrows each byte in order.
    for byte in name {
        // `*byte` reads the `u8` the loop variable points at.
        if *byte == b'*' || *byte == b'?' || *byte == b'[' || *byte == b'\\' {
            return true;
        }
    }
    return false;
}

/// What:
///  The token `git checkout` would turn into a new local branch if exactly one remote
///       has a branch of that name.
///  `Option<usize>` is "a region token index or nothing".
/// Why:
///   Git guesses only for `git checkout <name>` and `git checkout <name> --`;
///  a
///       leading `--` means paths,
///  and `-` means the previous branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutTarget(parsed: ParsedOptions, region: string[]): number | undefined;
/// ```
pub(crate) fn checkout_target(parsed: &ParsedOptions, region: &[OsString]) -> Option<usize> {
    // Git keeps `--` for checkout: its arguments are the leading ones, the `--`, the rest.
    let mut arguments: Vec<usize> = parsed.leading.clone();
    // Where the rest starts; the region length means "no rest".
    let mut rest: usize = region.len();
    if let Boundary::DashDash(at) = parsed.boundary {
        rest = at;
    }
    if let Boundary::EndOfOptions(at) = parsed.boundary {
        rest = at + 1;
    }
    while rest < region.len() {
        arguments.push(rest);
        rest += 1;
    }
    // Position, within Git's arguments, of the first `--`.
    let mut separator: usize = arguments.len();
    let mut position: usize = 0;
    while position < arguments.len() {
        if region[arguments[position]].as_encoded_bytes() == b"--" {
            separator = position;
            break;
        }
        position += 1;
    }
    // `git checkout <name>` or `git checkout <name> --` only (checkout.c:1487-1492, 1519-1522).
    let plain: bool = arguments.len() == 1 && separator == 1;
    let separated: bool = arguments.len() == 2 && separator == 1;
    if !plain && !separated {
        // `None` is the "absent" case of `Option`.
        return None;
    }
    let name: &[u8] = region[arguments[0]].as_encoded_bytes();
    if name == b"-" || (plain && has_wildcard(name)) {
        return None;
    }
    // `Some(x)` is the "present" case of `Option`.
    return Some(arguments[0]);
}

/// What:
///  The token `git switch` would turn into a new local branch by guessing.
/// Why:
///   `git switch` accepts exactly one reference and no paths (checkout.c:1473-1477),
///       so the single positional is the candidate,
///  also after `--`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function switchTarget(parsed: ParsedOptions, region: string[]): number | undefined;
/// ```
pub(crate) fn switch_target(parsed: &ParsedOptions, region: &[OsString]) -> Option<usize> {
    let arguments: Vec<usize> = positional_tokens(parsed, region.len());
    if arguments.len() != 1 {
        return None;
    }
    if region[arguments[0]].as_encoded_bytes() == b"-" {
        return None;
    }
    return Some(arguments[0]);
}
