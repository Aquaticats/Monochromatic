//! What:
//!  The controls of the three checks:
//!  the healthy tuple,
//!  each recorded identity a control changes,
//!  a promoted clone's transferred snapshot and transferred bookmark,
//!  an origin-side hit,
//!  a listing that could not be completed,
//!  and the origin's mark.
//! Why:
//!  Item 11 showed that an explicit snapshot delete erases protected history before the kernel
//!  refuses the origin,
//!  so every one of these must refuse before the removal writes or deletes anything
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//! "Removal and recovery",
//! step 1).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(proveTheLineage(pool, recorded, originInventory, history)).toThrow(/transaction group 13/);
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_removal_proof_and_plan::lineage_proof::{ProvenRemoval, prove_the_lineage};
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::{
    CLONE, CLONE_CREATETXG, CLONE_GUID, ORIGIN_CREATETXG, ORIGIN_DATASET, ORIGIN_DATASET_GUID, ORIGIN_SNAPSHOT, ORIGIN_SNAPSHOT_GUID, POOL, POOL_GUID, RemovalFixture, ScriptedPool, an_object, the_stats,
};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_removal_deferred_origin_check::fixtures::origin_properties;
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, NvList};

/// The history a first removal holds.
fn nothing_destroyed() -> RemovalHistory {
    return RemovalHistory::NothingDestroyed;
}

/// Prove one pool against one fixture,
/// which is what every control here does.
fn prove(fixture: &RemovalFixture, pool: &ScriptedPool) -> Result<ProvenRemoval, RemovalStop> {
    let recorded: RecordedBinding = fixture.recorded();
    return prove_the_lineage(pool, &recorded, &fixture.the_origin_inventory(), &nothing_destroyed());
}

/// The refusal one pool earns.
fn refusal_of(fixture: &RemovalFixture, pool: &ScriptedPool) -> RemovalStop {
    return prove(fixture, pool).expect_err("the planted state is not a proven lineage");
}

#[test]
fn the_healthy_tuple_is_proved_and_its_own_snapshots_are_the_plans_tuples() {
    let fixture: RemovalFixture = RemovalFixture::healthy("healthy");
    let proven: ProvenRemoval = prove(&fixture, &fixture.healthy_pool()).expect("the healthy tuple is a proved removal");
    assert_eq!(proven.identities.clone.guid.value(), CLONE_GUID);
    assert_eq!(proven.identities.clone.createtxg, CLONE_CREATETXG);
    assert_eq!(proven.identities.origin_dataset.guid.value(), ORIGIN_DATASET_GUID);
    assert_eq!(proven.identities.origin_snapshot.guid.value(), ORIGIN_SNAPSHOT_GUID);
    assert_eq!(proven.identities.origin_snapshot.createtxg, ORIGIN_CREATETXG);
    assert_eq!(proven.identities.pool_guid.value(), POOL_GUID);
    let names: Vec<String> = proven.own_snapshots.iter().map(|snapshot| return snapshot.name.clone()).collect();
    assert_eq!(names, vec![format!("{CLONE}@sanoid-hourly"), format!("{CLONE}@worktree-own")]);
}

