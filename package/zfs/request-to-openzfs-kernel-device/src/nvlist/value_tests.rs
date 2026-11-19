// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The device-free proof of the list's in-memory rules:
//! `NV_UNIQUE_NAME`
//!  replaces an earlier pair of the same name,
//! a lookup answers with the last
//!  pair the kernel would keep,
//! a mistyped lookup answers `None`,
//! and each
//!  builder produces the variant its name promises.
//! Why:
//!  The request packers build lists from these calls;
//! a lookup that answered
//!  the first duplicate instead of the last would read a different GUID than
//!  the kernel stored.

use super::*;
use crate::nvlist::NvData;

#[test]
fn a_new_list_carries_the_unique_name_flag() {
    let list: NvList = NvList::new();
    assert_eq!(list.version, 0);
    assert_eq!(list.nvflag, NV_UNIQUE_NAME);
    assert!(list.pairs.is_empty());
    assert_eq!(NvList::default(), list);
}

#[test]
fn adding_a_name_twice_keeps_only_the_later_pair() {
    let mut list: NvList = NvList::new();
    list.add_u64("guid", 1);
    list.add_u64("guid", 2);
    assert_eq!(list.pairs.len(), 1);
    assert_eq!(list.get_u64("guid"), Some(2));
}

#[test]
fn a_lookup_answers_the_last_pair_even_without_the_flag() {
    let mut list: NvList = NvList {
        version: 0,
        nvflag: 0,
        pairs: Vec::new(),
    };
    list.push("k", NvData::Uint64 { value: 1 })
        .push("k", NvData::Uint64 { value: 2 });
    assert_eq!(list.pairs.len(), 2);
    assert_eq!(list.get_u64("k"), Some(2));
}

#[test]
fn a_mistyped_lookup_answers_none() {
    let mut list: NvList = NvList::new();
    list.add_str("s", "text");
    assert_eq!(list.get_u64("s"), None);
    assert_eq!(list.get_str("s"), Some("text"));
    assert_eq!(list.get_list("s"), None);
    assert_eq!(list.get_list_array("s"), None);
    assert_eq!(list.get("absent"), None);
}

#[test]
fn every_builder_produces_its_variant() {
    let mut list: NvList = NvList::new();
    list.add_u64("u64", 7);
    list.add_str("str", "v");
    list.add_boolean("boolean", true);
    list.add_bool_flag("flag");
    list.add_i32("i32", -3);
    list.add_u64_array("u64s", vec![1, 2]);
    list.add_str_array("strs", vec!["a".to_string()]);
    list.add_nvlist("nested", NvList::new());
    assert_eq!(list.get("u64"), Some(&NvData::Uint64 { value: 7 }));
    assert_eq!(
        list.get("str"),
        Some(&NvData::Str {
            value: "v".to_string()
        })
    );
    assert_eq!(list.get("boolean"), Some(&NvData::Boolean { value: true }));
    assert_eq!(list.get("flag"), Some(&NvData::BooleanFlag));
    assert_eq!(list.get("i32"), Some(&NvData::Int32 { value: -3 }));
    assert_eq!(
        list.get("u64s"),
        Some(&NvData::Uint64Array { value: vec![1, 2] })
    );
    assert_eq!(
        list.get("strs"),
        Some(&NvData::StrArray {
            value: vec!["a".to_string()]
        })
    );
    assert!(matches!(
        list.get("nested"),
        Some(NvData::List { value: _ })
    ));
}

#[test]
fn iter_visits_every_pair_in_wire_order() {
    let mut list: NvList = NvList::new();
    list.add_u64("first", 1);
    list.add_u64("second", 2);
    let names: Vec<&str> = list
        .iter()
        .map(|pair| {
            return pair.name.as_str();
        })
        .collect();
    assert_eq!(names, ["first", "second"]);
}

#[test]
fn each_value_reports_its_wire_type() {
    let cases: [(NvData, i32); 6] = [
        (NvData::BooleanFlag, DATA_TYPE_BOOLEAN),
        (NvData::Boolean { value: true }, DATA_TYPE_BOOLEAN_VALUE),
        (
            NvData::Str {
                value: String::new(),
            },
            DATA_TYPE_STRING,
        ),
        (
            NvData::Uint64Array { value: Vec::new() },
            DATA_TYPE_UINT64_ARRAY,
        ),
        (
            NvData::List {
                value: NvList::new(),
            },
            DATA_TYPE_NVLIST,
        ),
        (
            NvData::Unknown {
                dtype: 99,
                raw: Vec::new(),
            },
            99,
        ),
    ];
    for (value, expected) in cases {
        assert_eq!(value.data_type(), expected);
    }
}
