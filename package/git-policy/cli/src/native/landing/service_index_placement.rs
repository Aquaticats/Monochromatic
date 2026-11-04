//! What:
//!  How the service puts its marked file in place as `index.lock`:
//! the system calls it makes,
//! the atomic placement itself,
//! and the removal of a stale lock.
//! Why:
//!  The lock must never exist without its marker,
//! so the marker is written to a file that is new and then made the lock in one step (a rename that does not replace,
//! else a hard link).
//! The calls are one `Platform` value so a control can refuse or count them for one acquisition alone.
//! The module is split from `service_index_lock.rs`,
//! which re-exports the public items here under their old names;
//! the staging files are in `service_index_staging.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await placeMarked(platform, staged, lock) // renameNoReplace ?? link ?? refuse
//! ```

use crate::fs_mount_options::read_mount_facts;
use crate::process_identity::ProcessIdentityError;
use crate::service_index_marker::own_identity;
use crate::service_index_ownership::{Ownership, inspect_ownership};
use crate::service_index_watch::Watch;
use std::path::Path;

/// What:
///  The system calls the lock makes that a control must be able to refuse,
/// count or observe:
/// the rename that does not replace,
/// the hard link,
/// the removal of a stale lock,
/// the process's own birth identity,
/// the setting of a watch on the lock's directory,
/// the flush of the staged marker to storage,
/// the question whether a lock is still the holder's,
/// and the removal of a lock the holder owns.
/// Why:
///   A filesystem with neither atomic placement,
/// a stale lock that cannot be unlinked,
/// a process with no identity,
/// a process out of descriptors and a watch quota that is used up
/// are environments no fixture can be made to have without changing the whole test process,
/// and a count kept in a process-wide static is read by whichever test runs beside it;
/// each acquisition takes the calls it makes from one value,
/// so a control substitutes them for that acquisition alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Platform = { rename; link; unlink; identity; watch; sync; inspect };
/// ```
pub struct Platform<'a> {
    /// A rename that fails when the target exists (`renameat2` with `RENAME_NOREPLACE`).
    pub rename: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>,
    /// A hard link that fails when the target exists.
    pub link: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>,
    /// The removal of a lock whose holder is gone.
    pub unlink: &'a dyn Fn(&Path) -> std::io::Result<()>,
    /// The birth identity of this process,
    /// which a marker names.
    pub identity: &'a dyn Fn() -> Result<String, ProcessIdentityError>,
    /// A watch on the directory of the lock.
    pub watch: &'a dyn Fn(&Path) -> Option<Watch>,
    /// The flush of the staged marker to storage,
    /// before it becomes the lock.
    pub sync: &'a dyn Fn(&std::fs::File) -> std::io::Result<()>,
    /// Whether the lock at a path is still the file the holder placed.
    pub inspect: fn(&Path, &std::fs::File, &str) -> Ownership,
    /// The removal of the lock the holder owns,
    /// at its release or when it goes out of scope.
    pub release: fn(&Path) -> std::io::Result<()>,
}

/// The operating system's own calls.
pub const SYSTEM: Platform<'static> = Platform {
    rename: &rename_noreplace,
    link: &link_file,
    unlink: &unlink_file,
    identity: &own_identity,
    watch: &Watch::on,
    sync: &sync_file,
    inspect: inspect_ownership,
    release: unlink_file,
};

pub fn sync_file(file: &std::fs::File) -> std::io::Result<()> {
    return crate::service_durable::flush_file(file, "lock-marker");
}

fn link_file(from: &Path, to: &Path) -> std::io::Result<()> {
    return std::fs::hard_link(from, to);
}

fn unlink_file(path: &Path) -> std::io::Result<()> {
    return std::fs::remove_file(path);
}

/// Renames `from` to `to` unless `to` exists:
/// one step that is as exclusive as `create_new` and leaves no second name.
#[cfg(target_os = "linux")]
pub fn rename_noreplace(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let source = std::ffi::CString::new(from.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    let target = std::ffi::CString::new(to.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    // SAFETY: both strings are valid NUL-terminated paths for the call.
    let done: i32 = unsafe { libc::renameat2(libc::AT_FDCWD, source.as_ptr(), libc::AT_FDCWD, target.as_ptr(), libc::RENAME_NOREPLACE) };
    return if done == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
}

#[cfg(not(target_os = "linux"))]
pub fn rename_noreplace(_from: &Path, _to: &Path) -> std::io::Result<()> {
    return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
}

/// Whether a failed rename without replacing means the filesystem cannot do one (renameat2(2):
/// `EINVAL`),
/// not that the lock exists or storage failed.
pub fn rename_unsupported(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(unix)]
    {
        return matches!(error.raw_os_error(), Some(code) if code == libc::EINVAL || code == libc::ENOSYS || code == libc::EOPNOTSUPP);
    }
    #[cfg(not(unix))]
    {
        return false;
    }
}

/// Whether a failed hard link means the filesystem makes none (link(2):
/// `EPERM`),
/// not that the lock exists or storage failed.
pub fn links_unsupported(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(unix)]
    {
        return matches!(error.raw_os_error(), Some(code) if code == libc::EPERM || code == libc::EOPNOTSUPP);
    }
    #[cfg(not(unix))]
    {
        return false;
    }
}

