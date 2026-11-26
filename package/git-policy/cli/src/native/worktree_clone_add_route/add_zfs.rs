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

use crate::doctor_zfs_route::zfs_checks::LEGACY_MOUNTPOINT;
use crate::doctor_zfs_route::{
    Grant,
    grants_of,
};
use crate::worktree_clone_binding_record::binding_inventory::{
    EntryKind,
    InventoryEntry,
    InventoryKind,
    InventoryRecord,
};
use crate::worktree_clone_managed_lock::dataset_guid_reads::DatasetGuidReads;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts,
    MOUNTPOINT_PROPERTY,
    PropertyValue,
    facts_of as facts_from_the_reply,
};
use crate::worktree_clone_mount_protocol::clone_mount_intent::{
    MountIntent,
    MountOperation,
};
use crate::worktree_clone_mount_protocol::clone_mount_intent_write::CloneIntentWriter;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::local_user_property::{
    PropertyReading,
    read_local_property,
};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::zfs_compatibility_gate::{
    BeforeRequests,
    LiveGate,
    Verdict,
    verdict_line,
};
use monochromatic_zfs_request_to_openzfs_kernel_device::{
    DatasetIdentity,
    NvList,
    ZfsDevice,
    requests,
};

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
///  What one clone request answered:
///  the identity the record keeps,
///  and the properties the add reads to report a clone it cannot use.
/// Why:
///  The kernel's own reply for the clone carries every property with its value
///  and its source,
///  so the add judges the clone without asking the pool a second question
///  (`doc/decision/cli-git-zfs-clone-inherits-legacy-from-the-container.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneMade = { identity: ObjectIdentity; facts: CloneObjectFacts };
/// ```
pub struct CloneMade {
    /// The clone's own identity, which the record writes.
    pub identity: ObjectIdentity,
    /// The clone's properties, read from the same reply.
    pub facts: CloneObjectFacts,
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

/// What:
///  Where the preflight's device comes from:
///  one call that answers the opened device or the sentence saying why it did not,
///  taken as a value so a control answers it without this machine's `/dev/zfs`.
/// Why:
///  Every host control of the preflight reached the real device and opened it
///  (the alpha.47 release pass's Low 4),
///  and the removal refusal's own controls stopped doing that when the reader
///  became a parameter (`b2ee59cff6`),
///  so this is the same seam for the second raw request the add makes.
///  A build of this crate without the test-only switches reads only the live answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TheDeviceOpened { open(): OpenedDevice }  // throws the refusal's sentence
/// ```
pub trait TheDeviceOpened {
    /// Run the start-time gate,
    /// then open the kernel's device,
    /// or answer the sentence saying which of the two refused.
    ///
    /// # Throws
    /// The gate's refusal,
    /// or the device's own.
    fn open(&self) -> Result<OpenedDevice, String>;
}

/// What:
///  The release's own answer to `TheDeviceOpened`:
///  the compatibility gate,
///  then the kernel's device.
/// Why:
///  The route's run hands this in,
///  and a control hands in an answer of its own,
///  so the release opens the device exactly where it always did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theLiveDevice: TheDeviceOpened = { open: () => OpenedDevice.open() };
/// ```
pub struct TheLiveDevice;

impl TheDeviceOpened for TheLiveDevice {
    fn open(&self) -> Result<OpenedDevice, String> {
        return OpenedDevice::open();
    }
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
            return Err(format!(
                "the ZFS compatibility gate refused before any pool request: {}. Ways forward: read what the gate refused above, fix it on the machine that holds the pool, and run the command again",
                verdict_line(&verdict)
            ));
        }
        let device: ZfsDevice = ZfsDevice::open().map_err(|error| return format!("the kernel's ZFS device cannot be opened ({error}), so the clone route cannot serve this add"))?;
        return Ok(OpenedDevice {
            device,
        });
    }

    /// One dataset's or snapshot's identity, read back from the pool.
    ///
    /// # Errors
    /// The object does not answer,
    /// or its statistics carry no GUID.
    pub fn identity_of(
        &self,
        name: &str,
    ) -> Result<ObjectIdentity, String> {
        return read_identity(&self.device, name);
    }

    /// Take the origin snapshot of the checkout and read its identity back.
    ///
    /// # Errors
    /// The kernel refused the write or reported a per-name failure,
    /// or the snapshot does not answer afterwards.
    pub fn snapshot_the_checkout(
        &self,
        pool: &str,
        snapshot: &str,
    ) -> Result<ObjectIdentity, String> {
        let reply: Option<NvList> = self.device.snapshot(pool, &[snapshot], None).map_err(|error| return format!("the snapshot {snapshot} was refused ({error}); the pool may be full, and the claim, the slot and the registration stand"))?;
        refuse_any_element(reply.as_ref(), "the snapshot")?;
        return read_identity(&self.device, snapshot);
    }

    /// One dataset's properties as the kernel's reply for it carries them.
    ///
    /// # Errors
    /// The dataset does not answer the pool.
    pub fn facts_of(
        &self,
        name: &str,
    ) -> Result<CloneObjectFacts, String> {
        let identity: DatasetIdentity = self
            .device
            .objset_stats(name)
            .map_err(|error| return format!("{name} does not answer the pool ({error})"))?;
        return Ok(facts_from_the_reply(name, &identity));
    }

    /// Clone the origin snapshot, read the clone's identity back,
    /// and keep the properties that reply carries.
    ///
    /// # Errors
    /// The kernel refused the write or reported a per-name failure,
    /// or the clone does not answer afterwards.
    pub fn clone_the_origin(
        &self,
        target: &str,
        origin: &str,
    ) -> Result<CloneMade, String> {
        // No property is sent with the request. A clone inherits its parent's
        // `mountpoint`, which is the clone container and not the origin
        // (`doc/planning/cli-git-zfs-clone-mountpoint-without-the-forbidden-grant.md`,
        // "Where a clone's properties actually come from"), and the accepted
        // design keeps the container at `mountpoint=legacy`
        // (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
        // "Dataset and path layout"). Sending `mountpoint` needs a delegation
        // the design forbids this account, so the request it carries is refused
        // (`doc/decision/cli-git-zfs-clone-inherits-legacy-from-the-container.md`).
        let reply: Option<NvList> = self.device.clone(target, origin, None).map_err(|error| return format!("the clone {target} was refused ({error}); the pool may be full, and the origin snapshot's identity is recorded"))?;
        refuse_any_element(reply.as_ref(), "the clone")?;
        return read_the_clone(&self.device, target);
    }

    /// The origin dataset's snapshots at or below `bounds_createtxg`, and its
    /// bookmarks, as the origin inventory's set.
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

    /// The origin snapshot a clone dataset came from,
    /// when it is a clone.
    ///
    /// # Errors
    /// The dataset does not answer the pool.
    pub fn origin_of(
        &self,
        name: &str,
    ) -> Result<Option<String>, String> {
        let identity: DatasetIdentity = self
            .device
            .objset_stats(name)
            .map_err(|error| return format!("{name} does not answer the pool ({error})"))?;
        return Ok(Some(identity.stats.origin()).filter(|origin| return !origin.is_empty()));
    }

    /// One dataset's delegations and its ancestors',
    /// as the claim's own check reads them.
    ///
    /// # Errors
    /// The pool refused the read,
    /// or answered a shape this reader cannot represent.
    pub fn grants_at(
        &self,
        dataset: &str,
    ) -> Result<Vec<Grant>, String> {
        let reply: NvList = self.device.get_fsacl(dataset).map_err(|error| {
            return format!("the delegations of {dataset} cannot be read ({error})");
        })?;
        return grants_of(&reply).map_err(|reason| {
            return format!(
                "the delegations of {dataset} are not a shape this reader represents ({reason})"
            );
        });
    }

    /// Ask the handler for the clone's mount.
    ///
    /// # Errors
    /// The gate refused again,
    /// the device cannot be reopened,
    /// or the property write was refused.
    pub fn request_the_mount(
        &self,
        clone: &str,
        nonce: &RequestNonce,
    ) -> Result<(), String> {
        let writer: CloneIntentWriter =
            CloneIntentWriter::open(&LiveGate, clone).map_err(|refusal| {
                return format!("the mount request cannot be written ({refusal:?})");
            })?;
        return writer.request(&MountIntent { operation: MountOperation::Mount, nonce: nonce.clone() }).map_err(|refusal| return format!("the mount request for {clone} was refused ({refusal:?}); the clone and its records stand, and the next sweep may still act on it"));
    }
}

