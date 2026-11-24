//! What:
//!  Reading one binding record's bytes into [`BindingRecord`],
//!  or refusing them by line and field.
//! Why:
//!  A record decides which dataset a destruction may target,
//!  so a file this module cannot read refuses rather than being skipped,
//!  and a field the record's phase has not proved refuses too:
//!  a phase ahead of the live facts is damage or another hand,
//!  never a lag the reader may guess past
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "A resumed add" and "What no reader may conclude").
//!  The reader checks size before it parses,
//!  then the header,
//!  then each line against the grammar's order,
//!  which catches an unknown key and a repeated one with one rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = parseBinding(bytes); // { ok: BindingRecord } | { refused: message }
//! ```

use super::binding_bound::{BINDING_MAXIMUM_BYTES, BINDING_MAXIMUM_LINE_BYTES};
use super::record_refusal::RecordRefusal;
use super::binding_encoding::{FolderIdentity, Request, hexadecimal_as_bytes, path_as_hexadecimal};
use super::binding_record::{BindingRecord, phase_that_proves, required_keys_for};
use super::binding_words::{BINDING_HEADER, BINDING_KEYS_IN_ORDER, Phase, SettleReason};
use crate::worktree_clone_mount_protocol::{DatasetGuid, RequestNonce};

/// What:
///  The fields a parse has read so far,
///  each still optional until the whole record answers.
/// Why:
///  One settable shape lets the line loop stay one match,
///  and a required field is checked once at the end rather than at each line's turn.
struct Fields {
    claim: Option<u64>,
    nonce: Option<RequestNonce>,
    request: Option<Request>,
    phase: Option<Phase>,
    settle_reason: Option<SettleReason>,
    checkout_dataset: Option<String>,
    checkout_dataset_guid: Option<DatasetGuid>,
    repository: Option<Vec<u8>>,
    repository_identity: Option<FolderIdentity>,
    slot: Option<Vec<u8>>,
    slot_identity: Option<FolderIdentity>,
    registration: Option<Vec<u8>>,
    registration_identity: Option<FolderIdentity>,
    pool: Option<String>,
    pool_guid: Option<DatasetGuid>,
    origin_snapshot: Option<String>,
    origin_snapshot_guid: Option<DatasetGuid>,
    origin_snapshot_createtxg: Option<u64>,
    origin_dataset: Option<String>,
    origin_dataset_guid: Option<DatasetGuid>,
    clone_dataset: Option<String>,
    clone_guid: Option<DatasetGuid>,
    clone_createtxg: Option<u64>,
    pointer: Option<Vec<u8>>,
}

