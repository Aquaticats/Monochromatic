//! Known-answer and property tests for the track fingerprint.
//!
//! Every expected digest below is the output of the xxHash reference C implementation
//! (`XXH3_64bits`), not of `twox-hash`,
//!  so these tests prove that `twox-hash` and the material
//! layout match an independent implementation.
//!  Provenance of the constants:
//!
//! - Reference: `https://github.com/Cyan4973/xxHash`, tag `v0.8.3`, commit
//!   `e626a72bc2321cd320e953a0ccf1584cad60f363`, compiled with `gcc -O1` and
//!   `#define XXH_INLINE_ALL` on x86_64.
//! - Harness: the C program reproduced in `doc/decision/music-player-fingerprint-hash.md`
//!   (section "Known-answer harness"). It builds the same material as `fingerprint_material` and
//!   prints `XXH3_64bits(material)`; the `raw` vectors hash the byte pattern
//!   `byte[i] = (i * 131 + 17) mod 256` of the stated length.
//!
//! The vectors cross the XXH3 length classes (0, 1 to 3, 4 to 8, 9 to 16, 17 to 128, 129 to 240,
//! and above 240 where the AVX2, SSE2, or NEON stripe kernels run), so a known answer exists for
//! the vectorized path as well as for the short scalar paths.

use super::*;
use std::collections::HashSet;

// The material of one known-answer case: the path, the size, and the mtime nanoseconds.
struct MaterialCase {
    path: String,
    size: u64,
    mtime_nanos: u128,
    expected: u64,
}

// An ASCII path of exactly `length` bytes: a slash, then the lowercase alphabet repeating.
// The reference harness builds the same path in `long_path`.
fn long_path(length: usize) -> String {
    return std::iter::once('/')
        .chain((1..length).map(|index| return char::from(b'a' + ((index - 1) % 26) as u8)))
        .collect()
}

// The six material cases with their reference digests. Cases 1, 2, 4, 5, and 6 fit the Android
// JNI call (a 64-bit mtime), and `NativeBridgeTest` asserts the same constants on a device.
fn material_cases() -> Vec<MaterialCase> {
    return vec![
        MaterialCase {
            path: "/music/invented/a.flac".to_string(),
            size: 12_345_678,
            mtime_nanos: 1_700_000_000_123_456_789,
            expected: 0x6a80_e982_cfb4_dab6,
        },
        MaterialCase { path: String::new(), size: 0, mtime_nanos: 0, expected: 0x13d1_8443_ce11_969a },
        MaterialCase {
            path: "/musique/été/chanson.ogg".to_string(),
            size: u64::MAX,
            mtime_nanos: 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210,
            expected: 0x2651_5b9b_8566_c812,
        },
        MaterialCase {
            path: "/musique/été/chanson.ogg".to_string(),
            size: u64::MAX,
            mtime_nanos: u128::from(i64::MAX as u64),
            expected: 0x5c24_c1d0_6cee_b36f,
        },
        MaterialCase {
            path: long_path(300),
            size: 987_654_321,
            mtime_nanos: 1_600_000_000_000_000_001,
            expected: 0x8edd_f5f5_9a84_2a1c,
        },
        MaterialCase {
            path: long_path(1100),
            size: 4_294_967_296,
            mtime_nanos: 1_700_000_000_999_999_999,
            expected: 0x0362_1517_49ae_2a5e,
        },
    ]
}

// Each material case hashes to the reference digest; this pins the tag, the field order, the
// field widths, the length prefix, the seed, and the hash together.
#[test]
fn material_known_answers_match_the_reference() {
    for case in material_cases() {
        assert_eq!(
            track_fingerprint(&case.path, case.size, case.mtime_nanos),
            case.expected,
            "path length {} size {} mtime {}",
            case.path.len(),
            case.size,
            case.mtime_nanos,
        );
    }
}

// The cases include materials past 240 and past 1024 bytes, so the stripe kernels have a known answer.
#[test]
fn material_cases_reach_the_vectorized_hash_paths() {
    let lengths: Vec<usize> = material_cases()
        .iter()
        .map(|case| return fingerprint_material(&case.path, case.size, case.mtime_nanos).len())
        .collect();
    assert!(lengths.iter().any(|length| return *length > 240), "no material above 240 bytes: {lengths:?}");
    assert!(lengths.iter().any(|length| return *length > 1024), "no material above 1024 bytes: {lengths:?}");
    assert!(lengths.iter().any(|length| return *length <= 128), "no short material: {lengths:?}");
}

