//! Watches user application cgroups and retains marker links for selected named and executable-owned targets.

/// Target discovery across named app-slice services and executable-owned processes.
use crate::application_targets::{scan_application_targets, ScanRoots};
/// Unpinned marker links owned by watcher lifetime.
use crate::bpf::{attach_marker_unpinned, MarkerLinks};
/// Polling and syscall failures.
use std::io;
/// Raw and owned descriptor support.
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
/// Raw Unix path bytes for inotify watch registration.
use std::os::unix::ffi::OsStrExt;
/// Target paths.
use std::path::{Path, PathBuf};

/// Periodic process rescan closes browser moves into already-existing cgroups.
const RESCAN_INTERVAL_MS: libc::c_int = 250;
/// Buffer size for draining cgroup directory inotify records.
const INOTIFY_BUFFER_SIZE: usize = 16_384;

/// One cgroup and four link descriptors retained by watcher.
struct ActiveAttachment {
    /// Cgroup identity used for reconciliation.
    path: PathBuf,
    /// Four links whose descriptors keep marker active.
    _links: MarkerLinks,
}

/// Prepared watcher owns event descriptors and active target links.
pub struct ApplicationWatch<'a> {
    /// Target scanner roots.
    roots: ScanRoots<'a>,
    /// Socket mark applied to all targets.
    mark: u32,
    /// Directory change notifications.
    inotify: OwnedFd,
    /// Blocked shutdown signals delivered through pollable descriptor.
    signals: OwnedFd,
    /// Current cgroups and retained links.
    active: Vec<ActiveAttachment>,
}

/// Creates owned descriptor from successful nonnegative syscall return.
fn owned_descriptor(raw: libc::c_int) -> io::Result<OwnedFd> {
    if raw < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful syscall returned a new descriptor owned by caller.
    return Ok(unsafe { OwnedFd::from_raw_fd(raw) });
}

/// Blocks termination signals and returns signalfd receiving them.
fn create_signal_fd() -> io::Result<OwnedFd> {
    // SAFETY: zeroed signal set is initialized immediately by sigemptyset.
    let mut mask: libc::sigset_t = unsafe { std::mem::zeroed() };
    // SAFETY: mask points to writable signal set.
    if unsafe { libc::sigemptyset(&mut mask) } < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: mask remains initialized and writable.
    if unsafe { libc::sigaddset(&mut mask, libc::SIGTERM) } < 0
        || unsafe { libc::sigaddset(&mut mask, libc::SIGINT) } < 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: blocks selected signals in dedicated watcher process before child threads exist.
    let mask_result = unsafe {
        libc::pthread_sigmask(libc::SIG_BLOCK, &mask, std::ptr::null_mut())
    };
    if mask_result != 0 {
        return Err(io::Error::from_raw_os_error(mask_result));
    }
    // SAFETY: signalfd copies mask during call and returns newly owned descriptor.
    let raw = unsafe {
        libc::signalfd(
            -1,
            &mask,
            libc::SFD_CLOEXEC | libc::SFD_NONBLOCK,
        )
    };
    return owned_descriptor(raw);
}

/// Creates inotify descriptor and installs watch before first target scan.
fn create_inotify(app_slice: &Path) -> io::Result<OwnedFd> {
    // SAFETY: init returns newly owned descriptor on success.
    let inotify = owned_descriptor(unsafe {
        libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK)
    })?;
    let path = std::ffi::CString::new(app_slice.as_os_str().as_bytes()).map_err(|_| {
        return io::Error::new(io::ErrorKind::InvalidInput, "app.slice path contains NUL");
    })?;
    let mask = libc::IN_CREATE | libc::IN_MOVED_TO | libc::IN_DELETE | libc::IN_MOVED_FROM;
    // SAFETY: descriptor is inotify instance and path remains NUL-terminated through call.
    let watch = unsafe { libc::inotify_add_watch(inotify.as_raw_fd(), path.as_ptr(), mask) };
    if watch < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(inotify);
}

/// Opens cgroup and transactionally creates four links when target still exists.
fn attach_target(path: &Path, mark: u32) -> io::Result<Option<ActiveAttachment>> {
    let cgroup = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let links = attach_marker_unpinned(cgroup.as_raw_fd(), mark)?;
    return Ok(Some(ActiveAttachment {
        path: path.to_path_buf(),
        _links: links,
    }));
}

/// Reconciles discovered targets and retains attached cgroups until directory removal.
fn reconcile(watch: &mut ApplicationWatch<'_>) -> io::Result<()> {
    watch.active.retain(|attachment| return attachment.path.is_dir());
    let targets = scan_application_targets(&watch.roots)?;
    for target in targets {
        if watch.active.iter().any(|attachment| return attachment.path == target) {
            continue;
        }
        if let Some(attachment) = attach_target(&target, watch.mark)? {
            watch.active.push(attachment);
        }
    }
    return Ok(());
}

/// Drains every pending inotify record so next scan follows all coalesced changes.
fn drain_inotify(inotify: &OwnedFd) -> io::Result<()> {
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

/// Reads one signalfd record and confirms shutdown event was delivered.
fn consume_signal(signals: &OwnedFd) -> io::Result<()> {
    // SAFETY: zeroed record is writable output storage for signalfd read.
    let mut signal: libc::signalfd_siginfo = unsafe { std::mem::zeroed() };
    // SAFETY: output pointer and length describe complete signalfd record.
    let count = unsafe {
        libc::read(
            signals.as_raw_fd(),
            std::ptr::addr_of_mut!(signal).cast(),
            std::mem::size_of::<libc::signalfd_siginfo>(),
        )
    };
    if count < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(());
}

/// Installs watch before scan,
///  then rescans after draining creation race events.
pub fn prepare_application_watch(
    roots: ScanRoots<'_>,
    mark: u32,
) -> io::Result<ApplicationWatch<'_>> {
    let inotify = create_inotify(roots.app_slice)?;
    let signals = create_signal_fd()?;
    let mut watch = ApplicationWatch {
        roots,
        mark,
        inotify,
        signals,
        active: Vec::new(),
    };
    reconcile(&mut watch)?;
    drain_inotify(&watch.inotify)?;
    reconcile(&mut watch)?;
    return Ok(watch);
}

/// Polls cgroup events plus signals and periodically rescans executable-owned processes.
pub fn run_application_watch(watch: &mut ApplicationWatch<'_>) -> io::Result<()> {
    loop {
        let mut descriptors = [
            libc::pollfd {
                fd: watch.inotify.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: watch.signals.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        // SAFETY: descriptor array is valid for synchronous poll duration.
        let result = unsafe {
            libc::poll(
                descriptors.as_mut_ptr(),
                descriptors.len() as libc::nfds_t,
                RESCAN_INTERVAL_MS,
            )
        };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if descriptors[1].revents & libc::POLLIN != 0 {
            consume_signal(&watch.signals)?;
            return Ok(());
        }
        if descriptors[0].revents & libc::POLLIN != 0 {
            drain_inotify(&watch.inotify)?;
        }
        if let Err(error) = reconcile(watch) {
            eprintln!("application exemption rescan failed: {error}");
        }
    }
}
