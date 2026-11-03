//! Controls for the untracked-cache extension on files real Git writes: reading then writing gives back the same bytes.

use super::*;
use crate::index_reset_support_probe::untracked_extension;

/// The extension Git wrote reads, and writing it back reproduces Git's bytes.
#[test]
fn git_written_untracked_cache_round_trips() {
    let bytes: Vec<u8> = untracked_extension("untracked-io-round-trip");
    let cache = read_untracked(bytes.as_slice(), 20).expect("read");
    assert_eq!(write_untracked(&cache), bytes);
}

/// The cache Git wrote holds listings (so the control above is not an empty cache) and survives a path invalidation that
/// is then written and read again.
#[test]
fn an_invalidated_cache_still_reads_back() {
    let bytes: Vec<u8> = untracked_extension("untracked-io-invalidate");
    let mut cache = read_untracked(bytes.as_slice(), 20).expect("read");
    let root = cache.root.as_ref().expect("a status run filled the cache");
    assert!(root.valid && !root.untracked.is_empty(), "the fixture's cache lists untracked files");
    cache.invalidate_path(b"d/sub-untracked/file");
    let written: Vec<u8> = write_untracked(&cache);
    assert_ne!(written, bytes, "the invalidation changed the bytes");
    // A directory that is not valid keeps no stat data or listing in the file, so only the bytes can be compared.
    let again = read_untracked(written.as_slice(), 20).expect("read back");
    assert_eq!(write_untracked(&again), written);
}

/// A cache no status run has filled has no directories, and Git writes only a zero count for it.
#[test]
fn a_cache_without_directories_round_trips() {
    let mut cache = read_untracked(untracked_extension("untracked-io-empty").as_slice(), 20).expect("read");
    cache.root = None;
    let bytes: Vec<u8> = write_untracked(&cache);
    assert_eq!(bytes.last(), Some(&0));
    assert_eq!(read_untracked(bytes.as_slice(), 20), Some(cache));
}

/// Bytes cut short, or with something after the records, are left to Git.
#[test]
fn irregular_forms_are_not_read() {
    let bytes: Vec<u8> = untracked_extension("untracked-io-irregular");
    for cut in [1, 2, bytes.len() / 2, bytes.len() - 2] {
        assert!(read_untracked(&bytes[..cut], 20).is_none(), "cut at {cut}");
    }
    let mut extra: Vec<u8> = bytes.clone();
    extra.insert(extra.len() - 1, 7);
    assert!(read_untracked(extra.as_slice(), 20).is_none());
}