#[test]
fn a_pool_of_another_guid_or_a_clone_of_another_identity_refuses() {
    let fixture: RemovalFixture = RemovalFixture::healthy("identity");
    let pool: ScriptedPool = fixture.healthy_pool();
    let other_pool: ScriptedPool = pool.copy()
        .with(POOL, DatasetIdentity { stats: the_stats(POOL_GUID + 1, false, 1, 0, ""), properties: NvList::new() });
    match refusal_of(&fixture, &other_pool) {
        RemovalStop::IdentityDiffers { field, .. } => assert_eq!(field, "poolGuid"),
        other => panic!("a pool of another GUID refuses on its GUID, not {other:?}"),
    }
    let other_clone: ScriptedPool = pool.copy()
        .with(CLONE, DatasetIdentity { stats: the_stats(CLONE_GUID + 1, false, CLONE_CREATETXG, 0, ORIGIN_SNAPSHOT), properties: NvList::new() });
    match refusal_of(&fixture, &other_clone) {
        RemovalStop::IdentityDiffers { field, .. } => assert_eq!(field, "cloneGuid"),
        other => panic!("a clone of another GUID refuses on its GUID, not {other:?}"),
    }
    let other_group: ScriptedPool = pool.copy()
        .with(CLONE, DatasetIdentity { stats: the_stats(CLONE_GUID, false, CLONE_CREATETXG + 1, 0, ORIGIN_SNAPSHOT), properties: NvList::new() });
    match refusal_of(&fixture, &other_group) {
        RemovalStop::IdentityDiffers { field, .. } => assert_eq!(field, "cloneCreatetxg"),
        other => panic!("a clone of another group refuses on its group, not {other:?}"),
    }
}

#[test]
fn a_clone_whose_live_origin_is_another_snapshot_or_none_refuses() {
    let fixture: RemovalFixture = RemovalFixture::healthy("origin");
    let pool: ScriptedPool = fixture.healthy_pool();
    for live in [String::from("tank/mono-checkout@other"), String::new()] {
        let changed: ScriptedPool = pool.copy()
            .with(CLONE, DatasetIdentity { stats: the_stats(CLONE_GUID, false, CLONE_CREATETXG, 0, live.as_str()), properties: NvList::new() });
        match refusal_of(&fixture, &changed) {
            RemovalStop::OriginIsNotTheRecordedOne { recorded, live: found, .. } => {
                assert_eq!(recorded, ORIGIN_SNAPSHOT);
                assert_eq!(found.as_deref(), Some(live.as_str()).filter(|text| return !text.is_empty()));
            }
            other => panic!("a clone whose origin moved refuses on its origin, not {other:?}"),
        }
    }
}

#[test]
fn the_origin_snapshot_and_the_origin_dataset_are_proved_by_their_own_identities() {
    let fixture: RemovalFixture = RemovalFixture::healthy("origin-identity");
    let pool: ScriptedPool = fixture.healthy_pool();
    let other_snapshot: ScriptedPool = pool.copy()
        .with(ORIGIN_SNAPSHOT, DatasetIdentity { stats: the_stats(ORIGIN_SNAPSHOT_GUID + 1, true, ORIGIN_CREATETXG, 1, ""), properties: origin_properties(Some(0), Some(0)) });
    match refusal_of(&fixture, &other_snapshot) {
        RemovalStop::IdentityDiffers { field, .. } => assert_eq!(field, "originSnapshotGuid"),
        other => panic!("another origin snapshot refuses on its GUID, not {other:?}"),
    }
    let other_group: ScriptedPool = pool.copy()
        .with(ORIGIN_SNAPSHOT, DatasetIdentity { stats: the_stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_CREATETXG + 1, 1, ""), properties: origin_properties(Some(0), Some(0)) });
    match refusal_of(&fixture, &other_group) {
        RemovalStop::IdentityDiffers { field, .. } => assert_eq!(field, "originSnapshotCreatetxg"),
        other => panic!("another origin group refuses on its group, not {other:?}"),
    }
    let other_dataset: ScriptedPool = pool.copy()
        .with(ORIGIN_DATASET, DatasetIdentity { stats: the_stats(ORIGIN_DATASET_GUID + 1, false, 2, 1, ""), properties: NvList::new() });
    match refusal_of(&fixture, &other_dataset) {
        RemovalStop::IdentityDiffers { field, .. } => assert_eq!(field, "originDatasetGuid"),
        other => panic!("another origin dataset refuses on its GUID, not {other:?}"),
    }
}

