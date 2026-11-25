//! What:
//!  The project's decided bookmark rule,
//!  `A: Live or planned`:
//!  a bookmark passes when a live passing clone snapshot or a recorded plan tuple explains its
//!  tuple,
//!  and every other bookmark is refused and named before the first delete.
//! Why:
//!  A bookmark carries the GUID and birth transaction group of the snapshot it was made from,
//!  and a dataset's bookmarks are destroyed with its head,
//!  so a bookmark on a clone is explained by a snapshot or it is not
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Bookmarks on a clone being removed").
//!  The rule refuses a bookmark whose tuple matches neither,
//!  one whose tuple the origin-side inventory holds,
//!  and every bookmark of a list that could not be read completely,
//!  which the inventory read refuses as a whole before this rule runs.
//!  A refused bookmark is a person's `zfs destroy <clone>#<bookmark>`:
//!  nothing in this design destroys a bookmark,
//!  and a plan never records one.
//!  A plan line explains a bookmark whatever its outcome,
//!  because an `unknown` line still says this clone had that snapshot.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refused = refuseTheUnexplainedBookmarks(inventory, planTuples, originInventory);
//! ```

use crate::worktree_clone_binding_record::binding_inventory::InventoryRecord;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{DestructionPlan, PlanSnapshot};
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::{LiveInventory, LiveObject};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::{BookmarkRefusalReason, RefusedBookmark};

/// What:
///  The tuples a bookmark may be explained by:
///  the live snapshots that passed the lineage check and the plan's own lines.
/// Why:
///  The design's rule reads one of the two for every bookmark,
///  and the plan is how a resumed removal still explains a bookmark whose snapshot an earlier run
///  destroyed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Explanations = { liveSnapshots: LiveObject[]; plannedTuples: PlanSnapshot[] };
/// ```
pub struct Explanations {
    /// The clone's live snapshots,
    /// each of which the lineage check already passed.
    pub live_snapshots: Vec<LiveObject>,
    /// Every snapshot the plan names,
    /// each once.
    pub planned_tuples: Vec<PlanSnapshot>,
}

/// What:
///  Apply the decided rule to every bookmark of one live inventory.
/// Why:
///  The removal refuses the whole set when the rule refuses one bookmark,
///  so this returns every refused bookmark and never the first one only:
///  a person needs the whole list before destroying any of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refused = refuseTheUnexplainedBookmarks(explanations, bookmark, originInventory);
/// ```
pub fn refuse_the_unexplained_bookmarks(explanations: &Explanations, bookmarks: &[LiveObject], origin_inventory: &InventoryRecord) -> Vec<RefusedBookmark> {
    let mut refused: Vec<RefusedBookmark> = Vec::new();
    for bookmark in bookmarks {
        let reason: Option<BookmarkRefusalReason> = judge_one(bookmark, explanations, origin_inventory);
        let Some(found) = reason else { continue };
        refused.push(RefusedBookmark { name: bookmark.full_name(), guid: bookmark.guid, createtxg: bookmark.createtxg, reason: found });
    }
    return refused;
}

/// What:
///  One bookmark's answer,
///  or nothing when the rule passes it.
/// Why:
///  The origin-side arm comes first:
///  a bookmark whose tuple the origin recorded belongs to the history the clone inherited,
///  whatever else the clone also holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = explain(bookmark) ? undefined : 'no-live-or-planned-snapshot';
/// ```
fn judge_one(bookmark: &LiveObject, explanations: &Explanations, origin_inventory: &InventoryRecord) -> Option<BookmarkRefusalReason> {
    if origin_inventory.entries.iter().any(|entry| return entry.guid == bookmark.guid) {
        return Some(BookmarkRefusalReason::OriginSide);
    }
    let by_a_live_snapshot: bool = explanations
        .live_snapshots
        .iter()
        .any(|snapshot| return snapshot.guid == bookmark.guid && snapshot.createtxg == bookmark.createtxg && snapshot.owning_dataset == bookmark.owning_dataset);
    if by_a_live_snapshot {
        return None;
    }
    let by_the_plan: bool = explanations
        .planned_tuples
        .iter()
        .any(|planned| return planned.guid == bookmark.guid && planned.createtxg == bookmark.createtxg);
    if by_the_plan {
        return None;
    }
    return Some(BookmarkRefusalReason::Unexplained);
}

/// What:
///  The explanations one removal holds:
///  its live passing snapshots and the plan it read,
///  when one stands.
/// Why:
///  The same two sources answer every bookmark,
///  and one value keeps the caller from passing a plan of one clone with an inventory of another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const explanations = explanationsOf(inventory, plan);
/// ```
pub fn explanations_of(inventory: &LiveInventory, plan: Option<&DestructionPlan>) -> Explanations {
    let planned: Vec<PlanSnapshot> = match plan {
        Some(found) => found.named_snapshots(),
        None => Vec::new(),
    };
    return Explanations { live_snapshots: inventory.snapshots(), planned_tuples: planned };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "bookmark_rule_tests.rs"]
mod tests;
