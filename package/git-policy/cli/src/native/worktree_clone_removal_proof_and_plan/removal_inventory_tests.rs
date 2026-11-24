//! What:
//!  The controls of the clone's live inventory:
//!  the own set,
//!  the bookmark list,
//!  and the full name a request uses.
//! Why:
//!  The plan records the clone's own snapshots and no bookmark,
//!  and the separation is the transaction group the design's `createtxg` check rests on
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`, "Removal and recovery", step 1).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(inventory.ownSnapshots(17).map(s => s.name)).toEqual(['...@sanoid-hourly', '...@worktree-own']);
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{CLONE, CLONE_CREATETXG, CLONE_GUID, ORIGIN_CREATETXG, ORIGIN_SNAPSHOT_GUID, OWN_GUID, SANOID_GUID, an_object, guid};
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::{LiveInventory, LiveObject};

/// The inventory two of the design's own snapshots make.
fn an_inventory() -> LiveInventory {
    return LiveInventory {
        dataset: String::from(CLONE),
        dataset_guid: guid(CLONE_GUID),
        objects: vec![an_object(EntryKind::Snapshot, "worktree-own", OWN_GUID, 18), an_object(EntryKind::Snapshot, "sanoid-hourly", SANOID_GUID, 20)],
    };
}

#[test]
fn the_own_set_is_every_snapshot_above_the_clones_own_group_in_name_order() {
    let inventory: LiveInventory = an_inventory();
    let own: Vec<String> = inventory.own_snapshots(17).into_iter().map(|snapshot| return snapshot.name).collect();
    assert_eq!(own, vec![format!("{CLONE}@sanoid-hourly"), format!("{CLONE}@worktree-own")]);
}

#[test]
fn a_snapshot_at_or_below_the_clones_own_group_is_not_its_own() {
    let inventory: LiveInventory = LiveInventory {
        dataset: String::from(CLONE),
        dataset_guid: guid(CLONE_GUID),
        objects: vec![
            an_object(EntryKind::Snapshot, "origin", ORIGIN_SNAPSHOT_GUID, ORIGIN_CREATETXG),
            an_object(EntryKind::Snapshot, "own", OWN_GUID, CLONE_CREATETXG),
        ],
    };
    assert!(inventory.own_snapshots(CLONE_CREATETXG).is_empty(), "a snapshot at or below the clone's own group is inherited, not the clone's");
    assert_eq!(inventory.snapshots().len(), 2, "and both are still listed");
}

#[test]
fn a_bookmark_is_listed_apart_and_never_recorded_as_an_own_snapshot() {
    let bookmark: LiveObject = an_object(EntryKind::Bookmark, "from-earliest", OWN_GUID, 18);
    let inventory: LiveInventory = LiveInventory {
        dataset: String::from(CLONE),
        dataset_guid: guid(CLONE_GUID),
        objects: vec![an_object(EntryKind::Snapshot, "worktree-own", OWN_GUID, 18), bookmark.clone()],
    };
    assert_eq!(inventory.bookmarks(), vec![bookmark]);
    assert_eq!(inventory.snapshots().len(), 1, "one snapshot and one bookmark are two objects");
    let own: Vec<String> = inventory.own_snapshots(17).into_iter().map(|snapshot| return snapshot.name).collect();
    assert_eq!(own, vec![format!("{CLONE}@worktree-own")], "no bookmark enters the plan");
}

#[test]
fn an_objects_full_name_uses_the_separator_its_kind_takes() {
    assert_eq!(an_object(EntryKind::Snapshot, "own", OWN_GUID, 18).full_name(), format!("{CLONE}@own"));
    assert_eq!(an_object(EntryKind::Bookmark, "own", OWN_GUID, 18).full_name(), format!("{CLONE}#own"));
}
