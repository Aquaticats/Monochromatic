//! What:
//!  The complete option table of `git branch` in Git 2.56.0.
//! Why:
//!  Whether `git branch <name>` creates a branch depends on which action options were
//!      given,
//!  and whether a token is a name depends on each option's arity.
//!  Rows follow
//!      `options[]` in `cmd_branch` (builtin/branch.c:992-1050) in source order.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const BRANCH_TABLE: readonly OptionSpec[] = [ ... ];
//! ```

/// What:
///  Bring the row type,
///  its builder and the arity names into this file.
///  `UNREAD` is the
///       identifier of rows no branch fact reads.
/// Why:
///   The table is data for the shared tokenizer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, type OptionSpec, row, UNREAD } from './command_options.ts';
/// ```
use super::command_options::{Arity, OptionSpec, UNREAD, row};

/// `-t`,
///  `--track[=(direct|inherit)]`:
///  writes the tracking mode.
pub const TRACK: u16 = 1;
/// `--set-upstream`:
///  hidden,
///  writes the tracking mode Git refuses when creating.
pub const SET_UPSTREAM: u16 = 2;
/// `-u`,
///  `--set-upstream-to=<upstream>`.
pub const SET_UPSTREAM_TO: u16 = 3;
/// `--unset-upstream`.
pub const UNSET_UPSTREAM: u16 = 4;
/// `-r`/`--remotes` and `-a`/`--all`:
///  both change the listed kind of ref.
pub const KIND: u16 = 5;
/// `--contains`,
///  `--no-contains`,
///  `--with`,
///  `--without`,
///  `--merged`,
///  `--no-merged`,
///  `--forked`.
pub const FILTER: u16 = 6;
/// `-d`,
///  `--delete`.
pub const DELETE: u16 = 7;
/// `-D`.
pub const DELETE_FORCE: u16 = 8;
/// `-m`,
///  `--move`.
pub const MOVE: u16 = 9;
/// `-M`.
pub const MOVE_FORCE: u16 = 10;
/// `-c`,
///  `--copy`.
pub const COPY: u16 = 11;
/// `-C`.
pub const COPY_FORCE: u16 = 12;
/// `-l`,
///  `--list`.
pub const LIST: u16 = 13;
/// `--show-current`.
pub const SHOW_CURRENT: u16 = 14;
/// `--edit-description`.
pub const EDIT_DESCRIPTION: u16 = 15;
/// `--delete-merged=<pattern>`.
pub const DELETE_MERGED: u16 = 16;
/// `--dry-run`,
///  valid only with `--delete-merged`.
pub const DRY_RUN: u16 = 17;
/// `--points-at=<object>`.
pub const POINTS_AT: u16 = 18;
/// `--recurse-submodules`:
///  Git accepts it only when creating a branch.
pub const RECURSE_SUBMODULES: u16 = 19;

/// What:
///  `pub const NAME: &[OptionSpec] = &[...]` is a table baked into the program.
/// Why:
///   `--contains` and its relatives are `PARSE_OPT_LASTARG_DEFAULT`:
///  they take the next
///       token unless they are last.
///  `--color`,
///  `--abbrev`,
///  `--column` and `--track` take
///       an attached value only,
///  so `git branch --color name` creates `name`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const BRANCH_TABLE = [row(UNREAD, 'v', 'verbose', 'none', true), /* ... */] as const;
/// ```
pub const BRANCH_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'v'), Some("verbose"), Arity::None, true),
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(TRACK, Some(b't'), Some("track"), Arity::Optional, true),
    row(SET_UPSTREAM, None, Some("set-upstream"), Arity::None, true),
    row(
        SET_UPSTREAM_TO,
        Some(b'u'),
        Some("set-upstream-to"),
        Arity::Required,
        true,
    ),
    row(
        UNSET_UPSTREAM,
        None,
        Some("unset-upstream"),
        Arity::None,
        true,
    ),
    row(UNREAD, None, Some("color"), Arity::Optional, true),
    row(KIND, Some(b'r'), Some("remotes"), Arity::None, false),
    row(FILTER, None, Some("contains"), Arity::LastArgDefault, false),
    row(
        FILTER,
        None,
        Some("no-contains"),
        Arity::LastArgDefault,
        false,
    ),
    row(FILTER, None, Some("with"), Arity::LastArgDefault, false),
    row(FILTER, None, Some("without"), Arity::LastArgDefault, false),
    row(UNREAD, None, Some("abbrev"), Arity::Optional, true),
    row(KIND, Some(b'a'), Some("all"), Arity::None, false),
    row(DELETE, Some(b'd'), Some("delete"), Arity::None, true),
    row(DELETE_FORCE, Some(b'D'), None, Arity::None, true),
    row(MOVE, Some(b'm'), Some("move"), Arity::None, true),
    row(MOVE_FORCE, Some(b'M'), None, Arity::None, true),
    row(UNREAD, None, Some("omit-empty"), Arity::None, true),
    row(COPY, Some(b'c'), Some("copy"), Arity::None, true),
    row(COPY_FORCE, Some(b'C'), None, Arity::None, true),
    row(LIST, Some(b'l'), Some("list"), Arity::None, true),
    row(SHOW_CURRENT, None, Some("show-current"), Arity::None, true),
    row(UNREAD, None, Some("create-reflog"), Arity::None, true),
    row(
        EDIT_DESCRIPTION,
        None,
        Some("edit-description"),
        Arity::None,
        true,
    ),
    row(
        DELETE_MERGED,
        None,
        Some("delete-merged"),
        Arity::Required,
        false,
    ),
    row(DRY_RUN, None, Some("dry-run"), Arity::None, true),
    row(UNREAD, Some(b'f'), Some("force"), Arity::None, true),
    row(FILTER, None, Some("merged"), Arity::LastArgDefault, false),
    row(
        FILTER,
        None,
        Some("no-merged"),
        Arity::LastArgDefault,
        false,
    ),
    row(FILTER, None, Some("forked"), Arity::Required, false),
    row(UNREAD, None, Some("column"), Arity::Optional, true),
    row(UNREAD, None, Some("sort"), Arity::Required, true),
    row(POINTS_AT, None, Some("points-at"), Arity::Required, true),
    row(UNREAD, Some(b'i'), Some("ignore-case"), Arity::None, true),
    row(
        RECURSE_SUBMODULES,
        None,
        Some("recurse-submodules"),
        Arity::None,
        true,
    ),
    row(UNREAD, None, Some("format"), Arity::Required, true),
];
