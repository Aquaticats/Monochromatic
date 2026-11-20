//! What:
//!  The reply buffer a request hands the kernel,
//! the rule for growing it when
//!  the kernel asks for more room,
//! and the bounded decode of what came back.
//! Why:
//!  Two failures are possible on the way back,
//! and they are not the same.
//!  A read may be retried with a bigger buffer:
//! the kernel writes the size it
//!  needs into `zc_nvlist_dst_size`,
//! and growing to that size costs one more
//!  read of unchanged state.
//! A write may not:
//! by the time the kernel asks,
//! it
//!  has already changed state,
//! so a retry would run a snapshot or a destroy a
//!  second time.
//!  A reported size is untrusted:
//! a value past the cap is refused rather than
//!  allocated,
//! and a size longer than the buffer is refused rather than read.

// The codec list and refusal types.
use crate::nvlist::{NvError, NvList};
// The formatting traits the diagnostics implement.
use std::fmt;

/// Initial reply capacity:
/// 256 KiB,
/// the buffer `libzfs` allocates when it is given no length for a
/// `zc_nvlist_dst` (`zcmd_alloc_dst_nvlist`,
/// `lib/libzfs/libzfs_util.c:1200-1204`).
/// `libzfs_core` starts at `MAX(size * 2, 128 * 1024)` instead
/// (`lib/libzfs_core/libzfs_core.c:250`);
/// this crate takes the larger floor,
/// and `initial_size` keeps `libzfs_core`'s doubling rule for a write.
pub const INITIAL_REPLY_SIZE: usize = 256 * 1024;

/// `ZCP_MAX_MEMLIMIT`:
/// the module's largest channel-program memory limit,
/// `10 * ZCP_DEFAULT_MEMLIMIT` with the default 10 MiB
/// (`include/sys/fs/zfs.h:1874-1875`).
const LARGEST_CHANNEL_PROGRAM_REPLY: usize = 100 * 1024 * 1024;

/// Most bytes a reply buffer may hold:
/// the next power of two at or above the module's largest channel-program
/// memory limit
/// (100 MiB,
/// so 128 MiB),
/// the largest reply this crate asks for.
/// `libzfs_core` sizes a channel-program reply buffer to that memlimit
/// (`lib/libzfs_core/libzfs_core.c:246-248`).
/// The module refuses a memlimit past `zfs_lua_max_memlimit`
/// (`module/zfs/zcp.c:1048`),
/// whose default is `ZCP_MAX_MEMLIMIT`
/// (`module/zfs/zcp.c:114`),
/// so 100 MiB is the stock maximum:
/// an administrator may raise that parameter
/// (`module/zfs/zcp.c:1453` and `ZMOD_RW`),
/// and a reply past this cap is refused rather than allocated.
pub const MAXIMUM_REPLY_SIZE: usize = LARGEST_CHANNEL_PROGRAM_REPLY.next_power_of_two();

/// What:
///  The pinning of the reply cap:
/// it stays above the module's largest channel-program reply,
/// and it is 128 MiB,
/// the next power of two at or above that.
/// Why:
///  A changed derivation would otherwise ship a cap that no longer covers the
///  reply it exists for.
const _: () = {
    assert!(LARGEST_CHANNEL_PROGRAM_REPLY < MAXIMUM_REPLY_SIZE);
    assert!(MAXIMUM_REPLY_SIZE == 128 * 1024 * 1024);
};

/// What the reply of one request may be used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyUse {
    /// A read:
    /// its reply may be requested again with a bigger buffer.
    Read,
    /// A write:
    /// its reply is never requested again,
    /// because the kernel may
    /// already have changed state.
    Mutation,
}

/// A reply the kernel made larger than the buffer allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplyRefusal {
    /// Bytes the kernel said it needs.
    pub requested: u64,
    /// Bytes the buffer holds now.
    pub held: usize,
    /// Most bytes this crate allocates.
    pub limit: usize,
    /// Whether the refusal is the cap or the no-retry rule.
    pub reason: ReplyRefusalReason,
}

/// Which rule refused a bigger reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyRefusalReason {
    /// The requested size is past [`MAXIMUM_REPLY_SIZE`].
    PastTheCap,
    /// A write asked for a reply it would have to replay to get.
    NeverReplayAWrite,
}

/// How the value prints to a person.
impl fmt::Display for ReplyRefusal {
    /// Says why the reply could not be used.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self.reason {
            ReplyRefusalReason::PastTheCap => write!(
                formatter,
                "the kernel asked for a {} byte reply, and this crate allocates at most {} bytes",
                self.requested, self.limit
            ),
            ReplyRefusalReason::NeverReplayAWrite => write!(
                formatter,
                "the kernel asked for a {} byte reply buffer for a request that changes state, so it is not replayed",
                self.requested
            ),
        };
    }
}

/// The standard-error view of the refusal.
impl std::error::Error for ReplyRefusal {}

/// A reply buffer the request owns for the length of the kernel's write.
pub struct ReplyBuffer {
    /// What the reply may be used for.
    use_of_reply: ReplyUse,
    /// The bytes the kernel writes into.
    bytes: Vec<u8>,
}

/// The methods of `ReplyBuffer`.
impl ReplyBuffer {
    /// A zeroed buffer of the size this use starts from.
    pub fn new(use_of_reply: ReplyUse, input_length: usize) -> ReplyBuffer {
        let size: usize = initial_size(use_of_reply, input_length);
        return ReplyBuffer {
            use_of_reply,
            bytes: vec![0; size],
        };
    }

