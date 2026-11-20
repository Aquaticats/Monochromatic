// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  Why a packed name-value list cannot be decoded or encoded,
//! and the text
//!  that names the offset,
//! pair or field the problem sits in.
//! Why:
//!  Every byte here can come from a kernel reply or from a caller's input,
//! and
//!  neither is trusted:
//! a truncated buffer,
//! a pair that claims a negative
//!  element count or a name that runs past its blob must be refused,
//! not read
//!  past.
//! The variants carry the offsets so a diagnostic points at the byte.

/// The formatting traits the diagnostics implement.
use std::fmt;

/// A packed list this crate refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NvError {
    /// The buffer ended before the field at `at` could be read.
    Truncated {
        /// Offset of the field that could not be read.
        at: usize,
        /// Bytes the field needs.
        need: usize,
        /// Bytes that remain.
        have: usize,
    },
    /// The stream header names an encoding other than native.
    UnsupportedEncoding {
        /// The encoding byte that was read.
        encoding: u8,
    },
    /// The native stream header names a byte order other than little-endian.
    UnsupportedEndian {
        /// The byte-order byte that was read.
        byte_order: u8,
    },
    /// The nvlist version is not the only one this crate knows.
    UnsupportedVersion {
        /// The version the list declared.
        version: i32,
    },
    /// A pair's `nvp_size` is negative,
    /// shorter than its header,
    /// or runs past
    /// the buffer.
    BadPairSize {
        /// Offset of the pair's `nvp_size`.
        at: usize,
        /// The value that was read.
        size: i64,
    },
    /// A pair's name length is zero,
    /// runs past its blob,
    /// or is not NUL
    /// terminated.
    BadName {
        /// The name length field that was read.
        name_length: i16,
    },
    /// A pair's element count is negative.
    BadArrayCount {
        /// The element count that was read.
        elements: i64,
    },
    /// Sizing arithmetic for an array or nested list overflowed.
    Overflow {
        /// The element count that overflowed the size arithmetic.
        elements: usize,
    },
    /// A string value has no terminating NUL inside its pair.
    BadString {
        /// Name of the pair whose string value is unterminated.
        pair: String,
    },
    /// A value's bytes run past the pair's own size.
    ValueOverflow {
        /// Name of the pair.
        name: String,
        /// Bytes the value needs.
        need: usize,
        /// Bytes the pair holds.
        have: usize,
    },
    /// Nested lists go deeper than this crate follows.
    TooDeep,
    /// A name,
    /// string or value does not fit the wire format's field widths.
    TooBigToPack {
        /// Name of the pair that cannot be encoded.
        pair: String,
    },
    /// A name or string value carries an interior NUL,
    /// which would end it early.
    EmbeddedNul {
        /// Name of the pair that carries the NUL.
        name: String,
    },
}

/// How the value prints to a person.
impl fmt::Display for NvError {
    /// Explains the bytes the codec refused.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            NvError::Truncated { at, need, have } => write!(
                formatter,
                "buffer truncated: need {need} bytes at offset {at}, have {have}"
            ),
            NvError::UnsupportedEncoding { encoding } => {
                write!(
                    formatter,
                    "unsupported nvlist encoding {encoding} (only native is read)"
                )
            }
            NvError::UnsupportedEndian { byte_order: endian } => {
                write!(formatter, "unsupported native nvlist byte order {endian}")
            }
            NvError::UnsupportedVersion { version } => {
                write!(
                    formatter,
                    "unsupported nvlist version {version} (only version 0 is read)"
                )
            }
            NvError::BadPairSize { at, size } => {
                write!(formatter, "invalid nvpair size {size} at offset {at}")
            }
            NvError::BadName {
                name_length: name_size,
            } => {
                write!(formatter, "invalid nvpair name (name length {name_size})")
            }
            NvError::BadArrayCount { elements: count } => {
                write!(formatter, "nvpair array element count {count} is negative")
            }
            NvError::Overflow { elements: count } => {
                write!(
                    formatter,
                    "nvpair size arithmetic overflowed (element count {count})"
                )
            }
            NvError::BadString { pair: name } => {
                write!(
                    formatter,
                    "string value not NUL-terminated in pair '{name}'"
                )
            }
            NvError::ValueOverflow { name, need, have } => write!(
                formatter,
                "value for '{name}' ({need} bytes) exceeds nvpair bounds ({have} bytes)"
            ),
            NvError::TooDeep => {
                write!(
                    formatter,
                    "nvlist nesting is deeper than this crate follows"
                )
            }
            NvError::TooBigToPack { pair: name } => write!(
                formatter,
                "cannot encode '{name}': name/value size exceeds the wire format's field width"
            ),
            NvError::EmbeddedNul { name } => {
                write!(
                    formatter,
                    "cannot encode '{name}': embedded NUL in name or string value"
                )
            }
        };
    }
}

/// The standard-error view of the refusal.
impl std::error::Error for NvError {}
