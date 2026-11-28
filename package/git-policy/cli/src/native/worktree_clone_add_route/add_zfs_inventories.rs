//! What:
//!  The two lists one add takes from the pool:
//!  the origin dataset's snapshots
//!  and bookmarks at or below the snapshot the clone is made from,
//!  and the clone's own snapshots and bookmarks immediately after it was made.
//! Why:
//!  The binding records both lists,
//!  so a later reader can tell what the clone
//!  was made from without asking the pool a question whose answer has moved
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  A9).
//!  `add_zfs.rs` sends the requests and would go over the file-size limit
//!  carrying the two listings,
//!  and `add_zfs_pool_answers.rs` holds the reads and refusals they list by,
//!  so this file is the two listings and nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const origin = device.originInventory(dataset, poolGuid, datasetGuid, bounds, identity);
//! const clone = device.cloneInventory(clone, poolGuid, identity);
//! ```

use super::add_zfs::OpenedDevice;
use super::add_zfs_pool_answers::{
    ObjectIdentity,
    guid_of,
    nested_u64,
    short_name,
};
use crate::worktree_clone_binding_record::binding_inventory::{
    EntryKind,
    InventoryEntry,
    InventoryKind,
    InventoryRecord,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

impl OpenedDevice {
    /// The origin dataset's snapshots at or below `bounds_createtxg`,
    /// and its bookmarks,
    /// as the origin inventory's set.
    ///
    /// # Errors
    /// A listing step failed,
    /// or a bookmark carries no GUID or creation transaction group.
    pub fn origin_inventory(
        &self,
        dataset: &str,
        pool_guid: DatasetGuid,
        dataset_guid: DatasetGuid,
        bounds: &str,
        bounds_identity: ObjectIdentity,
    ) -> Result<InventoryRecord, String> {
        let mut entries: Vec<InventoryEntry> = Vec::new();
        let mut cursor: u64 = 0;
        while let Some(object) =
            self.device
                .snapshot_list_next(dataset, cursor)
                .map_err(|error| {
                    return format!("the snapshots of {dataset} could not be listed ({error})");
                })?
        {
            cursor = object.cursor;
            let createtxg: u64 = object.stats.dds_creation_txg;
            if createtxg > bounds_identity.createtxg {
                continue;
            }
            let name: String = short_name(object.name.as_str(), dataset);
            entries.push(InventoryEntry {
                kind: EntryKind::Snapshot,
                guid: guid_of(object.stats.dds_guid, object.name.as_str())?,
                createtxg,
                name,
            });
        }
        let bookmarks: NvList = self.device.get_bookmarks(dataset, &[]).map_err(|error| {
            return format!("the bookmarks of {dataset} could not be listed ({error})");
        })?;
        for pair in bookmarks.iter() {
            let monochromatic_zfs_request_to_openzfs_kernel_device::NvData::List {
                value,
            } = &pair.data
            else {
                continue;
            };
            let guid: u64 = nested_u64(value, "guid")
                .ok_or_else(|| return format!("the bookmark {} carries no GUID", pair.name))?;
            let createtxg: u64 = nested_u64(value, "createtxg").ok_or_else(|| {
                return format!(
                    "the bookmark {} carries no creation transaction group",
                    pair.name
                );
            })?;
            entries.push(InventoryEntry {
                kind: EntryKind::Bookmark,
                guid: guid_of(guid, pair.name.as_str())?,
                createtxg,
                name: pair.name.clone(),
            });
        }
        return Ok(InventoryRecord {
            kind: InventoryKind::Origin,
            pool_guid,
            dataset: String::from(dataset),
            dataset_guid,
            bounds: String::from(bounds),
            bounds_guid: bounds_identity.guid,
            bounds_createtxg: bounds_identity.createtxg,
            entries,
        });
    }

    /// The clone's own snapshots and bookmarks immediately after it was made.
    ///
    /// # Errors
    /// A listing step failed,
    /// or a bookmark carries no GUID or creation transaction group.
    pub fn clone_inventory(
        &self,
        clone: &str,
        pool_guid: DatasetGuid,
        identity: ObjectIdentity,
    ) -> Result<InventoryRecord, String> {
        let mut entries: Vec<InventoryEntry> = Vec::new();
        let mut cursor: u64 = 0;
        while let Some(object) = self
            .device
            .snapshot_list_next(clone, cursor)
            .map_err(|error| {
                return format!("the clone's snapshots could not be listed ({error})");
            })?
        {
            cursor = object.cursor;
            entries.push(InventoryEntry {
                kind: EntryKind::Snapshot,
                guid: guid_of(object.stats.dds_guid, object.name.as_str())?,
                createtxg: object.stats.dds_creation_txg,
                name: short_name(object.name.as_str(), clone),
            });
        }
        return Ok(InventoryRecord {
            kind: InventoryKind::Clone,
            pool_guid,
            dataset: String::from(clone),
            dataset_guid: identity.guid,
            bounds: String::from(clone),
            bounds_guid: identity.guid,
            bounds_createtxg: identity.createtxg,
            entries,
        });
    }
}
