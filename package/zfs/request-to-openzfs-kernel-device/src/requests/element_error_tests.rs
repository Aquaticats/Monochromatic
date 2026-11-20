//! What:
//!  The device-free proof of the per-name error reader and its diagnostic:
//! the
//!  three integer shapes the kernel uses,
//! the property `{value, source}` shape,
//!  a nested list,
//! the ignored zero and non-error values,
//! the cap on how many
//!  names are kept,
//! and the suffix that summarises the rest.
//! Why:
//!  A removal that reports only the ioctl's error number cannot say which
//!  snapshots survived;
//! this reader is what makes the difference visible.

use super::*;
use crate::nvlist::{NvData, NvList};

#[test]
fn the_three_integer_shapes_are_read() {
    let mut list: NvList = NvList::new();
    list.push("int32", NvData::Int32 { value: 1 });
    list.push("int64", NvData::Int64 { value: 2 });
    list.push("uint64", NvData::Uint64 { value: 3 });
    let elements: Vec<ElementError> = element_errors(&list);
    assert_eq!(
        elements,
        [
            ElementError {
                name: "int32".to_string(),
                errno: 1
            },
            ElementError {
                name: "int64".to_string(),
                errno: 2
            },
            ElementError {
                name: "uint64".to_string(),
                errno: 3
            },
        ]
    );
}

#[test]
fn a_zero_error_number_means_the_element_succeeded() {
    let mut list: NvList = NvList::new();
    list.push("ok", NvData::Int32 { value: 0 });
    list.push("failed", NvData::Int32 { value: 5 });
    let elements: Vec<ElementError> = element_errors(&list);
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].name, "failed");
}

#[test]
fn a_property_error_unwraps_its_value() {
    let mut property: NvList = NvList::new();
    property.add_u64("value", 13);
    property.add_u64("source", 0);
    let mut list: NvList = NvList::new();
    list.push("readonly", NvData::List { value: property });
    let elements: Vec<ElementError> = element_errors(&list);
    assert_eq!(
        elements,
        [ElementError {
            name: "readonly".to_string(),
            errno: 13
        }]
    );
}

#[test]
fn an_ordinary_nested_list_is_read_in_turn() {
    let mut nested: NvList = NvList::new();
    nested.push("inner", NvData::Int32 { value: 21 });
    let mut list: NvList = NvList::new();
    list.push("outer", NvData::List { value: nested });
    let elements: Vec<ElementError> = element_errors(&list);
    assert_eq!(
        elements,
        [ElementError {
            name: "inner".to_string(),
            errno: 21
        }]
    );
}

#[test]
fn values_that_are_not_error_numbers_are_skipped() {
    let mut list: NvList = NvList::new();
    list.add_str("text", "not an error");
    list.add_str_array("strings", vec!["a".to_string()]);
    list.add_bool_flag("flag");
    list.push("bytes", NvData::ByteArray { value: vec![1] });
    list.push("real", NvData::Int32 { value: 2 });
    let elements: Vec<ElementError> = element_errors(&list);
    assert_eq!(
        elements,
        [ElementError {
            name: "real".to_string(),
            errno: 2
        }]
    );
}

#[test]
fn the_reader_keeps_at_most_the_documented_cap() {
    let mut list: NvList = NvList::new();
    for index in 0..(MAXIMUM_ELEMENTS_KEPT + 10) {
        list.push(format!("snapshot-{index}"), NvData::Int32 { value: 2 });
    }
    let elements: Vec<ElementError> = element_errors(&list);
    assert_eq!(elements.len(), MAXIMUM_ELEMENTS_KEPT);
}

#[test]
fn the_suffix_is_empty_without_elements() {
    assert_eq!(elements_suffix(&[]), "");
}

#[test]
fn the_suffix_lists_every_element_up_to_the_shown_count() {
    let elements: Vec<ElementError> = (0..ELEMENTS_SHOWN)
        .map(|index| {
            return ElementError {
                name: format!("snap-{index}"),
                errno: 2,
            };
        })
        .collect();
    let suffix: String = elements_suffix(&elements);
    assert!(suffix.starts_with("; snap-0: "));
    assert!(!suffix.contains("more"));
}

#[test]
fn the_suffix_summarises_elements_it_does_not_show() {
    let elements: Vec<ElementError> = (0..(ELEMENTS_SHOWN + 3))
        .map(|index| {
            return ElementError {
                name: format!("snap-{index}"),
                errno: 2,
            };
        })
        .collect();
    let suffix: String = elements_suffix(&elements);
    assert!(suffix.contains("+3 more"));
}

#[test]
fn an_element_prints_its_name_and_the_errno_text() {
    let element: ElementError = ElementError {
        name: "pool/ds@snap".to_string(),
        errno: libc::ENOENT,
    };
    let text: String = element.to_string();
    assert!(text.starts_with("pool/ds@snap: "));
    assert!(text.contains(&format!("(os error {})", libc::ENOENT)));
}
