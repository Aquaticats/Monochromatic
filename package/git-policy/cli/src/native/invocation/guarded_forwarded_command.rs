//! What:
//!  The guarded decision about the command a forwarded invocation really runs,
//!  ordinary Git alias expansion
//!       included.
//! Why:
//!  The guard reads the word on the command line,
//!  and Git runs the word an alias expands to:
//!  `git -c alias.wipe=reset --hard wipe` discarded a worktree while `wipe` matched no arm of the
//!       guard,
//!  so the reading has to happen on the expanded command,
//!  which is also the command the wrapper
//!       forwards and the one the audit line should name.
//!  The crate already resolves aliases for the worktree copy
//!       route,
//!  so this is the same resolution applied to the guard.
//!  A built-in word is decided without asking
//!       anything,
//!  because no alias can shadow a built-in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // decideGuardedForwarded({ args: ['-c','alias.wipe=reset --hard','wipe'] })
//! //   => { decision: { kind: 'needs-effective-target', command: 'reset-hard' }, args: ['reset','--hard'] }
//! ```

/// What:
///  Bring the resolver,
///  its route and the guard's own reading into this file.
/// Why:
///   This file adds no parsing:
///  it decides which command the guard reads,
/// and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { resolveForwardedCommand, Route } from './forwarded_command.ts';
/// import { decideGuardedWord } from '../command/rule_linked_worktree.ts';
/// ```
/// The Git directory Git gives the Git inside a `!` alias's shell.
use crate::external_git_command_the_git_dir_the_shell_reads::the_git_dir_the_alias_shell_reads;
use crate::forwarded_command::{
    ResolvedCommand,
    Route,
    resolve_forwarded_command,
    shell_run_directory,
    the_command_git_runs,
};
use crate::git_builtins::is_git_builtin;
use crate::global_arguments::{
    GlobalLayout,
    command_tokens,
    global_layout,
    names_the_place_the_command_acts,
    option_tokens,
    without_directory_changes,
};
use crate::rule_linked_worktree::{
    LinkedWorktreeDecision,
    decide_guarded_word,
};
/// The transaction Git context the resolver asks for an alias.
use crate::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::{
    OsStr,
    OsString,
};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw bytes.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The guard's decision,
///  the arguments it decided about and,
///  for a `!` alias,
///  the directory its shell runs in.
///  A `struct` is a record with named fields.
/// Why:
///   A caller that records what it decided has to name the command Git will run,
/// which for an alias is the
///  expansion and not the alias name,
///  and a caller that resolves a relative path of that command needs the directory Git
///  starts its shell in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GuardedForwarded = { decision: LinkedWorktreeDecision; args: string[]; shellDirectory?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardedForwarded {
    /// What the guard decides about the command Git will run.
    pub decision: LinkedWorktreeDecision,
    /// The arguments Git receives:
    ///  the caller's own,
    ///  or the alias expansion the wrapper forwards.
    pub arguments: Vec<OsString>,
    /// What:
    ///  The directory Git runs the shell of a `!` alias in,
    ///  when the decision is about the command that shell runs;
    ///  nothing for a direct command,
    ///  an ordinary alias and every pass.
    /// Why:
    ///   Git changes to the top of the working tree before it starts such a shell
    ///  (`git.c:388` to `git.c:407`),
    ///  so a relative path the command inside it names is read from there,
    ///  never from the caller's `-C` chain.
    pub shell_directory: Option<PathBuf>,
    /// What:
    ///  The global options every repository fact about this command is asked with,
    ///  when the command Git runs names a directory or a repository of its own;
    ///  nothing when the caller's own options reach the command.
    /// Why:
    ///   A bare `!git` whose caller's words carry `-C <another worktree>` runs that same Git in that worktree,
    ///  and Git hands the words after the alias name to the nested Git
    ///  (`git.c:401` to `git.c:402`),
    ///  so the worktree the command changes is the one those words name.
    ///  The guard and the slot refusals read this one answer,
    ///  so neither judges the worktree of the shell that started them.
    ///  Nothing here when Git could not be asked where that shell runs:
    ///  a place nothing read is no place a policy may act on,
    ///  and the slot refusals refuse such a command rather than place it.
    pub globals_that_place_the_command: Option<Vec<OsString>>,
}

