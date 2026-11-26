//! What:
//!  The clone's own snapshots,
//!  read from the pool,
//!  beside the states the destruction plan records for them:
//!  every name with its GUID and creation transaction group,
//!  the history it inherited from its origin,
//!  and every place the two disagree.
//! Why:
//!  One removal stop sends a person to "the clone's own snapshots" beside the
//!  plan
//!  (`RemovalStop::ThePlanCouldNotHoldTheAnswer`),
//!  and plan step 18 is where those lines are printed
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "What it builds").
//!  A snapshot is the clone's own when its creation transaction group is above
//!  the clone's,
//!  which is the removal's own rule for telling its history from the origin's,
//!  so the same split is read here rather than a second one.
//!  A name the plan calls `destroyed` that the pool still lists,
//!  and a name the plan calls `planned` that the pool does not list,
//!  are both states a person must see before any retry,
//!  and neither is guessed at:
//!  a pool that cannot be read leaves the whole answer unknown.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkCloneSnapshots({ clone, cloneCreatetxg, live, recorded }); // pass | finding('snapshots'|'guid') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use super::removal_plan_checks::RecordedSnapshot;
use super::zfs_checks::Reading;
use crate::worktree_clone_removal_proof_and_plan::plan_record::PlanSnapshot;
use crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::{
    LiveInventory,
    LiveObject,
};

/// What:
///  What the pool answers for one clone and what the plan recorded for it.
/// Why:
///  The own-history split needs the clone's own creation group,
///  the recorded states come from the plan and nowhere else,
///  and the reading keeps "not there" apart from "could not be read",
///  so one value carries every fact this check judges.
///  The plan's presence is its own fact beside its lines:
///  a plan records its first write's whole inventory,
///  so a plan that stands and holds no line is a real state,
///  and "no plan" against "a plan with no line" decides whether an own
///  snapshot is late history or unexplained.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneSnapshotFacts = { clone; cloneCreatetxg; live: Reading<LiveInventory>; planStands: boolean; recorded: RecordedSnapshot[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloneSnapshotFacts {
    /// The clone dataset whose inventory this is.
    pub clone: String,
    /// The clone's own creation transaction group.
    pub clone_createtxg: u64,
    /// What the pool answered for the clone's snapshots and bookmarks.
    pub live: Reading<LiveInventory>,
    /// Whether a destruction plan stands for the clone.
    pub plan_stands: bool,
    /// Every name the plan holds a line for,
    /// and the outcome its history ends on.
    pub recorded: Vec<RecordedSnapshot>,
}

/// What:
///  One own snapshot's name,
///  its GUID and creation group.
/// Why:
///  The tuple is what the plan's lines carry and what the removal settles a
///  name by,
///  so a person compares the report with `zfs get -H -p -o value guid,createtxg`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clause = 'clone@s1 (GUID 12, group 34)';
/// ```
fn clause(snapshot: &PlanSnapshot) -> String {
    return format!(
        "`{}` (GUID {}, group {})",
        snapshot.name,
        snapshot.guid.file_name(),
        snapshot.createtxg
    );
}

/// What:
///  The line for what the clone's own history holds,
///  naming each snapshot and how many snapshots it inherited.
/// Why:
///  A clone's transferred history is listed under its own name but was made by
///  the origin,
///  and a person who read those rows as the clone's own would expect a removal
///  to destroy them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const own = describeTheOwn(inventory.ownSnapshots(cloneCreatetxg), inventory.snapshots().length - own.length);
/// ```
fn describe_the_own(
    own: &[PlanSnapshot],
    inherited: usize,
) -> String {
    let named: Vec<String> = own.iter().map(clause).collect();
    let own_part: String = if named.is_empty() {
        String::from("it holds no snapshot of its own")
    } else {
        format!(
            "it holds {} snapshot(s) of its own: {}",
            named.len(),
            named.join("; ")
        )
    };
    return match inherited {
        0 => own_part,
        1 => format!("{own_part}, and 1 inherited snapshot that belongs to its origin"),
        count => format!("{own_part}, and {count} inherited snapshots that belong to its origin"),
    };
}

