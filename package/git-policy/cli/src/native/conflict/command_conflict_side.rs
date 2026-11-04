//! What:
//!  Which `git checkout` and `git restore` command lines take one side of a conflicted file,
//! and the paths they name.
//! Why:
//!  During a merge,
//! a rebase,
//! a cherry-pick or a revert the usual way to settle a conflict is
//!  `git checkout --ours|--theirs -- <path>`,
//! `git checkout <rev> -- <path>` or the `git restore` forms of the same.
//!  The linked-worktree policy rejects them as discarding forms;
//! the conflict-side guard lets them through for conflicted
//!  paths after saving the file.
//! Only a command line whose every token is understood qualifies:
//! a form with `--force`,
//!  `--patch`,
//! `--pathspec-from-file`,
//! a branch option,
//! an abbreviated `git restore` option,
//! an index-only restore,
//! or no
//!  `--` before the paths stays rejected,
//! because what it would discard is not known from the arguments alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! conflictSideRequest('checkout', ['--theirs', '--', 'a.rs']) // { paths: ['a.rs'] }
//! conflictSideRequest('checkout', ['-f', '--', 'a.rs'])       // undefined
//! ```

use crate::command_checkout_table::{
    CHECKOUT_TABLE, DETACH, FORCE, NEW_BRANCH, NEW_BRANCH_FORCE, ORPHAN, PATCH, PATHSPEC_FROM_FILE,
};
use crate::command_options::{Boundary, DEFAULT_MODE, ParsedOptions, parse_options};
use crate::command_options_query::is_stated;
use crate::diagnostic_log::debug;
use std::ffi::OsString;

/// Module tag for diagnostics.
const TAG: &str = "conflictSideRequest";

/// What:
///  Say why a command line is not read as a conflict-side form,
/// and give the answer that sends it to the usual rule.
/// Why:
///   The usual rejection then stands,
/// and a person asking why a line they thought was allowed was rejected can read the
///  reason in the verbose log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function declined(word: string, reason: string, region: string[]): undefined;
/// ```
fn declined(word: &str, reason: &str, region: &[OsString]) -> Option<SideRequest> {
    debug(TAG, &format!("git {word} {region:?} is not read as taking a side of a conflicted file: {reason}"));
    return None;
}

/// The paths a qualifying command line names,
/// as typed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SideRequest {
    /// The pathspec tokens after `--`,
    /// in order,
    /// none of them resolved or checked yet.
    pub paths: Vec<OsString>,
}

/// What:
///  Read a `git checkout` region for the conflict-side forms.
/// Why:
///   The form is `checkout [options] [<tree-ish>] -- <path>...`,
/// where the options are the ones that choose a side or
///  how a file is written.
/// Any option that creates or moves a branch,
/// forces,
/// patches or reads paths from a file
///  disqualifies the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkoutSideRequest(region: string[]): SideRequest | undefined;
/// ```
fn checkout_request(region: &[OsString]) -> Option<SideRequest> {
    let Ok(parsed): Result<ParsedOptions, _> = parse_options(region, CHECKOUT_TABLE, DEFAULT_MODE, &[]) else {
        return declined("checkout", "an option is not one Git's table lists", region);
    };
    let Boundary::DashDash(at) = parsed.boundary else {
        return declined("checkout", "there is no `--` before the paths", region);
    };
    for forbidden in [NEW_BRANCH, NEW_BRANCH_FORCE, ORPHAN, DETACH, PATCH, FORCE, PATHSPEC_FROM_FILE] {
        if is_stated(&parsed, forbidden) {
            return declined("checkout", "it forces, patches, makes a branch, detaches or reads paths from a file", region);
        }
    }
    if parsed.leading.len() > 1 {
        return declined("checkout", "it names more than one thing before `--`", region);
    }
    let paths: Vec<OsString> = region[at + 1..].to_vec();
    if paths.is_empty() {
        return declined("checkout", "no path follows `--`", region);
    }
    return Some(SideRequest { paths });
}

/// Whether a byte separates an option's name from its inline value.
fn is_equals(byte: &u8) -> bool {
    return *byte == b'=';
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

/// The spellings of `git restore` options that take a value,
/// as `--name=value` or as the next token.
const RESTORE_VALUED: [&[u8]; 3] = [b"--source", b"-s", b"--conflict"];

/// What:
///  Read a `git restore` region for the conflict-side forms.
/// Why:
///   `git restore` has no option table in this crate,
/// and Git accepts abbreviations,
/// so the reading is exact:
/// a token
///  that is not one of the listed spellings disqualifies the line.
/// A restore that names the index and not the worktree
///  (`--staged` alone) writes no file and is not a way to take a side,
/// so it stays rejected as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function restoreSideRequest(region: string[]): SideRequest | undefined;
/// ```
fn restore_request(region: &[OsString]) -> Option<SideRequest> {
    // Whether `--staged` and whether `--worktree` were parsed as options; a token that is the value of `--source`,
    // `-s` or `--conflict` is skipped with its option and is never one of them.
    let mut staged: bool = false;
    let mut worktree: bool = false;
    let mut at: usize = 0;
    while at < region.len() {
        let token: &[u8] = region[at].as_encoded_bytes();
        if token == b"--" {
            // `--staged` beside `--worktree` still writes files; `--staged` alone does not, and is not accepted.
            if staged && !worktree {
                return declined("restore", "it restores the index only, which writes no worktree file", region);
            }
            let paths: Vec<OsString> = region[at + 1..].to_vec();
            if paths.is_empty() {
                return declined("restore", "no path follows `--`", region);
            }
            return Some(SideRequest { paths });
        }
        if token == b"-W" || token == b"--worktree" {
            worktree = true;
            at += 1;
            continue;
        }
        if RESTORE_PLAIN.contains(&token) {
            at += 1;
            continue;
        }
        if token == b"-S" || token == b"--staged" {
            staged = true;
            at += 1;
            continue;
        }
        let name: &[u8] = token.split(is_equals).next().unwrap_or_default();
        if RESTORE_VALUED.contains(&name) {
            // `--source=<tree>` carries its value; `--source <tree>` takes the next token.
            at += if name.len() < token.len() { 1 } else { 2 };
            continue;
        }
        return declined("restore", format!("`{}` is not a spelling this reading lists", region[at].to_string_lossy()).as_str(), region);
    }
    // No `--`: the paths could be mistaken for option values, so this form is left to the usual rejection.
    return declined("restore", "there is no `--` before the paths", region);
}

/// What:
///  The conflict-side reading of a subcommand and its region,
/// or nothing when the command line does not qualify.
/// Why:
///   The caller asks only for `checkout` and `restore`;
/// every other command word is not a way to take a side.
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
