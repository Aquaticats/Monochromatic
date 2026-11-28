//! What:
//!  The route's pool requests:
//!  the device opened behind the start-time gate,
//!  the identity read,
//!  the snapshot,
//!  the clone,
//!  the origin and the delegations,
//!  and the mount request.
//! Why:
//!  Every call is the binding's own typed request,
//!  and no `zfs` program starts
//!  (`package/zfs/request-to-openzfs-kernel-device`;
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Binding crate and compatibility").
//!  A refused gate disables every raw request,
//!  and the device is opened once
//!  behind it,
//!  so a control hands in the gate's answer instead of this machine's
//!  `/dev/zfs`.
//!  `add_zfs_pool_answers.rs` reads what the pool answered and
//!  `add_zfs_inventories.rs` lists what an origin and a clone hold,
//!  so this file is the device and the requests it sends.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const device = OpenedDevice.open(); // the gate, then /dev/zfs
//! const origin = device.snapshotTheCheckout(pool, name);
//! const clone = device.cloneTheOrigin(target, origin);
//! ```

use super::add_zfs_pool_answers::{
    read_identity,
    read_the_clone,
    refuse_any_element,
};
use crate::doctor_zfs_route::{
    Grant,
    grants_of,
};
use crate::worktree_clone_managed_lock::dataset_guid_reads::DatasetGuidReads;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::{
    CloneObjectFacts,
    facts_of as facts_from_the_reply,
};
use crate::worktree_clone_mount_protocol::clone_mount_intent::{
    MountIntent,
    MountOperation,
};
use crate::worktree_clone_mount_protocol::clone_mount_intent_write::CloneIntentWriter;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
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
};

// The pool's GUID check, which the controls of `add_zfs_tests.rs` read through
// this file; the release reads it through `read_identity` in the answers' own file.
#[cfg(test)]
use super::add_zfs_pool_answers::guid_of;

/// What:
///  One dataset or snapshot's identity,
///  the two numbers the record writes.
/// Why:
///  Every read-back answers the same two numbers,
///  so the record and its readers
///  name them once.
pub use super::add_zfs_pool_answers::ObjectIdentity;

/// What:
///  What one clone request answered:
///  the identity the record keeps,
///  and the properties the add reads to report a clone it cannot use.
/// Why:
///  The kernel's own reply for the clone carries both,
///  so one request answers
///  what the record and the add's report need.
pub use super::add_zfs_pool_answers::CloneMade;

/// The sentence for a clone whose mountpoint did not stay the container's.
pub use super::add_zfs_pool_answers::the_clones_mountpoint_failure;

/// What:
///  The device the route makes its requests through,
///  opened after the gate.
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
    /// Run the start-time gate,
    /// then open the kernel's device.
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

    /// One dataset's or snapshot's identity,
    /// read back from the pool.
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

    /// Clone the origin snapshot,
    /// read the clone's identity back,
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

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_zfs_tests.rs"]
mod tests;
