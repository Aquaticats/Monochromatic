//! What:
//!  Running a forwarded index writer as a child of the wrapper:
//!  wait for the commit service's own hold on the index,
//! complete an isolated merge when one applies,
//! run real Git
//!  (again when the service took the index first),
//! and report its status.
//! Why:
//!  A writer competes for the index only through Git's own `index.lock`,
//! as it does without the wrapper;
//! the one holder
//!  that is waited for is the commit service,
//! whose hold ends by itself.
//! The wrapper stays alive around Git so that a
//!  run the service beat to the lock can be started again and so that a merge the staged index would make Git refuse can
//!  be completed.
//! A command that needs neither replaces the process with Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await waitForServiceHold({ realIndex, command });
//! await runGitWithWorktreeCopy({ args, gitPath, environment });
//! ```

/// The wait for any program's hold on the index lock,
/// before a merge that native Git runs.
use crate::index_holder_wait::wait_for_index_release_here;
/// The wait for the commit service's own hold on the index.
use crate::service_index_wait::wait_for_service_hold_here;
/// Starting a command again when the commit service took the index first.
use crate::service_hold_retry::{may_run_again, run_again_while_service_holds};
/// Running real Git,
/// settling the linked-worktree copy around it when one applies.
use crate::worktree_copy_forward::{CopyFacts, ForwardedRun, run_forwarded_git};
/// The resolved forwarded command.
use crate::forwarded_command::ResolvedCommand;
/// The classification of index writers.
use crate::index_writer_commands::is_index_writer;
/// The index a command writes.
use crate::landing_location::{LandingLocation, resolve_landing_location};
/// The failure event rendered for standard error.
use crate::diagnostics::{EngineFailureCode, render_engine_failure};
/// The once-per-process Git version check.
use crate::git_version::check_git_version;
/// The repository shape the invocation selects.
use crate::worktree_identity::WorktreeIdentity;
/// A merge the staged index would make native Git refuse,
/// completed without disturbing the staged work.
use crate::merge_isolation::{IsolatedMerge, IsolatedMergeInput, run_isolated_merge};
/// A pull that merges,
/// split into its fetch and its merge so the merge is completed in isolation.
use crate::pull_isolation::{PullStep, pull_step};
/// The word-level second pass for the conflicts a merge,
/// rebase,
/// cherry-pick or revert created.
use crate::merge_second_pass_worktree::{Watch, resolve_after, watch_before};
/// Running real Git for the location query.
use crate::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// Writing the merge's own output.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::PathBuf;

/// What:
///  Everything a forward that stays alive around Git needs,
/// owned,
/// so the planned action can carry it.
/// Why:
///   The plan decides that the wrapper stays alive;
/// the run performs it.
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
    /// The linked-worktree copy to settle around Git,
    /// when it applies to this invocation.
    pub copy: Option<CopyFacts>,
}

