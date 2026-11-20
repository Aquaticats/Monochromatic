// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The device-free proof of the encoder:
//! the exact bytes of one simple list,
//!  one pair of each shape,
//! a round trip of every value variant,
//!  the fixed point of decode and re-pack,
//! and every refusal (an interior NUL,
//!  an unknown version,
//! a name past the field width,
//! nesting past the
//!  bound).
//! Why:
//!  The native encoding is the only one the kernel accepts on input;
//! a byte in
//!  the wrong place is a kernel read at the wrong offset,
//! and the fixed point
//!  is what the fuzz target asserts over arbitrary input.
//!  Each golden vector's hex is the wire bytes the cited OpenZFS C rules give
//!  that shape,
//! so the test fails when the encoder and the module disagree,
//!  not when this crate's decoder agrees with this crate's encoder.

use super::*;
use crate::nvlist::{NvData, NvError};

/// Decode the hex a golden vector is written in,
/// split at eight-byte boundaries for reading.
fn bytes_of(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0, "a golden vector is whole bytes");
    return (0..hex.len() / 2)
        .map(|index| {
            let pair: &str = &hex[index * 2..index * 2 + 2];
            return u8::from_str_radix(pair, 16).expect("a golden vector is hex");
        })
        .collect();
}

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
fn a_string_pair_has_the_bytes_the_c_rules_give_it() {
    // A pair is `nvp_size`, `nvp_name_sz`, `nvp_reserve`, `nvp_value_elem`,
    // `nvp_type` (include/sys/nvpair.h:74-83), the name with its NUL padded
    // to 8 bytes (`NVP_VALOFF`, module/nvpair/nvpair.c:147), then the
    // NUL-terminated string padded to 8 bytes (`NVP_SIZE_CALC`, :135).
    // The stream is the native header (0, 1, 0, 0), the root version and
    // flag (:2890-2908), and the four-byte zero that ends a list (:2932).
    let mut list: NvList = NvList::new();
    list.add_str("name", "tank");
    let packed: Vec<u8> = list.pack().expect("a string pair packs");
    assert_eq!(
        packed,
        bytes_of(concat!(
            "0001000000000000",
            "0100000020000000",
            "0500000001000000",
            "090000006e616d65",
            "0000000074616e6b",
            "0000000000000000",
        ))
    );
}

#[test]
fn an_embedded_list_pair_has_the_bytes_the_c_rules_give_it() {
    // An embedded list's value is the 24-byte `nvlist_t` with its private
    // pointer zeroed (`nvpair_native_embedded`,
    // module/nvpair/nvpair.c:2951-2967), and its own pairs follow the parent
    // pair's blob, ended by a four-byte zero (:2932-2949).
    let mut inner: NvList = NvList::new();
    inner.add_bool_flag("pool/ds@s1");
    inner.add_bool_flag("pool/ds@s2");
    let mut list: NvList = NvList::new();
    list.push("snaps", NvData::List { value: inner });
    let packed: Vec<u8> = list.pack().expect("an embedded list packs");
    assert_eq!(
        packed,
        bytes_of(concat!(
            "0001000000000000",
            "0100000030000000",
            "0600000001000000",
            "13000000736e6170",
            "7300000000000000",
            "0100000000000000",
            "0000000000000000",
            "0000000020000000",
            "0b00000000000000",
            "01000000706f6f6c",
            "2f64734073310000",
            "0000000020000000",
            "0b00000000000000",
            "01000000706f6f6c",
            "2f64734073320000",
            "0000000000000000",
            "00000000",
        ))
    );
}

#[test]
fn a_string_array_pair_has_the_bytes_the_c_rules_give_it() {
    // A string array's value is `nelem * 8` zeroed placeholders followed by
    // that many NUL-terminated strings (`i_get_value_size`,
    // module/nvpair/nvpair.c:1102-1114; `nvpair_native_string_array`,
    // :2999-3013).
    let mut list: NvList = NvList::new();
    list.add_str_array("strings", vec!["a".to_string(), "bc".to_string()]);
    let packed: Vec<u8> = list.pack().expect("a string array packs");
    assert_eq!(
        packed,
        bytes_of(concat!(
            "0001000000000000",
            "0100000030000000",
            "0800000002000000",
            "1100000073747269",
            "6e67730000000000",
            "0000000000000000",
            "0000000061006263",
            "0000000000000000",
        ))
    );
}

#[test]
fn an_nvlist_array_pair_has_the_bytes_the_c_rules_give_it() {
    // An nvlist array's value is `nelem * 8` zeroed placeholders, then
    // `nelem` 24-byte `nvlist_t` copies with their private pointers zeroed
    // (`i_get_value_size`, module/nvpair/nvpair.c:1150-1153;
    // `nvpair_native_embedded_array`, :2970-2996), and each list's pairs
    // follow in turn, each ended by a four-byte zero.
    let mut first: NvList = NvList::new();
    first.add_u64("x", 1);
    let mut second: NvList = NvList::new();
    second.add_u64("y", 2);
    let mut list: NvList = NvList::new();
    list.push(
        "listarray",
        NvData::ListArray {
            value: vec![first, second],
        },
    );
    let packed: Vec<u8> = list.pack().expect("an nvlist array packs");
    assert_eq!(
        packed,
        bytes_of(concat!(
            "0001000000000000",
            "0100000060000000",
            "0a00000002000000",
            "140000006c697374",
            "6172726179000000",
            "0000000000000000",
            "0000000000000000",
            "0000000000000000",
            "0100000000000000",
            "0000000000000000",
            "0000000000000000",
            "0100000000000000",
            "0000000000000000",
            "0000000020000000",
            "0200000001000000",
            "0800000078000000",
            "0000000001000000",
            "0000000000000000",
            "2000000002000000",
            "0100000008000000",
            "7900000000000000",
            "0200000000000000",
            "0000000000000000",
        ))
    );
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
            elements: 2,
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