#[test]
fn a_snapshot_at_or_below_the_clones_group_is_transferred_history() {
    let fixture: RemovalFixture = RemovalFixture::healthy("transferred");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.snapshots.push(an_object(EntryKind::Snapshot, "earliest", ORIGIN_SNAPSHOT_GUID, ORIGIN_CREATETXG));
    match refusal_of(&fixture, &pool) {
        RemovalStop::TransferredHistory { snapshot, createtxg, clone_createtxg, .. } => {
            assert_eq!(snapshot, format!("{CLONE}@earliest"));
            assert_eq!(createtxg, ORIGIN_CREATETXG);
            assert_eq!(clone_createtxg, CLONE_CREATETXG);
        }
        other => panic!("a promoted clone's transferred snapshot refuses as inherited history, not {other:?}"),
    }
    let message: String = refusal_of(&fixture, &pool).message();
    assert!(message.contains("Nothing was destroyed"), "the refusal says nothing was destroyed: {message}");
    assert!(message.contains("promoted or otherwise mismatched clone"), "the refusal names the promoted clone: {message}");
}

#[test]
fn a_transferred_bookmark_is_an_origin_side_hit_even_with_an_unchanged_tuple() {
    let fixture: RemovalFixture = RemovalFixture::healthy("transferred-bookmark");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.bookmarks.push(an_object(EntryKind::Bookmark, "from-earliest", ORIGIN_SNAPSHOT_GUID, ORIGIN_CREATETXG));
    match refusal_of(&fixture, &pool) {
        RemovalStop::OriginSideHit { kind, name, .. } => {
            assert_eq!(kind, EntryKind::Bookmark);
            assert_eq!(name, format!("{CLONE}#from-earliest"));
        }
        other => panic!("a bookmark the origin side holds refuses as an origin-side hit, not {other:?}"),
    }
}

#[test]
fn a_listing_that_could_not_be_completed_refuses_rather_than_reading_as_a_short_history() {
    let fixture: RemovalFixture = RemovalFixture::healthy("listing");
    let mut pool: ScriptedPool = fixture.healthy_pool();
    pool.listing_refusal = Some(String::from("the bookmarks of the clone could not be listed (input/output error)"));
    match refusal_of(&fixture, &pool) {
        RemovalStop::InventoryUnreadable { reason, .. } => assert!(reason.contains("input/output error"), "the refusal names the read: {reason}"),
        other => panic!("an incomplete list refuses, not {other:?}"),
    }
}

#[test]
fn a_marked_origin_refuses_inside_the_proof_with_nothing_destroyed() {
    let fixture: RemovalFixture = RemovalFixture::healthy("marked-origin");
    let pool: ScriptedPool = fixture.healthy_pool().with(
        ORIGIN_SNAPSHOT,
        DatasetIdentity { stats: the_stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_CREATETXG, 1, ""), properties: origin_properties(Some(1), Some(0)) },
    );
    let stop: RemovalStop = refusal_of(&fixture, &pool);
    match &stop {
        RemovalStop::OriginCheckRefused { refusal, history } => {
            assert_eq!(refusal.step(), "origin-mark");
            assert_eq!(history, &RemovalHistory::NothingDestroyed);
        }
        other => panic!("a marked origin refuses with step 15's own cause, not {other:?}"),
    }
    assert!(stop.message().contains("Nothing was destroyed") || stop.message().contains("not destroyed"), "the refusal says nothing was destroyed: {}", stop.message());
}

#[test]
fn an_origin_mark_that_cannot_be_read_refuses_as_marked() {
    let fixture: RemovalFixture = RemovalFixture::healthy("unreadable-mark");
    let pool: ScriptedPool = fixture
        .healthy_pool()
        .with(ORIGIN_SNAPSHOT, DatasetIdentity { stats: the_stats(ORIGIN_SNAPSHOT_GUID, true, ORIGIN_CREATETXG, 1, ""), properties: NvList::new() });
    match refusal_of(&fixture, &pool) {
        RemovalStop::OriginCheckRefused { refusal, .. } => assert_eq!(refusal.cause_name(), "mark-unreadable"),
        other => panic!("a mark that cannot be read refuses, not {other:?}"),
    }
}