/// What:
///  What the guard decides about one invocation,
/// after alias expansion.
///  `Option<&Path>` is the directory this command was started in,
///       and nothing when the caller can name none.
/// Why:
///   A built-in word is never an alias,
///  so it needs no configuration read;
///  a shell alias and a name nothing
///       defines have no subcommand this guard can read,
///  so they are left to Git and to whatever the shell
///       finally runs.
///  The alias value and the shell's own directory both follow from where this command
///  was started,
///  so that directory is an input rather than a read of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideGuardedForwarded({ context, args, start }): GuardedForwarded;
/// ```
pub fn decide_guarded_forwarded(
    context: &GitContext,
    environment: &[(OsString, OsString)],
    arguments: &[OsString],
    start: Option<&Path>,
) -> GuardedForwarded {
    // `let Some((word, region)) = ... else { ... };` unwraps the subcommand, or passes when Git runs none.
    let Some((word, region)) = command_tokens(arguments) else {
        return GuardedForwarded {
            decision: LinkedWorktreeDecision::Pass,
            arguments: arguments.to_vec(),
            shell_directory: None,
            globals_that_place_the_command: None,
        };
    };
    // `.as_encoded_bytes()` lends the raw bytes of the word.
    if is_git_builtin(word.as_encoded_bytes()) {
        return GuardedForwarded {
            decision: decide_guarded_word(word.as_encoded_bytes(), region),
            arguments: arguments.to_vec(),
            shell_directory: None,
            globals_that_place_the_command: None,
        };
    }
    // The word names no built-in, so Git looks it up as an alias, and an undefined name is left to Git.
    let resolved: ResolvedCommand =
        resolve_forwarded_command(context, environment, arguments, start);
    // What the guard decides about, and the same value the slot refusals read: for a shell alias that is the one
    //  plain `git` command the resolver read from its value, and for every other route it is the resolved command.
    //  A shell alias the wrapper cannot read has no reading here, so Git runs it as Git runs any shell alias.
    let Some(command_git_runs) = the_command_git_runs(&resolved) else {
        return GuardedForwarded {
            decision: LinkedWorktreeDecision::Pass,
            arguments: arguments.to_vec(),
            shell_directory: None,
            globals_that_place_the_command: None,
        };
    };
    // A value that starts with `!` runs a shell, and Git puts its own executable directory first on that shell's
    //  `PATH`, so the `git` such an alias names is the real Git and never this wrapper. The caller's own arguments stay
    //  untouched: Git runs the shell with an environment a substituted argument vector would lose.
    if resolved.route == Route::ShellAlias {
        // What: The directory Git runs this shell in, and the globals every fact about the command it runs is
        //  asked with.
        // Why: The command's own `-C`, `--git-dir` or `--work-tree` names another worktree or repository, and the
        //  policies judged the caller's until they were asked from there; the caller's own options before the alias
        //  name and the `GIT_DIR` Git exports to the shell move the repository the nested Git uses, and a guard that
        //  read neither measured one place while the command wrote two (issue 6641).
        let shell_directory: Option<PathBuf> = shell_run_directory(context, &resolved, start);
        // What the command Git runs adds before its subcommand: the alias value's own words and then the caller's.
        let own: &[OsString] = command_git_runs
            .arguments
            .get(..command_git_runs.subcommand_index)
            .unwrap_or(&[]);
        // What the caller typed before the alias name, which Git reads first and hands to the shell as well.
        let caller_layout: GlobalLayout = global_layout(arguments);
        let callers_globals: &[OsString] = arguments.get(..caller_layout.prefix_len).unwrap_or(&[]);
        // Git names the repository of the shell's Git only when something named one, so that is the one question the
        //  reading asks about it; a command whose own words name no place keeps the caller's options, which reach the
        //  same worktree.
        let place_named_by_the_callers_words: bool = the_place_was_named(callers_globals);
        let inherited_git_dir: Option<OsString> = if shell_directory.is_some()
            && own
                .iter()
                .any(|token| return names_the_place_the_command_acts(token.as_encoded_bytes()))
        {
            the_git_dir_the_alias_shell_reads(
                context,
                callers_globals,
                start,
                place_named_by_the_callers_words,
            )
        } else {
            None
        };
        return GuardedForwarded {
            decision: decide_guarded_word(command_git_runs.word(), command_git_runs.region()),
            arguments: arguments.to_vec(),
            // The command the shell runs is read from the directory Git put the shell in, not from the caller's.
            shell_directory: shell_directory.clone(),
            globals_that_place_the_command: globals_that_place_the_command(
                own,
                shell_directory.as_deref(),
                callers_globals,
                inherited_git_dir.as_deref(),
            ),
        };
    }
    if resolved.route != Route::Alias || resolved.subcommand.is_none() {
        return GuardedForwarded {
            decision: LinkedWorktreeDecision::Pass,
            arguments: arguments.to_vec(),
            shell_directory: None,
            globals_that_place_the_command: None,
        };
    }
    return GuardedForwarded {
        decision: decide_guarded_word(command_git_runs.word(), command_git_runs.region()),
        arguments: command_git_runs.arguments,
        shell_directory: None,
        globals_that_place_the_command: None,
    };
}

