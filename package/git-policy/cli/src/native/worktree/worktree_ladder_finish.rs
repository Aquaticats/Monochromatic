//! What:
//!  What the ladder does after real Git returned:
//! bind the add to the directories Git made,
//! the snapshot rung's steps (read
//!  the new `HEAD`,
//! snapshot,
//! apply the tree difference,
//! swap into place,
//! print Git's checkout line),
//! the clone rung's steps,
//!  and the fall back to Git's own `reset --hard` in the bound worktree when a step cannot finish.
//! Why:
//!  Every step before the swap leaves the destination exactly as Git's `--no-checkout` run made it,
//! so a failure there
//!  completes the worktree the way Git would have.
//! The ladder never deletes or unregisters a worktree:
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

use crate::add_timing::{begin_step, end_step};
use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::ObjectId;
use crate::transaction_git::GitContext;
use crate::worktree_add_binding::{Bound, bind_registration, verify_association};
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone::{Cloned, clone_tracked_files, leftover_report};
use crate::worktree_clean_content::ConversionEnvironment;
use crate::worktree_copy::{GitRunner, GitStatus};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_stage_claim::StageClaim;
use crate::worktree_ladder_run::{Ladder, LadderResult, Rung, discard_stage};
use crate::worktree_dirty_restore::{left_out_note, record_left_out, restore_dirty};
use crate::worktree_ladder_target::{Target, apply_difference, checkout_environment, prepare_target, run_step};
use crate::worktree_slow_route::{SlowRoute, explain_failure};
use crate::worktree_snapshot::{CopyState, PlaceError, put_in_place, settle_displaced, take_snapshot};
use crate::worktree_snapshot_source::{ChangedPaths, SnapshotSource};
use crate::worktree_subvolume::settle_created_destination;
use crate::worktree_trash::{PendingTrash, Recorded, TrashManifest, delete_trash, record_the_trash};
use std::ffi::OsString;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeLadder";

/// Why a rung ended before it finished,
/// and whether the object at the stage name is still the snapshot this process made.
struct Ended {
    /// The reason,
    /// in words.
    reason: String,
    /// Whether the stage name holds this process's snapshot,
    /// so that destroying it destroys nothing else.
    stage_is_ours: bool,
}

impl From<String> for Ended {
    fn from(reason: String) -> Ended {
        return Ended { reason, stage_is_ours: true };
    }
}

