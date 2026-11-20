//! What:
//!  The clone rung's steps after real Git returned:
//! clone the paths the source's proof settles,
//! apply the tree
//!  difference,
//! and prove the worktree is the one Git's own checkout makes.
//! Why:
//!  A clean source clones every tracked file and installs its index;
//! a source with uncommitted changes
//!  clones the paths the proof settles,
//! has its index built from the tree of the new `HEAD`,
//! and lets Git write the rest
//!  (`doc/planning/cli-git-clone-rung-partial-clone-of-a-dirty-source.md`).
//! Either way the destination is filled in place and nothing
//!  a failed run created is removed:
//! Git's own checkout completes the worktree and overwrites the tracked ones,
//!  so one report names what stays.
//! The proof is one `git status` and one index comparison against the new `HEAD`,
//! which is the
//!  state a person asks about first,
//! and a rung that left a path the source's change touched ends there and lets Git's
//!  checkout make the worktree.
//! The steps sit in a file of their own so `worktree_ladder_finish.rs` stays under the line limit,
//! and this file
//!  needs nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = serveClones({ source, plan, bound, runGit, context }); // throws a reason
//! ```

use crate::add_timing::{begin_step, end_step};
use crate::diagnostic_log::warn;
use crate::transaction_git::GitContext;
use crate::worktree_add_binding::{Bound, verify_association};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone::{Cloned, Created, clone_tracked_files, leftover_report};
use crate::worktree_clone_partial::{PartCloned, SizeRules, clone_the_unchanged, prove_the_worktree};
use crate::worktree_copy::GitRunner;
use crate::worktree_ladder_target::{Target, apply_difference, prepare_target};
use crate::worktree_snapshot_source::SnapshotSource;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadderFinishCloneRung";

/// What:
///  Say what a failed clone run left in the destination,
/// in one warning.
/// Why:
///   The entries a run created stay,
/// and the person is told where to look when the checkout that completes the
///  worktree does not overwrite all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// warn(leftoverReport(created, destination));
/// ```
pub(crate) fn report_leftovers(created: &[Created], plan: &AddPlan) {
    if created.is_empty() {
        return;
    }
    warn(TAG, format!("monochromatic-git-policy-cli: {}", leftover_report(created, plan.destination.as_path())).as_str());
}

/// What:
///  The clone rung:
/// clone the tracked files into the destination,
/// install the index,
/// apply the tree difference,
/// and prove the result is the worktree Git's own checkout makes.
/// Why:
///   The destination is filled in place;
/// an error removes nothing:
/// the entries the run created stay,
/// are reported,
/// and Git's
///  own checkout completes the worktree and overwrites the tracked ones.
/// A source with uncommitted changes clones the paths the proof still settles,
/// has its index built from
///  the tree of the new `HEAD`,
/// and lets Git write the rest;
/// a clean source keeps the rung it had.
/// The proof after the tree difference is one `git status` and one comparison of the index with the new `HEAD`;
/// a rung that
///  left a path the source's change touched ends here,
/// and Git's own checkout makes the worktree instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serveClones(args): Promise<string>; // the checkout line; throws a reason
/// ```
pub(crate) fn serve_clones(source: &SnapshotSource, plan: &AddPlan, bound: &Bound, run_git: &mut dyn GitRunner, context: &GitContext) -> Result<String, String> {
    verify_association(bound)?;
    let target: Target = prepare_target(source, plan, context, bound)?;
    let part: Option<PartCloned> = match source.dirty.as_ref() {
        Some(_changes) => Some(clone_the_unchanged(context, source, &target, plan.destination.as_path(), &bound.destination, &bound.admin, &SizeRules::from_the_environment())?),
        None => None,
    };
    let full: Option<Cloned> = if part.is_some() { None } else { Some(clone_tracked_files(source, &bound.destination, &bound.admin)?) };
    let created: &[Created] = match part.as_ref() {
        Some(found) => found.created(),
        None => match full.as_ref() {
            Some(found) => found.created(),
            None => &[],
        },
    };
    let applied: Result<(), String> = match verify_association(bound) {
        Ok(()) => apply_difference(source, &target, plan.destination.as_path(), run_git),
        Err(reason) => Err(reason),
    };
    if let Err(reason) = applied {
        report_leftovers(created, plan);
        return Err(reason);
    }
    if part.is_some() {
        let prove_entered: bool = begin_step("partProve");
        let proven: Result<(), String> = prove_the_worktree(context, &target, plan.destination.as_path());
        end_step(prove_entered);
        if let Err(reason) = proven {
            report_leftovers(created, plan);
            return Err(reason);
        }
    }
    return Ok(target.line);
}
