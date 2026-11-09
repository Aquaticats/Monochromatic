//! What:
//!  The nine 32-bit stat fields an index entry and the untracked-cache extension store,
//! and the Git varint.
//! Why:
//!  Both file layouts write the same fields (the entry puts the mode between inode and owner),
//! and both use Git's
//!  offset varint for lengths;
//! one definition keeps the readers and writers of the in-process index update in step.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type StatData = { ctimeSec: number; ctimeNsec: number; mtimeSec: number; mtimeNsec: number; dev: number; ino: number; uid: number; gid: number; size: number };
//! ```

/// The stat fields of one file as an index stores them (each truncated to 32 bits);
/// all zero is the "never statted" value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StatData {
    /// Change time,
    /// seconds.
    pub ctime_sec: u32,
    /// Change time,
    /// nanoseconds.
    pub ctime_nsec: u32,
    /// Modification time,
    /// seconds.
    pub mtime_sec: u32,
    /// Modification time,
    /// nanoseconds.
    pub mtime_nsec: u32,
    /// Device.
    pub dev: u32,
    /// Inode.
    pub ino: u32,
    /// Owner.
    pub uid: u32,
    /// Group.
    pub gid: u32,
    /// Size.
    pub size: u32,
}

/// A big-endian `u32` at `at`.
pub fn be32(bytes: &[u8], at: usize) -> Option<u32> {
    let quad: &[u8] = bytes.get(at..at.checked_add(4)?)?;
    return Some(u32::from_be_bytes([quad[0], quad[1], quad[2], quad[3]]));
}

/// What:
///  The date of the index file being written,
/// to the nanosecond,
/// as Git compares an entry's times with it.
/// Why:
///  Git decides that an entry is racily clean by comparing its modification time with the index file's own
/// (`is_racy_stat` in `read-cache.c`),
/// and a file system that stores nanoseconds makes an entry written in the same second as the
///  index older or newer within that second.
/// A comparison in whole seconds refuses entries Git would trust,
/// and a
///  comparison without the rule trusts entries Git would hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexStamp = { seconds: number; nanoseconds: number }; // the index file's own modification time
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexStamp {
    /// The index file's modification time,
    /// seconds.
    pub seconds: u32,
    /// The index file's modification time,
    /// nanoseconds.
    pub nanoseconds: u32,
}

impl IndexStamp {
    /// A stamp no entry can be as new as,
    /// for a caller that has no index file time.
    pub const NONE: IndexStamp = IndexStamp { seconds: u32::MAX, nanoseconds: 0 };

    /// What:
    ///  Whether an entry whose stat data is `recorded` is racily clean against this stamp,
    /// as Git's `is_racy_stat` with
    ///  nanosecond times decides.
    /// Why:
    ///   A racily clean entry proves nothing by its stat data:
    /// Git hashes the file,
    /// and so does the refresh this wrapper asks Git for,
    /// so nothing may trust the
    ///  record as proof of the file's content.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const racy = (stamp, recorded) => stamp.seconds !== 0 && (stamp.seconds < recorded.mtimeSec
    ///   || (stamp.seconds === recorded.mtimeSec && stamp.nanoseconds <= recorded.mtimeNsec));
    /// ```
    pub fn is_racy(&self, recorded: &StatData) -> bool {
        if self.seconds == u32::MAX {
            return false;
        }
        return self.seconds < recorded.mtime_sec || (self.seconds == recorded.mtime_sec && self.nanoseconds <= recorded.mtime_nsec);
    }
}

impl StatData {
    /// The fields in the order the untracked cache stores them (36 bytes),
    /// as `stat_data_to_disk`.
    pub fn to_cache_bytes(&self, out: &mut Vec<u8>) {
        for value in [self.ctime_sec, self.ctime_nsec, self.mtime_sec, self.mtime_nsec, self.dev, self.ino, self.uid, self.gid, self.size] {
            out.extend_from_slice(&value.to_be_bytes());
        }
    }

    /// The fields from 36 bytes in the untracked cache's order (`stat_data_from_disk`).
    pub fn from_cache_bytes(bytes: &[u8]) -> Option<StatData> {
        return Some(StatData {
            ctime_sec: be32(bytes, 0)?,
            ctime_nsec: be32(bytes, 4)?,
            mtime_sec: be32(bytes, 8)?,
            mtime_nsec: be32(bytes, 12)?,
            dev: be32(bytes, 16)?,
            ino: be32(bytes, 20)?,
            uid: be32(bytes, 24)?,
            gid: be32(bytes, 28)?,
            size: be32(bytes, 32)?,
        });
    }
}

/// The bytes of one stat record in the untracked cache.
pub const CACHE_STAT_LENGTH: usize = 36;

/// Append `value` as Git's offset varint (`encode_varint` in `varint.c`).
pub fn encode_varint(value: u64, out: &mut Vec<u8>) {
    let mut reversed: Vec<u8> = vec![(value & 127) as u8];
    let mut rest: u64 = value >> 7;
    while rest > 0 {
        rest -= 1;
        reversed.push(128 | (rest & 127) as u8);
        rest >>= 7;
    }
    reversed.reverse();
    out.extend_from_slice(reversed.as_slice());
}

/// The varint at the start of `bytes` and the bytes after it (`decode_varint`),
/// or `None` when it is cut off or overflows.
pub fn decode_varint(bytes: &[u8]) -> Option<(u64, &[u8])> {
    let mut used: usize = 1;
    let mut byte: u8 = *bytes.first()?;
    let mut value: u64 = u64::from(byte & 127);
    while byte & 128 != 0 {
        byte = *bytes.get(used)?;
        used += 1;
        value = value.checked_add(1)?;
        if value > (u64::MAX >> 7) {
            return None;
        }
        value = (value << 7) | u64::from(byte & 127);
    }
    return Some((value, &bytes[used..]));
}

#[cfg(test)]
#[path = "index_stat_tests.rs"]
mod tests;
