//! Controls for the fsmonitor extension.

use super::*;

/// More entries than any bitmap below names.
const ENTRIES: usize = 1000;

/// Version 2 round trips;
/// version 1 reads its timestamp as the decimal token Git builds.
#[test]
fn both_versions_read_and_version_two_is_written() {
    let bytes: Vec<u8> = write_fsmonitor(b"builtin:abc:1", &[0, 5, 70]);
    assert_eq!(read_fsmonitor(bytes.as_slice(), ENTRIES), Some(Fsmonitor { token: b"builtin:abc:1".to_vec(), dirty: vec![0, 5, 70] }));
    let mut old: Vec<u8> = Vec::new();
    old.extend_from_slice(&1u32.to_be_bytes());
    old.extend_from_slice(&1_700_000_000_000_000_000u64.to_be_bytes());
    let mut bitmap: Vec<u8> = Vec::new();
    crate::index_ewah::Ewah::from_bits(&[2]).serialize(&mut bitmap);
    old.extend_from_slice(&(bitmap.len() as u32).to_be_bytes());
    old.extend_from_slice(bitmap.as_slice());
    assert_eq!(read_fsmonitor(old.as_slice(), ENTRIES), Some(Fsmonitor { token: b"1700000000000000000".to_vec(), dirty: vec![2] }));
}

/// A bad version,
/// a bitmap size that is not the bytes' size,
/// and trailing bytes are not read.
#[test]
fn irregular_forms_are_not_read() {
    let good: Vec<u8> = write_fsmonitor(b"t", &[1]);
    let mut version: Vec<u8> = good.clone();
    version[3] = 9;
    assert_eq!(read_fsmonitor(version.as_slice(), ENTRIES), None);
    let mut trailing: Vec<u8> = good.clone();
    trailing.push(0);
    assert_eq!(read_fsmonitor(trailing.as_slice(), ENTRIES), None);
    assert_eq!(read_fsmonitor(&good[..good.len() - 1], ENTRIES), None);
}

/// A bitmap that names more entries than the index has is not read,
/// and the review's tiny bitmap does not expand:
/// both are
/// refused before the positions are listed.
#[test]
fn a_bitmap_longer_than_the_index_is_refused() {
    let bytes: Vec<u8> = write_fsmonitor(b"t", &[0, 70]);
    assert!(read_fsmonitor(bytes.as_slice(), 71).is_some());
    assert!(read_fsmonitor(bytes.as_slice(), 70).is_none());
    let mut hostile: Vec<u8> = Vec::new();
    hostile.extend_from_slice(&2u32.to_be_bytes());
    hostile.extend_from_slice(b"t\0");
    let mut bitmap: Vec<u8> = Vec::new();
    bitmap.extend_from_slice(&1u32.to_be_bytes());
    bitmap.extend_from_slice(&1u32.to_be_bytes());
    bitmap.extend_from_slice(&0x0000_0001_ffff_ffffu64.to_be_bytes());
    bitmap.extend_from_slice(&0u32.to_be_bytes());
    hostile.extend_from_slice(&(bitmap.len() as u32).to_be_bytes());
    hostile.extend_from_slice(bitmap.as_slice());
    assert_eq!(read_fsmonitor(hostile.as_slice(), ENTRIES), None);
}
