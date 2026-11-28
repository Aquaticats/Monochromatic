//! What:
//!  Which forwarded invocations the linked-worktree ignored-state copy applies to,
//!  and the runs that answer them:
//!  the plain start of real Git,
//!  the settled run that copies around it,
//!  and the removal a subvolume destroy answers.
//! Why:
//!  A command that creates a worktree must copy the ignored state once real Git returns,
//!  and a command run from a linked worktree must first recover a copy a dead process left;
//!  replacing the process with Git would drop both duties,
//!  so these invocations run Git as a child.
//!  No lock is held:
//!  concurrent commands never wait for one another.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
//! ```

use crate::hooks_off::StartOwner;
use crate::transaction_git::GitContext;
use crate::worktree_clone_removal_proof_and_plan::removal_force::RemovalForce;
use crate::worktree_command_facts::CopyFacts;
/// The ending every run of this route reports,
/// re-exported here so a caller keeps naming one module for the route's answers.
pub use crate::worktree_copy_forward_outcome::ForwardedRun;
use crate::worktree_copy_forward_outcome::{
    common_directory,
    run_plain_of,
};
use crate::worktree_copy_forward_settled_run::run_settled;
use crate::worktree_copying::the_slots_cleanliness::TheLiveCleanliness;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_removal_clears_the_pool::clear_the_pool_of_a_removed_worktree;
use crate::worktree_removal_of_a_bound_clone::{
    BoundSlotRemovalAnswer,
    TheSlotRemoval,
};
use crate::worktree_removal_stops_units::{
    WorktreeAboutToGo,
    before_removal,
    stop_units_of_removed_worktree,
};
use std::ffi::OsString;
use std::path::Path;

/// What:
///  Whether the copy applies to an invocation.
/// Why:
///   Linked worktrees and bare repositories always take the lifecycle (recovery,
/// creation);
///  the main checkout only when it creates a worktree and has paths to hand over,
/// so its other commands
///  keep their plain forwarding and cost no filesystem reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.kind !== 'outside-worktree' && (identity.kind !== 'main-worktree' || (creates && paths.length > 0))
/// ```
pub fn copy_applies(
    identity: &WorktreeIdentity,
    creates: bool,
) -> bool {
    return copy_applies_with(identity, creates, false);
}

/// What:
///  Whether the copy applies to an invocation,
/// when a `git worktree add` or `git worktree remove` that the ladder
///  serves may also run in the main checkout.
/// Why:
///   The ladder snapshots or clones from the creating worktree,
/// and the main checkout is one;
/// a removal is served from
///  any checkout.
/// A creating command from the main checkout takes the lifecycle whatever the configuration names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyApplies(identity, creates) || (servesLadder && identity.kind === 'main-worktree')
/// ```
pub fn copy_applies_with(
    identity: &WorktreeIdentity,
    creates: bool,
    serves_ladder: bool,
) -> bool {
    // `match` on the repository shape; the main checkout takes the lifecycle when the command creates a worktree
    // or the ladder serves it.
    match identity {
        WorktreeIdentity::OutsideWorktree => return false,
        WorktreeIdentity::MainWorktree {
            ..
        } => {
            return serves_ladder || creates;
        }
        WorktreeIdentity::LinkedWorktree {
            ..
        }
        | WorktreeIdentity::BareRepository {
            ..
        } => {
            return true;
        }
    }
}

/// What:
///  Runs the forwarded Git command,
/// settling the ignored-state copy around it when one applies.
/// Why:
///   Git's status is the answer unless the copy failed:
/// a copy failure after a successful Git is status 2,
///  after a failed Git it keeps Git's status,
/// and a failure of the lifecycle itself is status 2,
/// each with its message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
/// ```
pub fn run_forwarded_git(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    copy: Option<&CopyFacts>,
) -> ForwardedRun {
    return run_forwarded_git_with_its_own_steps(
        real_git,
        arguments,
        overlay,
        copy,
        TheRunsOwnSteps {
            the_bound_slots_removal:
                &crate::worktree_removal_of_a_bound_clone::the_removal_of_a_bound_clone,
            stop_units: &stop_units_of_removed_worktree,
        },
    );
}

/// What:
///  The two steps the settled run takes around Git:
///  the removal of a slot a whole managed clone is bound to,
///  which runs before Git,
///  and the stop of a removed worktree's units,
///  which runs after a removal that ended well.
/// Why:
///  Both are this run's own wiring,
///  and a control must reach both without the machine's ZFS device and
///  without the session manager:
///  the removal's step reads a pool,
///  and the stop needs a bus,
///  so each is handed in.
///  The steps belong to one value,
///  so a caller cannot pass the two in the wrong order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const steps: TheRunsOwnSteps = { theBoundSlotsRemoval, stopUnits };
/// ```
pub struct TheRunsOwnSteps<'a> {
    /// What a removal of a slot a whole managed clone is bound to does,
    /// which answers nothing for every other removal.
    pub the_bound_slots_removal: &'a dyn Fn(&TheSlotRemoval<'_>) -> Option<BoundSlotRemovalAnswer>,
    /// The stop of a removed worktree's units,
    /// which is asked only after a removal ended well.
    pub stop_units: &'a dyn Fn(&WorktreeAboutToGo),
}

