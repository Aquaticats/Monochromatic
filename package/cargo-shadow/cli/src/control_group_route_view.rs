//! What:
//! The mount view of `/sys/fs/cgroup` that route discovery reads:
//! the device,
//! the inode and the mount number of the directory,
//! and the text a refusal or a record writes about it.
//! Why:
//! A route names a directory,
//! and two runs must be able to tell whether they read the same one:
//! a mount that was made again keeps neither the device nor the mount number.
//! Split out of `control_group_route_discovery.rs`;
//! `View` and `view_of` are re-exported from there,
//! so their paths are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control_group_route_view.ts: interface View and viewOf(path)
//! ```

/// What:
/// `use rustix::fs::{AtFlags, CWD, StatxFlags, statx};` imports the safe wrapper of `statx`:
/// the flags that say which fields are asked for,
/// the current directory as the base of a relative path,
/// and the one bit that makes the mount number unique across a remount.
/// Why:
/// The view is a `statx` answer,
/// and the unique mount number is the field that tells a remounted tree from the one an earlier record saw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { statSync } from 'node:fs';   // which reports no mount number at all
/// ```
use rustix::fs::{
    AtFlags,
    CWD,
    StatxFlags,
    statx,
};
/// What:
/// `use std::fmt;` imports the standard library's formatting module,
/// which holds `Display` (a type that can be printed with `{}`) and `Formatter` (the printer it writes into).
/// Why:
/// A view is named in the message of a refusal and in the record of a journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // String(value) and template literals
/// ```
use std::fmt;
/// What:
/// `use std::io;` imports the standard library's input/output module so this file can name it unqualified.
/// Why:
/// A `statx` that fails is reported to the caller as an I/O failure,
/// not as an absent view.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an I/O failure is an ordinary thrown Error in TS
/// ```
use std::io;
/// What:
/// `use std::path::Path;` imports the borrowed path type from the standard library so this file can name it unqualified.
/// Why:
/// The view is read from a path that is only lent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::Path;

/// `STATX_MNT_ID_UNIQUE` of `linux/stat.h` (Linux 6.8):
/// the mount id that is never reused while the system runs.
/// `rustix` 1.1.6 has the field and not the flag,
/// so the bit is spelled here.
pub(crate) const STATX_MNT_ID_UNIQUE: u32 = 0x4000;

/// How a path is seen by this process:
/// the mount it is on and its inode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct View {
    /// The mount id (`STATX_MNT_ID`),
    /// which differs in another mount namespace.
    pub(crate) mount: u64,
    /// The inode number.
    pub(crate) inode: u64,
}

/// How a view reads in a journal line and in a message.
impl fmt::Display for View {
    /// What:
    /// `fn fmt(&self, formatter)` writes `mount:inode`.
    /// Why:
    /// Both numbers sit in one field of a record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        return write!(formatter, "{}:{}", self.mount, self.inode);
    }
}

/// What:
/// `pub(crate) fn view_of(path: &Path) -> io::Result<View>` asks the kernel for the mount id and the inode of a path.
/// Why:
/// A path means the same thing in a recovery as in the run only when both are equal (contract assumption c).
/// A kernel without `STATX_MNT_ID` (before Linux 5.8) is an error,
/// never a view of zero.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function viewOf(path: string): View
/// ```
pub(crate) fn view_of(path: &Path) -> io::Result<View> {
    // The unique id is asked for alone first (a kernel before 6.8 ignores the bit); the reusable one is the fallback.
    let unique_bit: StatxFlags = StatxFlags::from_bits_retain(STATX_MNT_ID_UNIQUE);
    let unique = statx(CWD, path, AtFlags::empty(), unique_bit | StatxFlags::INO)
        .map_err(io::Error::from)?;
    if StatxFlags::from_bits_retain(unique.stx_mask).contains(unique_bit)
        && StatxFlags::from_bits_retain(unique.stx_mask).contains(StatxFlags::INO)
    {
        return Ok(View {
            mount: unique.stx_mnt_id,
            inode: unique.stx_ino,
        });
    }
    let found = statx(
        CWD,
        path,
        AtFlags::empty(),
        StatxFlags::MNT_ID | StatxFlags::INO,
    )
    .map_err(io::Error::from)?;
    let reported: StatxFlags = StatxFlags::from_bits_retain(found.stx_mask);
    if !reported.contains(StatxFlags::MNT_ID) || !reported.contains(StatxFlags::INO) {
        return Err(io::Error::other(format!(
            "the kernel did not report the mount id and inode of {}",
            path.display()
        )));
    }
    return Ok(View {
        mount: found.stx_mnt_id,
        inode: found.stx_ino,
    });
}
