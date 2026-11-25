//! What:
//!  Reading an origin cleanup record's bytes back:
//!  the header line,
//!  then the keys in the grammar's order,
//!  with each state's lines required or forbidden by the state beside them.
//! Why:
//!  The reader refuses rather than repairs,
//!  because a record that does not read is the only evidence of what the cleanup owes
//!  (`doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The lines, the encodings and the order the reader requires").
//!  A key outside the list,
//!  a key out of order,
//!  a repeated key,
//!  a line that does not parse,
//!  and a missing key the state does not excuse each refuse the whole record,
//!  and nothing is read as a partial record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = parseRecord(bytes); // throws CleanupRefusal
//! ```

use crate::worktree_clone_binding_record::binding_encoding::path_from_bytes;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_origin_cleanup::origin_cleanup_bounds::{ORIGIN_CLEANUP_MAXIMUM_BYTES, field_maximum_bytes};
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::{CleanupBlock, OriginCleanupRecord};
use crate::worktree_clone_origin_cleanup::origin_cleanup_refusal::{CleanupRefusal, CleanupRefusalCause};
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::{
    BlockReason, CleanupState, ORIGIN_CLEANUP_HEADER, ORIGIN_CLEANUP_KEYS_IN_ORDER, ORIGIN_SNAPSHOT_PREFIX,
};
use crate::worktree_clone_removal_proof_and_plan::started_moment::is_seconds_with_offset;

/// The keys whose presence the record's state decides,
/// and so the ones a reader must see absent or present as the state says.
const STATE_KEYS: [&str; 5] = ["attemptedAt", "blockedReason", "blockedErrno", "numClones", "userRefs"];

/// What:
///  One record's bytes as its value,
///  with every rule the grammar holds checked.
/// Why:
///  The look destroys a snapshot on this record's word,
///  so the bytes are read whole and the identity lines are required:
///  a record missing one of them could name a snapshot without the dataset that proves it.
///
/// # Throws
/// The key, the line or the state that refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = parseCleanupRecord(bytes);
/// ```
pub fn parse_the_record(bytes: &[u8]) -> Result<OriginCleanupRecord, CleanupRefusal> {
    if bytes.len() as u64 > ORIGIN_CLEANUP_MAXIMUM_BYTES {
        return Err(CleanupRefusal::of(CleanupRefusalCause::Oversized, format!("{} bytes where the bound is {}", bytes.len(), ORIGIN_CLEANUP_MAXIMUM_BYTES)));
    }
    let text: &str = std::str::from_utf8(bytes).map_err(|error| return CleanupRefusal::of(CleanupRefusalCause::NotCanonical, error.to_string()))?;
    let mut region: &str = text;
    let first: &str = take_line(&mut region)?;
    if first != ORIGIN_CLEANUP_HEADER {
        return Err(CleanupRefusal::at(CleanupRefusalCause::NotCanonical, "header", String::from(first)));
    }
    let mut held: Vec<(&str, &str)> = Vec::with_capacity(ORIGIN_CLEANUP_KEYS_IN_ORDER.len());
    while !region.is_empty() {
        let line: &str = take_line(&mut region)?;
        let Some((key, value)) = line.split_once('=') else {
            return Err(CleanupRefusal::at(CleanupRefusalCause::NotCanonical, "record", String::from(line)));
        };
        if !ORIGIN_CLEANUP_KEYS_IN_ORDER.contains(&key) {
            return Err(CleanupRefusal::at(CleanupRefusalCause::UnknownKey, key, String::from("the grammar holds no such key")));
        }
        let widest: usize = field_maximum_bytes(key).unwrap_or(0);
        if value.len() > widest {
            return Err(CleanupRefusal::at(CleanupRefusalCause::FieldTooWide, key, String::from(value)));
        }
        held.push((key, value));
    }
    return the_record(&held);
}

