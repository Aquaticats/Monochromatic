//! What:
//!  The bytes the kernel writes into and reads out of:
//! the OpenZFS 2.4.4
//!  `zfs_cmd_t`,
//! the fixed name-field sizes,
//! and the zeroed aligned buffer one
//!  request owns.
//! Why:
//!  The ioctl call is one system call with one address and one size,
//! so the
//!  whole ABI surface a request can get wrong is here:
//! a field at another
//!  offset,
//! a buffer too short for the kernel's copy,
//! or an address that is
//!  not fixed-width.
//!  Nothing in this folder knows what a request means;
//! it only places bytes.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `abi_limits.rs`:
//!   names the fixed field sizes and the struct size the module reported.
//! - `zfs_cmd_t.rs`:
//!   mirrors the struct field for field,
//!   with the union and the size and
//!   offset assertions that pin it.
//! - `command_buffer.rs`:
//!   a zeroed,
//!   8-byte aligned buffer holding the struct and spare room.

/// The fixed field sizes and the struct size the design pins.
mod abi_limits;
/// The zeroed,
/// aligned buffer one request owns.
mod command_buffer;
/// The OpenZFS 2.4.4 command struct and its pinning assertions.
mod zfs_cmd_t;

/// The field bounds and the struct size the design pins.
pub use abi_limits::{COMMAND_STRUCT_SIZE, MAX_VALUE_LEN, MAXNAMELEN, MAXPATHLEN};
/// The command buffer and the size it refuses.
pub use command_buffer::{
    CommandBuffer, CommandBufferSize, DEFAULT_COMMAND_BUFFER_SIZE, MAXIMUM_COMMAND_BUFFER_SIZE,
};
/// The mirrored struct and the records it holds.
pub use zfs_cmd_t::{
    DmuObjsetStats, DrrBegin, ZcDestroyControl, ZcInjectOverlay, ZfsCmd, ZfsShare, ZfsStat,
    ZinjectRecord,
};