impl From<PlaceError> for Ended {
    fn from(error: PlaceError) -> Ended {
        return Ended { reason: error.reason, stage_is_ours: error.stage_is_ours };
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
struct Served {
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
#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "the two seams of the snapshot ahead are kept past this call with the context and the source a named function would have to be handed through signatures that do not carry them"))]
fn serve_snapshot(source: &SnapshotSource, plan: &AddPlan, stage: &Path, bound: &Bound, made: &mut Option<ObjectId>, run_git: &mut dyn GitRunner, context: &GitContext) -> Result<Served, Ended> {
    verify_association(bound)?;
    let prepare_target_entered: bool = begin_step("prepareTarget");
    let prepare_target_outcome = prepare_target(source, plan, context, bound);
    end_step(prepare_target_entered);
    let target: Target = prepare_target_outcome?;
    let environment: ConversionEnvironment = crate::worktree_clean_content_attribute_sources::for_context(context);
    // The service that makes the snapshots ahead is this repository's commit service, and the add asks it for one only when
    //  it can reach it: a control hands in a service that reaches nothing, and then the add takes its own snapshot.
    // The rung keeps both seams for as long as it runs, with the context and the source this call cannot pass on.
    let ask = || { return crate::worktree_snapshot_ahead::ask_for_one(context, source); };
    let hand_over = |stale: &[std::ffi::OsString]| { return crate::worktree_snapshot_ahead::hand_over_to_the_service(context, source, stale); };
    let service: crate::worktree_snapshot_ahead::AheadService<'_> =
        crate::worktree_snapshot_ahead::AheadService { ask: &ask, hand_over: &hand_over, ceiling: crate::wait_ceilings::SNAPSHOT_AHEAD_CEILING };
    let take_snapshot_entered: bool = begin_step("takeSnapshot");
    let take_snapshot_outcome = take_snapshot(source, plan, stage, &bound.admin, target.pointer.as_slice(), &environment, &service);
    end_step(take_snapshot_entered);
    let (state, trash, taken_over): (CopyState, Option<TrashManifest>, usize) = match take_snapshot_outcome {
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
        let restore_dirty_outcome = restore_dirty(source, &target, stage, run_git, context);
        end_step(restore_dirty_entered);
        restore_dirty_outcome?;
    } else {
        let apply_difference_entered: bool = begin_step("applyDifference");
        let apply_difference_outcome = apply_difference(source, &target, stage, run_git);
        end_step(apply_difference_entered);
        apply_difference_outcome?;
    }
    let root_metadata = match std::fs::metadata(source.root.as_path()) {
        Ok(found) => found,
        Err(error) => return Err(Ended::from(format!("the source's root is unreadable ({error})"))),
    };
    let mode: u32 = root_metadata.mode() & 0o7777;
    // The wrapper's last write into the stage is done, so the record of the object it made now names the instant after
    // which nothing of the wrapper's is in that folder: everything born later is somebody else's, and keeps it.
    // What a snapshot of the main checkout set aside is handed to the commit service once the worktree is in place, and the add
    //  does not wait for it. Nothing is set aside for a linked worktree.
    let mut pending: Option<PendingTrash> = None;
    if let Some(manifest) = trash {
        pending = Some(PendingTrash { destination: plan.destination.clone(), common_dir: source.common_dir.clone(), manifest, taken_over, quiet: plan.quiet });
    }
    crate::worktree_snapshot::record_made_of_the_stage(source, stage, *made);
    let exchange_entered: bool = begin_step("exchange");
    let exchange_outcome = put_in_place(stage, plan.destination.as_path(), mode, bound.binding.destination_id);
    end_step(exchange_entered);
    let placed: ObjectId = exchange_outcome?;
    // The trash stands in the published worktree from here, so its record is written now, before this rung returns: a command
    //  killed after the swap leaves a trash a service will empty, where a manifest that lived only in memory left one nothing
    //  would ever name.
    let handed_later: Option<PendingTrash> = pending.and_then(record_before_the_rung_returns);
    let settle_displaced_entered: bool = begin_step("settleDisplaced");
    let settle_displaced_outcome = settle_displaced(stage, plan.destination.as_path(), bound.binding.destination_id, target.pointer.as_slice());
    end_step(settle_displaced_entered);
    for note in settle_displaced_outcome {
        warn(TAG, format!("monochromatic-git-policy-cli: {note}").as_str());
    }
    return Ok(Served { line: target.line, placed, trash: handed_later });
}

/// What:
///  Write the trash's record now that the worktree holding it is published,
/// and answer what the hand-over still owes,
///  which is the same trash when the record is written.
/// Why:
///   The record is what outlives the command,
/// so it is written before the rung returns,
/// and a command killed after
///  the swap leaves a trash the service empties.
/// A record that cannot be written is a trash nothing will ever empty,
/// and the add
///  pays the emptying itself in that case,
/// which no ordinary add reaches;
/// a trash that then stays is reported with its path
///  and the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordBeforeTheRungReturns = (pending) => { try { recordTheTrash(pending); return pending; } catch (reason) { warn(reason); deleteTrash(pending); return undefined; } };
/// ```
fn record_before_the_rung_returns(pending: PendingTrash) -> Option<PendingTrash> {
    let shown: PathBuf = pending.destination.join(crate::worktree_trash::TRASH_PARENT).join(&pending.manifest.name);
    // A record that is already there is this trash's record, because the trash's name is its id; only a record the service
    //  gave up on leaves nothing to empty the trash, and then this add empties it itself as for a record it could not write.
    let refused: Option<String> = match record_the_trash(&pending) {
        Ok(Recorded::Written | Recorded::There) => None,
        Ok(Recorded::GivenUp) => Some(String::from("the commit service gave up on that record and never claims it again")),
        Err(reason) => Some(format!("the work item could not be written: {reason}")),
    };
    if let Some(reason) = refused {
        warn(TAG, format!("monochromatic-git-policy-cli: no record the service still works on holds the trash {} ({reason}), so this add empties it before it returns", shown.display()).as_str());
        if let Err(left) = delete_trash(pending.destination.as_path(), pending.common_dir.as_path(), &pending.manifest) {
            warn(TAG, format!("monochromatic-git-policy-cli: the trash {} stays: {}", shown.display(), left.reason).as_str());
        }
        return None;
    }
    return Some(pending);
}

/// What:
///  The paths the source has changed since its last commit,
/// when there are any.
/// Why:
///   A snapshot carries the source's uncommitted state,
/// so the person is told which of it was left out of the new worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// source.dirty?.hasChanges() ? source.dirty : undefined
/// ```
fn changes_left_out(source: &SnapshotSource) -> Option<&ChangedPaths> {
    return match source.dirty.as_ref() {
        Some(found) if found.has_changes() => Some(found),
        _ => None,
    };
}

/// What:
///  The clone rung:
/// clone the tracked files into the destination,
/// install the index,
/// apply the tree difference.
/// Why:
///   The destination is filled in place;
/// an error removes nothing:
/// the entries the run created stay,
/// are reported,
/// and Git's
///  own checkout completes the worktree and overwrites the tracked ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serveClones(args): Promise<string>; // the checkout line; throws a reason
/// ```
fn serve_clones(source: &SnapshotSource, plan: &AddPlan, bound: &Bound, run_git: &mut dyn GitRunner, context: &GitContext) -> Result<String, String> {
    verify_association(bound)?;
    let target: Target = prepare_target(source, plan, context, bound)?;
    let cloned: Cloned = clone_tracked_files(source, &bound.destination, &bound.admin)?;
    let applied: Result<(), String> = match verify_association(bound) {
        Ok(()) => apply_difference(source, &target, plan.destination.as_path(), run_git),
        Err(reason) => Err(reason),
    };
    if let Err(reason) = applied {
        warn(TAG, format!("monochromatic-git-policy-cli: {}", leftover_report(cloned.created(), plan.destination.as_path())).as_str());
        return Err(reason);
    }
    return Ok(target.line);
}

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
fn checkout_like_git(plan: &AddPlan, bound: &Bound, run_git: &mut dyn GitRunner) -> GitStatus {
    let destination: &Path = plan.destination.as_path();
    if let Err(reason) = verify_association(bound) {
        warn(TAG, format!("monochromatic-git-policy-cli: the checkout of {} was not started: {reason}", destination.display()).as_str());
        return GitStatus { code: Some(1) };
    }
    let mut arguments: Vec<OsString> = ["-C"].iter().map(OsString::from).chain([destination.as_os_str().to_os_string()]).chain(["reset", "--hard", "--no-recurse-submodules"].iter().map(OsString::from)).collect();
    if plan.quiet {
        arguments.push(OsString::from("--quiet"));
    }
    let environment: Vec<(OsString, OsString)> = checkout_environment(bound.binding.admin.as_path(), destination);
    let status: GitStatus = match run_step(run_git, arguments.as_slice(), environment.as_slice()) {
        Ok(status) => status,
        Err(reason) => {
            warn(TAG, format!("Git's checkout of {} could not start ({reason})", destination.display()).as_str());
            return GitStatus { code: Some(1) };
        }
    };
    if let (true, Err(reason)) = (status.succeeded(), verify_association(bound)) {
        warn(TAG, format!("monochromatic-git-policy-cli: the checkout of {} ran, but it cannot be taken for the add's: {reason}", destination.display()).as_str());
        return GitStatus { code: Some(1) };
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
fn report_incomplete(plan: &AddPlan) {
    let shown: String = plan.destination.display().to_string();
    let finish: String = crate::worktree_finish_commands::finish_or_remove_sentence(plan.destination.as_path());
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
fn complete_like_git(plan: &AddPlan, bound: &Bound, run_git: &mut dyn GitRunner) -> GitStatus {
    let status: GitStatus = checkout_like_git(plan, bound, run_git);
    if !status.succeeded() {
        report_incomplete(plan);
    }
    return status;
}

/// What:
///  Bind the add to the directories Git made,
/// and write that binding to the add's record.
/// Why:
///   Every write below goes to the bound worktree and is checked against the binding.
/// A binding that cannot be proven ends
///  the ladder for this add,
/// leaving what Git made as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = await bind(claim, source, plan); // throws a reason
/// ```
fn bind(claim: &StageClaim, source: &SnapshotSource, plan: &AddPlan) -> Result<Bound, String> {
    let bind_registration_entered: bool = begin_step("bindRegistration");
    let bind_registration_outcome = bind_registration(source.common_dir.as_path(), plan.destination.as_path());
    end_step(bind_registration_entered);
    let bound: Bound = bind_registration_outcome?;
    let record_binding_entered: bool = begin_step("recordBinding");
    let record_binding_outcome = claim.record_binding(&bound.binding);
    end_step(record_binding_entered);
    if let Err(error) = record_binding_outcome {
        warn(TAG, format!("the binding of the add could not be recorded ({}); an interruption will be reported by path only", error.0).as_str());
    }
    return Ok(bound);
}

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
fn finish_clones(source: &SnapshotSource, plan: &AddPlan, git: GitStatus, claim: StageClaim, run_git: &mut dyn GitRunner, context: &GitContext, out: &mut dyn Write, slow: &mut Option<SlowRoute>) -> LadderResult {
    if !git.succeeded() {
        return LadderResult { git, carried: None, placed: None, trash: None, claim: Some(claim) };
    }
    let bound: Bound = match bind(&claim, source, plan) {
        Ok(found) => found,
        Err(reason) => {
            warn(TAG, format!("monochromatic-git-policy-cli: {} was left as Git made it, without a checkout: {reason}", plan.destination.display()).as_str());
            report_incomplete(plan);
            return LadderResult { git: GitStatus { code: Some(1) }, carried: None, placed: None, trash: None, claim: Some(claim) };
        }
    };
    return match serve_clones(source, plan, &bound, run_git, context) {
        Ok(line) => {
            debug(TAG, format!("clones served {}", plan.destination.display()).as_str());
            // A closed standard output leaves nowhere to print Git's line; the exit status still carries the result.
            let _written = out.write_all(line.as_bytes());
            LadderResult { git, carried: None, placed: None, trash: None, claim: Some(claim) }
        }
        Err(reason) => {
            debug(TAG, format!("the clone rung ended: {reason}; Git checks the worktree out").as_str());
            // The clones failed after the cause that already sent the add this way, if any: the person hears the later one.
            *slow = Some(explain_failure(reason.as_str()));
            LadderResult { git: complete_like_git(plan, &bound, run_git), carried: None, placed: None, trash: None, claim: Some(claim) }
        }
    };
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
fn finish_snapshot(source: &SnapshotSource, plan: &AddPlan, stage: &Path, claim: StageClaim, run_git: &mut dyn GitRunner, context: &GitContext, out: &mut dyn Write, note: &mut dyn Write, slow: &mut Option<SlowRoute>) -> LadderResult {
    let bound: Bound = match bind(&claim, source, plan) {
        Ok(found) => found,
        Err(reason) => {
            warn(TAG, format!("monochromatic-git-policy-cli: {} was left as Git made it, without a checkout: {reason}", plan.destination.display()).as_str());
            report_incomplete(plan);
            return LadderResult { git: GitStatus { code: Some(1) }, carried: None, placed: None, trash: None, claim: Some(claim) };
        }
    };
    let mut made: Option<ObjectId> = None;
    return match serve_snapshot(source, plan, stage, &bound, &mut made, run_git, context) {
        Ok(served) => {
            debug(TAG, format!("the snapshot served {}", plan.destination.display()).as_str());
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
            LadderResult { git: GitStatus { code: Some(0) }, carried: Some(plan.destination.clone()), placed: Some(served.placed), trash: served.trash, claim: Some(claim) }
        }
        Err(ended) if ended.stage_is_ours => {
            debug(TAG, format!("the snapshot rung ended: {}; Git checks the worktree out", ended.reason).as_str());
            discard_stage(stage, made);
            *slow = Some(explain_failure(ended.reason.as_str()));
            LadderResult { git: complete_like_git(plan, &bound, run_git), carried: None, placed: None, trash: None, claim: Some(claim) }
        }
        Err(ended) => {
            warn(TAG, format!("monochromatic-git-policy-cli: the add of {} ended in a state the wrapper cannot untangle: {}; {} is not touched", plan.destination.display(), ended.reason, stage.display()).as_str());
            report_incomplete(plan);
            LadderResult { git: GitStatus { code: Some(1) }, carried: None, placed: None, trash: None, claim: Some(claim) }
        }
    };
}

/// Release the claim on a stage name,
/// saying so when that fails;
/// flushing nothing when a subvolume was swapped into place.
fn release_claim(claim: StageClaim, swapped: bool) {
    let release_claim_entered: bool = begin_step("releaseClaim");
    let released: Result<(), WorktreeCopyError> = if swapped { claim.release_unsynced() } else { claim.release() };
    end_step(release_claim_entered);
    if let Err(error) = released {
        warn(TAG, format!("the stage claim could not be released ({})", error.0).as_str());
    }
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
    pub fn finish(self, git: GitStatus, run_git: &mut dyn GitRunner, context: &GitContext, out: &mut dyn Write, note: &mut dyn Write) -> LadderResult {
        let mut slow: Option<SlowRoute> = self.slow;
        let result: LadderResult = match (self.rung, self.plan.as_ref()) {
            (Rung::Snapshot(source, claim), Some(plan)) => {
                if !git.succeeded() {
                    release_claim(claim, false);
                    return LadderResult { git, carried: None, placed: None, trash: None, claim: None };
                }
                let stage: PathBuf = claim.container().to_path_buf();
                finish_snapshot(&source, plan, stage.as_path(), claim, run_git, context, out, note, &mut slow)
            }
            (Rung::Snapshot(_, claim), None) => {
                release_claim(claim, false);
                LadderResult { git, carried: None, placed: None, trash: None, claim: None }
            }
            (Rung::Clone(source, subvolume, claim), Some(plan)) => {
                let result: LadderResult = finish_clones(&source, plan, git, claim, run_git, context, out, &mut slow);
                if let Some(created) = subvolume.as_ref() {
                    settle_created_destination(created);
                }
                result
            }
            (Rung::Clone(_, subvolume, claim), None) => {
                if let Some(created) = subvolume.as_ref() {
                    settle_created_destination(created);
                }
                release_claim(claim, false);
                LadderResult { git, carried: None, placed: None, trash: None, claim: None }
            }
            (Rung::Git(subvolume), _) => {
                if let Some(created) = subvolume.as_ref() {
                    settle_created_destination(created);
                }
                LadderResult { git, carried: None, placed: None, trash: None, claim: None }
            }
        };
        // Only an add that succeeded is told as a slow one; a refused or failed add has a worse thing to say.
        if let (Some(route), Some(plan), true, None) = (slow.as_ref(), self.plan.as_ref(), result.git.succeeded(), result.carried.as_ref()) {
            route.tell(plan);
        }
        return result;
    }
}

/// Ladder finish controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_ladder_finish_tests.rs"]
mod tests;
