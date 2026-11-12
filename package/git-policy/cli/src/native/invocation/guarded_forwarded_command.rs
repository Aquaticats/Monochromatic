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
use crate::external_git_command::alias_run_directory;
use crate::forwarded_command::{ResolvedCommand, Route, resolve_forwarded_command};
use crate::git_builtins::is_git_builtin;
use crate::global_arguments::{GlobalLayout, command_tokens, global_layout};
use crate::rule_linked_worktree::{LinkedWorktreeDecision, decide_guarded_word};
/// The transaction Git context the resolver asks for an alias.
use crate::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path of raw bytes.
use std::path::PathBuf;

/// What:
///  The guard's decision,
///  the words it decided about,
///  the arguments Git receives and,
///  for a `!` alias,
///  the directory its shell runs in.
///  A `struct` is a record with named fields.
/// Why:
///   A caller that records what it decided has to name the command Git will run,
/// which for an alias is the
///  expansion and not the alias name;
///  a caller that reads the flags and paths of that command needs the words the decision was
///  made on,
///  which are not the words Git receives for a `!` alias;
///  and a caller that resolves a relative path of that
///  command needs the directory Git starts its shell in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GuardedForwarded = { decision: LinkedWorktreeDecision; decidedWords: string[]; args: string[]; shellDirectory?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardedForwarded {
    /// What the guard decides about the command Git will run.
    pub decision: LinkedWorktreeDecision,
    /// What:
    ///  The words the guard decided about:
    ///  the caller's own for a built-in word or a pass,
    ///  the expansion for an alias Git runs in place of the word,
    ///  and the words the shell of a `!` alias runs.
    /// Why:
    ///   A `!` alias's shell runs a Git command whose flags and paths Git appends the caller's
    ///  own words to,
    ///  and the wrapper forwards the caller's words unchanged,
    ///  so `arguments` names the alias while this names the command inside it;
    ///  a policy that reads the flags or paths of the command Git runs reads them here,
    ///  or it judges a command nobody runs.
    pub decided_words: Vec<OsString>,
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
}

/// What:
///  What the guard decides about one invocation,
/// after alias expansion.
/// Why:
///   A built-in word is never an alias,
///  so it needs no configuration read;
///  a shell alias and a name nothing
///       defines have no subcommand this guard can read,
///  so they are left to Git and to whatever the shell
///       finally runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decideGuardedForwarded({ context, args }): GuardedForwarded;
/// ```
pub fn decide_guarded_forwarded(
    context: &GitContext,
    environment: &[(OsString, OsString)],
    arguments: &[OsString],
) -> GuardedForwarded {
    // `let Some((word, region)) = ... else { ... };` unwraps the subcommand, or passes when Git runs none.
    let Some((word, region)) = command_tokens(arguments) else {
        return GuardedForwarded {
            decision: LinkedWorktreeDecision::Pass,
            decided_words: arguments.to_vec(),
            arguments: arguments.to_vec(),
            shell_directory: None,
        };
    };
    // `.as_encoded_bytes()` lends the raw bytes of the word.
    if is_git_builtin(word.as_encoded_bytes()) {
        return GuardedForwarded {
            decision: decide_guarded_word(word.as_encoded_bytes(), region),
            decided_words: arguments.to_vec(),
            arguments: arguments.to_vec(),
            shell_directory: None,
        };
    }
    // The word names no built-in, so Git looks it up as an alias, and an undefined name is left to Git.
    let resolved: ResolvedCommand = resolve_forwarded_command(context, environment, arguments);
    // A value that starts with `!` runs a shell, and Git puts its own executable directory first on that shell's
    //  `PATH`, so the `git` such an alias names is the real Git and never this wrapper. The one plain `git` command
    //  the resolver read from the value is what the shell runs, and the guard decides on it while the caller's own
    //  arguments stay untouched: Git runs the shell with an environment a substituted argument vector would lose.
    if resolved.route == Route::ShellAlias {
        let Some(shell_command) = &resolved.shell_command else {
            return GuardedForwarded {
                decision: LinkedWorktreeDecision::Pass,
                decided_words: arguments.to_vec(),
                arguments: arguments.to_vec(),
                shell_directory: None,
            };
        };
        let Some((shell_word, shell_region)) = command_tokens(shell_command.as_slice()) else {
            return GuardedForwarded {
                decision: LinkedWorktreeDecision::Pass,
                decided_words: arguments.to_vec(),
                arguments: arguments.to_vec(),
                shell_directory: None,
            };
        };
        return GuardedForwarded {
            decision: decide_guarded_word(shell_word.as_encoded_bytes(), shell_region),
            // The shell runs this command, so its flags and paths are the ones the rules read.
            decided_words: shell_command.clone(),
            arguments: arguments.to_vec(),
            // The command the shell runs is read from the directory Git put the shell in, not from the caller's.
            shell_directory: shell_run_directory(context, arguments),
        };
    }
    if resolved.route != Route::Alias || resolved.subcommand.is_none() {
        return GuardedForwarded {
            decision: LinkedWorktreeDecision::Pass,
            decided_words: arguments.to_vec(),
            arguments: arguments.to_vec(),
            shell_directory: None,
        };
    }
    return GuardedForwarded {
        decision: decide_guarded_word(resolved.word(), resolved.region()),
        // An ordinary alias is replaced by its expansion, which is also what Git receives.
        decided_words: resolved.arguments.clone(),
        arguments: resolved.arguments,
        shell_directory: None,
    };
}

/// What:
///  The directory Git runs the shell of a `!` alias in,
///  asked of Git with the caller's own global options.
///  `&[OsString]` borrows the caller's whole argument vector;
///  `Option<PathBuf>` is that directory,
///  and nothing when it cannot be named.
/// Why:
///   Git changes to the top of the working tree before it starts such a shell
///  (`git.c:388` to `git.c:407`),
///  so the same relative path names one file in the caller's directory
///  and another in the shell's.
///  `alias_run_directory` is the one answer to that question in this crate,
///  the same one the search for an installed `git-<word>` uses,
///  so a path read here cannot land somewhere Git never goes.
///  This asks Git only for an alias whose shell runs one plain Git command,
///  and a caller that gets nothing falls back to the directory the caller's options reach,
///  which is where Git runs every other command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shellRunDirectory = (globals, start) => aliasRunDirectory({ globals, start });
/// ```
fn shell_run_directory(context: &GitContext, arguments: &[OsString]) -> Option<PathBuf> {
    let layout: GlobalLayout = global_layout(arguments);
    let globals: &[OsString] = &arguments[..layout.prefix_len];
    let start: PathBuf = std::env::current_dir().ok()?;
    return alias_run_directory(context, globals, Some(start.as_path()));
}

/// Alias resolution,
/// the built-in boundary and the shell alias stay out of the release executable.
#[cfg(test)]
#[path = "guarded_forwarded_command_tests.rs"]
mod tests;
