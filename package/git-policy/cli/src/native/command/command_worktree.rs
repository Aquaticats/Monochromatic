//! What:
//!  The one fact the wrapper needs about `git worktree`:
//!  whether the invocation
//!       changes worktree registrations.
//! Why:
//!  The worktree-copy opt-out is written after `git worktree add` or `git worktree move`,
//!  so the flag is recognized after that second word;
//! only an add is a source of ignored-state
//!      synchronization.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // changesWorktreeRegistrations(['add', '../topic']) === true
//! ```

/// What:
///  `OsString` is owned operating-system text of raw bytes.
///  Sibling the reader might
///       expect:
///  `String`,
///  which must be valid UTF-8.
/// Why:
///   The region is compared as bytes and never decoded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;

/// What:
/// What:
///  One `git worktree remove` as the audit reads it:
///  the directory it names and whether the person forced it.
///  A `struct` is a record with named fields.
/// Why:
///   The removal is the one form that takes a whole worktree at once,
///  and the owner chose to let Git run and record what it took,
///  so the two things the record must know are which directory and whether `--force` was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeRemoval = { target: string; force: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeRemoval {
    /// The directory the person named,
    ///  exactly as typed.
    pub target: OsString,
    /// `-f` or `--force` was given,
    ///  in any cluster or abbreviation Git accepts.
    pub force: bool,
}

/// What:
///  Read a `git worktree` region as a removal.
///  `Option<WorktreeRemoval>` is "a removal,
/// or nothing".
/// Why:
///   `cmd_worktree` declares subcommands and no option of its own,
///  so the word directly after `worktree` selects the action and is never abbreviated;
///  `worktree remove` takes exactly one directory and `-f`/`--force`,
///  and every other form is forwarded without a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeRemoval(region: string[]): WorktreeRemoval | undefined;
/// ```
pub fn worktree_removal(region: &[OsString]) -> Option<WorktreeRemoval> {
    if region.first()?.as_encoded_bytes() != b"remove" {
        return None;
    }
    let rest: &[OsString] = &region[1..];
    // `force_discard` reads `-f` in any short cluster and `--force` in every prefix this command's own option table
    //  leaves unambiguous (`--f` included), which are exactly the spellings Git accepts here.
    let force: bool = crate::command_discard::force_discard(b"worktree", rest) == crate::command_discard::Discard::Overwrites;
    // The first token that is not an option is the directory; `--` separates the two, and Git stops reading options
    //  after it.
    let mut options_end: bool = false;
    for token in rest {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !options_end && bytes == b"--" {
            options_end = true;
            continue;
        }
        if !options_end && bytes.starts_with(b"-") {
            continue;
        }
        return Some(WorktreeRemoval {
            target: token.clone(),
            force,
        });
    }
    return None;
}

///  The `git worktree` subcommand words that register or re-register a worktree path.
///       `&[&[u8]]` is a borrowed list of byte spellings baked into the program.
/// Why:
///   Git 2.56.0 `cmd_worktree` declares only subcommands and no option of its own
///       (builtin/worktree.c),
///  so the word directly after `worktree` selects the action
///       and is never abbreviated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REGISTRATION_SUBCOMMANDS = ['add', 'move'];
/// ```
const REGISTRATION_SUBCOMMANDS: &[&[u8]] = &[b"add", b"move"];

/// What:
///  Whether the tokens after `worktree` start with `add` or `move`.
///  `&[OsString]`
///       borrows those tokens;
///  `bool` is true or false.
/// Why:
///   The caller decides from this alone,
///  without reading the subcommand's options,
///  so
///       a mistaken option table can never hide a worktree creation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function changesWorktreeRegistrations(region: string[]): boolean;
/// ```
pub fn changes_worktree_registrations(region: &[OsString]) -> bool {
    // `let Some(x) = ... else { ... };` unwraps the first token or returns for an empty region.
    let Some(first) = region.first() else {
        return false;
    };
    // `.as_encoded_bytes()` lends the raw bytes; `.contains(&x)` borrows them for the lookup.
    return REGISTRATION_SUBCOMMANDS.contains(&first.as_encoded_bytes());
}

/// Word-position cases,
/// the removal reading and a real Git 2.56.0 control stay out of the release executable.
#[cfg(test)]
#[path = "command_worktree_tests.rs"]
mod tests;
