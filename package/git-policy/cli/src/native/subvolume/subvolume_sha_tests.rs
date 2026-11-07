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

/// Bytes from a fixed xorshift sequence, so a failing length can be rebuilt.
fn pseudo_random(length: usize, seed: u64) -> Vec<u8> {
    let mut state: u64 = seed | 1;
    let mut bytes: Vec<u8> = Vec::with_capacity(length);
    while bytes.len() < length {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        bytes.extend_from_slice(&state.to_le_bytes());
    }
    bytes.truncate(length);
    return bytes;
}

/// What the system's `sha1sum` prints for `bytes`, the reference these controls compare with.
fn reference_sha1(bytes: Vec<u8>) -> String {
    use std::io::Write as _;
    let mut command: std::process::Command = std::process::Command::new("sha1sum");
    command.stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped());
    let mut child: std::process::Child = crate::test_support::spawn_owned(&mut command).expect("start sha1sum");
    let mut input = child.stdin.take().expect("piped input");
    // The writer runs beside the reader so a large input cannot fill the pipe and stop both.
    let writer = std::thread::spawn(move || { input.write_all(bytes.as_slice()).expect("write"); });
    let output = child.wait_with_output().expect("sha1sum output");
    writer.join().expect("writer");
    return String::from_utf8(output.stdout).expect("UTF-8").split_whitespace().next().expect("a digest").to_string();
}

/// Every length from empty to a few blocks, which includes the padding boundaries (55, 56, 63, 64, 65, 119, 120, 127, 128),
/// equals the system's `sha1sum`.
#[test]
fn sha1_equals_the_reference_at_every_length_to_a_few_blocks() {
    for length in 0..=300usize {
        let message: Vec<u8> = pseudo_random(length, 0x9e37_79b9_7f4a_7c15);
        assert_eq!(hex(&sha1(message.as_slice())), reference_sha1(message), "length {length}");
    }
}

/// Random multi-block inputs of random lengths equal the reference.
#[test]
fn sha1_equals_the_reference_on_random_multiblock_inputs() {
    let lengths: Vec<usize> = pseudo_random(8 * 24, 42).as_chunks::<8>().0.iter().map(|chunk| { return (u64::from_le_bytes(*chunk) % 200_000) as usize; }).collect();
    for (index, length) in lengths.into_iter().enumerate() {
        let message: Vec<u8> = pseudo_random(length, index as u64 + 7);
        assert_eq!(hex(&sha1(message.as_slice())), reference_sha1(message), "input {index} of length {length}");
    }
}

/// An input longer than 512 MiB, so the bit count passes 32 bits, equals the reference. It hashes the input twice, so it runs
/// on request: `--run-ignored only`.
#[test]
#[ignore = "hashes 512 MiB twice; run once with --run-ignored only"]
fn sha1_of_more_than_512_mib_equals_the_reference() {
    let message: Vec<u8> = pseudo_random((512usize << 20) + 1, 5);
    assert_eq!(hex(&sha1(message.as_slice())), reference_sha1(message));
}

/// The owner's ruling of 2026-11-07 ("Digests: sha1 + sha2") replaces the hand-written compression functions with the published
///  crates, so the module holds none: no round constant, no rotation, no wrapping addition of words.
#[test]
fn the_module_holds_no_hand_written_compression_function() {
    let source: &str = include_str!("subvolume_sha.rs");
    let code: String = source.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
    for needle in ["rotate_left", "rotate_right", "wrapping_add", "0x428a2f98", "0x67452301"] {
        assert!(!code.contains(needle), "subvolume_sha.rs still holds `{needle}`: the digests come from the sha1 and sha2 crates");
    }
}
