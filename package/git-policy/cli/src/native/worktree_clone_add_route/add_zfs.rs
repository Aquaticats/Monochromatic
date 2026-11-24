//! What:
//!  The route's pool calls: the device opened behind the start-time gate, the
//!  identities of the datasets and the pool, the snapshot, the clone, the two
//!  inventories, and the mount request.
//! Why:
//!  Every call is the binding's own typed request, and no `zfs` program starts
//!  (`package/zfs/request-to-openzfs-kernel-device`;
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Binding crate and compatibility").
//!  A write's outcome is unknown the moment the kernel refuses it, so none is
//!  sent twice and each answer is read back before its identity is recorded
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`, A8 and A9).
//!  The mount request is the protocol's own property write, with the clone's
//!  dataset name and nothing a caller composes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const read = identityOf(device, dataset);
//! const origin = snapshotTheCheckout(device, pool, name, dataset);
//! const clone = cloneTheOrigin(device, target, origin, guid);
//! ```

use crate::doctor_zfs_route::{Grant, grants_of};
use crate::worktree_clone_binding_record::binding_inventory::{EntryKind, InventoryEntry, InventoryKind, InventoryRecord};
use crate::worktree_clone_mount_protocol::clone_mount_intent::{MountIntent, MountOperation};
use crate::worktree_clone_mount_protocol::clone_mount_intent_write::CloneIntentWriter;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::zfs_compatibility_gate::{BeforeRequests, LiveGate, Verdict, verdict_line};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, NvList, ZfsDevice, requests};

/// What:
///  One dataset or snapshot's identity,
///  the two numbers the record writes.
/// Why:
///  A name alone proves nothing about which object the pool holds,
///  so every read-back gives the GUID and the transaction group together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ObjectIdentity = { guid: DatasetGuid; createtxg: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectIdentity {
    /// The object's GUID.
    pub guid: DatasetGuid,
    /// The transaction group that created it.
    pub createtxg: u64,
}

/// What:
///  The device the route makes its requests through, opened after the gate.
/// Why:
///  A refused gate disables every raw request,
///  and the refusal names the file or the difference rather than the add.
pub struct OpenedDevice {
    /// The open handle.
    pub device: ZfsDevice,
}

impl OpenedDevice {
    /// Run the start-time gate, then open the kernel's device.
    ///
    /// # Errors
    /// The gate refused,
    /// or the device cannot be opened.
    pub fn open() -> Result<OpenedDevice, String> {
        let verdict: Verdict = LiveGate.verdict();
        if let Verdict::Refused(_refusal) = &verdict {
            return Err(format!("the ZFS compatibility gate refused before any pool request: {}. Ways forward: run `monochromatic-git-policy-cli doctor` on the machine that holds the pool", verdict_line(&verdict)));
        }
        let device: ZfsDevice = ZfsDevice::open().map_err(|error| return format!("the kernel's ZFS device cannot be opened ({error}), so the clone route cannot serve this add"))?;
        return Ok(OpenedDevice { device });
    }

    /// One dataset's or snapshot's identity, read back from the pool.
    ///
    /// # Errors
    /// The object does not answer,
    /// or its statistics carry no GUID.
    pub fn identity_of(&self, name: &str) -> Result<ObjectIdentity, String> {
        return read_identity(&self.device, name);
    }

    /// Take the origin snapshot of the checkout and read its identity back.
    ///
    /// # Errors
    /// The kernel refused the write or reported a per-name failure,
    /// or the snapshot does not answer afterwards.
    pub fn snapshot_the_checkout(&self, pool: &str, snapshot: &str) -> Result<ObjectIdentity, String> {
        let reply: Option<NvList> = self.device.snapshot(pool, &[snapshot], None).map_err(|error| return format!("the snapshot {snapshot} was refused ({error}); the pool may be full, and the claim, the slot and the registration stand"))?;
        refuse_any_element(reply.as_ref(), "the snapshot")?;
        return read_identity(&self.device, snapshot);
    }

