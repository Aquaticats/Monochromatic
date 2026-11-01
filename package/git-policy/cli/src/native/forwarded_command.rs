//! What:
//!  The Git command a forwarded invocation actually runs,
//!  after ordinary alias expansion.
//! Why:
//!  Git expands an alias only when no built-in has that name,
//!  reads it from `alias.<name>`
//!      or `alias.<name>.command`,
//!  splits it with `split_cmdline` quoting,
//!  lets its leading words
//!      be global options,
//!  and expands again when the result is an alias.
//!  Index-writer
//!      coordination and the commit rules decide from the expanded command,
//!  so
//!      `git -c alias.c=commit c` is a commit (`src/forwarded-command.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const command = await resolveForwardedCommand({ args, gitPath });
//! ```

/// Debug diagnostics.
use super::diagnostic_log::debug;
/// The built-in command names.
use super::git_builtins::is_git_builtin;
/// The global-option layout.
use super::global_arguments::{GlobalLayout, GlobalOutcome, global_layout};
/// Trimming with the incumbent's `.trim()` semantics.
use super::js_text::trim_javascript;
/// Running real Git.
use super::transaction_git::{GitContext, GitRequest, run_git};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// Alias expansions followed before giving up,
///  well above any realistic chain.
pub const MAX_ALIAS_EXPANSIONS: usize = 16;

/// What:
///  How the command was reached.
/// Why:
///   A shell alias or an unresolved name is never classified as a built-in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardedCommandRoute = 'direct' | 'alias' | 'shell-alias' | 'unresolved';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Route {
    /// A built-in named directly.
    Direct,
    /// A built-in reached through one or more aliases.
    Alias,
    /// An alias running a shell command (`!...`).
    ShellAlias,
    /// An external command,
    ///  an unknown name,
    ///  an alias loop or a malformed alias value.
    Unresolved,
}

/// What:
///  The resolved command:
///  the arguments after expansion,
///  where the subcommand is,
///  its
///       name,
///  and the route.
/// Why:
///   `ResolvedGitCommand`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ResolvedGitCommand = { args: string[]; subcommandIndex: number; subcommand?: string; route: ForwardedCommandRoute };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedCommand {
    /// Arguments after expansion,
    ///  global options included.
    pub arguments: Vec<OsString>,
    /// Index of the subcommand in `arguments`.
    pub subcommand_index: usize,
    /// The subcommand,
    ///  or nothing for a bare `git` or a help,
    ///  version or error form.
    pub subcommand: Option<OsString>,
    /// How it was reached.
    pub route: Route,
}

/// Methods over a resolved command.
impl ResolvedCommand {
    /// What:
    ///  The subcommand as bytes,
    ///  empty when there is none.
    /// Why:
    ///   Classification compares names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// command.subcommand ?? ''
    /// ```
    pub fn word(&self) -> &[u8] {
        match &self.subcommand {
            Some(name) => return name.as_encoded_bytes(),
            None => return b"",
        }
    }

    /// What:
    ///  The arguments after the subcommand.
    /// Why:
    ///   Option classification reads them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// command.args.slice(command.subcommandIndex + 1)
    /// ```
    pub fn region(&self) -> &[OsString] {
        return self.arguments.get(self.subcommand_index + 1..).unwrap_or(&[]);
    }

    /// What:
    ///  Whether this is a built-in reached directly or through aliases.
    /// Why:
    ///   Shell aliases and unresolved names are never index writers or commits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// command.route !== 'shell-alias' && command.route !== 'unresolved' && command.subcommand !== undefined
    /// ```
    pub fn is_builtin(&self) -> bool {
        return matches!(self.route, Route::Direct | Route::Alias) && self.subcommand.is_some();
    }
}

/// What:
///  Whether a byte is whitespace as C `isspace` sees it in the C locale.
/// Why:
///   `split_cmdline` separates words on exactly these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [' ', '\t', '\n', '\r', '\v', '\f'].includes(character)
/// ```
fn is_space(byte: u8) -> bool {
    return matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c);
}

