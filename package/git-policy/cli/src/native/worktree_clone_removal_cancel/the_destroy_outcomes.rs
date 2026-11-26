//! What:
//!  The destruction plan's tuples settled by the clone's live inventory:
//!  which snapshots the removal destroyed,
//!  and every state in which an outcome cannot be settled.
//! Why:
//!  The command settles outstanding destroy requests before anything moves
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "Settling the destroy outcomes before anything moves").
//!  A tuple whose recorded GUID is in the clone's live inventory was not destroyed,
//!  and one whose GUID is absent was,
//!  because a dataset's GUID is random at each creation and a name alone proves nothing.
//!  A name the live inventory carries with another GUID is a refusal rather than a settlement,
//!  and so is a tuple the plan itself marks `unknown`,
//!  because the command may not attach over an outcome nobody can prove.
//!  Nothing here writes to the plan:
//!  "a settle is not an outcome",
//!  so the report names the destroyed set and the plan keeps exactly what the removal wrote.
//!  The answer is the names and nothing else,
//!  because a name that was not destroyed is a name no report prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const destroyed = settleTheDestroyOutcomes(plan, live); // string[] or refuses
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;

/// What:
///  Settle every tuple of one plan against the clone's live snapshots.
/// Why:
///  Only the GUID settles a tuple,
///  so a name that now carries another object is a refusal rather than either answer,
///  and a tuple the plan calls `unknown` is refused before the inventory is even read for it.
///
/// # Errors
/// A tuple the plan marks unknown,
/// a recorded name the pool carries with another GUID,
/// or a live snapshot carrying the recorded GUID under another name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settleTheDestroyOutcomes(plan, live);
/// ```
pub fn settle_the_destroy_outcomes(
    plan: &DestructionPlan,
    live: &[LiveObject],
) -> Result<Vec<String>, CancelRefusal> {
    let mut destroyed: Vec<String> = Vec::new();
    for snapshot in plan.named_snapshots() {
        let name: String = snapshot.name.clone();
        let state: Option<SnapshotOutcome> = plan
            .current_state(name.as_str())
            .map(|held| return held.outcome);
        if state == Some(SnapshotOutcome::Unknown) {
            return Err(CancelRefusal::AnUnknownOutcome {
                snapshot: name,
                guid: snapshot.guid.file_name(),
                said: String::from("the plan marks it `unknown`"),
            });
        }
        if is_destroyed_by_its_line(state) {
            destroyed.push(name);
            continue;
        }
        let by_guid: Option<&LiveObject> = live.iter().find(|object| {
            return object.kind == EntryKind::Snapshot && object.guid == snapshot.guid;
        });
        if let Some(object) = by_guid {
            if object.full_name() != name {
                return Err(CancelRefusal::TheRecordsDisagree {
                    what: String::from(
                        "the live snapshot carrying the recorded GUID does not carry the recorded name",
                    ),
                    first: format!("{name} (GUID {})", snapshot.guid.file_name()),
                    second: object.full_name(),
                });
            }
            continue;
        }
        let reused: Option<&LiveObject> = live
            .iter()
            .find(|object| return object.kind == EntryKind::Snapshot && object.full_name() == name);
        if let Some(object) = reused {
            return Err(CancelRefusal::ASnapshotNameIsReused {
                name,
                recorded_guid: snapshot.guid.file_name(),
                live_guid: object.guid.file_name(),
            });
        }
        destroyed.push(name);
    }
    return Ok(destroyed);
}

/// Whether the outcome written on a tuple's highest line already says it is gone.
///
/// Why:
///  `destroyed` is final for a name,
///  so a plan that already recorded the destroy settles that tuple without the inventory.
fn is_destroyed_by_its_line(state: Option<SnapshotOutcome>) -> bool {
    return state == Some(SnapshotOutcome::Destroyed);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_destroy_outcomes_tests.rs"]
mod tests;