// The route's identity read,
// which the managed lock's mounted arm compares a record's `cloneGuid=` with:
// one reader for one question,
// asked the way the add's own read-backs ask it,
// through the device this command already opened behind the gate.
impl DatasetGuidReads for OpenedDevice {
    fn guid_of(
        &self,
        dataset: &str,
    ) -> Result<DatasetGuid, String> {
        return self
            .identity_of(dataset)
            .map(|identity| return identity.guid);
    }
}

/// One dataset's identity from the pool's statistics.
fn read_identity(
    device: &ZfsDevice,
    name: &str,
) -> Result<ObjectIdentity, String> {
    let identity: DatasetIdentity = device
        .objset_stats(name)
        .map_err(|error| return format!("{name} does not answer the pool ({error})"))?;
    return Ok(ObjectIdentity {
        guid: guid_of(identity.stats.dds_guid, name)?,
        createtxg: identity.stats.dds_creation_txg,
    });
}

/// What:
///  A clone's identity and its properties from the one reply that follows the
///  clone request.
/// Why:
///  The record needs the identity and the add's report needs the properties,
///  and the kernel gave both in this reply,
///  so the clone costs one read rather than two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made: CloneMade = { identity: identityOf(reply), facts: factsOf(name, reply) };
/// ```
fn read_the_clone(
    device: &ZfsDevice,
    name: &str,
) -> Result<CloneMade, String> {
    let identity: DatasetIdentity = device
        .objset_stats(name)
        .map_err(|error| return format!("{name} does not answer the pool ({error})"))?;
    let object: ObjectIdentity = ObjectIdentity {
        guid: guid_of(identity.stats.dds_guid, name)?,
        createtxg: identity.stats.dds_creation_txg,
    };
    return Ok(CloneMade {
        identity: object,
        facts: facts_from_the_reply(name, &identity),
    });
}

