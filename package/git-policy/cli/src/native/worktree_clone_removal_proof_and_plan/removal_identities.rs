//! What:
//!  What the add recorded for one slot,
//!  read by the slot's name,
//!  as the identities a removal proves itself against.
//! Why:
//!  The plan copies the clone's and the origin's identities and refers to the binding for the
//!  origin-side inventory,
//! and no record wins when they disagree:
//!  the removal refuses before the first delete and names the three readings
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "6.
//! The binding record and the removal intent",
//!  "What is copied,
//! and what is referred to" and "When they disagree").
//!  The key is the slot's name,
//!  the last component of the slot path the binding holds,
//!  and the five origin fields are compared with the plan's and with the live pool.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const recorded = readTheRecordedBinding(commonDir, 'fix-flaky-test');
//! expect(recorded.cloneGuid).toBe(7828944690444299989n);
//! ```

use crate::subvolume::fs_beneath::ObjectId;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::binding_inventory::{
    InventoryKind,
    InventoryRecord,
};
use crate::worktree_clone_binding_record::record_publication::read_binding;
use crate::worktree_clone_binding_record::record_publication::read_inventory;
use crate::worktree_clone_binding_record::registration_home::home_folder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The identities one add recorded for one slot,
///  as a removal reads them.
/// Why:
///  A removal needs the clone's name,
///  GUID and group,
///  the origin dataset's and origin snapshot's,
///  the slot's own path,
///  and the registration's name and identity,
///  and it needs all of them before it reads the pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Recorded = { slot, slotPath, cloneDataset, cloneGuid, cloneCreatetxg, originDataset, originDatasetGuid, originSnapshot, originSnapshotGuid, originSnapshotCreatetxg, registrationId, registrationIdentity };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedBinding {
    /// The slot's name,
    /// which is the record's key.
    pub slot: String,
    /// The slot's path,
    /// whose bytes the plan records as `targetHex`.
    pub slot_path: PathBuf,
    /// The pool the clone lives in,
    /// which is the dataset whose GUID the first check proves.
    pub pool: String,
    /// The pool's GUID as the add recorded it.
    pub pool_guid: DatasetGuid,
    /// The clone's full dataset name.
    pub clone_dataset: String,
    /// The clone's GUID.
    pub clone_guid: DatasetGuid,
    /// The clone's own creation transaction group.
    pub clone_createtxg: u64,
    /// The origin dataset's full name.
    pub origin_dataset: String,
    /// The origin dataset's GUID.
    pub origin_dataset_guid: DatasetGuid,
    /// The origin snapshot's full name.
    pub origin_snapshot: String,
    /// The origin snapshot's GUID.
    pub origin_snapshot_guid: DatasetGuid,
    /// The origin snapshot's creation transaction group.
    pub origin_snapshot_createtxg: u64,
    /// The registration's directory name under Git's `worktrees` folder.
    pub registration_id: Vec<u8>,
    /// The registration directory's identity as the add recorded it,
    /// which carries no device number because a record outlives a boot.
    pub registration_identity: FolderIdentity,
}

