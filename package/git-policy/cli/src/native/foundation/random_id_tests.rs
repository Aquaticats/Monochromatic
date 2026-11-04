//! Controls for the identifier layout.

use super::*;

/// Fixed input bytes give the canonical layout with version and variant bits set.
#[test]
fn the_layout_sets_version_and_variant() {
    assert_eq!(
        format_uuid_v4([0; 16]),
        "00000000-0000-4000-8000-000000000000"
    );
    assert_eq!(
        format_uuid_v4([0xff; 16]),
        "ffffffff-ffff-4fff-bfff-ffffffffffff"
    );
    assert_eq!(
        format_uuid_v4([
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
            0xcd, 0xef
        ]),
        "01234567-89ab-4def-8123-456789abcdef"
    );
}

/// Fresh identifiers have the layout and differ.
#[test]
fn fresh_identifiers_are_canonical_and_distinct() {
    let first: String = random_uuid().expect("random source");
    let second: String = random_uuid().expect("random source");
    assert_ne!(first, second);
    assert_eq!(first.len(), 36);
    for (index, byte) in first.bytes().enumerate() {
        if index == 8 || index == 13 || index == 18 || index == 23 {
            assert_eq!(byte, b'-');
        } else {
            assert!(byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        }
    }
    assert_eq!(first.as_bytes()[14], b'4');
}

/// A random-source failure is described in words.
#[test]
fn a_source_failure_is_described() {
    let error: RandomSourceError = random_source_error(getrandom::Error::UNSUPPORTED);
    assert!(
        error
            .0
            .starts_with("the operating system's random source failed: ")
    );
}

/// Only the canonical lowercase layout is a canonical UUID.
#[test]
fn only_the_canonical_lowercase_layout_is_a_uuid() {
    let made: String = random_uuid().expect("random source");
    assert!(is_canonical_uuid(made.as_str()));
    assert!(is_canonical_uuid("00000000-0000-0000-0000-000000000000"));
    assert!(is_canonical_uuid("ffffffff-ffff-ffff-ffff-ffffffffffff"));
    for refused in [
        "",
        "0B6C2C1E-6F5B-4D0E-9A55-3F5D8E2F6A10",
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a1",
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a100",
        "0b6c2c1e_6f5b-4d0e-9a55-3f5d8e2f6a10",
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a1g",
        "0b6c2c1e-6f5b-4d0e-9a5-53f5d8e2f6a10",
        "0b6c2c1e6f5b-4d0e-9a55--3f5d8e2f6a10",
        "-b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10",
    ] {
        assert!(!is_canonical_uuid(refused), "{refused}");
    }
}
