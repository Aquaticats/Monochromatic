// Adapted from `CmdBuf` in `src/ioctl.rs` of `Ukko-Ylijumala/zfsdev-rs` at
// commit 4559ffc711; see PROVENANCE.md. Copyright (c) 2026 Mikko Tanner,
// MIT OR Apache-2.0.
//! What:
//!  A zeroed,
//! 8-byte aligned buffer that holds one `zfs_cmd_t` and spare room
//!  past it,
//! and the fixed-width address the ioctl call is given.
//! Why:
//!  The kernel copies its own `sizeof (zfs_cmd_t)` in and out of the address it
//!  is handed,
//! so a buffer shorter than that would be overrun,
//! and a buffer
//!  that is not 8-byte aligned would be read at the wrong offsets.
//!  The spare room is for a newer module whose struct grew:
//! it reads zeros for
//!  its new fields (every field's "unset") and writes them into the padding
//!  instead of past the allocation.
//! A request never relies on that room being
//!  a particular size,
//! so a grown struct is reported by the start-time check,
//!  not guessed at here.
//!  The address is a `u64` because that is the width the ABI stores;
//! a
//!  pointer-sized integer would differ on a future 128-bit target.

// The size the mirrored struct must have.
use super::abi_limits::COMMAND_STRUCT_SIZE;
// The mirrored struct the buffer is laid over.
use super::zfs_cmd_t::ZfsCmd;
// The formatting traits the diagnostics implement.
use std::fmt;
// The deref traits that expose the struct over the words.
use std::ops::{Deref, DerefMut};

/// Bytes a command buffer holds when a caller asks for no size:
/// the mirrored struct rounded up to the next power of two,
/// one 16 KiB slab with room past the struct for a module whose
/// `zfs_cmd_t` grew a little.
/// It is also the largest size this crate accepts.
pub const DEFAULT_COMMAND_BUFFER_SIZE: usize = COMMAND_STRUCT_SIZE.next_power_of_two();

/// Most bytes a command buffer may hold:
/// the slab [`DEFAULT_COMMAND_BUFFER_SIZE`] names,
/// because the kernel copies only its own `zfs_cmd_t` in and out and the spare
/// room only hedges a struct that grew a little.
/// A struct that outgrew the slab is what the start-time BTF check refuses:
/// it does not run against a module whose struct is bigger
/// (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
/// "ABI gate and tests",
/// the check being step 6 of
/// `doc/planning/cli-git-zfs-worktree-implementation-plan.md`),
/// and a larger buffer is not meant to absorb that growth.
pub const MAXIMUM_COMMAND_BUFFER_SIZE: usize = DEFAULT_COMMAND_BUFFER_SIZE;

/// What:
///  The pinning of the two buffer sizes:
/// the default keeps room past the struct,
/// and the maximum is that same slab.
/// Why:
///  A maximum given its own size would let a later growth narrow the default
/// to the struct alone,
/// costing the room a newer module's struct lands in
/// while every runtime test still passes.
const _: () = {
    assert!(DEFAULT_COMMAND_BUFFER_SIZE > COMMAND_STRUCT_SIZE);
    assert!(DEFAULT_COMMAND_BUFFER_SIZE == MAXIMUM_COMMAND_BUFFER_SIZE);
    // The slab is the 16 KiB the doc names, so a struct that grew past it stops
    // the build here instead of turning the default into 32 KiB unannounced.
    assert!(DEFAULT_COMMAND_BUFFER_SIZE == 16 * 1024);
};

/// A command buffer size this crate refuses to allocate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandBufferSize {
    /// The size the caller asked for.
    pub requested: usize,
    /// The largest size this crate accepts.
    pub maximum: usize,
}

/// How the value prints to a person.
impl fmt::Display for CommandBufferSize {
    /// Shows the buffer size rather than its words.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            formatter,
            "a command buffer of {} bytes exceeds the {} byte maximum",
            self.requested, self.maximum
        );
    }
}