impl RecordedBinding {
    /// The five origin fields,
    /// in the order a refusal prints them.
    pub fn origin_fields(&self) -> [(&'static str, String); 5] {
        return [
            ("originDataset", self.origin_dataset.clone()),
            ("originDatasetGuid", self.origin_dataset_guid.file_name()),
            ("originSnapshot", self.origin_snapshot.clone()),
            ("originSnapshotGuid", self.origin_snapshot_guid.file_name()),
            (
                "originSnapshotCreatetxg",
                self.origin_snapshot_createtxg.to_string(),
            ),
        ];
    }
}

/// What:
///  Read one slot's binding record as a removal reads it.
/// Why:
///  A field this removal compares is required,
///  and a record that never reached the clone cannot carry one:
///  the binding's own reader requires each key at its phase and refuses a field ahead of its phase,
///  so a record of an earlier phase reaches this read as one missing the field it names
///
/// # Throws
/// The record that cannot be read,
/// or the field it lacks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = readTheRecordedBinding(commonDir, slot);
/// ```
pub fn read_the_recorded_binding(
    common_dir: &Path,
    slot: &str,
) -> Result<RecordedBinding, RemovalStop> {
    let home: PathBuf = home_folder(common_dir);
    let record = read_binding(home.as_path(), slot).map_err(|refusal| {
        return RemovalStop::NoBinding {
            slot: String::from(slot),
            reason: format!("{refusal:?}"),
        };
    })?;
    let clone_dataset: String = record.clone_dataset.clone();
    let clone_guid: DatasetGuid = record.clone_guid.ok_or_else(|| {
        return RemovalStop::BindingIncomplete {
            slot: String::from(slot),
            field: "cloneGuid",
        };
    })?;
    let clone_createtxg: u64 = record.clone_createtxg.ok_or_else(|| {
        return RemovalStop::BindingIncomplete {
            slot: String::from(slot),
            field: "cloneCreatetxg",
        };
    })?;
    let origin_dataset: String = record.origin_dataset.clone().ok_or_else(|| {
        return RemovalStop::BindingIncomplete {
            slot: String::from(slot),
            field: "originDataset",
        };
    })?;
    let origin_dataset_guid: DatasetGuid = record.origin_dataset_guid.ok_or_else(|| {
        return RemovalStop::BindingIncomplete {
            slot: String::from(slot),
            field: "originDatasetGuid",
        };
    })?;
    let origin_snapshot: String = record.origin_snapshot.clone();
    let origin_snapshot_guid: DatasetGuid = record.origin_snapshot_guid.ok_or_else(|| {
        return RemovalStop::BindingIncomplete {
            slot: String::from(slot),
            field: "originSnapshotGuid",
        };
    })?;
    let origin_snapshot_createtxg: u64 = record.origin_snapshot_createtxg.ok_or_else(|| {
        return RemovalStop::BindingIncomplete {
            slot: String::from(slot),
            field: "originSnapshotCreatetxg",
        };
    })?;
    let registration: Vec<u8> = record.registration.clone().ok_or_else(|| {
        return RemovalStop::BindingIncomplete {
            slot: String::from(slot),
            field: "registration",
        };
    })?;
    let registration_identity: FolderIdentity = record.registration_identity.ok_or_else(|| {
        return RemovalStop::BindingIncomplete {
            slot: String::from(slot),
            field: "registrationIdentity",
        };
    })?;
    let slot_path: PathBuf =
        crate::worktree_clone_binding_record::binding_encoding::path_from_bytes(
            record.slot.as_slice(),
        )
        .ok_or_else(|| {
            return RemovalStop::BindingIncomplete {
                slot: String::from(slot),
                field: "slot",
            };
        })?;
    let pool_guid: DatasetGuid = record.pool_guid;
    let registration_id: Vec<u8> = registration_path_name(slot, registration.as_slice())?;
    return Ok(RecordedBinding {
        slot: String::from(slot),
        slot_path,
        pool: record.pool.clone(),
        pool_guid,
        clone_dataset,
        clone_guid,
        clone_createtxg,
        origin_dataset,
        origin_dataset_guid,
        origin_snapshot,
        origin_snapshot_guid,
        origin_snapshot_createtxg,
        registration_id,
        registration_identity,
    });
}

/// What:
///  The registration directory's name,
///  which is the last component of the path the binding recorded.
/// Why:
///  The plan records that name's bytes,
///  and a reader matches them against the registration standing there.
///
/// # Throws
/// A recorded path that names no last component.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const id = lastComponent(registrationPath);
/// ```
fn registration_path_name(
    slot: &str,
    path: &[u8],
) -> Result<Vec<u8>, RemovalStop> {
    let parts: Vec<&[u8]> = path
        .split(|byte| return *byte == b'/')
        .filter(|part| return !part.is_empty())
        .collect();
    let Some(name) = parts.last() else {
        return Err(RemovalStop::BindingIncomplete {
            slot: String::from(slot),
            field: "registration",
        });
    };
    return Ok(Vec::from(*name));
}

/// What:
///  Whether the registration standing in the repository is the one the add recorded,
///  and that registration's identity with its device,
///  which is what the plan's `admin` field holds.
/// Why:
///  A removal settles only the matching registration,
///  and the plan records the identity a later reader matches against the directory
///  standing there.
///
/// # Throws
/// A registration that is absent or of another identity,
/// which is another removal's or another worktree's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const admin = provedRegistration(commonDir, recorded);
/// ```
pub fn prove_the_adds_registration(
    common_dir: &Path,
    recorded: &RecordedBinding,
) -> Result<ObjectId, RemovalStop> {
    let name: std::ffi::OsString = std::str::from_utf8(recorded.registration_id.as_slice())
        .map_or_else(
            |_| {
                return std::ffi::OsString::from(
                    String::from_utf8_lossy(recorded.registration_id.as_slice()).into_owned(),
                );
            },
            std::ffi::OsString::from,
        );
    let registration: PathBuf = common_dir.join("worktrees").join(name);
    let folder: std::fs::File = std::fs::File::open(registration.as_path()).map_err(|error| {
        return RemovalStop::DoesNotAnswer {
            object: registration.display().to_string(),
            reason: error.to_string(),
        };
    })?;
    let identity: ObjectId = crate::fs_beneath::identity_of(&folder).map_err(|error| {
        return RemovalStop::DoesNotAnswer {
            object: registration.display().to_string(),
            reason: error.to_string(),
        };
    })?;
    let recorded_birth: Option<(i64, u32)> = recorded.registration_identity.birth();
    if identity.inode != recorded.registration_identity.inode() || identity.birth != recorded_birth
    {
        return Err(RemovalStop::IdentityDiffers {
            object: registration.display().to_string(),
            field: "registrationIdentity",
            recorded: recorded.registration_identity.encode(),
            live: identity.encode(),
        });
    }
    return Ok(identity);
}

/// What:
///  The origin-side snapshot and bookmark inventory the add recorded for this slot.
/// Why:
///  The third check compares the live clone against it,
///  and it is an add-time fact written once:
///  a removal never rebuilds it,
///  never updates it to match an external change,
///  and refuses one whose own header disagrees with the binding's origin fields.
///
/// # Throws
/// The inventory that cannot be read,
/// or the field in which it disagrees with the binding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const originInventory = readTheOriginInventory(commonDir, recorded);
/// ```
pub fn read_the_origin_inventory(
    common_dir: &Path,
    recorded: &RecordedBinding,
) -> Result<InventoryRecord, RemovalStop> {
    let home: PathBuf = home_folder(common_dir);
    let inventory: InventoryRecord = read_inventory(
        home.as_path(),
        recorded.slot.as_str(),
        InventoryKind::Origin,
    )
    .map_err(|refusal| {
        return RemovalStop::OriginInventoryUnreadable {
            slot: recorded.slot.clone(),
            reason: format!("{refusal:?}"),
        };
    })?;
    if inventory.dataset != recorded.origin_dataset {
        return Err(RemovalStop::OriginInventoryOfAnotherDataset {
            inventory_dataset: inventory.dataset.clone(),
            recorded_dataset: recorded.origin_dataset.clone(),
        });
    }
    let pairs: [(&'static str, String, String); 4] = [
        (
            "originDatasetGuid",
            recorded.origin_dataset_guid.file_name(),
            inventory.dataset_guid.file_name(),
        ),
        (
            "originSnapshot",
            recorded.origin_snapshot.clone(),
            inventory.bounds.clone(),
        ),
        (
            "originSnapshotGuid",
            recorded.origin_snapshot_guid.file_name(),
            inventory.bounds_guid.file_name(),
        ),
        (
            "originSnapshotCreatetxg",
            recorded.origin_snapshot_createtxg.to_string(),
            inventory.bounds_createtxg.to_string(),
        ),
    ];
    for (field, recorded_value, inventory_value) in pairs {
        if recorded_value != inventory_value {
            return Err(RemovalStop::IdentityDiffers {
                object: format!("the origin-side inventory of the slot `{}`", recorded.slot),
                field,
                recorded: recorded_value,
                live: inventory_value,
            });
        }
    }
    return Ok(inventory);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_identities_tests.rs"]
mod tests;
