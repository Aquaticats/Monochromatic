//! What:
//!  The option groups of `git config` in Git 2.56.0,
//!  one constant per group.
//! Why:
//!  Git builds each `git config` subcommand's table from shared groups
//!      (`CONFIG_LOCATION_OPTIONS`,
//!  `CONFIG_TYPE_OPTIONS`,
//!  `CONFIG_DISPLAY_OPTIONS`,
//!      builtin/config.c:67-117) plus a few rows of its own.
//!  Keeping the same groups keeps
//!      one copy of every row.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const LOCATION_ROWS: readonly OptionSpec[] = [ ... ];
//! ```

/// What:
///  Bring the row type,
///  its builder,
///  the arity names and the two tokenizer modes into
///       this file.
///  `UNREAD` is the identifier of rows no config fact reads.
/// Why:
///   The groups are data for the shared tokenizer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, type OptionSpec, row, UNREAD } from './command_options.ts';
/// ```
use crate::command_options::{Arity, OptionSpec, ParseMode, UNREAD, row};

/// `--global`:
///  use the per-user configuration file.
pub const GLOBAL: u16 = 1;
/// `--system`:
///  use the system-wide configuration file.
pub const SYSTEM: u16 = 2;
/// `-l`,
///  `--list` of the form without a subcommand word:
///  list every variable.
pub const LIST: u16 = 3;
/// `--local`:
///  the repository's own configuration file,
///  which every worktree of the repository shares,
///  and the file Git reads when no scope is named.
pub const LOCAL: u16 = 4;
/// `--worktree`:
///  this worktree's own configuration file,
///  which Git reads only once `extensions.worktreeConfig` is on.
pub const WORKTREE: u16 = 5;
/// `--file`:
///  a file the command names itself.
pub const FILE: u16 = 6;
/// `--blob`:
///  a blob the command names itself.
pub const BLOB: u16 = 7;
/// `--replace-all`:
///  replace the values a pattern selects.
pub const REPLACE_ALL: u16 = 8;
/// `--add`:
///  store a value beside the existing ones.
pub const ADD: u16 = 9;
/// `--unset`:
///  remove one value.
pub const UNSET: u16 = 10;
/// `--unset-all`:
///  remove every value.
pub const UNSET_ALL: u16 = 11;
/// `--rename-section`:
///  rename a section.
pub const RENAME_SECTION: u16 = 12;
/// `--remove-section`:
///  remove a section.
pub const REMOVE_SECTION: u16 = 13;
/// `--edit` of the form without a subcommand word:
///  open the file in the editor.
pub const EDIT: u16 = 14;
/// `--get`,
///  `--get-all`,
///  `--get-regexp`,
///  `--get-urlmatch`,
///  `--get-color` and `--get-colorbool`:
///  read the file.
pub const GET: u16 = 15;

/// What:
///  `CONFIG_LOCATION_OPTIONS` (builtin/config.c:67-74).
///  `&[OptionSpec]` is a borrowed
///       table baked into the program;
///  `Some(b'f')` is "the byte `f`",
///  `None` "no letter".
/// Why:
///   Every `git config` form starts with these rows;
///  `--file` and `--blob` take a
///       value,
///  so a token after them is never an option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const LOCATION_ROWS = [row(GLOBAL, undefined, 'global', 'none', true), /* ... */] as const;
/// ```
pub const LOCATION_ROWS: &[OptionSpec] = &[
    row(GLOBAL, None, Some("global"), Arity::None, true),
    row(SYSTEM, None, Some("system"), Arity::None, true),
    row(LOCAL, None, Some("local"), Arity::None, true),
    row(WORKTREE, None, Some("worktree"), Arity::None, true),
    row(FILE, Some(b'f'), Some("file"), Arity::Required, true),
    row(BLOB, None, Some("blob"), Arity::Required, true),
];

/// What:
///  `CONFIG_TYPE_OPTIONS` (builtin/config.c:101-109).
/// Why:
///   The six type names are `PARSE_OPT_NOARG | PARSE_OPT_NONEG` (config.c:140-149).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const TYPE_ROWS = [row(UNREAD, 't', 'type', 'required', true), /* ... */] as const;
/// ```
pub const TYPE_ROWS: &[OptionSpec] = &[
    row(UNREAD, Some(b't'), Some("type"), Arity::Required, true),
    row(UNREAD, None, Some("bool"), Arity::None, false),
    row(UNREAD, None, Some("int"), Arity::None, false),
    row(UNREAD, None, Some("bool-or-int"), Arity::None, false),
    row(UNREAD, None, Some("bool-or-str"), Arity::None, false),
    row(UNREAD, None, Some("path"), Arity::None, false),
    row(UNREAD, None, Some("expiry-date"), Arity::None, false),
];