    /// What this buffer is for.
    pub fn use_of_reply(&self) -> ReplyUse {
        return self.use_of_reply;
    }

    /// Bytes the buffer holds.
    pub fn len(&self) -> usize {
        return self.bytes.len();
    }

    /// Whether the buffer holds no bytes,
    /// which no constructed buffer does.
    pub fn is_empty(&self) -> bool {
        return self.bytes.is_empty();
    }

    /// The buffer's bytes,
    /// for a live call's fault tests and for the fakes a
    /// device-free test writes a reply into.
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        return &mut self.bytes;
    }

    /// The fixed-width address the kernel is given.
    pub fn as_mut_abi_address(&mut self) -> u64 {
        return self.bytes.as_mut_ptr() as u64;
    }

    /// The size the kernel is told it may write,
    /// as the ABI stores it.
    pub fn as_abi_size(&self) -> u64 {
        return self.bytes.len() as u64;
    }

    /// Grow to the size the kernel said it needs,
    /// or refuse.
    ///
    /// A read grows to the next power of two at or above the request,
    /// capped at
    /// [`MAXIMUM_REPLY_SIZE`],
    /// so a kernel asking for one more byte does not
    /// cost an exact-size allocation on every call.
    /// A write never grows,
    /// and a
    /// size past the cap is refused for both.
    ///
    /// # Throws
    ///
    /// [`ReplyRefusal`] when the request is past the cap,
    /// or when a write's
    /// reply would have to be read again.
    pub fn grow_to_requested(&mut self, requested: u64) -> Result<usize, ReplyRefusal> {
        if usize::try_from(requested).is_ok_and(|size| {
            return size <= self.bytes.len();
        }) {
            return Ok(self.bytes.len());
        }
        if requested > MAXIMUM_REPLY_SIZE as u64 {
            return Err(ReplyRefusal {
                requested,
                held: self.bytes.len(),
                limit: MAXIMUM_REPLY_SIZE,
                reason: ReplyRefusalReason::PastTheCap,
            });
        }
        if self.use_of_reply == ReplyUse::Mutation {
            return Err(ReplyRefusal {
                requested,
                held: self.bytes.len(),
                limit: MAXIMUM_REPLY_SIZE,
                reason: ReplyRefusalReason::NeverReplayAWrite,
            });
        }
        let needed: usize = requested as usize;
        let grown: usize = needed
            .checked_next_power_of_two()
            .unwrap_or(MAXIMUM_REPLY_SIZE)
            .min(MAXIMUM_REPLY_SIZE);
        self.bytes.resize(grown, 0);
        return Ok(self.bytes.len());
    }

    /// Decode the reply the kernel says it wrote.
    ///
    /// `filled` is `zc_nvlist_dst_filled`,
    /// and `reported` is the size the
    /// kernel left in `zc_nvlist_dst_size`.
    /// No list is `None`;
    /// a list longer
    /// than the buffer,
    /// or one that does not decode,
    /// is a [`ReplyError`].
    ///
    /// # Throws
    ///
    /// [`ReplyError::LargerThanBuffer`] when the reported size exceeds the
    /// buffer,
    /// and [`ReplyError::Decode`] when the bytes are not a native list.
    pub fn decode(&self, filled: bool, reported: u64) -> Result<Option<NvList>, ReplyError> {
        if !filled {
            return Ok(None);
        }
        let Ok(length) = usize::try_from(reported) else {
            return Err(ReplyError::LargerThanBuffer {
                reported,
                capacity: self.bytes.len(),
            });
        };
        if length > self.bytes.len() {
            return Err(ReplyError::LargerThanBuffer {
                reported,
                capacity: self.bytes.len(),
            });
        }
        return NvList::unpack(&self.bytes[..length])
            .map(Some)
            .map_err(|error| {
                return ReplyError::Decode { value: error };
            });
    }
}

/// Why a reply could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplyError {
    /// The kernel reported more bytes than the buffer holds.
    LargerThanBuffer {
        /// Bytes the kernel reported.
        reported: u64,
        /// Bytes the buffer holds.
        capacity: usize,
    },
    /// The bytes are not a native name-value list.
    Decode {
        /// The codec's refusal.
        value: NvError,
    },
}

/// How the value prints to a person.
impl fmt::Display for ReplyError {
    /// Says why the reply could not be used.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            ReplyError::LargerThanBuffer { reported, capacity } => write!(
                formatter,
                "the kernel reported a {reported} byte reply, and the buffer holds {capacity} bytes"
            ),
            ReplyError::Decode { value } => write!(formatter, "decoding the reply: {value}"),
        };
    }
}

/// The standard-error view of the refusal.
impl std::error::Error for ReplyError {
    /// The codec refusal,
    /// when the reply was bytes.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        return match self {
            ReplyError::Decode { value } => Some(value),
            ReplyError::LargerThanBuffer { .. } => None,
        };
    }
}

/// The size a reply starts from:
/// [`INITIAL_REPLY_SIZE`] for a read,
/// and for a write the larger of that and twice the input,
/// which is `libzfs_core`'s rule with this crate's floor
/// (`MAX(size * 2, 128 * 1024)`,
/// `lib/libzfs_core/libzfs_core.c:250`).
pub fn initial_size(use_of_reply: ReplyUse, input_length: usize) -> usize {
    return match use_of_reply {
        ReplyUse::Read => INITIAL_REPLY_SIZE,
        ReplyUse::Mutation => INITIAL_REPLY_SIZE
            .max(input_length.saturating_mul(2))
            .min(MAXIMUM_REPLY_SIZE),
    };
}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "reply_tests.rs"]
mod tests;