/// What:
///  Split an alias value as Git's `split_cmdline` does:
///  whitespace separates words outside
///       quotes,
///  single and double quotes group,
///  and a backslash escapes the next byte except
///       inside single quotes.
///  `None` is an unclosed quote or a trailing backslash.
/// Why:
///   `splitAliasCommand`;
///  bytes are kept as they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitAliasCommand(value: string): string[] | typeof ALIAS_SPLIT_FAILED;
/// ```
pub fn split_alias_command(value: &[u8]) -> Option<Vec<Vec<u8>>> {
    let mut words: Vec<Vec<u8>> = Vec::new();
    let mut current: Vec<u8> = Vec::new();
    // Starts true as the incumbent's `splitAliasCommand` does, so an empty value is one empty word.
    let mut started: bool = true;
    let mut quote: u8 = 0;
    let mut escaped: bool = false;
    for &byte in value {
        if escaped {
            current.push(byte);
            escaped = false;
        } else if quote == 0 && is_space(byte) {
            if started {
                words.push(std::mem::take(&mut current));
            }
            current.clear();
            started = false;
        } else if quote == 0 && (byte == b'\'' || byte == b'"') {
            quote = byte;
            started = true;
        } else if quote != 0 && byte == quote {
            quote = 0;
        } else if byte == b'\\' && quote != b'\'' {
            escaped = true;
            started = true;
        } else {
            current.push(byte);
            started = true;
        }
    }
    if quote != 0 || escaped {
        return None;
    }
    if started {
        words.push(current);
    }
    return Some(words);
}

/// What:
///  An alias value with surrounding whitespace removed,
///  as JavaScript's `.trim()` does for
///       text;
///  a value that is not UTF-8 loses only ASCII whitespace.
/// Why:
///   The incumbent trims `git config --get` output,
///  newline included,
///  before splitting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// value.trim()
/// ```
pub fn trim_alias_value(value: &[u8]) -> Vec<u8> {
    if let Ok(text) = std::str::from_utf8(value) {
        return trim_javascript(text).as_bytes().to_vec();
    }
    let start: usize = value.iter().position(is_not_ascii_space).unwrap_or(value.len());
    let end: usize = value.iter().rposition(is_not_ascii_space).map_or(start, next_index);
    return value[start..end].to_vec();
}

/// What:
///  Whether a byte is not ASCII whitespace.
/// Why:
///   A named predicate keeps the trimming free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => !isspace(byte)
/// ```
fn is_not_ascii_space(byte: &u8) -> bool {
    return !is_space(*byte);
}

/// What:
///  The index after `index`.
/// Why:
///   A named function keeps `map_or` free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (index: number) => index + 1
/// ```
fn next_index(index: usize) -> usize {
    return index + 1;
}

/// What:
///  The value of one config key through `git config --get`,
///  or nothing when Git reports
///       none or cannot be asked.
/// Why:
///   `readAliasKey`:
///  every failure of the lookup is "no alias".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readAliasKey({ gitPath, globalArgs, key }): Promise<string | typeof ALIAS_UNSET>;
/// ```
fn read_config_value(context: &GitContext, global: &[OsString], key: &OsString) -> Option<Vec<u8>> {
    let mut arguments: Vec<OsString> = global.to_vec();
    arguments.push(OsString::from("config"));
    arguments.push(OsString::from("--get"));
    arguments.push(key.clone());
    let mut request: GitRequest = GitRequest::new(Path::new("."), arguments.as_slice());
    request.without_prefix = true;
    match run_git(context, &request) {
        Ok(output) if output.succeeded() => return Some(output.stdout),
        Ok(_) | Err(_) => {
            debug(
                "readAliasKey",
                format!("no {}", key.to_string_lossy()).as_str(),
            );
            return None;
        }
    }
}

/// What:
///  An alias value in either of Git's spellings,
///  `alias.<name>` first.
/// Why:
///   `lookupAlias`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function lookupAlias({ gitPath, globalArgs, name }): Promise<string | typeof ALIAS_UNSET>;
/// ```
pub fn lookup_alias(context: &GitContext, global: &[OsString], name: &OsString) -> Option<Vec<u8>> {
    let mut plain: OsString = OsString::from("alias.");
    plain.push(name);
    if let Some(value) = read_config_value(context, global, &plain) {
        return Some(value);
    }
    let mut spelled: OsString = plain;
    spelled.push(".command");
    return read_config_value(context, global, &spelled);
}

