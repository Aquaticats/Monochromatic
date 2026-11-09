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
//!      `git -c alias.c=commit c` is a commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const command = await resolveForwardedCommand({ args, gitPath });
//! ```

/// Debug diagnostics.
use crate::diagnostic_log::debug;
/// The built-in command names.
use crate::git_builtins::is_git_builtin;
/// The global-option layout,
///  and the alias-only refusal about the environment.
use crate::global_arguments::{GlobalLayout, GlobalOutcome, global_layout, prefix_changes_environment};
/// Running real Git.
use crate::transaction_git::{GitContext, GitRequest, run_git};
/// The command a Git alias really runs,
///  the executable Git selects first included.
use crate::external_git_command::{alias_run_directory, dashed_external_selected, effective_directory};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

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
    /// What:
    ///  The command Git runs inside the shell of a `!` alias,
    ///  when the alias holds one plain `git` command.
    /// Why:
    ///   Git runs the shell with its own executable directory first on `PATH`
    ///  (`exec-cmd.c:333` to `:350`),
    ///  so the `git` such an alias names is the real Git and no guard sees it;
    ///  a plain reading of the alias value at
    ///  least lets the guard read the command that the shell will run.
    ///  The wrapper never forwards this reading:
    ///  the shell keeps the
    ///  environment Git gives it,
    ///  which a substituted argument vector would lose.
    pub shell_command: Option<Vec<OsString>>,
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

// The alias value splitter and its record terminator, read beside the value they serve.
pub use crate::alias_value::{alias_value_of, split_alias_command};