/// What:
///  The global options every repository fact about one command Git runs is asked with,
///  from the directory Git runs its shell in,
///  or nothing when that command names no place of its own or that directory was not read.
///  `&[OsString]` borrows the command's own globals before its subcommand;
///  the other slice borrows the caller's globals before the alias name;
///  `Option<&OsStr>` is what Git gives the shell's Git as `GIT_DIR`,
///  and nothing when it gives none.
/// Why:
///  The nested Git starts where the shell runs,
///  keeps the `GIT_DIR` Git exported to that shell,
///  keeps the caller's `-c` options Git exported there too,
///  and only then applies the words Git appended to the alias value
///  (`git.c:401` to `git.c:402`),
///  so the same list in that order is the one that asks real Git about the place the command acts in.
///  `-C` first makes the facts start where the shell started,
///  and the command's own options then move them exactly as Git moves the nested Git.
///  The caller's own `-C` options are left out,
///  because the shell already stands where they reached,
///  and its `--git-dir` or `--work-tree` are kept as written,
///  so a later one of the command's own overrides them as it does for Git.
///  A command whose own options name no place keeps the caller's own options,
///  which reach the same worktree the shell runs in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const globalsThatPlaceTheCommand = ({ own, shellDirectory, callersGlobals, inheritedGitDir }) => {
///   if (!own.some(namesThePlace) || shellDirectory === undefined) return undefined;
///   return ['-C', shellDirectory, ...(inheritedGitDir === undefined ? [] : ['--git-dir', inheritedGitDir]), ...withoutDirectoryChanges(callersGlobals), ...own];
/// };
/// ```
pub(crate) fn globals_that_place_the_command(
    own: &[OsString],
    shell_directory: Option<&Path>,
    callers_globals: &[OsString],
    inherited_git_dir: Option<&OsStr>,
) -> Option<Vec<OsString>> {
    if !own
        .iter()
        .any(|token| return names_the_place_the_command_acts(token.as_encoded_bytes()))
    {
        return None;
    }
    let directory: &Path = shell_directory?;
    let mut globals: Vec<OsString> = Vec::with_capacity(own.len() + callers_globals.len() + 4);
    globals.push(OsString::from("-C"));
    globals.push(directory.as_os_str().to_os_string());
    if let Some(git_dir) = inherited_git_dir {
        globals.push(OsString::from("--git-dir"));
        globals.push(git_dir.to_os_string());
    }
    globals.extend(without_directory_changes(callers_globals));
    globals.extend(own.iter().cloned());
    return Some(globals);
}

/// What:
///  Whether the caller's own words before the alias name named the repository the Git inside a `!` alias's shell
///  reads: a Git directory, or a work tree.
/// Why:
///  Git gives such a shell a `GIT_DIR` whenever either was named
///  (`setup.c:1247` to `:1254` of Git 2.56.1, through the explicit path that stores the Git directory it found),
///  and the value is read from Git rather than rebuilt from these words,
///  so a relative spelling stays relative as Git stored it.
///  A `-C` is not one of these:
///  Git changes into that directory and finds the repository from there,
///  which is the discovery that exports nothing in a plain main checkout.
///  A `GIT_DIR` or `GIT_WORK_TREE` in the caller's environment is not read here,
///  because the facts' own Git inherits that environment:
///  measured on 2026-11-28, an absolute `GIT_DIR` names the same state whether the reading pins it with
///  `--git-dir` or the facts inherit it, a relative one is rebased by the command's own `-C` either way,
///  and a `GIT_WORK_TREE` makes `--show-toplevel` differ from the Git directory, which the comparison already reads.
///  Gotcha:
///  this rests on every fact request inheriting the caller's environment;
///  a request that unset `GIT_DIR` or `GIT_WORK_TREE` would make the reading name no state where Git's nested
///  command has one, so such a request belongs beside this sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const thePlaceWasNamed = (callersGlobals) => callersGlobals.some((word) => /^--(git-dir|work-tree)(=|$)/.test(word));
/// ```
fn the_place_was_named(callers_globals: &[OsString]) -> bool {
    /// The globals that store a Git directory or a work tree,
    ///  in both spellings Git accepts.
    const NAMED: &[&[u8]] = &[b"--git-dir", b"--git-dir=", b"--work-tree", b"--work-tree="];
    return option_tokens(callers_globals).iter().any(|token| {
        let bytes: &[u8] = token.as_encoded_bytes();
        return NAMED
            .iter()
            .any(|spelling| return bytes == *spelling || bytes.starts_with(spelling));
    });
}

/// Alias resolution,
/// the built-in boundary and the shell alias stay out of the release executable.
#[cfg(test)]
#[path = "guarded_forwarded_command_tests.rs"]
mod tests;