/// What:
///  The resolved command for the current arguments with a route.
/// Why:
///   Every ending of the expansion builds one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ args, subcommandIndex, subcommand, route })
/// ```
fn resolved(arguments: &[OsString], layout: &GlobalLayout, route: Route) -> ResolvedCommand {
    let subcommand: Option<OsString> = if layout.outcome == GlobalOutcome::Command {
        arguments.get(layout.prefix_len).cloned()
    } else {
        None
    };
    return ResolvedCommand {
        arguments: arguments.to_vec(),
        subcommand_index: layout.prefix_len,
        subcommand,
        route,
    };
}

/// What:
///  Words of an alias value as arguments.
/// Why:
///   The expansion replaces the alias name with them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// words
/// ```
fn os_words(words: Vec<Vec<u8>>) -> Vec<OsString> {
    let mut owned: Vec<OsString> = Vec::with_capacity(words.len());
    for word in words {
        #[cfg(unix)]
        {
            /// The Unix byte-to-string conversion for command words.
            use std::os::unix::ffi::OsStringExt;
            owned.push(OsString::from_vec(word));
        }
        // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
        #[cfg(not(unix))]
        {
            owned.push(OsString::from(String::from_utf8_lossy(word.as_slice()).into_owned()));
        }
    }
    return owned;
}

/// What:
///  Resolve the command a forwarded invocation runs,
///  expanding aliases up to
///       `MAX_ALIAS_EXPANSIONS` times;
///  a built-in name starts no Git process.
/// Why:
///   `resolveForwardedCommand`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveForwardedCommand({ args, gitPath }): Promise<ResolvedGitCommand>;
/// ```
pub fn resolve_forwarded_command(context: &GitContext, arguments: &[OsString]) -> ResolvedCommand {
    // `mut` allows replacing the arguments with each expansion.
    let mut current: Vec<OsString> = arguments.to_vec();
    let mut seen: Vec<OsString> = Vec::new();
    for _ in 0..=MAX_ALIAS_EXPANSIONS {
        let layout: GlobalLayout = global_layout(current.as_slice());
        let route: Route = if seen.is_empty() { Route::Direct } else { Route::Alias };
        if layout.outcome != GlobalOutcome::Command {
            return resolved(current.as_slice(), &layout, route);
        }
        let name: OsString = current[layout.prefix_len].clone();
        if is_git_builtin(name.as_encoded_bytes()) {
            return resolved(current.as_slice(), &layout, route);
        }
        if seen.contains(&name) {
            debug(
                "resolveForwardedCommand",
                format!("alias loop at {}", name.to_string_lossy()).as_str(),
            );
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        }
        seen.push(name.clone());
        let global: &[OsString] = &current[..layout.prefix_len];
        let Some(value) = lookup_alias(context, global, &name) else {
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        };
        let trimmed: Vec<u8> = trim_alias_value(value.as_slice());
        if trimmed.first() == Some(&b'!') {
            return resolved(current.as_slice(), &layout, Route::ShellAlias);
        }
        let Some(words) = split_alias_command(trimmed.as_slice()) else {
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        };
        if words.is_empty() {
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        }
        let mut expanded: Vec<OsString> = global.to_vec();
        expanded.extend(os_words(words));
        expanded.extend(current[layout.prefix_len + 1..].iter().cloned());
        debug(
            "resolveForwardedCommand",
            format!("expanded alias {}", name.to_string_lossy()).as_str(),
        );
        current = expanded;
    }
    let layout: GlobalLayout = global_layout(current.as_slice());
    return resolved(current.as_slice(), &layout, Route::Unresolved);
}

/// What:
///  Whether a resolved command creates or moves linked worktrees.
/// Why:
///   `createsOrMovesWorktrees`:
///  `git worktree add` and `git worktree move`,
///  decided by the
///       first positional token after `worktree`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createsOrMovesWorktrees(command: ResolvedGitCommand): boolean;
/// ```
pub fn creates_or_moves_worktrees(command: &ResolvedCommand) -> bool {
    if !command.is_builtin() || command.word() != b"worktree" {
        return false;
    }
    for token in command.region() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !bytes.starts_with(b"-") {
            return bytes == b"add" || bytes == b"move";
        }
    }
    return false;
}

/// Resolution controls stay out of the release executable.
#[cfg(test)]
#[path = "forwarded_command_tests.rs"]
mod tests;
