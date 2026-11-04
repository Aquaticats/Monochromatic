//! Controls for the delta decoder and the pack header reads;
//!  the whole-pack comparison with real Git is in `repo_read_odb_tests.rs`.

use super::*;

/// A delta that copies, inserts and copies again gives the bytes the encoding states.
#[test]
fn a_delta_copies_and_inserts() {
    let base: &[u8] = b"hello world";
    // Source size 11, target size 14: copy "hello", insert "!!!", copy " world".
    let mut delta: Vec<u8> = vec![11, 14];
    // Copy: offset 0 (no offset bits), size 5 (bit 4 set, one size byte).
    delta.extend_from_slice(&[0x90, 5]);
    // Insert three literal bytes.
    delta.extend_from_slice(&[3, b'!', b'!', b'!']);
    // Copy: offset 5 (bit 0), size 6 (bit 4).
    delta.extend_from_slice(&[0x91, 5, 6]);
    assert_eq!(apply_delta(base, delta.as_slice()), Some(b"hello!!! world".to_vec()));
}

/// A delta whose sizes, ranges or opcodes disagree with its base is refused.
#[test]
fn a_bad_delta_is_refused() {
    let base: &[u8] = b"abcdef";
    assert!(apply_delta(base, &[7, 1, 1, b'x']).is_none(), "wrong base size");
    assert!(apply_delta(base, &[6, 2, 1, b'x']).is_none(), "result shorter than stated");
    assert!(apply_delta(base, &[6, 1, 0x91, 5, 5]).is_none(), "copy past the base");
    assert!(apply_delta(base, &[6, 1, 0]).is_none(), "opcode zero is reserved");
    assert!(apply_delta(base, &[6, 1, 2, b'x']).is_none(), "insert past the delta");
}

/// An index with a bad magic or a short size is refused without panicking.
#[test]
fn a_bad_index_is_refused() {
    let root = crate::test_support::fixture("repo-read-pack-bad");
    let index = root.join("pack-x.idx");
    std::fs::write(&index, b"not an index").expect("write");
    std::fs::write(root.join("pack-x.pack"), b"PACK").expect("write");
    assert!(Pack::open(index.as_path(), 20).is_none());
    let mut short: Vec<u8> = b"\xfftOc\0\0\0\x02".to_vec();
    short.extend(std::iter::repeat_n(0_u8, 256 * 4));
    short[8 + 255 * 4 + 3] = 5;
    std::fs::write(&index, short).expect("write");
    assert!(Pack::open(index.as_path(), 20).is_none(), "five objects claimed, none stored");
    crate::test_support::remove(root.as_path());
}
