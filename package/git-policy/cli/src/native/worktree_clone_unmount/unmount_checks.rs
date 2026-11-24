//! What:
//!  The identity comparisons the unmount makes before any move:
//!  the reopened slot's mount id,
//!  device,
//!  inode and observed path,
//!  its source,
//!  filesystem type and root,
//!  its parent mount,
//!  the live dataset GUID,
//!  and the canonical lookup's second answer.
//! Why:
//!  Every one of these is a way a mount at the slot can be another mount than
//!  the clone this removal is about,
//!  and the design refuses each rather than unmounting what it finds
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount":
//!  "Compare mount ID from `fdinfo`,
//! device and inode from `fstat`,
//! and the
//!  expected slot,
//! parent mount ID,
//! ZFS root and source in the handler's
//!  `mountinfo`").
//!  The comparison runs before the bind,
//!  so a mismatch makes no kernel mount call at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const checked = verifyTheSlot(request); // throws on the first field that differs
//! ```

use super::slot_walk_calls::{DatasetGuids, SlotWalk};
use super::staging_calls::{StagingCall, StagingCallRefusal, StagingCalls};
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The filesystem type a clone's dataset is attached as.
pub const ZFS_FILE_SYSTEM_TYPE: &str = "zfs";

/// What:
///  The mount the removal expects at the slot,
///  from the receipt,
/// the store or the live table the caller read.
/// Why:
///  Every comparison names a number the caller already believes,
///  so a mount that moved refuses rather than becoming the new truth.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ExpectedMount = { mountId; parentMountId; device; inode; canonicalSlot; source; guid };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedMount {
    /// The clone's mount number.
    pub mount_id: u64,
    /// The mount the slot's parent folder is on.
    pub parent_mount_id: u64,
    /// The clone root's device.
    pub device: u64,
    /// The clone root's inode.
    pub inode: u64,
    /// The slot's canonical path.
    pub canonical_slot: PathBuf,
    /// The clone's dataset source.
    pub source: String,
    /// The clone's dataset GUID.
    pub guid: DatasetGuid,
}

/// What:
///  Why the slot is not the mount the removal expects.
/// Why:
///  A person needs the field that differs,
///  the two values and the mount row that was read,
///  because each sends them to a different check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlotRefusal = { step: string; said: string; observed?: MountFacts };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotRefusal {
    /// The step the refusal names.
    pub step: String,
    /// What differed,
    /// in words.
    pub said: String,
    /// The mount's row when one was read.
    pub observed: Option<MountFacts>,
}

impl SlotRefusal {
    /// One field's mismatch.
    pub fn field(step: &'static str, expected: impl std::fmt::Display, found: impl std::fmt::Display) -> SlotRefusal {
        return SlotRefusal { step: String::from(step), said: format!("the unmount expects {expected}, and the slot reports {found}"), observed: None };
    }

    /// The step's word.
    pub fn step(&self) -> &str {
        return self.step.as_str();
    }
}

/// What:
///  The slot handle and the row a passed check leaves for the move.
/// Why:
///  The handle is the source `move_mount` acts on,
///  and the row is what a receipt reports after a later step refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckedSlot = { target: File; mountId: number; facts: MountFacts };
/// ```
#[derive(Debug)]
pub struct CheckedSlot {
    /// The slot's path handle,
    /// on the clone's mount.
    pub target: std::fs::File,
    /// The clone's mount number.
    pub mount_id: u64,
    /// The clone's row.
    pub facts: MountFacts,
}

/// The row a failed read leaves out of a refusal.
fn observed(calls: &dyn StagingCalls, mount_id: u64) -> Option<MountFacts> {
    return calls.mount_facts(mount_id).ok();
}

