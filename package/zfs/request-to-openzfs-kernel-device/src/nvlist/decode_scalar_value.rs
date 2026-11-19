// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  Decode every non-list value:
//! the scalars,
//! strings,
//! arrays and unknown
//!  types whose bytes all live inside their own pair.
//! Why:
//!  Only an embedded list reads past its pair,
//! from the stream cursor;
//! every
//!  other value is bounded by the pair's size,
//! so its decoder needs neither the
//!  cursor nor recursion.
//! Keeping it separate is what lets the recursive pair
//!  loop and this bounded decoder sit in files that do not call back into one
//!  another.

/// The wire constants,
/// type numbers,
/// alignment and version check.
use super::data_type::{
    DATA_TYPE_BOOLEAN, DATA_TYPE_BOOLEAN_ARRAY, DATA_TYPE_BOOLEAN_VALUE, DATA_TYPE_BYTE,
    DATA_TYPE_BYTE_ARRAY, DATA_TYPE_DOUBLE, DATA_TYPE_HRTIME, DATA_TYPE_INT8, DATA_TYPE_INT8_ARRAY,
    DATA_TYPE_INT16, DATA_TYPE_INT16_ARRAY, DATA_TYPE_INT32, DATA_TYPE_INT32_ARRAY,
    DATA_TYPE_INT64, DATA_TYPE_INT64_ARRAY, DATA_TYPE_STRING, DATA_TYPE_STRING_ARRAY,
    DATA_TYPE_UINT8, DATA_TYPE_UINT8_ARRAY, DATA_TYPE_UINT16, DATA_TYPE_UINT16_ARRAY,
    DATA_TYPE_UINT32, DATA_TYPE_UINT32_ARRAY, DATA_TYPE_UINT64, DATA_TYPE_UINT64_ARRAY,
};
/// The codec own refusal type.
use super::error::NvError;
/// The decoded value type.
use super::value::NvData;
/// The byte width of the element types.
use std::mem::size_of;

