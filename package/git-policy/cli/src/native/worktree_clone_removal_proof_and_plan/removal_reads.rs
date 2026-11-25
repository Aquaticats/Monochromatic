//! What:
//!  The pool reads a removal's lineage proof makes,
//!  as one seam with one live implementation over the route's binding.
//! Why:
//!  The proof reads one dataset's identity and properties,
//!  its snapshot list and its bookmark list,
//!  and nothing else,
//!  so a control can answer all three without a pool and the live answer is the binding's own
//!  typed requests.
//!  Every read is the binding's,
//!  behind the start-time compatibility gate,
//!  so no `zfs` program starts between the proof and the kernel
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Binding crate and compatibility").
//!  The add route reads the same lists,
//!  and this module writes its own two readers because the add's are private to it and the removal
//!  needs the owning dataset of every object,
//! which the add's inventory of a single dataset does not
//!  carry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reads = LiveRemovalReads.open();
//! const identity = reads.identity(clone);
//! const inventory = reads.inventory(clone, guid);
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::removal_inventory::{LiveInventory, LiveObject};
use crate::zfs_compatibility_gate::{BeforeRequests, LiveGate, Verdict, verdict_line};
use monochromatic_zfs_request_to_openzfs_kernel_device::{DatasetIdentity, NvData, NvList, ZfsDevice};

/// What:
///  The three reads one removal's proof makes of the pool.
/// Why:
///  The proof judges values and never a live system,
///  so a control answers these three and the release answers them with the binding's requests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RemovalPoolReads { identity(name); snapshotsOf(dataset, guid); bookmarksOf(dataset, guid); }
/// ```
pub trait RemovalPoolReads {
    /// One dataset's or snapshot's identity and property reply.
    ///
    /// # Throws
    ///
    /// What the name that does not answer says.
    fn identity(&self, name: &str) -> Result<DatasetIdentity, String>;

    /// Every snapshot of `dataset`,
    /// each with the dataset it is listed under.
    ///
    /// # Throws
    ///
    /// The listing step that failed,
    /// which makes the inventory incomplete and therefore a refusal.
    fn snapshots_of(&self, dataset: &str, dataset_guid: DatasetGuid) -> Result<Vec<LiveObject>, String>;

    /// Every bookmark of `dataset`,
    /// each with its owning dataset.
    ///
    /// # Throws
    ///
    /// The bookmark read that failed,
    /// or one bookmark whose reply carries no GUID or creation group.
    fn bookmarks_of(&self, dataset: &str, dataset_guid: DatasetGuid) -> Result<Vec<LiveObject>, String>;
}

/// What:
///  The release answer:
///  the binding's open `/dev/zfs` handle.
/// Why:
///  The removal runs as the account and reads through the same typed requests the add does,
///  after the gate that refuses a module this build does not know.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reads = new LiveRemovalReads(ZfsDevice.open());
/// ```
pub struct LiveRemovalReads {
    device: ZfsDevice,
}

impl LiveRemovalReads {
    /// Run the start-time gate,
    /// then open the device.
    ///
    /// # Errors
    /// The gate refused,
    /// or the device cannot be opened.
    pub fn open() -> Result<LiveRemovalReads, String> {
        let verdict: Verdict = LiveGate.verdict();
        if let Verdict::Refused(_refusal) = &verdict {
            return Err(format!(
                "the ZFS compatibility gate refused before any pool request: {}. Ways forward: run `monochromatic-git-policy-cli doctor` on the machine that holds the pool",
                verdict_line(&verdict)
            ));
        }
        let device: ZfsDevice = ZfsDevice::open().map_err(|error| return format!("the kernel's ZFS device cannot be opened ({error}), so the removal cannot prove its lineage"))?;
        return Ok(LiveRemovalReads { device });
    }
}

impl RemovalPoolReads for LiveRemovalReads {
    fn identity(&self, name: &str) -> Result<DatasetIdentity, String> {
        return self.device.objset_stats(name).map_err(|error| return format!("{name} does not answer the pool ({error})"));
    }

