//! What:
//!  The Git token tables the argument generator and invariants draw from.
//! Why:
//!  Keeping the spellings apart from the logic lets each list be reviewed against
//!      Git's documentation on its own;
//!  the inspection and mutation lists restate the
//!      subject's rules independently instead of importing them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export const TOKENS = ['-C', '-c', '--git-dir', ...];
//! ```

/// What:
///  Tokens the structured generator draws from.
///       `&[&[u8]]` is a borrowed list of borrowed byte strings compiled into the program.
/// Why:
///   Random bytes rarely spell a Git option;
///  drawing from real option,
///  command and
///       boundary spellings reaches the deep branches of both classifiers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TOKENS = ['-C', '-c', '--git-dir', ...];
/// ```
pub(crate) const TOKENS: &[&[u8]] = &[
    b"-C",
    b"-c",
    b"--git-dir",
    b"--work-tree",
    b"--namespace",
    b"--config-env",
    b"--shallow-file",
    b"--attr-source",
    b"--git-dir=x",
    b"--work-tree=",
    b"--exec-path",
    b"--exec-path=/x",
    b"--help",
    b"-h",
    b"--version",
    b"-v",
    b"--html-path",
    b"--list-cmds=main",
    b"--no-pager",
    b"--bare",
    b"-p",
    b"--super-prefix",
    b"--",
    b"-",
    b"",
    b"status",
    b"log",
    b"commit",
    b"branch",
    b"tag",
    b"cli-git",
    b"--list",
    b"-l",
    b"-vl",
    b"--delete",
    b"-d",
    b"-D",
    b"-m",
    b"--force",
    b"--contains",
    b"--format",
    b"--format=-D",
    b"--sort",
    b"--color",
    b"--color=always",
    b"--column",
    b"--abbrev",
    b"--verbose",
    b"-n3",
    b"-n",
    b"--verify",
    b"--show-current",
    b"--del",
    b"--unknown",
    b"-x",
    b"name",
    b"a=b",
    b"\xff\xfe",
    b"-\xff",
    b"--\xff=",
];

/// Commands that never need policy configuration,
///  restated independently of the subject.
pub(crate) const INSPECTION_COMMANDS: &[&[u8]] = &[
    b"annotate",
    b"blame",
    b"cat-file",
    b"count-objects",
    b"describe",
    b"diff",
    b"diff-files",
    b"diff-index",
    b"diff-tree",
    b"for-each-ref",
    b"grep",
    b"help",
    b"log",
    b"ls-files",
    b"ls-remote",
    b"ls-tree",
    b"merge-base",
    b"name-rev",
    b"rev-list",
    b"rev-parse",
    b"shortlog",
    b"show",
    b"show-branch",
    b"show-ref",
    b"status",
    b"version",
    b"whatchanged",
];

/// Flags that make `git branch` change refs or branch metadata.
pub(crate) const BRANCH_MUTATIONS: &[&[u8]] = &[
    b"-c",
    b"-C",
    b"-d",
    b"-D",
    b"-f",
    b"-m",
    b"-M",
    b"-u",
    b"-t",
    b"-vd",
    b"--copy",
    b"--delete",
    b"--delete=x",
    b"--delete-merged",
    b"--edit-description",
    b"--force",
    b"--move",
    b"--set-upstream",
    b"--set-upstream-to",
    b"--set-upstream-to=origin/main",
    b"--unset-upstream",
    b"--create-reflog",
];

/// Flags that make `git tag` create,
///  replace or delete a tag.
pub(crate) const TAG_MUTATIONS: &[&[u8]] = &[
    b"-a",
    b"-d",
    b"-e",
    b"-f",
    b"-s",
    b"-u",
    b"-m",
    b"-F",
    b"-lm",
    b"--annotate",
    b"--delete",
    b"--edit",
    b"--force",
    b"--sign",
    b"--local-user",
    b"--message",
    b"--message=x",
    b"--file",
    b"--trailer",
    b"--create-reflog",
];
