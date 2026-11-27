//! What:
//!  The identity of a filesystem object:
//! its device,
//! its inode and,
//! where the filesystem reports one,
//! its birth time,
//! read for an open file,
//! for a name below a held directory and for a whole open file or directory.
//! Why:
//!  A path or a name proves nothing about which object it holds a moment later;
//! a device and an inode do,
//! up to the reuse of an inode number within one birth-time tick,
//! so every removal of something this program made compares the identity it recorded with the one a name holds now.
//! The device of an open file is also how a walk tells the filesystem it started on from a nested subvolume.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const id = identityOf(file);                // { device, inode, birth }
//! const named = identityAt(directory, 'a');   // the entry's own, a link not followed
//! const device = deviceOf(file);              // the filesystem the descriptor is on
//! ```

use super::fs_beneath_names::{
    checked,
    name_of,
};
use std::ffi::{
    CString,
    OsStr,
};
use std::fs::File;
use std::os::fd::AsRawFd;
/// The identity of a filesystem object:
/// device,
/// inode and,
/// where the filesystem gives one,
/// the birth time.
///
/// Why:
/// a path or a name proves nothing about which object it names later;
/// this does,
/// up to the reuse of an inode number
/// within one birth-time tick.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectId {
    /// The device number.
    pub device: u64,
    /// The inode number.
    pub inode: u64,
    /// The birth time as seconds and nanoseconds,
    /// when the filesystem reports it.
    pub birth: Option<(i64, u32)>,
}

impl ObjectId {
    /// The identity as `device:inode:seconds:nanoseconds` (`-` for an unknown birth time).
    pub fn encode(&self) -> String {
        return match self.birth {
            Some((seconds, nanos)) => format!("{}:{}:{seconds}:{nanos}", self.device, self.inode),
            None => format!("{}:{}:-:-", self.device, self.inode),
        };
    }

    /// The identity from its encoding.
    pub fn decode(text: &str) -> Option<ObjectId> {
        let mut parts = text.split(':');
        let device: u64 = parts.next()?.parse().ok()?;
        let inode: u64 = parts.next()?.parse().ok()?;
        let seconds: &str = parts.next()?;
        let nanos: &str = parts.next()?;
        let birth: Option<(i64, u32)> = if seconds == "-" {
            None
        } else {
            Some((seconds.parse().ok()?, nanos.parse().ok()?))
        };
        return if parts.next().is_none() {
            Some(ObjectId {
                device,
                inode,
                birth,
            })
        } else {
            None
        };
    }
}

/// Build an identity from the fields of `statx`.
fn id_of_statx(found: &libc::statx) -> ObjectId {
    let birth: Option<(i64, u32)> = if found.stx_mask & libc::STATX_BTIME != 0 {
        Some((found.stx_btime.tv_sec, found.stx_btime.tv_nsec))
    } else {
        None
    };
    return ObjectId {
        device: libc::makedev(found.stx_dev_major, found.stx_dev_minor),
        inode: found.stx_ino,
        birth,
    };
}

/// The identity of the entry `name` in `dir`,
/// without following a link (`name` empty means `dir` itself).
fn identity_statx(
    dir: &File,
    name: Option<&OsStr>,
) -> std::io::Result<ObjectId> {
    let (c_name, flags): (CString, libc::c_int) = match name {
        Some(plain) => (name_of(plain)?, libc::AT_SYMLINK_NOFOLLOW),
        None => (CString::default(), libc::AT_EMPTY_PATH),
    };
    // SAFETY: an all-zero `statx` is a valid value for the call to fill.
    let mut found: libc::statx = unsafe { std::mem::zeroed() };
    // SAFETY: `c_name` outlives the call, `found` is a live `statx`, and `dir` is an open descriptor.
    checked(unsafe {
        libc::statx(
            dir.as_raw_fd(),
            c_name.as_ptr(),
            flags,
            libc::STATX_BASIC_STATS | libc::STATX_BTIME,
            &mut found,
        )
    })?;
    return Ok(id_of_statx(&found));
}

/// The identity of an open file or directory.
pub fn identity_of(file: &File) -> std::io::Result<ObjectId> {
    return identity_statx(file, None);
}

/// The identity of the entry `name` in `dir`,
/// a link not followed.
pub fn identity_at(
    dir: &File,
    name: &OsStr,
) -> std::io::Result<ObjectId> {
    return identity_statx(dir, Some(name));
}

/// The device number of an open file.
pub fn device_of(file: &File) -> std::io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    return Ok(file.metadata()?.dev());
}
