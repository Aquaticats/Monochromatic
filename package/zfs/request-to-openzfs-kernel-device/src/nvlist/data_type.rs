// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The wire constants of the native name-value list format:
//! the flag the
//!  kernel expects,
//! the `data_type_t` numbers,
//! the pair and list header sizes,
//!  the alignment rule and the nesting bound.
//! Why:
//!  The numbers come from OpenZFS's `nvpair.h` and `nvpair.c`,
//! and a wrong one
//!  makes the kernel read a value as another type.
//! Naming each here keeps the
//!  encoder and decoder reading one table,
//! and the tests pin the table against
//!  the values OpenZFS documents.
//!
//! The native stream is a 4-byte header,
//! then the root list's version and flag,
//! then pairs,
//! then a 4-byte zero:
//!
//! ```text
//! [0..4)   u8 encoding (0 native), u8 byte order (1 little), 2 reserved
//! [4..12)  root list only: i32 version, u32 nvflag
//! pairs:   i32 nvp_size, i16 name length with NUL, i16 reserved,
//!          i32 value element count, i32 data_type_t,
//!          name bytes with NUL, aligned to 8,
//!          value bytes, aligned to 8
//! end:     i32 zero
//! ```

/// `NV_UNIQUE_NAME`:
/// adding a name twice replaces the earlier pair.
pub const NV_UNIQUE_NAME: u32 = 0x1;

/// Bytes of one pair's fixed header.
pub const NVPAIR_HDR_SIZE: usize = 16;

/// Bytes of the in-memory `nvlist_t` copy an embedded list carries.
pub const NVLIST_STRUCT_SIZE: usize = 24;

/// Nested lists deeper than this are refused,
/// so a hostile reply cannot
/// recurse the decoder without bound.
pub const MAX_DEPTH: usize = 64;

/// `DATA_TYPE_BOOLEAN`:
/// a valueless flag whose presence is the information.
pub const DATA_TYPE_BOOLEAN: i32 = 1;
/// `DATA_TYPE_BYTE`.
pub const DATA_TYPE_BYTE: i32 = 2;
/// `DATA_TYPE_INT16`.
pub const DATA_TYPE_INT16: i32 = 3;
/// `DATA_TYPE_UINT16`.
pub const DATA_TYPE_UINT16: i32 = 4;
/// `DATA_TYPE_INT32`.
pub const DATA_TYPE_INT32: i32 = 5;
/// `DATA_TYPE_UINT32`.
pub const DATA_TYPE_UINT32: i32 = 6;
/// `DATA_TYPE_INT64`.
pub const DATA_TYPE_INT64: i32 = 7;
/// `DATA_TYPE_UINT64`.
pub const DATA_TYPE_UINT64: i32 = 8;
/// `DATA_TYPE_STRING`.
pub const DATA_TYPE_STRING: i32 = 9;
/// `DATA_TYPE_BYTE_ARRAY`.
pub const DATA_TYPE_BYTE_ARRAY: i32 = 10;
/// `DATA_TYPE_INT16_ARRAY`.
pub const DATA_TYPE_INT16_ARRAY: i32 = 11;
/// `DATA_TYPE_UINT16_ARRAY`.
pub const DATA_TYPE_UINT16_ARRAY: i32 = 12;
/// `DATA_TYPE_INT32_ARRAY`.
pub const DATA_TYPE_INT32_ARRAY: i32 = 13;
/// `DATA_TYPE_UINT32_ARRAY`.
pub const DATA_TYPE_UINT32_ARRAY: i32 = 14;
/// `DATA_TYPE_INT64_ARRAY`.
pub const DATA_TYPE_INT64_ARRAY: i32 = 15;
/// `DATA_TYPE_UINT64_ARRAY`.
pub const DATA_TYPE_UINT64_ARRAY: i32 = 16;
/// `DATA_TYPE_STRING_ARRAY`.
pub const DATA_TYPE_STRING_ARRAY: i32 = 17;
/// `DATA_TYPE_HRTIME`:
/// a 64-bit nanosecond count.
pub const DATA_TYPE_HRTIME: i32 = 18;
/// `DATA_TYPE_NVLIST`:
/// an embedded list.
pub const DATA_TYPE_NVLIST: i32 = 19;
/// `DATA_TYPE_NVLIST_ARRAY`:
/// an array of embedded lists.
pub const DATA_TYPE_NVLIST_ARRAY: i32 = 20;
/// `DATA_TYPE_BOOLEAN_VALUE`:
/// a 32-bit boolean.
pub const DATA_TYPE_BOOLEAN_VALUE: i32 = 21;
/// `DATA_TYPE_INT8`.
pub const DATA_TYPE_INT8: i32 = 22;
/// `DATA_TYPE_UINT8`.
pub const DATA_TYPE_UINT8: i32 = 23;
/// `DATA_TYPE_BOOLEAN_ARRAY`.
pub const DATA_TYPE_BOOLEAN_ARRAY: i32 = 24;
/// `DATA_TYPE_INT8_ARRAY`.
pub const DATA_TYPE_INT8_ARRAY: i32 = 25;
/// `DATA_TYPE_UINT8_ARRAY`.
pub const DATA_TYPE_UINT8_ARRAY: i32 = 26;
/// `DATA_TYPE_DOUBLE`.
pub const DATA_TYPE_DOUBLE: i32 = 27;

/// Round `bytes` up to the next multiple of 8,
/// the format's `NV_ALIGN`.
pub const fn align8(bytes: usize) -> usize {
    return (bytes + 7) & !7;
}

/// The only nvlist version this crate reads and writes.
pub const SUPPORTED_VERSION: i32 = 0;

/// Refuse a list version this crate does not know.
pub fn check_version(version: i32) -> Result<(), super::error::NvError> {
    if version != SUPPORTED_VERSION {
        return Err(super::error::NvError::UnsupportedVersion { version });
    }
    return Ok(());
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "data_type_tests.rs"]
mod tests;
