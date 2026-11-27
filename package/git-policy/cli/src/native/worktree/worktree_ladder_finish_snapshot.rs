//! What:
//!  The snapshot rung after real Git returned:
//! everything before the swap
//! (the target,
//! the snapshot,
//! the dirty restore or the tree difference,
//! and the conversion fence's write through Git),
//! the swap itself,
//! and the fall back to Git's own checkout when a step cannot finish.
//! Why:
//!  Every step before the swap leaves the destination exactly as Git's `--no-checkout` run made it,
//! so a failure there completes the worktree the way Git would have.
//! An error that leaves the stage name holding this process's snapshot destroys the stage,
//! checking its identity first;
//! an error that leaves somebody else's directory there destroys nothing and ends the add as failed.
//! The trash a snapshot set aside is recorded before the rung returns,
//! so a command killed after the swap leaves a trash a service will empty.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const served = serveSnapshot(args); // throws an Ended
//! const result = finishSnapshot(args); // falls back to Git's checkout
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
};
use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::fs_beneath::ObjectId;
use crate::the_names_and_the_manifest_of_the_trash::PendingTrash;
use crate::transaction_git::GitContext;
use crate::worktree_add_binding::{
    Bound,
    verify_association,
};
use crate::worktree_clean_content::ConversionEnvironment;
use crate::worktree_clone_conversion_fence::Fence;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_copy::{
    GitRunner,
    GitStatus,
};
use crate::worktree_copy_stage_claim::StageClaim;
use crate::worktree_dirty_restore::{
    changes_left_out,
    left_out_note,
    record_left_out,
    restore_dirty,
};
use crate::worktree_ladder_finish_binding::bind;
use crate::worktree_ladder_finish_git_checkout::{
    complete_like_git,
    report_incomplete,
};
use crate::worktree_ladder_finish_trash_record::record_before_the_rung_returns;
use crate::worktree_ladder_run::{
    LadderResult,
    discard_stage,
};
use crate::worktree_ladder_target::{
    Target,
    apply_difference,
    prepare_target,
};
use crate::worktree_slow_route::{
    SlowRoute,
    explain_failure,
};
use crate::worktree_snapshot::{
    CopyState,
    PlaceError,
    put_in_place,
    settle_displaced,
    take_snapshot,
};
use crate::worktree_snapshot_source::SnapshotSource;
use crate::worktree_trash_manifest::TrashManifest;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// Import the tag every line of this folder's ladder work carries.
use crate::worktree::worktree_ladder_finish_binding::TAG;

/// Why a rung ended before it finished,
/// and whether the object at the stage name is still the snapshot this process made.
pub(crate) struct Ended {
    /// The reason,
    /// in words.
    pub(crate) reason: String,
    /// Whether the stage name holds this process's snapshot,
    /// so that destroying it destroys nothing else.
    pub(crate) stage_is_ours: bool,
}

impl From<String> for Ended {
    fn from(reason: String) -> Ended {
        return Ended {
            reason,
            stage_is_ours: true,
        };
    }
}

impl From<PlaceError> for Ended {
    fn from(error: PlaceError) -> Ended {
        return Ended {
            reason: error.reason,
            stage_is_ours: error.stage_is_ours,
        };
    }
}

/// What:
///  What a served snapshot leaves for the caller:
/// the line Git's own checkout would have printed,
/// and the trash the
///  leave-out made inside the worktree.
/// Why:
///   The line is printed once the worktree is whole,
/// and the trash is handed over and emptied later,
/// beside the package repair,
/// so
///  the two travel out of this module instead of being settled here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Served = { line: string; placed: ObjectId; trash?: PendingTrash };
/// ```
pub(crate) struct Served {
    /// The line Git's checkout prints.
    line: String,
    /// The object the destination names now:
    /// the snapshot that was swapped into it.
    placed: ObjectId,
    /// The trash the snapshot left,
    /// when it left one.
    trash: Option<PendingTrash>,
}

