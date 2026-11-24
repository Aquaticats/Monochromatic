//! Controls for the two inventories:
//!  a set that round-trips,
//!  a bookmark beside a snapshot,
//!  a count that disagrees with its lines,
//!  and a kind that disagrees with the file's name.

use crate::worktree_clone_binding_record::binding_inventory::{EntryKind, InventoryEntry, InventoryKind, InventoryRecord, parse_inventory};
use crate::worktree_clone_binding_record::record_refusal::RecordRefusal;
use crate::worktree_clone_mount_protocol::DatasetGuid;

/// One inventory of the fixture's origin.
fn origin() -> InventoryRecord {
    let guid = |value: &str| return DatasetGuid::parse(value).expect("the fixture guid reads");
    return InventoryRecord {
        kind: InventoryKind::Origin,
        pool_guid: guid("17293822569102704641"),
        dataset: String::from("tank/monochromatic-git-checkout"),
        dataset_guid: guid("4815162342109876543"),
        bounds: String::from("tank/monochromatic-git-checkout@git-worktree-clone-origin-9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f"),
        bounds_guid: guid("16371409254608711923"),
        bounds_createtxg: 121_293,
        entries: vec![
            InventoryEntry { kind: EntryKind::Snapshot, guid: guid("1122334455667788990"), createtxg: 93_823, name: String::from("autosnap_2026-11-21_22:00:00_hourly") },
            InventoryEntry { kind: EntryKind::Snapshot, guid: guid("1122334455667788991"), createtxg: 95_291, name: String::from("autosnap_2026-11-21_23:00:00_hourly") },
            InventoryEntry { kind: EntryKind::Bookmark, guid: guid("1122334455667788992"), createtxg: 95_291, name: String::from("a bookmark with spaces") },
        ],
    };
}

/// A set writes and reads back whole,
/// its bookmark and its name's spaces included.
#[test]
fn a_set_round_trips_with_its_bookmark() {
    let record: InventoryRecord = origin();
    let read: InventoryRecord = parse_inventory(record.encode().as_slice(), InventoryKind::Origin).expect("the inventory reads");
    assert_eq!(read, record);
    assert_eq!(read.entries.len(), 3);
    assert_eq!(read.entries[2].kind, EntryKind::Bookmark);
    assert_eq!(read.entries[2].name, "a bookmark with spaces");
}

/// A count above or below its lines refuses,
/// so a truncated file is never read as short history.
#[test]
fn a_count_that_disagrees_with_its_lines_refuses() {
    let record: InventoryRecord = origin();
    let text: String = String::from_utf8(record.encode()).expect("the inventory is UTF-8");
    for (stated, found) in [(4_u64, 3_u64), (2, 3)] {
        let replaced: String = text.replace("entries=3", format!("entries={stated}").as_str());
        let refusal: RecordRefusal = parse_inventory(replaced.as_bytes(), InventoryKind::Origin).expect_err("the count refused");
        assert!(matches!(refusal, RecordRefusal::EntryCount { stated: seen, found: lines } if seen == stated && lines == found), "{}", refusal.message());
    }
}

/// A file whose kind disagrees with its name refuses.
#[test]
fn a_kind_that_disagrees_with_the_name_refuses() {
    let record: InventoryRecord = origin();
    let refusal: RecordRefusal = parse_inventory(record.encode().as_slice(), InventoryKind::Clone).expect_err("the kind refused");
    assert!(matches!(refusal, RecordRefusal::Kind { .. }), "{}", refusal.message());
}

/// An empty clone inventory is a fact,
/// and it reads back as one.
#[test]
fn an_empty_clone_inventory_reads_as_empty() {
    let mut record: InventoryRecord = origin();
    record.kind = InventoryKind::Clone;
    record.dataset = String::from("tank/monochromatic-git-worktree-clone-datasets/add-cache-layer");
    record.bounds = record.dataset.clone();
    record.entries.clear();
    let read: InventoryRecord = parse_inventory(record.encode().as_slice(), InventoryKind::Clone).expect("the empty inventory reads");
    assert!(read.entries.is_empty());
    assert!(String::from_utf8(read.encode()).expect("UTF-8").contains("entries=0"));
}
