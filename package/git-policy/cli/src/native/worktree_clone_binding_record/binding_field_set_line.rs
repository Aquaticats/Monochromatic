//! What:
//!  One line of a binding record recorded into the fields:
//! the value each key may carry,
//! the words a value that cannot be read is refused with.
//! Why:
//!  The reader calls one setter per line,
//! so the one match on the key is where a field's grammar lives,
//! kept apart from the fields themselves and from the check that the whole record answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // fields.set('claim', '2', line)
//! ```

/// The four encodings a record's values are written in.
use super::binding_encoding::{
    FolderIdentity,
    Request,
    hexadecimal_as_bytes,
};
/// The fields one line is recorded into.
use super::binding_field_set::Fields;
/// The eight phases and the seven settle reasons.
use super::binding_words::{
    Phase,
    SettleReason,
};
/// Why a line is not a field this record can hold.
use super::record_refusal::RecordRefusal;
/// The mount protocol's two typed values.
use crate::worktree_clone_mount_protocol::{
    DatasetGuid,
    RequestNonce,
};

/// What:
///  One line's key and value recorded into the fields,
///  or the value the field cannot hold.
/// Why:
///   A refused line stops the parse at its own line and key,
/// so a person reads which line of the record is wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // impl Fields { set(key, value, line): void }
/// ```
impl Fields {
    /// Record one line's value,
    /// or name the value the field cannot hold.
    pub(super) fn set(
        &mut self,
        key: &str,
        value: &str,
        line: usize,
    ) -> Result<(), RecordRefusal> {
        let refusal = |reason: String| {
            return RecordRefusal::Value {
                line,
                key: String::from(key),
                reason,
            };
        };
        return match key {
            "claim" => {
                self.claim = Some(decimal(value, 20).ok_or_else(|| {
                    return refusal(String::from(
                        "a claim generation is decimal digits with no sign",
                    ));
                })?);
                Ok(())
            }
            "nonce" => {
                self.nonce = Some(
                    RequestNonce::parse(value).map_err(|error| return refusal(error.message()))?,
                );
                Ok(())
            }
            "request" => {
                self.request = Some(Request::decode(value).ok_or_else(|| {
                    return refusal(String::from(
                        "a request is a form word, a space and a hexadecimal argument or `-`",
                    ));
                })?);
                Ok(())
            }
            "phase" => {
                self.phase = Some(Phase::parse(value).ok_or_else(|| {
                    return refusal(format!("`{value}` is not one of the eight phases"));
                })?);
                Ok(())
            }
            "settleReason" => {
                self.settle_reason = Some(SettleReason::parse(value).ok_or_else(|| {
                    return refusal(format!("`{value}` is not one of the seven settle reasons"));
                })?);
                Ok(())
            }
            "checkoutDataset" | "pool" | "originSnapshot" | "originDataset" | "cloneDataset" => {
                if value.is_empty()
                    || value.len() > 255
                    || value.contains('=')
                    || value.contains('\n')
                {
                    return Err(refusal(String::from(
                        "a ZFS name is one to 255 bytes with no newline and no `=`",
                    )));
                }
                match key {
                    "checkoutDataset" => self.checkout_dataset = Some(String::from(value)),
                    "pool" => self.pool = Some(String::from(value)),
                    "originSnapshot" => self.origin_snapshot = Some(String::from(value)),
                    "originDataset" => self.origin_dataset = Some(String::from(value)),
                    _ => self.clone_dataset = Some(String::from(value)),
                }
                Ok(())
            }
            "checkoutDatasetGuid"
            | "poolGuid"
            | "originSnapshotGuid"
            | "originDatasetGuid"
            | "cloneGuid" => {
                let guid: DatasetGuid =
                    DatasetGuid::parse(value).map_err(|error| return refusal(error.message()))?;
                match key {
                    "checkoutDatasetGuid" => self.checkout_dataset_guid = Some(guid),
                    "poolGuid" => self.pool_guid = Some(guid),
                    "originSnapshotGuid" => self.origin_snapshot_guid = Some(guid),
                    "originDatasetGuid" => self.origin_dataset_guid = Some(guid),
                    _ => self.clone_guid = Some(guid),
                }
                Ok(())
            }
            "originSnapshotCreatetxg" | "cloneCreatetxg" => {
                let txg: u64 = decimal(value, 20).ok_or_else(|| {
                    return refusal(String::from("a createtxg is decimal digits with no sign"));
                })?;
                match key {
                    "originSnapshotCreatetxg" => self.origin_snapshot_createtxg = Some(txg),
                    _ => self.clone_createtxg = Some(txg),
                }
                Ok(())
            }
            "repository" | "slot" | "registration" | "pointer" => {
                let bytes: Vec<u8> = hexadecimal_as_bytes(value).ok_or_else(|| {
                    return refusal(String::from(
                        "this field is lowercase hexadecimal, two digits a byte",
                    ));
                })?;
                match key {
                    "repository" => self.repository = Some(bytes),
                    "slot" => self.slot = Some(bytes),
                    "registration" => self.registration = Some(bytes),
                    _ => self.pointer = Some(bytes),
                }
                Ok(())
            }
            "repositoryIdentity" | "slotIdentity" | "registrationIdentity" => {
                let identity: FolderIdentity = FolderIdentity::decode(value).ok_or_else(|| {
                    return refusal(String::from(
                        "an identity is `inode:seconds:nanoseconds`, or `inode:-:-`",
                    ));
                })?;
                match key {
                    "repositoryIdentity" => self.repository_identity = Some(identity),
                    "slotIdentity" => self.slot_identity = Some(identity),
                    _ => self.registration_identity = Some(identity),
                }
                Ok(())
            }
            _ => Err(RecordRefusal::UnknownKey {
                line,
                key: String::from(key),
            }),
        };
    }
}

/// A decimal field's value,
/// refused when it is not digits or runs past its widest form.
fn decimal(
    text: &str,
    widest: usize,
) -> Option<u64> {
    if text.is_empty()
        || text.len() > widest
        || !text.bytes().all(|byte| return byte.is_ascii_digit())
    {
        return None;
    }
    return text.parse().ok();
}
