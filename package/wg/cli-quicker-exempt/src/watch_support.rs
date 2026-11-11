//! The leaf helpers of the application watcher:
//! the directory notification on the app slice,
//! and the marker links retained for each target cgroup.

/// Marker links on a target cgroup.
use crate::bpf::{attach_marker_unpinned, MarkerLinks};
/// Polling and syscall failures.
use std::io;
/// Raw and owned descriptor support.
use std::os::fd::{AsRawFd, OwnedFd};
/// Raw Unix path bytes for inotify watch registration.
use std::os::unix::ffi::OsStrExt;
/// Target paths.
use std::path::{Path, PathBuf};
/// Shutdown signal descriptors.
use crate::signal_descriptor::owned_descriptor;

/// Buffer size for draining cgroup directory inotify records.
const INOTIFY_BUFFER_SIZE: usize = 16_384;

/// One cgroup and four link descriptors retained by watcher.
pub struct ActiveAttachment {
    /// Cgroup identity used for reconciliation.
    pub path: PathBuf,
    /// Four links whose descriptors keep marker active.
    pub _links: MarkerLinks,
}

/// Reports the `app.slice` path error for an embedded NUL byte.
fn app_slice_contains_nul(_error: std::ffi::NulError) -> io::Error {
    return io::Error::new(io::ErrorKind::InvalidInput, "app.slice path contains NUL");
}

/// Reports whether an attachment's cgroup directory still exists.
pub fn attachment_directory_exists(attachment: &ActiveAttachment) -> bool {
    return attachment.path.is_dir();
}

/// Reports whether `target` already has an attachment.
pub fn is_attached(active: &[ActiveAttachment], target: &Path) -> bool {
    for attachment in active {
        if attachment.path == target {
            return true;
        }
    }
    return false;
}

/// Creates inotify descriptor and installs watch before first target scan.
pub fn create_inotify(app_slice: &Path) -> io::Result<OwnedFd> {
    // SAFETY: init returns newly owned descriptor on success.
    let inotify = owned_descriptor(unsafe {
        libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK)
    })?;
    let path = std::ffi::CString::new(app_slice.as_os_str().as_bytes()).map_err(app_slice_contains_nul)?;
    let mask = libc::IN_CREATE | libc::IN_MOVED_TO | libc::IN_DELETE | libc::IN_MOVED_FROM;
    // SAFETY: descriptor is inotify instance and path remains NUL-terminated through call.
    let watch = unsafe { libc::inotify_add_watch(inotify.as_raw_fd(), path.as_ptr(), mask) };
    if watch < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(inotify);
}

/// Opens cgroup and transactionally creates four links when target still exists.
pub fn attach_target(path: &Path, mark: u32) -> io::Result<Option<ActiveAttachment>> {
    let cgroup = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if crate::process_info::means_vanished(&error) => return Ok(None),
        Err(error) => return Err(error),
    };
    let links = attach_marker_unpinned(cgroup.as_raw_fd(), mark)?;
    return Ok(Some(ActiveAttachment {
        path: path.to_path_buf(),
        _links: links,
    }));
}

/// Drains every pending inotify record so next scan follows all coalesced changes.
pub fn drain_inotify(inotify: &OwnedFd) -> io::Result<()> {
    let mut buffer = [0_u8; INOTIFY_BUFFER_SIZE];
    loop {
        // SAFETY: buffer is writable and descriptor is nonblocking inotify stream.
        let count = unsafe {
            libc::read(
                inotify.as_raw_fd(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
            )
        };
        if count > 0 {
            continue;
        }
        if count == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::WouldBlock {
            return Ok(());
        }
        return Err(error);
    }
}
