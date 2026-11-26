//! What:
//!  What the ladder does after real Git returned:
//! the clone rung's serving step,
//! the stage claim a rung took or gives back,
//! and the choice between the two rungs and the fall back to Git's own checkout.
//! The binding of an add, the snapshot rung's serving and Git's own checkout
//! are the sibling files `worktree_ladder_finish_binding.rs`,
//! `worktree_ladder_finish_snapshot.rs` and `worktree_ladder_finish_git_checkout.rs`.
//! Why:
//!  The ladder never deletes or unregisters a worktree:
//! when the fallback
//!  fails,
//! or the bound objects are no longer what they were,
//! the worktree and its registration stay exactly as they are and
//!  one loud report names what is incomplete and the command that removes it.
//! The only deletions are of objects this process
//!  made and still checks:
//! the stage it took and never exchanged.
//! The line that Git's
//!  own checkout would have printed is printed here,
//! so the person cannot tell the rungs apart except by the time they take.
//!  No hook runs on any rung:
//! every real-Git start of the wrapper switches hooks off (`hooks_off.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await ladder.finish({ git, runGit, context, out, note });
//! ```

use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::transaction_git::GitContext;
use crate::worktree_add_binding::Bound;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_copy::{
    GitRunner,
    GitStatus,
};
use crate::worktree_copy_stage_claim::StageClaim;
use crate::worktree_ladder_finish_binding::{
    bind,
    release_claim,
};
use crate::worktree_ladder_finish_git_checkout::{
    complete_like_git,
    report_incomplete,
};
use crate::worktree_ladder_finish_snapshot::finish_snapshot;
use crate::worktree_ladder_run::{
    Ladder,
    LadderResult,
    Rung,
};
use crate::worktree_slow_route::{
    SlowRoute,
    explain_clone_failure,
};
use crate::worktree_snapshot_source::SnapshotSource;
use crate::worktree_subvolume::settle_created_destination;
use std::io::Write;
use std::path::PathBuf;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadder";

/// What:
///  Serve the clone rung after real Git returned,
/// or complete the worktree with Git's own checkout.
/// Why:
///   A failed attempt has removed what it created;
/// the ignored state is copied by the lifecycle afterwards,
/// as for Git's
///  checkout,
/// so nothing is reported as carried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finishClones(args): Promise<LadderResult>;
/// ```
fn finish_clones(
    source: &SnapshotSource,
    fence: crate::worktree_clone_conversion_beside_the_clone::CloneFence,
    plan: &AddPlan,
    git: GitStatus,
    claim: StageClaim,
    run_git: &mut dyn GitRunner,
    context: &GitContext,
    out: &mut dyn Write,
    slow: &mut Option<SlowRoute>,
    entries: Option<Vec<crate::worktree_clean::StatEntry>>,
) -> LadderResult {
    if !git.succeeded() {
        return LadderResult {
            git,
            carried: None,
            placed: None,
            trash: None,
            claim: Some(claim),
        };
    }
    let bound: Bound = match bind(&claim, source, plan) {
        Ok(found) => found,
        Err(reason) => {
            warn(TAG, format!("monochromatic-git-policy-cli: {} was left as Git made it, without a checkout: {reason}", plan.destination.display()).as_str());
            report_incomplete(plan);
            return LadderResult {
                git: GitStatus {
                    code: Some(1),
                },
                carried: None,
                placed: None,
                trash: None,
                claim: Some(claim),
            };
        }
    };
    return match serve_clones(source, fence, plan, &bound, run_git, context, entries) {
        Ok(line) => {
            debug(
                TAG,
                format!("clones served {}", plan.destination.display()).as_str(),
            );
            // A closed standard output leaves nowhere to print Git's line; the exit status still carries the result.
            let _written = out.write_all(line.as_bytes());
            LadderResult {
                git,
                carried: None,
                placed: None,
                trash: None,
                claim: Some(claim),
            }
        }
        Err(reason) => {
            debug(
                TAG,
                format!("the clone rung ended: {reason}; Git checks the worktree out").as_str(),
            );
            // The record said which rung was chosen; the worktree this add publishes is Git's checkout, and a report that
            // read the route would otherwise take the clone for what wrote it.
            crate::add_timing::note("route", "Git's checkout after the clone rung ended");
            // The person is told why the clone rung ended, the fence's decline included, and the sentence names the clone
            // rung and not the snapshot.
            *slow = Some(explain_clone_failure(reason.as_str()));
            LadderResult {
                git: complete_like_git(plan, &bound, run_git),
                carried: None,
                placed: None,
                trash: None,
                claim: Some(claim),
            }
        }
    };
}

