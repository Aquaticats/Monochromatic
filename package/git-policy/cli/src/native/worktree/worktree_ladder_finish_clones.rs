//! What:
//!  The clone rung's serving step:
//! the copies,
//! the fence beside them,
//! the index install and the write step through Git.
//! Why:
//!  The module that binds an add to the directories Git made is under the line limit,
//! and this is the one rung whose steps are reordered around a concurrency
//! (`doc/planning/cli-git-clone-rung-conversion-fence.md`,
//! option B).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = serveClones(source, fence, plan, bound, runGit, context); // throws the reason
//! ```

use crate::add_timing::{begin_step, end_step};
use crate::diagnostic_log::warn;
use crate::transaction_git::GitContext;
use crate::worktree_add_binding::{Bound, verify_association};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone::{ClonedBytes, clone_tracked_bytes_beside, leftover_report};
#[cfg(feature = "measurement-routes")]
use crate::worktree_clone::{Cloned, clone_tracked_files};
use crate::worktree_clone_conversion_beside_the_clone::{fence_beside_the_clone, install_beside_the_clone};
use crate::worktree_copy::GitRunner;
use crate::worktree_ladder_target::{Target, apply_difference, prepare_target};
use crate::worktree_snapshot_source::SnapshotSource;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadder";

/// What:
///  The clone rung:
/// copy the tracked files into the destination,
/// have the fence answer,
/// install the index,
/// apply the tree difference,
/// and write through Git every path the fence left to it.
/// Why:
///   The destination is filled in place;
/// an error removes nothing:
/// the entries the run created stay,
/// are reported,
/// and Git's
///  own checkout completes the worktree and overwrites the tracked ones.
/// The fence's two shapes put the same steps in a different order
/// (`CloneFence`):
/// beside the copies,
///  nothing is installed until the fence has answered,
///  and the seed it hands the install holds no stat data for the paths Git still has to write;
/// before the copies,
///  option A′ of the fence's report,
///  every copied path is proven and the index may vouch for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serveClones(args): Promise<string>; // the checkout line; throws a reason
/// ```
pub(super) fn serve_clones(source: &SnapshotSource, fence: crate::worktree_clone_conversion_beside_the_clone::CloneFence, plan: &AddPlan, bound: &Bound, run_git: &mut dyn GitRunner, context: &GitContext) -> Result<String, String> {
    verify_association(bound)?;
    let target: Target = prepare_target(source, plan, context, bound)?;
    let beside: bool = matches!(fence, crate::worktree_clone_conversion_beside_the_clone::CloneFence::Beside(ref _held));
    let (created, decided): (Vec<crate::worktree_clone::Created>, crate::worktree_clone_conversion_fence::Fence) = clone_with_the_fence(source, fence, bound, context)?;
    let applied: Result<(), String> = match verify_association(bound) {
        Ok(()) => apply_difference(source, &target, plan.destination.as_path(), run_git),
        Err(reason) => Err(reason),
    };
    if let Err(reason) = applied {
        warn(TAG, format!("monochromatic-git-policy-cli: {}", leftover_report(created.as_slice(), plan.destination.as_path())).as_str());
        return Err(reason);
    }
    if !decided.git_writes.is_empty() {
        if beside {
            crate::worktree_clone_conversion_fence::checkout_index_through_git(target.admin.as_path(), plan.destination.as_path(), context, decided.git_writes.as_slice(), source.clean.hash_length)?;
        } else {
            crate::worktree_clone_conversion_fence::write_through_git(target.admin.as_path(), plan.destination.as_path(), context, decided.git_writes.as_slice(), source.clean.hash_length)?;
        }
    }
    return Ok(target.line);
}

/// What:
///  Copy the tracked files and get the fence's answer,
/// in the order the rung's fence shape names,
/// and install the index the fence's answer allows.
/// Why:
///   The order is the option.
/// In the shipped shape the copies and the fence's remaining reads (`git ls-tree` and the decision) run together,
/// the join is taken whatever the copy's answer was,
/// and the index is installed from a seed that holds no stat data for a path Git still has to write,
///  so no step between the copy and Git's write records such a path clean;
/// the attributes and the decline were decided before any copy,
/// so a declined source never reaches this step;
/// a fence that ends the rung leaves no index at all,
///  and Git's own checkout writes the whole worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function cloneWithTheFence(args): Promise<[Created[], Fence]>; // throws a reason
/// ```
pub(super) fn clone_with_the_fence(source: &SnapshotSource, fence: crate::worktree_clone_conversion_beside_the_clone::CloneFence, bound: &Bound, context: &GitContext) -> Result<(Vec<crate::worktree_clone::Created>, crate::worktree_clone_conversion_fence::Fence), String> {
    return match fence {
        #[cfg(feature = "measurement-routes")]
        crate::worktree_clone_conversion_beside_the_clone::CloneFence::BeforeTheClone(decided) => {
            let cloned: Cloned = clone_tracked_files(source, &bound.destination, &bound.admin)?;
            Ok((cloned.into_created(), decided))
        }
        crate::worktree_clone_conversion_beside_the_clone::CloneFence::Beside(held) => {
            let beside_entered: bool = begin_step("conversionBesideTheClone");
            let (cloned, answered): (ClonedBytes, Option<Result<crate::worktree_clone_conversion_fence::Fence, String>>) = clone_tracked_bytes_beside(source, &bound.destination, fence_beside_the_clone(source, context, held))?;
            end_step(beside_entered);
            // A fence's reason, or a pool that would not run it, ends the rung here before any index exists: Git's own
            // checkout writes the worktree.
            let decided: crate::worktree_clone_conversion_fence::Fence = match answered {
                Some(Ok(decided)) => decided,
                Some(Err(reason)) => return Err(reason),
                None => return Err(String::from("the fence beside the copies answered nothing")),
            };
            install_beside_the_clone(&bound.admin, source, &decided, cloned.stats())?;
            Ok((cloned.into_created(), decided))
        }
    };
}
