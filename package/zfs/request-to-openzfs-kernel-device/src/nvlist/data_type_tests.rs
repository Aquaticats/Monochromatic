// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The device-free proof that the `data_type_t` numbers,
//! the alignment rule and
//!  the version check match the table OpenZFS documents,
//! rather than whatever
//!  this crate happened to write.
//! Why:
//!  A wrong type number makes the kernel read a value's bytes as another type
//!  (a `u64` as a string,
//! or a list as a number),
//! which no test with a device
//!  would show without a live pool;
//! the table is checkable here.

use super::*;
use crate::nvlist::NvError;

#[test]
fn the_type_numbers_are_openzfs_data_type_t() {
    assert_eq!(DATA_TYPE_BOOLEAN, 1);
    assert_eq!(DATA_TYPE_BYTE, 2);
    assert_eq!(DATA_TYPE_INT16, 3);
    assert_eq!(DATA_TYPE_UINT16, 4);
    assert_eq!(DATA_TYPE_INT32, 5);
    assert_eq!(DATA_TYPE_UINT32, 6);
    assert_eq!(DATA_TYPE_INT64, 7);
    assert_eq!(DATA_TYPE_UINT64, 8);
    assert_eq!(DATA_TYPE_STRING, 9);
    assert_eq!(DATA_TYPE_BYTE_ARRAY, 10);
    assert_eq!(DATA_TYPE_INT16_ARRAY, 11);
    assert_eq!(DATA_TYPE_UINT16_ARRAY, 12);
    assert_eq!(DATA_TYPE_INT32_ARRAY, 13);
    assert_eq!(DATA_TYPE_UINT32_ARRAY, 14);
    assert_eq!(DATA_TYPE_INT64_ARRAY, 15);
    assert_eq!(DATA_TYPE_UINT64_ARRAY, 16);
    assert_eq!(DATA_TYPE_STRING_ARRAY, 17);
    assert_eq!(DATA_TYPE_HRTIME, 18);
    assert_eq!(DATA_TYPE_NVLIST, 19);
    assert_eq!(DATA_TYPE_NVLIST_ARRAY, 20);
    assert_eq!(DATA_TYPE_BOOLEAN_VALUE, 21);
    assert_eq!(DATA_TYPE_INT8, 22);
    assert_eq!(DATA_TYPE_UINT8, 23);
    assert_eq!(DATA_TYPE_BOOLEAN_ARRAY, 24);
    assert_eq!(DATA_TYPE_INT8_ARRAY, 25);
    assert_eq!(DATA_TYPE_UINT8_ARRAY, 26);
    assert_eq!(DATA_TYPE_DOUBLE, 27);
}

#[test]
fn the_pair_and_list_header_sizes_are_the_wire_format() {
    assert_eq!(NVPAIR_HDR_SIZE, 16);
    assert_eq!(NVLIST_STRUCT_SIZE, 24);
    assert_eq!(NV_UNIQUE_NAME, 0x1);
}

#[test]
fn alignment_rounds_up_to_eight_bytes() {
    assert_eq!(align8(0), 0);
    assert_eq!(align8(1), 8);
    assert_eq!(align8(7), 8);
    assert_eq!(align8(8), 8);
    assert_eq!(align8(9), 16);
    assert_eq!(align8(16), 16);
}

#[test]
fn only_version_zero_is_accepted() {
    assert!(check_version(0).is_ok());
    let error = check_version(1).expect_err("version 1 is not read");
    assert_eq!(error, NvError::UnsupportedVersion { version: 1 });
    assert!(error.to_string().contains("only version 0"));
}

#[test]
fn the_nesting_bound_is_the_module_one() {
    // `nvpair_max_recursion = 20` under `_KERNEL`
    // (`module/nvpair/nvpair.c:152`).
    assert_eq!(MAX_DEPTH, 20);
}
