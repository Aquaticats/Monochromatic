//! What:
//!  The Git command a forwarded invocation actually runs,
//!  after ordinary alias expansion.
//! Why:
//!  Git expands an alias only when no built-in has that name,
//!  keeps the last of the two spellings `alias.<name>` and `[alias "<name>"] command` in config order,
//!  splits its value with `split_cmdline` quoting,
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
/// The command a Git alias really runs,
///  the executable Git selects first included.
use crate::external_git_command::{
    alias_run_directory,
    dashed_external_selected,
    effective_directory,
};
/// The built-in command names.
use crate::git_builtins::is_git_builtin;
/// The global-option layout,
///  and the alias-only refusal about the environment.
use crate::global_arguments::{
    GlobalLayout,
    GlobalOutcome,
    global_layout,
    prefix_changes_environment,
};
/// Running real Git,
///  for the nested reads of a shell alias and the refusals they answer.
use crate::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Alias expansions followed before this reading gives up,
///  a bound on work and never a claim about Git.
/// Why:
///   Git ends a chain by reaching a name it has already expanded,
///  which this reading also refuses at once,
///  so the count stands only against a configuration a second writer grows while
///  the reading walks it:
///  one expansion asks Git for the configuration again,
///  and a chain deeper than this costs more than a person waits for.
///  It is far past the chains people write:
///  real Git runs a chain of 30 links.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_ALIAS_EXPANSIONS = 64;
/// ```
pub const MAX_ALIAS_EXPANSIONS: usize = 64;

/// What:
///  Shells of `!` aliases followed,
///  one for each shell Git starts in turn.
/// Why:
///   A `!` alias whose value names another `!` alias starts a second shell,
///  and the command Git finally runs is the one the last shell runs;
///  a value that names itself would make the reading start shells without end,
///  so it stops at a depth no chain a person writes reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_SHELL_ALIAS_DEPTH = 4;
/// ```
pub(crate) const MAX_SHELL_ALIAS_DEPTH: usize = 4;

/// What:
///  How the command was reached.
/// Why:
///   A shell alias or an unresolved name is never classified as a built-in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardedCommandRoute = 'direct' | 'alias' | 'shell-alias' | 'unresolved' | 'unread';
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
    /// What:
    ///  A chain of aliases longer than this reading follows,
    ///  so the words are not the command Git runs.
    /// Why:
    ///   A chain the reading gave up on is left to Git as an alias the wrapper
    ///  cannot follow is,
    ///  and no word of the half-read vector names a subcommand a planner could act
    ///  on.
    Unread,
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
        return self
            .arguments
            .get(self.subcommand_index + 1..)
            .unwrap_or(&[]);
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

// The alias value readers, the configuration variables and the alias
//  they select, and the words a `!` shell alias runs, read beside the value they serve.
use crate::alias_value::shell_git_words;
pub use crate::alias_value::{
    ConfigVariable,
    alias_selection,
    alias_variable_matches,
    config_variables,
    lookup_alias,
    split_alias_command,
};

/// What:
///  The resolved command for the current arguments with a route.
/// Why:
///   Every ending of the expansion builds one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ({ args, subcommandIndex, subcommand, route })
/// ```
fn resolved(
    arguments: &[OsString],
    layout: &GlobalLayout,
    route: Route,
) -> ResolvedCommand {
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
    // A chain the reading gave up on names no subcommand: the word standing where one would be is an
    //  alias's name or a command nobody read, so no planner may act on it.
    let subcommand: Option<OsString> =
        if route != Route::Unread && layout.outcome == GlobalOutcome::Command {
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
            owned.push(OsString::from(
                String::from_utf8_lossy(word.as_slice()).into_owned(),
            ));
        }
    }
    return owned;
}