/// What:
///  Everything before the swap and the swap itself;
/// the error is the reason the rung could not finish,
/// and the success is Git's line
///  with the trash the snapshot left.
/// Why:
///   The caller destroys the stage on an error that left the stage name holding the snapshot,
/// and completes the worktree
///  with Git's own checkout.
/// `made` receives the identity of the stage as soon as it exists.
/// The trash is handed over after the swap,
///  and the service empties it after the command returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serveSnapshot(args): Promise<{ line: string; placed: ObjectId; trash?: PendingTrash }>;
/// ```
pub(crate) fn serve_snapshot(
    source: &SnapshotSource,
    plan: &AddPlan,
    stage: &Path,
    bound: &Bound,
    made: &mut Option<ObjectId>,
    fence: &Fence,
    run_git: &mut dyn GitRunner,
    context: &GitContext,
) -> Result<Served, Ended> {
    verify_association(bound)?;
    let prepare_target_entered: bool = begin_step("prepareTarget");
    let prepare_target_outcome = prepare_target(source, plan, context, bound);
    end_step(prepare_target_entered);
    let target: Target = prepare_target_outcome?;
    let environment: ConversionEnvironment =
        crate::worktree_clean_content_attribute_sources::for_context(context);
    // The written paths are the fence's, so the snapshot's index names them without stat data and the checkout below
    // writes them: no step between that install and that write vouches for the source's bytes there.
    let written: std::collections::HashSet<Vec<u8>> = fence.git_writes.iter().cloned().collect();
    let take_snapshot_entered: bool = begin_step("takeSnapshot");
    let take_snapshot_outcome = take_snapshot(
        source,
        plan,
        stage,
        &bound.admin,
        target.pointer.as_slice(),
        &environment,
        &written,
    );
    end_step(take_snapshot_entered);
    let (state, trash, taken_over): (CopyState, Option<TrashManifest>, usize) =
        match take_snapshot_outcome {
            Ok(taken) => {
                *made = Some(taken.stage);
                (taken.state, taken.trash, taken.taken_over)
            }
            Err(failure) => {
                *made = failure.stage;
                return Err(Ended::from(failure.reason));
            }
        };
    verify_association(bound)?;
    // A snapshot of a source with changes carries them, and one the source changed under carries the wrong bytes: both are
    //  reset to the new worktree's HEAD from the index the snapshot carries, which is one instant with its files.
    if source.dirty.is_some() || state == CopyState::ChangedUnderCopy {
        let restore_dirty_entered: bool = begin_step("restoreDirty");
        let restore_dirty_outcome =
            restore_dirty(source, &target, stage, run_git, context, &environment);
        end_step(restore_dirty_entered);
        restore_dirty_outcome?;
    } else {
        let apply_difference_entered: bool = begin_step("applyDifference");
        let apply_difference_outcome = apply_difference(source, &target, stage, run_git);
        end_step(apply_difference_entered);
        apply_difference_outcome?;
    }
    // What:
    //  The snapshot rung's conversion fence: the paths Git's checkout would write other bytes for are written by Git,
    //  in the stage, before it is swapped in.
    // Why:
    //  A snapshot carries the source's whole working tree, so where a line-ending rule, `core.autocrlf`, `ident`, an
    //  encoding or a filter applies, the stage holds the source's bytes and not the conversion of the blob, which neither
    //  the stat data nor `git status` can tell (issue 6323; the guest measured rung 1 wrong for a clean and a dirty
    //  source). The fence is the clone rung's own decision, decided before the snapshot (rule: no path recorded clean
    //  before its bytes are Git's), and the write is the clone rung's own checkout run, with the stage as its work tree.
    //  The index the snapshot installed holds no stat data for these paths (`install_the_index`, the seed
    //  with_zeroed_stat made), so no step between the install and this run vouches for the source's bytes there; `-u`
    //  records each written path's own stat data.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fence = fenceTheClone(source, context); // decided before the snapshot (chooseRung)
    // if (fence.gitWrites.length > 0) checkoutIndexThroughGit(stage, bound.admin, context, fence.gitWrites);
    // ```
    if !fence.git_writes.is_empty() {
        let write_entered: bool = begin_step("writeConversions");
        let write_outcome = crate::worktree_clone_conversion_fence::checkout_index_through_git(
            target.admin.as_path(),
            stage,
            context,
            fence.git_writes.as_slice(),
            source.clean.hash_length,
        );
        end_step(write_entered);
        write_outcome?;
    }
    let root_metadata = match std::fs::metadata(source.root.as_path()) {
        Ok(found) => found,
        Err(error) => {
            return Err(Ended::from(format!(
                "the source's root is unreadable ({error})"
            )));
        }
    };
    let mode: u32 = root_metadata.mode() & 0o7777;
    // The wrapper's last write into the stage is done, so the record of the object it made now names the instant after
    // which nothing of the wrapper's is in that folder: everything born later is somebody else's, and keeps it.
    // What a snapshot of the main checkout set aside is handed to the commit service once the worktree is in place, and the add
    //  does not wait for it. Nothing is set aside for a linked worktree.
    let mut pending: Option<PendingTrash> = None;
    if let Some(manifest) = trash {
        pending = Some(PendingTrash {
            destination: plan.destination.clone(),
            common_dir: source.common_dir.clone(),
            manifest,
            taken_over,
            quiet: plan.quiet,
        });
    }
    crate::worktree_snapshot::record_made_of_the_stage(source, stage, *made);
    let exchange_entered: bool = begin_step("exchange");
    let exchange_outcome = put_in_place(
        stage,
        plan.destination.as_path(),
        mode,
        bound.binding.destination_id,
    );
    end_step(exchange_entered);
    let placed: ObjectId = exchange_outcome?;
    // The trash stands in the published worktree from here, so its record is written now, before this rung returns: a command
    //  killed after the swap leaves a trash a service will empty, where a manifest that lived only in memory left one nothing
    //  would ever name.
    let handed_later: Option<PendingTrash> = pending.and_then(record_before_the_rung_returns);
    let settle_displaced_entered: bool = begin_step("settleDisplaced");
    let settle_displaced_outcome = settle_displaced(
        stage,
        plan.destination.as_path(),
        bound.binding.destination_id,
        target.pointer.as_slice(),
    );
    end_step(settle_displaced_entered);
    for note in settle_displaced_outcome {
        warn(
            TAG,
            format!("monochromatic-git-policy-cli: {note}").as_str(),
        );
    }
    return Ok(Served {
        line: target.line,
        placed,
        trash: handed_later,
    });
}