impl Ladder {
    /// What:
    ///  Settle the ladder after real Git returned `git`:
    /// serve the snapshot rung or fall back to Git's checkout,
    /// or settle
    ///  the empty subvolume of the Git rung.
    /// Why:
    ///   The result carries the status the person should see and the destination whose ignored state a snapshot carried.
    /// A claim a rung took travels out in the result,
    /// for the copy that owns it until its journal names the stage,
    /// so the record of an
    ///  add never disappears between the end of its checkout and the copy;
    /// a claim no rung took,
    /// or one whose rung cannot serve the add at all,
    ///  is released here.
    /// The two streams are the ones the command was given:
    /// `out` takes the line Git's own checkout prints,
    /// and `note`
    ///  takes the notes about what the add did instead,
    /// because a caller that hands in its own report reads the whole of it.
    /// `note` is one plain writer and not a shared one:
    /// every note here is written before any thread
    ///  this command starts exists.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const result = await ladder.finish({ git, runGit, context, out, note });
    /// ```
    pub fn finish(
        self,
        git: GitStatus,
        run_git: &mut dyn GitRunner,
        context: &GitContext,
        out: &mut dyn Write,
        note: &mut dyn Write,
    ) -> LadderResult {
        // What:
        //  The size read this add started before its source check is settled here,
        //  after Git's own run,
        //  when the fence never took it.
        // Why:
        //  An add the check refused or the fence declined may pay that read in the background
        //  (`doc/decision/cli-git-fence-size-read-keeps-git-ls-tree.md`, "The overlap, answered 2026-11-22"),
        //  so the add's own record states it when it has already answered, as work,
        //  and as started and not joined when it has not;
        //  a read the fence used has nothing left.
        // The settle never waits: a read longer than this add's own Git run must not make the add slower than on `main`,
        //  and the read's thread and child end with this process (`GitRequest::ends_with_the_starting_thread`).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // self.sizesBeside?.settle();
        // ```
        let sizes_beside: Option<
            crate::worktree_clone_conversion_size_read::SizeReadBesideTheCheck,
        > = self.sizes_beside;
        if let Some(read) = sizes_beside {
            read.settle();
        }
        let mut slow: Option<SlowRoute> = self.slow;
        let result: LadderResult = match (self.rung, self.plan.as_ref()) {
            (Rung::Snapshot(source, claim, fence), Some(plan)) => {
                if !git.succeeded() {
                    release_claim(claim, false);
                    return LadderResult {
                        git,
                        carried: None,
                        placed: None,
                        trash: None,
                        claim: None,
                    };
                }
                let stage: PathBuf = claim.container().to_path_buf();
                finish_snapshot(
                    &source,
                    plan,
                    stage.as_path(),
                    claim,
                    &fence,
                    run_git,
                    context,
                    out,
                    note,
                    &mut slow,
                )
            }
            (Rung::Snapshot(_, claim, _fence), None) => {
                release_claim(claim, false);
                LadderResult {
                    git,
                    carried: None,
                    placed: None,
                    trash: None,
                    claim: None,
                }
            }
            (Rung::Clone(source, subvolume, claim, fence, entries), Some(plan)) => {
                let result: LadderResult = finish_clones(
                    &source, fence, plan, git, claim, run_git, context, out, &mut slow, entries,
                );
                if let Some(created) = subvolume.as_ref() {
                    settle_created_destination(created);
                }
                result
            }
            (Rung::Clone(_, subvolume, claim, _, _), None) => {
                if let Some(created) = subvolume.as_ref() {
                    settle_created_destination(created);
                }
                release_claim(claim, false);
                LadderResult {
                    git,
                    carried: None,
                    placed: None,
                    trash: None,
                    claim: None,
                }
            }
            (Rung::Git(subvolume), _) => {
                if let Some(created) = subvolume.as_ref() {
                    settle_created_destination(created);
                }
                LadderResult {
                    git,
                    carried: None,
                    placed: None,
                    trash: None,
                    claim: None,
                }
            }
        };
        // Only an add that succeeded is told as a slow one; a refused or failed add has a worse thing to say.
        if let (Some(route), Some(plan), true, None) = (
            slow.as_ref(),
            self.plan.as_ref(),
            result.git.succeeded(),
            result.carried.as_ref(),
        ) {
            route.tell(plan);
        }
        return result;
    }
}

/// The clone rung's serving step,
/// which is its own file so this one stays under the line limit.
#[path = "worktree_ladder_finish_clones.rs"]
mod clones;

use clones::serve_clones;

/// Ladder finish controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_finish_tests.rs"]
mod tests;