    fn snapshots_of(&self, dataset: &str, dataset_guid: DatasetGuid) -> Result<Vec<LiveObject>, String> {
        let mut objects: Vec<LiveObject> = Vec::new();
        let mut cursor: u64 = 0;
        while let Some(listed) = self.device.snapshot_list_next(dataset, cursor).map_err(|error| return format!("the snapshots of {dataset} could not be listed ({error})"))? {
            cursor = listed.cursor;
            objects.push(LiveObject {
                kind: EntryKind::Snapshot,
                name: name_below(listed.name.as_str(), dataset, '@')?,
                guid: guid_of(listed.stats.dds_guid, listed.name.as_str())?,
                createtxg: listed.stats.dds_creation_txg,
                owning_dataset: String::from(dataset),
                owning_dataset_guid: dataset_guid,
            });
        }
        return Ok(objects);
    }

    fn bookmarks_of(&self, dataset: &str, dataset_guid: DatasetGuid) -> Result<Vec<LiveObject>, String> {
        let reply: NvList = self.device.get_bookmarks(dataset, &[]).map_err(|error| return format!("the bookmarks of {dataset} could not be listed ({error})"))?;
        let mut objects: Vec<LiveObject> = Vec::new();
        for pair in reply.iter() {
            let NvData::List { value } = &pair.data else { continue };
            let guid: u64 = nested_u64(value, "guid").ok_or_else(|| return format!("the bookmark {} carries no GUID", pair.name))?;
            let createtxg: u64 = nested_u64(value, "createtxg").ok_or_else(|| return format!("the bookmark {} carries no creation transaction group", pair.name))?;
            objects.push(LiveObject {
                kind: EntryKind::Bookmark,
                name: name_below(pair.name.as_str(), dataset, '#')?,
                guid: guid_of(guid, pair.name.as_str())?,
                createtxg,
                owning_dataset: String::from(dataset),
                owning_dataset_guid: dataset_guid,
            });
        }
        return Ok(objects);
    }
}

/// What:
///  One dataset's whole live inventory:
///  its snapshots and its bookmarks in one list.
/// Why:
///  The proof reads one list and judges it whole,
///  and a bookmark the clone carries travels with its snapshots.
///
/// # Throws
///
/// The read that failed,
/// which makes the inventory incomplete and therefore a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inventory = wholeInventory(reads, clone, guid);
/// ```
pub fn whole_inventory(reads: &dyn RemovalPoolReads, dataset: &str, dataset_guid: DatasetGuid) -> Result<LiveInventory, String> {
    let mut objects: Vec<LiveObject> = reads.snapshots_of(dataset, dataset_guid)?;
    objects.extend(reads.bookmarks_of(dataset, dataset_guid)?);
    return Ok(LiveInventory { dataset: String::from(dataset), dataset_guid, objects });
}

/// The name one listing reports below its dataset,
/// refused when it is not that dataset's object.
fn name_below(full: &str, dataset: &str, separator: char) -> Result<String, String> {
    let Some((owner, name)) = full.split_once(separator) else {
        return Err(format!("the pool listed `{full}`, which is not a {separator} object"));
    };
    if owner != dataset {
        return Err(format!("the pool listed `{full}` under {dataset}, and its owner is {owner}"));
    }
    return Ok(String::from(name));
}

/// What:
///  One property of a bookmark's reply as its number.
/// Why:
///  The kernel answers each bookmark with a property list whose entries are themselves lists,
///  so the number sits under `value` and a reader that looked for it one level up would find nothing and
///  read a live bookmark as a damaged reply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const number = (entry, key) => entry.getList(key)?.getU64('value');
/// ```
fn nested_u64(entry: &NvList, key: &str) -> Option<u64> {
    return entry.get_list(key).and_then(|value| return value.get_u64("value"));
}

/// A GUID the kernel reported,
/// refused when it is zero,
/// which no live object answers.
fn guid_of(value: u64, name: &str) -> Result<DatasetGuid, String> {
    if value == 0 {
        return Err(format!("the pool reported GUID 0 for {name}, and no live object has that identity"));
    }
    return DatasetGuid::parse(value.to_string().as_str()).map_err(|_| return format!("the pool's GUID for {name} is not canonical"));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_reads_tests.rs"]
mod tests;
