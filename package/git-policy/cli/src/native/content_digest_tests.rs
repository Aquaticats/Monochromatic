//! Controls for the content digest of an index snapshot.

use super::*;
use crate::test_support::{fixture, remove};

/// The published FNV-1a 64-bit vectors (the empty input is the offset basis).
#[test]
fn the_hash_matches_the_published_fnv1a_vectors() {
    assert_eq!(digest_bytes(b"").fnv1a64, 0xcbf2_9ce4_8422_2325);
    assert_eq!(digest_bytes(b"a").fnv1a64, 0xaf63_dc4c_8601_ec8c);
    assert_eq!(digest_bytes(b"foobar").fnv1a64, 0x8594_4171_f739_67e8);
    assert_eq!(digest_bytes(b"foobar").length, 6);
}

/// The hash text is 16 lowercase digits and reads back to the same digest;
///  any other spelling is refused.
#[test]
fn the_hash_text_round_trips_and_other_spellings_are_refused() {
    let digest: ContentDigest = digest_bytes(b"a");
    assert_eq!(digest.hash_text(), "af63dc4c8601ec8c");
    assert_eq!(ContentDigest::from_record(1, "af63dc4c8601ec8c"), Some(digest));
    assert_eq!(digest_bytes(b"").hash_text().len(), 16);
    let zero_padded: ContentDigest = ContentDigest {
        length: 0,
        fnv1a64: 0x1,
    };
    assert_eq!(zero_padded.hash_text(), "0000000000000001");
    assert_eq!(ContentDigest::from_record(0, "0000000000000001"), Some(zero_padded));
    for refused in ["", "AF63DC4C8601EC8C", "af63dc4c8601ec8", "af63dc4c8601ec8c0", "af63dc4c8601ec8g", "+f63dc4c8601ec8c"] {
        assert_eq!(ContentDigest::from_record(1, refused), None, "{refused:?}");
    }
}

/// A file larger than one read chunk hashes exactly as the same bytes in memory,
///  and a change of one byte or of the length changes the digest.
#[test]
fn a_file_digest_equals_the_digest_of_its_bytes_across_chunk_boundaries() {
    let root: std::path::PathBuf = fixture("content-digest");
    let bytes: Vec<u8> = (0..(CHUNK_BYTES * 2 + 17)).map(|index| return (index % 251) as u8).collect();
    let path: std::path::PathBuf = root.join("snapshot");
    std::fs::write(&path, &bytes).expect("snapshot");
    assert_eq!(digest_file(path.as_path()).expect("digest"), digest_bytes(bytes.as_slice()));
    let mut changed: Vec<u8> = bytes.clone();
    changed[CHUNK_BYTES] ^= 1;
    assert_ne!(digest_bytes(changed.as_slice()), digest_bytes(bytes.as_slice()));
    let halved: &[u8] = &bytes[..bytes.len() / 2];
    assert_ne!(digest_bytes(halved).length, digest_bytes(bytes.as_slice()).length);
    let zero_filled: Vec<u8> = vec![0; bytes.len()];
    assert_eq!(digest_bytes(zero_filled.as_slice()).length, digest_bytes(bytes.as_slice()).length);
    assert_ne!(digest_bytes(zero_filled.as_slice()).fnv1a64, digest_bytes(bytes.as_slice()).fnv1a64);
    assert!(digest_file(root.join("missing").as_path()).is_err());
    remove(root.as_path());
}