/// What:
///  How a forward that stays alive around Git ended.
/// Why:
///   Only a run that did not stay alive may replace the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardOutcome = 'not-coordinated' | { exitCode: number } | { signal: number } | { stderr: string; exitCode: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoordinatedRun {
    /// Nothing needs the wrapper alive;
    ///  the caller replaces its process with Git.
    NotCoordinated,
    /// Git ran as a child and exited with this status.
    Ran(i32),
    /// The signal that ended Git,
    /// whose death the caller repeats after the run has returned.
    Killed(i32),
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
///  The location of the real index a command writes,
/// or nothing when the command writes none that the commit service holds.
/// Why:
///   Only a command that writes the worktree's own index competes with the service's landing;
/// a command outside a
///  repository,
/// or one writing another index file,
/// has nothing to wait for.
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
///  Run the forward when it stays alive around Git,
/// and say when it does not.
/// Why:
///   `forwardToRealGit`:
/// a writer waits for the service's hold,
/// runs Git as a child,
/// and returns Git's status;
/// a merge
///  that native Git would refuse only for the staged index is completed first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function forwardToRealGit({ args, gitPath, identity }): Promise<void>;
/// ```
pub fn run_coordinated_forward(typed: &CoordinatedForward) -> CoordinatedRun {
    // The context runs the location query with the same overlay as the command.
    let context: GitContext = GitContext {
        real_git: typed.real_git.clone(),
        overlay: typed.overlay.clone(),
        global_prefix: Vec::new(),
        // The location query sends no input, so no scratch directory is needed.
        common_dir: None,
    };
    let location: LandingLocation = match written_real_index(&context, &typed.command) {
        Ok(Some(found)) => found,
        Ok(None) => {
            // A copy keeps the wrapper alive around Git; otherwise the caller may replace the process.
            let Some(facts) = typed.copy.as_ref() else {
                return CoordinatedRun::NotCoordinated;
            };
            return from_forwarded(run_forwarded_git(
                typed.real_git.as_path(),
                typed.arguments.as_slice(),
                typed.overlay.as_slice(),
                Some(facts),
            ));
        }
        Err(reason) => return failed_with(EngineFailureCode::ContentUnavailable, reason.as_str()),
    };
    // The version check runs once per process, before the first work that depends on Git's behavior.
    if let Err(reason) = check_git_version(&context) {
        return failed_with(EngineFailureCode::ContentUnavailable, reason.as_str());
    }
    // A pull that merges runs its fetch here and goes on as the `git merge FETCH_HEAD` it ends with; any other pull is native Git's.
    let merge_of_pull: CoordinatedForward;
    // A merge that cannot create a commit (a fast-forward, or nothing to merge) is native Git's without the isolated attempt.
    let mut isolate: bool = true;
    let forward: &CoordinatedForward = if typed.command.word() == b"pull" {
        match pull_step(typed, location.real_index_path.as_path()) {
            PullStep::Native => typed,
            PullStep::Ended(run) => return run,
            PullStep::Merge { forward: merge, makes_commit } => {
                merge_of_pull = *merge;
                isolate = makes_commit;
                &merge_of_pull
            }
        }
    } else {
        typed
    };
    let name: String = String::from_utf8_lossy(forward.command.word()).into_owned();
    // What: The branch as the merge found it, read before Git moves it.
    // Why:  The worktrees whose branches the merge lands are those new to the landed branch, and only this reading says
    //       which commits the landed branch did not hold before (`worktree_landed_after_a_merge.rs`).
    let merge_window: Option<(GitContext, String)> = if forward.command.word() == b"merge" {
        let sweep_context: GitContext = GitContext {
            real_git: forward.real_git.clone(),
            overlay: forward.overlay.clone(),
            global_prefix: Vec::new(),
            common_dir: None,
        };
        match crate::worktree_landed_after_a_merge::head_of(&sweep_context, &forward.identity) {
            Some(before) => Some((sweep_context, before)),
            None => None,
        }
    } else {
        None
    };
    // The commit service holds the index while it lands a commit, which ends by itself: the command waits for it, naming
    //  it, before Git starts. A lock that is not the service's is a foreign program's and keeps the behavior of native Git.
    wait_for_service_hold_here(location.real_index_path.as_path(), name.as_str());
    // A merge that native Git would refuse only because the index holds staged changes is completed in isolation.
    if forward.command.word() == b"merge" && !isolate {
        // Native Git never waits for the index lock; the fast-forward it is handed waits for the holder first.
        wait_for_index_release_here(location.real_index_path.as_path(), name.as_str());
    } else if forward.command.word() == b"merge" {
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
                // Native Git never waits for the index lock and dies on a held one (`stash failed` for a merge commit):
                //  the merge it is handed waits for the holder first, bounded and naming it.
                wait_for_index_release_here(location.real_index_path.as_path(), name.as_str());
            }
            IsolatedMerge::Finished { code, stdout, stderr } => {
                let _ = std::io::stdout().write_all(stdout.as_bytes());
                let _ = std::io::stderr().write_all(stderr.as_bytes());
                swept_after_a_merge(merge_window.as_ref(), &forward.identity, code);
                return CoordinatedRun::Ran(code);
            }
        }
    }
    // The paths that are unmerged before the command runs are not the second merge pass's to touch afterwards.
    let second_pass_watch: Option<Watch> = watch_before(forward);
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
    let finished: CoordinatedRun = from_forwarded(outcome);
    // A merge, rebase, cherry-pick or revert that Git stopped with a conflict gets the word-level second pass.
    // The pass awaits every Git start it makes, so a person's command reaches it through one entry of the runtime.
    if let CoordinatedRun::Ran(code) = &finished {
        let passed_on: Option<i32> = resolve_after(second_pass_watch.as_ref(), forward, *code, &mut std::io::stderr());
        // Every conflicted file resolved and the operation was ended like a clean run: that run's status is the command's.
        if let Some(ended) = passed_on {
            swept_after_a_merge(merge_window.as_ref(), &forward.identity, ended);
            return CoordinatedRun::Ran(ended);
        }
    }
    if let CoordinatedRun::Ran(code) = &finished {
        swept_after_a_merge(merge_window.as_ref(), &forward.identity, *code);
    }
    return finished;
}

/// What:
///  Sweep the worktrees whose branches the merge landed,
///  when this command was one.
/// Why:
///  A merge that landed a branch is the event that ends those worktrees' use,
///  and the sweep must never turn a made merge into a failed command,
///  so every answer of it is only printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweptAfterAMerge(window, identity, code); // nothing when this command was no merge
/// ```
fn swept_after_a_merge(window: Option<&(GitContext, String)>, identity: &WorktreeIdentity, code: i32) {
    let Some((context, before)) = window else {
        return;
    };
    let _swept = crate::worktree_landed_after_a_merge::after_a_merge(context, identity, before.as_str(), code);
}

/// What:
///  The coordinated run that a forwarded run stands for.
/// Why:
///   Both report the same two endings,
/// so the status and the failure text pass through unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // the same exit status or the same failure text
/// ```
fn from_forwarded(run: ForwardedRun) -> CoordinatedRun {
    // `match` on the ending: a status, or a failure with its own text.
    match run {
        ForwardedRun::Ran(code) => return CoordinatedRun::Ran(code),
        // What: A Git that a signal ended stays a death by that signal.
        // Why:  This run kept a process around Git, so it cannot simply be Git's own process; the caller ends itself
        //       by the same signal once the run has returned (`end_as_git_ended`). Reporting 128 + the signal here made a
        //       caller see a normal exit with that status, where native Git's caller sees `WIFSIGNALED`
        //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, finding 4 of the round-1 review).
        ForwardedRun::Killed(signal) => return CoordinatedRun::Killed(signal),
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
        code: crate::action::ENGINE_FAILURE_EXIT_CODE,
    };
}

/// Coordinated forwarding controls stay out of the release executable.
#[cfg(test)]
#[path = "index_writer_forward_tests.rs"]
mod tests;
