//! What:
//!  Running a forwarded index writer as a child of the wrapper:
//!  wait for the commit service's own hold on the index, complete an isolated merge when one applies, run real Git
//!  (again when the service took the index first), and report its status.
//! Why:
//!  A writer competes for the index only through Git's own `index.lock`, as it does without the wrapper; the one holder
//!  that is waited for is the commit service, whose hold ends by itself. The wrapper stays alive around Git so that a
//!  run the service beat to the lock can be started again and so that a merge the staged index would make Git refuse can
//!  be completed. A command that needs neither replaces the process with Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await waitForServiceHold({ realIndex, command });
//! await runGitWithWorktreeCopy({ args, gitPath, environment });
//! ```

/// The wait for the commit service's own hold on the index.
use super::service_index_wait::wait_for_service_hold_here;
/// Starting a command again when the commit service took the index first.
use super::service_hold_retry::{may_run_again, run_again_while_service_holds};
/// Running real Git, settling the linked-worktree copy around it when one applies.
use super::worktree_copy_forward::{CopyFacts, ForwardedRun, run_forwarded_git};
/// The resolved forwarded command.
use super::forwarded_command::ResolvedCommand;
/// The classification of index writers.
use super::index_writer_commands::is_index_writer;
/// The index a command writes.
use super::landing_location::{LandingLocation, resolve_landing_location};
/// The failure event rendered for standard error.
use super::diagnostics::{EngineFailureCode, render_engine_failure};
/// The once-per-process Git version check.
use super::git_version::check_git_version;
/// The repository shape the invocation selects.
use super::worktree_identity::WorktreeIdentity;
/// A merge the staged index would make native Git refuse, completed without disturbing the staged work.
use super::merge_isolation::{IsolatedMerge, IsolatedMergeInput, run_isolated_merge};
/// Running real Git for the location query.
use super::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// Writing the merge's own output.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::PathBuf;

/// What:
///  Everything a forward that stays alive around Git needs, owned, so the planned action can carry it.
/// Why:
///   The plan decides that the wrapper stays alive; the run performs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardToRealGit = { realGit: string; args: string[]; overlay: [string, string][]; identity; command; environment };
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
    ///  which decides what runs around Git.
    pub command: ResolvedCommand,
    /// The environment the invocation inherited.
    pub environment: Vec<(OsString, OsString)>,
    /// The linked-worktree copy to settle around Git, when it applies to this invocation.
    pub copy: Option<CopyFacts>,
}

/// What:
///  How a forward that stays alive around Git ended.
/// Why:
///   Only a run that did not stay alive may replace the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardOutcome = 'not-coordinated' | { exitCode: number } | { stderr: string; exitCode: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoordinatedRun {
    /// Nothing needs the wrapper alive;
    ///  the caller replaces its process with Git.
    NotCoordinated,
    /// Git ran as a child and exited with this status.
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
///  The location of the real index a command writes, or nothing when the command writes none that the commit service holds.
/// Why:
///   Only a command that writes the worktree's own index competes with the service's landing; a command outside a
///  repository, or one writing another index file, has nothing to wait for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!isIndexWriter(command) || location.indexPath !== location.realIndexPath) return undefined;
/// ```
fn written_real_index(context: &GitContext, command: &ResolvedCommand) -> Result<Option<LandingLocation>, String> {
    if !is_index_writer(command) {
        return Ok(None);
    }
    let global: &[OsString] = &command.arguments[..command.subcommand_index];
    let Some(location) = resolve_landing_location(context, global)? else {
        return Ok(None);
    };
    if location.index_path != location.real_index_path {
        return Ok(None);
    }
    return Ok(Some(location));
}

/// What:
///  Run the forward when it stays alive around Git, and say when it does not.
/// Why:
///   `forwardToRealGit`: a writer waits for the service's hold, runs Git as a child, and returns Git's status; a merge
///  that native Git would refuse only for the staged index is completed first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function forwardToRealGit({ args, gitPath, identity }): Promise<void>;
/// ```
pub fn run_coordinated_forward(forward: &CoordinatedForward) -> CoordinatedRun {
    // The context runs the location query with the same overlay as the command.
    let context: GitContext = GitContext {
        real_git: forward.real_git.clone(),
        overlay: forward.overlay.clone(),
        global_prefix: Vec::new(),
        // The location query sends no input, so no scratch directory is needed.
        common_dir: None,
    };
    let location: LandingLocation = match written_real_index(&context, &forward.command) {
        Ok(Some(found)) => found,
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
        Err(reason) => return failed_with(EngineFailureCode::ContentUnavailable, reason.as_str()),
    };
    // The version check runs once per process, before the first work that depends on Git's behavior.
    if let Err(reason) = check_git_version(&context) {
        return failed_with(EngineFailureCode::ContentUnavailable, reason.as_str());
    }
    let name: String = String::from_utf8_lossy(forward.command.word()).into_owned();
    // The commit service holds the index while it lands a commit, which ends by itself: the command waits for it, naming
    //  it, before Git starts. A lock that is not the service's is a foreign program's and keeps the behavior of native Git.
    wait_for_service_hold_here(location.real_index_path.as_path(), name.as_str());
    // A merge that native Git would refuse only because the index holds staged changes is completed in isolation.
    if forward.command.word() == b"merge" {
        let merge_input: IsolatedMergeInput<'_> = IsolatedMergeInput {
            real_git: forward.real_git.as_path(),
            overlay: forward.overlay.as_slice(),
            identity: &forward.identity,
            command: &forward.command,
            environment: forward.environment.as_slice(),
        };
        match run_isolated_merge(&merge_input) {
            IsolatedMerge::Forward { notice } => {
                if let Some(text) = notice {
                    let _ = std::io::stderr().write_all(text.as_bytes());
                }
            }
            IsolatedMerge::Finished { code, stdout, stderr } => {
                let _ = std::io::stdout().write_all(stdout.as_bytes());
                let _ = std::io::stderr().write_all(stderr.as_bytes());
                return CoordinatedRun::Ran(code);
            }
        }
    }
    // A command that takes the index lock before it changes anything is run again when the commit service took the index first.
    let outcome: ForwardedRun = if forward.copy.is_none() && may_run_again(forward.command.word(), forward.command.region()) {
        run_again_while_service_holds(
            forward.real_git.as_path(),
            forward.arguments.as_slice(),
            forward.overlay.as_slice(),
            forward.environment.as_slice(),
            location.real_index_path.as_path(),
            name.as_str(),
            &mut std::io::stderr(),
        )
    } else {
        run_forwarded_git(
            forward.real_git.as_path(),
            forward.arguments.as_slice(),
            forward.overlay.as_slice(),
            forward.copy.as_ref(),
        )
    };
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
///  The engine failure a command that could not start reports.
/// Why:
///   A location or version that could not be read stops the command with the engine failure's status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderPolicyEvents([createEngineFailureEvent({ code, message })])
/// ```
fn failed_with(code: EngineFailureCode, message: &str) -> CoordinatedRun {
    return CoordinatedRun::Failed {
        stderr: render_engine_failure(0, code, message),
        code: super::action::ENGINE_FAILURE_EXIT_CODE,
    };
}

/// Coordinated forwarding controls stay out of the release executable.
#[cfg(test)]
#[path = "index_writer_forward_tests.rs"]
mod tests;
