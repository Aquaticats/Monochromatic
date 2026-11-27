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
use crate::forwarded_command::{
    ResolvedCommand,
    Route,
    resolve_forwarded_command,
    shell_run_directory,
    the_command_git_runs,
};
use crate::git_builtins::is_git_builtin;
use crate::global_arguments::{
    command_tokens,
    names_the_place_the_command_acts,
};
use crate::rule_linked_worktree::{
    LinkedWorktreeDecision,
    decide_guarded_word,
};
/// The transaction Git context the resolver asks for an alias.
use crate::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
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
        //  policies judged the caller's until they were asked from there.
        let shell_directory: Option<PathBuf> = shell_run_directory(context, &resolved, start);
        return GuardedForwarded {
            decision: decide_guarded_word(command_git_runs.word(), command_git_runs.region()),
            arguments: arguments.to_vec(),
            // The command the shell runs is read from the directory Git put the shell in, not from the caller's.
            shell_directory: shell_directory.clone(),
            globals_that_place_the_command: globals_that_place_the_command(
                command_git_runs.arguments.as_slice(),
                command_git_runs.subcommand_index,
                shell_directory.as_deref(),
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
///  The global options that reach the place one command Git runs acts on,
///  read from the directory Git runs its shell in,
///  or nothing when that command names no place of its own or that directory was not read.
/// Why:
///   A `!` alias's shell runs at the top of the working tree Git changed to,
///  and the words Git appends to the alias value may begin with a global option
///  (`git.c:401` to `git.c:402`),
///  so `-C <main>` there names a directory relative to that top.
///  `-C` first makes the facts start where the shell started,
///  and the command's own options then move them exactly as Git moves the nested Git.
///  A command whose own options name no place keeps the caller's own options,
///  which reach the same worktree the shell runs in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const globalsThatPlaceTheCommand = (args, index, shellDirectory) => {
///   const own = args.slice(0, index);
///   if (!own.some(namesThePlace) || shellDirectory === undefined) return undefined;
///   return ['-C', shellDirectory, ...own];
/// };
/// ```
pub(crate) fn globals_that_place_the_command(
    arguments: &[OsString],
    subcommand_index: usize,
    shell_directory: Option<&Path>,
) -> Option<Vec<OsString>> {
    let own: &[OsString] = arguments.get(..subcommand_index).unwrap_or(&[]);
    if !own
        .iter()
        .any(|token| return names_the_place_the_command_acts(token.as_encoded_bytes()))
    {
        return None;
    }
    let directory: &Path = shell_directory?;
    let mut globals: Vec<OsString> = Vec::with_capacity(own.len() + 2);
    globals.push(OsString::from("-C"));
    globals.push(directory.as_os_str().to_os_string());
    globals.extend(own.iter().cloned());
    return Some(globals);
}

/// Alias resolution,
/// the built-in boundary and the shell alias stay out of the release executable.
#[cfg(test)]
#[path = "guarded_forwarded_command_tests.rs"]
mod tests;
