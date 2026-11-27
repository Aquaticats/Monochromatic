//! What:
//!  The names the plan recorded settled by their GUIDs against the live inventory:
//!  the new own snapshots the clone's history holds,
//!  the amended plan a new own snapshot takes,
//!  and the one line each settlement appends.
//! Why:
//!  A snapshot ends only when no live snapshot carries its recorded GUID,
//!  never from a missing name alone,
//!  because a name may be absent and then present again
//!  and a reused name holds another object
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The absence rule,
//! object by object").
//!  The amended-plan rule submits a newly created own snapshot by the wrapper and keeps every
//!  earlier line,
//!  and history at or below the clone's own transaction group is transferred from the origin,
//!  which no removal of a clone destroys.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! settleTheRecordedNames(plan, live);
//! const added = theNewOwnSnapshots(plan, live);
//! amendThePlan(plan, added);
//! ```

use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanAttempt,
    PlanSnapshot,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::Path;

/// What:
///  Every name the plan has not settled,
///  judged by the live inventory:
///  absent by GUID is `destroyed`,
///  live with its recorded GUID and group is `intact`,
///  and a live GUID with another group refuses.
/// Why:
///  The three answers are the design's own,
///  and the third is permanent:
///  a live snapshot carrying the recorded GUID in another transaction group is another object,
///  and no wrapper command replaces an outcome with a guess.
///
/// # Throws
/// A snapshot whose recorded GUID answers with another group,
/// or a line the plan's merge rule refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settled = settleTheRecordedNames(plan, live);
/// ```
pub fn settle_the_recorded_names(
    plan: &mut DestructionPlan,
    live: &[LiveObject],
) -> Result<Vec<String>, RemovalStop> {
    let named: Vec<PlanSnapshot> = plan.named_snapshots();
    let mut settled: Vec<String> = Vec::new();
    for snapshot in named {
        if plan
            .current_state(snapshot.name.as_str())
            .map(|held| return held.outcome == SnapshotOutcome::Destroyed)
            .unwrap_or(false)
        {
            continue;
        }
        let Some(found) = live
            .iter()
            .find(|object| return object.guid == snapshot.guid)
        else {
            push_a_line(plan, &snapshot, SnapshotOutcome::Destroyed, None)?;
            settled.push(snapshot.name.clone());
            continue;
        };
        let own_name: &str = snapshot
            .name
            .rsplit_once('@')
            .map_or(snapshot.name.as_str(), |pair| return pair.1);
        if found.createtxg != snapshot.createtxg || found.name != own_name {
            return Err(RemovalStop::ASnapshotIsNotTheRecordedOne {
                snapshot: snapshot.name.clone(),
                recorded_guid: snapshot.guid.file_name(),
                recorded_txg: snapshot.createtxg,
                live_txg: found.createtxg,
                live_name: format!("{}@{}", found.owning_dataset, found.name),
            });
        }
        push_a_line(plan, &snapshot, SnapshotOutcome::Intact, None)?;
    }
    return Ok(settled);
}

/// What:
///  The live snapshots of the clone the plan does not name,
///  as the amended plan's new lines:
///  the clone's own history,
/// verified by its transaction group.
/// Why:
///  The proposal's amended-plan rule submits a newly created own snapshot by the wrapper and keeps
///  every earlier line,
///  and history at or below the clone's own group is transferred from the origin,
///  which no removal of a clone destroys.
///
/// # Throws
/// A live snapshot that is not the clone's own history.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = theNewOwnSnapshots(plan, live);
/// ```
pub fn the_new_own_snapshots(
    plan: &DestructionPlan,
    live: &[LiveObject],
) -> Result<Vec<PlanSnapshot>, RemovalStop> {
    let mut added: Vec<PlanSnapshot> = Vec::new();
    for object in live {
        if plan
            .current_state(&format!("{}@{}", object.owning_dataset, object.name))
            .is_some()
        {
            continue;
        }
        if plan
            .named_snapshots()
            .iter()
            .any(|held| return held.guid == object.guid)
        {
            continue;
        }
        if object.createtxg <= plan.facts.clone_createtxg {
            return Err(RemovalStop::TransferredHistory {
                clone: plan.facts.clone_dataset.clone(),
                snapshot: object.full_name(),
                createtxg: object.createtxg,
                clone_createtxg: plan.facts.clone_createtxg,
            });
        }
        added.push(PlanSnapshot {
            name: object.full_name(),
            guid: object.guid,
            createtxg: object.createtxg,
        });
    }
    return Ok(added);
}

/// What:
///  The plan with one `planned` line at attempt `0` for each new own snapshot,
///  and the earlier lines kept.
/// Why:
///  The amended record is one rewrite with every earlier tuple and its outcome kept,
///  so a person reading it still sees what the earlier attempts did.
///
/// # Throws
/// A line the plan's merge rule refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// amendThePlan(plan, added);
/// ```
pub fn amend_the_plan(
    plan: &mut DestructionPlan,
    added: &[PlanSnapshot],
) -> Result<(), RemovalStop> {
    for snapshot in added {
        let Some(existing) = plan
            .attempts
            .iter()
            .find(|line| return line.snapshot.guid == snapshot.guid)
        else {
            let line: PlanAttempt = PlanAttempt {
                snapshot: snapshot.clone(),
                attempt: 0,
                outcome: SnapshotOutcome::Planned,
                errno: None,
            };
            plan.record_line(line).map_err(|refusal| {
                return RemovalStop::ThePlanCouldNotHoldTheAnswer {
                    path: std::path::PathBuf::new(),
                    said: refusal.message(Path::new("the plan")),
                };
            })?;
            continue;
        };
        if existing.snapshot.name != snapshot.name {
            return Err(RemovalStop::ASnapshotIsNotTheRecordedOne {
                snapshot: snapshot.name.clone(),
                recorded_guid: existing.snapshot.guid.file_name(),
                recorded_txg: existing.snapshot.createtxg,
                live_txg: snapshot.createtxg,
                live_name: snapshot.name.clone(),
            });
        }
    }
    return Ok(());
}

/// What:
///  One line appended at one name's next attempt.
/// Why:
///  The merge rule reads a name's highest attempt,
///  so a settlement is a new line and never an edit of an old one.
///
/// # Throws
/// A duplicate attempt or a line after a `destroyed` one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plan.recordLine({ snapshot, attempt: next(name), outcome });
/// ```
pub fn push_a_line(
    plan: &mut DestructionPlan,
    snapshot: &PlanSnapshot,
    outcome: SnapshotOutcome,
    errno: Option<u32>,
) -> Result<(), RemovalStop> {
    let attempt: u64 = plan
        .current_state(snapshot.name.as_str())
        .map_or(1, |held| return held.attempt + 1);
    let line: PlanAttempt = PlanAttempt {
        snapshot: snapshot.clone(),
        attempt,
        outcome,
        errno,
    };
    return plan.record_line(line).map_err(|refusal| {
        return RemovalStop::ThePlanCouldNotHoldTheAnswer {
            path: std::path::PathBuf::new(),
            said: refusal.message(Path::new("the plan")),
        };
    });
}
