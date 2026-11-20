// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The non-list half of the encoder:
//! the wire width and bytes of every scalar,
//!  string,
//! array and unknown value.
//! Why:
//!  A value's width decides the pair's `nvp_value_elem`,
//! and the pair's size
//!  decides where the next pair starts;
//! one table for every non-list type
//!  keeps the encoder's widths and the decoder's in step.
//!  Embedded lists are written by the pair encoder beside this file,
//! because
//!  their children go into the parent pair's own stream.

// The wire constants,
// type numbers,
// alignment and version check.
use super::data_type::{
    DATA_TYPE_BOOLEAN, DATA_TYPE_BOOLEAN_ARRAY, DATA_TYPE_BOOLEAN_VALUE, DATA_TYPE_BYTE,
    DATA_TYPE_BYTE_ARRAY, DATA_TYPE_DOUBLE, DATA_TYPE_HRTIME, DATA_TYPE_INT8, DATA_TYPE_INT8_ARRAY,
    DATA_TYPE_INT16, DATA_TYPE_INT16_ARRAY, DATA_TYPE_INT32, DATA_TYPE_INT32_ARRAY,
    DATA_TYPE_INT64, DATA_TYPE_INT64_ARRAY, DATA_TYPE_STRING, DATA_TYPE_STRING_ARRAY,
    DATA_TYPE_UINT8, DATA_TYPE_UINT8_ARRAY, DATA_TYPE_UINT16, DATA_TYPE_UINT16_ARRAY,
    DATA_TYPE_UINT32, DATA_TYPE_UINT32_ARRAY, DATA_TYPE_UINT64, DATA_TYPE_UINT64_ARRAY,
};
// The decoded value type.
use super::value::NvData;
// The byte width of the element types.
use std::mem::size_of;

/// Append the unpadded value bytes of a non-list `data`,
/// returning its
/// `(data_type_t, nvp_value_elem)`.
pub fn write_scalar_value(out: &mut Vec<u8>, data: &NvData) -> (i32, usize) {
    return match data {
        NvData::BooleanFlag => (DATA_TYPE_BOOLEAN, 0),
        NvData::Boolean { value } => {
            out.extend_from_slice(&i32::from(*value).to_le_bytes());
            (DATA_TYPE_BOOLEAN_VALUE, 1)
        }
        NvData::Byte { value } => {
            out.push(*value);
            (DATA_TYPE_BYTE, 1)
        }
        NvData::Int8 { value } => {
            out.push(*value as u8);
            (DATA_TYPE_INT8, 1)
        }
        NvData::Uint8 { value } => {
            out.push(*value);
            (DATA_TYPE_UINT8, 1)
        }
        NvData::Int16 { value } => {
            out.extend_from_slice(&value.to_le_bytes());
            (DATA_TYPE_INT16, 1)
        }
        NvData::Uint16 { value } => {
            out.extend_from_slice(&value.to_le_bytes());
            (DATA_TYPE_UINT16, 1)
        }
        NvData::Int32 { value } => {
            out.extend_from_slice(&value.to_le_bytes());
            (DATA_TYPE_INT32, 1)
        }
        NvData::Uint32 { value } => {
            out.extend_from_slice(&value.to_le_bytes());
            (DATA_TYPE_UINT32, 1)
        }
        NvData::Int64 { value } => {
            out.extend_from_slice(&value.to_le_bytes());
            (DATA_TYPE_INT64, 1)
        }
        NvData::Uint64 { value } => {
            out.extend_from_slice(&value.to_le_bytes());
            (DATA_TYPE_UINT64, 1)
        }
        NvData::HrTime { value } => {
            out.extend_from_slice(&value.to_le_bytes());
            (DATA_TYPE_HRTIME, 1)
        }
        NvData::Double { value } => {
            out.extend_from_slice(&value.to_le_bytes());
            (DATA_TYPE_DOUBLE, 1)
        }
        NvData::Str { value } => {
            out.extend_from_slice(value.as_bytes());
            out.push(0);
            (DATA_TYPE_STRING, 1)
        }
        NvData::ByteArray { value: bytes } => {
            out.extend_from_slice(bytes);
            (DATA_TYPE_BYTE_ARRAY, bytes.len())
        }
        NvData::Uint8Array { value: bytes } => {
            out.extend_from_slice(bytes);
            (DATA_TYPE_UINT8_ARRAY, bytes.len())
        }
        NvData::Int8Array { value: bytes } => {
            out.extend(bytes.iter().map(|byte| {
                return *byte as u8;
            }));
            (DATA_TYPE_INT8_ARRAY, bytes.len())
        }
        NvData::Int16Array { value: values } => {
            write_array(out, values, i16::to_le_bytes);
            (DATA_TYPE_INT16_ARRAY, values.len())
        }
        NvData::Uint16Array { value: values } => {
            write_array(out, values, u16::to_le_bytes);
            (DATA_TYPE_UINT16_ARRAY, values.len())
        }
        NvData::Int32Array { value: values } => {
            write_array(out, values, i32::to_le_bytes);
            (DATA_TYPE_INT32_ARRAY, values.len())
        }
        NvData::Uint32Array { value: values } => {
            write_array(out, values, u32::to_le_bytes);
            (DATA_TYPE_UINT32_ARRAY, values.len())
        }
        NvData::Int64Array { value: values } => {
            write_array(out, values, i64::to_le_bytes);
            (DATA_TYPE_INT64_ARRAY, values.len())
        }
        NvData::Uint64Array { value: values } => {
            write_array(out, values, u64::to_le_bytes);
            (DATA_TYPE_UINT64_ARRAY, values.len())
        }
        NvData::BooleanArray { value: values } => {
            write_array(out, values, |value| {
                return i32::from(value).to_le_bytes();
            });
            (DATA_TYPE_BOOLEAN_ARRAY, values.len())
        }
        NvData::StrArray { value: strings } => {
            out.resize(out.len() + strings.len() * size_of::<u64>(), 0);
            for string in strings {
                out.extend_from_slice(string.as_bytes());
                out.push(0);
            }
            (DATA_TYPE_STRING_ARRAY, strings.len())
        }
        // An embedded list is written by the pair encoder, which also appends
        // its child stream; reaching here would mean the caller lost that.
        NvData::List { value: _ } | NvData::ListArray { value: _ } => (0, 0),
        NvData::Unknown {
            dtype,
            elements,
            raw,
        } => {
            out.extend_from_slice(raw);
            (*dtype, *elements)
        }
    };
}

/// Append `values` in `to_bytes` order,
/// one after another.
fn write_array<T: Copy, const N: usize>(
    out: &mut Vec<u8>,
    values: &[T],
    to_bytes: impl Fn(T) -> [u8; N],
) {
    out.reserve(values.len() * N);
    for value in values {
        out.extend_from_slice(&to_bytes(*value));
    }
}

/// Whether `text` carries a NUL byte,
/// which would end a packed string early.
pub(super) fn contains_nul(text: &str) -> bool {
    return text.as_bytes().contains(&0);
}

/// Whether a string or string-array value carries a NUL byte.
pub(super) fn value_contains_nul(data: &NvData) -> bool {
    return match data {
        NvData::Str { value } => contains_nul(value),
        NvData::StrArray { value: values } => values.iter().any(|value| {
            return contains_nul(value);
        }),
        _ => false,
    };
}
