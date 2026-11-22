//! What:
//!  The clone rung's serving step:
//! the copies,
//! the fence beside them,
//! the index written under a name of its own,
//! the write step through Git and the moment the index takes the worktree's name.
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
use crate::worktree_clone_index::PendingIndex;
#[cfg(feature = "measurement-routes")]
use crate::worktree_clone::{Cloned, clone_tracked_files};
use crate::worktree_clone_conversion_beside_the_clone::{fence_beside_the_clone, install_beside_the_clone};
use crate::worktree_copy::{GitRunner, GitStatus};
use crate::worktree_ladder_target::{Target, apply_difference_with_index, prepare_target};
use crate::worktree_snapshot_source::SnapshotSource;
use std::path::Path;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadder";

/// What:
///  The clone rung:
/// copy the tracked files into the destination,
/// have the fence answer,
/// write the index under a name of its own,
/// apply the tree difference,
/// write through Git every path the fence left to it,
/// and give the index the worktree's own name.
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
///  the seed it hands the install holds no stat data for the paths Git still has to write,
///  and it is published under the worktree's own name only after that write;
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
    let (created, decided, pending): (Vec<crate::worktree_clone::Created>, crate::worktree_clone_conversion_fence::Fence, Option<PendingIndex>) = clone_with_the_fence((source, fence, target.admin.as_path(), bound, context))?;
    let index_file: Option<std::path::PathBuf> = pending.as_ref().map(|held| return held.path_in().to_path_buf());
    let applied: Result<(), String> = match verify_association(bound) {
        Ok(()) => apply_difference_with_index((source, &target, plan.destination.as_path(), run_git), index_file.as_deref()),
        Err(reason) => Err(reason),
    };
    if let Err(reason) = applied {
        // What:
        //  Complete the worktree with Git's own checkout against the index this rung wrote under its own name.
        // Why:
        //  The two-tree update refuses an entry the fence left to Git (`error: Entry 'extra.txt' not uptodate. Cannot
        //  merge.`), because that entry holds no stat data and the update verifies the entries of the paths it is about to
        //  change. Git's own `reset --hard` against the same file is the checkout the rung owes: the file holds the source's
        //  entries, so a path the target tree lacks is a tracked-then-deleted one there and the reset removes it, and a path
        //  Git must write is checked out because its entry is not up to date. The index gains the worktree's name only after
        //  that run, so a kill inside it leaves no index that could vouch for a path Git had not written; the ladder's own
        //  fallback then runs the same checkout against the published index, whose entries describe what is on disk.
        //  The run is silent because that fallback prints Git's line.
        let Some(waits) = pending else {
            // A fence that decided before the copies installed its index under the worktree's own name, so the ladder's
            // fallback runs against that file and removes what the target tree lacks by itself.
            return Err(reason);
        };
        let settled: GitStatus = super::checkout_like_git_with((plan, bound, run_git), Some(waits.path_in()), true);
        if !settled.succeeded() {
            warn(TAG, format!("monochromatic-git-policy-cli: {}", leftover_report(created.as_slice(), plan.destination.as_path())).as_str());
            waits.discard(&bound.admin);
            return Err(reason);
        }
        waits.publish(&bound.admin)?;
        return Err(reason);
    }
    if !decided.git_writes.is_empty() {
        let written: Result<(), String> = match pending.as_ref().map(|held| return held.path_in()) {
            Some(index) => crate::worktree_clone_conversion_fence::checkout_index_through_the_index_at((target.admin.as_path(), plan.destination.as_path(), context), index, decided.git_writes.as_slice(), source.clean.hash_length),
            None => crate::worktree_clone_conversion_fence::write_through_git(target.admin.as_path(), plan.destination.as_path(), context, decided.git_writes.as_slice(), source.clean.hash_length),
        };
        if let Err(reason) = written {
            discard_the_pending_index(pending, &bound.admin);
            return Err(reason);
        }
    }
    // What:
    //  The index takes the worktree's own name here, after the last write this rung makes through Git.
    // Why:
    //  A published index whose entry holds the clean form of the source's bytes makes `git status` refresh the entry instead of
    //  reporting the path (issue 6323), so a kill between the publication and the write left a worktree that vouched for
    //  bytes Git had not written. Written under a name of its own and published here, the file is not the worktree's index
    //  while that window is open, and a kill there leaves no index at all: the unfinished shape the recovery already names.
    if let Some(held) = pending {
        held.publish(&bound.admin)?;
    }
    return Ok(target.line);
}

/// What:
///  Copy the tracked files and get the fence's answer,
/// in the order the rung's fence shape names,
/// and write the index the fence's answer allows,
/// under a name of its own when the worktree's name must wait for Git's write.
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
/// async function cloneWithTheFence(args): Promise<[Created[], Fence, PendingIndex?]>; // throws a reason
/// ```
pub(super) fn clone_with_the_fence((source, fence, admin_path, bound, context): (&SnapshotSource, crate::worktree_clone_conversion_beside_the_clone::CloneFence, &Path, &Bound, &GitContext)) -> Result<(Vec<crate::worktree_clone::Created>, crate::worktree_clone_conversion_fence::Fence, Option<PendingIndex>), String> {
    return match fence {
        #[cfg(feature = "measurement-routes")]
        crate::worktree_clone_conversion_beside_the_clone::CloneFence::BeforeTheClone(decided) => {
            let cloned: Cloned = clone_tracked_files(source, &bound.destination, &bound.admin)?;
            Ok((cloned.into_created(), decided, None))
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
            let pending: PendingIndex = install_beside_the_clone((&bound.admin, admin_path), source, &decided, cloned.stats())?;
            Ok((cloned.into_created(), decided, Some(pending)))
        }
    };
}

/// What:
///  Remove the index this rung wrote under its own name,
/// so the record of what is on disk is Git's checkout's to write.
/// Why:
///   A rung that ends before the publication leaves the file behind,
/// and nothing but this run knows it is not the worktree's index;
/// the removal checks the identity,
///  so a name another process took stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (pending) pending.discard(admin);
/// ```
fn discard_the_pending_index(pending: Option<PendingIndex>, admin: &std::fs::File) {
    if let Some(held) = pending {
        held.discard(admin);
    }
}