/// What:
///  Serve the snapshot rung after real Git returned,
/// or complete the worktree with Git's own checkout.
/// Why:
///   An error that leaves the stage name holding the snapshot destroys the stage (checking its identity) and falls to Git's
///  checkout;
/// an error that leaves somebody else's directory there destroys nothing and ends the add as failed,
/// with the
///  names in the message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finishSnapshot(args): Promise<LadderResult>;
/// ```
pub(crate) fn finish_snapshot(
    source: &SnapshotSource,
    plan: &AddPlan,
    stage: &Path,
    claim: StageClaim,
    fence: &Fence,
    run_git: &mut dyn GitRunner,
    context: &GitContext,
    out: &mut dyn Write,
    note: &mut dyn Write,
    slow: &mut Option<SlowRoute>,
) -> LadderResult {
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
    let mut made: Option<ObjectId> = None;
    return match serve_snapshot(
        source, plan, stage, &bound, &mut made, fence, run_git, context,
    ) {
        Ok(served) => {
            debug(
                TAG,
                format!("the snapshot served {}", plan.destination.display()).as_str(),
            );
            // A closed standard output leaves nowhere to print Git's line; the exit status still carries the result.
            let _written = out.write_all(served.line.as_bytes());
            // A snapshot of the main checkout prints no copy summary, so it says what it did, on the stream the command was given for its notes; `--quiet` stays silent.
            if source.git_dir == source.common_dir && !plan.quiet {
                let _noted = note.write_all(crate::worktree_trash::MAIN_SNAPSHOT_NOTE.as_bytes());
                if let Some(changed) = changes_left_out(source) {
                    let _left_out = note.write_all(left_out_note(changed).as_bytes());
                }
            }
            if let Some(changed) = changes_left_out(source) {
                record_left_out(plan.destination.as_path(), changed);
            }
            LadderResult {
                git: GitStatus {
                    code: Some(0),
                },
                carried: Some(plan.destination.clone()),
                placed: Some(served.placed),
                trash: served.trash,
                claim: Some(claim),
            }
        }
        Err(ended) if ended.stage_is_ours => {
            debug(
                TAG,
                format!(
                    "the snapshot rung ended: {}; Git checks the worktree out",
                    ended.reason
                )
                .as_str(),
            );
            // The record said a snapshot was chosen; Git's checkout is what publishes this worktree.
            crate::add_timing::note("route", "Git's checkout after the snapshot rung ended");
            discard_stage(stage, made);
            *slow = Some(explain_failure(ended.reason.as_str()));
            LadderResult {
                git: complete_like_git(plan, &bound, run_git),
                carried: None,
                placed: None,
                trash: None,
                claim: Some(claim),
            }
        }
        Err(ended) => {
            warn(TAG, format!("monochromatic-git-policy-cli: the add of {} ended in a state the wrapper cannot untangle: {}; {} is not touched", plan.destination.display(), ended.reason, stage.display()).as_str());
            report_incomplete(plan);
            LadderResult {
                git: GitStatus {
                    code: Some(1),
                },
                carried: None,
                placed: None,
                trash: None,
                claim: Some(claim),
            }
        }
    };
}
