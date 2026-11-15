//! What:
//!  The complete option table of `git checkout-index` in Git 2.56.0.
//! Why:
//!  A forced `checkout-index` writes every path it names,
//!  or every path of the index with `-a`/`--all`,
//!  each below the directory `--prefix` names.
//!  Which words those are turns on this table:
//!  `--prefix` and `--stage` take a value,
//!  so a reading that hands the token after them back to the path list names a file Git never
//!  writes,
//!  and one that does not know `-a` inside a cluster reads a form that writes the whole index as
//!  one that writes nothing.
//!  The rows are `builtin_checkout_index_options` (`builtin/checkout-index.c:227-250`),
//!  in Git's own order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const CHECKOUT_INDEX_TABLE: readonly OptionSpec[] = [ ... ];
//! ```

/// What:
///  Bring the row type,
///  its builder and the arity names into this file.
///  `UNREAD` is the identifier of rows no checkout-index reading asks about.
/// Why:
///   The table is data for the shared tokenizer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, type OptionSpec, row, UNREAD } from './command_options.ts';
/// ```
use crate::command_options::{Arity, OptionSpec, UNREAD, row};

/// `-a`/`--all`:
///  every path the index holds.
pub const CHECKOUT_INDEX_ALL: u16 = 1;
/// `-f`/`--force`/`--no-force`:
///  a counted option,
///  so `--no-force` sets the count back to zero.
pub const CHECKOUT_INDEX_FORCE: u16 = 2;
/// `--prefix <directory>`:
///  the directory the index's paths are written below.
pub const CHECKOUT_INDEX_PREFIX: u16 = 3;
/// `--stage <n>`:
///  the index stage whose entry is written.
pub const CHECKOUT_INDEX_STAGE: u16 = 4;
/// `--stdin`:
///  the paths come from standard input, which the guard cannot read before Git does.
pub const CHECKOUT_INDEX_STDIN: u16 = 5;

/// What:
///  The rows of `git checkout-index`.
///  `--prefix` and `--stage` take a value,
///  `--stage` refuses the `--no-` form,
///  and `-z` has no long spelling.
/// Why:
///  A value that spells an option of this command is still a value,
///  and a value that names a file is no path Git writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const CHECKOUT_INDEX_TABLE = [row(CHECKOUT_INDEX_ALL, 'a', 'all', 'none', true), /* ... */] as const;
/// ```
pub const CHECKOUT_INDEX_TABLE: &[OptionSpec] = &[
    row(CHECKOUT_INDEX_ALL, Some(b'a'), Some("all"), Arity::None, true),
    row(UNREAD, None, Some("ignore-skip-worktree-bits"), Arity::None, true),
    row(CHECKOUT_INDEX_FORCE, Some(b'f'), Some("force"), Arity::None, true),
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(UNREAD, Some(b'n'), Some("no-create"), Arity::None, true),
    row(UNREAD, Some(b'u'), Some("index"), Arity::None, true),
    row(UNREAD, Some(b'z'), None, Arity::None, true),
    row(CHECKOUT_INDEX_STDIN, None, Some("stdin"), Arity::None, true),
    row(UNREAD, None, Some("temp"), Arity::None, true),
    row(CHECKOUT_INDEX_PREFIX, None, Some("prefix"), Arity::Required, true),
    row(CHECKOUT_INDEX_STAGE, None, Some("stage"), Arity::Required, false),
];
