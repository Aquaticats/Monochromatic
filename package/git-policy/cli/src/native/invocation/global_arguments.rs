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
    ///  `--version`,
    /// `-v`,
    /// `--help` and `-h` run Git as a child and add the wrapper's text after Git's answer
    ///  (see `announced_query`);
    /// every other query is forwarded by replacing the process.
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

/// The no-value globals of `FLAG_OPTIONS` that make Git refuse an alias expansion,
///  because they change a variable of the environment Git runs the expansion in
///  (`envchanged` in git.c::handle_options).
const ENVIRONMENT_FLAGS: &[&[u8]] = &[
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

/// The separated-value globals of `VALUE_OPTIONS` that change such a variable,
///  `-C` beside them because it moves the directory instead.
const ENVIRONMENT_VALUE_OPTIONS: &[&[u8]] = &[
    b"--git-dir",
    b"--work-tree",
    b"--namespace",
    b"--shallow-file",
    b"--attr-source",
];

/// Inline-value forms explicitly recognized by the native release.
const INLINE_OPTIONS: &[&[u8]] = &[
    b"--git-dir=",
    b"--work-tree=",
    b"--namespace=",
    b"--config-env=",
    b"--attr-source=",
];

/// The inline-value globals of `INLINE_OPTIONS` that change such a variable.
const ENVIRONMENT_INLINE_OPTIONS: &[&[u8]] = &[
    b"--git-dir=",
    b"--work-tree=",
    b"--namespace=",
    b"--attr-source=",
];

/// What:
///  Whether Git reads this token as a global option that consumes the token after it.
/// Why:
///   Two readings need the same answer:
///  the boundary scan below skips such a value,
///  and `option_tokens` leaves it out of the options.
///  A second list of the value-taking globals could disagree with this one,
///  and a rule reading a value as an option lets a command through that Git reads differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const takesValue = (token: string) => VALUE_OPTIONS.includes(token);
/// ```
pub(crate) fn takes_value(token: &[u8]) -> bool {
    return VALUE_OPTIONS.contains(&token);
}

/// What:
///  The tokens of a global prefix that Git reads as options,
///  in the order the caller wrote them,
///  with every value a separated-value option consumes left out.
/// Why:
///   A global option's value is not an option:
///  `git -C --literal-pathspecs add -- '**'` changes into the directory of that name,
///  so Git sets no pathspec mode and expands the pattern over the whole worktree,
///  and a rule that read the value as a mode would permit what Git does.
///  The values are the ones the boundary scan skips,
///  so both readings consume the same tokens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const optionTokens = (prefix: string[]) => prefix.filter((token, index) => !(index > 0 && takesValue(prefix[index - 1])));
/// ```
pub fn option_tokens(global_prefix: &[OsString]) -> Vec<&OsString> {
    let mut options: Vec<&OsString> = Vec::new();
    // True while the token being visited is the value of the separated-value option before it.
    let mut is_value: bool = false;
    for token in global_prefix {
        if is_value {
            is_value = false;
            continue;
        }
        options.push(token);
        is_value = takes_value(token.as_encoded_bytes());
    }
    return options;
}

/// What:
///  The directories the caller's `-C` options move Git through,
///  in the order Git reads them,
///  and without the empty value Git treats as no move.
/// Why:
///   A relative `PATH` entry or exec path is resolved against the directory Git runs in,
///  and `-C` moves that directory before any command is looked up
///  (`git.c:316` to `git.c:328`,
///  then `git.c:988`);
///  an empty `-C` moves Git nowhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directoryChanges = (prefix: string[]) =>
///   prefix.flatMap((token, index) => token === '-C' && prefix[index + 1] ? [prefix[index + 1]] : []);
/// ```
pub fn directory_changes(global_prefix: &[OsString]) -> Vec<OsString> {
    let mut directories: Vec<OsString> = Vec::new();
    // True while the token being visited is the value of the separated-value option before it.
    let mut is_value: bool = false;
    for (index, token) in global_prefix.iter().enumerate() {
        if is_value {
            is_value = false;
            continue;
        }
        let argument: &[u8] = token.as_encoded_bytes();
        if !takes_value(argument) {
            continue;
        }
        is_value = true;
        if argument != b"-C" {
            continue;
        }
        let Some(value) = global_prefix.get(index + 1) else {
            continue;
        };
        if value.as_encoded_bytes().is_empty() {
            continue;
        }
        directories.push(value.clone());
    }
    return directories;
}

/// What:
///  The directory the last `--exec-path=` option of a global prefix names,
///  or nothing when the prefix holds no such option.
///  A value is returned as the caller wrote it,
///  relative and empty included.
/// Why:
///   Git's own option loop sets its exec path from that option,
///  and `git_exec_path` answers with the setting before it reads the
///       environment
///  (`git.c:178` to `git.c:181`,
///  `exec-cmd.c:313` to `exec-cmd.c:323`),
///  so a reading that looked at `GIT_EXEC_PATH` first
///  would search a directory Git never searches.
///  An empty value is a real answer:
///  Git adds no executables directory for it
///  (`exec-cmd.c:325`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const execPathOption = (prefix: string[]) =>
///   prefix.filter((token, index) => !(index > 0 && takesValue(prefix[index - 1])))
///     .filter((token) => token.startsWith('--exec-path='))
///     .at(-1)?.slice('--exec-path='.length);
/// ```
pub fn exec_path_option(global_prefix: &[OsString]) -> Option<OsString> {
    /// Counted bytes of the inline spelling this reads.
    const SPELLING: &[u8] = b"--exec-path=";
    let mut value: Option<OsString> = None;
    // True while the token being visited is the value of the separated-value option before it.
    let mut is_value: bool = false;
    for token in global_prefix {
        if is_value {
            is_value = false;
            continue;
        }
        let argument: &[u8] = token.as_encoded_bytes();
        if let Some(configured) = argument.strip_prefix(SPELLING) {
            #[cfg(unix)]
            {
                /// The Unix byte-to-native-string conversion for a configured path.
                use std::os::unix::ffi::OsStrExt;
                value = Some(std::ffi::OsStr::from_bytes(configured).to_os_string());
            }
            // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
            #[cfg(not(unix))]
            {
                value = Some(OsString::from(String::from_utf8_lossy(configured).into_owned()));
            }
        }
        is_value = takes_value(argument);
    }
    return value;
}

/// What:
///  Where Git's command boundary is,
///  and whether a global before it changed a variable of the environment.
/// Why:
///   `global_layout` answers the boundary for every caller;
///  `prefix_changes_environment` answers the other
///       question from the same scan,
///  so neither can read an option differently from the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GlobalScan = { prefixLen: number; outcome: GlobalOutcome; changesEnvironment: boolean };
/// ```
struct GlobalScan {
    /// Count of globals before the first word that is not one.
    prefix_len: usize,
    /// What the scan found there.
    outcome: GlobalOutcome,
    /// Whether one of those globals sets an environment variable or moves the directory.
    changes_environment: bool,
}

/// What:
///  Scan the globals once,
///  answering both the boundary and the environment question.
/// Why:
///   Git stops its option loop at the first word that is not an option,
///  so a `-C` after the
///       command word is that command's argument and not a global.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scanGlobals(args): GlobalScan;
/// ```
fn scan_globals(arguments: &[OsString]) -> GlobalScan {
    // True while the token being visited is the value of the separated-value option before it.
    let mut is_value: bool = false;
    // Whether a global so far sets a variable; `-C` with an empty value does not.
    let mut changes_environment: bool = false;
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
            return GlobalScan {
                prefix_len: index,
                outcome: GlobalOutcome::Command,
                changes_environment,
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
            return GlobalScan {
                prefix_len: index,
                outcome: GlobalOutcome::Query,
                changes_environment,
            };
        }
        // Git's own prefix branch treats every --exec-path suffix except '=' as a query.
        if let Some(remainder) = argument.strip_prefix(b"--exec-path") {
            if remainder.first() != Some(&b'=') {
                return GlobalScan {
                    prefix_len: index,
                    outcome: GlobalOutcome::Query,
                    changes_environment,
                };
            }
            // `--exec-path=` rewrites the path Git starts commands from, not an environment variable.
            continue;
        }
        if takes_value(argument) {
            if index + 1 == arguments.len() {
                return GlobalScan {
                    prefix_len: index,
                    outcome: GlobalOutcome::MissingValue,
                    changes_environment,
                };
            }
            if ENVIRONMENT_VALUE_OPTIONS.contains(&argument) {
                changes_environment = true;
            } else if argument == b"-C" && !arguments[index + 1].as_encoded_bytes().is_empty() {
                // An empty `-C` moves Git nowhere, and Git's own flag stays clear for it.
                changes_environment = true;
            }
            is_value = true;
            continue;
        }
        if FLAG_OPTIONS.contains(&argument) {
            if ENVIRONMENT_FLAGS.contains(&argument) {
                changes_environment = true;
            }
            continue;
        }
        let mut inline: bool = false;
        for prefix in ENVIRONMENT_INLINE_OPTIONS {
            if argument.starts_with(prefix) {
                changes_environment = true;
                inline = true;
                break;
            }
        }
        if inline {
            continue;
        }
        for prefix in INLINE_OPTIONS {
            if argument.starts_with(prefix) {
                inline = true;
                break;
            }
        }
        if inline {
            continue;
        }
        return GlobalScan {
            prefix_len: index,
            outcome: GlobalOutcome::InvalidOption,
            changes_environment,
        };
    }
    return GlobalScan {
        prefix_len: arguments.len(),
        outcome: GlobalOutcome::NoCommand,
        changes_environment,
    };
}

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
    let scan: GlobalScan = scan_globals(arguments);
    return GlobalLayout {
        prefix_len: scan.prefix_len,
        outcome: scan.outcome,
    };
}

/// What:
///  Whether Git refuses these words as an alias expansion,
///  because one of the globals moves
///       the directory or sets a variable of the environment
///  (`git.c:316`,
/// `git.c:418`).
/// Why:
///   The wrapper expands an alias itself before handing Git the arguments,
///  so it must not build an
///       expansion Git would refuse;
///  the caller's own globals are never refused this way,
///  only the words an alias
///       value contributes,
///  which is why this reads a word list of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prefixChangesEnvironment(words: string[]): boolean;
/// ```
pub fn prefix_changes_environment(arguments: &[OsString]) -> bool {
    return scan_globals(arguments).changes_environment;
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
