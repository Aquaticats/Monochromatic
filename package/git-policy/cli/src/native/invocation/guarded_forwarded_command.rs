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
use crate::forwarded_command::{ResolvedCommand, Route, resolve_forwarded_command};
use crate::git_builtins::is_git_builtin;
use crate::global_arguments::command_tokens;
use crate::rule_linked_worktree::{LinkedWorktreeDecision, decide_guarded_word};
/// The transaction Git context the resolver asks for an alias.
use crate::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  The guard's decision and the arguments it decided about.
///  A `struct` is a record with named fields.
/// Why:
///   A caller that records what it decided has to name the command Git will run,
/// which for an alias is the
///  expansion and not the alias name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GuardedForwarded = { decision: LinkedWorktreeDecision; args: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuardedForwarded {
    /// What the guard decides about the command Git will run.
    pub decision: LinkedWorktreeDecision,
    /// The arguments Git receives:
    ///  the caller's own,
    ///  or the alias expansion the wrapper forwards.
    pub arguments: Vec<OsString>,
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
            arguments: arguments.to_vec(),
        };
    };
    // `.as_encoded_bytes()` lends the raw bytes of the word.
    if is_git_builtin(word.as_encoded_bytes()) {
        return GuardedForwarded {
            decision: decide_guarded_word(word.as_encoded_bytes(), region),
            arguments: arguments.to_vec(),
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
                arguments: arguments.to_vec(),
            };
        };
        let Some((shell_word, shell_region)) = command_tokens(shell_command.as_slice()) else {
            return GuardedForwarded {
                decision: LinkedWorktreeDecision::Pass,
                arguments: arguments.to_vec(),
            };
        };
        return GuardedForwarded {
            decision: decide_guarded_word(shell_word.as_encoded_bytes(), shell_region),
            arguments: arguments.to_vec(),
        };
    }
    if resolved.route != Route::Alias || resolved.subcommand.is_none() {
        return GuardedForwarded {
            decision: LinkedWorktreeDecision::Pass,
            arguments: arguments.to_vec(),
        };
    }
    return GuardedForwarded {
        decision: decide_guarded_word(resolved.word(), resolved.region()),
        arguments: resolved.arguments,
    };
}

/// Alias resolution,
/// the built-in boundary and the shell alias stay out of the release executable.
#[cfg(test)]
#[path = "guarded_forwarded_command_tests.rs"]
mod tests;