/// What:
///  The same run with the steps it takes around Git given.
///  `TheRunsOwnSteps` is the removal and the stop a control answers itself.
/// Why:
///   The removal's step reads a pool and the stop reaches the session manager,
///  both of which only the release path may do;
///  the controls hand in their own,
/// so a change that drops either wiring fails there
///  rather than only in the worktree-removal tests that share the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runForwardedGit({ args, gitPath, copy, steps });
/// ```
pub fn run_forwarded_git_with_its_own_steps(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    copy: Option<&CopyFacts>,
    steps: TheRunsOwnSteps<'_>,
) -> ForwardedRun {
    #[cfg(unix)]
    {
        if let Some(facts) = copy {
            // The instance of a removed worktree is a hash of a directory the removal deletes, so it is named first.
            let about: Option<WorktreeAboutToGo> = units_of_worktree_about_to_go(facts);
            let finished: ForwardedRun = run_served(
                real_git,
                arguments,
                overlay,
                facts,
                steps.the_bound_slots_removal,
            );
            // Only a removal that ended well can have taken the worktree; the stop checks that its Git directory is gone.
            if let (Some(gone), ForwardedRun::Ran(0)) = (about.as_ref(), &finished) {
                (steps.stop_units)(gone);
                // The pool of a worktree is named from the same registration the removal deleted, and nothing of the repository looks at it again once the worktree is gone.
                clear_the_pool_of_a_removed_worktree(
                    gone.common_dir.as_path(),
                    gone.git_dir.as_path(),
                );
            }
            return finished;
        }
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = copy;
        let _ = steps;
    }
    return run_plain_of(StartOwner::Person, real_git, arguments, overlay);
}