// The raw hash of inputs at every XXH3 length-class boundary matches the reference.
#[test]
fn raw_xxh3_64_known_answers_match_the_reference() {
    let expected: [(usize, u64); 17] = [
        (0, 0x2d06_8005_38d3_94c2),
        (1, 0xf319_fe2b_dfcd_febd),
        (3, 0xa107_bb65_b715_c89b),
        (4, 0x509f_0567_aa8a_3123),
        (8, 0xb143_3dc3_9b7f_946e),
        (9, 0xfe25_4244_0b36_ddc7),
        (16, 0x7151_89ff_3dcd_cff6),
        (17, 0x7d33_163b_8af0_179c),
        (128, 0xee84_7f7f_cef4_ddbc),
        (129, 0x7e3e_7b75_0239_d4fc),
        (240, 0x4408_9a14_4aad_e02d),
        (241, 0xed93_572e_52ac_ac83),
        (512, 0xed6b_fa5c_6aa6_bf07),
        (1023, 0x90e9_c9b2_131a_cc97),
        (1024, 0xf0c5_763f_adfa_cd25),
        (1025, 0xd9b8_e93e_f3fb_e416),
        (2048, 0xa5aa_5f57_cf12_9fab),
    ];
    for (length, digest) in expected {
        let input: Vec<u8> = (0..length).map(|index| return ((index * 131 + 17) & 0xff) as u8).collect();
        assert_eq!(XxHash3_64::oneshot(&input), digest, "raw length {length}");
    }
}

// The hashed bytes begin with the domain tag and have the stated total length.
#[test]
fn material_starts_with_the_domain_tag_and_has_the_documented_length() {
    let material = fingerprint_material("/music/invented/a.flac", 7, 9);
    assert!(material.starts_with(FINGERPRINT_DOMAIN_TAG));
    assert_eq!(material.len(), FINGERPRINT_DOMAIN_TAG.len() + 8 + 22 + 8 + 16);
}

// The path length prefix, path, size, and mtime sit at their documented offsets, little-endian.
#[test]
fn material_fields_sit_at_their_documented_offsets() {
    let material = fingerprint_material("ab", 0x0102_0304_0506_0708, 0x1112_1314_1516_1718_191a_1b1c_1d1e_1f20);
    let tag_end = FINGERPRINT_DOMAIN_TAG.len();
    assert_eq!(&material[tag_end..tag_end + 8], &2_u64.to_le_bytes());
    assert_eq!(&material[tag_end + 8..tag_end + 10], b"ab");
    assert_eq!(&material[tag_end + 10..tag_end + 18], &0x0102_0304_0506_0708_u64.to_le_bytes());
    assert_eq!(
        &material[tag_end + 18..],
        &0x1112_1314_1516_1718_191a_1b1c_1d1e_1f20_u128.to_le_bytes()
    );
}

// Changing any one input changes the key; the unchanged input repeats it.
#[test]
fn every_input_changes_the_key() {
    let base = track_fingerprint("/music/invented/a.flac", 100, 200);
    assert_eq!(base, track_fingerprint("/music/invented/a.flac", 100, 200));
    assert_ne!(base, track_fingerprint("/music/invented/b.flac", 100, 200));
    assert_ne!(base, track_fingerprint("/music/invented/a.flac", 101, 200));
    assert_ne!(base, track_fingerprint("/music/invented/a.flac", 100, 201));
}

// A path cannot absorb the bytes of the fields after it: the encoding is unambiguous, so a path
// that ends in what looks like a size field still differs from the shorter path with that size.
#[test]
fn field_boundaries_are_unambiguous() {
    let mut seen: HashSet<Vec<u8>> = HashSet::new();
    let trailing_size_bytes = String::from_utf8(vec![1, 0, 0, 0, 0, 0, 0, 0]).expect("ASCII control bytes are UTF-8");
    let cases: [(String, u64, u128); 4] = [
        (format!("a{trailing_size_bytes}"), 5, 6),
        ("a".to_string(), 1, 6),
        (String::new(), 0, 0),
        ("\0".to_string(), 0, 0),
    ];
    for (path, size, mtime) in cases {
        assert!(seen.insert(fingerprint_material(&path, size, mtime)), "duplicate material for {path:?}");
    }
}

// The default seed 0 is in use: a nonzero seed gives a different digest for the same material,
// and the explicit zero seed equals the shipped one. A positive control for the seed mutants.
#[test]
fn the_default_seed_is_zero() {
    let material = fingerprint_material("/music/invented/a.flac", 12_345_678, 1_700_000_000_123_456_789);
    assert_eq!(XxHash3_64::oneshot_with_seed(0, &material), track_fingerprint("/music/invented/a.flac", 12_345_678, 1_700_000_000_123_456_789));
    assert_ne!(XxHash3_64::oneshot_with_seed(1, &material), track_fingerprint("/music/invented/a.flac", 12_345_678, 1_700_000_000_123_456_789));
}

// The key is a full 64 bits: across many invented tracks, both halves vary (a truncation to 32
// bits would zero the high half).
#[test]
fn the_key_uses_all_sixty_four_bits() {
    let high_halves: HashSet<u32> = (0..2000_u64)
        .map(|index| return (track_fingerprint(&format!("/music/invented/{index}.flac"), index, u128::from(index)) >> 32) as u32)
        .collect();
    assert!(high_halves.len() > 1900, "high halves collapsed: {} distinct of 2000", high_halves.len());
}
