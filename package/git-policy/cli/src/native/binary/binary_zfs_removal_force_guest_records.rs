//! What:
//!  The records an add leaves beside one arm's slot,
//!  and the two records an arm plants to make a state a removal meets:
//!  an origin-side inventory the add wrote,
//!  and a destruction plan whose file cannot be read.
//! Why:
//!  A removal refuses a slot whose origin-side inventory cannot be read,
//!  and the state the project decided to refuse is a plan nothing can read,
//!  so both are records of this crate's own grammar written where the reader
//!  derives their names,
//!  and a control that hand-wrote either would prove nothing about the reader.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeTheOriginInventory(place, sbin);
//! const plan = plantAnUnreadablePlan(common, guid);
//! ```

use super::zfs_removal_force_guest_fixture::SLOT;
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
use git_policy_cli::worktree_clone_binding_record::registration_home::{
    home_folder,
    open_or_create_home,
    plans_folder,
};
use git_policy_cli::worktree_clone_mount_protocol::DatasetGuid;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The origin-side inventory the add records for one slot:
///  the checkout's snapshots at or below the origin snapshot's group and every
///  bookmark it holds.
/// Why:
///  A removal refuses a slot whose origin-side inventory cannot be read,
///  and the clone's proof compares the live objects with this set,
///  so the fixture must be the add's own record rather than a hand-written
///  list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeTheOriginInventory(place, sbin);
/// ```
pub(super) fn write_the_origin_inventory(
    place: &Place,
    sbin: &str,
) {
    let origin_snapshot: String = format!("{}@{ORIGIN_PREFIX}{NONCE}", place.checkout_dataset);
    let bounds_createtxg: u64 = createtxg_of(sbin, origin_snapshot.as_str());
    let bounds_guid: DatasetGuid = guid_of(sbin, origin_snapshot.as_str());
    let mut entries: Vec<InventoryEntry> = Vec::new();
    for (kind, separator) in [("snapshot", '@'), ("bookmark", '#')] {
        let (ok, listed, error): (bool, String, String) = zfs(
            sbin,
            "zfs",
            &[
                "list",
                "-H",
                "-p",
                "-o",
                "name,guid,createtxg",
                "-t",
                kind,
                "-r",
                place.checkout_dataset.as_str(),
            ],
            None,
        );
        assert!(ok, "the checkout's {kind}s list: {listed}{error}");
        for line in listed.lines() {
            let mut fields = line.split('\t');
            let (Some(name), Some(guid), Some(createtxg)) =
                (fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            let group: u64 = createtxg
                .trim()
                .parse()
                .expect("the object's group is a number");
            let entry_kind: EntryKind = match kind {
                "snapshot" => EntryKind::Snapshot,
                _ => EntryKind::Bookmark,
            };
            if entry_kind == EntryKind::Snapshot && group > bounds_createtxg {
                continue;
            }
            entries.push(InventoryEntry {
                kind: entry_kind,
                guid: DatasetGuid::parse(guid.trim()).expect("the object's GUID is canonical"),
                createtxg: group,
                name: String::from(
                    name.split_once(separator)
                        .expect("the object names its dataset")
                        .1,
                ),
            });
        }
    }
    let home: std::fs::File =
        open_or_create_home(place.common.as_path()).expect("the record home opens");
    let record: InventoryRecord = InventoryRecord {
        kind: InventoryKind::Origin,
        pool_guid: guid_of(sbin, place.pool.as_str()),
        dataset: place.checkout_dataset.clone(),
        dataset_guid: guid_of(sbin, place.checkout_dataset.as_str()),
        bounds: origin_snapshot,
        bounds_guid,
        bounds_createtxg,
        entries,
    };
    publish_inventory(&home, SLOT, InventoryKind::Origin, &record)
        .expect("the origin inventory publishes once");
}

/// The record home's folder,
/// for an arm that checks the records went.
pub(super) fn the_record_home(common: &Path) -> PathBuf {
    return home_folder(common);
}

/// What:
///  Write one unreadable destruction plan where a removal's plan for this clone
///  would stand,
///  in the folder and at the name the reader derives.
/// Why:
///  The state the project decided to refuse is a plan whose file cannot be read,
///  so the bytes are ones the grammar refuses and the name is the clone GUID's,
///  which is the only name the removal looks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = plantAnUnreadablePlan(common, cloneGuid);
/// ```
pub(super) fn plant_an_unreadable_plan(
    common: &Path,
    clone_guid: DatasetGuid,
) -> PathBuf {
    let folder: PathBuf = plans_folder(common);
    std::fs::create_dir_all(folder.as_path()).expect("the plan folder is made");
    std::fs::set_permissions(
        folder.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o700),
    )
    .expect("the plan folder holds the design's mode");
    let path: PathBuf = folder.join(clone_guid.file_name());
    std::fs::write(path.as_path(), "this file is no destruction plan\n")
        .expect("the unreadable plan is written");
    std::fs::set_permissions(
        path.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o600),
    )
    .expect("the unreadable plan holds a mode the reader accepts");
    return path;
}
