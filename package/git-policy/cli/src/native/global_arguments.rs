//! What:
//!  Locate Git 2.56.0 command boundaries without reconstructing native arguments.
//! Why:
//!  Policy stages need the command index,
//!  but Git remains authoritative for option validation and -C semantics.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Scan argument boundaries; forward the original global prefix to every repository query.
//! ```

/// Import native argument storage;
///  no lossy UTF-8 conversion occurs.
use std::ffi::OsString;

/// Which native Git path follows the global prefix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlobalOutcome {
    /// A subcommand starts at prefix_len.
    Command,
    /// Git prints help/version/path/command inventory without executing a repository command.
    ///  `--version`, `-v`, `--help` and `-h` run Git as a child and add the wrapper's text after Git's answer
    ///  (see `announced_query`); every other query is forwarded by replacing the process.
    Query,
    /// A value-taking global option has no following token;
    ///  Git must render its own error.
    MissingValue,
    /// Git 2.56.0 does not recognize this global option;
    ///  do not reinterpret it as a subcommand.
    InvalidOption,
    /// The argument list ended without a subcommand.
    NoCommand,
}

/// Immutable positions into the caller's untouched argument vector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlobalLayout {
    /// Count of complete native globals before the command/query/error token.
    pub prefix_len: usize,
    /// Native interpretation after the completed global prefix.
    pub outcome: GlobalOutcome,
}

/// Complete separated-value globals accepted by Git 2.56.0 git.c::handle_options.
pub(crate) const VALUE_OPTIONS: &[&[u8]] = &[
    b"-C",
    b"-c",
    b"--git-dir",
    b"--work-tree",
    b"--namespace",
    b"--config-env",
    b"--shallow-file",
    b"--attr-source",
];

/// Complete no-value globals accepted by that same release,
///  not a cross-version fallback list.
const FLAG_OPTIONS: &[&[u8]] = &[
    b"-p",
    b"--paginate",
    b"-P",
    b"--no-pager",
    b"--no-lazy-fetch",
    b"--no-replace-objects",
    b"--bare",
    b"--literal-pathspecs",
    b"--no-literal-pathspecs",
    b"--glob-pathspecs",
    b"--noglob-pathspecs",
    b"--icase-pathspecs",
    b"--no-optional-locks",
    b"--no-advice",
];

/// Inline-value forms explicitly recognized by the native release.
const INLINE_OPTIONS: &[&[u8]] = &[
    b"--git-dir=",
    b"--work-tree=",
    b"--namespace=",
    b"--config-env=",
    b"--attr-source=",
];

/// What:
///  Find a command/query/error boundary with one forward scan.
/// Why:
///  Values such as '-h',
///  empty -C paths and non-UTF-8 filenames remain original opaque argument values.
/// Symlink-sensitive -C chaining is not approximated lexically;
///  repository queries forward the complete prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function globalLayout(nativeArgs): { prefixLen: number; outcome: GlobalOutcome };
/// ```
pub fn global_layout(arguments: &[OsString]) -> GlobalLayout {
    // True while the token being visited is the value of the separated-value option before it.
    let mut is_value: bool = false;
    // `.iter().enumerate()` visits every argument once with its position, so the scan
    // always ends: there is no hand-stepped index that a mistake could leave standing still.
    for (index, token) in arguments.iter().enumerate() {
        if is_value {
            is_value = false;
            continue;
        }
        // Borrow native encoded bytes solely to recognize Git's ASCII option syntax.
        let argument: &[u8] = token.as_encoded_bytes();
        if !argument.starts_with(b"-") {
            return GlobalLayout {
                prefix_len: index,
                outcome: GlobalOutcome::Command,
            };
        }
        if [
            b"--help".as_slice(),
            b"-h",
            b"--version",
            b"-v",
            b"--html-path",
            b"--man-path",
            b"--info-path",
        ]
        .contains(&argument)
            || argument.starts_with(b"--list-cmds=")
        {
            return GlobalLayout {
                prefix_len: index,
                outcome: GlobalOutcome::Query,
            };
        }
        // Git's own prefix branch treats every --exec-path suffix except '=' as a query.
        if let Some(remainder) = argument.strip_prefix(b"--exec-path") {
            if remainder.first() != Some(&b'=') {
                return GlobalLayout {
                    prefix_len: index,
                    outcome: GlobalOutcome::Query,
                };
            }
            continue;
        }
        if VALUE_OPTIONS.contains(&argument) {
            if index + 1 == arguments.len() {
                return GlobalLayout {
                    prefix_len: index,
                    outcome: GlobalOutcome::MissingValue,
                };
            }
            is_value = true;
            continue;
        }
        if FLAG_OPTIONS.contains(&argument) {
            continue;
        }
        let mut inline: bool = false;
        for prefix in INLINE_OPTIONS {
            if argument.starts_with(prefix) {
                inline = true;
                break;
            }
        }
        if inline {
            continue;
        }
        return GlobalLayout {
            prefix_len: index,
            outcome: GlobalOutcome::InvalidOption,
        };
    }
    return GlobalLayout {
        prefix_len: arguments.len(),
        outcome: GlobalOutcome::NoCommand,
    };
}

/// What:
///  The command word and the tokens after it,
///  when the arguments name a command.
///       `Option<(&OsString, &[OsString])>` is "a pair of borrowed views,
///  or nothing":
///       the word,
///  and the list of everything after it.
/// Why:
///   Every rule that reads a command's own options needs exactly this split.
///  Taking it
///       from one function means no rule computes an offset of its own,
///  so no rule can
///       read the command word as one of its options by an arithmetic slip.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandTokens(args: string[]): [word: string, region: string[]] | undefined;
/// ```
pub fn command_tokens(arguments: &[OsString]) -> Option<(&OsString, &[OsString])> {
    let layout: GlobalLayout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command {
        // `None` is the "absent" case of `Option`.
        return None;
    }
    // `&arguments[n..]` borrows from the command word on; `.split_first()` separates the
    // first item from the rest, or gives `None` for an empty list.
    return arguments[layout.prefix_len..].split_first();
}

/// Boundary controls include real Git probes and native non-UTF-8 arguments.
#[cfg(test)]
#[path = "global_arguments_tests.rs"]
mod tests;
