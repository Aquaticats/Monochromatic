//! What:
//!  The choice of rung for one invocation,
//! every check that can be made before real Git runs,
//! cheapest first:
//! whether a snapshot applies,
//! the source it would carry,
//! the conversion fence's answer,
//! and then the same for clones,
//! with the sentence that says why the add is not a snapshot.
//! Why:
//!  A refused rung must cost no extra Git run,
//! so each refusal leaves its reason in the log and,
//! in words,
//! with the person once Git has succeeded.
//! The one exception is rung 1's size read:
//! a read started before the source check is ended,
//! not paid,
//! when the check refuses.
//! The record of an add comes before everything that writes,
//! because without it a killed add could not be removed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { rung, slow } = chooseRung(plan, context, identity, held, sizesBeside);
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
};
use crate::diagnostic_log::debug;
use crate::transaction_git::GitContext;
use crate::worktree_clone_conversion_beside_the_clone::CloneFence;
use crate::worktree_clone_conversion_fence::{
    AttributeFence,
    Fence,
};
use crate::worktree_command_facts::AddPlan;
use crate::worktree_copy_stage_claim::{
    StageClaim,
    claim_stage_for_add,
};
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_ladder_run_rung_and_refusal::{
    Rung,
    abandon_the_read_beside_the_check,
    log_git_checkout,
};
use crate::worktree_slow_route::{
    SlowRoute,
    explain_cheap_refusal,
    explain_decline,
    explain_refusal,
    explain_unsupported,
};
use crate::worktree_snapshot_source::{
    SnapshotSource,
    clone_refusal,
    snapshot_blocker,
    snapshot_refusal,
    source_or_dirty,
};
use crate::worktree_subvolume::prepare_subvolume_destination;

/// Import the tag every line of this folder's ladder work carries.
use crate::worktree::worktree_ladder_finish_binding::TAG;

