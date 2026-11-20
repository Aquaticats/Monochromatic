// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The device-free proof that the decoder refuses every malformed framing the
//!  fuzz target can find:
//! another encoding or byte order,
//! an unknown version,
//! a
//!  truncated buffer,
//! a pair that claims a wrong size or name,
//! a negative count,
//!  a value that runs past its pair,
//! and nesting past the bound.
//! Why:
//!  These bytes can come from a kernel reply or from a fuzzer;
//! a decoder that
//!  read one past its buffer would be the defect this crate exists to avoid.

use super::*;
use crate::nvlist::NvData;
use crate::nvlist::NvError;
use crate::nvlist::data_type::MAX_DEPTH;

/// A packed list of one string pair,
/// whose bytes the tests then damage.
fn one_string_pair(name: &str, value: &str) -> Vec<u8> {
    let mut list: NvList = NvList::new();
    list.add_str(name, value);
    return list.pack().expect("a simple list packs");
}

/// Offset of the first pair's header,
/// after the stream header and root list
/// header.
const FIRST_PAIR: usize = 4 + 4 + 4;

#[test]
fn an_empty_buffer_is_truncated() {
    let error = NvList::unpack(&[]).expect_err("no header");
    assert!(matches!(
        error,
        NvError::Truncated {
            at: 0,
            need: 4,
            have: 0
        }
    ));
}

#[test]
fn a_header_without_a_root_list_is_truncated() {
    let error = NvList::unpack(&[0, 1, 0, 0]).expect_err("no version");
    assert!(matches!(
        error,
        NvError::Truncated {
            at: 4,
            need: 4,
            have: 0
        }
    ));
}

#[test]
fn another_encoding_is_refused() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    packed[0] = 1;
    assert_eq!(
        NvList::unpack(&packed),
        Err(NvError::UnsupportedEncoding { encoding: 1 })
    );
}

#[test]
fn another_byte_order_is_refused() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    packed[1] = 0;
    assert_eq!(
        NvList::unpack(&packed),
        Err(NvError::UnsupportedEndian { byte_order: 0 })
    );
}

#[test]
fn another_version_is_refused() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    packed[4..8].copy_from_slice(&1i32.to_le_bytes());
    assert_eq!(
        NvList::unpack(&packed),
        Err(NvError::UnsupportedVersion { version: 1 })
    );
}

#[test]
fn a_pair_without_its_terminator_is_truncated() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    packed.truncate(packed.len() - 4);
    let error = NvList::unpack(&packed).expect_err("the terminator is missing");
    assert!(matches!(error, NvError::Truncated { .. }));
}

#[test]
fn a_pair_size_past_the_buffer_is_refused() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    packed[FIRST_PAIR..FIRST_PAIR + 4].copy_from_slice(&10_000i32.to_le_bytes());
    let error = NvList::unpack(&packed).expect_err("a pair cannot be longer than the buffer");
    assert!(matches!(error, NvError::BadPairSize { at: FIRST_PAIR, .. }));
}

#[test]
fn a_negative_pair_size_is_refused() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    packed[FIRST_PAIR..FIRST_PAIR + 4].copy_from_slice(&(-1i32).to_le_bytes());
    let error = NvList::unpack(&packed).expect_err("a negative size is malformed");
    assert!(matches!(error, NvError::BadPairSize { size: -1, .. }));
}

#[test]
fn a_zero_length_name_is_refused() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    packed[FIRST_PAIR + 4..FIRST_PAIR + 6].copy_from_slice(&0i16.to_le_bytes());
    let error = NvList::unpack(&packed).expect_err("a name has at least its NUL");
    assert_eq!(error, NvError::BadName { name_length: 0 });
}

#[test]
fn a_name_without_its_nul_is_refused() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    // The name is "k" and its NUL; replacing the NUL leaves the pair without one.
    let name_nul_at: usize = FIRST_PAIR + 16 + 1;
    packed[name_nul_at] = b'z';
    let error = NvList::unpack(&packed).expect_err("the name must end in a NUL");
    assert!(matches!(error, NvError::BadName { name_length: 2 }));
}