/// What:
///  Open the slot and compare every field the removal expects.
/// Why:
///  A refusal here is made before the bind,
///  so nothing is mounted,
/// moved or unbound on a mount that is not the clone's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checked = verifyTheSlot(request);
/// ```
#[allow(clippy::result_large_err, reason = "the refusal carries the mount row a person needs, and the call chain is not hot")]
pub fn verify_the_slot(slot: &str, expected: &ExpectedMount, checkout: &Path, calls: &dyn StagingCalls, walk: &dyn SlotWalk, guids: &dyn DatasetGuids) -> Result<CheckedSlot, SlotRefusal> {
    let target: std::fs::File = walk.open_the_slot(checkout, slot).map_err(|refusal| return walk_refusal(refusal))?;
    let mount_id: u64 = calls.mount_id(&target).map_err(|refusal| return call_refusal(refusal))?;
    if mount_id != expected.mount_id {
        return Err(SlotRefusal { step: String::from("mount-id"), said: format!("the unmount expects mount {}, and the slot's descriptor is on mount {mount_id}", expected.mount_id), observed: observed(calls, mount_id) });
    }
    let facts: std::fs::Metadata = target.metadata().map_err(|error| return SlotRefusal { step: String::from("device"), said: format!("the slot's metadata could not be read: {error}"), observed: None })?;
    if facts.dev() != expected.device {
        return Err(SlotRefusal::field("device", expected.device, facts.dev()));
    }
    if facts.ino() != expected.inode {
        return Err(SlotRefusal::field("inode", expected.inode, facts.ino()));
    }
    let observed_path: PathBuf = calls.observed_path(&target).map_err(|refusal| return call_refusal(refusal))?;
    if observed_path != expected.canonical_slot {
        return Err(SlotRefusal::field("canonical-path", expected.canonical_slot.display(), observed_path.display()));
    }
    let row: MountFacts = calls.mount_facts(mount_id).map_err(|refusal| return call_refusal(refusal))?;
    let mismatch: Option<SlotRefusal> = compare_the_row(&row, expected);
    if let Some(refusal) = mismatch {
        return Err(SlotRefusal { observed: Some(row), ..refusal });
    }
    let guid: DatasetGuid = guids.live_guid(row.source.as_str()).map_err(|said| return SlotRefusal { step: String::from("guid"), said, observed: Some(row.clone()) })?;
    if guid != expected.guid {
        return Err(SlotRefusal { step: String::from("guid"), said: format!("the unmount expects dataset GUID {}, and the pool reports {} for `{}`", expected.guid.file_name(), guid.file_name(), row.source), observed: Some(row) });
    }
    let again: u64 = walk.look_again(checkout, slot).map_err(|refusal| return call_refusal(refusal))?;
    if again != expected.mount_id {
        return Err(SlotRefusal { step: String::from("canonical-lookup"), said: format!("a fresh walk of the canonical slot reaches mount {again}, and the unmount expects mount {}", expected.mount_id), observed: None });
    }
    return Ok(CheckedSlot { target, mount_id, facts: row });
}

/// The source,
/// type,
/// root and parent clauses of one mount row.
pub fn compare_the_row(row: &MountFacts, expected: &ExpectedMount) -> Option<SlotRefusal> {
    if row.source != expected.source {
        return Some(SlotRefusal::field("source", expected.source.as_str(), row.source.as_str()));
    }
    if row.filesystem_type != ZFS_FILE_SYSTEM_TYPE {
        return Some(SlotRefusal::field("filesystem", ZFS_FILE_SYSTEM_TYPE, row.filesystem_type.as_str()));
    }
    if row.root != Path::new("/") {
        return Some(SlotRefusal::field("root", "/", row.root.display()));
    }
    if row.parent_id != expected.parent_mount_id {
        return Some(SlotRefusal::field("parent-mount", expected.parent_mount_id, row.parent_id));
    }
    return None;
}

/// One walk refusal as a slot refusal.
fn walk_refusal(refusal: StagingCallRefusal) -> SlotRefusal {
    return SlotRefusal { step: String::from(refusal.call.word()), said: refusal.message("the slot could not be opened"), observed: None };
}

/// One read call's refusal as a slot refusal.
fn call_refusal(refusal: StagingCallRefusal) -> SlotRefusal {
    return SlotRefusal { step: String::from(refusal.call.word()), said: refusal.message("the unmount's check could not read the slot"), observed: None };
}

/// The step a bind or move failure names,
/// which the turn reuses.
pub const STAGING_FOLDER_STEP: &str = "staging-folder";

/// The step a pin failure names.
pub const PIN_STEP: &str = "user-cache-pin";

/// The step a record failure names.
pub const RECORD_STEP: &str = "staged-record";

/// The step a placement failure names.
pub const PLACEMENT_STEP: &str = "post-move-check";

/// The step an unmount failure names.
pub fn unmount_step() -> &'static str {
    return StagingCall::Unmount.word();
}
