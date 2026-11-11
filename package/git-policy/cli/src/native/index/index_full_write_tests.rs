//! Controls for the index writer:
//! the version rules,
//! name compression and the checksum.

use super::*;
use crate::index_full::{ASSUME_VALID, FullEntry, parse_full_index};
use crate::index_stat::StatData;
use crate::subvolume_sha::sha1;

/// An entry for `path` with the given extended flags.
fn entry(path: &str, extended: u16) -> FullEntry {
    return FullEntry { stat: StatData::default(), mode: 0o100_644, id: vec![1; 20], flags: 0, extended, path: path.as_bytes().to_vec(), monitor_valid: false };
}

/// An index of the given version.
fn index(version: u32, entries: Vec<FullEntry>) -> FullIndex {
    return FullIndex { version, entries, cache_tree: None, resolve_undo: None, untracked: None, fsmonitor: None };
}

/// The checksum of a SHA-1 repository.
fn checksum(data: &[u8]) -> Vec<u8> {
    return sha1(data).to_vec();
}

/// Version 3 without an extended flag is written as version 2,
/// and version 2 with one as version 3.
#[test]
fn the_version_follows_the_extended_flags() {
    let plain: Vec<u8> = write_full_index(&index(3, vec![entry("a", 0)]), &checksum, Trailer::Hashed).expect("written");
    assert_eq!(plain[4..8], [0, 0, 0, 2]);
    let flagged: Vec<u8> = write_full_index(&index(2, vec![entry("a", crate::index_full::INTENT_TO_ADD)]), &checksum, Trailer::Hashed).expect("written");
    assert_eq!(flagged[4..8], [0, 0, 0, 3]);
    let four: Vec<u8> = write_full_index(&index(4, vec![entry("a", 0)]), &checksum, Trailer::Hashed).expect("written");
    assert_eq!(four[4..8], [0, 0, 0, 4]);
}

/// Names are padded to eight bytes in versions 2 and 3,
/// and prefix-compressed against the entry before in version 4.
#[test]
fn names_are_padded_or_compressed_by_version() {
    let entries: Vec<FullEntry> = vec![entry("dir/alpha", 0), entry("dir/beta", 0)];
    let two: Vec<u8> = write_full_index(&index(2, entries.clone()), &checksum, Trailer::Hashed).expect("written");
    // Header, then two entries of (62 + name + 1..8) rounded up to eight, then the checksum.
    assert_eq!(two.len(), 12 + ((62 + 9 + 8) & !7) + ((62 + 8 + 8) & !7) + 20);
    let four: Vec<u8> = write_full_index(&index(4, entries.clone()), &checksum, Trailer::Hashed).expect("written");
    assert_eq!(four.len(), 12 + (62 + 1 + 9 + 1) + (62 + 1 + 4 + 1) + 20, "the second name keeps `dir/` and writes `beta`");
    for bytes in [two, four] {
        let read: FullIndex = parse_full_index(bytes.as_slice(), 20).expect("read back");
        assert_eq!(read.entries.iter().map(|each| { return each.path.clone(); }).collect::<Vec<Vec<u8>>>(), entries.iter().map(|each| { return each.path.clone(); }).collect::<Vec<Vec<u8>>>());
    }
}

/// The checksum is the hash of everything before it,
/// and the stat data and assume-valid bit are written as held.
#[test]
fn the_checksum_covers_the_file_and_fields_are_kept() {
    let mut held: FullEntry = entry("a", 0);
    held.stat = StatData { ctime_sec: 1, ctime_nsec: 2, mtime_sec: 3, mtime_nsec: 4, dev: 5, ino: 6, uid: 7, gid: 8, size: 9 };
    held.flags = ASSUME_VALID;
    let bytes: Vec<u8> = write_full_index(&index(2, vec![held.clone()]), &checksum, Trailer::Hashed).expect("written");
    let (body, stored) = bytes.split_at(bytes.len() - 20);
    assert_eq!(stored, sha1(body).as_slice());
    assert_eq!(parse_full_index(bytes.as_slice(), 20).expect("read").entries, vec![held]);
}

/// The fsmonitor bitmap lists the entries not marked valid,
/// by position.
#[test]
fn the_monitor_bitmap_lists_entries_that_are_not_valid() {
    let mut first: FullEntry = entry("a", 0);
    first.monitor_valid = true;
    let mut last: FullEntry = entry("c", 0);
    last.monitor_valid = true;
    let mut written: FullIndex = index(2, vec![first, entry("b", 0), last]);
    written.fsmonitor = Some(crate::index_fsmonitor::Fsmonitor { token: b"tok".to_vec(), dirty: Vec::new() });
    let bytes: Vec<u8> = write_full_index(&written, &checksum, Trailer::Hashed).expect("written");
    let read: FullIndex = parse_full_index(bytes.as_slice(), 20).expect("read");
    assert_eq!(read.fsmonitor.expect("kept").dirty, vec![1]);
}

/// What:
///  A skipped hash writes zeros where the checksum would be,
///  and the rest of the file is the one the hashed write produces.
/// Why:
///   Git writes exactly that under `index.skipHash`,
///  so a repository that sets it keeps a file the in-process update wrote,
///  and the differential controls compare the two files byte for byte.
#[test]
fn a_skipped_hash_writes_zeros_where_the_checksum_would_be() {
    let held: Vec<FullEntry> = vec![entry("dir/alpha", 0), entry("dir/beta", 0)];
    let hashed: Vec<u8> = write_full_index(&index(4, held.clone()), &checksum, Trailer::Hashed).expect("written");
    let skipped: Vec<u8> = write_full_index(&index(4, held), &checksum, Trailer::Skipped { checksum_length: 20 }).expect("written");
    let (hashed_body, hashed_trailer) = hashed.split_at(hashed.len() - 20);
    let (skipped_body, skipped_trailer) = skipped.split_at(skipped.len() - 20);
    assert_eq!(hashed_body, skipped_body, "only the trailer differs");
    assert_eq!(hashed_trailer, sha1(hashed_body).as_slice(), "the hashed write keeps the checksum");
    assert_eq!(skipped_trailer, [0u8; 20], "a skipped hash writes zeros, as Git does");
    assert_eq!(parse_full_index(skipped.as_slice(), 20).expect("read back").entries, vec![entry("dir/alpha", 0), entry("dir/beta", 0)]);
}
