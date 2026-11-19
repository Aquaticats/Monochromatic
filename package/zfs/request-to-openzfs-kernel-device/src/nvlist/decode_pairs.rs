// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The recursive half of the decoder:
//! the pair loop,
//! one pair's framing,
//! and
//!  the two value arms (an embedded list and an array of them) that read their
//!  children from the stream.
//! Why:
//!  The format puts an embedded list's pairs after the parent pair's own blob,
//!  so a list value and the pair loop call each other.
//! Keeping both in one file
//!  is what stops the file tree from showing a loop between two files:
//! the
//!  recursion is the format's,
//! contained here,
//! and every other value is
//!  decoded by the bounded scalar decoder beside this file.

/// The wire constants,
/// type numbers,
/// alignment and version check.
use super::data_type::{NVLIST_STRUCT_SIZE, NVPAIR_HDR_SIZE, align8, check_version};
/// The decoder for values that stay inside their pair.
use super::decode_scalar_value::decode_scalar_value;
/// The codec own refusal type.
use super::error::NvError;
/// The checked position in a packed list.
use super::stream_cursor::Cursor;
/// The decoded list,
/// pair and value types.
use super::value::{NvData, NvList, NvPair};
/// The byte width of the element types.
use std::mem::size_of;

/// Decode a pair sequence up to and including the format's 4-byte zero.
///
/// # Throws
///
/// [`NvError`] on a malformed pair header or name,
/// or on nesting past
/// [`super::data_type::MAX_DEPTH`].
pub fn decode_pairs(cursor: &mut Cursor<'_>, depth: usize) -> Result<Vec<NvPair>, NvError> {
    if depth > super::data_type::MAX_DEPTH {
        return Err(NvError::TooDeep);
    }
    let mut pairs: Vec<NvPair> = Vec::new();
    loop {
        let size: i32 = cursor.peek_i32()?;
        if size == 0 {
            cursor.take(size_of::<i32>())?;
            return Ok(pairs);
        }
        if size < NVPAIR_HDR_SIZE as i32 || size as usize > cursor.remaining() {
            return Err(NvError::BadPairSize {
                at: cursor.position,
                size: i64::from(size),
            });
        }
        pairs.push(decode_pair(cursor, depth)?);
    }
}

/// Decode one pair,
/// its blob and (for an embedded list) its child stream.
///
/// # Throws
///
/// [`NvError`] on a malformed name,
/// a negative element count,
/// or a value that
/// runs past the pair.
fn decode_pair(cursor: &mut Cursor<'_>, depth: usize) -> Result<NvPair, NvError> {
    let size: i32 = cursor.peek_i32()?;
    let blob: &[u8] = cursor.take(size as usize)?;
    let name_size: i16 = i16::from_le_bytes(le(&blob[4..6]));
    let elements: i32 = i32::from_le_bytes(le(&blob[8..12]));
    let data_type: i32 = i32::from_le_bytes(le(&blob[12..16]));
    /*
    `i_validate_nvpair_name` requires strlen(name) == nvp_name_sz - 1: the
    name fits the blob, ends in a NUL, and carries no earlier NUL.
    */
    if name_size < 1 || NVPAIR_HDR_SIZE + name_size as usize > blob.len() {
        return Err(NvError::BadName {
            name_length: name_size,
        });
    }
    let name_end: usize = NVPAIR_HDR_SIZE + name_size as usize;
    if blob[name_end - 1] != 0 {
        return Err(NvError::BadName {
            name_length: name_size,
        });
    }
    let name_bytes: &[u8] = &blob[NVPAIR_HDR_SIZE..name_end - 1];
    if name_bytes.contains(&0) {
        return Err(NvError::BadName {
            name_length: name_size,
        });
    }
    let name: String = String::from_utf8_lossy(name_bytes).into_owned();
    if elements < 0 {
        return Err(NvError::BadArrayCount {
            elements: i64::from(elements),
        });
    }
    let value: &[u8] = blob.get(align8(name_end)..).unwrap_or(&[]);
    let data: NvData = decode_value(cursor, &name, data_type, elements as usize, value, depth)?;
    return Ok(NvPair { name, data });
}

/// Decode one pair's value,
/// reading embedded children from the stream.
///
/// # Throws
///
/// [`NvError`] from the scalar decoder,
/// or on an unknown embedded list
/// version.
fn decode_value(
    cursor: &mut Cursor<'_>,
    name: &str,
    data_type: i32,
    elements: usize,
    value: &[u8],
    depth: usize,
) -> Result<NvData, NvError> {
    return match data_type {
        super::data_type::DATA_TYPE_NVLIST => {
            let header: &[u8] = value.get(..NVLIST_STRUCT_SIZE).ok_or_else(|| {
                return NvError::ValueOverflow {
                    name: name.to_string(),
                    need: NVLIST_STRUCT_SIZE,
                    have: value.len(),
                };
            })?;
            let version: i32 = i32::from_le_bytes(le(&header[..4]));
            check_version(version)?;
            let nvflag: u32 = u32::from_le_bytes(le(&header[4..8]));
            let pairs: Vec<NvPair> = decode_pairs(cursor, depth + 1)?;
            Ok(NvData::List {
                value: NvList {
                    version,
                    nvflag,
                    pairs,
                },
            })
        }
        super::data_type::DATA_TYPE_NVLIST_ARRAY => {
            decode_list_array(cursor, name, elements, value, depth)
        }
        other => decode_scalar_value(name, other, elements, value),
    };
}

/// `DATA_TYPE_NVLIST_ARRAY`:
/// `elements` 8-byte placeholders,
/// then that many
/// 24-byte list headers,
/// whose pair streams follow in the stream in order.
///
/// # Throws
///
/// [`NvError`] on a value region too short,
/// an unknown version,
/// or a malformed
/// child stream.
fn decode_list_array(
    cursor: &mut Cursor<'_>,
    name: &str,
    elements: usize,
    value: &[u8],
    depth: usize,
) -> Result<NvData, NvError> {
    let total: usize = elements
        .checked_mul(size_of::<u64>() + NVLIST_STRUCT_SIZE)
        .ok_or(NvError::Overflow { elements })?;
    let Some(headers) = value.get(..total) else {
        return Err(NvError::ValueOverflow {
            name: name.to_string(),
            need: total,
            have: value.len(),
        });
    };
    let mut lists: Vec<NvList> = Vec::with_capacity(elements);
    for index in 0..elements {
        let at: usize = elements * size_of::<u64>() + index * NVLIST_STRUCT_SIZE;
        let version: i32 = i32::from_le_bytes(le(&headers[at..at + 4]));
        check_version(version)?;
        let nvflag: u32 = u32::from_le_bytes(le(&headers[at + 4..at + 8]));
        let pairs: Vec<NvPair> = decode_pairs(cursor, depth + 1)?;
        lists.push(NvList {
            version,
            nvflag,
            pairs,
        });
    }
    return Ok(NvData::ListArray { value: lists });
}

/// Take a fixed-width little-endian group out of a checked slice.
fn le<const N: usize>(bytes: &[u8]) -> [u8; N] {
    return <[u8; N]>::try_from(bytes).expect("the slice was checked to hold exactly N bytes");
}
