//! What:
//!  Git's own checkout,
//! run in the worktree the add is bound to when a rung cannot finish the add,
//! and the report of an add that could not be completed.
//! Why:
//!  Real Git ran with `--no-checkout`,
//! so the destination holds only its `.git` file and this checkout is Git's own step.
//! Nothing is deleted and nothing is unregistered on a failure:
//! the worktree and its registration stay exactly as Git left them,
//! and the person is told where they are and which command finishes or removes them.
//! The binding is verified before the run and again after it,
//! so a worktree that was moved or replaced meanwhile is never reset and a success against a replacement is never taken for the add's.
//! A checkout pointed at an index file of a rung's own keeps that name the rung's to publish,
//! and the rung's own run is silent because the person reads the fallback's line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const status = completeLikeGit({ plan, bound, runGit }); // reports what it left when Git fails
//! const quietStatus = checkoutLikeGitWith({ plan, bound, runGit }, indexPath, true);
//! ```

use crate::diagnostic_log::warn;
use crate::worktree_add_binding::{
    Bound,
    verify_association,
};
use crate::worktree_command_facts::AddPlan;
use crate::worktree_copy::{
    GitRunner,
    GitStatus,
};
use crate::worktree_ladder_target::{
    checkout_environment_with_index,
    run_step,
};
use std::ffi::OsString;
use std::path::Path;

/// Import the tag every line of this folder's ladder work carries.
use crate::worktree::worktree_ladder_finish_binding::TAG;

/// What:
///  Complete the worktree the way Git does after a refused rung:
/// `git reset --hard --no-recurse-submodules`,
/// in the bound
///  worktree and nowhere else.
/// Why:
///   Real Git ran with `--no-checkout`,
/// so the destination holds only its `.git` file;
/// this is Git's own checkout step.
///  The binding is verified first and again after,
/// so a worktree that was moved or replaced meanwhile is never reset and a
///  success against a replacement is never taken for the add's.
/// Git's `post-checkout` hook is not run:
/// every real-Git start
///  of the wrapper switches hooks off (`hooks_off.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkoutLikeGit(args): Promise<GitStatus>; // failure when the binding does not hold
/// ```
fn checkout_like_git(
    plan: &AddPlan,
    bound: &Bound,
    run_git: &mut dyn GitRunner,
) -> GitStatus {
    return checkout_like_git_with((plan, bound, run_git), None, plan.quiet);
}

/// What:
///  `checkout_like_git` against the index file `index` when it is named,
///  and silent when `quiet` says the caller's own run prints Git's line.
/// Why:
///   The clone rung writes its index under a name of its own until the last write through Git is done
/// (`worktree_clone_index::PendingIndex`),
/// so a checkout of a worktree whose tree difference failed is pointed at that file with `GIT_INDEX_FILE`:
/// the file holds the source's entries,
/// which is what makes a path the target tree lacks a tracked-then-deleted one and removes it,
///  and a path Git must write is checked out because its entry holds no stat data.
/// The rung's own run is silent because the person reads the line of the fallback that follows,
///  which runs after the index took the worktree's own name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkoutLikeGitWith({ plan, bound, runGit }, index, quiet): Promise<GitStatus>;
/// ```
pub(crate) fn checkout_like_git_with(
    (plan, bound, run_git): (&AddPlan, &Bound, &mut dyn GitRunner),
    index: Option<&Path>,
    quiet: bool,
) -> GitStatus {
    let destination: &Path = plan.destination.as_path();
    if let Err(reason) = verify_association(bound) {
        warn(
            TAG,
            format!(
                "monochromatic-git-policy-cli: the checkout of {} was not started: {reason}",
                destination.display()
            )
            .as_str(),
        );
        return GitStatus {
            code: Some(1),
        };
    }
    let mut arguments: Vec<OsString> = ["-C"]
        .iter()
        .map(OsString::from)
        .chain([destination.as_os_str().to_os_string()])
        .chain(
            ["reset", "--hard", "--no-recurse-submodules"]
                .iter()
                .map(OsString::from),
        )
        .collect();
    if quiet {
        arguments.push(OsString::from("--quiet"));
    }
    let environment: Vec<(OsString, OsString)> =
        checkout_environment_with_index((bound.binding.admin.as_path(), destination), index);
    let status: GitStatus = match run_step(run_git, arguments.as_slice(), environment.as_slice()) {
        Ok(status) => status,
        Err(reason) => {
            warn(
                TAG,
                format!(
                    "Git's checkout of {} could not start ({reason})",
                    destination.display()
                )
                .as_str(),
            );
            return GitStatus {
                code: Some(1),
            };
        }
    };
    if let (true, Err(reason)) = (status.succeeded(), verify_association(bound)) {
        warn(TAG, format!("monochromatic-git-policy-cli: the checkout of {} ran, but it cannot be taken for the add's: {reason}", destination.display()).as_str());
        return GitStatus {
            code: Some(1),
        };
    }
    return status;
}

/// What:
///  Say what an add left,
/// loudly and once,
/// with the commands that finish or remove it.
/// Why:
///   The ladder deletes and unregisters nothing,
/// so a worktree that could not be completed stays registered and incomplete,
///  as Git's own interrupted add leaves it;
/// the person is told where and how to remove it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reportIncomplete(plan);
/// ```
pub(crate) fn report_incomplete(plan: &AddPlan) {
    let shown: String = plan.destination.display().to_string();
    let finish: String =
        crate::worktree_finish_commands::finish_or_remove_sentence(plan.destination.as_path());
    warn(TAG, format!("monochromatic-git-policy-cli: the `git worktree add` of {shown} could not be completed; the worktree and its registration were left as they are. {finish}").as_str());
}

/// What:
///  Complete the worktree with Git's own checkout after a refused rung.
/// When that fails,
/// report what was left.
/// Why:
///   Nothing is removed on a failure:
/// the worktree and its registration stay and the person gets the report.
/// Success records nothing:
/// the claim travels on to the copy,
/// so a command killed before that copy's journal exists still leaves a record naming the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const status = await completeLikeGit({ plan, bound, runGit });
/// ```
pub(crate) fn complete_like_git(
    plan: &AddPlan,
    bound: &Bound,
    run_git: &mut dyn GitRunner,
) -> GitStatus {
    let status: GitStatus = checkout_like_git(plan, bound, run_git);
    if !status.succeeded() {
        report_incomplete(plan);
    }
    return status;
}
