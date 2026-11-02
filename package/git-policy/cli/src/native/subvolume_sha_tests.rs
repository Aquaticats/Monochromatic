//! Controls for the SHA functions: the published vectors of FIPS 180-4 and RFC 3174, and the block boundaries.

use super::*;

/// Lowercase hexadecimal text of a digest.
fn hex(bytes: &[u8]) -> String {
    return bytes.iter().map(|byte| { return format!("{byte:02x}"); }).collect();
}

/// The vectors for the empty message, "abc" and the 56-byte two-block message of the standards.
#[test]
fn sha1_matches_the_published_vectors() {
    assert_eq!(hex(&sha1(b"")), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    assert_eq!(hex(&sha1(b"abc")), "a9993e364706816aba3e25717850c26c9cd0d89d");
    assert_eq!(
        hex(&sha1(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")),
        "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
    );
}

/// The same messages under SHA-256.
#[test]
fn sha256_matches_the_published_vectors() {
    assert_eq!(hex(&sha256(b"")), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(hex(&sha256(b"abc")), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(
        hex(&sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

/// One million "a" characters (many blocks) under both functions.
#[test]
fn a_million_a_characters_hash_to_the_published_values() {
    let message: Vec<u8> = vec![b'a'; 1_000_000];
    assert_eq!(hex(&sha1(message.as_slice())), "34aa973cd4c4daa4f61eeb2bdbad27316534016f");
    assert_eq!(hex(&sha256(message.as_slice())), "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");
}

/// Lengths around the padding boundaries (55, 56, 63, 64 and 65 bytes) hash without panicking and differ from each
///  other, which a wrong padding loop would not guarantee.
#[test]
fn the_padding_boundaries_hash_to_distinct_digests() {
    let mut seen: Vec<String> = Vec::new();
    for length in [55usize, 56, 63, 64, 65, 119, 120] {
        let message: Vec<u8> = vec![0x5a; length];
        let first: String = hex(&sha1(message.as_slice()));
        let second: String = hex(&sha256(message.as_slice()));
        assert!(!seen.contains(&first) && !seen.contains(&second), "length {length}");
        seen.push(first);
        seen.push(second);
    }
}