/// What:
///  Resolve the command a forwarded invocation runs,
///  expanding aliases up to
///       `MAX_ALIAS_EXPANSIONS` times;
///  a built-in name starts no Git process.
///  `Option<&Path>` is the directory this command was started in,
///       which Git starts in too and moves only for the caller's `-C`,
///       and nothing when the caller can name none.
/// Why:
///   `resolveForwardedCommand`.
///  A relative `PATH` entry and a relative exec path name a directory of the caller's,
///  and the alias value is read there,
///  so that directory is an input of the reading and never a fresh read of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function resolveForwardedCommand({ args, environment, gitPath, start }): Promise<ResolvedGitCommand>;
/// ```
pub fn resolve_forwarded_command(
    context: &GitContext,
    environment: &[(OsString, OsString)],
    arguments: &[OsString],
    start: Option<&Path>,
) -> ResolvedCommand {
    // The directory this command was started in; Git starts in it too and moves only for the caller's `-C`, so a
    //  relative `PATH` entry or exec path is resolved from here.
    return resolve_command(context, environment, arguments, 0, &[], start);
}

/// What:
///  Resolve the command,
///  and read the `git` command a shell alias runs,
///  through the shells a `!` alias inside its value starts.
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
    caller_globals.extend(
        arguments[..global_layout(arguments).prefix_len]
            .iter()
            .cloned(),
    );
    for _ in 0..=MAX_ALIAS_EXPANSIONS {
        let layout: GlobalLayout = global_layout(current.as_slice());
        let route: Route = if seen.is_empty() {
            Route::Direct
        } else {
            Route::Alias
        };
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
            && let Some(directory) =
                alias_run_directory(context, inherited_globals, current_directory)
        {
            run_directory = Some(effective_directory(
                &directory,
                &current[..layout.prefix_len],
            ));
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
        let Some(value) = lookup_alias(
            context,
            process_globals.as_slice(),
            &name,
            current_directory,
        ) else {
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        };
        // Git decides what an alias is from its first byte, and splits the value it read, without trimming it.
        if value.first() == Some(&b'!') {
            // What the shell of this `!` alias runs: the value's own words and then the caller's, resolved the way
            //  any nested command is, through as many shells as the value's own words name a `!` alias.
            // `mut` allows the shell's command to be filled in once the nested reading answered.
            let mut shell_command: Option<Vec<OsString>> = None;
            // A value that names itself would start shells without end, and each shell Git starts is a process, so
            //  this walk stops at a depth no chain a person writes reaches.
            if shell_depth < MAX_SHELL_ALIAS_DEPTH
                // What the shell's own words are, when every byte of the value passes through the shell unchanged.
                && let Some(own) = shell_git_words(value.as_slice()).map(os_words)
                // The value's own words must start the command themselves, because a global option there would move
                //  the command away from the repository the caller described. The caller's own words may name one,
                //  because Git hands them to that same nested Git after the value.
                && (own.is_empty() || global_layout(own.as_slice()).prefix_len == 0)
            {
                // Git appends the caller's own words to the value before the shell reads it (`git.c:401` to
                //  `git.c:402`), so the nested Git parses both: a value of `!git` alone has no word of its own and
                //  the caller's words start its command.
                let mut words: Vec<OsString> = own;
                words.extend(current[layout.prefix_len + 1..].iter().cloned());
                let nested: ResolvedCommand = resolve_command(
                    context,
                    environment,
                    words.as_slice(),
                    shell_depth + 1,
                    process_globals.as_slice(),
                    current_directory,
                );
                // Only a built-in this crate read is handed on, through a second `!` alias's shell when the value
                //  names one; anything else stays unread.
                if let Some(read) = the_command_git_runs(&nested)
                    && read.is_builtin()
                {
                    shell_command = Some(read.arguments);
                }
            }
            return resolved_with_shell_command(
                current.as_slice(),
                &layout,
                Route::ShellAlias,
                shell_command,
            );
        }
        let Some(words) = split_alias_command(value.as_slice()) else {
            return resolved(current.as_slice(), &layout, Route::Unresolved);
        };
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
    // The count ran out before the chain ended, so these words are not the command Git runs.
    let layout: GlobalLayout = global_layout(current.as_slice());
    return resolved_with_shell_command(current.as_slice(), &layout, Route::Unread, None);
}

/// What:
///  The directory a command's relative operands are read from:
///  the top of the working tree for the command a `!` alias's shell runs,
///  and the directory this command was started in for every other spelling,
///  whose own reader then applies the caller's `-C` chain.
///  `Option<&Path>` is the directory this command was started in.
/// Why:
///   A reader of a relative path of that command needs the directory Git runs it
///  in,
///  and the slot refusals and the guard must read the same one,
///  so the answer stands here once rather than in each of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const operandsDirectory = (command, start) => command.route === 'shell-alias' ? shellRunDirectory(command, start) : start;
/// ```
pub(crate) fn the_directory_a_commands_operands_are_read_from(
    context: &GitContext,
    command: &ResolvedCommand,
    start: Option<&Path>,
) -> Option<PathBuf> {
    if command.route != Route::ShellAlias {
        return start.map(Path::to_path_buf);
    }
    return shell_run_directory(context, command, start);
}

/// What:
///  The command Git will run for one invocation,
///  as far as the wrapper can read it:
///  the one plain `git` command a `!` alias runs,
///  when that alias value could be read,
///  and the resolved command itself otherwise.
///  `None` is a command the reading could not reach:
///  a `!` alias whose value the wrapper could not read,
///  and a chain of aliases longer than it follows.
/// Why:
///   A `!` alias starts a shell,
///  and the `git` inside it is real Git,
///  so what Git runs is the command the value and the caller's words make,
///  never the alias's own name.
///  The destructive-command guard and the slot refusals decide about that
///  command,
///  so both read it here and cannot disagree about what an alias is.
///  A command the wrapper cannot read is left to Git,
///  which is what the guard does with it
///  (`SPEC.md`,
///  "git by the word Git will run").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theCommandGitRuns(command: ResolvedGitCommand): ResolvedGitCommand | undefined;
/// ```
pub fn the_command_git_runs(command: &ResolvedCommand) -> Option<ResolvedCommand> {
    if command.route == Route::Unread {
        return None;
    }
    if command.route != Route::ShellAlias {
        return Some(command.clone());
    }
    let shell_command: &[OsString] = command.shell_command.as_deref()?;
    let layout: GlobalLayout = global_layout(shell_command);
    return Some(resolved(shell_command, &layout, Route::Direct));
}

/// What:
///  The directory Git runs the command in,
///  which for a `!` alias is the top of the working tree its shell changes to,
///  and nothing when the command is not a shell alias.
///  `Option<&Path>` is the directory this command was started in.
/// Why:
///   Git changes to the top of the working tree before it starts such a shell
///  (`git.c:388` to `git.c:407`),
///  so a relative path the command inside it names is read from there,
///  never from the caller's `-C` chain;
///  the guard and the slot refusals both read paths of that command,
///  and one answer keeps them from reading different directories.
///  `alias_run_directory` is the same question the search for an installed
///  `git-<word>` asks,
///  so a path read here cannot land where Git never goes.
///  A command that is no shell alias runs where the caller's options reach,
///  which its own reader derives,
///  so this answers nothing for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shellRunDirectory = (command, start) => command.route === 'shell-alias' ? aliasRunDirectory(command.args.slice(0, command.subcommandIndex), start) : undefined;
/// ```
pub(crate) fn shell_run_directory(
    context: &GitContext,
    command: &ResolvedCommand,
    start: Option<&Path>,
) -> Option<PathBuf> {
    let globals: &[OsString] = command
        .arguments
        .get(..command.subcommand_index)
        .unwrap_or(&[]);
    return alias_run_directory(context, globals, start);
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

/// Resolution controls stay out of the release executable.
#[cfg(test)]
#[path = "forwarded_command_tests.rs"]
mod tests;