/// One line of the region and the region after it,
/// refused when the region holds no line at all.
fn take_line<'a>(region: &mut &'a str) -> Result<&'a str, CleanupRefusal> {
    let Some(end) = region.find('\n') else {
        return Err(CleanupRefusal::of(CleanupRefusalCause::EndsEarly, String::from(*region)));
    };
    let line: &'a str = &region[..end];
    *region = &region[end + 1..];
    return Ok(line);
}

/// What:
///  The keys a record was read with,
///  checked for order,
///  repetition and a key the grammar does not hold.
/// Why:
///  One walk in the grammar's order refuses all three at once:
///  a key the reader does not expect next is either out of place or one the grammar lacks.
fn ordered(held: &[(&str, &str)]) -> Result<(), CleanupRefusal> {
    let mut place: usize = 0;
    for (index, (key, _)) in held.iter().enumerate() {
        if held[..index].iter().any(|pair| return pair.0 == *key) {
            return Err(CleanupRefusal::at(CleanupRefusalCause::RepeatedKey, key, String::from("written twice")));
        }
        let Some(found) = ORIGIN_CLEANUP_KEYS_IN_ORDER.get(place..).and_then(|rest| return rest.iter().position(|named| return named == key)) else {
            if ORIGIN_CLEANUP_KEYS_IN_ORDER.contains(key) {
                return Err(CleanupRefusal::at(CleanupRefusalCause::OutOfOrder, key, String::from("the grammar writes it earlier")));
            }
            return Err(CleanupRefusal::at(CleanupRefusalCause::UnknownKey, key, String::from("the grammar holds no such key")));
        };
        place += found + 1;
    }
    return Ok(());
}