/// What:
///  Choose the rung for one invocation and prepare it:
/// a snapshot source and the claim on its stage,
/// a clone source,
/// or an
///  empty subvolume at the destination for Git to fill;
/// with the sentence that says why it is not the snapshot,
/// when it is not.
/// Why:
///   Every check that can be made before Git runs is made now,
/// cheapest first,
/// so a refused rung costs no extra Git run,
///  with the one exception rung 1's size read states
/// (`worktree_clone_conversion_size_read.rs`:
///  a read started before the check is ended,
/// not paid,
/// when the check refuses),
///  and each refusal leaves its reason in the log and,
/// in words,
/// with the person once Git has succeeded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chooseRung(plan: AddPlan, context: GitContext, identity: WorktreeIdentity): { rung: Rung; slow?: SlowRoute };
/// ```
pub(crate) fn choose_rung(
    plan: &AddPlan,
    context: &GitContext,
    identity: &WorktreeIdentity,
    held: &mut Option<StageClaim>,
    sizes_beside: &mut Option<crate::worktree_clone_conversion_size_read::SizeReadBesideTheCheck>,
) -> (Rung, Option<SlowRoute>) {
    let refusals_entered: bool = begin_step("refusals");
    let snapshot_refusal: Option<String> = snapshot_refusal(identity, plan);
    end_step(refusals_entered);
    let refusals_again_entered: bool = begin_step("refusals");
    let clone_refusal: Option<String> = clone_refusal(identity, plan);
    end_step(refusals_again_entered);
    // The cheap refusal comes first in the sentence: it is the structural one, which no commit or wait removes.
    let cheap: Option<SlowRoute> = match snapshot_refusal.as_deref() {
        Some(reason) => explain_cheap_refusal(identity, reason),
        None => None,
    };
    if snapshot_refusal.is_some() && clone_refusal.is_some() {
        log_git_checkout(
            plan,
            "neither a snapshot nor clones apply",
            &snapshot_refusal,
            &clone_refusal,
        );
        return (Rung::Git(prepare_subvolume_destination(plan)), cheap);
    }
    // What:
    //  Rung 1's size read starts here,
    //  before the source check the ladder runs next.
    // Why:
    //  The snapshot rung decides the whole fence on the add's waiting path after that check and after `git check-attr`,
    //  and the read needs only the worktree root,
    //  the commit `HEAD` names and the object format, which the ref files answer now.
    // The other rungs are untouched:
    //  a clone rung reads beside its copies,
    //  and an add that ends before the fence settles or ends this read at its own end.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (snapshotRefusal === undefined) sizesBesideTheCheck = startBesideTheCheck(context, identity);
    // ```
    if snapshot_refusal.is_none() {
        let start_entered: bool = begin_step("startBesideTheCheck");
        *sizes_beside =
            crate::worktree_clone_conversion_size_read::start_beside_the_check(context, identity);
        end_step(start_entered);
    }
    let check_source_entered: bool = begin_step("checkSource");
    let check_source_outcome = source_or_dirty(context, identity);
    end_step(check_source_entered);
    let source: SnapshotSource = match check_source_outcome {
        Ok(found) => found,
        Err(refusal) => {
            abandon_the_read_beside_the_check(sizes_beside);
            log_git_checkout(
                plan,
                refusal.reason.as_str(),
                &snapshot_refusal,
                &clone_refusal,
            );
            // The explanation is made only when no cheap one was found, as `or_else` did.
            let slow: Option<SlowRoute> = match cheap {
                Some(structural) => Some(structural),
                None => explain_refusal(context, identity, &refusal),
            };
            return (Rung::Git(prepare_subvolume_destination(plan)), slow);
        }
    };
    let snapshot_blocked: Option<String> = match snapshot_refusal.clone() {
        Some(reason) => Some(reason),
        None => snapshot_blocker(&source),
    };
    let mut slow: Option<SlowRoute> = match cheap {
        Some(structural) => Some(structural),
        None => snapshot_blocked.as_deref().map(explain_unsupported),
    };
    if snapshot_blocked.is_some() {
        abandon_the_read_beside_the_check(sizes_beside);
    }
    if snapshot_blocked.is_none() {
        // What:
        //  The snapshot rung's conversion fence is decided here,
        //  before the snapshot copies anything:
        //  its index is installed by the snapshot itself,
        //  so the paths Git must write have to be known before that install,
        //  or the installed index would vouch for a path whose bytes are not Git's.
        // Why:
        //  The fence's reads need the source and its frozen commit, both of which exist now,
        //  and a decline or a refusal here costs the add no snapshot at all;
        //  decided after the snapshot, as the last round's rung 1 did, a declined add paid the whole snapshot
        //  and then Git's checkout over it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const fence = fenceTheClone(source, context); // throws the reason the rung cannot be proven
        // ```
        let snapshot_fence_entered: bool = begin_step("conversionFence");
        let snapshot_fence_outcome: Result<Fence, String> = crate::worktree_clone_conversion_size_read::fence_the_clone_with_the_read_beside_the_check(&source, context, sizes_beside);
        end_step(snapshot_fence_entered);
        match snapshot_fence_outcome {
            Ok(fence) => {
                // What:
                //  A target tree that changes a path this fence hands to Git ends the rung here,
                //  before the stage name is claimed and before the snapshot copies anything.
                // Why:
                //  The index the snapshot installs holds no stat data for those paths, and `git read-tree -m -u` verifies
                //  the entries of the paths it changes, so such a tree difference ends the rung after the snapshot and pays
                //  Git's whole checkout over it (`doc/planning/cli-git-clone-rung-conversion-fence.md`, "Limitations").
                //  The fence's own answer is the set, so one name-only read decides it;
                //  a source with changes is restored to its `HEAD` rather than tree-differenced, so it is not this case,
                //  and a fence that hands Git no path at all pays no read.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const skip = source.dirty ? undefined : skipReasonForTheTargetTree(source, context, plan, fence.gitWrites.length > 0, (changed) => firstWrittenChange(fence, changed));
                // if (skip) return { rung: git, slow: explainDecline(skip) };
                // ```
                let skip_entered: bool = begin_step("targetTreeDifference");
                let skip: Option<String> = match source.dirty.is_none() {
                    true => {
                        crate::worktree_ladder_target_tree_skip::skip_reason_for_the_target_tree(
                            &source,
                            context,
                            plan,
                            !fence.git_writes.is_empty(),
                            &|changed: &[Vec<u8>]| {
                                return crate::worktree_clone_conversion_fence::first_written_change(&fence, changed);
                            },
                        )
                    }
                    false => None,
                };
                end_step(skip_entered);
                if let Some(reason) = skip {
                    abandon_the_read_beside_the_check(sizes_beside);
                    log_git_checkout(plan, reason.as_str(), &snapshot_refusal, &clone_refusal);
                    return (
                        Rung::Git(prepare_subvolume_destination(plan)),
                        Some(explain_decline(reason.as_str())),
                    );
                }
                // A claim the caller recorded before Git names this add's stage already, so the rung reuses it.
                let claim_stage_entered: bool = begin_step("claimStage");
                let claim_stage_outcome = match held.take() {
                    Some(recorded) => Ok(recorded),
                    None => {
                        claim_stage_for_add(source.common_dir.as_path(), plan.destination.as_path())
                    }
                };
                end_step(claim_stage_entered);
                match claim_stage_outcome {
                    Ok(claim) => return (Rung::Snapshot(Box::new(source), claim, fence), None),
                    Err(error) => {
                        debug(
                            TAG,
                            format!("the stage cannot be claimed ({}); no snapshot", error.0)
                                .as_str(),
                        );
                        slow = Some(explain_unsupported(
                            format!("the add could not be recorded durably ({})", error.0).as_str(),
                        ));
                    }
                }
            }
            Err(reason) => {
                abandon_the_read_beside_the_check(sizes_beside);
                log_git_checkout(plan, reason.as_str(), &snapshot_refusal, &clone_refusal);
                return (
                    Rung::Git(prepare_subvolume_destination(plan)),
                    Some(explain_decline(reason.as_str())),
                );
            }
        }
    } else {
        debug(
            TAG,
            format!(
                "no snapshot for {} ({:?})",
                plan.destination.display(),
                snapshot_blocked
            )
            .as_str(),
        );
    }
    // Clones copy the tracked files of a source the checks proved: every path of a clean source, and of a source with
    //  uncommitted changes every path the proof still settles, with Git writing the rest (the partial clone rung).
    if clone_refusal.is_none() {
        // What:
        //  The fence's attributes are read here,
        //  before the rung's limits are asked and before its claim on a stage name and its first copy:
        //  the cheap refusal for an untracked or ignored `.gitattributes`,
        //  the `git check-attr` read,
        //  and the decline their count prices.
        // Why:
        //  The whole-tree limit sums the bytes the clones would carry,
        //  which is the attributes' answer,
        //  so the limit cannot be asked before this read
        //  (`doc/decision/cli-git-clone-rung-four-answers-after-the-alpha-45-release-pass.md`,
        //  "What the whole-tree limit sums (D3)").
        //  A source the fence refuses or declines must not pay the claim, Git's `--no-checkout` run and a copy first:
        //  the decline needs the attributes alone,
        //  so deciding it here makes a declined add one Git checkout plus that one query.
        //  What the read costs a size-declined add, and what it cost on this repository,
        //  is stated beside the limit (`worktree_clone_rung_limits`, `bytes_the_clones_carry`) and in the round's report.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const attributes = fenceTheAttributes(source, context); // throws the reason the rung cannot be proven
        // ```
        let conversion_fence_entered: bool = begin_step("conversionFence");
        let attributes_outcome: Result<AttributeFence, String> =
            crate::worktree_clone_conversion_fence::fence_the_attributes(&source, context);
        end_step(conversion_fence_entered);
        let attributes: AttributeFence = match attributes_outcome {
            Ok(read) => read,
            Err(reason) => {
                abandon_the_read_beside_the_check(sizes_beside);
                log_git_checkout(plan, reason.as_str(), &snapshot_refusal, &clone_refusal);
                return (
                    Rung::Git(prepare_subvolume_destination(plan)),
                    Some(explain_decline(reason.as_str())),
                );
            }
        };
        // What:
        //  The rung's provisional limits are asked here,
        //  before its claim on a stage name and its first copy:
        //  a source with changes whose clones would copy too few bytes for a clone to win on its filesystem,
        //  a source with no tracked change whose mean tracked file is too small for one,
        //  and a source whose changed paths exceed the cut.
        // Why:
        //  They decide between this rung and Git's own checkout,
        //  so asking them later makes a declined add pay the claim and the copies it was declined for;
        //  each limit's value, its measurement and why it sits at that edge are beside it
        //  (`worktree_clone_rung_limits`), and each sends the add to Git's own checkout, which is the baseline it was
        //  measured against,
        //  only on a filesystem whose measurement gives it a value.
        //  The index is parsed once here and shared with the rung's own copies,
        //  so the limits cost no parse of their own and the rung no second one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const entries = trackedEntries(source);
        // const limit = entries && declineBeforeCloning(source, entries, attributes.bytesTheClonesCarry());
        // if (limit) return { rung: git, slow: explainDecline(limit) };
        // ```
        let entries_entered: bool = begin_step("trackedEntries");
        let entries: Option<Vec<crate::worktree_clean::StatEntry>> =
            crate::worktree_clone_rung_limits::tracked_entries(&source);
        end_step(entries_entered);
        let limit: Option<String> = entries.as_deref().and_then(|read| {
            return crate::worktree_clone_rung_limits::decline_before_cloning(
                &source,
                read,
                attributes.bytes_the_clones_carry(),
            );
        });
        if let Some(reason) = limit {
            log_git_checkout(plan, reason.as_str(), &snapshot_refusal, &clone_refusal);
            return (
                Rung::Git(prepare_subvolume_destination(plan)),
                Some(explain_decline(reason.as_str())),
            );
        }
        // What:
        //  The fence's remaining shape is chosen here,
        //  before the add's record is claimed and before real Git runs:
        //  the rest beside the copies unless this build's routes decide it here too.
        // Why:
        //  A source the fence refuses must not pay the claim, Git's `--no-checkout` run and a copy first.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const fence = theFenceOfThisBuild(source, context, attributes); // throws the reason the rung cannot be proven
        // ```
        let fence_outcome: Result<CloneFence, String> =
            crate::worktree_clone_conversion_beside_the_clone::the_fence_of_this_build(
                &source, context, attributes,
            );
        let fence: CloneFence = match fence_outcome {
            Ok(chosen) => chosen,
            Err(reason) => {
                abandon_the_read_beside_the_check(sizes_beside);
                log_git_checkout(plan, reason.as_str(), &snapshot_refusal, &clone_refusal);
                return (
                    Rung::Git(prepare_subvolume_destination(plan)),
                    Some(explain_decline(reason.as_str())),
                );
            }
        };
        // What:
        //  A target tree that changes a path this fence cannot serve ends the rung here,
        //  before the add's record is claimed and before the first copy.
        // Why:
        //  The paths the fence installs without stat data are exactly what `git read-tree -m -u` refuses when the target
        //  tree changes one of them, and the fallback then pays the copies and Git's whole checkout over them
        //  (`doc/planning/cli-git-clone-rung-conversion-fence.md`, "Limitations").
        //  A link the copies make again is the same refusal: its index entry cannot describe the destination's link.
        //  The attributes half already names every path a rule of its own converts, which is what this rung can answer
        //  before its copies; a path the size proof would still hand to Git is left to the rung's own ending.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const couldMatter = fence.kind === 'beside' ? fence.read.aChangedPathCouldMatter() : fence.decided.gitWrites.length > 0;
        // const skip = skipReasonForTheTargetTree(source, context, plan, couldMatter, (changed) => firstChangeItCannotServe(fence, changed));
        // if (skip) return { rung: git, slow: explainDecline(skip) };
        // ```
        let a_changed_path_could_matter: bool = match &fence {
            CloneFence::Beside(read) => read.a_changed_path_could_matter(),
            #[cfg(feature = "measurement-routes")]
            CloneFence::BeforeTheClone(decided) => !decided.git_writes.is_empty(),
        };
        let skip_entered: bool = begin_step("targetTreeDifference");
        let skip: Option<String> =
            crate::worktree_ladder_target_tree_skip::skip_reason_for_the_target_tree(
                &source,
                context,
                plan,
                a_changed_path_could_matter,
                &|changed: &[Vec<u8>]| {
                    return match &fence {
                        CloneFence::Beside(read) => read.first_change_it_cannot_serve(changed),
                        #[cfg(feature = "measurement-routes")]
                        CloneFence::BeforeTheClone(decided) => {
                            crate::worktree_clone_conversion_fence::first_written_change(
                                decided, changed,
                            )
                        }
                    };
                },
            );
        end_step(skip_entered);
        if let Some(reason) = skip {
            abandon_the_read_beside_the_check(sizes_beside);
            log_git_checkout(plan, reason.as_str(), &snapshot_refusal, &clone_refusal);
            return (
                Rung::Git(prepare_subvolume_destination(plan)),
                Some(explain_decline(reason.as_str())),
            );
        }
        // The record comes first: without it a killed add could not be removed, so the rung is not tried.
        let claim_stage_again_entered: bool = begin_step("claimStage");
        let claim_stage_again_outcome = match held.take() {
            Some(recorded) => Ok(recorded),
            None => claim_stage_for_add(source.common_dir.as_path(), plan.destination.as_path()),
        };
        end_step(claim_stage_again_entered);
        match claim_stage_again_outcome {
            Ok(claim) => {
                return (
                    Rung::Clone(
                        Box::new(source),
                        prepare_subvolume_destination(plan),
                        claim,
                        fence,
                        entries,
                    ),
                    slow,
                );
            }
            Err(error) => debug(
                TAG,
                format!("the add cannot be recorded ({}); no clones", error.0).as_str(),
            ),
        }
    }
    abandon_the_read_beside_the_check(sizes_beside);
    log_git_checkout(plan, "no rung applies", &snapshot_refusal, &clone_refusal);
    return (Rung::Git(prepare_subvolume_destination(plan)), slow);
}
