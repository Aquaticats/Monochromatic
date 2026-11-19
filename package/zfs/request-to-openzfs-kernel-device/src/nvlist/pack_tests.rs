// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The device-free proof of the encoder:
//! the exact bytes of one simple list,
//!  a round trip of every value variant,
//! the fixed point of decode and re-pack,
//!  and every refusal (an interior NUL,
//! an unknown version,
//! a name past the
//!  field width,
//! nesting past the bound).
//! Why:
//!  The native encoding is the only one the kernel accepts on input;
//! a byte in
//!  the wrong place is a kernel read at the wrong offset,
//! and the fixed point
//!  is what the fuzz target asserts over arbitrary input.

use super::*;
use crate::nvlist::{NvData, NvError};

#[test]
fn a_one_pair_list_has_the_native_header_and_a_zero_terminator() {
    let mut list: NvList = NvList::new();
    list.add_bool_flag("abc");
    let packed: Vec<u8> = list.pack().expect("a flag packs");
    assert_eq!(&packed[..4], &[0, 1, 0, 0]);
    assert_eq!(&packed[4..8], &0i32.to_le_bytes());
    assert_eq!(&packed[8..12], &1u32.to_le_bytes());
    let pair_at: usize = 12;
    let pair_size: i32 =
        i32::from_le_bytes(packed[pair_at..pair_at + 4].try_into().expect("4 bytes"));
    // 16-byte header, the 4-byte name with its NUL, and the flag's empty value.
    assert_eq!(pair_size, 24);
    assert_eq!(
        i16::from_le_bytes(
            packed[pair_at + 4..pair_at + 6]
                .try_into()
                .expect("2 bytes")
        ),
        4
    );
    assert_eq!(
        i16::from_le_bytes(
            packed[pair_at + 6..pair_at + 8]
                .try_into()
                .expect("2 bytes")
        ),
        0
    );
    assert_eq!(
        i32::from_le_bytes(
            packed[pair_at + 8..pair_at + 12]
                .try_into()
                .expect("4 bytes")
        ),
        0
    );
    assert_eq!(
        i32::from_le_bytes(
            packed[pair_at + 12..pair_at + 16]
                .try_into()
                .expect("4 bytes")
        ),
        crate::nvlist::data_type::DATA_TYPE_BOOLEAN
    );
    assert_eq!(&packed[pair_at + 16..pair_at + 20], b"abc\0");
    assert_eq!(&packed[packed.len() - 4..], &0i32.to_le_bytes());
}

#[test]
fn every_value_variant_round_trips() {
    let mut inner: NvList = NvList::new();
    inner.add_u64("guid", 9);
    let mut list: NvList = NvList::new();
    list.push("flag", NvData::BooleanFlag);
    list.push("boolean", NvData::Boolean { value: true });
    list.push("byte", NvData::Byte { value: 1 });
    list.push("int8", NvData::Int8 { value: -1 });
    list.push("uint8", NvData::Uint8 { value: 255 });
    list.push("int16", NvData::Int16 { value: -2 });
    list.push("uint16", NvData::Uint16 { value: 65535 });
    list.push("int32", NvData::Int32 { value: -3 });
    list.push(
        "uint32",
        NvData::Uint32 {
            value: 4_000_000_000,
        },
    );
    list.push("int64", NvData::Int64 { value: -4 });
    list.push("uint64", NvData::Uint64 { value: u64::MAX });
    list.push("hrtime", NvData::HrTime { value: 123 });
    list.push("double", NvData::Double { value: 0.5 });
    list.push(
        "string",
        NvData::Str {
            value: "text".to_string(),
        },
    );
    list.push("bytes", NvData::ByteArray { value: vec![1, 2] });
    list.push("int8s", NvData::Int8Array { value: vec![-1, 2] });
    list.push("uint8s", NvData::Uint8Array { value: vec![1, 2] });
    list.push("int16s", NvData::Int16Array { value: vec![-1, 2] });
    list.push("uint16s", NvData::Uint16Array { value: vec![1, 2] });
    list.push("int32s", NvData::Int32Array { value: vec![-1, 2] });
    list.push("uint32s", NvData::Uint32Array { value: vec![1, 2] });
    list.push("int64s", NvData::Int64Array { value: vec![-1, 2] });
    list.push("uint64s", NvData::Uint64Array { value: vec![1, 2] });
    list.push(
        "booleans",
        NvData::BooleanArray {
            value: vec![false, true],
        },
    );
    list.push(
        "strings",
        NvData::StrArray {
            value: vec!["a".to_string(), "b".to_string()],
        },
    );
    list.push("list", NvData::List { value: inner });
    list.push(
        "listarray",
        NvData::ListArray {
            value: vec![NvList::new()],
        },
    );
    list.push(
        "mystery",
        NvData::Unknown {
            dtype: 40,
            raw: vec![7, 0, 0, 0, 0, 0, 0, 0],
        },
    );
    let packed: Vec<u8> = list.pack().expect("every variant packs");
    let decoded: NvList = NvList::unpack(&packed).expect("every variant decodes");
    assert_eq!(decoded, list);
}

