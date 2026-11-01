//! What:
//!  The complete option table of `git commit` in Git 2.56.0.
//! Why:
//!  Declaring every option Git accepts removes the guess about whether an undeclared
//!      option consumes the next token.
//!  Rows follow `builtin_commit_options`
//!      (builtin/commit.c:1705-1785) in source order,
//!  which the completion control compares.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const COMMIT_TABLE: readonly OptionSpec[] = [ ... ];
//! ```

/// What:
///  Bring the row type,
///  its builder and the arity names into this file.
///  `UNREAD` is the
///       identifier of rows no commit fact reads.
/// Why:
///   The table is data for the shared tokenizer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Arity, type OptionSpec, row, UNREAD } from './command_options.ts';
/// ```
use super::command_options::{Arity, OptionSpec, UNREAD, row};

/// `-a`,
///  `--all`:
///  stage every tracked modification before committing.
pub const ALL: u16 = 1;
/// `-o`,
///  `--only`:
///  commit only the named paths.
pub const ONLY: u16 = 2;
/// `-i`,
///  `--include`:
///  add the named paths to the index for this commit.
pub const INCLUDE: u16 = 3;
/// `--interactive`:
///  choose content interactively.
pub const INTERACTIVE: u16 = 4;
/// `-p`,
///  `--patch`:
///  choose hunks interactively.
pub const PATCH: u16 = 5;
/// `--dry-run`:
///  show what would be committed.
pub const DRY_RUN: u16 = 6;
/// `--short`:
///  status format that implies a dry run.
pub const SHORT: u16 = 7;
/// `--porcelain`:
///  status format that implies a dry run.
pub const PORCELAIN: u16 = 8;
/// `--long`:
///  status format that implies a dry run.
pub const LONG: u16 = 9;
/// `-z`,
///  `--null`:
///  NUL-terminated status output,
///  which implies a dry run.
pub const NULL: u16 = 10;
/// `--amend`:
///  replace the tip commit.
pub const AMEND: u16 = 11;
/// `--allow-empty`:
///  permit a commit that records no change.
pub const ALLOW_EMPTY: u16 = 12;
/// `--fixup=[(amend|reword):]<commit>`.
pub const FIXUP: u16 = 13;
/// `--pathspec-from-file=<file>`.
pub const PATHSPEC_FROM_FILE: u16 = 14;
/// `--pathspec-file-nul`.
pub const PATHSPEC_FILE_NUL: u16 = 15;

/// What:
///  `pub const NAME: &[OptionSpec] = &[...]` is a table baked into the program.
///       `&[T]` is a borrowed list;
///  `Some(b'q')` is "the byte `q`";
///  `None` is "no letter".
/// Why:
///   Each row copies one Git declaration:
///  letter,
///  name,
///  value arity and whether
///       `--no-<name>` is accepted (false only for `PARSE_OPT_NONEG`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const COMMIT_TABLE = [row(UNREAD, 'q', 'quiet', 'none', true), /* ... */] as const;
/// ```
pub const COMMIT_TABLE: &[OptionSpec] = &[
    row(UNREAD, Some(b'q'), Some("quiet"), Arity::None, true),
    row(UNREAD, Some(b'v'), Some("verbose"), Arity::None, true),
    row(UNREAD, Some(b'F'), Some("file"), Arity::Required, true),
    row(UNREAD, None, Some("author"), Arity::Required, true),
    row(UNREAD, None, Some("date"), Arity::Required, true),
    row(UNREAD, Some(b'm'), Some("message"), Arity::Required, true),
    row(
        UNREAD,
        Some(b'c'),
        Some("reedit-message"),
        Arity::Required,
        true,
    ),
    row(
        UNREAD,
        Some(b'C'),
        Some("reuse-message"),
        Arity::Required,
        true,
    ),
    row(FIXUP, None, Some("fixup"), Arity::Required, true),
    row(UNREAD, None, Some("squash"), Arity::Required, true),
    row(UNREAD, None, Some("reset-author"), Arity::None, true),
    row(UNREAD, None, Some("trailer"), Arity::Required, true),
    row(UNREAD, Some(b's'), Some("signoff"), Arity::None, true),
    row(UNREAD, Some(b't'), Some("template"), Arity::Required, true),
    row(UNREAD, Some(b'e'), Some("edit"), Arity::None, true),
    row(UNREAD, None, Some("cleanup"), Arity::Required, true),
    row(UNREAD, None, Some("status"), Arity::None, true),
    // `PARSE_OPT_OPTARG`: `-S<key>` and `--gpg-sign=<key>` only (commit.c:1730-1739).
    row(UNREAD, Some(b'S'), Some("gpg-sign"), Arity::Optional, true),
    row(ALL, Some(b'a'), Some("all"), Arity::None, true),
    row(INCLUDE, Some(b'i'), Some("include"), Arity::None, true),
    row(INTERACTIVE, None, Some("interactive"), Arity::None, true),
    row(PATCH, Some(b'p'), Some("patch"), Arity::None, true),
    // `OPT_DIFF_UNIFIED` and `OPT_DIFF_INTERHUNK_CONTEXT` are `PARSE_OPT_NONEG`
    // (parse-options.h:633-634).
    row(UNREAD, Some(b'U'), Some("unified"), Arity::Required, false),
    row(
        UNREAD,
        None,
        Some("inter-hunk-context"),
        Arity::Required,
        false,
    ),
    row(ONLY, Some(b'o'), Some("only"), Arity::None, true),
    row(UNREAD, Some(b'n'), Some("no-verify"), Arity::None, true),
    row(DRY_RUN, None, Some("dry-run"), Arity::None, true),
    row(SHORT, None, Some("short"), Arity::None, true),
    row(UNREAD, None, Some("branch"), Arity::None, true),
    row(UNREAD, None, Some("ahead-behind"), Arity::None, true),
    // Unlike `git status`, `git commit --porcelain` takes no value (commit.c:1757-1758).
    row(PORCELAIN, None, Some("porcelain"), Arity::None, true),
    row(LONG, None, Some("long"), Arity::None, true),
    row(NULL, Some(b'z'), Some("null"), Arity::None, true),
    row(AMEND, None, Some("amend"), Arity::None, true),
    row(UNREAD, None, Some("no-post-rewrite"), Arity::None, true),
    // `PARSE_OPT_OPTARG`: `-u<mode>` and `--untracked-files=<mode>` only (commit.c:1766-1775).
    row(
        UNREAD,
        Some(b'u'),
        Some("untracked-files"),
        Arity::Optional,
        true,
    ),
    row(
        PATHSPEC_FROM_FILE,
        None,
        Some("pathspec-from-file"),
        Arity::Required,
        true,
    ),
    row(
        PATHSPEC_FILE_NUL,
        None,
        Some("pathspec-file-nul"),
        Arity::None,
        true,
    ),
    row(ALLOW_EMPTY, None, Some("allow-empty"), Arity::None, true),
    row(UNREAD, None, Some("allow-empty-message"), Arity::None, true),
];
