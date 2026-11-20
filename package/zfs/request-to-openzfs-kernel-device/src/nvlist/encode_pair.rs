// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The recursive half of the encoder:
//! one pair's header,
//! value and padding,
//!  the patched sizes,
//! and the child stream of an embedded list.
//! Why:
//!  An embedded list's pairs follow the parent pair's blob,
//! so writing a list
//!  and writing a pair sequence call each other.
//! Keeping both in one file
//!  stops the file tree from showing a loop between two files;
//! every non-list
//!  value is written by the scalar writer beside this file.
//!  A size that does not fit its wire field would wrap on the cast and desync
//!  the kernel's framing of the whole list,
//! so it is refused before the patch.

// The wire constants,
// type numbers,
// alignment and version check.
use super::data_type::{MAX_DEPTH, NVLIST_STRUCT_SIZE, NVPAIR_HDR_SIZE, align8, check_version};
// The scalar writer and its NUL checks.
use super::encode_scalar_value::{contains_nul, value_contains_nul, write_scalar_value};
// The codec own refusal type.
use super::error::NvError;
// The pair and value types the writer encodes.
use super::value::{NvData, NvPair};

/// Encode a pair sequence and its 4-byte zero terminator.
///
/// # Throws
///
/// [`NvError`] from [`encode_pair`],
/// or [`NvError::TooDeep`] past
/// [`MAX_DEPTH`].
pub fn encode_pairs(out: &mut Vec<u8>, pairs: &[NvPair], depth: usize) -> Result<(), NvError> {
    if depth > MAX_DEPTH {
        return Err(NvError::TooDeep);
    }
    for pair in pairs {
        encode_pair(out, &pair.name, &pair.data, depth)?;
    }
    out.extend_from_slice(&0i32.to_le_bytes());
    return Ok(());
}

/// Encode one pair and,
/// for an embedded list,
/// its child pair stream.
///
/// # Throws
///
/// [`NvError`] on an embedded NUL,
/// an unknown embedded list version,
/// or a size
/// that does not fit its field.
pub fn encode_pair(
    out: &mut Vec<u8>,
    name: &str,
    data: &NvData,
    depth: usize,
) -> Result<(), NvError> {
    match data {
        NvData::List { value: list } => check_version(list.version)?,
        NvData::ListArray { value: lists } => {
            for list in lists {
                check_version(list.version)?;
            }
        }
        _ => {}
    }
    if contains_nul(name) || value_contains_nul(data) {
        return Err(NvError::EmbeddedNul {
            name: name.to_string(),
        });
    }
    let name_size: usize = name.len() + 1;
    if name_size > i16::MAX as usize {
        return Err(NvError::TooBigToPack {
            pair: name.to_string(),
        });
    }
    let value_at: usize = align8(NVPAIR_HDR_SIZE + name_size);
    let pair_at: usize = out.len();
    // Reserve the header, then the NUL-terminated name padded to 8 bytes.
    out.extend_from_slice(&0i32.to_le_bytes());
    out.extend_from_slice(&(name_size as i16).to_le_bytes());
    out.extend_from_slice(&0i16.to_le_bytes());
    out.extend_from_slice(&0i32.to_le_bytes());
    out.extend_from_slice(&0i32.to_le_bytes());
    out.extend_from_slice(name.as_bytes());
    out.resize(pair_at + value_at, 0);
    let value_start: usize = out.len();
    let (data_type, elements): (i32, usize) = write_value(out, data);
    let padded_value: usize = align8(out.len() - value_start);
    out.resize(value_start + padded_value, 0);
    let size: usize = value_at + padded_value;
    if size > i32::MAX as usize || elements > i32::MAX as usize {
        return Err(NvError::TooBigToPack {
            pair: name.to_string(),
        });
    }
    out[pair_at..pair_at + 4].copy_from_slice(&(size as i32).to_le_bytes());
    out[pair_at + 8..pair_at + 12].copy_from_slice(&(elements as i32).to_le_bytes());
    out[pair_at + 12..pair_at + 16].copy_from_slice(&data_type.to_le_bytes());
    match data {
        NvData::List { value: list } => encode_pairs(out, &list.pairs, depth + 1)?,
        NvData::ListArray { value: lists } => {
            for list in lists {
                encode_pairs(out, &list.pairs, depth + 1)?;
            }
        }
        _ => {}
    }
    return Ok(());
}

/// Write a pair's value:
/// an embedded list's header or a scalar's bytes,
/// returning the `(data_type_t, nvp_value_elem)` the header carries.
fn write_value(out: &mut Vec<u8>, data: &NvData) -> (i32, usize) {
    return match data {
        NvData::List { value: list } => {
            write_nvlist_header(out, list.version, list.nvflag);
            (super::data_type::DATA_TYPE_NVLIST, 1)
        }
        NvData::ListArray { value: lists } => {
            out.resize(out.len() + lists.len() * size_of::<u64>(), 0);
            for list in lists {
                write_nvlist_header(out, list.version, list.nvflag);
            }
            (super::data_type::DATA_TYPE_NVLIST_ARRAY, lists.len())
        }
        other => write_scalar_value(out, other),
    };
}

/// Append the 24-byte in-memory `nvlist_t` copy an embedded value carries;
/// only its leading version and flag are meaningful on the wire.
fn write_nvlist_header(out: &mut Vec<u8>, version: i32, nvflag: u32) {
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&nvflag.to_le_bytes());
    out.resize(out.len() + (NVLIST_STRUCT_SIZE - 8), 0);
}