/// What:
///  The value of one config key through `git config --null --get`,
///  or nothing when Git reports
///       none or cannot be asked.
/// Why:
///   `readAliasKey`:
///  every failure of the lookup is "no alias".
///  `--null` ends the value with a NUL instead of a newline,
/// which is the one terminator this
///       removes,
///  so a value that ends with a newline or a space keeps it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readAliasKey({ gitPath, globalArgs, key }): Promise<string | typeof ALIAS_UNSET>;
/// ```
fn read_config_value(context: &GitContext, global: &[OsString], key: &OsString) -> Option<Vec<u8>> {
    let mut arguments: Vec<OsString> = global.to_vec();
    arguments.push(OsString::from("config"));
    arguments.push(OsString::from("--null"));
    arguments.push(OsString::from("--get"));
    arguments.push(key.clone());
    let mut request: GitRequest = GitRequest::new(Path::new("."), arguments.as_slice());
    request.without_prefix = true;
    match run_git(context, &request) {
        Ok(output) if output.succeeded() => return Some(alias_value_of(output.stdout.as_slice())),
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
    return resolved_with_shell_command(arguments, layout, route, None);
}

/// What:
///  The resolved command with the reading of a shell alias,
///  when this ending has one.
/// Why:
///   Every ending but a shell alias leaves the shell reading absent,
///  and one builder keeps that field from
///       being forgotten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolvedWithShellCommand(args, layout, route, shellCommand): ResolvedGitCommand;
/// ```
fn resolved_with_shell_command(
    arguments: &[OsString],
    layout: &GlobalLayout,
    route: Route,
    shell_command: Option<Vec<OsString>>,
) -> ResolvedCommand {
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
        shell_command,
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
/// async function resolveForwardedCommand({ args, environment, gitPath }): Promise<ResolvedGitCommand>;
/// ```
pub fn resolve_forwarded_command(
    context: &GitContext,
    environment: &[(OsString, OsString)],
    arguments: &[OsString],
) -> ResolvedCommand {
    // The directory this process was started in; Git starts in it too and moves only for the caller's `-C`, so a
    //  relative `PATH` entry or exec path is resolved from here.
    let current_directory: Option<PathBuf> = std::env::current_dir().ok();
    return resolve_command(context, environment, arguments, 0, &[], current_directory.as_deref());
}

/// What:
///  Resolve the command,
///  and read the one `git` command a shell alias runs,
///  one shell deep.
///  `usize` counts the shells already read;
///  `&[OsString]` borrows the globals of the process that started this one,
///       which the Git a shell alias starts reads too;
///       no globals when Git starts the command itself;
///  `Option<&Path>` is the directory this process was started in,
///       and nothing when it cannot be read.
/// Why:
///   A `!` alias runs a shell that runs the real Git,
///  so no guard sees that command unless the value
///  is read here;
///  the count stops a shell alias that names itself from reading without end.
///  The Git inside the shell inherits the configuration of every `-c` option
///  (`config.c:469`)
///  and the exec path of the process that started it
///  (`exec-cmd.c:303`),
///  so a nested
///  reading without those globals finds no alias the caller configured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveCommand({ context, environment, args, shellDepth, inheritedGlobals, currentDirectory }): ResolvedGitCommand;
/// ```
fn resolve_command(
    context: &GitContext,
    environment: &[(OsString, OsString)],
    arguments: &[OsString],
    shell_depth: usize,
    inherited_globals: &[OsString],
    current_directory: Option<&Path>,
) -> ResolvedCommand {
    // `mut` allows replacing the arguments with each expansion.
    let mut current: Vec<OsString> = arguments.to_vec();
    let mut seen: Vec<OsString> = Vec::new();
    // The globals of the caller are the ones Git read before it reached the command word, and they decide the exec
    //  path the search for an executable uses; an alias expansion adds none of them back.
    let mut caller_globals: Vec<OsString> = inherited_globals.to_vec();
    caller_globals.extend(arguments[..global_layout(arguments).prefix_len].iter().cloned());
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
        // Where this frame's word runs. Every frame but the one the shell of a `!` alias starts runs in the directory
        //  the caller's `-C` options reach. That frame runs at the top of the working tree Git changed to before it
        //  started the shell (`git.c:388` to `git.c:407`), and the globals of this frame move it from there; applying
        //  the inherited ones again would move it twice.
        // `mut` allows the directory the word runs in to be filled in for a shell alias's frame.
        let mut run_directory: Option<PathBuf> = None;
        if shell_depth != 0
            && let Some(directory) = alias_run_directory(context, inherited_globals, current_directory)
        {
            run_directory = Some(effective_directory(&directory, &current[..layout.prefix_len]));
        }
        // Git runs an installed `git-<word>` before it looks for an alias of that word (git.c:905 to git.c:912), so
        //  such a word reaches no alias and the wrapper leaves it to Git.
        if dashed_external_selected(
            context,
            environment,
            caller_globals.as_slice(),
            current_directory,
            run_directory.as_deref(),
            name.as_os_str(),
        ) {
            debug(
                "resolveForwardedCommand",
                format!("Git runs {} as an external command", name.to_string_lossy()).as_str(),
            );
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        }
        if seen.contains(&name) {
            debug(
                "resolveForwardedCommand",
                format!("alias loop at {}", name.to_string_lossy()).as_str(),
            );
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        }
        seen.push(name.clone());
        // What this Git process reads as its configuration and its exec path: the globals it inherited and its own
        //  before the command word, which an expansion of `-c` adds to.
        // Why: the alias lookup and the Git the shell starts read both, and a lookup that read only the own ones
        //  would miss what the process that started this one passed along.
        let mut process_globals: Vec<OsString> = inherited_globals.to_vec();
        process_globals.extend(current[..layout.prefix_len].iter().cloned());
        let Some(value) = lookup_alias(context, process_globals.as_slice(), &name) else {
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        };
        // Git decides what an alias is from its first byte, and splits the value it read, without trimming it.
        if value.first() == Some(&b'!') {
            // Git appends the caller's own words to the alias value before it starts the shell (`git.c:401` to
            //  `git.c:402`), so the command the shell runs carries them after the alias's own words; a value of `!git`
            //  alone would otherwise read as no command at all.
            let shell_command: Option<Vec<OsString>> = shell_command_of(
                context,
                environment,
                value.as_slice(),
                shell_depth,
                &current[layout.prefix_len + 1..],
                process_globals.as_slice(),
                current_directory,
            );
            return resolved_with_shell_command(current.as_slice(), &layout, Route::ShellAlias, shell_command);
        }
        let Some(words) = split_alias_command(value.as_slice()) else {
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        };
        if words.is_empty() {
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        }
        let expanded_words: Vec<OsString> = os_words(words);
        // Git refuses an alias whose expansion moves the directory or sets an environment variable, and runs nothing
        //  (git.c:418): `git -c 'alias.audit=-C elsewhere reset --hard' audit` is such a refusal, not a reset.
        if prefix_changes_environment(expanded_words.as_slice()) {
            debug(
                "resolveForwardedCommand",
                format!(
                    "alias {} changes the environment, so Git refuses it",
                    name.to_string_lossy()
                )
                .as_str(),
            );
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        }
        let mut expanded: Vec<OsString> = current[..layout.prefix_len].to_vec();
        expanded.extend(expanded_words);
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
///  Whether a resolved command creates a linked worktree.
/// Why:
///   `createsWorktrees`:
///  `git worktree add`,
///  decided by the first positional token after `worktree`.
///  A `git worktree move` registers nothing new,
/// so the ignored-state copy has nothing to do for it and it is forwarded
///  like any other command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function createsWorktrees(command: ResolvedGitCommand): boolean;
/// ```
pub fn creates_worktrees(command: &ResolvedCommand) -> bool {
    if !command.is_builtin() || command.word() != b"worktree" {
        return false;
    }
    for token in command.region() {
        let bytes: &[u8] = token.as_encoded_bytes();
        if !bytes.starts_with(b"-") {
            return bytes == b"add";
        }
    }
    return false;
}

/// What:
///  The argument vector of the one plain `git` command a `!` alias runs,
///  when the value names one.
///  `&[(OsString, OsString)]` borrows the environment the nested reading asks about;
///  `&[OsString]` borrows the caller's own words,
///       which Git hands the shell after the alias's;
///  `&[OsString]` borrows the globals of the process that started this one,
///       which the Git
///       inside the shell reads too;
///  `Option<&Path>` is the directory this process was started in,
///       and nothing when it cannot be read.
/// Why:
///   Git prepends its own executable directory to `PATH` before it runs the shell
///  (`exec-cmd.c:333` to
///  `:350`),
///  so the `git` a shell alias names is the real Git and the wrapper never sees that command
///  run;
///  reading the value reaches the guard the decision this crate already knows.
///  Git appends the caller's words to the value before the shell reads it,
///  so the command that runs
///  is the words of the value and then the caller's,
///  and a reading that stopped at the value would guard a command nobody runs.
///  The nested Git also reads the configuration and exec path the caller passed,
///  which Git
///  exports to what it starts.
///  The reading is kept to one command with no global
///  option of its own:
///  a global such as `-C` would move the command away from the repository the caller described.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shellCommandOf({ context, environment, value, shellDepth, callerArguments, inheritedGlobals, currentDirectory }): string[] | undefined;
/// ```
fn shell_command_of(
    context: &GitContext,
    environment: &[(OsString, OsString)],
    value: &[u8],
    shell_depth: usize,
    caller_arguments: &[OsString],
    inherited_globals: &[OsString],
    current_directory: Option<&Path>,
) -> Option<Vec<OsString>> {
    // One shell is read: a shell alias named inside another is left unread, so a chain of them cannot repeat.
    if shell_depth >= 1 {
        return None;
    }
    // The shell runs the alias's own words first and the caller's after them (`git.c:401` to `git.c:402`), so the
    //  nested Git parses both: a value of `!git` alone has no word of its own and the caller's words start its command.
    let mut words: Vec<OsString> = os_words(shell_git_words(value)?);
    words.extend(caller_arguments.iter().cloned());
    if words.is_empty() {
        return None;
    }
    let nested: ResolvedCommand = resolve_command(
        context,
        environment,
        words.as_slice(),
        shell_depth + 1,
        inherited_globals,
        current_directory,
    );
    // Only a built-in this crate already read is handed on, and only when no global option stands before it.
    if !nested.is_builtin() || nested.subcommand_index != 0 {
        return None;
    }
    return Some(nested.arguments);
}

/// What:
///  The words after `!git` in an alias value,
///  when the value is exactly one plain `git` command;
///  no word at all when the value names `git` and nothing else,
///  whose command the caller's own words then make.
/// Why:
///   The shell splits and expands its command line,
///  so only a value whose bytes the shell passes through
///       unchanged reads here as the words the shell runs;
///  anything the shell would expand,
///  quote or separate
///  is left unread rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shellGitWords(value: string): string[] | undefined;
/// ```
fn shell_git_words(value: &[u8]) -> Option<Vec<Vec<u8>>> {
    let remainder: &[u8] = value.strip_prefix(b"!git")?;
    // `!gitk` names another program, so only a space or the end of the value may follow.
    if !remainder.is_empty() && remainder.first() != Some(&b' ') {
        return None;
    }
    let mut words: Vec<Vec<u8>> = Vec::new();
    let mut current: Vec<u8> = Vec::new();
    for &byte in remainder {
        if byte == b' ' {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }
        if !is_plain_shell_byte(byte) {
            return None;
        }
        current.push(byte);
    }
    if !current.is_empty() {
        words.push(current);
    }
    return Some(words);
}

/// What:
///  Whether the shell passes a byte through in the same word,
///  without quoting it,
///  expanding it or separating
///       on it.
/// Why:
///   These are the bytes of ordinary arguments;
///  `$`,
///  a backtick,
///  a quote,
///  a backslash,
///  a glob,
///  a
///  redirection,
///  a separator,
///  a brace and a leading `~` all mean something to the shell,
/// and `#` can start a
///  comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => /[A-Za-z0-9_./:=@+,-]/.test(String.fromCharCode(byte))
/// ```
fn is_plain_shell_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric()
        || matches!(byte, b'-' | b'_' | b'.' | b'/' | b':' | b'=' | b'@' | b'+' | b',' );
}

/// Resolution controls stay out of the release executable.
#[cfg(test)]
#[path = "forwarded_command_tests.rs"]
mod tests;
