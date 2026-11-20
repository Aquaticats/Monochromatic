//! What:
//!  The header and section checks:
//! both header lengths pass,
//! and a magic in the other
//!  byte order,
//! a wrong version,
//! flags that are not zero,
//! a header length between the two
//!  known ones,
//! a gap,
//! an overlap and a section that ends short of the file each
//!  refuse at the header step.
//! Why:
//!  A section list that does not add up is a file this process would read out
//! of bounds,
//!  and the kernel's own reader refuses that shape,
//! so the refusal is the control
//!  each arm needs.

use super::blob_fixture::Blob;
use super::btf_header::{Sections, read_sections};
use super::refusal::{Refusal, Step};
use std::path::Path;

/// The file one arm reads,
/// whose path only names it in the refusal.
const FIXTURE: &str = "fixture";

/// One valid file with one structure.
fn valid() -> Vec<u8> {
    let mut blob: Blob = Blob::new();
    blob.int("unsigned char", 1);
    return blob.bytes();
}

/// The refusal one file's sections must get.
fn refused(bytes: &[u8]) -> Refusal {
    let answer: Result<Sections<'_>, Refusal> = read_sections(bytes, Path::new(FIXTURE));
    let Err(refusal): Result<Sections<'_>, Refusal> = answer else {
        panic!("the file was accepted");
    };
    assert_eq!(refusal.step, Step::Header);
    return refusal;
}

#[test]
fn the_current_header_with_its_layout_fields_passes() {
    let bytes: Vec<u8> = valid();
    let sections: Sections<'_> = read_sections(&bytes, Path::new(FIXTURE)).expect("the header is read");
    assert_eq!(sections.header_bytes, 32);
    assert_eq!(sections.types.len(), 16);
    assert_eq!(sections.strings.len(), 15);
}

#[test]
fn the_oldest_header_passes_when_the_strings_end_the_file() {
    let mut blob: Blob = Blob::new();
    blob.int("unsigned char", 1);
    let bytes: Vec<u8> = blob.bytes_with_header(24);
    let sections: Sections<'_> = read_sections(&bytes, Path::new(FIXTURE)).expect("the header is read");
    assert_eq!(sections.header_bytes, 24);
}

#[test]
fn a_file_shorter_than_the_oldest_header_refuses() {
    let refusal: Refusal = refused(&[0x9f, 0xeb, 1, 0, 24, 0, 0, 0, 0, 0]);
    assert!(refusal.detail.contains("10 bytes"), "{}", refusal.detail);
}

#[test]
fn the_magic_in_the_other_byte_order_refuses_as_endianness() {
    let bytes: Vec<u8> = Blob::with_byte(Blob::with_byte(valid(), 0, 0xeb), 1, 0x9f);
    let refusal: Refusal = refused(&bytes);
    assert!(refusal.detail.contains("big-endian"), "{}", refusal.detail);
}

#[test]
fn another_magic_refuses_naming_both() {
    let bytes: Vec<u8> = Blob::with_byte(valid(), 0, 0x00);
    let refusal: Refusal = refused(&bytes);
    assert!(refusal.detail.contains("not 0xeb9f"), "{}", refusal.detail);
}

#[test]
fn a_version_other_than_one_refuses() {
    let bytes: Vec<u8> = Blob::with_byte(valid(), 2, 2);
    let refusal: Refusal = refused(&bytes);
    assert!(refusal.detail.contains("version is 2"), "{}", refusal.detail);
}

#[test]
fn flags_that_are_not_zero_refuse() {
    let bytes: Vec<u8> = Blob::with_byte(valid(), 3, 1);
    let refusal: Refusal = refused(&bytes);
    assert!(refusal.detail.contains("flags byte is 1"), "{}", refusal.detail);
}

#[test]
fn a_header_between_the_two_known_lengths_refuses() {
    let bytes: Vec<u8> = Blob::with_byte(valid(), 4, 28);
    let refusal: Refusal = refused(&bytes);
    assert!(refusal.detail.contains("28 bytes"), "{}", refusal.detail);
}

#[test]
fn a_header_longer_than_the_file_refuses() {
    let bytes: Vec<u8> = Blob::with_byte(valid(), 4, 200);
    let refusal: Refusal = refused(&bytes);
    assert!(refusal.detail.contains("200 bytes"), "{}", refusal.detail);
}

#[test]
fn two_sections_that_overlap_refuse() {
    let bytes: Vec<u8> = Blob::with_byte(valid(), 16, 0);
    let refusal: Refusal = refused(&bytes);
    assert!(refusal.detail.contains("before the"), "{}", refusal.detail);
}

#[test]
fn a_gap_between_two_sections_refuses() {
    let mut bytes: Vec<u8> = valid();
    bytes[16] = 20;
    let refusal: Refusal = refused(&bytes);
    assert!(refusal.detail.contains("leaving a gap"), "{}", refusal.detail);
}

#[test]
fn sections_that_end_short_of_the_file_refuse() {
    let mut bytes: Vec<u8> = valid();
    bytes.push(0);
    let refusal: Refusal = refused(&bytes);
    assert!(
        refusal.detail.contains("sections end at byte") || refusal.detail.contains("ends at byte"),
        "{}",
        refusal.detail
    );
}