/// The standard-error view of the refusal.
impl std::error::Error for CommandBufferSize {}

/// What a command buffer is made of:
/// whole 8-byte words,
/// so the struct can be
/// laid over every byte of it.
pub struct CommandBuffer {
    /// The whole words the struct is laid over.
    words: Box<[u64]>,
}

/// The methods of `CommandBuffer`.
impl CommandBuffer {
    /// A buffer of [`DEFAULT_COMMAND_BUFFER_SIZE`] bytes.
    pub fn new() -> CommandBuffer {
        let words: usize = DEFAULT_COMMAND_BUFFER_SIZE / size_of::<u64>();
        return CommandBuffer {
            words: vec![0; words].into_boxed_slice(),
        };
    }

    /// A buffer of `bytes` bytes,
    /// rounded up to whole words and never shorter
    /// than the mirrored struct.
    ///
    /// # Throws
    ///
    /// [`CommandBufferSize`] when `bytes` exceeds
    /// [`MAXIMUM_COMMAND_BUFFER_SIZE`].
    pub fn with_size(bytes: usize) -> Result<CommandBuffer, CommandBufferSize> {
        if bytes > MAXIMUM_COMMAND_BUFFER_SIZE {
            return Err(CommandBufferSize {
                requested: bytes,
                maximum: MAXIMUM_COMMAND_BUFFER_SIZE,
            });
        }
        let whole_bytes: usize = bytes.max(COMMAND_STRUCT_SIZE);
        let words: usize = whole_bytes.div_ceil(size_of::<u64>());
        return Ok(CommandBuffer {
            words: vec![0; words].into_boxed_slice(),
        });
    }

    /// Bytes the buffer holds,
    /// spare room after the struct included.
    pub fn len(&self) -> usize {
        return self.words.len() * size_of::<u64>();
    }

    /// Whether the buffer holds no bytes at all,
    /// which no constructed buffer
    /// does.
    pub fn is_empty(&self) -> bool {
        return self.words.is_empty();
    }

    /// The fixed-width address of the whole buffer,
    /// spare room included,
    /// for
    /// the ioctl call's argument.
    pub fn as_mut_abi_address(&mut self) -> u64 {
        return self.words.as_mut_ptr() as u64;
    }

    /// The mirrored struct laid over the start of the buffer.
    pub fn command(&self) -> &ZfsCmd {
        /*
        SAFETY: the buffer holds at least `size_of::<ZfsCmd>()` bytes (both
        constructors enforce it) at 8-byte alignment (whole `u64` words), and
        every field is an integer or a byte array, so any bit pattern is a
        valid value.
        */
        return unsafe { &*self.words.as_ptr().cast::<ZfsCmd>() };
    }

    /// The mirrored struct laid over the start of the buffer,
    /// for writing.
    pub fn command_mut(&mut self) -> &mut ZfsCmd {
        // SAFETY: as in `command`.
        return unsafe { &mut *self.words.as_mut_ptr().cast::<ZfsCmd>() };
    }
}

/// The value a caller gets without choosing one.
impl Default for CommandBuffer {
    /// The default buffer a caller gets without choosing a size.
    fn default() -> CommandBuffer {
        return CommandBuffer::new();
    }
}

/// How the value prints in a debug dump.
impl fmt::Debug for CommandBuffer {
    /// Shows the buffer size rather than its words.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return formatter
            .debug_struct("CommandBuffer")
            .field("bytes", &self.len())
            .finish();
    }
}

/// The struct view the buffer hands out.
impl Deref for CommandBuffer {
    /// The type the deref implementation exposes.
    type Target = ZfsCmd;

    /// The struct view over the buffer first bytes.
    fn deref(&self) -> &ZfsCmd {
        return self.command();
    }
}

/// The writable struct view the buffer hands out.
impl DerefMut for CommandBuffer {
    /// The writable struct view over the buffer first bytes.
    fn deref_mut(&mut self) -> &mut ZfsCmd {
        return self.command_mut();
    }
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "command_buffer_tests.rs"]
mod tests;
