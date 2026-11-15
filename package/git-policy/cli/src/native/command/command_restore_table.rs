//! What:
//!  The complete option table of `git restore` in Git 2.56.0.
//! Why:
//!  The guard asks a restore region three questions
//!  (which tree its paths are read from, whether it touches the index alone, and which file holds
//!  its pathspecs),
//!  and every one of them turns on which word is an option and which word is that option's value.
//!  `git restore -s -h -- a.txt` writes the content of the branch `-h`:
//!  a reading that scans the words for a help request takes `-h` for one and lets the edit go.
//!  A complete table with the shared tokenizer is what makes the words mean what Git means by them.
//!  The rows are assembled as Git assembles them:
//!  the command's own rows (`builtin/checkout.c:2209-2222`),
//!  then `add_common_options` (`builtin/checkout.c:1782-1802`),
//!  then `add_checkout_path_options` (`builtin/checkout.c:1830-1851`),
//!  the two groups `git restore` shares with `git checkout`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const RESTORE_TABLE: readonly OptionSpec[] = [ ... ];
//! ```

/// What:
///  Bring the row type,
///  its builder and the arity names into this file.
///  `UNREAD` is the identifier of rows no restore reading asks about.
/// Why:
///   The table is data for the shared tokenizer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, type OptionSpec, row, UNREAD } from './command_options.ts';
/// ```
use crate::command_options::{Arity, OptionSpec, UNREAD, row};

/// `-s`/`--source <tree-ish>`:
///  the tree the restored paths are read from.
pub const RESTORE_SOURCE: u16 = 1;
/// `-S`/`--staged`:
///  restore the index.
pub const RESTORE_STAGED: u16 = 2;
/// `-W`/`--worktree`:
///  restore the working tree files,
///  which is what a restore does without it.
pub const RESTORE_WORKTREE: u16 = 3;
/// `--ignore-skip-worktree-bits`:
///  the person asks Git to write the files a `skip-worktree` mark names.
pub const RESTORE_IGNORE_SKIP_WORKTREE_BITS: u16 = 4;
/// `--pathspec-from-file <file>`:
///  the paths come from a file no other word of the command names.
pub const RESTORE_PATHSPEC_FROM_FILE: u16 = 5;
/// `--pathspec-file-nul`:
///  that file separates its entries with NUL bytes.
pub const RESTORE_PATHSPEC_FILE_NUL: u16 = 6;
/// `-m`/`--merge`:
///  merge mode,
///  which decides whether a file outside the pathspec is written.
pub const RESTORE_MERGE: u16 = 7;
/// `--overlay`/`--no-overlay`:
///  whether a path the source tree does not hold is deleted.
pub const RESTORE_OVERLAY: u16 = 8;

/// What:
///  The rows of `git restore`,
///  in the order Git concatenates them.
///  `Some(b's')` with `Some("source")` is one option in two spellings;
///  `Some(b'U')` with no negation is Git's `PARSE_OPT_NONEG`.
/// Why:
///  `--source` takes a value,
///  so the token after it is that value whatever it looks like;
///  `-U`/`--unified` and `--inter-hunk-context` do too,
///  and a reading that does not know them reads their numbers as paths.
///  `--pathspec-from-file` decides which words are paths at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const RESTORE_TABLE = [row(RESTORE_SOURCE, 's', 'source', 'required', true), /* ... */] as const;
/// ```
pub const RESTORE_TABLE: &[OptionSpec] = &[
    row(RESTORE_SOURCE, Some(b's'), Some("source"), Arity::Required, true),
    row(RESTORE_STAGED, Some(b'S'), Some("staged"), Arity::None, true),
    row(RESTORE_WORKTREE, Some(b'W'), Some("worktree"), Arity::None, true),
    row(UNREAD, None, Some("ignore-unmerged"), Arity::None, true),
    row(RESTORE_OVERLAY, None, Some("overlay"), Arity::None, true),
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(UNREAD, None, Some("recurse-submodules"), Arity::Optional, true),
    row(UNREAD, None, Some("progress"), Arity::None, true),
    row(RESTORE_MERGE, Some(b'm'), Some("merge"), Arity::None, true),
    row(UNREAD, None, Some("conflict"), Arity::Required, true),
    row(UNREAD, Some(b'2'), Some("ours"), Arity::None, false),
    row(UNREAD, Some(b'3'), Some("theirs"), Arity::None, false),
    row(UNREAD, Some(b'p'), Some("patch"), Arity::None, true),
    row(UNREAD, Some(b'U'), Some("unified"), Arity::Required, false),
    row(UNREAD, None, Some("inter-hunk-context"), Arity::Required, false),
    row(
        RESTORE_IGNORE_SKIP_WORKTREE_BITS,
        None,
        Some("ignore-skip-worktree-bits"),
        Arity::None,
        true,
    ),
    row(
        RESTORE_PATHSPEC_FROM_FILE,
        None,
        Some("pathspec-from-file"),
        Arity::Required,
        true,
    ),
    row(
        RESTORE_PATHSPEC_FILE_NUL,
        None,
        Some("pathspec-file-nul"),
        Arity::None,
        true,
    ),
];