/// What:
///  The sentence for a clone whose `mountpoint` is not `legacy` inherited from
///  the container,
///  or nothing when it is.
/// Why:
///  The design has the add report such a clone and destroy nothing
///  (`doc/decision/cli-git-zfs-clone-inherits-legacy-from-the-container.md`),
///  and a clone whose property moved is not one this route may mount.
///  The value and the source are read from the reply the pool already gave,
///  so a person learns which dataset set the value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failure = theClonesMountpointFailure(facts, container); // string | undefined
/// ```
pub fn the_clones_mountpoint_failure(
    facts: &CloneObjectFacts,
    container: &str,
) -> Option<String> {
    let value: PropertyValue = facts.mountpoint.clone();
    let source: PropertyReading<'_> = read_local_property(
        Some(&facts.properties),
        facts.name.as_str(),
        MOUNTPOINT_PROPERTY,
    );
    let required: String = format!(
        "a clone must read `{MOUNTPOINT_PROPERTY}` `{LEGACY_MOUNTPOINT}` inherited from its container `{container}`"
    );
    return match &source {
        PropertyReading::Inherited {
            from,
        } if from == container && value.text() == Some(LEGACY_MOUNTPOINT) => None,
        PropertyReading::Inherited {
            from,
        } => Some(format!(
            "the clone `{}`: {}, inherited from `{from}`, and {required}",
            facts.name,
            value.describe(MOUNTPOINT_PROPERTY)
        )),
        PropertyReading::Local {
            ..
        } => Some(format!(
            "the clone `{}`: {}, set on the clone itself, and {required}",
            facts.name,
            value.describe(MOUNTPOINT_PROPERTY)
        )),
        PropertyReading::Absent => Some(format!(
            "the clone `{}` answers no `{MOUNTPOINT_PROPERTY}` property, and {required}",
            facts.name
        )),
        PropertyReading::Malformed(refusal) => Some(refusal.message(MOUNTPOINT_PROPERTY)),
    };
}

/// A GUID the kernel reported, refused when it is zero,
/// which no live object answers.
fn guid_of(
    value: u64,
    name: &str,
) -> Result<DatasetGuid, String> {
    if value == 0 {
        return Err(format!(
            "the pool reported GUID 0 for {name}, and no live object has that identity"
        ));
    }
    return DatasetGuid::parse(value.to_string().as_str()).map_err(|refusal| {
        return format!("the pool's GUID for {name} is not canonical ({refusal:?})");
    });
}

/// The name one listing reports without its dataset prefix.
fn short_name(
    full: &str,
    dataset: &str,
) -> String {
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
fn nested_u64(
    entry: &NvList,
    key: &str,
) -> Option<u64> {
    return entry
        .get_list(key)
        .and_then(|value| return value.get_u64("value"));
}

/// Refuse when a write's reply carries a per-name failure,
/// naming each failed name and the kernel's error.
fn refuse_any_element(
    reply: Option<&NvList>,
    what: &str,
) -> Result<(), String> {
    let Some(found_reply) = reply else {
        return Ok(());
    };
    let failures: Vec<requests::ElementError> = requests::element_errors(found_reply);
    if failures.is_empty() {
        return Ok(());
    }
    let shown: Vec<String> = failures
        .iter()
        .map(|failure| return failure.to_string())
        .collect();
    return Err(format!(
        "{what} was refused for {}; nothing was retried. Ways forward: free space in the pool or inspect the objects, then run the add again",
        shown.join(", ")
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_zfs_tests.rs"]
mod tests;
