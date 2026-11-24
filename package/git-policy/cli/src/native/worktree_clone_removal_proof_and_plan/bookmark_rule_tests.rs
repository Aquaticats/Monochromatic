//! What:
//!  The controls of the decided bookmark rule,
//!  `A: Live or planned`:
//!  a bookmark a live snapshot explains,
//!  a bookmark a plan line explains,
//!  and the two refusals.
//! Why:
//!  The project decided this rule on 2026-11-19 and the alternatives are recorded beside it:
//!  the live-only rule would strand a removal after its first chunk,
//!  and the transaction-group rule would pass a stale booking
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Bookmarks on a clone being removed").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(refuseTheUnexplainedBookmarks(...).map(b => b.name)).toEqual(['...']);
//! ```

use crate::worktree_clone_binding_record::binding_inventory::{EntryKind, InventoryEntry, InventoryRecord};
use crate::worktree_clone_removal_proof_and_plan::bookmark_rule::{Explanations, explanations_of, refuse_the_unexplained_bookmarks};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{CLONE, CLONE_GUID, ORIGIN_CREATETXG, ORIGIN_SNAPSHOT_GUID, OWN_GUID, SANOID_GUID, an_object, guid};
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::{LiveInventory, LiveObject};
use crate::worktree_clone_removal_proof_and_plan::removal_stop::{BookmarkRefusalReason, RefusedBookmark};

/// The origin-side inventory one control compares against,
/// which holds the origin snapshot's tuple.
fn the_origin_inventory() -> InventoryRecord {
    return InventoryRecord {
        kind: crate::worktree_clone_binding_record::binding_inventory::InventoryKind::Origin,
        pool_guid: guid(1),
        dataset: String::from("tank/mono-checkout"),
        dataset_guid: guid(2),
        bounds: String::from("tank/mono-checkout@origin"),
        bounds_guid: guid(ORIGIN_SNAPSHOT_GUID),
        bounds_createtxg: ORIGIN_CREATETXG,
        entries: vec![InventoryEntry { kind: EntryKind::Snapshot, guid: guid(ORIGIN_SNAPSHOT_GUID), createtxg: ORIGIN_CREATETXG, name: String::from("origin") }],
    };
}

/// One clone holding its two own snapshots and the bookmarks a control plants.
fn an_inventory(bookmarks: Vec<LiveObject>) -> LiveInventory {
    let mut objects: Vec<LiveObject> = vec![an_object(EntryKind::Snapshot, "worktree-own", OWN_GUID, 18), an_object(EntryKind::Snapshot, "sanoid-hourly", SANOID_GUID, 20)];
    objects.extend(bookmarks);
    return LiveInventory { dataset: String::from(CLONE), dataset_guid: guid(CLONE_GUID), objects };
}

#[test]
fn a_bookmark_a_live_snapshot_explains_passes() {
    let inventory: LiveInventory = an_inventory(vec![an_object(EntryKind::Bookmark, "from-own", OWN_GUID, 18)]);
    let explanations: Explanations = explanations_of(&inventory, None);
    let refused: Vec<RefusedBookmark> = refuse_the_unexplained_bookmarks(&explanations, inventory.bookmarks().as_slice(), &the_origin_inventory());
    assert!(refused.is_empty(), "a bookmark its own live snapshot explains passes");
}

#[test]
fn a_bookmark_a_plan_line_explains_passes_whatever_that_line_outcome_is() {
    let inventory: LiveInventory = an_inventory(vec![an_object(EntryKind::Bookmark, "from-own", OWN_GUID, 18)]);
    let mut plan = crate::worktree_clone_removal_proof_and_plan::fixtures::sample_plan();
    plan.attempts[0].snapshot.guid = guid(OWN_GUID);
    plan.attempts[0].snapshot.createtxg = 18;
    plan.attempts[0].outcome = crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome::Unknown;
    let without_the_snapshot: LiveInventory = LiveInventory { objects: Vec::new(), ..inventory.clone() };
    let explanations: Explanations = explanations_of(&without_the_snapshot, Some(&plan));
    let refused: Vec<RefusedBookmark> = refuse_the_unexplained_bookmarks(&explanations, inventory.bookmarks().as_slice(), &the_origin_inventory());
    assert!(refused.is_empty(), "a plan line explains a bookmark whose snapshot a chunk already destroyed, and an unknown line still does");
}

#[test]
fn a_bookmark_neither_a_snapshot_nor_a_plan_line_explains_is_refused_and_named() {
    let inventory: LiveInventory = an_inventory(vec![an_object(EntryKind::Bookmark, "from-a-gone-snapshot", 42, 19)]);
    let explanations: Explanations = explanations_of(&inventory, None);
    let refused: Vec<RefusedBookmark> = refuse_the_unexplained_bookmarks(&explanations, inventory.bookmarks().as_slice(), &the_origin_inventory());
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].name, format!("{CLONE}#from-a-gone-snapshot"));
    assert_eq!(refused[0].guid, guid(42));
    assert_eq!(refused[0].createtxg, 19);
    assert_eq!(refused[0].reason, BookmarkRefusalReason::Unexplained);
}

#[test]
fn a_bookmark_the_origin_side_holds_is_refused_even_when_a_live_snapshot_explains_it() {
    let inventory: LiveInventory = an_inventory(vec![an_object(EntryKind::Bookmark, "from-origin", ORIGIN_SNAPSHOT_GUID, ORIGIN_CREATETXG)]);
    let with_the_tuple: LiveInventory = LiveInventory {
        dataset: String::from(CLONE),
        dataset_guid: guid(CLONE_GUID),
        objects: vec![
            an_object(EntryKind::Snapshot, "worktree-own", OWN_GUID, 18),
            an_object(EntryKind::Snapshot, "origin", ORIGIN_SNAPSHOT_GUID, ORIGIN_CREATETXG),
        ],
    };
    // The live snapshot carries the tuple as well, so only the origin-side arm refuses it.
    let explanations: Explanations = explanations_of(&with_the_tuple, None);
    let refused: Vec<RefusedBookmark> = refuse_the_unexplained_bookmarks(&explanations, inventory.bookmarks().as_slice(), &the_origin_inventory());
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].reason, BookmarkRefusalReason::OriginSide);
}

#[test]
fn every_refused_bookmark_is_named_and_not_only_the_first() {
    let inventory: LiveInventory = an_inventory(vec![
        an_object(EntryKind::Bookmark, "one", 41, 19),
        an_object(EntryKind::Bookmark, "from-own", OWN_GUID, 18),
        an_object(EntryKind::Bookmark, "two", 43, 21),
    ]);
    let explanations: Explanations = explanations_of(&inventory, None);
    let refused: Vec<RefusedBookmark> = refuse_the_unexplained_bookmarks(&explanations, inventory.bookmarks().as_slice(), &the_origin_inventory());
    let names: Vec<String> = refused.iter().map(|bookmark| return bookmark.name.clone()).collect();
    assert_eq!(names, vec![format!("{CLONE}#one"), format!("{CLONE}#two")]);
}