/// What:
///  Whether the clone's own snapshots are the ones its plan records,
///  in both directions.
/// Why:
///  The pool decides what is there and the plan decides what a removal
///  submitted,
///  so a name only one of them holds is a sentence:
///  a `destroyed` name the pool still lists means the plan and the pool
///  disagree about whether a destroy happened,
///  and a `planned` name the pool lost means a snapshot was destroyed outside
///  this removal.
///  Neither is reported as damage to a person's files:
///  both are read again before anything is sent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkCloneSnapshots(facts); // pass | finding('snapshots') | finding('guid') | unknown
/// ```
pub fn check_clone_snapshots(facts: &CloneSnapshotFacts) -> CheckOutcome {
    let clone: &str = facts.clone.as_str();
    let inventory: &LiveInventory = match &facts.live {
        Reading::Read(held) => held,
        Reading::Missing {
            why,
        } => {
            return finding(
                "snapshots",
                format!(
                    "the pool holds no dataset `{clone}` ({why}), so it holds no snapshot of it either: whatever the clone's files were, they are not on this pool any more, and the record and the plan that name it are the only record of it"
                ),
            );
        }
        Reading::Unreadable {
            why,
        } => {
            return unknown(format!(
                "the pool could not be read for `{clone}`'s snapshots ({why}), so neither the clone's own history nor the plan's states are proved"
            ));
        }
    };
    if inventory.dataset != facts.clone {
        return unknown(format!(
            "the inventory that was handed in describes `{}` rather than `{clone}`, so nothing here is a state of this clone",
            inventory.dataset
        ));
    }
    let own: Vec<PlanSnapshot> = inventory.own_snapshots(facts.clone_createtxg);
    let own_objects: Vec<LiveObject> = inventory
        .snapshots()
        .into_iter()
        .filter(|object| return object.createtxg > facts.clone_createtxg)
        .collect();
    let inherited: usize = inventory.snapshots().len() - own.len();
    let mut disagreements: Vec<String> = Vec::new();
    let mut late: Vec<String> = Vec::new();
    for recorded in &facts.recorded {
        let live: Option<&LiveObject> = own_objects
            .iter()
            .find(|object| return object.full_name() == recorded.name);
        match (live, recorded.outcome) {
            (Some(found), SnapshotOutcome::Destroyed) => disagreements.push(format!(
                "the plan says `{}` (GUID {}) was destroyed on attempt {}, and the pool still lists it with GUID {} in group {}",
                recorded.name,
                recorded.guid.file_name(),
                recorded.attempt,
                found.guid.file_name(),
                found.createtxg
            )),
            (Some(found), _) if found.guid != recorded.guid => disagreements.push(format!(
                "the plan recorded `{}` with GUID {} and the name now holds GUID {}, so the object the removal planned is not the one that stands",
                recorded.name,
                recorded.guid.file_name(),
                found.guid.file_name()
            )),
            (Some(_found), _) => {}
            (None, SnapshotOutcome::Destroyed) => {}
            (None, _) => disagreements.push(format!("the plan holds `{}` (GUID {}) as `{}` on attempt {}, and the pool lists no such snapshot of the clone", recorded.name, recorded.guid.file_name(), recorded.outcome.word(), recorded.attempt)),
        }
    }
    if facts.plan_stands {
        for snapshot in &own {
            if facts
                .recorded
                .iter()
                .any(|recorded| return recorded.name == snapshot.name)
            {
                continue;
            }
            late.push(clause(snapshot));
        }
    }
    let history: String = describe_the_own(own.as_slice(), inherited);
    let after: String = if late.is_empty() {
        String::new()
    } else {
        format!(
            " {} snapshot(s) were made after the plan's first write and no line names them ({}); the removal's next amendment records each of them, so they are late history and not a disagreement.",
            late.len(),
            late.join("; ")
        )
    };
    if !disagreements.is_empty() {
        return finding(
            "snapshots",
            format!(
                "`{clone}`: {history}. {} of the plan's name(s) disagree with the pool: {}. A later attempt settles each name by a fresh read before anything is sent for it.",
                disagreements.len(),
                disagreements.join("; ")
            ),
        );
    }
    if !facts.plan_stands {
        return pass(format!(
            "`{clone}`: {history}, and no destruction plan stands for it"
        ));
    }
    return pass(format!(
        "`{clone}`: {history}, and every one of the plan's {} name(s) agrees with the pool.{after}",
        facts.recorded.len()
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_snapshot_checks_tests.rs"]
mod tests;