impl Fields {
    /// No field read yet.
    fn empty() -> Fields {
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

    /// Record one line's value,
    /// or name the value the field cannot hold.
    fn set(&mut self, key: &str, value: &str, line: usize) -> Result<(), RecordRefusal> {
        let refusal = |reason: String| return RecordRefusal::Value { line, key: String::from(key), reason };
        return match key {
            "claim" => {
                self.claim = Some(decimal(value, 20).ok_or_else(|| return refusal(String::from("a claim generation is decimal digits with no sign")))?);
                Ok(())
            }
            "nonce" => {
                self.nonce = Some(RequestNonce::parse(value).map_err(|error| return refusal(error.message()))?);
                Ok(())
            }
            "request" => {
                self.request = Some(Request::decode(value).ok_or_else(|| return refusal(String::from("a request is a form word, a space and a hexadecimal argument or `-`")))?);
                Ok(())
            }
            "phase" => {
                self.phase = Some(Phase::parse(value).ok_or_else(|| return refusal(format!("`{value}` is not one of the eight phases")))?);
                Ok(())
            }
            "settleReason" => {
                self.settle_reason = Some(SettleReason::parse(value).ok_or_else(|| return refusal(format!("`{value}` is not one of the seven settle reasons")))?);
                Ok(())
            }
            "checkoutDataset" | "pool" | "originSnapshot" | "originDataset" | "cloneDataset" => {
                if value.is_empty() || value.len() > 255 || value.contains('=') || value.contains('\n') {
                    return Err(refusal(String::from("a ZFS name is one to 255 bytes with no newline and no `=`")));
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
            "checkoutDatasetGuid" | "poolGuid" | "originSnapshotGuid" | "originDatasetGuid" | "cloneGuid" => {
                let guid: DatasetGuid = DatasetGuid::parse(value).map_err(|error| return refusal(error.message()))?;
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
                let txg: u64 = decimal(value, 20).ok_or_else(|| return refusal(String::from("a createtxg is decimal digits with no sign")))?;
                match key {
                    "originSnapshotCreatetxg" => self.origin_snapshot_createtxg = Some(txg),
                    _ => self.clone_createtxg = Some(txg),
                }
                Ok(())
            }
            "repository" | "slot" | "registration" | "pointer" => {
                let bytes: Vec<u8> = hexadecimal_as_bytes(value).ok_or_else(|| return refusal(String::from("this field is lowercase hexadecimal, two digits a byte")))?;
                match key {
                    "repository" => self.repository = Some(bytes),
                    "slot" => self.slot = Some(bytes),
                    "registration" => self.registration = Some(bytes),
                    _ => self.pointer = Some(bytes),
                }
                Ok(())
            }
            "repositoryIdentity" | "slotIdentity" | "registrationIdentity" => {
                let identity: FolderIdentity = FolderIdentity::decode(value).ok_or_else(|| return refusal(String::from("an identity is `inode:seconds:nanoseconds`, or `inode:-:-`")))?;
                match key {
                    "repositoryIdentity" => self.repository_identity = Some(identity),
                    "slotIdentity" => self.slot_identity = Some(identity),
                    _ => self.registration_identity = Some(identity),
                }
                Ok(())
            }
            _ => Err(RecordRefusal::UnknownKey { line, key: String::from(key) }),
        };
    }

    /// The record these fields describe,
    /// or the first field its phase still lacks.
    fn into_record(self) -> Result<BindingRecord, RecordRefusal> {
        let phase: Phase = self.phase.ok_or_else(|| return RecordRefusal::Missing { key: String::from("phase"), phase: String::from("-") })?;
        for key in required_keys_for(phase) {
            if !self.holds(key) {
                return Err(RecordRefusal::Missing { key: String::from(key), phase: String::from(phase.word()) });
            }
        }
        for key in BINDING_KEYS_IN_ORDER {
            let Some(proving) = phase_that_proves(key) else { continue };
            if self.holds(key) && proving.order() > phase.order() {
                return Err(RecordRefusal::AheadOfItsPhase { key: String::from(key), phase: String::from(phase.word()) });
            }
        }
        match (phase, self.settle_reason) {
            (Phase::Settling, None) => return Err(RecordRefusal::SettleReason { reason: String::from("a record at phase `settling` carries the reason its fallback stopped for") }),
            (Phase::Settling, Some(_)) => {}
            (_, Some(_)) => return Err(RecordRefusal::SettleReason { reason: String::from("a settle reason is written only beside phase `settling`") }),
            (_, None) => {}
        }
        return Ok(BindingRecord {
            claim: self.claim.ok_or_else(|| return RecordRefusal::Missing { key: String::from("claim"), phase: String::from(phase.word()) })?,
            nonce: self.nonce.ok_or_else(|| return RecordRefusal::Missing { key: String::from("nonce"), phase: String::from(phase.word()) })?,
            request: self.request.ok_or_else(|| return RecordRefusal::Missing { key: String::from("request"), phase: String::from(phase.word()) })?,
            phase,
            settle_reason: self.settle_reason,
            checkout_dataset: self.checkout_dataset.ok_or_else(|| return RecordRefusal::Missing { key: String::from("checkoutDataset"), phase: String::from(phase.word()) })?,
            checkout_dataset_guid: self.checkout_dataset_guid.ok_or_else(|| return RecordRefusal::Missing { key: String::from("checkoutDatasetGuid"), phase: String::from(phase.word()) })?,
            repository: self.repository.ok_or_else(|| return RecordRefusal::Missing { key: String::from("repository"), phase: String::from(phase.word()) })?,
            repository_identity: self.repository_identity.ok_or_else(|| return RecordRefusal::Missing { key: String::from("repositoryIdentity"), phase: String::from(phase.word()) })?,
            slot: self.slot.ok_or_else(|| return RecordRefusal::Missing { key: String::from("slot"), phase: String::from(phase.word()) })?,
            slot_identity: self.slot_identity,
            registration: self.registration,
            registration_identity: self.registration_identity,
            pool: self.pool.ok_or_else(|| return RecordRefusal::Missing { key: String::from("pool"), phase: String::from(phase.word()) })?,
            pool_guid: self.pool_guid.ok_or_else(|| return RecordRefusal::Missing { key: String::from("poolGuid"), phase: String::from(phase.word()) })?,
            origin_snapshot: self.origin_snapshot.ok_or_else(|| return RecordRefusal::Missing { key: String::from("originSnapshot"), phase: String::from(phase.word()) })?,
            origin_snapshot_guid: self.origin_snapshot_guid,
            origin_snapshot_createtxg: self.origin_snapshot_createtxg,
            origin_dataset: self.origin_dataset,
            origin_dataset_guid: self.origin_dataset_guid,
            clone_dataset: self.clone_dataset.ok_or_else(|| return RecordRefusal::Missing { key: String::from("cloneDataset"), phase: String::from(phase.word()) })?,
            clone_guid: self.clone_guid,
            clone_createtxg: self.clone_createtxg,
            pointer: self.pointer,
        });
    }

    /// Whether one field has a value.
    fn holds(&self, key: &str) -> bool {
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

/// A decimal field's value,
/// refused when it is not digits or runs past its widest form.
fn decimal(text: &str, widest: usize) -> Option<u64> {
    if text.is_empty() || text.len() > widest || !text.bytes().all(|byte| return byte.is_ascii_digit()) {
        return None;
    }
    return text.parse().ok();
}

/// What:
///  Read one binding record's bytes.
/// Why:
///  This is the one reader every command uses,
///  so size,
///  header,
///  order and each field's encoding are decided once.
///
/// # Errors
/// A refusal naming the line or the field,
/// which the caller prints and acts on by changing nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = parseBinding(bytes); // BindingRecord or a refusal message
/// ```
pub fn parse_binding(bytes: &[u8]) -> Result<BindingRecord, RecordRefusal> {
    let length: u64 = bytes.len() as u64;
    if length > BINDING_MAXIMUM_BYTES {
        return Err(RecordRefusal::TooLarge { found: length, bound: BINDING_MAXIMUM_BYTES });
    }
    let text: &str = std::str::from_utf8(bytes).map_err(|_| return RecordRefusal::NotUtf8 { line: 1 })?;
    let mut lines = text.split_terminator('\n');
    let header: &str = lines.next().ok_or_else(|| return RecordRefusal::Header { found: String::new() })?;
    if header != BINDING_HEADER {
        return Err(RecordRefusal::Header { found: String::from(header) });
    }
    let mut fields: Fields = Fields::empty();
    let mut last_order: Option<usize> = None;
    let mut line_number: usize = 1;
    for line in lines {
        line_number += 1;
        let bytes_in_line: u64 = line.len() as u64 + 1;
        if bytes_in_line > BINDING_MAXIMUM_LINE_BYTES {
            return Err(RecordRefusal::LineTooLarge { line: line_number, found: bytes_in_line, bound: BINDING_MAXIMUM_LINE_BYTES });
        }
        let Some((key, value)): Option<(&str, &str)> = line.split_once('=') else {
            return Err(RecordRefusal::NotAField { line: line_number, found: String::from(line) });
        };
        let Some(order) = BINDING_KEYS_IN_ORDER.iter().position(|known| return known == &key) else {
            return Err(RecordRefusal::UnknownKey { line: line_number, key: String::from(key) });
        };
        if last_order.is_some_and(|seen| return order <= seen) {
            let expected: &str = BINDING_KEYS_IN_ORDER.get(last_order.unwrap_or(0) + 1).copied().unwrap_or("-");
            return Err(RecordRefusal::KeyOutOfOrder { line: line_number, key: String::from(key), expected: String::from(expected) });
        }
        last_order = Some(order);
        fields.set(key, value, line_number)?;
    }
    return fields.into_record();
}

/// A slot path as the record writes it,
/// for the tests and the writers that build one.
pub fn slot_hexadecimal(slot: &std::path::Path) -> String {
    return path_as_hexadecimal(slot);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "binding_parse_tests.rs"]
mod tests;