    /// Clone the origin snapshot and read the clone's identity back.
    ///
    /// # Errors
    /// The kernel refused the write or reported a per-name failure,
    /// or the clone does not answer afterwards.
    pub fn clone_the_origin(&self, target: &str, origin: &str) -> Result<ObjectIdentity, String> {
        // A clone inherits the origin's `mountpoint` unless it sets its own, and
        // the design keeps every clone at a legacy mountpoint no automatic mount
        // reaches (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
        // "Dataset and path layout"), so the property is written with the
        // creation. `canmount` is not: the probe saw the container's local
        // `canmount=off` not reach its child, and a legacy dataset is never
        // mounted by itself whatever it says.
        let mut properties: NvList = NvList::new();
        properties.add_str("mountpoint", "legacy");
        let reply: Option<NvList> = self.device.clone(target, origin, Some(&properties)).map_err(|error| return format!("the clone {target} was refused ({error}); the pool may be full, and the origin snapshot's identity is recorded"))?;
        refuse_any_element(reply.as_ref(), "the clone")?;
        return read_identity(&self.device, target);
    }

    /// The origin dataset's snapshots at or below `bounds_createtxg`, and its
    /// bookmarks, as the origin inventory's set.
    ///
    /// # Errors
    /// A listing step failed,
    /// or a bookmark carries no GUID or creation transaction group.
    pub fn origin_inventory(&self, dataset: &str, pool_guid: DatasetGuid, dataset_guid: DatasetGuid, bounds: &str, bounds_identity: ObjectIdentity) -> Result<InventoryRecord, String> {
        let mut entries: Vec<InventoryEntry> = Vec::new();
        let mut cursor: u64 = 0;
        while let Some(object) = self.device.snapshot_list_next(dataset, cursor).map_err(|error| return format!("the snapshots of {dataset} could not be listed ({error})"))? {
            cursor = object.cursor;
            let createtxg: u64 = object.stats.dds_creation_txg;
            if createtxg > bounds_identity.createtxg {
                continue;
            }
            let name: String = short_name(object.name.as_str(), dataset);
            entries.push(InventoryEntry { kind: EntryKind::Snapshot, guid: guid_of(object.stats.dds_guid, object.name.as_str())?, createtxg, name });
        }
        let bookmarks: NvList = self.device.get_bookmarks(dataset, &[]).map_err(|error| return format!("the bookmarks of {dataset} could not be listed ({error})"))?;
        for pair in bookmarks.iter() {
            let monochromatic_zfs_request_to_openzfs_kernel_device::NvData::List { value } = &pair.data else { continue };
            let guid: u64 = nested_u64(value, "guid").ok_or_else(|| return format!("the bookmark {} carries no GUID", pair.name))?;
            let createtxg: u64 = nested_u64(value, "createtxg").ok_or_else(|| return format!("the bookmark {} carries no creation transaction group", pair.name))?;
            entries.push(InventoryEntry { kind: EntryKind::Bookmark, guid: guid_of(guid, pair.name.as_str())?, createtxg, name: pair.name.clone() });
        }
        return Ok(InventoryRecord { kind: InventoryKind::Origin, pool_guid, dataset: String::from(dataset), dataset_guid, bounds: String::from(bounds), bounds_guid: bounds_identity.guid, bounds_createtxg: bounds_identity.createtxg, entries });
    }

    /// The clone's own snapshots and bookmarks immediately after it was made.
    ///
    /// # Errors
    /// A listing step failed,
    /// or a bookmark carries no GUID or creation transaction group.
    pub fn clone_inventory(&self, clone: &str, pool_guid: DatasetGuid, identity: ObjectIdentity) -> Result<InventoryRecord, String> {
        let mut entries: Vec<InventoryEntry> = Vec::new();
        let mut cursor: u64 = 0;
        while let Some(object) = self.device.snapshot_list_next(clone, cursor).map_err(|error| return format!("the clone's snapshots could not be listed ({error})"))? {
            cursor = object.cursor;
            entries.push(InventoryEntry { kind: EntryKind::Snapshot, guid: guid_of(object.stats.dds_guid, object.name.as_str())?, createtxg: object.stats.dds_creation_txg, name: short_name(object.name.as_str(), clone) });
        }
        return Ok(InventoryRecord { kind: InventoryKind::Clone, pool_guid, dataset: String::from(clone), dataset_guid: identity.guid, bounds: String::from(clone), bounds_guid: identity.guid, bounds_createtxg: identity.createtxg, entries });
    }

