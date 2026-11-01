//! What:
//!  The token tables the control generators draw from.
//! Why:
//!  Keeping the spellings apart from the logic lets each list be reviewed on its own.
//!      Control spellings are restated here instead of imported,
//!  so a spelling dropped
//!      from the subject's table shows up as a control that is no longer removed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const CONTROL_TOKENS = ['commit', '--cli-git-keep-going', '--', ...];
//! ```

/// What:
///  Every wrapper control and hatch as the installed wrapper spells it.
///       `&[&[u8]]` is a borrowed list of byte strings baked into the program.
///       `#[cfg(test)]` compiles the list only for the generator controls.
/// Why:
///   The generator controls compare this list with what the subject recognizes;
///  the
///       fuzz targets need only the generator table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WRAPPER_SPELLINGS = ['--cli-git-keep-going', '--no-enforce-require-root', ...];
/// ```
#[cfg(test)]
pub(crate) const WRAPPER_SPELLINGS: &[&[u8]] = &[
    b"--cli-git-keep-going",
    b"--no-enforce-require-root",
    b"--no-enforce-linked-worktree-only",
    b"--no-enforce-branch-worktree-only",
    b"--no-enforce-add-explicit",
    b"--no-enforce-final-newline",
    b"--no-enforce-markdown/autofix",
    b"--no-enforce-mono/forbidden-root-context",
    b"--no-enforce-mono/dependent-version-bump",
    b"--no-enforce-security/forbidden-strings",
    b"--no-enforce-worktree",
    b"--no-enforce-worktree-branch",
    b"--no-enforce-bulk-add",
    b"--no-worktree-copy",
    b"--no-enforce-only",
];

/// What:
///  Commands,
///  Git options,
///  separators,
///  values and every wrapper spelling,
///  plus
///       near-misses of the spellings and bytes that are not UTF-8.
/// Why:
///   One byte of fuzz input selects one token,
///  so short inputs already place a
///       control before a command,
///  after a value option,
///  or behind a separator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONTROL_TOKENS = [...commands, ...options, ...WRAPPER_SPELLINGS, ...nearMisses];
/// ```
pub(crate) const CONTROL_TOKENS: &[&[u8]] = &[
    // Commands with a ported option table, without one, and words that are not commands.
    b"commit",
    b"add",
    b"reset",
    b"clean",
    b"stash",
    b"push",
    b"status",
    b"config",
    b"branch",
    b"checkout",
    b"switch",
    b"worktree",
    b"fetch",
    b"log",
    b"st",
    b"wta",
    b"cli-git",
    // Git's own global options and their values.
    b"-C",
    b"dir",
    b"-c",
    b"a=b",
    b"--no-pager",
    b"--version",
    b"--git-dir=x",
    // The wrapper's controls and hatches.
    b"--cli-git-keep-going",
    b"--no-enforce-require-root",
    b"--no-enforce-linked-worktree-only",
    b"--no-enforce-branch-worktree-only",
    b"--no-enforce-add-explicit",
    b"--no-enforce-final-newline",
    b"--no-enforce-markdown/autofix",
    b"--no-enforce-mono/forbidden-root-context",
    b"--no-enforce-mono/dependent-version-bump",
    b"--no-enforce-security/forbidden-strings",
    b"--no-enforce-worktree",
    b"--no-enforce-worktree-branch",
    b"--no-enforce-bulk-add",
    b"--no-worktree-copy",
    b"--no-enforce-only",
    // Near-misses that must stay where they are.
    b"--cli-git-keep-goin",
    b"--cli-git-keep-going=1",
    b"--no-enforce-",
    b"--no-enforce-worktree=1",
    b"--NO-ENFORCE-WORKTREE",
    b"--no-enforce-only\xff",
    // Separators.
    b"--",
    b"--end-of-options",
    // Options that take the next token as their value.
    b"-m",
    b"--message",
    b"-F",
    b"-b",
    b"-B",
    b"-c",
    b"-u",
    b"--pathspec-from-file",
    b"--author",
    b"-o",
    b"--push-option",
    // Options without a value, and one Git refuses.
    b"-a",
    b"--all",
    b"-A",
    b"--hard",
    b"--soft",
    b"-n",
    b"--dry-run",
    b"--no-dry-run",
    b"--dry",
    b"-f",
    b"-q",
    b"--amend",
    b"--atomic",
    b"--short",
    b"--list",
    b"--no-such-option",
    b"-nq",
    b"--message=--no-enforce-only",
    // Positional words, including the worktree and stash subcommands.
    b".",
    b"file",
    b"HEAD",
    b"origin",
    b"topic",
    b"move",
    b"list",
    b"pop",
    b"",
    b"\xff\xfe",
    b"-\xff",
];

/// What:
///  The commands whose options the subject reads with Git's own table.
/// Why:
///   The separated generator starts every list with one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TABLE_COMMANDS = ['commit', 'add', 'reset', ...];
/// ```
pub(crate) const TABLE_COMMANDS: &[&[u8]] = &[
    b"commit",
    b"add",
    b"reset",
    b"clean",
    b"stash",
    b"push",
    b"status",
    b"branch",
    b"checkout",
    b"switch",
];

/// What:
///  Tokens that never take the following token as their value in any table command:
///       wrapper controls,
///  a few flags,
///  and options Git refuses.
/// Why:
///   With only these before it,
///  a `--` is certainly Git's separator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const VALUELESS_TOKENS = ['--cli-git-keep-going', '--no-enforce-worktree', '-q', ...];
/// ```
pub(crate) const VALUELESS_TOKENS: &[&[u8]] = &[
    b"--cli-git-keep-going",
    b"--no-enforce-require-root",
    b"--no-enforce-final-newline",
    b"--no-enforce-worktree",
    b"--no-enforce-worktree-branch",
    b"--no-enforce-bulk-add",
    b"--no-worktree-copy",
    b"--no-enforce-only",
    b"-q",
    b"--quiet",
    b"-f",
    b"--force",
    b"--dry-run",
    b"--no-such-option",
];
