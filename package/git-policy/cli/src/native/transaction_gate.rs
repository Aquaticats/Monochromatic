//! What:
//!  Startup recovery as a step of a wrapped or direct command:
//!  recover,
//!  then report how the
//!       command may go on.
//! Why:
//!  Every command that reads the repository location recovers dead transactions first.
//!  A
//!      guarded command beside a live transaction is still refused until index writers coordinate
//!      with live transactions;
//!  a read-only command goes on.
//!  A legacy directory or a recovery
//!      failure stops the command with exit 2,
//!  as the incumbent's `content-unavailable` engine
//!      failure does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recoverCommitTransaction({ args, gitPath, identity }); // throws CommitTransactionRecoveryError
//! ```

/// Startup recovery.
use super::commit_recovery::{
    RecoveryStop, leaves_live_transactions, legacy_notice, recover_commit_transactions,
    recovery_directories,
};
/// The engine failure event.
use super::diagnostics::{EngineFailureCode, render_engine_failure_with_hint};
/// Running real Git.
use super::transaction_git::GitContext;
/// Registry names.
use super::transaction_registry::TRANSACTION_ROOT_NAME;
/// The repository shape the invocation selects.
use super::worktree_identity::WorktreeIdentity;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  How a command may go on after recovery.
/// Why:
///   Callers refuse,
///  stop or continue from this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Gate = { kind: 'clear' } | { kind: 'live'; registry: string } | { kind: 'stop'; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Gate {
    /// No transaction remains that another process may be running.
    Clear,
    /// Live transactions remain in this registry.
    Live(PathBuf),
    /// Recovery stopped the command;
    ///  this is its standard error.
    Stop(String),
}

/// What:
///  Recover the invocation's worktree and say how the command may go on.
/// Why:
///   Shared by wrapped commands,
///  read-only commands and direct commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverForCommand({ gitPath, identity }): Promise<Gate>;
/// ```
pub fn recover_for_command(context: &GitContext, identity: &WorktreeIdentity) -> Gate {
    // Git runs where this process runs; the context's global options select the repository.
    match recover_commit_transactions(context, Path::new("."), identity) {
        Ok(outcomes) => {
            if !leaves_live_transactions(outcomes.as_slice()) {
                return Gate::Clear;
            }
            let Some((git_dir, _)) = recovery_directories(identity) else {
                return Gate::Clear;
            };
            return Gate::Live(git_dir.join(TRANSACTION_ROOT_NAME));
        }
        Err(RecoveryStop::Legacy(directory)) => {
            return Gate::Stop(legacy_notice(directory.as_path()));
        }
        Err(RecoveryStop::Failure(error)) => {
            // The code stays `content-unavailable` (transaction state is content that could not be read), but the
            // generic hint for that code tells the person to fetch history, which does not apply: the message
            // itself names the file or directory and the ways out.
            return Gate::Stop(render_engine_failure_with_hint(
                0,
                EngineFailureCode::ContentUnavailable,
                error.0.as_str(),
                None,
            ));
        }
    }
}

/// Gate controls stay out of the release executable.
#[cfg(test)]
#[path = "transaction_gate_tests.rs"]
mod tests;
