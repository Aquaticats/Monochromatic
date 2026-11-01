//! What:
//!  The one fact the wrapper needs about `git worktree`:
//!  whether the invocation
//!       creates or moves a worktree.
//! Why:
//!  Only `git worktree add` and `git worktree move` are sources of ignored-state
//!      synchronization,
//!  and the worktree-copy opt-out is written after that second word.
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

/// Word-position cases and a real Git 2.56.0 control stay out of the release executable.
#[cfg(test)]
#[path = "command_worktree_tests.rs"]
mod tests;
