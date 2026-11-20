// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  A position in a packed list whose every read is checked against the
//!  buffer before it indexes.
//! Why:
//!  Every offset in a packed reply is attacker-controlled,
//! so a read that
//!  trusted its own arithmetic would index past the buffer.
//! One cursor that
//!  owns the position keeps the check in one place.

// The codec own refusal type.
use super::error::NvError;

/// A position in a packed list,
/// with every read length-checked.
pub struct Cursor<'a> {
    /// The bytes being read.
    pub buffer: &'a [u8],
    /// Offset of the next unread byte.
    pub position: usize,
}

/// The methods of `Cursor`.
impl<'a> Cursor<'a> {
    /// A cursor at the start of `buffer`.
    pub fn new(buffer: &'a [u8]) -> Cursor<'a> {
        return Cursor {
            buffer,
            position: 0,
        };
    }

    /// Bytes left after the cursor.
    pub fn remaining(&self) -> usize {
        return self.buffer.len() - self.position;
    }

    /// Take the next `count` bytes.
    ///
    /// # Throws
    ///
    /// [`NvError::Truncated`] when fewer remain.
    pub fn take(&mut self, count: usize) -> Result<&'a [u8], NvError> {
        if self.remaining() < count {
            return Err(NvError::Truncated {
                at: self.position,
                need: count,
                have: self.remaining(),
            });
        }
        let bytes: &'a [u8] = &self.buffer[self.position..self.position + count];
        self.position += count;
        return Ok(bytes);
    }

    /// Read the next little-endian `i32`.
    ///
    /// # Throws
    ///
    /// [`NvError::Truncated`] when fewer than 4 bytes remain.
    pub fn read_i32(&mut self) -> Result<i32, NvError> {
        let bytes: &[u8] = self.take(size_of::<i32>())?;
        return Ok(i32::from_le_bytes(
            <[u8; 4]>::try_from(bytes).expect("4 bytes were taken"),
        ));
    }

    /// Read the next little-endian `u32`.
    ///
    /// # Throws
    ///
    /// [`NvError::Truncated`] when fewer than 4 bytes remain.
    pub fn read_u32(&mut self) -> Result<u32, NvError> {
        let bytes: &[u8] = self.take(size_of::<u32>())?;
        return Ok(u32::from_le_bytes(
            <[u8; 4]>::try_from(bytes).expect("4 bytes were taken"),
        ));
    }

    /// The next little-endian `i32` without consuming it.
    ///
    /// # Throws
    ///
    /// [`NvError::Truncated`] when fewer than 4 bytes remain.
    pub fn peek_i32(&self) -> Result<i32, NvError> {
        if self.remaining() < size_of::<i32>() {
            return Err(NvError::Truncated {
                at: self.position,
                need: 4,
                have: self.remaining(),
            });
        }
        let bytes: &[u8] = &self.buffer[self.position..self.position + size_of::<i32>()];
        return Ok(i32::from_le_bytes(
            <[u8; 4]>::try_from(bytes).expect("4 bytes were checked"),
        ));
    }
}
