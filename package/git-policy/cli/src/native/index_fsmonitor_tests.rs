//! Controls for the fsmonitor extension.

use super::*;

/// Version 2 round trips; version 1 reads its timestamp as the decimal token Git builds.
#[test]
fn both_versions_read_and_version_two_is_written() {
    let bytes: Vec<u8> = write_fsmonitor(b"builtin:abc:1", &[0, 5, 70]);
    assert_eq!(read_fsmonitor(bytes.as_slice()), Some(Fsmonitor { token: b"builtin:abc:1".to_vec(), dirty: vec![0, 5, 70] }));
    let mut old: Vec<u8> = Vec::new();
    old.extend_from_slice(&1u32.to_be_bytes());
    old.extend_from_slice(&1_700_000_000_000_000_000u64.to_be_bytes());
    let mut bitmap: Vec<u8> = Vec::new();
    crate::index_ewah::Ewah::from_bits(&[2]).serialize(&mut bitmap);
    old.extend_from_slice(&(bitmap.len() as u32).to_be_bytes());
    old.extend_from_slice(bitmap.as_slice());
    assert_eq!(read_fsmonitor(old.as_slice()), Some(Fsmonitor { token: b"1700000000000000000".to_vec(), dirty: vec![2] }));
}

/// A bad version, a bitmap size that is not the bytes' size, and trailing bytes are not read.
#[test]
fn irregular_forms_are_not_read() {
    let good: Vec<u8> = write_fsmonitor(b"t", &[1]);
    let mut version: Vec<u8> = good.clone();
    version[3] = 9;
    assert_eq!(read_fsmonitor(version.as_slice()), None);
    let mut trailing: Vec<u8> = good.clone();
    trailing.push(0);
    assert_eq!(read_fsmonitor(trailing.as_slice()), None);
    assert_eq!(read_fsmonitor(&good[..good.len() - 1]), None);
}
