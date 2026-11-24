//! What:
//!  The add's durable binding record:
//!  its fields,
//!  which of them a phase proves,
//!  and the bytes its grammar writes.
//! Why:
//!  The record is the durable half of a clone the pool already committed,
//!  so each field is a fact the writer confirmed before that write and never a guess
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The binding" and "What the writer confirms before each write").
//!  The object is built from confirmed facts only:
//!  the parser is `binding_parse.rs`,
//!  which fills the same struct from bytes,
//!  and the writer takes a record and writes every line its grammar holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type BindingRecord = { claim: number; nonce: string; request: string; phase: Phase; ... };
//! const bytes = new TextEncoder().encode([HEADER, `claim=${r.claim}`, ...].join('\n') + '\n');
//! ```

use super::binding_encoding::{FolderIdentity, Request};
use super::binding_words::{BINDING_HEADER, BINDING_KEYS_IN_ORDER, Phase, SettleReason};
use crate::worktree_clone_mount_protocol::{DatasetGuid, RequestNonce};
use std::path::{Path, PathBuf};

/// What:
///  One add's durable binding,
///  with a field absent until the phase that proves it.
/// Why:
///  Every reader of the add,
///  the removal and `doctor` reads this shape,
///  and the grammar's order keeps the bytes one record rather than a bag of fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BindingRecord = { claim: number; nonce: string; ...; pointer?: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingRecord {
    /// The claim generation that wrote these bytes.
    pub claim: u64,
    /// The random number every lock sentence is recomputed from.
    pub nonce: RequestNonce,
    /// The request the add was given.
    pub request: Request,
    /// How far the add got.
    pub phase: Phase,
    /// Which step's object stopped answering,
    /// present only at `phase=settling`.
    pub settle_reason: Option<SettleReason>,
    /// The dataset holding the checkout and this record's own home.
    pub checkout_dataset: String,
    /// That dataset's GUID,
    /// which anchors every inode the record names.
    pub checkout_dataset_guid: DatasetGuid,
    /// The Git common directory's path bytes.
    pub repository: Vec<u8>,
    /// The Git common directory's identity.
    pub repository_identity: FolderIdentity,
    /// The slot's whole path bytes.
    pub slot: Vec<u8>,
    /// The underlay folder's identity,
    /// recorded while the slot is empty.
    pub slot_identity: Option<FolderIdentity>,
    /// The registration's path bytes.
    pub registration: Option<Vec<u8>>,
    /// The registration directory's identity.
    pub registration_identity: Option<FolderIdentity>,
    /// The pool the origin and the clone live in.
    pub pool: String,
    /// That pool's GUID.
    pub pool_guid: DatasetGuid,
    /// The name of the snapshot the add took of the checkout.
    pub origin_snapshot: String,
    /// That snapshot's GUID.
    pub origin_snapshot_guid: Option<DatasetGuid>,
    /// That snapshot's `createtxg`.
    pub origin_snapshot_createtxg: Option<u64>,
    /// The dataset the snapshot was taken on.
    pub origin_dataset: Option<String>,
    /// That dataset's GUID.
    pub origin_dataset_guid: Option<DatasetGuid>,
    /// The clone's dataset name,
    /// chosen by the add before the clone exists.
    pub clone_dataset: String,
    /// The clone's GUID,
    /// written once the clone answers.
    pub clone_guid: Option<DatasetGuid>,
    /// The clone's `createtxg`.
    pub clone_createtxg: Option<u64>,
    /// The `.git` file's bytes,
    /// as Git wrote them in the empty slot.
    pub pointer: Option<Vec<u8>>,
}

impl BindingRecord {
    /// The slot's last path component,
    /// which is the record's own key.
    pub fn slot_name(&self) -> Option<String> {
        let path: PathBuf = super::binding_encoding::path_from_bytes(&self.slot)?;
        return path.file_name().map(|name| return name.to_string_lossy().into_owned());
    }

    /// The slot's whole path.
    pub fn slot_path(&self) -> Option<PathBuf> {
        return super::binding_encoding::path_from_bytes(&self.slot);
    }

    /// The Git common directory's path.
    pub fn repository_path(&self) -> Option<PathBuf> {
        return super::binding_encoding::path_from_bytes(&self.repository);
    }

    /// The registration's path,
    /// once the phase records one.
    pub fn registration_path(&self) -> Option<PathBuf> {
        let bytes: &[u8] = self.registration.as_deref()?;
        return super::binding_encoding::path_from_bytes(bytes);
    }

