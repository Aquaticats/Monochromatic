//! What:
//!  Which `git checkout` and `git restore` command lines take one side of a conflicted file, and the paths they name.
//! Why:
//!  During a merge, a rebase, a cherry-pick or a revert the usual way to settle a conflict is
//!  `git checkout --ours|--theirs -- <path>`, `git checkout <rev> -- <path>` or the `git restore` forms of the same.
//!  The linked-worktree policy rejects them as discarding forms; the conflict-side guard lets them through for conflicted
//!  paths after saving the file. Only a command line whose every token is understood qualifies: a form with `--force`,
//!  `--patch`, `--pathspec-from-file`, a branch option, an abbreviated `git restore` option, an index-only restore, or no
//!  `--` before the paths stays rejected, because what it would discard is not known from the arguments alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! conflictSideRequest('checkout', ['--theirs', '--', 'a.rs']) // { paths: ['a.rs'] }
//! conflictSideRequest('checkout', ['-f', '--', 'a.rs'])       // undefined
//! ```

use super::command_checkout_table::{
    CHECKOUT_TABLE, DETACH, FORCE, NEW_BRANCH, NEW_BRANCH_FORCE, ORPHAN, PATCH, PATHSPEC_FROM_FILE,
};
use super::command_options::{Boundary, DEFAULT_MODE, ParsedOptions, parse_options};
use super::command_options_query::is_stated;
use std::ffi::OsString;

/// The paths a qualifying command line names, as typed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SideRequest {
    /// The pathspec tokens after `--`, in order, none of them resolved or checked yet.
    pub paths: Vec<OsString>,
}

/// What:
///  Read a `git checkout` region for the conflict-side forms.
/// Why:
///   The form is `checkout [options] [<tree-ish>] -- <path>...`, where the options are the ones that choose a side or
///  how a file is written. Any option that creates or moves a branch, forces, patches or reads paths from a file
///  disqualifies the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutSideRequest(region: string[]): SideRequest | undefined;
/// ```
fn checkout_request(region: &[OsString]) -> Option<SideRequest> {
    let parsed: ParsedOptions = parse_options(region, CHECKOUT_TABLE, DEFAULT_MODE, &[]).ok()?;
    let Boundary::DashDash(at) = parsed.boundary else {
        return None;
    };
    for forbidden in [NEW_BRANCH, NEW_BRANCH_FORCE, ORPHAN, DETACH, PATCH, FORCE, PATHSPEC_FROM_FILE] {
        if is_stated(&parsed, forbidden) {
            return None;
        }
    }
    if parsed.leading.len() > 1 {
        return None;
    }
    let paths: Vec<OsString> = region[at + 1..].to_vec();
    if paths.is_empty() {
        return None;
    }
    return Some(SideRequest { paths });
}

/// The spellings of `git restore` options that take no value and keep the command a worktree restore of named paths.
const RESTORE_PLAIN: [&[u8]; 14] = [
    b"--ours",
    b"--theirs",
    b"-m",
    b"--merge",
    b"-W",
    b"--worktree",
    b"-q",
    b"--quiet",
    b"--progress",
    b"--no-progress",
    b"--overlay",
    b"--no-overlay",
    b"--ignore-unmerged",
    b"--ignore-skip-worktree-bits",
];

/// The spellings of `git restore` options that take a value, as `--name=value` or as the next token.
const RESTORE_VALUED: [&[u8]; 3] = [b"--source", b"-s", b"--conflict"];

/// What:
///  Read a `git restore` region for the conflict-side forms.
/// Why:
///   `git restore` has no option table in this crate, and Git accepts abbreviations, so the reading is exact: a token
///  that is not one of the listed spellings disqualifies the line. A restore that names the index and not the worktree
///  (`--staged` alone) writes no file and is not a way to take a side, so it stays rejected as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreSideRequest(region: string[]): SideRequest | undefined;
/// ```
fn restore_request(region: &[OsString]) -> Option<SideRequest> {
    let mut at: usize = 0;
    while at < region.len() {
        let token: &[u8] = region[at].as_encoded_bytes();
        if token == b"--" {
            let paths: Vec<OsString> = region[at + 1..].to_vec();
            if paths.is_empty() {
                return None;
            }
            return Some(SideRequest { paths });
        }
        if RESTORE_PLAIN.contains(&token) {
            at += 1;
            continue;
        }
        // `--staged` beside `--worktree` still writes files; `--staged` alone does not, and is not accepted.
        if token == b"-S" || token == b"--staged" {
            let writes_worktree: bool = region.iter().take_while(|each| { return each.as_encoded_bytes() != b"--"; }).any(|each| {
                return matches!(each.as_encoded_bytes(), b"-W" | b"--worktree");
            });
            if !writes_worktree {
                return None;
            }
            at += 1;
            continue;
        }
        let name: &[u8] = token.split(|byte| { return *byte == b'='; }).next().unwrap_or_default();
        if RESTORE_VALUED.contains(&name) {
            // `--source=<tree>` carries its value; `--source <tree>` takes the next token.
            at += if name.len() < token.len() { 1 } else { 2 };
            continue;
        }
        return None;
    }
    // No `--`: the paths could be mistaken for option values, so this form is left to the usual rejection.
    return None;
}

/// What:
///  The conflict-side reading of a subcommand and its region, or nothing when the command line does not qualify.
/// Why:
///   The caller asks only for `checkout` and `restore`; every other command word is not a way to take a side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conflictSideRequest(word: string, region: string[]): SideRequest | undefined;
/// ```
pub fn conflict_side_request(word: &[u8], region: &[OsString]) -> Option<SideRequest> {
    if word == b"checkout" {
        return checkout_request(region);
    }
    if word == b"restore" {
        return restore_request(region);
    }
    return None;
}

/// Each spelling that qualifies and each that does not.
#[cfg(test)]
#[path = "command_conflict_side_tests.rs"]
mod tests;
