// Adapted from `src/nvlist.rs` of `Ukko-Ylijumala/zfsdev-rs` at commit
// 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  The codec for packed name-value lists (`nvlists`),
//! the currency of the
//!  `/dev/zfs` ioctl interface:
//! it reads the native encoding the kernel sends
//!  back and writes the native encoding the kernel accepts on input.
//! Why:
//!  The request packers carry `snaps`,
//! `origin` and `props` as lists,
//! and every
//!  reply (property values,
//! per-name errors) arrives as one.
//! The format is a
//!  byte-level contract from OpenZFS's `nvpair.c`,
//! so this crate owns the
//!  reading and writing rather than linking the CDDL library.
//!  Nothing here trusts a length:
//! every read is bounded by the buffer,
//! every
//!  value by its pair,
//! and nesting by a fixed depth,
//! because a reply is
//!  kernel-written bytes and a fuzz target feeds arbitrary ones.
//!
//! One deliberate difference from OpenZFS:
//! a `data_type_t` this crate does not name is
//! kept in [`NvData::Unknown`] with the pair's element count and value bytes,
//! where the kernel refuses such a pair outright
//! (`i_get_value_size`'s `default: return (-1)`,
//! `module/nvpair/nvpair.c:1153-1156`).
//! Keeping it makes a decode and re-pack a fixed point and lets a newer
//! module's type be seen rather than hidden;
//! no request forwards a decoded list to
//! the kernel,
//! so nothing built here sends one back.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `error.rs`:
//!   why a packed list is refused,
//!   with the offset or pair that is wrong.
//! - `data_type.rs`:
//!   the wire constants,
//!   `data_type_t` numbers,
//!   alignment and version check.
//! - `value.rs`:
//!   the decoded list,
//!   pair and typed value,
//!   with lookups and builders.
//! - `stream_cursor.rs`:
//!   a checked position in a packed list.
//! - `decode_scalar_value.rs`:
//!   the values whose bytes all live inside their own pair.
//! - `decode_pairs.rs`:
//!   the recursive pair loop,
//!   including embedded lists.
//! - `unpack.rs`:
//!   the entry point that checks the stream header and reads the root list.
//! - `encode_scalar_value.rs`:
//!   the wire bytes and width of every non-list value.
//! - `encode_pair.rs`:
//!   the recursive pair writer,
//!   including an embedded list's child stream.
//! - `pack.rs`:
//!   the entry point that writes the stream header and the root list.

/// The wire constants,
/// type numbers,
/// alignment and version check.
mod data_type;
/// The recursive pair loop,
/// including embedded lists.
mod decode_pairs;
/// The values whose bytes stay inside their own pair.
mod decode_scalar_value;
/// The recursive pair writer,
/// including a child stream.
mod encode_pair;
/// The wire bytes and width of every non-list value.
mod encode_scalar_value;
/// Why a packed list or a request is refused.
mod error;
/// The stream header and the root list writer.
mod pack;
/// The checked position in a packed list.
mod stream_cursor;
/// The stream header check and the root list reader.
mod unpack;
/// The decoded list,
/// pair and typed value.
mod value;

/// The codec refusal type.
pub use error::NvError;
/// The decoded list,
/// pair and value types.
pub use value::{NvData, NvList, NvPair};
