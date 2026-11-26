//! What:
//!  The settlement of one forwarded invocation:
//!  its bypasses,
//!  what a dead owner left to recover,
//!  and the creating command handed to its own settlement.
//! Why:
//!  A worktree that Git creates (or a bare repository's) needs its ignored files from the source worktree,
//!  and a copy a dead process left is recovered first by a command in the worktree it fills,
//!  under a ceiling,
//!  and never by a command in another worktree.
//!  The caller decides the bypass facts and supplies the runner;
//!  this module never runs the commit refusal.
//!  No lock stands around the command,
//!  so concurrent adds in one repository never wait for one another (the owner's decision of 2026-11-03).
//!
//! Integrator contract:
//!  The caller decides the bypass facts (`--no-worktree-copy`,
//!  short-circuit global help or version,
//!  whether the command creates a worktree after alias resolution,
//!  and the destination its command line names) with the native modules that already own them,
//!  passes the repository identity and the effective working directory,
//!  and supplies the runner that starts real Git.
//!  A main-checkout source is observed only when the command creates a worktree;
//!  it then hands over its ignored state under the main checkout's leave-out rule,
//!  with its registered worktrees excluded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
//! ```

use crate::add_timing::{
    begin_on_route as begin_timing_on_route,
    begin_step,
    end_step,
    finish as finish_timing,
};
use crate::forwarding::NoChildRecorded;
use crate::worktree_copy::{
    GitRunner,
    GitStatus,
    GitStep,
    Invocation,
    Settlement,
    exit_code,
};
/// The hand-over of a snapshot's trash and the package repairs that follow the copy,
/// re-exported so the lifecycle and its controls name one module for them.
pub(crate) use crate::worktree_copy_after_lock::finish_trash_and_repair;
use crate::worktree_copy_creation_settlement::settle_creation;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    lifecycle_failure,
};
use crate::worktree_copy_recovery::{
    RecoveryDestinations,
    recover_pending_worktree_copies,
};
use crate::worktree_copy_registry::{
    WorktreeCopyObservation,
    observe_copy_source,
};
use std::io::Write;

/// What:
///  Runs one forwarded invocation with no synchronization,
/// and settles its status.
/// Why:
///   Every bypass forwards exactly the same way,
/// so the runner is called once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await forwardOnly({ args, gitPath, environment });
/// ```
fn forward_only(run_git: &mut dyn GitRunner) -> Result<Settlement, WorktreeCopyError> {
    // Nothing is copied here, so there is no record for the child's name.
    let git: GitStatus = run_git.run(&GitStep::Person, &mut NoChildRecorded)?;
    return Ok(Settlement {
        git,
        copy_failure: None,
    });
}

/// What:
///  Settles one forwarded invocation:
/// bypasses forward untouched;
/// every other command first recovers what dead owners left,
///  and a creating command then copies the ignored state into the worktree it created.
/// Why:
///   The integrator calls this once per forwarded command and turns the settlement into the exit status with [`exit_code`].
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
/// ```
///
/// The stream is also the one the disposal of a snapshot's trash writes to from a thread of its own,
/// so it is asked
///  for what a thread may carry.
pub fn run_with_worktree_copy(
    invocation: &Invocation,
    run_git: &mut dyn GitRunner,
    stderr: &mut (dyn Write + Send),
) -> Result<Settlement, WorktreeCopyError> {
    return settle_invocation(invocation, run_git, stderr).map_err(lifecycle_failure);
}

/// What:
///  The settlement of one invocation,
/// before the lifecycle words its raw failures.
/// Why:
///   Only the top of the lifecycle words raw failures,
/// as the incumbent's `asWorktreeCopyError` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // body of runGitWithWorktreeCopy
/// ```
fn settle_invocation(
    invocation: &Invocation,
    run_git: &mut dyn GitRunner,
    stderr: &mut (dyn Write + Send),
) -> Result<Settlement, WorktreeCopyError> {
    // `if` chain in the incumbent's order: each bypass forwards without touching the repository.
    if invocation.opted_out || invocation.short_circuit {
        return forward_only(run_git);
    }
    let Some(observation) =
        observe_copy_source(&invocation.identity, invocation.effective_cwd.as_path())?
    else {
        return forward_only(run_git);
    };
    // The record of an add's steps covers every add this wrapper runs,
    // the one the ladder serves and the one it hands to
    //  plain Git: an add the ladder refuses is still an add a person waits through, so it says it is
    // running and it records where its time went.
    let recorded: bool = invocation.creates;
    if recorded {
        // A form the ladder does not serve says which fact put it on the plain route; the ladder's own
        // rung names its steps as it enters them.
        let plain: Option<String> = match &invocation.add_plan {
            Some(_served) => None,
            None => Some(crate::worktree_add_plan::why_a_plain_add(
                invocation.destination.as_deref(),
            )),
        };
        begin_timing_on_route(observation.common_dir.clone(), invocation.quiet, plain);
    }
    let settled: Result<Settlement, WorktreeCopyError> =
        settle_observed(invocation, &observation, run_git, stderr);
    if recorded {
        finish_timing(settled.as_ref().map_or(1, exit_code));
    }
    return settled;
}

/// What:
///  The part of a forwarded command's settlement that follows the observation of its source:
/// recovery,
/// then,
/// for a creating
///  command,
/// the ladder,
/// the copy of the ignored state and the repairs.
/// Why:
///   The caller records the steps of a creating command around this whole part,
/// whichever way it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleObserved({ invocation, observation, runGit, stderr });
/// ```
fn settle_observed(
    invocation: &Invocation,
    observation: &WorktreeCopyObservation,
    run_git: &mut dyn GitRunner,
    stderr: &mut (dyn Write + Send),
) -> Result<Settlement, WorktreeCopyError> {
    // Recovery never waits: it takes over only journals and claims whose owner is dead, and one rename picks its single winner.
    // A copy is finished only into the worktree this command runs in, so no command pays for another worktree's ignored
    // state ("Agents should never wait on each other on commits or merges").
    let destinations: RecoveryDestinations<'_> =
        match crate::worktree_identity::worktree_root(&invocation.identity) {
            Some(root) => RecoveryDestinations::OnlyThisWorktree(root),
            None => RecoveryDestinations::NoWorktree,
        };
    let recover_entered: bool = begin_step("recover");
    let recover_outcome = recover_pending_worktree_copies(
        observation.common_dir.as_path(),
        destinations,
        &invocation.context,
        stderr,
    );
    end_step(recover_entered);
    recover_outcome?;
    if !invocation.creates {
        return forward_only(run_git);
    }
    let (settlement, after) = settle_creation(invocation, observation, run_git, stderr)?;
    // No lock stands around the add: the repairs touch only the new worktree and read the source, and another add never waits
    //  for them.
    #[cfg(target_os = "linux")]
    finish_trash_and_repair(invocation, &after, stderr);
    #[cfg(not(target_os = "linux"))]
    let _after = after;
    return Ok(settlement);
}
