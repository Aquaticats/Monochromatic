//! What:
//!  The complete option table of `git worktree remove` in Git 2.56.0.
//! Why:
//!  The wrapper records every worktree removal and never refuses it,
//!  and it names the directory the person gave and whether a force carried it.
//!  Which word is the directory turns on this table,
//!  and a reading that guesses at the words can name a directory Git never removes.
//!  The rows are the ones `remove_worktree` declares (`builtin/worktree.c:1429-1434`):
//!  the force and nothing else,
//!  with Git's own help options added by the parser.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const WORKTREE_REMOVE_TABLE: readonly OptionSpec[] = [ ... ];
//! ```

/// What:
///  Bring the row type,
///  its builder and the arity names into this file.
/// Why:
///   The table is data for the shared tokenizer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, type OptionSpec, row } from './command_options.ts';
/// ```
use crate::command_options::{Arity, OptionSpec, row};

/// `-f`/`--force`/`--no-force`:
///  a counted option,
///  so `--no-force` sets the count back to zero.
pub const WORKTREE_REMOVE_FORCE: u16 = 1;

/// What:
///  The row of `git worktree remove`.
///  `OPT__FORCE` is a counted option whose `--no-force` resets the count,
///  and no row of this command takes a value,
///  so every word that is no option is the directory.
/// Why:
///  The directory a removal names is the only thing the record carries about what it takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const WORKTREE_REMOVE_TABLE = [row(WORKTREE_REMOVE_FORCE, 'f', 'force', 'none', true)] as const;
/// ```
pub const WORKTREE_REMOVE_TABLE: &[OptionSpec] = &[row(
    WORKTREE_REMOVE_FORCE,
    Some(b'f'),
    Some("force"),
    Arity::None,
    true,
)];
