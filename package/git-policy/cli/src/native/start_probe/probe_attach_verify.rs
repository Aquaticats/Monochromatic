//! What:
//!  Arm 5's read-back after the move:
//!  the placed mount's ID,
//!  filesystem,
//!  source and flags from the worker's own table,
//!  `fstatfs` on the mount,
//!  and a bounded directory read.
//! Why:
//!  The arm reports the flags it asked for and reads them back,
//!  and it reads no file content
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The arms" and "Limits").
//!  An `fstatfs` the kernel answers with the ZFS magic is the mount's own
//!  word,
//!  and the bounded read touches no file.
//!  Every descriptor the read opens on the mount is closed here:
//!  one the worker kept would make arm 8's unmount answer `EBUSY`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const evidence = verifyAfterMove(slotPath, mountId, source); // { mount, entries }
//! ```

use super::probe_errno_names::errno_of;
use super::probe_mount_calls::read_own_mount_table;
use super::probe_mount_verify::{
    ExpectedMount,
    VerifiedMount,
    VerifyFailure,
    verify_mount,
};
use std::path::Path;

/// The most directory entries the read-back looks at,
/// so a never-ending listing cannot hold the probe.
pub const ENTRY_BOUND: usize = 32;

/// What:
///  What the attach and the read-back proved.
/// Why:
///   The record names the mount the attach made,
///  and the source is the exact name the configuration produced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttachEvidence = { mountId: number; mount: VerifiedMount; entries: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachEvidence {
    /// The mount the descriptor created and the read-back found.
    pub mount: VerifiedMount,
    /// How many entries the bounded read saw,
    /// never more than the bound.
    pub entries: usize,
}

/// The `fstatfs` magic of a ZFS mount,
/// which the kernel's own header names `ZFS_SUPER_MAGIC`.
pub const ZFS_SUPER_MAGIC: i64 = 0x2fc1_2fc1;

/// What:
///  Why the read-back refused,
///  with the errno where a call answered one.
/// Why:
///  The record names the step and its errno,
///  and a comparison that refused has no errno to name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadBackFailure = { errno?: number; reason: VerifyFailure | 'magic' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadBackFailure {
    /// The errno a call answered,
    /// where one answered.
    pub errno: Option<i32>,
    /// The comparison that refused.
    pub reason: String,
}

impl ReadBackFailure {
    /// The errno's name,
    /// where this build names it.
    pub fn errno_name(&self) -> Option<&'static str> {
        return self.errno.and_then(errno_of);
    }
}

/// What:
///  Read the mount the attach made back from the kernel.
/// Why:
///   The table names the mount by ID and checks the source and flags,
///  `fstatfs` says the mount's own filesystem is ZFS,
///  and the bounded read says its root can be listed.
///
/// # Errors
/// The table could not be read,
///  the mount is not the one the attach asked for,
///  `fstatfs` answered another magic,
///  or the root could not be opened or listed.
pub fn verify_after_move(
    slot_path: &Path,
    mount_id: u64,
    source: &str,
) -> Result<AttachEvidence, ReadBackFailure> {
    let table = match read_own_mount_table() {
        Ok(table) => table,
        Err(error) => {
            return Err(ReadBackFailure {
                errno: Some(error.errno),
                reason: String::from(error.call),
            });
        }
    };
    let mount = match verify_mount(
        table.as_slice(),
        ExpectedMount {
            mount_id,
            source,
        },
    ) {
        Ok(mount) => mount,
        Err(failure) => {
            return Err(ReadBackFailure {
                errno: None,
                reason: refusal_words(&failure),
            });
        }
    };
    let opened: std::fs::File = match std::fs::File::open(slot_path) {
        Ok(file) => file,
        Err(error) => {
            return Err(ReadBackFailure {
                errno: error.raw_os_error(),
                reason: String::from("open"),
            });
        }
    };
    // SAFETY: an all-zero `statfs` is the empty structure `fstatfs` fills, and every field is a plain number.
    let mut magic: libc::statfs = unsafe { std::mem::zeroed() };
    // SAFETY: the descriptor is this call's own and the structure is sized for the call.
    let asked: libc::c_int =
        unsafe { libc::fstatfs(std::os::fd::AsRawFd::as_raw_fd(&opened), &mut magic) };
    if asked != 0 {
        return Err(ReadBackFailure {
            errno: std::io::Error::last_os_error().raw_os_error(),
            reason: String::from("fstatfs"),
        });
    }
    if magic.f_type != ZFS_SUPER_MAGIC {
        return Err(ReadBackFailure {
            errno: None,
            reason: format!(
                "fstatfs answered magic {} rather than the ZFS magic",
                magic.f_type
            ),
        });
    }
    let entries = match std::fs::read_dir(slot_path) {
        Ok(listing) => listing
            .take(ENTRY_BOUND)
            .filter(|entry| return entry.is_ok())
            .count(),
        Err(error) => {
            return Err(ReadBackFailure {
                errno: error.raw_os_error(),
                reason: String::from("read_dir"),
            });
        }
    };
    return Ok(AttachEvidence {
        mount,
        entries,
    });
}

/// The comparison's refusal in words.
fn refusal_words(failure: &VerifyFailure) -> String {
    return failure.message();
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_attach_verify_tests.rs"]
mod tests;
