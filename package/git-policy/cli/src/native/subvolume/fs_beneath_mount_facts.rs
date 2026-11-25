//! What:
//!  What a held descriptor says about the mount it is on and the path the kernel finds at it now:
//!  the mount ID from the descriptor's `/proc/self/fdinfo` line,
//!  the unique mount number from `statx(2)`
//!  and the path its `/proc/self/fd` link names.
//! Why:
//!  An attach that landed beside its slot is seen by comparing the mount the attach recorded with
//!  the mount a fresh lookup reaches,
//!  and by comparing the path a descriptor names with the slot's
//!  canonical path
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  Both answers come from the kernel and from the object the descriptor holds,
//!  never from a path string this process resolves again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const now = observedPath(handle); // readlink('/proc/self/fd/<fd>')
//! const mount = mountIdOf(handle);  // the `mnt_id:` line of /proc/self/fdinfo/<fd>
//! ```

use super::fs_beneath::handle_path;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};

/// What:
///  The path the kernel now finds at the name a held handle was opened at.
/// Why:
///   A descriptor keeps the object it was opened on,
/// so the path it reports is what that object is called now:
/// a folder renamed after the open reports the new name,
/// which is how an attach that landed beside its slot is seen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const now = await readlink('/proc/self/fd/' + handle.fd);
/// ```
pub fn observed_path(handle: &std::fs::File) -> std::io::Result<PathBuf> {
    return std::fs::read_link(handle_path(handle));
}

/// What:
///  The unique mount number of the mount an open handle is on,
///  from `statx(2)`'s unique mount identity.
/// Why:
///   The number `/proc/self/mountinfo` prints is reused as soon as the mount
///  that held it goes away,
///  so a copy of the table keyed by it can name a mount that no longer exists;
///  `statx(2)` documents the unique number as "guaranteed to not be reused
///  while the system is running" and dates it from Linux 6.8,
///  which is what lets `statmount(2)` be asked about this one mount.
///  A kernel without it answers an error rather than the reused number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unique = statx(handle.fd, '', AT_EMPTY_PATH, STATX_MNT_ID_UNIQUE).stx_mnt_id;
/// ```
///
/// # Errors
/// The call refused,
/// or this kernel reports no unique mount identity for the descriptor.
pub fn unique_mount_number_of(handle: &std::fs::File) -> std::io::Result<u64> {
    let mut facts: libc::statx = unsafe { std::mem::zeroed() };
    let empty: [libc::c_char; 1] = [0];
    // SAFETY: the descriptor is the caller's and open, the path is the empty string the flag tells the call to ignore,
    // and the answer is written into `facts`, which this function owns.
    let answered: libc::c_int = unsafe { libc::statx(handle.as_raw_fd(), empty.as_ptr(), libc::AT_EMPTY_PATH, libc::STATX_MNT_ID_UNIQUE, &mut facts) };
    if answered != 0 {
        return Err(std::io::Error::last_os_error());
    }
    if facts.stx_mask & libc::STATX_MNT_ID_UNIQUE == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("the kernel reports no unique mount number for {}", handle_path(handle).display()),
        ));
    }
    return Ok(facts.stx_mnt_id);
}

/// What:
///  The mount ID the kernel has recorded for the mount an open handle is on,
///  read from `/proc/self/fdinfo`.
/// Why:
///   Two folders are on one mount exactly when the kernel gives them one number,
/// and a fresh lookup that answers another number than the attach recorded
///  is the attach landing somewhere else,
///  not a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mntId = Number(readFileSync(`/proc/self/fdinfo/${fd}`, 'utf8').match(/^mnt_id:\s+(\d+)$/m)?.[1]);
/// ```
///
/// # Errors
/// The read failure,
/// or a refusal naming the descriptor's path because the text holds no mount ID.
pub fn mount_id_of(handle: &std::fs::File) -> std::io::Result<u64> {
    let text: String = std::fs::read_to_string(Path::new("/proc/self/fdinfo").join(handle.as_raw_fd().to_string()))?;
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("mnt_id:") else { continue };
        return rest.trim().parse::<u64>().map_err(|_error| return std::io::Error::new(std::io::ErrorKind::InvalidData, format!("the mount ID of {} is not a number: {}", handle_path(handle).display(), rest.trim())));
    }
    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("the kernel reports no mount ID for {}", handle_path(handle).display())));
}
