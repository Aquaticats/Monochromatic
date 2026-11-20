//! What:
//!  What a held descriptor says about the mount it is on and the path the kernel finds at it now:
//!  the mount ID from the descriptor's `/proc/self/fdinfo` line
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
