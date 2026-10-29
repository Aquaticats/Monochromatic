//! What:
//!  Running a forwarded index writer under its landing lock:
//!  coordinate,
//!  run real Git as a child
//!       that inherits the landing lease,
//!  release the lock once Git returns.
//! Why:
//!  Replacing the process would drop the lock with the process image,
//!  so a coordinated writer
//!      waits for Git instead.
//!  The nested wrappers Git starts see the lease and do not wait on it
//!      (`src/forward-real-git.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await using coordination = await coordinateIndexWriter({ command, gitPath, timeoutMs });
//! await runGitWithWorktreeCopy({ args, gitPath, environment: coordination.environment });
//! ```

/// Coordination with landings.
use super::index_writer_coordination::{CoordinationFailure, CoordinationInput, coordinate_index_writer};
/// Running real Git, settling the linked-worktree copy around it when one applies.
use super::worktree_copy_forward::{CopyFacts, ForwardedRun, run_forwarded_git};
/// The resolved forwarded command.
use super::forwarded_command::ResolvedCommand;
/// The repository shape the invocation selects.
use super::worktree_identity::WorktreeIdentity;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::PathBuf;

/// What:
///  Everything a coordinated forward needs,
///  owned,
///  so the planned action can carry it.
/// Why:
///   The plan decides that coordination is needed;
///  the run performs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardToRealGit = { realGit: string; args: string[]; overlay: [string, string][]; identity; command; environment; timeoutMs };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoordinatedForward {
    /// Real Git executable.
    pub real_git: PathBuf,
    /// Arguments Git receives,
    ///  as the caller gave them.
    pub arguments: Vec<OsString>,
    /// Environment variables added to Git's inherited environment.
    pub overlay: Vec<(OsString, OsString)>,
    /// Repository shape of the invocation.
    pub identity: WorktreeIdentity,
    /// The command after alias expansion,
    ///  which decides the coordination.
    pub command: ResolvedCommand,
    /// The environment the invocation inherited,
    ///  which may carry an ancestor's lease.
    pub environment: Vec<(OsString, OsString)>,
    /// `indexLock.unprovenOwnerTimeoutMs`.
    pub timeout_ms: u64,
    /// The linked-worktree copy to settle around Git, when it applies to this invocation.
    pub copy: Option<CopyFacts>,
}

/// What:
///  How a coordinated forward ended.
/// Why:
///   Only a run that was not coordinated may replace the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardOutcome = 'not-coordinated' | { exitCode: number } | { stderr: string; exitCode: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoordinatedRun {
    /// No coordination applies;
    ///  the caller replaces its process with Git.
    NotCoordinated,
    /// Git ran under the landing lock and exited with this status.
    Ran(i32),
    /// The command did not run;
    ///  its standard error and exit status.
    Failed {
        /// Text for standard error.
        stderr: String,
        /// Exit status.
        code: i32,
    },
}

/// What:
///  Coordinate,
///  run Git as a child with the lease,
///  release the lock,
///  and report the status.
/// Why:
///   `forwardToRealGit`:
///  the lock is held until real Git returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function forwardToRealGit({ args, gitPath, identity, unprovenOwnerTimeoutMs }): Promise<void>;
/// ```
pub fn run_coordinated_forward(forward: &CoordinatedForward) -> CoordinatedRun {
    // The context runs the coordination's own Git queries with the same overlay as the command.
    let context: crate::transaction_git::GitContext = crate::transaction_git::GitContext {
        real_git: forward.real_git.clone(),
        overlay: forward.overlay.clone(),
        global_prefix: Vec::new(),
        // The coordination queries send no input, so no scratch directory is needed.
        common_dir: None,
    };
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &context,
        identity: &forward.identity,
        command: &forward.command,
        environment: forward.environment.as_slice(),
        timeout_ms: forward.timeout_ms,
    };
    let coordination = match coordinate_index_writer(&input) {
        Ok(Some(held)) => held,
        Ok(None) => {
            // A copy keeps the wrapper alive around Git; otherwise the caller may replace the process.
            let Some(facts) = forward.copy.as_ref() else {
                return CoordinatedRun::NotCoordinated;
            };
            return from_forwarded(run_forwarded_git(
                forward.real_git.as_path(),
                forward.arguments.as_slice(),
                forward.overlay.as_slice(),
                Some(facts),
            ));
        }
        Err(failure) => return failed_with(&failure),
    };
    let mut child_overlay: Vec<(OsString, OsString)> = forward.overlay.clone();
    child_overlay.push(coordination.lease.clone());
    let outcome: ForwardedRun = run_forwarded_git(
        forward.real_git.as_path(),
        forward.arguments.as_slice(),
        child_overlay.as_slice(),
        forward.copy.as_ref(),
    );
    let released: Result<(), String> = coordination.release();
    return finish(outcome, released);
}

/// What:
///  The run's result once Git returned and the lock was released.
/// Why:
///   Git's status is the answer unless the lock could not be released,
///  which is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return { exitCode } } finally { await coordination[Symbol.asyncDispose](); }
/// ```
fn finish(outcome: ForwardedRun, released: Result<(), String>) -> CoordinatedRun {
    if let Err(reason) = released {
        return CoordinatedRun::Failed {
            stderr: format!("cli-git: the landing lock could not be released: {reason}\n"),
            code: super::action::ENGINE_FAILURE_EXIT_CODE,
        };
    }
    return from_forwarded(outcome);
}

/// What:
///  The coordinated run that a forwarded run stands for.
/// Why:
///   Both report the same two endings, so the status and the failure text pass through unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the same exit status or the same failure text
/// ```
fn from_forwarded(run: ForwardedRun) -> CoordinatedRun {
    // `match` on the ending: a status, or a failure with its own text.
    match run {
        ForwardedRun::Ran(code) => return CoordinatedRun::Ran(code),
        ForwardedRun::Failed { stderr, code } => return CoordinatedRun::Failed { stderr, code },
    }
}

/// What:
///  The engine failure a refused coordination reports.
/// Why:
///   The unproven owner and read failures each keep their own event.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderPolicyEvents([createEngineFailureEvent({ code, message })])
/// ```
fn failed_with(failure: &CoordinationFailure) -> CoordinatedRun {
    return CoordinatedRun::Failed {
        stderr: failure.stderr(),
        code: super::action::ENGINE_FAILURE_EXIT_CODE,
    };
}

/// Coordinated forwarding controls stay out of the release executable.
#[cfg(test)]
#[path = "index_writer_forward_tests.rs"]
mod tests;