    /// The origin snapshot a clone dataset came from,
    /// when it is a clone.
    ///
    /// # Errors
    /// The dataset does not answer the pool.
    pub fn origin_of(&self, name: &str) -> Result<Option<String>, String> {
        let identity: DatasetIdentity = self.device.objset_stats(name).map_err(|error| return format!("{name} does not answer the pool ({error})"))?;
        return Ok(Some(identity.stats.origin()).filter(|origin| return !origin.is_empty()));
    }

    /// One dataset's delegations and its ancestors',
    /// as the claim's own check reads them.
    ///
    /// # Errors
    /// The pool refused the read,
    /// or answered a shape this reader cannot represent.
    pub fn grants_at(&self, dataset: &str) -> Result<Vec<Grant>, String> {
        let reply: NvList = self.device.get_fsacl(dataset).map_err(|error| return format!("the delegations of {dataset} cannot be read ({error})"))?;
        return grants_of(&reply).map_err(|reason| return format!("the delegations of {dataset} are not a shape this reader represents ({reason})"));
    }

    /// Ask the handler for the clone's mount.
    ///
    /// # Errors
    /// The gate refused again,
    /// the device cannot be reopened,
    /// or the property write was refused.
    pub fn request_the_mount(&self, clone: &str, nonce: &RequestNonce) -> Result<(), String> {
        let writer: CloneIntentWriter = CloneIntentWriter::open(&LiveGate, clone).map_err(|refusal| return format!("the mount request cannot be written ({refusal:?})"))?;
        return writer.request(&MountIntent { operation: MountOperation::Mount, nonce: nonce.clone() }).map_err(|refusal| return format!("the mount request for {clone} was refused ({refusal:?}); the clone and its records stand, and the next sweep may still act on it"));
    }
}

/// One dataset's identity from the pool's statistics.
fn read_identity(device: &ZfsDevice, name: &str) -> Result<ObjectIdentity, String> {
    let identity: DatasetIdentity = device.objset_stats(name).map_err(|error| return format!("{name} does not answer the pool ({error})"))?;
    return Ok(ObjectIdentity { guid: guid_of(identity.stats.dds_guid, name)?, createtxg: identity.stats.dds_creation_txg });
}

/// A GUID the kernel reported, refused when it is zero,
/// which no live object answers.
fn guid_of(value: u64, name: &str) -> Result<DatasetGuid, String> {
    if value == 0 {
        return Err(format!("the pool reported GUID 0 for {name}, and no live object has that identity"));
    }
    return DatasetGuid::parse(value.to_string().as_str()).map_err(|refusal| return format!("the pool's GUID for {name} is not canonical ({refusal:?})"));
}

/// The name one listing reports without its dataset prefix.
fn short_name(full: &str, dataset: &str) -> String {
    if let Some((prefix, rest)) = full.split_once('@')
        && prefix == dataset
    {
        return String::from(rest);
    }
    if let Some((prefix, rest)) = full.split_once('#')
        && prefix == dataset
    {
        return String::from(rest);
    }
    return String::from(full);
}

/// A property nested as `{value}` in a bookmark's reply.
fn nested_u64(entry: &NvList, key: &str) -> Option<u64> {
    return entry.get_list(key).and_then(|value| return value.get_u64("value"));
}

/// Refuse when a write's reply carries a per-name failure,
/// naming each failed name and the kernel's error.
fn refuse_any_element(reply: Option<&NvList>, what: &str) -> Result<(), String> {
    let Some(found_reply) = reply else { return Ok(()) };
    let failures: Vec<requests::ElementError> = requests::element_errors(found_reply);
    if failures.is_empty() {
        return Ok(());
    }
    let shown: Vec<String> = failures.iter().map(|failure| return failure.to_string()).collect();
    return Err(format!("{what} was refused for {}; nothing was retried. Ways forward: free space in the pool or inspect the objects, then run the add again", shown.join(", ")));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_zfs_tests.rs"]
mod tests;
