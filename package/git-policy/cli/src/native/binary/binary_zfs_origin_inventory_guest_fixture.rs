//! What:
//!  The origin-side inventory the add records for one slot,
//!  built from the guest's own pool rather than written by hand:
//!  the checkout dataset's snapshots at or below the origin snapshot's group,
//!  and every bookmark the checkout holds.
//! Why:
//!  A removal's lineage proof compares the clone's live GUIDs with this record to
//!  tell the clone's own history from the history it inherited,
//!  so a fixture that planted the record by hand would pass a proof the add never
//!  makes,
//!  and one that wrote no record would leave the proof nothing to compare.
//!  The clone,
//!  the registration and the slot are the slot-state fixture's
//!  (`binary_zfs_slot_state_guest_fixture.rs`),
//!  and this file is the fourth piece of an add's own records beside them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeTheOriginInventory(place, sbin, 'one');
//! ```

use super::zfs_slot_state_guest_fixture::{
    Place,
    createtxg_of,
    guid_of,
};
use super::zfs_sweep_guest::{
    NONCE,
    ORIGIN_PREFIX,
    zfs,
};
use git_policy_cli::worktree_clone_binding_record::binding_inventory::{
    EntryKind,
    InventoryEntry,
    InventoryKind,
    InventoryRecord,
};
use git_policy_cli::worktree_clone_binding_record::record_publication::publish_inventory;
use git_policy_cli::worktree_clone_binding_record::registration_home::open_or_create_home;
use git_policy_cli::worktree_clone_mount_protocol::DatasetGuid;

/// The properties one inventory listing asks a pool for,
/// which is the tuple every entry records and every proof compares.
const LISTED_PROPERTIES: &str = "name,guid,createtxg";

/// What:
///  Publish the add's origin-side inventory under one slot's name.
/// Why:
///  Every entry is read from the pool at the moment the record is written,
///  so the record names what the pool holds rather than what a fixture expected,
///  and a pool that refuses the listing stops the fixture at that listing's own
///  words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const published = writeTheOriginInventory(place, sbin, 'one');
/// ```
pub(super) fn write_the_origin_inventory(
    place: &Place,
    sbin: &str,
    slot: &str,
) {
    let origin_snapshot: String = format!("{}@{ORIGIN_PREFIX}{NONCE}", place.checkout_dataset);
    let origin_createtxg: u64 = createtxg_of(sbin, origin_snapshot.as_str());
    // A snapshot younger than the origin snapshot is the clone's own history and is not part of what the origin side
    //  recorded, and every bookmark the checkout holds is recorded whatever its group.
    let mut entries: Vec<InventoryEntry> = Vec::new();
    for (object, mark, kind) in [
        ("snapshot", '@', EntryKind::Snapshot),
        ("bookmark", '#', EntryKind::Bookmark),
    ] {
        entries.extend(
            listed(sbin, place.checkout_dataset.as_str(), object)
                .into_iter()
                .filter(|(_, _, group)| {
                    return kind != EntryKind::Snapshot || origin_createtxg >= *group;
                })
                .map(|(name, guid, group)| {
                    return InventoryEntry {
                        kind,
                        guid,
                        createtxg: group,
                        name: String::from(
                            name.split_once(mark)
                                .expect("an object names its dataset")
                                .1,
                        ),
                    };
                }),
        );
    }
    let home: std::fs::File =
        open_or_create_home(place.common.as_path()).expect("the record home opens");
    let record: InventoryRecord = InventoryRecord {
        kind: InventoryKind::Origin,
        pool_guid: guid_of(sbin, place.pool.as_str()),
        dataset: place.checkout_dataset.clone(),
        dataset_guid: guid_of(sbin, place.checkout_dataset.as_str()),
        bounds: origin_snapshot.clone(),
        bounds_guid: guid_of(sbin, origin_snapshot.as_str()),
        bounds_createtxg: origin_createtxg,
        entries,
    };
    publish_inventory(&home, slot, InventoryKind::Origin, &record)
        .expect("the origin inventory publishes once");
}

/// What:
///  One pool listing of the checkout's snapshots or bookmarks,
///  as each object's name,
///  GUID and creation transaction group.
/// Why:
///  The two listings differ in the set they name and in nothing else,
///  so one reader serves both and neither can drift from the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rows = listed(sbin, dataset, 'snapshot'); // [{ name, guid, createtxg }]
/// ```
fn listed(
    sbin: &str,
    dataset: &str,
    object: &str,
) -> Vec<(String, DatasetGuid, u64)> {
    let (ok, said, error): (bool, String, String) = zfs(
        sbin,
        "zfs",
        &[
            "list",
            "-H",
            "-p",
            "-o",
            LISTED_PROPERTIES,
            "-t",
            object,
            "-r",
            dataset,
        ],
        None,
    );
    assert!(ok, "the `{object}`s of `{dataset}` list: {said}{error}");
    let mut rows: Vec<(String, DatasetGuid, u64)> = Vec::new();
    for line in said.lines() {
        let mut fields = line.split('\t');
        let (Some(name), Some(guid), Some(createtxg)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        rows.push((
            String::from(name.trim()),
            DatasetGuid::parse(guid.trim()).expect("the object's GUID is canonical"),
            createtxg
                .trim()
                .parse()
                .expect("the object's group is a number"),
        ));
    }
    return rows;
}