/// One key's value,
/// or nothing when the record does not carry it.
fn value_of<'a>(held: &'a [(&str, &'a str)], key: &str) -> Option<&'a str> {
    for (named, value) in held {
        if *named == key {
            return Some(value);
        }
    }
    return None;
}

/// What:
///  The record the held keys spell,
///  with each line's kind checked and the state's own lines required or forbidden.
/// Why:
///  The state paragraph is the record's whole consistency rule:
///  a blocked record without its counts is one a person cannot act on,
///  and an owed one with them would say an attempt ran that did not.
fn the_record(held: &[(&str, &str)]) -> Result<OriginCleanupRecord, CleanupRefusal> {
    ordered(held)?;
    let state: CleanupState = required(held, "state").and_then(|word| return CleanupState::parse(word).ok_or_else(|| return not_canonical("state", word)))?;
    let started_at: String = String::from(required(held, "startedAt")?);
    if !is_seconds_with_offset(started_at.as_str()) {
        return Err(not_canonical("startedAt", started_at.as_str()));
    }
    let blocked: bool = state == CleanupState::Blocked;
    let attempted_at: Option<String> = optional(held, "attemptedAt");
    for key in STATE_KEYS {
        let present: bool = value_of(held, key).is_some();
        if present != blocked && key != "attemptedAt" {
            return Err(CleanupRefusal::at(CleanupRefusalCause::UnexpectedKey, key, format!("`{}` holds it as {present}", state.word())));
        }
    }
    if attempted_at.is_some() != blocked {
        return Err(CleanupRefusal::at(CleanupRefusalCause::UnexpectedKey, "attemptedAt", format!("`{}` holds it as {}", state.word(), attempted_at.is_some())));
    }
    if let Some(when) = &attempted_at
        && !is_seconds_with_offset(when.as_str())
    {
        return Err(not_canonical("attemptedAt", when.as_str()));
    }
    let block: Option<CleanupBlock> = if blocked {
        let reason: BlockReason = required(held, "blockedReason").and_then(|word| return BlockReason::parse(word).ok_or_else(|| return not_canonical("blockedReason", word)))?;
        Some(CleanupBlock {
            reason,
            errno: small_number(held, "blockedErrno")?,
            clones: number(held, "numClones")?,
            holds: number(held, "userRefs")?,
        })
    } else {
        None
    };
    let origin_dataset: String = String::from(required(held, "originDataset")?);
    let origin_dataset_guid: DatasetGuid = guid(held, "originDatasetGuid")?;
    let origin_snapshot: String = String::from(required(held, "originSnapshot")?);
    if origin_snapshot.len() != ORIGIN_SNAPSHOT_PREFIX.len() + 32 || !origin_snapshot.starts_with(ORIGIN_SNAPSHOT_PREFIX) {
        return Err(not_canonical("originSnapshot", origin_snapshot.as_str()));
    }
    let origin_snapshot_guid: DatasetGuid = guid(held, "originSnapshotGuid")?;
    let origin_snapshot_createtxg: u64 = number(held, "originSnapshotCreatetxg")?;
    let clone_dataset: String = String::from(required(held, "cloneDataset")?);
    let clone_guid: Option<DatasetGuid> = match value_of(held, "cloneGuid") {
        Some(_) => Some(guid(held, "cloneGuid")?),
        None => None,
    };
    let Some(slot_path) = path_from_bytes(hexadecimal(required(held, "slotPath")?)?.as_slice()) else {
        return Err(not_canonical("slotPath", required(held, "slotPath")?));
    };
    return Ok(OriginCleanupRecord {
        state,
        started_at,
        attempted_at,
        block,
        origin_dataset,
        origin_dataset_guid,
        origin_snapshot,
        origin_snapshot_guid,
        origin_snapshot_createtxg,
        clone_dataset,
        clone_guid,
        slot_path,
    });
}

/// One key the record must carry.
fn required<'a>(held: &'a [(&str, &'a str)], key: &str) -> Result<&'a str, CleanupRefusal> {
    return value_of(held, key).ok_or_else(|| return CleanupRefusal::at(CleanupRefusalCause::MissingKey, key, String::from("the record does not carry it")));
}

/// One key the record may carry.
fn optional(held: &[(&str, &str)], key: &str) -> Option<String> {
    return value_of(held, key).map(String::from);
}

/// One canonical `u64` value.
fn number(held: &[(&str, &str)], key: &str) -> Result<u64, CleanupRefusal> {
    let text: &str = required(held, key)?;
    if !crate::worktree_clone_removal_proof_and_plan::canonical_decimal::is_canonical_decimal(text, crate::worktree_clone_origin_cleanup::origin_cleanup_bounds::NUMBER_MAXIMUM_DIGITS) {
        return Err(not_canonical(key, text));
    }
    return text.parse::<u64>().map_err(|_| return not_canonical(key, text));
}

/// One canonical number that fits an errno.
fn small_number(held: &[(&str, &str)], key: &str) -> Result<u32, CleanupRefusal> {
    let held_number: u64 = number(held, key)?;
    return u32::try_from(held_number).map_err(|_| return not_canonical(key, required(held, key).unwrap_or_default()));
}

/// One canonical GUID value.
fn guid(held: &[(&str, &str)], key: &str) -> Result<DatasetGuid, CleanupRefusal> {
    let text: &str = required(held, key)?;
    return DatasetGuid::parse(text).map_err(|_| return not_canonical(key, text));
}

/// One path's hexadecimal bytes decoded,
/// refused when the spelling is not lowercase hexadecimal of even length.
fn hexadecimal(text: &str) -> Result<Vec<u8>, CleanupRefusal> {
    if !text.len().is_multiple_of(2) {
        return Err(not_canonical("slotPath", text));
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(text.len() / 2);
    let digits: Vec<char> = text.chars().collect();
    for pair in digits.chunks(2) {
        let mut value: u8 = 0;
        for digit in pair {
            let Some(nibble) = digit.to_digit(16) else {
                return Err(not_canonical("slotPath", text));
            };
            value = (value << 4) | (nibble as u8);
        }
        bytes.push(value);
    }
    return Ok(bytes);
}

/// One value's refusal,
/// naming the key and what it held.
fn not_canonical(key: &str, said: &str) -> CleanupRefusal {
    return CleanupRefusal::at(CleanupRefusalCause::NotCanonical, key, String::from(said));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_parse_tests.rs"]
mod tests;
