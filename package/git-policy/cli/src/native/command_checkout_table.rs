//! What:
//!  The complete option tables of `git checkout` and `git switch` in Git 2.56.0.
//! Why:
//!  Both commands create branches explicitly (`-b`,
//!  `-c`,
//!  `--orphan`,
//!  `--track`) and
//!      implicitly (a name that matches one remote branch).
//!  The tables are assembled as
//!      Git assembles them:
//!  the command's own rows,
//!  then the shared groups
//!      (builtin/checkout.c:1786-1847,
//!  2118-2129,
//!  2170-2179).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const CHECKOUT_TABLE: readonly OptionSpec[] = [ ... ];
//! ```

/// What:
///  Bring the row type,
///  its builder and the arity names into this file.
///  `UNREAD` is the
///       identifier of rows no branch-creation fact reads.
/// Why:
///   The tables are data for the shared tokenizer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, type OptionSpec, row, UNREAD } from './command_options.ts';
/// ```
use super::command_options::{Arity, OptionSpec, UNREAD, row};

/// `checkout -b <branch>`,
///  `switch -c`/`--create <branch>`.
pub const NEW_BRANCH: u16 = 1;
/// `checkout -B <branch>`,
///  `switch -C`/`--force-create <branch>`.
pub const NEW_BRANCH_FORCE: u16 = 2;
/// `--orphan <new-branch>`.
pub const ORPHAN: u16 = 3;
/// `-t`,
///  `--track[=(direct|inherit)]`;
///  `--no-track` writes the same variable.
pub const TRACK: u16 = 4;
/// `--guess`,
///  `--no-guess`.
pub const GUESS: u16 = 5;
/// `-d`,
///  `--detach`.
pub const DETACH: u16 = 6;
/// `checkout -p`/`--patch`.
pub const PATCH: u16 = 7;
/// `checkout -2`/`--ours` and `-3`/`--theirs`.
pub const STAGE: u16 = 8;
/// `checkout --overlay`,
///  `--no-overlay`.
pub const OVERLAY: u16 = 9;
/// `-f`/`--force`: throw away local changes.
pub const FORCE: u16 = 10;
/// `switch --discard-changes`: throw away local changes.
pub const DISCARD_CHANGES: u16 = 11;
/// `--pathspec-from-file`: the paths come from a file, so the command is the path form.
pub const PATHSPEC_FROM_FILE: u16 = 12;

/// What:
///  `git checkout`:
///  `checkout_options`,
///  then the common,
///  switch-branch and
///       checkout-path groups.
///  `Some(b'b')` with `None` is an option that has only a letter.
/// Why:
///   `-b` and `-B` take the branch name as a value;
///  `--track` and
///       `--recurse-submodules` take an attached value only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const CHECKOUT_TABLE = [row(NEW_BRANCH, 'b', undefined, 'required', true), /* ... */] as const;
/// ```
pub const CHECKOUT_TABLE: &[OptionSpec] = &[
    row(NEW_BRANCH, Some(b'b'), None, Arity::Required, true),
    row(NEW_BRANCH_FORCE, Some(b'B'), None, Arity::Required, true),
    row(UNREAD, Some(b'l'), None, Arity::None, true),
    row(GUESS, None, Some("guess"), Arity::None, true),
    row(OVERLAY, None, Some("overlay"), Arity::None, true),
    row(UNREAD, None, Some("auto-advance"), Arity::None, true),
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(
        UNREAD,
        None,
        Some("recurse-submodules"),
        Arity::Optional,
        true,
    ),
    row(UNREAD, None, Some("progress"), Arity::None, true),
    row(UNREAD, Some(b'm'), Some("merge"), Arity::None, true),
    row(UNREAD, None, Some("conflict"), Arity::Required, true),
    row(DETACH, Some(b'd'), Some("detach"), Arity::None, true),
    row(TRACK, Some(b't'), Some("track"), Arity::Optional, true),
    row(FORCE, Some(b'f'), Some("force"), Arity::None, true),
    row(ORPHAN, None, Some("orphan"), Arity::Required, true),
    row(UNREAD, None, Some("overwrite-ignore"), Arity::None, true),
    row(
        UNREAD,
        None,
        Some("ignore-other-worktrees"),
        Arity::None,
        true,
    ),
    row(STAGE, Some(b'2'), Some("ours"), Arity::None, false),
    row(STAGE, Some(b'3'), Some("theirs"), Arity::None, false),
    row(PATCH, Some(b'p'), Some("patch"), Arity::None, true),
    row(UNREAD, Some(b'U'), Some("unified"), Arity::Required, false),
    row(
        UNREAD,
        None,
        Some("inter-hunk-context"),
        Arity::Required,
        false,
    ),
    row(
        UNREAD,
        None,
        Some("ignore-skip-worktree-bits"),
        Arity::None,
        true,
    ),
    row(
        PATHSPEC_FROM_FILE,
        None,
        Some("pathspec-from-file"),
        Arity::Required,
        true,
    ),
    row(UNREAD, None, Some("pathspec-file-nul"), Arity::None, true),
];

/// What:
///  `git switch`:
///  `switch_options`,
///  then the common and switch-branch groups.
/// Why:
///   `switch` has no path options;
///  `-c` and `-C` have long names and can be negated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const SWITCH_TABLE = [row(NEW_BRANCH, 'c', 'create', 'required', true), /* ... */] as const;
/// ```
pub const SWITCH_TABLE: &[OptionSpec] = &[
    row(
        NEW_BRANCH,
        Some(b'c'),
        Some("create"),
        Arity::Required,
        true,
    ),
    row(
        NEW_BRANCH_FORCE,
        Some(b'C'),
        Some("force-create"),
        Arity::Required,
        true,
    ),
    row(GUESS, None, Some("guess"), Arity::None, true),
    row(DISCARD_CHANGES, None, Some("discard-changes"), Arity::None, true),
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(
        UNREAD,
        None,
        Some("recurse-submodules"),
        Arity::Optional,
        true,
    ),
    row(UNREAD, None, Some("progress"), Arity::None, true),
    row(UNREAD, Some(b'm'), Some("merge"), Arity::None, true),
    row(UNREAD, None, Some("conflict"), Arity::Required, true),
    row(DETACH, Some(b'd'), Some("detach"), Arity::None, true),
    row(TRACK, Some(b't'), Some("track"), Arity::Optional, true),
    row(FORCE, Some(b'f'), Some("force"), Arity::None, true),
    row(ORPHAN, None, Some("orphan"), Arity::Required, true),
    row(UNREAD, None, Some("overwrite-ignore"), Arity::None, true),
    row(
        UNREAD,
        None,
        Some("ignore-other-worktrees"),
        Arity::None,
        true,
    ),
];