/// What:
///  The rows `CONFIG_DISPLAY_OPTIONS` adds before the type rows (config.c:111-117).
/// Why:
///   Git appends `CONFIG_TYPE_OPTIONS` to this group,
///  so users list both in that order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const DISPLAY_ROWS = [row(UNREAD, 'z', 'null', 'none', true), /* ... */] as const;
/// ```
pub const DISPLAY_ROWS: &[OptionSpec] = &[
    row(UNREAD, Some(b'z'), Some("null"), Arity::None, true),
    row(UNREAD, None, Some("name-only"), Arity::None, true),
    row(UNREAD, None, Some("show-origin"), Arity::None, true),
    row(UNREAD, None, Some("show-scope"), Arity::None, true),
    row(UNREAD, None, Some("show-names"), Arity::None, true),
];

/// `git config list`:
///  the row after the display group (config.c:1043-1050).
pub const LIST_OTHER_ROWS: &[OptionSpec] =
    &[row(UNREAD, None, Some("includes"), Arity::None, true)];

/// `git config get`:
///  the filter rows between location and display (config.c:1084-1089).
pub const GET_FILTER_ROWS: &[OptionSpec] = &[
    row(UNREAD, None, Some("all"), Arity::None, true),
    row(UNREAD, None, Some("regexp"), Arity::None, true),
    row(UNREAD, None, Some("value"), Arity::Required, true),
    row(UNREAD, None, Some("fixed-value"), Arity::None, true),
    row(UNREAD, None, Some("url"), Arity::Required, true),
];

/// `git config get`:
///  the rows after the display group (config.c:1091-1095).
pub const GET_OTHER_ROWS: &[OptionSpec] = &[
    row(UNREAD, None, Some("includes"), Arity::None, true),
    row(UNREAD, None, Some("default"), Arity::Required, true),
];

/// `git config set`:
///  the rows after location and type (config.c:1142-1148).
pub const SET_ROWS: &[OptionSpec] = &[
    row(UNREAD, None, Some("all"), Arity::None, true),
    row(UNREAD, None, Some("value"), Arity::Required, true),
    row(UNREAD, None, Some("fixed-value"), Arity::None, true),
    row(UNREAD, None, Some("comment"), Arity::Required, true),
    row(UNREAD, None, Some("append"), Arity::None, true),
];

/// `git config unset`:
///  the rows after location (config.c:1201-1204).
pub const UNSET_ROWS: &[OptionSpec] = &[
    row(UNREAD, None, Some("all"), Arity::None, true),
    row(UNREAD, None, Some("value"), Arity::Required, true),
    row(UNREAD, None, Some("fixed-value"), Arity::None, true),
];

/// What:
///  The action rows of the form without a subcommand word (config.c:1373-1387).
/// Why:
///   Each is an `OPT_CMDMODE`,
///  which takes no value and has no `--no-` form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const LEGACY_ACTION_ROWS = [row(UNREAD, undefined, 'get', 'none', false), /* ... */] as const;
/// ```
pub const LEGACY_ACTION_ROWS: &[OptionSpec] = &[
    row(GET, None, Some("get"), Arity::None, false),
    row(GET, None, Some("get-all"), Arity::None, false),
    row(GET, None, Some("get-regexp"), Arity::None, false),
    row(GET, None, Some("get-urlmatch"), Arity::None, false),
    row(REPLACE_ALL, None, Some("replace-all"), Arity::None, false),
    row(ADD, None, Some("add"), Arity::None, false),
    row(UNSET, None, Some("unset"), Arity::None, false),
    row(UNSET_ALL, None, Some("unset-all"), Arity::None, false),
    row(RENAME_SECTION, None, Some("rename-section"), Arity::None, false),
    row(REMOVE_SECTION, None, Some("remove-section"), Arity::None, false),
    row(LIST, Some(b'l'), Some("list"), Arity::None, false),
    row(EDIT, Some(b'e'), Some("edit"), Arity::None, false),
    row(GET, None, Some("get-color"), Arity::None, false),
    row(GET, None, Some("get-colorbool"), Arity::None, false),
];

/// The form without a subcommand word:
///  the rows after the display group (config.c:1389-1395).
pub const LEGACY_OTHER_ROWS: &[OptionSpec] = &[
    row(UNREAD, None, Some("default"), Arity::Required, true),
    row(UNREAD, None, Some("comment"), Arity::Required, true),
    row(UNREAD, None, Some("fixed-value"), Arity::None, true),
    row(UNREAD, None, Some("includes"), Arity::None, true),
];

/// `PARSE_OPT_STOP_AT_NON_OPTION`:
///  every form except `list` and `edit` stops reading
/// options at the first name (config.c:1100-1101,
///  1155-1156,
///  1209-1210,
///  1241-1242,
/// 1271-1272,
///  1403-1405).
pub const CONFIG_STOP_MODE: ParseMode = ParseMode {
    keep_unknown: false,
    stop_at_non_option: true,
};
