// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The entry point of the native decoder:
//! check the stream header,
//! read the
//!  root list's version and flag,
//! then hand the pairs to the pair loop.
//! Why:
//!  The kernel's ioctl replies are native streams;
//! anything else is refused
//!  rather than guessed at,
//! because another encoding would make every offset
//!  in the loop mean something else.

// The version this codec reads and writes.
use super::data_type::check_version;
// The pair loop that reads an embedded list children.
use super::decode_pairs::decode_pairs;
// The codec own refusal type.
use super::error::NvError;
// The checked position in a packed list.
use super::stream_cursor::Cursor;
// The list and pair types the reader builds.
use super::value::{NvList, NvPair};

/// The methods of `NvList`.
impl NvList {
    /// Decode one packed native list,
    /// with its stream header.
    ///
    /// # Throws
    ///
    /// [`NvError`] when the header names another encoding or byte order,
    /// the
    /// version is not zero,
    /// a pair's framing is malformed,
    /// a value runs past
    /// its pair,
    /// or nesting passes `MAX_DEPTH`.
    pub fn unpack(buffer: &[u8]) -> Result<NvList, NvError> {
        let mut cursor: Cursor<'_> = Cursor::new(buffer);
        let header: &[u8] = cursor.take(4)?;
        if header[0] != 0 {
            return Err(NvError::UnsupportedEncoding {
                encoding: header[0],
            });
        }
        if header[1] != 1 {
            return Err(NvError::UnsupportedEndian {
                byte_order: header[1],
            });
        }
        let version: i32 = cursor.read_i32()?;
        check_version(version)?;
        let nvflag: u32 = cursor.read_u32()?;
        let pairs: Vec<NvPair> = decode_pairs(&mut cursor, 0)?;
        return Ok(NvList {
            version,
            nvflag,
            pairs,
        });
    }
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "unpack_tests.rs"]
mod tests;
