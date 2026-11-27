// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The wire type number of one decoded value:
//! the `data_type_t` the kernel stores beside a pair,
//!  and that the encoder writes for the value it holds.
//! Why:
//!  A value and its type number must agree,
//! so the number is answered from the variant itself rather than kept beside it
//!  in the pair,
//! and a variant this crate cannot name answers with the number the kernel
//!  declared for it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function dataType(value: NvData): number;
//! ```

/// The wire constants each variant answers with.
use super::data_type::{
    DATA_TYPE_BOOLEAN,
    DATA_TYPE_BOOLEAN_ARRAY,
    DATA_TYPE_BOOLEAN_VALUE,
    DATA_TYPE_BYTE,
    DATA_TYPE_BYTE_ARRAY,
    DATA_TYPE_DOUBLE,
    DATA_TYPE_HRTIME,
    DATA_TYPE_INT8,
    DATA_TYPE_INT8_ARRAY,
    DATA_TYPE_INT16,
    DATA_TYPE_INT16_ARRAY,
    DATA_TYPE_INT32,
    DATA_TYPE_INT32_ARRAY,
    DATA_TYPE_INT64,
    DATA_TYPE_INT64_ARRAY,
    DATA_TYPE_NVLIST,
    DATA_TYPE_NVLIST_ARRAY,
    DATA_TYPE_STRING,
    DATA_TYPE_STRING_ARRAY,
    DATA_TYPE_UINT8,
    DATA_TYPE_UINT8_ARRAY,
    DATA_TYPE_UINT16,
    DATA_TYPE_UINT16_ARRAY,
    DATA_TYPE_UINT32,
    DATA_TYPE_UINT32_ARRAY,
    DATA_TYPE_UINT64,
    DATA_TYPE_UINT64_ARRAY,
};
/// The decoded value whose type number this answers.
use super::value::NvData;

/// The methods of `NvData`.
impl NvData {
    /// The `data_type_t` number this value is encoded with.
    pub fn data_type(&self) -> i32 {
        return match self {
            NvData::BooleanFlag => DATA_TYPE_BOOLEAN,
            NvData::Boolean {
                value: _,
            } => DATA_TYPE_BOOLEAN_VALUE,
            NvData::Byte {
                value: _,
            } => DATA_TYPE_BYTE,
            NvData::Int8 {
                value: _,
            } => DATA_TYPE_INT8,
            NvData::Uint8 {
                value: _,
            } => DATA_TYPE_UINT8,
            NvData::Int16 {
                value: _,
            } => DATA_TYPE_INT16,
            NvData::Uint16 {
                value: _,
            } => DATA_TYPE_UINT16,
            NvData::Int32 {
                value: _,
            } => DATA_TYPE_INT32,
            NvData::Uint32 {
                value: _,
            } => DATA_TYPE_UINT32,
            NvData::Int64 {
                value: _,
            } => DATA_TYPE_INT64,
            NvData::Uint64 {
                value: _,
            } => DATA_TYPE_UINT64,
            NvData::HrTime {
                value: _,
            } => DATA_TYPE_HRTIME,
            NvData::Double {
                value: _,
            } => DATA_TYPE_DOUBLE,
            NvData::Str {
                value: _,
            } => DATA_TYPE_STRING,
            NvData::ByteArray {
                value: _,
            } => DATA_TYPE_BYTE_ARRAY,
            NvData::Int8Array {
                value: _,
            } => DATA_TYPE_INT8_ARRAY,
            NvData::Uint8Array {
                value: _,
            } => DATA_TYPE_UINT8_ARRAY,
            NvData::Int16Array {
                value: _,
            } => DATA_TYPE_INT16_ARRAY,
            NvData::Uint16Array {
                value: _,
            } => DATA_TYPE_UINT16_ARRAY,
            NvData::Int32Array {
                value: _,
            } => DATA_TYPE_INT32_ARRAY,
            NvData::Uint32Array {
                value: _,
            } => DATA_TYPE_UINT32_ARRAY,
            NvData::Int64Array {
                value: _,
            } => DATA_TYPE_INT64_ARRAY,
            NvData::Uint64Array {
                value: _,
            } => DATA_TYPE_UINT64_ARRAY,
            NvData::BooleanArray {
                value: _,
            } => DATA_TYPE_BOOLEAN_ARRAY,
            NvData::StrArray {
                value: _,
            } => DATA_TYPE_STRING_ARRAY,
            NvData::List {
                value: _,
            } => DATA_TYPE_NVLIST,
            NvData::ListArray {
                value: _,
            } => DATA_TYPE_NVLIST_ARRAY,
            NvData::Unknown {
                dtype,
                ..
            } => *dtype,
        };
    }
}
