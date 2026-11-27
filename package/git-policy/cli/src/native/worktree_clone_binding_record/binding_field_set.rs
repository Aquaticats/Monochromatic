//! What:
//!  The fields one binding parse has read so far,
//!  each still optional until the whole record answers,
//!  and which of them has a value.
//! Why:
//!  One settable shape lets the line loop stay one match,
//!  and a required field is checked once at the end rather than at each line's turn.
//!  Kept apart from the reader so both stay under the file limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fields = Fields.empty(); fields.set('claim', '2'); const record = fields.intoRecord();
//! ```

use super::binding_encoding::{
    FolderIdentity,
    Request,
};
use super::binding_words::{
    Phase,
    SettleReason,
};
use crate::worktree_clone_mount_protocol::{
    DatasetGuid,
    RequestNonce,
};

/// What:
///  The fields a parse has read so far,
///  each still optional until the whole record answers.
/// Why:
///  One settable shape lets the line loop stay one match,
///  and a required field is checked once at the end rather than at each line's turn.
pub(super) struct Fields {
    pub(super) claim: Option<u64>,
    pub(super) nonce: Option<RequestNonce>,
    pub(super) request: Option<Request>,
    pub(super) phase: Option<Phase>,
    pub(super) settle_reason: Option<SettleReason>,
    pub(super) checkout_dataset: Option<String>,
    pub(super) checkout_dataset_guid: Option<DatasetGuid>,
    pub(super) repository: Option<Vec<u8>>,
    pub(super) repository_identity: Option<FolderIdentity>,
    pub(super) slot: Option<Vec<u8>>,
    pub(super) slot_identity: Option<FolderIdentity>,
    pub(super) registration: Option<Vec<u8>>,
    pub(super) registration_identity: Option<FolderIdentity>,
    pub(super) pool: Option<String>,
    pub(super) pool_guid: Option<DatasetGuid>,
    pub(super) origin_snapshot: Option<String>,
    pub(super) origin_snapshot_guid: Option<DatasetGuid>,
    pub(super) origin_snapshot_createtxg: Option<u64>,
    pub(super) origin_dataset: Option<String>,
    pub(super) origin_dataset_guid: Option<DatasetGuid>,
    pub(super) clone_dataset: Option<String>,
    pub(super) clone_guid: Option<DatasetGuid>,
    pub(super) clone_createtxg: Option<u64>,
    pub(super) pointer: Option<Vec<u8>>,
}

/// What:
///  The fields of a record that has read nothing yet,
///  and whether one of them already has a value.
/// Why:
///   A parse starts from every field absent and fills them one line at a time,
/// and the phase's required fields are asked of the same shape at the end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // fields.empty(); fields.holds('claim')
/// ```
impl Fields {
    /// No field read yet.
    pub(super) fn empty() -> Fields {
        return Fields {
            claim: None,
            nonce: None,
            request: None,
            phase: None,
            settle_reason: None,
            checkout_dataset: None,
            checkout_dataset_guid: None,
            repository: None,
            repository_identity: None,
            slot: None,
            slot_identity: None,
            registration: None,
            registration_identity: None,
            pool: None,
            pool_guid: None,
            origin_snapshot: None,
            origin_snapshot_guid: None,
            origin_snapshot_createtxg: None,
            origin_dataset: None,
            origin_dataset_guid: None,
            clone_dataset: None,
            clone_guid: None,
            clone_createtxg: None,
            pointer: None,
        };
    }

    /// Whether one field has a value.
    pub(super) fn holds(
        &self,
        key: &str,
    ) -> bool {
        return match key {
            "claim" => self.claim.is_some(),
            "nonce" => self.nonce.is_some(),
            "request" => self.request.is_some(),
            "phase" => self.phase.is_some(),
            "settleReason" => self.settle_reason.is_some(),
            "checkoutDataset" => self.checkout_dataset.is_some(),
            "checkoutDatasetGuid" => self.checkout_dataset_guid.is_some(),
            "repository" => self.repository.is_some(),
            "repositoryIdentity" => self.repository_identity.is_some(),
            "slot" => self.slot.is_some(),
            "slotIdentity" => self.slot_identity.is_some(),
            "registration" => self.registration.is_some(),
            "registrationIdentity" => self.registration_identity.is_some(),
            "pool" => self.pool.is_some(),
            "poolGuid" => self.pool_guid.is_some(),
            "originSnapshot" => self.origin_snapshot.is_some(),
            "originSnapshotGuid" => self.origin_snapshot_guid.is_some(),
            "originSnapshotCreatetxg" => self.origin_snapshot_createtxg.is_some(),
            "originDataset" => self.origin_dataset.is_some(),
            "originDatasetGuid" => self.origin_dataset_guid.is_some(),
            "cloneDataset" => self.clone_dataset.is_some(),
            "cloneGuid" => self.clone_guid.is_some(),
            "cloneCreatetxg" => self.clone_createtxg.is_some(),
            "pointer" => self.pointer.is_some(),
            _ => false,
        };
    }
}
