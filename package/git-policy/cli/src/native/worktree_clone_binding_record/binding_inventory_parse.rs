//! What:
//!  Reading one inventory's bytes as the file its name says it is.
//! Why:
//!  The count is the completeness proof and the kind is the name's own field,
//!  so a truncated file and a copied one are both refused rather than misread
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The two inventories").
//!  The grammar itself is `binding_inventory.rs`;
//!  this file is the reader, kept apart so both stay under the file limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const inventory = parseInventory(bytes, 'origin');
//! ```

use super::binding_bound::INVENTORY_ENTRY_LINE_MAXIMUM_BYTES;
use super::binding_inventory::{EntryKind, INVENTORY_KEYS_IN_ORDER, InventoryEntry, InventoryKind, InventoryRecord};
use super::binding_words::INVENTORY_HEADER;
use super::record_refusal::RecordRefusal;
use crate::worktree_clone_mount_protocol::DatasetGuid;

/// What:
///  Read one inventory's bytes as the file its name says it is.
/// Why:
///  The count is the completeness proof,
///  and the kind is the name's own field,
///  so a truncated file and a copied one are both refused rather than misread.
///
/// # Errors
/// A refusal naming the line,
/// the count or the kind,
/// which the caller prints and acts on by changing nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inventory = parseInventory(bytes, 'origin');
/// ```
pub fn parse_inventory(bytes: &[u8], filed_as: InventoryKind) -> Result<InventoryRecord, RecordRefusal> {
    let text: &str = std::str::from_utf8(bytes).map_err(|_| return RecordRefusal::NotUtf8 { line: 1 })?;
    let mut lines = text.split_terminator('\n');
    let header: &str = lines.next().ok_or_else(|| return RecordRefusal::Header { found: String::new() })?;
    if header != INVENTORY_HEADER {
        return Err(RecordRefusal::Header { found: String::from(header) });
    }
    let mut values: Vec<String> = Vec::new();
    let mut entries: Vec<InventoryEntry> = Vec::new();
    let mut line_number: usize = 1;
    let mut stated_count: Option<u64> = None;
    for line in lines {
        line_number += 1;
        let bytes_in_line: u64 = line.len() as u64 + 1;
        if bytes_in_line > INVENTORY_ENTRY_LINE_MAXIMUM_BYTES {
            return Err(RecordRefusal::LineTooLarge { line: line_number, found: bytes_in_line, bound: INVENTORY_ENTRY_LINE_MAXIMUM_BYTES });
        }
        if let Some(entry_text) = line.strip_prefix("entry=") {
            entries.push(parse_entry(entry_text, line_number)?);
            continue;
        }
        let Some((key, value)): Option<(&str, &str)> = line.split_once('=') else {
            return Err(RecordRefusal::NotAField { line: line_number, found: String::from(line) });
        };
        let wanted: &str = INVENTORY_KEYS_IN_ORDER.get(values.len()).copied().unwrap_or("-");
        if key != wanted {
            return Err(RecordRefusal::KeyOutOfOrder { line: line_number, key: String::from(key), expected: String::from(wanted) });
        }
        if key == "kind" {
            let kind: InventoryKind = InventoryKind::parse(value).ok_or_else(|| return RecordRefusal::Value {
                line: line_number,
                key: String::from("kind"),
                reason: String::from("a kind is `origin` or `clone`"),
            })?;
            if kind != filed_as {
                return Err(RecordRefusal::Kind { found: String::from(kind.word()), expected: String::from(filed_as.word()) });
            }
        }
        if key == "entries" {
            stated_count = Some(value.parse::<u64>().map_err(|_| return RecordRefusal::Value {
                line: line_number,
                key: String::from("entries"),
                reason: String::from("a count is decimal digits"),
            })?);
        }
        values.push(String::from(value));
    }
    if values.len() != INVENTORY_KEYS_IN_ORDER.len() {
        let missing: &str = INVENTORY_KEYS_IN_ORDER.get(values.len()).copied().unwrap_or("entries");
        return Err(RecordRefusal::Missing { key: String::from(missing), phase: String::from("inventory") });
    }
    let stated: u64 = stated_count.unwrap_or(0);
    if stated != entries.len() as u64 {
        return Err(RecordRefusal::EntryCount { stated, found: entries.len() as u64 });
    }
    let parse_guid = |guid_text: &str, key: &str, line: usize| -> Result<DatasetGuid, RecordRefusal> {
        return DatasetGuid::parse(guid_text).map_err(|error| return RecordRefusal::Value { line, key: String::from(key), reason: error.message() });
    };
    return Ok(InventoryRecord {
        kind: filed_as,
        pool_guid: parse_guid(&values[1], "poolGuid", 3)?,
        dataset: values[2].clone(),
        dataset_guid: parse_guid(&values[3], "datasetGuid", 5)?,
        bounds: values[4].clone(),
        bounds_guid: parse_guid(&values[5], "boundsGuid", 7)?,
        bounds_createtxg: values[6].parse::<u64>().map_err(|_| return RecordRefusal::Value { line: 8, key: String::from("boundsCreatetxg"), reason: String::from("a createtxg is decimal digits") })?,
        entries,
    });
}

/// One entry line's three fields and its name,
/// which may hold spaces.
fn parse_entry(text: &str, line: usize) -> Result<InventoryEntry, RecordRefusal> {
    let mut parts = text.splitn(4, ' ');
    let kind_word: &str = parts.next().unwrap_or("");
    let kind: EntryKind = EntryKind::parse(kind_word).ok_or_else(|| return RecordRefusal::Value {
        line,
        key: String::from("entry"),
        reason: String::from("an entry's kind is `snapshot` or `bookmark`"),
    })?;
    let guid: DatasetGuid = DatasetGuid::parse(parts.next().unwrap_or("")).map_err(|error| return RecordRefusal::Value { line, key: String::from("entry"), reason: error.message() })?;
    let createtxg: u64 = parts.next().unwrap_or("").parse().map_err(|_| return RecordRefusal::Value { line, key: String::from("entry"), reason: String::from("an entry's `createtxg` is decimal digits") })?;
    let name: &str = parts.next().ok_or_else(|| return RecordRefusal::Value {
        line,
        key: String::from("entry"),
        reason: String::from("an entry names its snapshot or bookmark"),
    })?;
    return Ok(InventoryEntry { kind, guid, createtxg, name: String::from(name) });
}


/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "binding_inventory_tests.rs"]
mod tests;
