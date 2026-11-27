//! What:
//!  The plan's own names and the state each one's history ends on:
//!  the names still to destroy,
//!  the names already destroyed,
//!  every name with the word a stop prints beside it,
//!  and the fresh listing the report and the stop read.
//! Why:
//!  The whole-removal order asks these four questions at three moments,
//!  and a reader of either answers from one place:
//!  a name is destroyed when its highest attempt line says so,
//!  and a name no line settles is still planned,
//!  which is the merge rule `plan_record.rs` writes and this file reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const remaining = theNamesNotDestroyed(plan);
//! const destroyed = theNamesDestroyed(plan);
//! const lines = theStatesOf(plan, remaining);
//! ```

use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use crate::worktree_clone_removal_proof_and_plan::removal_completion_inputs::RemovalCompletion;
use crate::worktree_clone_removal_proof_and_plan::removal_destruction::the_remaining_snapshots;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
/// Every name of the plan whose current state is not `destroyed`,
/// sorted.
pub fn the_names_not_destroyed(plan: &DestructionPlan) -> Vec<String> {
    let mut names: Vec<String> = plan
        .named_snapshots()
        .iter()
        .filter(|snapshot| {
            return plan
                .current_state(snapshot.name.as_str())
                .map(|held| return held.outcome != SnapshotOutcome::Destroyed)
                .unwrap_or(true);
        })
        .map(|snapshot| return snapshot.name.clone())
        .collect();
    names.sort();
    return names;
}

/// Every name of the plan whose current state is `destroyed`.
pub(super) fn the_names_destroyed(plan: &DestructionPlan) -> Vec<String> {
    return plan
        .named_snapshots()
        .iter()
        .filter(|snapshot| {
            return plan
                .current_state(snapshot.name.as_str())
                .map(|held| return held.outcome == SnapshotOutcome::Destroyed)
                .unwrap_or(false);
        })
        .map(|snapshot| return snapshot.name.clone())
        .collect();
}

/// The names with the state their history ends on,
/// which is what a stop prints.
pub fn the_states_of(
    plan: &DestructionPlan,
    names: &[String],
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for name in names {
        let state: &str = plan
            .current_state(name.as_str())
            .map_or("planned", |held| return held.outcome.word());
        lines.push(format!("{name} ({state})"));
    }
    return lines;
}

/// What:
///  The snapshot the removal's own enumeration still finds,
///  as the batch's fresh enumeration did,
///  which the report and the stop read.
/// Why:
///  The design enumerates again after the batch and before the head,
///  and this is that read's own list,
///  which differs from the plan's names by a snapshot that appeared after the first write.
///
/// # Throws
/// The listing that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remaining = theRemainingSnapshots(reads, plan);
/// ```
pub fn the_remaining(
    input: &RemovalCompletion<'_>,
    plan: &DestructionPlan,
) -> Result<Vec<String>, RemovalStop> {
    return the_remaining_snapshots(input.reads, plan);
}