/// What:
///  Makes the marked `staged` file the lock at `lock`;
/// `AlreadyExists` when any process holds it,
/// and `staged` then stays for the next attempt.
/// Why:
///   Git's lock is taken by exclusive creation,
/// and a restart tells the service's own stale lock from a foreign one only by its marker,
/// so the lock must never exist without it.
/// A rename that does not replace is exclusive and atomic and leaves no second name;
/// where the filesystem has none,
/// a hard link is exclusive too and its staging name is removed after.
///  Where it has neither,
/// nothing is created:
/// exclusive creation followed by a write would leave an empty lock for as long as a kill between the two lasts,
/// and an empty lock may be a Git mid-write,
/// so no restart could take it over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await renameNoReplace(staged, lock) ?? await link(staged, lock) ?? refuse()
/// ```
///
/// # Errors
///
/// `AlreadyExists` when the lock exists,
/// `Unsupported` (naming both refusals) when the filesystem can place neither way,
/// and the storage error otherwise.
pub fn place_marked(platform: &Platform, staged: &Path, lock: &Path) -> std::io::Result<()> {
    let refused_rename: std::io::Error = match (platform.rename)(staged, lock) {
        Ok(()) => return Ok(()),
        Err(error) if !rename_unsupported(&error) => return Err(error),
        Err(error) => error,
    };
    match (platform.link)(staged, lock) {
        Ok(()) => {
            let _removed = std::fs::remove_file(staged);
            return Ok(());
        }
        Err(error) if !links_unsupported(&error) => return Err(error),
        Err(refused_link) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("renaming without replacing failed ({refused_rename}) and hard links failed ({refused_link})"),
            ));
        }
    }
}

/// What:
///  Says in the service's log why a lock was not placed:
/// the lock,
/// the cause,
/// and for a filesystem that can place a lock in neither atomic way,
/// the filesystem and what to do.
/// Why:
///   The clients are answered with a code only;
/// the person who reads the log must find the filesystem and the remedy there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.error(`cli-git-service: could not place ${lock}: ${error}`)
/// ```
pub fn log_placement_failure(lock: &Path, error: &std::io::Error) {
    if let Some(text) = placement_failure_text(lock, error) {
        crate::service_error!("{text}");
    }
}

/// What:
///  The log line for a failure to place `lock`,
/// or nothing when the failure is that the lock exists.
/// Why:
///   A lock that is held is the ordinary case of a wait and is no failure to place;
/// every other error says what happened,
/// and one that means the filesystem can place a lock neither way names the filesystem and what to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// placementFailureText(lock, error) // string | undefined
/// ```
pub fn placement_failure_text(lock: &Path, error: &std::io::Error) -> Option<String> {
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        return None;
    }
    if error.kind() != std::io::ErrorKind::Unsupported {
        return Some(format!("cannot take the index lock {}: {error}", lock.display()));
    }
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let filesystem: String = match read_mount_facts(directory) {
        Ok(Some(facts)) => format!("the {} filesystem mounted at {}", facts.filesystem_type, facts.mount_point.display()),
        Ok(None) => format!("the filesystem of {}", directory.display()),
        Err(reason) => format!("the filesystem of {} ({reason})", directory.display()),
    };
    return Some(format!(
        "cannot take the index lock {}: {filesystem} can neither rename a file without replacing another nor make a hard link ({error}), \
         so the lock cannot be created with its marker in one step, and the service takes no lock that a kill could leave empty. \
         Keep this repository's Git directory on a filesystem that supports one of the two (ext4, xfs, btrfs, tmpfs and overlayfs do), or update the kernel.",
        lock.display()
    ));
}

/// Removes the stale `lock`,
/// whose holder is gone.
/// False,
/// after saying why in the log,
/// when it is still there;
/// a lock that is already gone counts as removed.
pub fn reclaim(platform: &Platform, lock: &Path) -> bool {
    return reclaim_if(platform, lock, &crate::service_index_marker::is_reclaimable);
}

pub use crate::service_index_reclaim::reclaim_if;