/// What:
///  Runs Git the wrapper starts for its own work,
/// with the report a forwarded run gives.
/// Why:
///   A step the wrapper substitutes for part of the person's command (the fetch of a pull that merges) is still the
/// wrapper's own Git:
/// it takes the wrapper's options,
/// and its failure is reported like any other run of this route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWrapperOptions({ args, gitPath });
/// ```
pub fn run_wrapper_own_git(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> ForwardedRun {
    return run_plain_of(StartOwner::Wrapper, real_git, arguments, overlay);
}

/// What:
///  The worktree a plain `git worktree remove` names,
/// as its commit service units know it.
/// Why:
///   The units of a removed worktree are stopped once the removal ended (`worktree_removal_stops_units.rs`),
/// and the name they have is made from the registration the removal deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const about = facts.removePlan && beforeRemoval(commonDir, facts.removePlan);
/// ```
#[cfg(unix)]
fn units_of_worktree_about_to_go(facts: &CopyFacts) -> Option<WorktreeAboutToGo> {
    let plan = facts.remove_plan.as_ref()?;
    let common_dir: &Path = common_directory(&facts.identity)?;
    return before_removal(common_dir, plan);
}

/// What:
///  Serve the invocation the way the copy applies to it:
/// a removal by subvolume destroy when every condition holds,
/// else the settled run with Git as a child.
/// Why:
///   The two ways end in the same `ForwardedRun`,
/// so what follows a removal is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await theBoundSlotsRemoval(facts)) ?? (await removeBySubvolume(facts)) ?? (await runSettled(facts));
/// ```
#[cfg(unix)]
fn run_served(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    facts: &CopyFacts,
    the_bound_slots_removal: &dyn Fn(&TheSlotRemoval<'_>) -> Option<BoundSlotRemovalAnswer>,
) -> ForwardedRun {
    // The ZFS clone route serves one `git worktree add` on the design's checkout
    // dataset, and answers nothing for every other command or checkout
    // (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, step 10).
    #[cfg(target_os = "linux")]
    if facts.creates
        && facts.add_plan.is_some()
        && let Some(finished) =
            crate::worktree_clone_add_route::run_a_zfs_add(real_git, arguments, overlay, facts)
    {
        return finished;
    }
    // The refusal reads the worktree the command names whatever its options, which is the target the planner took from
    // every spelling Git accepts, and the common directory the command's repository has (`SPEC.md`, "Removal of a ZFS
    // worktree clone"). The steps are handed in, so a control reaches this refusal without opening the machine's pool.
    #[cfg(target_os = "linux")]
    if let (Some(target), Some(common)) =
        (facts.removal_target.as_deref(), facts.common_dir.as_deref())
    {
        // Git reads the worktree's own files only while the command carries no force, and this route answers before Git
        //  runs, so the same proof is made here through the Git this very command would have started.
        let context: GitContext = GitContext {
            real_git: real_git.to_path_buf(),
            overlay: overlay.to_vec(),
            global_prefix: Vec::new(),
            common_dir: Some(common.to_path_buf()),
        };
        let cleanliness: TheLiveCleanliness<'_> = TheLiveCleanliness {
            context: &context,
        };
        if let Some(answer) = the_bound_slots_removal(&TheSlotRemoval {
            target,
            common_dir: common,
            force: RemovalForce::of_count(facts.removal_force),
            cleanliness: &cleanliness,
        }) {
            return ForwardedRun::Failed {
                stderr: answer.stderr,
                code: answer.code,
            };
        }
    }
    // A command that stands in no worktree has no ladder step and no ignored state to settle: the refusal above is the
    // whole of what this route owes it, and Git runs as it would have without the wrapper.
    if matches!(facts.identity, WorktreeIdentity::OutsideWorktree) {
        return run_plain_of(StartOwner::Person, real_git, arguments, overlay);
    }
    #[cfg(target_os = "linux")]
    if let Some(finished) = remove_by_subvolume(real_git, overlay, facts) {
        return finished;
    }
    return run_settled(real_git, arguments, overlay, facts);
}

/// What:
///  Remove the worktree a plain `git worktree remove` names by destroying its subvolume,
/// when every condition holds.
/// Why:
///   Git's own removal unlinks every file;
/// the destroy is one call.
/// `None` leaves the whole command to Git;
/// a removal
///  that destroyed the files but could not remove the registration fails the command with the repair steps and is never
///  handed to Git,
/// whose removal would find the worktree already gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// facts.removePlan === undefined ? undefined : outcomeToRun(await removeSubvolumeWorktree(request))
/// ```
#[cfg(all(unix, target_os = "linux"))]
fn remove_by_subvolume(
    real_git: &Path,
    overlay: &[(OsString, OsString)],
    facts: &CopyFacts,
) -> Option<ForwardedRun> {
    use crate::worktree_remove::{
        RemoveOutcome,
        RemoveRequest,
        remove_subvolume_worktree,
    };
    use std::io::Write;
    let plan = facts.remove_plan.as_ref()?;
    let common_dir: &Path = common_directory(&facts.identity)?;
    let context: crate::transaction_git::GitContext = crate::transaction_git::GitContext {
        real_git: real_git.to_path_buf(),
        overlay: overlay.to_vec(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    let outcome: crate::worktree_remove::RemoveOutcome =
        remove_subvolume_worktree(&RemoveRequest {
            context: &context,
            plan,
            common_dir,
            last_look: None,
        });
    match outcome {
        RemoveOutcome::LeftToGit => return None,
        RemoveOutcome::Removed | RemoveOutcome::DestroyedInPlace => {
            // Exact terminal output: the person runs a destroy that cannot be undone and accepts the risk that comes with it, which a forced removal words differently because nothing was checked for changes, and which a destroy made here words differently because nothing waits for the commit service then.
            let handed_over: bool = outcome == RemoveOutcome::Removed;
            let notice: &str = crate::worktree_remove::destroy_notice(plan.force, handed_over);
            let _closed = writeln!(std::io::stderr(), "{notice}");
            return Some(ForwardedRun::Ran(0));
        }
        RemoveOutcome::RemovedAround(left) => {
            // The worktree is gone from Git's records and what was not this removal's stands where it was, so the command did
            // what it was asked and says the rest in the same breath: the path is still taken, and this is what frees it.
            let report: String =
                crate::worktree_removal_goes_around_a_foreign_volume::around_removal_report(
                    plan.target.as_path(),
                    left.as_slice(),
                );
            let _closed = writeln!(std::io::stderr(), "{report}");
            return Some(ForwardedRun::Ran(0));
        }
        RemoveOutcome::RemovedAroundNothingLeft => {
            // The destroy's line says a delete the commit service still has to make, and this path has none: the files went
            // one by one here, the volume that kept the root standing went away during that pass, and the empty root went
            // with one `rmdir`.
            let report: String =
                crate::worktree_removal_goes_around_a_foreign_volume::emptied_around_report(
                    plan.target.as_path(),
                );
            let _closed = writeln!(std::io::stderr(), "{report}");
            return Some(ForwardedRun::Ran(0));
        }
        RemoveOutcome::Incomplete(report) => {
            return Some(ForwardedRun::Failed {
                stderr: format!("{report}\n"),
                code: crate::action::ENGINE_FAILURE_EXIT_CODE,
            });
        }
        RemoveOutcome::Refused(report) => {
            // A refusal is not a failure of a step: nothing changed, and the status is the one every refusal of this wrapper
            // uses.
            return Some(ForwardedRun::Failed {
                stderr: format!("{report}\n"),
                code: crate::subvolume_command::REFUSED_EXIT_CODE,
            });
        }
    }
}

/// Forwarding-run controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_forward_tests.rs"]
mod tests;
