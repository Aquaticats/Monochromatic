//! What:
//!  The record a parse's fields describe,
//! and the check that every field its phase needs is there,
//! in the order its phases prove them.
//! Why:
//!  The record is answered once,
//! at the end of the parse,
//! so a missing field,
//! a field ahead of its phase and a settle reason out of place are named together,
//! and the reader of the record holds no half-filled shape.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // fields.intoRecord(): BindingRecord
//! ```

/// The fields a record is built from.
use super::binding_field_set::Fields;
/// The record the fields describe,
/// and which of its phases proves each field.
use super::binding_record::{
    BindingRecord,
    phase_that_proves,
    required_keys_for,
};
/// The keys in their order and the phases that carry them.
use super::binding_words::{
    BINDING_KEYS_IN_ORDER,
    Phase,
};
/// Why the fields are not yet a record.
use super::record_refusal::RecordRefusal;

/// What:
///  The record these fields describe,
///  or the first field its phase still lacks.
/// Why:
///   A record is either whole and in the order its phases prove,
/// or it is refused with the key and the phase a person has to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // impl Fields { intoRecord(): BindingRecord }
/// ```
impl Fields {
    /// The record these fields describe,
    /// or the first field its phase still lacks.
    pub(super) fn into_record(self) -> Result<BindingRecord, RecordRefusal> {
        let phase: Phase = self.phase.ok_or_else(|| {
            return RecordRefusal::Missing {
                key: String::from("phase"),
                phase: String::from("-"),
            };
        })?;
        for key in required_keys_for(phase) {
            if !self.holds(key) {
                return Err(RecordRefusal::Missing {
                    key: String::from(key),
                    phase: String::from(phase.word()),
                });
            }
        }
        for &key in BINDING_KEYS_IN_ORDER {
            let Some(proving) = phase_that_proves(key) else {
                continue;
            };
            if self.holds(key) && proving.order() > phase.order() {
                return Err(RecordRefusal::AheadOfItsPhase {
                    key: String::from(key),
                    phase: String::from(phase.word()),
                });
            }
        }
        match (phase, self.settle_reason) {
            (Phase::Settling, None) => {
                return Err(RecordRefusal::SettleReason {
                    reason: String::from(
                        "a record at phase `settling` carries the reason its fallback stopped for",
                    ),
                });
            }
            (Phase::Settling, Some(_)) => {}
            (_, Some(_)) => {
                return Err(RecordRefusal::SettleReason {
                    reason: String::from("a settle reason is written only beside phase `settling`"),
                });
            }
            (_, None) => {}
        }
        return Ok(BindingRecord {
            claim: self.claim.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("claim"),
                    phase: String::from(phase.word()),
                };
            })?,
            nonce: self.nonce.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("nonce"),
                    phase: String::from(phase.word()),
                };
            })?,
            request: self.request.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("request"),
                    phase: String::from(phase.word()),
                };
            })?,
            phase,
            settle_reason: self.settle_reason,
            checkout_dataset: self.checkout_dataset.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("checkoutDataset"),
                    phase: String::from(phase.word()),
                };
            })?,
            checkout_dataset_guid: self.checkout_dataset_guid.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("checkoutDatasetGuid"),
                    phase: String::from(phase.word()),
                };
            })?,
            repository: self.repository.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("repository"),
                    phase: String::from(phase.word()),
                };
            })?,
            repository_identity: self.repository_identity.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("repositoryIdentity"),
                    phase: String::from(phase.word()),
                };
            })?,
            slot: self.slot.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("slot"),
                    phase: String::from(phase.word()),
                };
            })?,
            slot_identity: self.slot_identity,
            registration: self.registration,
            registration_identity: self.registration_identity,
            pool: self.pool.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("pool"),
                    phase: String::from(phase.word()),
                };
            })?,
            pool_guid: self.pool_guid.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("poolGuid"),
                    phase: String::from(phase.word()),
                };
            })?,
            origin_snapshot: self.origin_snapshot.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("originSnapshot"),
                    phase: String::from(phase.word()),
                };
            })?,
            origin_snapshot_guid: self.origin_snapshot_guid,
            origin_snapshot_createtxg: self.origin_snapshot_createtxg,
            origin_dataset: self.origin_dataset,
            origin_dataset_guid: self.origin_dataset_guid,
            clone_dataset: self.clone_dataset.ok_or_else(|| {
                return RecordRefusal::Missing {
                    key: String::from("cloneDataset"),
                    phase: String::from(phase.word()),
                };
            })?,
            clone_guid: self.clone_guid,
            clone_createtxg: self.clone_createtxg,
            pointer: self.pointer,
        });
    }
}