    /// The record's bytes,
    /// every field of its grammar in the grammar's order.
    pub fn encode(&self) -> Vec<u8> {
        let mut text: String = String::from(BINDING_HEADER);
        text.push('\n');
        for key in BINDING_KEYS_IN_ORDER {
            if let Some(value) = self.value_of(key) {
                text.push_str(key);
                text.push('=');
                text.push_str(value.as_str());
                text.push('\n');
            }
        }
        return text.into_bytes();
    }

    /// One field's value,
    /// or nothing when the phase has not proved it.
    fn value_of(&self, key: &str) -> Option<String> {
        return match key {
            "claim" => Some(self.claim.to_string()),
            "nonce" => Some(String::from(self.nonce.as_str())),
            "request" => Some(self.request.encode()),
            "phase" => Some(String::from(self.phase.word())),
            "settleReason" => self.settle_reason.map(|reason| return String::from(reason.word())),
            "checkoutDataset" => Some(self.checkout_dataset.clone()),
            "checkoutDatasetGuid" => Some(self.checkout_dataset_guid.file_name()),
            "repository" => Some(super::binding_encoding::bytes_as_hexadecimal(&self.repository)),
            "repositoryIdentity" => Some(self.repository_identity.encode()),
            "slot" => Some(super::binding_encoding::bytes_as_hexadecimal(&self.slot)),
            "slotIdentity" => self.slot_identity.map(FolderIdentity::encode),
            "registration" => self.registration.as_deref().map(super::binding_encoding::bytes_as_hexadecimal),
            "registrationIdentity" => self.registration_identity.map(FolderIdentity::encode),
            "pool" => Some(self.pool.clone()),
            "poolGuid" => Some(self.pool_guid.file_name()),
            "originSnapshot" => Some(self.origin_snapshot.clone()),
            "originSnapshotGuid" => self.origin_snapshot_guid.map(DatasetGuid::file_name),
            "originSnapshotCreatetxg" => self.origin_snapshot_createtxg.map(|txg| return txg.to_string()),
            "originDataset" => self.origin_dataset.clone(),
            "originDatasetGuid" => self.origin_dataset_guid.map(DatasetGuid::file_name),
            "cloneDataset" => Some(self.clone_dataset.clone()),
            "cloneGuid" => self.clone_guid.map(DatasetGuid::file_name),
            "cloneCreatetxg" => self.clone_createtxg.map(|txg| return txg.to_string()),
            "pointer" => self.pointer.as_deref().map(super::binding_encoding::bytes_as_hexadecimal),
            _ => None,
        };
    }

    /// The slot's path as the claim's and the file names' key,
    /// refused when it is no plain slot name.
    pub fn slot_name_refusal(&self) -> Option<String> {
        let name: String = self.slot_name()?;
        return crate::worktree_clone_mount::slot_name_refusal(&name);
    }

    /// The slot path in the same lowercase hexadecimal the record holds.
    pub fn slot_hexadecimal(slot: &Path) -> String {
        return super::binding_encoding::path_as_hexadecimal(slot);
    }
}

/// The keys a record of `phase` must hold,
/// which is every field's proof written before that phase.
pub fn required_keys_for(phase: Phase) -> Vec<&'static str> {
    let mut keys: Vec<&'static str> = vec![
        "claim",
        "nonce",
        "request",
        "phase",
        "checkoutDataset",
        "checkoutDatasetGuid",
        "repository",
        "repositoryIdentity",
        "slot",
        "pool",
        "poolGuid",
        "cloneDataset",
        "originSnapshot",
    ];
    if phase != Phase::Claimed {
        keys.push("slotIdentity");
    }
    if matches!(phase, Phase::Registered | Phase::Snapshotted | Phase::Cloned | Phase::Mounted | Phase::Filled | Phase::Settling) {
        keys.extend(["registration", "registrationIdentity", "pointer"]);
    }
    if matches!(phase, Phase::Snapshotted | Phase::Cloned | Phase::Mounted | Phase::Filled | Phase::Settling) {
        keys.extend(["originSnapshotGuid", "originSnapshotCreatetxg", "originDataset", "originDatasetGuid"]);
    }
    if matches!(phase, Phase::Cloned | Phase::Mounted | Phase::Filled | Phase::Settling) {
        keys.extend(["cloneGuid", "cloneCreatetxg"]);
    }
    return keys;
}

/// The first phase that proves one key,
/// or nothing for a key every phase may carry.
pub fn phase_that_proves(key: &str) -> Option<Phase> {
    return match key {
        "slotIdentity" => Some(Phase::Slot),
        "registration" | "registrationIdentity" | "pointer" => Some(Phase::Registered),
        "originSnapshotGuid" | "originSnapshotCreatetxg" | "originDataset" | "originDatasetGuid" => Some(Phase::Snapshotted),
        "cloneGuid" | "cloneCreatetxg" => Some(Phase::Cloned),
        _ => None,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "binding_record_tests.rs"]
mod tests;
