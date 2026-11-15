//! What:
//!  The complete option table of `git rm` in Git 2.56.0.
//! Why:
//!  The guard asks a removal two questions:
//!  whether it carries a force,
//!  and whether it reaches a file a `skip-worktree` mark hides (`--sparse` without `--cached`).
//!  Both turn on which word is an option and which word is that option's value:
//!  `git rm -f --pathspec-from-file --no-force` reads the file named `--no-force` and keeps `-f`,
//!  while a scan of the words takes `--no-force` for a negation and lets the edited file go.
//!  A complete table with the shared tokenizer is what makes the words mean what Git means by them.
//!  The rows are `builtin_rm_options` (`builtin/rm.c:251-263`),
//!  in Git's own order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const RM_TABLE: readonly OptionSpec[] = [ ... ];
//! ```

/// What:
///  Bring the row type,
///  its builder and the arity names into this file.
///  `UNREAD` is the identifier of rows no removal reading asks about.
/// Why:
///   The table is data for the shared tokenizer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, type OptionSpec, row, UNREAD } from './command_options.ts';
/// ```
use crate::command_options::{Arity, OptionSpec, UNREAD, row};

/// `-f`/`--force`/`--no-force`:
///  a counted option,
///  so `--no-force` sets the count back to zero.
pub const RM_FORCE: u16 = 1;
/// `-n`/`--dry-run`:
///  list what would be removed and remove nothing.
pub const RM_DRY_RUN: u16 = 2;
/// `--cached`:
///  remove the index entry and leave the worktree file alone.
pub const RM_CACHED: u16 = 3;
/// `--sparse`:
///  let Git reach an entry its sparse-checkout rules hide.
pub const RM_SPARSE: u16 = 4;
/// `--pathspec-from-file <file>`:
///  the paths come from a file no other word of the command names.
pub const RM_PATHSPEC_FROM_FILE: u16 = 5;
/// `--pathspec-file-nul`:
///  that file separates its entries with NUL bytes.
pub const RM_PATHSPEC_FILE_NUL: u16 = 6;

/// What:
///  The rows of `git rm`.
///  `-r` has no long spelling,
///  and `--pathspec-from-file` takes a value,
///  so the token after it is that value whatever it looks like.
/// Why:
///  A value that spells an option of this command is still a value:
///  `--pathspec-from-file --no-force` names a file,
///  and `--pathspec-from-file --sparse` does too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const RM_TABLE = [row(RM_DRY_RUN, 'n', 'dry-run', 'none', true), /* ... */] as const;
/// ```
pub const RM_TABLE: &[OptionSpec] = &[
    row(RM_DRY_RUN, Some(b'n'), Some("dry-run"), Arity::None, true),
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(RM_CACHED, None, Some("cached"), Arity::None, true),
    row(RM_FORCE, Some(b'f'), Some("force"), Arity::None, true),
    row(UNREAD, Some(b'r'), None, Arity::None, true),
    row(UNREAD, None, Some("ignore-unmatch"), Arity::None, true),
    row(RM_SPARSE, None, Some("sparse"), Arity::None, true),
    row(RM_PATHSPEC_FROM_FILE, None, Some("pathspec-from-file"), Arity::Required, true),
    row(RM_PATHSPEC_FILE_NUL, None, Some("pathspec-file-nul"), Arity::None, true),
];
