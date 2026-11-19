// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  Encode a whole list:
//! the 4-byte stream header,
//! the root list's version and
//!  flag,
//! every pair,
//! and the format's zero terminator.
//! Why:
//!  The kernel's ioctl interface accepts only the native encoding and only
//!  version 0,
//! so the header is written here rather than left to a caller.
//!  A list that claims another version would make the kernel read the pair
//!  stream differently,
//! so it is refused before a byte is written.

/// The version this codec reads and writes.
use super::data_type::check_version;
/// The pair writer that emits an embedded list children.
use super::encode_pair::encode_pairs;
/// The codec own refusal type.
use super::error::NvError;
/// The list the writer encodes.
use super::value::NvList;

/// The stream header the kernel expects on input:
/// native encoding,
/// little
/// byte order,
/// two reserved zero bytes.
const NATIVE_HEADER: [u8; 4] = [0, 1, 0, 0];

/// The methods of `NvList`.
impl NvList {
    /// Encode the list in the native format the `/dev/zfs` requests accept.
    ///
    /// # Throws
    ///
    /// [`NvError`] on a version this crate does not write,
    /// an interior NUL in
    /// a name or string value,
    /// a size that does not fit its wire field,
    /// or
    /// nesting past [`super::data_type::MAX_DEPTH`].
    pub fn pack(&self) -> Result<Vec<u8>, NvError> {
        check_version(self.version)?;
        let mut out: Vec<u8> = Vec::with_capacity(16 + self.pairs.len() * 32);
        out.extend_from_slice(&NATIVE_HEADER);
        out.extend_from_slice(&self.version.to_le_bytes());
        out.extend_from_slice(&self.nvflag.to_le_bytes());
        encode_pairs(&mut out, &self.pairs, 0)?;
        return Ok(out);
    }
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "pack_tests.rs"]
mod tests;
