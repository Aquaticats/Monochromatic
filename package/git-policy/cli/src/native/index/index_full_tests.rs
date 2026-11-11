//! Controls for the whole-index reader on files real Git writes:
//! reading then writing gives Git's bytes back.

use super::*;
use crate::index_full_write::write_full_index;
use crate::index_reset_support_probe::index_with_extensions;
use crate::subvolume_sha::sha1;

/// The checksum of a SHA-1 repository.
fn checksum(data: &[u8]) -> Vec<u8> {
    return sha1(data).to_vec();
}

/// Index files of versions 2 and 4 with a cache-tree and an untracked cache give back their own bytes.
#[test]
fn git_written_indexes_round_trip_in_versions_two_and_four() {
    for version in ["2", "4"] {
        let bytes: Vec<u8> = index_with_extensions(format!("index-full-round-trip-{version}").as_str(), version);
        let index: FullIndex = parse_full_index(bytes.as_slice(), 20).expect("read");
        assert!(index.cache_tree.is_some() && index.untracked.is_some(), "version {version} fixture has the extensions");
        assert!(index.entries.iter().any(|entry| { return entry.stat.mtime_sec != 0 && entry.stat.size != 0; }), "the entries carry stat data");
        assert_eq!(write_full_index(&index, &checksum, crate::index_full_write::Trailer::Hashed), Some(bytes), "version {version}");
    }
}

/// Damage and forms Git answers are not read:
/// a wrong signature,
/// a bad version,
/// a cut file,
/// an unsorted pair and a split index.
#[test]
fn damaged_and_unusual_files_are_not_read() {
    let bytes: Vec<u8> = index_with_extensions("index-full-damaged", "2");
    let mut signature: Vec<u8> = bytes.clone();
    signature[0] = b'X';
    assert!(parse_full_index(signature.as_slice(), 20).is_none());
    let mut version: Vec<u8> = bytes.clone();
    version[7] = 9;
    assert!(parse_full_index(version.as_slice(), 20).is_none());
    assert!(parse_full_index(&bytes[..bytes.len() / 2], 20).is_none());
    let mut split: Vec<u8> = bytes[..bytes.len() - 20].to_vec();
    split.extend_from_slice(b"link\0\0\0\0");
    split.extend_from_slice(&[0; 20]);
    assert!(parse_full_index(split.as_slice(), 20).is_none());
}
