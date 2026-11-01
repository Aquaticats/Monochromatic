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
