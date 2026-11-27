//! What:
//!  The path where the plan's head word is already `gone`:
//!  the clone's absence proved again,
//!  every name the proof settles,
//!  the origin attempt,
//!  the Git settle and the removal's own records.
//! Why:
//!  Once the head is gone no live inventory exists,
//!  so a name is settled by the absence proof and by nothing else,
//!  and a name that now holds another object is a reused clone name rather than this removal's work.
//!  Nothing is destroyed on this path:
//!  the destroy it records already happened in the run that wrote the plan.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = theHeadGonePath(input, folder, plan, planPath, lost, pool);
//! ```

use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_names_and_states::the_names_destroyed;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    SnapshotOutcome,
};
use crate::worktree_clone_removal_proof_and_plan::removal_completion_head_present::publish_the_plan;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_inputs::RemovalCompletion;
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::CloneAbsence;
use crate::worktree_clone_removal_proof_and_plan::removal_origin_step::attempt_the_origin_cleanup;
use crate::worktree_clone_removal_proof_and_plan::removal_records_end::{
    RecordsGone,
    remove_the_removal_records,
};
use crate::worktree_clone_removal_proof_and_plan::removal_report::{
    OriginCleanupDone,
    TheRemovalDone,
};
use crate::worktree_clone_removal_proof_and_plan::removal_settle::SettleAnswer;
use crate::worktree_clone_removal_proof_and_plan::removal_settle_step::settle_the_registration;
use crate::worktree_clone_removal_proof_and_plan::removal_slot_claim::ClaimEnd;
use crate::worktree_clone_removal_proof_and_plan::removal_snapshot_settlement::push_a_line;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::PathBuf;
/// The head-gone path:
/// the absent clone proved again,
/// every name settled by that proof,
/// the origin attempt,
/// the settle,
/// and the records.
pub(super) fn the_head_gone_path(
    input: &RemovalCompletion<'_>,
    folder: PlanFolder,
    standing: DestructionPlan,
    plan_path: PathBuf,
    lost: NamesToBeLost,
    _pool: String,
) -> Result<TheRemovalDone, RemovalStop> {
    let mut plan: DestructionPlan = standing;
    match crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::the_clone_absence(
        input.probe,
        plan.facts.clone_dataset.as_str(),
        plan.facts.clone_guid,
    )? {
        CloneAbsence::Gone => {}
        CloneAbsence::There {
            name,
            guid,
        } => {
            return Err(RemovalStop::TheCloneNameWasReused {
                name,
                recorded_guid: plan.facts.clone_guid.file_name(),
                live_guid: guid.file_name(),
            });
        }
        CloneAbsence::Reused {
            guid,
        } => {
            return Err(RemovalStop::TheCloneNameWasReused {
                name: plan.facts.clone_dataset.clone(),
                recorded_guid: plan.facts.clone_guid.file_name(),
                live_guid: guid.file_name(),
            });
        }
    }
    for snapshot in plan.named_snapshots() {
        if plan
            .current_state(snapshot.name.as_str())
            .map(|held| return held.outcome == SnapshotOutcome::Destroyed)
            .unwrap_or(false)
        {
            continue;
        }
        push_a_line(&mut plan, &snapshot, SnapshotOutcome::Destroyed, None)?;
    }
    plan.head = HeadWord::Gone;
    publish_the_plan(&folder, &plan)?;
    let origin: OriginCleanupDone = attempt_the_origin_cleanup(input, &plan)?;
    let settle: SettleAnswer = settle_the_registration(input, &plan)?;
    let records: RecordsGone =
        remove_the_removal_records(input.entry.common_dir, input.entry.slot, &plan)?;
    return Ok(TheRemovalDone {
        slot: String::from(input.entry.slot),
        dataset: plan.facts.clone_dataset.clone(),
        destroyed: the_names_destroyed(&plan),
        bookmarks: lost.bookmarks.clone(),
        remaining: Vec::new(),
        head: HeadWord::Gone,
        holder_reads_unfinished: 0,
        origin,
        settle,
        records,
        claim: ClaimEnd::Held,
        plan_path,
        requests: Vec::new(),
    });
}
