//! What:
//!  The complete option table of `git mv` in Git 2.56.0.
//! Why:
//!  A forced move writes over the destination the person names,
//!  and the guard reads that destination from the words that are not options.
//!  Which words those are turns on this table:
//!  `git mv -f --sparse a.txt b.txt` moves `a.txt` onto `b.txt`,
//!  and a reading that does not know `--sparse` reads it as a source of its own.
//!  The rows are `builtin_mv_options` (`builtin/mv.c:222-230`),
//!  in Git's own order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const MV_TABLE: readonly OptionSpec[] = [ ... ];
//! ```

/// What:
///  Bring the row type,
///  its builder and the arity names into this file.
///  `UNREAD` is the identifier of rows no move reading asks about.
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
pub const MV_FORCE: u16 = 1;
/// `-n`/`--dry-run`:
///  report the move and write nothing.
pub const MV_DRY_RUN: u16 = 2;

/// What:
///  The rows of `git mv`.
///  `-k` has no long spelling,
///  and no row of this command takes a value,
///  so no word of a move region is another word's value.
/// Why:
///  Every word that is no option is a source or the destination,
///  and the guard reads them as the worktree resolves them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MV_TABLE = [row(UNREAD, 'v', 'verbose', 'none', true), /* ... */] as const;
/// ```
pub const MV_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'v'), Some("verbose"), Arity::None, true),
    row(MV_DRY_RUN, Some(b'n'), Some("dry-run"), Arity::None, true),
    row(MV_FORCE, Some(b'f'), Some("force"), Arity::None, true),
    row(UNREAD, Some(b'k'), None, Arity::None, true),
    row(UNREAD, None, Some("sparse"), Arity::None, true),
];