#[test]
fn a_negative_element_count_is_refused() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    packed[FIRST_PAIR + 8..FIRST_PAIR + 12].copy_from_slice(&(-1i32).to_le_bytes());
    let error = NvList::unpack(&packed).expect_err("a count cannot be negative");
    assert_eq!(error, NvError::BadArrayCount { elements: -1 });
}

#[test]
fn a_value_wider_than_its_pair_is_refused() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    // Shrink the pair so its value region is empty, then read it as a u64.
    packed[FIRST_PAIR..FIRST_PAIR + 4].copy_from_slice(&24i32.to_le_bytes());
    packed[FIRST_PAIR + 12..FIRST_PAIR + 16]
        .copy_from_slice(&crate::nvlist::data_type::DATA_TYPE_UINT64.to_le_bytes());
    let error = NvList::unpack(&packed).expect_err("a u64 needs eight bytes");
    assert!(matches!(error, NvError::ValueOverflow { .. }));
}

#[test]
fn a_string_without_its_nul_is_refused() {
    let mut packed: Vec<u8> = one_string_pair("k", "v");
    // Overwrite the value and all its padding: no NUL follows inside the pair.
    let value_at: usize = FIRST_PAIR + 24;
    for byte in &mut packed[value_at..] {
        *byte = b'x';
    }
    let error = NvList::unpack(&packed).expect_err("a string ends in a NUL");
    assert_eq!(
        error,
        NvError::BadString {
            pair: "k".to_string()
        }
    );
}

#[test]
fn nesting_past_the_bound_is_refused() {
    let mut cursor: Cursor<'_> = Cursor::new(&[0, 0, 0, 0]);
    let error = decode_pairs(&mut cursor, MAX_DEPTH + 1).expect_err("deeper than the bound");
    assert_eq!(error, NvError::TooDeep);
}

#[test]
fn an_embedded_list_decodes_its_pairs_from_the_stream() {
    let mut inner: NvList = NvList::new();
    inner.add_u64("guid", 9);
    let mut outer: NvList = NvList::new();
    outer.add_nvlist("props", inner);
    let packed: Vec<u8> = outer.pack().expect("a nested list packs");
    let decoded: NvList = NvList::unpack(&packed).expect("our own packing decodes");
    assert_eq!(decoded, outer);
}

#[test]
fn a_string_array_decodes_each_string() {
    let mut list: NvList = NvList::new();
    list.add_str_array("strs", vec!["one".to_string(), "two".to_string()]);
    let packed: Vec<u8> = list.pack().expect("a string array packs");
    let decoded: NvList = NvList::unpack(&packed).expect("our own packing decodes");
    assert_eq!(
        decoded.get("strs"),
        Some(&NvData::StrArray {
            value: vec!["one".to_string(), "two".to_string()]
        })
    );
}

#[test]
fn an_unknown_type_keeps_its_bytes_and_element_count() {
    let mut list: NvList = NvList::new();
    list.push(
        "mystery",
        NvData::Unknown {
            dtype: 40,
            elements: 1,
            raw: vec![1, 2, 3],
        },
    );
    let packed: Vec<u8> = list.pack().expect("an unknown type packs");
    let decoded: NvList = NvList::unpack(&packed).expect("an unknown type decodes");
    // The pair's value region includes the alignment padding, so the kept
    // bytes grow from three to eight; the element count is the pair's own.
    assert_eq!(
        decoded.get("mystery"),
        Some(&NvData::Unknown {
            dtype: 40,
            elements: 1,
            raw: vec![1, 2, 3, 0, 0, 0, 0, 0]
        })
    );
    let repacked: Vec<u8> = decoded.pack().expect("a decoded unknown type re-packs");
    assert_eq!(repacked, packed);
}
