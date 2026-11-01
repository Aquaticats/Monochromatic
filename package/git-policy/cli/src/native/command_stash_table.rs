//! What:
//!  The option tables of every `git stash` subcommand in Git 2.56.0.
//! Why:
//!  `git stash` dispatches to subcommands that each parse their own options with their
//!      own flags (builtin/stash.c);
//!  whether a token is an option value depends on which
//!      subcommand reads it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const STASH_PUSH_TABLE: readonly OptionSpec[] = [ ... ];
//! ```

/// What:
///  Bring the row type,
///  its builder,
///  the arity names and the parse-mode type into
///       this file.
///  `UNREAD` is the identifier of rows no stash fact reads.
/// Why:
///   The tables are data for the shared tokenizer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, type OptionSpec, type ParseMode, row, UNREAD } from './command_options.ts';
/// ```
use super::command_options::{Arity, OptionSpec, ParseMode, UNREAD, row};

/// `git stash push` (`push_stash`,
///  builtin/stash.c:1920-1940).
pub const STASH_PUSH_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'k'), Some("keep-index"), Arity::None, true),
    row(UNREAD, Some(b'S'), Some("staged"), Arity::None, true),
    row(UNREAD, Some(b'p'), Some("patch"), Arity::None, true),
    row(UNREAD, None, Some("auto-advance"), Arity::None, true),
    row(UNREAD, Some(b'U'), Some("unified"), Arity::Required, false),
    row(
        UNREAD,
        None,
        Some("inter-hunk-context"),
        Arity::Required,
        false,
    ),
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(
        UNREAD,
        Some(b'u'),
        Some("include-untracked"),
        Arity::None,
        true,
    ),
    row(UNREAD, Some(b'a'), Some("all"), Arity::None, true),
    row(UNREAD, Some(b'm'), Some("message"), Arity::Required, true),
    row(
        UNREAD,
        None,
        Some("pathspec-from-file"),
        Arity::Required,
        true,
    ),
    row(UNREAD, None, Some("pathspec-file-nul"), Arity::None, true),
];

/// `git stash save` (`save_stash`,
///  builtin/stash.c:2027-2045):
///  `push` without the pathspec
/// file options.
pub const STASH_SAVE_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'k'), Some("keep-index"), Arity::None, true),
    row(UNREAD, Some(b'S'), Some("staged"), Arity::None, true),
    row(UNREAD, Some(b'p'), Some("patch"), Arity::None, true),
    row(UNREAD, None, Some("auto-advance"), Arity::None, true),
    row(UNREAD, Some(b'U'), Some("unified"), Arity::Required, false),
    row(
        UNREAD,
        None,
        Some("inter-hunk-context"),
        Arity::Required,
        false,
    ),
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(
        UNREAD,
        Some(b'u'),
        Some("include-untracked"),
        Arity::None,
        true,
    ),
    row(UNREAD, Some(b'a'), Some("all"), Arity::None, true),
    row(UNREAD, Some(b'm'), Some("message"), Arity::Required, true),
];

/// `git stash apply` (`apply_stash`,
///  builtin/stash.c:783-793).
pub const STASH_APPLY_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(UNREAD, None, Some("index"), Arity::None, true),
    row(UNREAD, None, Some("label-ours"), Arity::Required, true),
    row(UNREAD, None, Some("label-theirs"), Arity::Required, true),
    row(UNREAD, None, Some("label-base"), Arity::Required, true),
];

/// `git stash pop` (`pop_stash`,
///  builtin/stash.c:889-893).
pub const STASH_POP_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(UNREAD, None, Some("index"), Arity::None, true),
];

/// `git stash drop` (`drop_stash`,
///  builtin/stash.c:865-867).
pub const STASH_DROP_TABLE: &[OptionSpec] =
    &[row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true)];

/// `git stash store` (`store_stash`,
///  builtin/stash.c:1162-1166).
pub const STASH_STORE_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(UNREAD, Some(b'm'), Some("message"), Arity::Required, true),
];

/// `git stash show` (`show_stash`,
///  builtin/stash.c:1041-1048).
pub const STASH_SHOW_TABLE: &[OptionSpec] = &[
    row(
        UNREAD,
        Some(b'u'),
        Some("include-untracked"),
        Arity::None,
        true,
    ),
    row(UNREAD, None, Some("only-untracked"), Arity::None, false),
];

/// `git stash export` (`export_stash`,
///  builtin/stash.c:2434-2440).
pub const STASH_EXPORT_TABLE: &[OptionSpec] = &[
    row(UNREAD, None, Some("print"), Arity::None, false),
    row(UNREAD, None, Some("to-ref"), Arity::Required, true),
];

/// `git stash clear`,
///  `branch`,
///  `list` and `import` declare no options.
pub const STASH_EMPTY_TABLE: &[OptionSpec] = &[];

/// Flags `0` or only `PARSE_OPT_KEEP_DASHDASH`:
///  `push`,
///  `save`,
///  `apply`,
///  `pop`,
///  `drop`,
/// `branch`,
///  `export` and `import`.
pub const STASH_PLAIN_MODE: ParseMode = ParseMode {
    keep_unknown: false,
    stop_at_non_option: false,
};

/// `PARSE_OPT_KEEP_UNKNOWN_OPT`:
///  `list`,
///  `show` and `store` forward unknown options.
pub const STASH_KEEP_UNKNOWN_MODE: ParseMode = ParseMode {
    keep_unknown: true,
    stop_at_non_option: false,
};

/// `PARSE_OPT_STOP_AT_NON_OPTION`:
///  `clear`,
///  and `push` when no subcommand was written
/// (builtin/stash.c:1945-1948).
pub const STASH_STOP_MODE: ParseMode = ParseMode {
    keep_unknown: false,
    stop_at_non_option: true,
};