#[test]
fn packing_is_a_fixed_point() {
    let mut list: NvList = NvList::new();
    list.add_u64("guid", 9);
    list.add_str("name", "pool/dataset@snapshot");
    let packed: Vec<u8> = list.pack().expect("a simple list packs");
    let decoded: NvList = NvList::unpack(&packed).expect("our own packing decodes");
    let repacked: Vec<u8> = decoded.pack().expect("a decoded list re-packs");
    assert_eq!(packed, repacked);
}

#[test]
fn a_nul_in_a_name_is_refused() {
    let mut list: NvList = NvList::new();
    list.push("a\0b", NvData::Uint64 { value: 1 });
    let error = list.pack().expect_err("a NUL would shorten the name");
    assert!(matches!(error, NvError::EmbeddedNul { name: _ }));
}

#[test]
fn a_nul_in_a_string_value_is_refused() {
    let mut list: NvList = NvList::new();
    list.add_str("k", "a\0b");
    let error = list.pack().expect_err("a NUL would shorten the value");
    assert_eq!(
        error,
        NvError::EmbeddedNul {
            name: "k".to_string()
        }
    );
}

#[test]
fn a_nul_in_a_string_array_value_is_refused() {
    let mut list: NvList = NvList::new();
    list.add_str_array("k", vec!["a".to_string(), "b\0".to_string()]);
    let error = list.pack().expect_err("a NUL would shorten an element");
    assert_eq!(
        error,
        NvError::EmbeddedNul {
            name: "k".to_string()
        }
    );
}

#[test]
fn an_unsupported_version_is_refused() {
    let list: NvList = NvList {
        version: 3,
        nvflag: 1,
        pairs: Vec::new(),
    };
    let error = list.pack().expect_err("only version 0 is written");
    assert_eq!(error, NvError::UnsupportedVersion { version: 3 });
}

#[test]
fn a_name_past_the_field_width_is_refused() {
    let mut list: NvList = NvList::new();
    list.push("n".repeat(i16::MAX as usize), NvData::BooleanFlag);
    let error = list.pack().expect_err("the name length field is an i16");
    assert!(matches!(error, NvError::TooBigToPack { pair: _ }));
}

#[test]
fn nesting_past_the_bound_is_refused() {
    let mut list: NvList = NvList::new();
    for _ in 0..(super::super::data_type::MAX_DEPTH + 2) {
        let mut outer: NvList = NvList::new();
        outer.add_nvlist("child", list);
        list = outer;
    }
    let error = list.pack().expect_err("deeper than the bound");
    assert_eq!(error, NvError::TooDeep);
}

#[test]
fn a_child_list_with_an_unknown_version_is_refused() {
    let mut child: NvList = NvList::new();
    child.version = 2;
    let mut list: NvList = NvList::new();
    list.add_nvlist("child", child);
    let error = list.pack().expect_err("the child version is checked first");
    assert_eq!(error, NvError::UnsupportedVersion { version: 2 });
}

#[test]
fn an_empty_name_is_packed_as_a_one_byte_nul() {
    let mut list: NvList = NvList::new();
    list.push("", NvData::BooleanFlag);
    let packed: Vec<u8> = list.pack().expect("the format allows an empty name");
    let decoded: NvList = NvList::unpack(&packed).expect("it decodes");
    assert!(decoded.get("").is_some());
}
