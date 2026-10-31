//! Watches user application cgroups and retains marker links for selected named and executable-owned targets.

/// Target discovery across named app-slice services and executable-owned processes.
use crate::application_targets::{scan_application_targets, ScanRoots};
/// Process events and the update traffic policy.
use crate::process_events::{drain_exec_events, subscribe};
/// Delivery of failures after startup.
use crate::runtime_report::RuntimeReporter;
/// Policy engine.
use crate::update_routing::UpdateRouting;
/// The words of the policy.
use crate::update_report::{describe_outcome, policy_failure, Outcome, POLICY_PROCESS_WATCH};
/// Timing of the process scan.
use std::time::{Duration, Instant};
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
    /// Update traffic policy for the watched user.
    routing: UpdateRouting,
    /// Process-event connector,
    /// absent when the subscription failed and was reported.
    events: Option<OwnedFd>,
    /// When the last full process scan finished.
    last_full_scan: Instant,
    /// Delivers failures after startup to a person.
    reporter: RuntimeReporter,
}

/// Time between full process scans,
/// the backstop for events the kernel dropped.
/// It is a duration and not a count of loop wakeups,
/// because a busy event stream wakes the loop far more often than the poll timeout.
const FULL_SCAN_INTERVAL: Duration = Duration::from_secs(2);

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

/// Reports the `app.slice` path error for an embedded NUL byte.
fn app_slice_contains_nul(_error: std::ffi::NulError) -> io::Error {
    return io::Error::new(io::ErrorKind::InvalidInput, "app.slice path contains NUL");
}

/// Reports whether an attachment's cgroup directory still exists.
fn attachment_directory_exists(attachment: &ActiveAttachment) -> bool {
    return attachment.path.is_dir();
}

/// Reports whether `target` already has an attachment.
fn is_attached(active: &[ActiveAttachment], target: &Path) -> bool {
    for attachment in active {
        if attachment.path == target {
            return true;
        }
    }
    return false;
}

/// Creates inotify descriptor and installs watch before first target scan.
fn create_inotify(app_slice: &Path) -> io::Result<OwnedFd> {
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
    watch.active.retain(attachment_directory_exists);
    let targets = scan_application_targets(&watch.roots)?;
    for target in targets {
        if is_attached(&watch.active, &target) {
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
    routing: UpdateRouting,
    reporter: RuntimeReporter,
) -> io::Result<ApplicationWatch<'_>> {
    let inotify = create_inotify(roots.app_slice)?;
    let signals = create_signal_fd()?;
    if let Err(error) = routing.ensure_slice(roots.app_slice) {
        eprintln!("{}", policy_failure(&format!("the updates slice cannot be created: {error}")));
    }
    let events = match subscribe() {
        Ok(socket) => Some(socket),
        Err(error) => {
            eprintln!(
                "{}",
                policy_failure(&format!(
                    "the kernel process-event subscription failed: {error}; update entry points are found only by the periodic scan"
                ))
            );
            None
        }
    };
    let mut watch = ApplicationWatch {
        roots,
        mark,
        inotify,
        signals,
        active: Vec::new(),
        routing,
        events,
        last_full_scan: Instant::now(),
        reporter,
    };
    reconcile(&mut watch)?;
    drain_inotify(&watch.inotify)?;
    reconcile(&mut watch)?;
    let outcomes = watch.routing.rescan();
    report_outcomes(&mut watch.reporter, &outcomes);
    return Ok(watch);
}

/// Delivers each outcome that needs saying:
/// a failure through the reporter so a person sees it,
/// and a successful move to the log.
fn report_outcomes(reporter: &mut RuntimeReporter, outcomes: &[Outcome]) {
    for outcome in outcomes {
        let Some(line) = describe_outcome(outcome) else {
            continue;
        };
        if outcome.is_failure() {
            reporter.report_failure(&line);
        } else {
            eprintln!("{line}");
        }
    }
}

/// Applies the exec events the kernel queued,
/// and falls back to a full scan after an overrun or when the scan interval passed.
fn handle_process_events(watch: &mut ApplicationWatch<'_>) {
    let mut full_scan = false;
    watch.routing.begin_pass();
    if let Some(socket) = &watch.events {
        match drain_exec_events(socket) {
            Ok(drained) => {
                if drained.overrun {
                    watch.reporter.report_failure(&policy_failure("the kernel dropped process events; every process is rescanned now"));
                    full_scan = true;
                }
                for pid in drained.pids {
                    let outcome = watch.routing.consider(pid);
                    report_outcomes(&mut watch.reporter, &[outcome]);
                }
            }
            Err(error) => {
                watch.reporter.report_failure(&policy_failure(&format!(
                    "reading process events failed: {error}; the periodic scan continues"
                )));
            }
        }
    }
    if full_scan || watch.last_full_scan.elapsed() >= FULL_SCAN_INTERVAL {
        let outcomes = watch.routing.rescan();
        report_outcomes(&mut watch.reporter, &outcomes);
        watch.last_full_scan = Instant::now();
    }
}

/// Polls cgroup events plus signals and periodically rescans executable-owned processes.
/// A failure that ends the loop is reported loudly before it is returned,
/// because the watcher then stops enforcing every policy it owns.
pub fn run_application_watch(watch: &mut ApplicationWatch<'_>) -> io::Result<()> {
    let result = poll_until_stopped(watch);
    if let Err(error) = &result {
        watch.reporter.report_failure(&format!(
            "wg-quicker-exempt: POLICY NOT IN FORCE: socket marking of application and update cgroups, and {POLICY_PROCESS_WATCH}: \
the watcher stopped because {error}, and neither is enforced any more; run wg-quicker up again after fixing the cause"
        ));
    }
    return result;
}

/// The poll loop behind `run_application_watch`.
fn poll_until_stopped(watch: &mut ApplicationWatch<'_>) -> io::Result<()> {
    let mut last_reconcile = Instant::now();
    loop {
        let mut descriptors = vec![
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
        if let Some(socket) = &watch.events {
            descriptors.push(libc::pollfd {
                fd: socket.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            });
        }
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
        // Bursts of process events must not trigger a full target scan each time.
        let cgroup_event = descriptors[0].revents & libc::POLLIN != 0;
        if cgroup_event || last_reconcile.elapsed().as_millis() >= RESCAN_INTERVAL_MS as u128 {
            last_reconcile = Instant::now();
            if let Err(error) = reconcile(watch) {
                watch.reporter.report_failure(&format!(
                    "wg-quicker-exempt: POLICY NOT IN FORCE: socket marking of application and update cgroups: the rescan failed: {error}"
                ));
            }
        }
        handle_process_events(watch);
    }
}
