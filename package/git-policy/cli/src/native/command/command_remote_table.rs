//! What:
//!  The option table of `git remote` in Git 2.56.0,
//!  which holds the one option Git reads before a subcommand.
//! Why:
//!  `git remote -v set-url origin <url>` is the same write as `git remote set-url origin <url>`,
//!  so a rule that reads the
//!       subcommand from the first token is bypassed by one letter.
//!  Git's own table is the only authority on
//!       which spellings sit there,
//!  and it holds exactly `-v`/`--verbose`,
//!  `git remote
//!       --git-completion-helper-all` printing `--verbose add rename rm remove set-head
//!       set-branches get-url set-url show prune update --no-verbose`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const REMOTE_TABLE: readonly OptionSpec[] = [row(UNREAD, 'v', 'verbose', 'none', true)];
//! ```

/// The row type and its builder,
///  and the arity names.
use crate::command_options::{Arity, OptionSpec, UNREAD, row};

/// What:
///  `OPT__VERBOSE` of `builtin/remote.c`:
///  count the `-v` letters,
///  and accept `--no-verbose` as the same switch turned off.
/// Why:
///   Every other option of `git remote` belongs to a subcommand,
///  so this row is the whole
///       table the wrapper parses before that subcommand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const REMOTE_TABLE = [row(UNREAD, 'v', 'verbose', 'none', true)] as const;
/// ```
pub const REMOTE_TABLE: &[OptionSpec] = &[row(UNREAD, Some(b'v'), Some("verbose"), Arity::None, true)];
