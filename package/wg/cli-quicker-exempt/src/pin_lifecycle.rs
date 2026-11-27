//! One lifecycle command:
//! the lock that keeps two of them apart,
//! and the attach or detach a command runs.
//! The lock is held across the whole mutation,
//! so an attach and a detach of one cgroup never interleave.

/// Raw BPF attachment function.
use crate::bpf::attach_marker;
/// Debug-only typed pin failure constructor for deterministic fallback tests.
#[cfg(debug_assertions)]
use crate::bpf_error::pin_object_invalid;
/// Standard filesystem and syscall error channel.
use std::io;
/// Raw descriptor access for the cgroup file the hooks attach to.
use std::os::fd::AsRawFd;
/// Unix filesystem device number proves bpffs mount boundaries.
use std::os::unix::fs::MetadataExt;
/// Owned filesystem paths used for canonical cgroup and pin locations.
use std::path::Path;

/// The bpffs path a cgroup's pins live at.
use super::key::pin_dir_for;
/// The transaction that replaces one pin set with another.
use super::replacement::{
    cleanup_stale_staging,
    commit_staging,
    remove_exact_pin_dir,
    rollback_staging_error,
    staging_dir_for,
};
/// The bpffs check,
/// and the staging root this tool creates there.
use super::root::{
    STAGING_ROOT,
    ensure_pin_root,
};

/// Debug-only environment flag simulating affected kernel's typed object-pin failure.
#[cfg(debug_assertions)]
const FORCE_PIN_INVALID_ENV: &str = "WG_QUICKER_EXEMPT_TEST_FORCE_PIN_INVALID";

/// Keeps the global lifecycle lock file open until attach or detach command ends.
pub struct LifecycleLock {
    /// Open file owns the descriptor carrying kernel advisory lock.
    _file: std::fs::File,
}

/// Acquires process-crash-safe global lock for pin directory mutations.
pub fn acquire_lifecycle_lock() -> io::Result<LifecycleLock> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open("/run/wg-quicker-exempt.lock")?;
    // SAFETY: `file` owns a valid descriptor and `flock` changes only its kernel lock state.
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(LifecycleLock {
        _file: file,
    });
}

/// Attaches all hooks in staging and commits them without dropping prior working pins.
pub fn attach_cgroup(
    mark: u32,
    cgroup_dir: &Path,
) -> io::Result<usize> {
    ensure_pin_root()?;
    let final_dir = pin_dir_for(cgroup_dir)?;
    let Some(parent) = final_dir.parent() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "pin path lacks parent",
        ));
    };
    std::fs::create_dir_all(parent)?;
    cleanup_stale_staging()?;
    let staging_dir = staging_dir_for();
    let staging_device = std::fs::metadata(STAGING_ROOT)?.dev();
    let final_device = std::fs::metadata(parent)?.dev();
    if staging_device != final_device {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "staging and final pin directories are on different filesystems",
        ));
    }
    if staging_dir.exists() {
        remove_exact_pin_dir(&staging_dir)?;
    }
    std::fs::create_dir(&staging_dir)?;
    let cgroup = std::fs::File::open(cgroup_dir)?;
    let Some(staging_text) = staging_dir.to_str() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "staging path is not UTF-8",
        ));
    };
    #[cfg(debug_assertions)]
    if std::env::var_os(FORCE_PIN_INVALID_ENV).is_some() {
        let source = io::Error::from_raw_os_error(libc::EINVAL);
        let error = pin_object_invalid(staging_text.to_string(), source);
        return Err(rollback_staging_error(&staging_dir, error));
    }
    let pinned = match attach_marker(cgroup.as_raw_fd(), mark, staging_text) {
        Ok(paths) => paths,
        Err(error) => return Err(rollback_staging_error(&staging_dir, error)),
    };
    if let Err(error) = commit_staging(&staging_dir, &final_dir) {
        let commit_error = io::Error::new(
            error.kind(),
            format!("atomic bpffs link commit failed: {error}"),
        );
        return Err(rollback_staging_error(&staging_dir, commit_error));
    }
    return Ok(pinned.len());
}

/// Detaches only four exact persisted links for one canonical cgroup.
pub fn detach_cgroup(cgroup_dir: &Path) -> io::Result<()> {
    ensure_pin_root()?;
    let final_dir = pin_dir_for(cgroup_dir)?;
    return remove_exact_pin_dir(&final_dir);
}