/// Decode one value that is not an embedded list.
///
/// # Throws
///
/// [`NvError`] when the value region is shorter than the type needs,
/// or a
/// count overflows the size arithmetic.
pub fn decode_scalar_value(
    name: &str,
    data_type: i32,
    elements: usize,
    value: &[u8],
) -> Result<NvData, NvError> {
    let need = |count: usize| -> Result<&[u8], NvError> {
        return value.get(..count).ok_or_else(|| {
            return NvError::ValueOverflow {
                name: name.to_string(),
                need: count,
                have: value.len(),
            };
        });
    };
    let array = |element_size: usize| -> Result<std::slice::ChunksExact<'_, u8>, NvError> {
        let total: usize = elements
            .checked_mul(element_size)
            .ok_or(NvError::Overflow { elements })?;
        return Ok(need(total)?.chunks_exact(element_size));
    };
    return match data_type {
        DATA_TYPE_BOOLEAN => Ok(NvData::BooleanFlag),
        DATA_TYPE_BOOLEAN_VALUE => Ok(NvData::Boolean {
            value: i32::from_le_bytes(le(need(4)?)) != 0,
        }),
        DATA_TYPE_BYTE => Ok(NvData::Byte { value: need(1)?[0] }),
        DATA_TYPE_INT8 => Ok(NvData::Int8 {
            value: need(1)?[0] as i8,
        }),
        DATA_TYPE_UINT8 => Ok(NvData::Uint8 { value: need(1)?[0] }),
        DATA_TYPE_INT16 => Ok(NvData::Int16 {
            value: i16::from_le_bytes(le(need(2)?)),
        }),
        DATA_TYPE_UINT16 => Ok(NvData::Uint16 {
            value: u16::from_le_bytes(le(need(2)?)),
        }),
        DATA_TYPE_INT32 => Ok(NvData::Int32 {
            value: i32::from_le_bytes(le(need(4)?)),
        }),
        DATA_TYPE_UINT32 => Ok(NvData::Uint32 {
            value: u32::from_le_bytes(le(need(4)?)),
        }),
        DATA_TYPE_INT64 => Ok(NvData::Int64 {
            value: i64::from_le_bytes(le(need(8)?)),
        }),
        DATA_TYPE_UINT64 => Ok(NvData::Uint64 {
            value: u64::from_le_bytes(le(need(8)?)),
        }),
        DATA_TYPE_HRTIME => Ok(NvData::HrTime {
            value: i64::from_le_bytes(le(need(8)?)),
        }),
        DATA_TYPE_DOUBLE => Ok(NvData::Double {
            value: f64::from_le_bytes(le(need(8)?)),
        }),
        DATA_TYPE_STRING => Ok(NvData::Str {
            value: read_cstr(value, name)?.0,
        }),
        DATA_TYPE_BYTE_ARRAY => Ok(NvData::ByteArray {
            value: need(elements)?.to_vec(),
        }),
        DATA_TYPE_UINT8_ARRAY => Ok(NvData::Uint8Array {
            value: need(elements)?.to_vec(),
        }),
        DATA_TYPE_INT8_ARRAY => Ok(NvData::Int8Array {
            value: need(elements)?
                .iter()
                .map(|byte| {
                    return *byte as i8;
                })
                .collect(),
        }),
        DATA_TYPE_INT16_ARRAY => Ok(NvData::Int16Array {
            value: array(size_of::<i16>())?
                .map(|bytes| {
                    return i16::from_le_bytes(le(bytes));
                })
                .collect(),
        }),
        DATA_TYPE_UINT16_ARRAY => Ok(NvData::Uint16Array {
            value: array(size_of::<u16>())?
                .map(|bytes| {
                    return u16::from_le_bytes(le(bytes));
                })
                .collect(),
        }),
        DATA_TYPE_INT32_ARRAY => Ok(NvData::Int32Array {
            value: array(size_of::<i32>())?
                .map(|bytes| {
                    return i32::from_le_bytes(le(bytes));
                })
                .collect(),
        }),
        DATA_TYPE_UINT32_ARRAY => Ok(NvData::Uint32Array {
            value: array(size_of::<u32>())?
                .map(|bytes| {
                    return u32::from_le_bytes(le(bytes));
                })
                .collect(),
        }),
        DATA_TYPE_INT64_ARRAY => Ok(NvData::Int64Array {
            value: array(size_of::<i64>())?
                .map(|bytes| {
                    return i64::from_le_bytes(le(bytes));
                })
                .collect(),
        }),
        DATA_TYPE_UINT64_ARRAY => Ok(NvData::Uint64Array {
            value: array(size_of::<u64>())?
                .map(|bytes| {
                    return u64::from_le_bytes(le(bytes));
                })
                .collect(),
        }),
        DATA_TYPE_BOOLEAN_ARRAY => Ok(NvData::BooleanArray {
            value: array(size_of::<u32>())?
                .map(|bytes| {
                    return u32::from_le_bytes(le(bytes)) != 0;
                })
                .collect(),
        }),
        DATA_TYPE_STRING_ARRAY => decode_string_array(name, elements, value),
        other => Ok(NvData::Unknown {
            dtype: other,
            raw: value.to_vec(),
        }),
    };
}

/// `DATA_TYPE_STRING_ARRAY`:
/// `elements` 8-byte placeholders,
/// then that many
/// NUL-terminated strings packed one after another.
///
/// # Throws
///
/// [`NvError::ValueOverflow`] when the placeholders do not fit,
/// and
/// [`NvError::BadString`] when a string has no NUL.
fn decode_string_array(name: &str, elements: usize, value: &[u8]) -> Result<NvData, NvError> {
    let placeholders: usize = elements
        .checked_mul(size_of::<u64>())
        .ok_or(NvError::Overflow { elements })?;
    let Some(mut rest) = value.get(placeholders..) else {
        return Err(NvError::ValueOverflow {
            name: name.to_string(),
            need: placeholders,
            have: value.len(),
        });
    };
    let mut strings: Vec<String> = Vec::with_capacity(elements);
    for _ in 0..elements {
        let (string, consumed) = read_cstr(rest, name)?;
        strings.push(string);
        rest = &rest[consumed..];
    }
    return Ok(NvData::StrArray { value: strings });
}

/// Read a NUL-terminated string from the start of `buffer`,
/// returning it with
/// the bytes consumed,
/// terminating NUL included.
///
/// # Throws
///
/// [`NvError::BadString`] when no NUL follows inside `buffer`.
pub fn read_cstr(buffer: &[u8], pair_name: &str) -> Result<(String, usize), NvError> {
    let Some(end) = buffer.iter().position(|byte| {
        return *byte == 0;
    }) else {
        return Err(NvError::BadString {
            pair: pair_name.to_string(),
        });
    };
    return Ok((
        String::from_utf8_lossy(&buffer[..end]).into_owned(),
        end + 1,
    ));
}

/// Take a fixed-width little-endian group out of a checked slice.
fn le<const N: usize>(bytes: &[u8]) -> [u8; N] {
    return <[u8; N]>::try_from(bytes).expect("the slice was checked to hold exactly N bytes");
}
