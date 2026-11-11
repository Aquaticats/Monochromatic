//! Watches user application cgroups and retains marker links for selected named and executable-owned targets.

/// Target discovery across named app-slice services and executable-owned processes.
use crate::application_targets::{scan_application_targets, ScanRoots};
/// Process events and the update traffic policy.
use crate::process_events::{drain_exec_events, subscribe, unsubscribe};
/// Delivery of failures after startup.
use crate::runtime_report::RuntimeReporter;
/// The one-time sweep of the scopes of the earlier design,
/// and the pass over the origin scopes that must run before the first adoption.
use crate::leftover_sweep::{describe_sweep, spawn_sweep, stop_empty_origin_scopes_now, SweepSetup};
/// The guard of the manager's bus name.
use crate::impostor_guard;
/// Non-panicking bounded log.
use crate::log_line::{log_line, sanitize};
/// Shutdown signal descriptors.
use crate::signal_descriptor::{consume_signal, create_signal_fd, SignalPending};
/// Policy engine and its seams.
use crate::routing_support::{StopCheck, SystemClock};
/// Policy engine.
use crate::update_routing::UpdateRouting;
/// The words of the policy.
use crate::update_report::{describe_outcome, policy_failure, Outcome, POLICY_PROCESS_WATCH};
/// Timing of the process scan.
use std::time::{Duration, Instant};
/// Cgroup paths of the marks the watcher holds.
use std::path::PathBuf;
/// The directory notification and the marker links retained for each target.
#[path = "watch_support.rs"]
mod watch_support;
/// The items of that module the watcher uses.
use watch_support::{attach_target, attachment_directory_exists, create_inotify, drain_inotify, is_attached, ActiveAttachment};
/// Polling and syscall failures.
use std::io;
/// Raw and owned descriptor support.
use std::os::fd::{AsRawFd, OwnedFd};

/// Who the failure reporter notifies.
#[derive(Clone, Copy)]
pub struct ReporterIdentity {
    /// User id of the watched user.
    pub uid: u32,
    /// Group id of the watched user.
    pub gid: u32,
}

/// Periodic process rescan closes browser moves into already-existing cgroups.
const RESCAN_INTERVAL_MS: libc::c_int = 250;
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
    /// Whether the kernel dropped process events and the rescan that finds the lost ones has not run yet.
    rescan_pending: bool,
    /// Delivers failures after startup to a person.
    reporter: RuntimeReporter,
    /// Who the reporter notifies,
    /// which the guard of the manager's bus name also runs as.
    identity: ReporterIdentity,
    /// The sweep of the scopes of the earlier design,
    /// started once when the watcher runs and absent in tests.
    sweep: Option<SweepSetup>,
    /// Cgroups a desktop application's name selected that held no program of that application,
    /// as last reported,
    /// so a rescan every quarter second does not repeat the same line for ever.
    reported_without_application: Vec<PathBuf>,
    /// The guard of the manager's bus name this watcher started,
    /// held so the guard ends when the watcher does:
    /// dropping the handle closes the guard's standard input.
    guard: Option<impostor_guard::GuardHandle>,
}

/// Time between full process scans,
/// the backstop for events the kernel dropped.
/// It is a duration and not a count of loop wakeups,
/// because a busy event stream wakes the loop far more often than the poll timeout.
const FULL_SCAN_INTERVAL: Duration = Duration::from_secs(2);

/// Least time between two full process scans when the kernel dropped process events.
/// A scan reads every process on the host (about 26 ms of CPU for 949 processes,
/// measured),
/// and a host that drops events is a busy one,
/// so a scan after every dropped batch would spend a large share of a CPU on a storm.
/// The drop is still reported at once,
/// and a missed exec is found within this time.
const OVERRUN_RESCAN_SPACING: Duration = Duration::from_secs(1);

/// Reports whether a full process scan is due:
/// always after the scan interval,
/// and after a drop of process events once the spacing since the last scan has passed.
pub fn rescan_due(overrun_pending: bool, since_last_scan: Duration) -> bool {
    return since_last_scan >= FULL_SCAN_INTERVAL || (overrun_pending && since_last_scan >= OVERRUN_RESCAN_SPACING);
}

/// Reconciles discovered targets and retains attached cgroups until directory removal.
///
/// A name-selected cgroup that holds no program of the application the name claims is not marked,
/// and the watcher says so once for each such cgroup,
/// because an application whose program the scan cannot recognize would otherwise lose its exemption in silence.
/// The marks the watcher holds are also what the process watch treats as covered,
/// so the two can never disagree about which cgroup carries the mark.
fn reconcile(watch: &mut ApplicationWatch<'_>) -> io::Result<()> {
    watch.active.retain(attachment_directory_exists);
    let scan = scan_application_targets(&watch.roots)?;
    for target in scan.targets {
        if is_attached(&watch.active, &target) {
            continue;
        }
        if let Some(attachment) = attach_target(&target, watch.mark)? {
            watch.active.push(attachment);
        }
    }
    // The cgroup paths of the marks the watcher holds.
    let marked: Vec<PathBuf> = watch.active.iter().map(|attachment| return attachment.path.clone()).collect();
    watch.routing.note_marked(&marked);
    for cgroup in scan.without_application {
        if watch.reported_without_application.contains(&cgroup) {
            continue;
        }
        watch.reported_without_application.push(cgroup.clone());
        watch.reporter.report_failure(
            "application unit without its program",
            &format!(
                "wg-quicker-exempt: POLICY NOT IN FORCE for {:?}: its name claims a desktop application, and no process in it runs that application's program, so the socket mark is not attached to it and the traffic of every process in it stays in the tunnel. A program of the application may be spelled in a way this build does not know, and `list-targets` shows the cgroups it does mark",
                cgroup.display(),
            ),
        );
    }
    return Ok(());
}

/// Installs watch before scan,
///  then rescans after draining creation race events.
/// The failure reporter is built here,
/// after the signal descriptor,
/// because its notification worker is a thread:
/// a thread created before `SIGTERM` is blocked would inherit the signal unblocked,
/// receive a process-directed `SIGTERM` itself,
/// and end the whole process without the descriptor ever seeing it.
pub fn prepare_application_watch(
    roots: ScanRoots<'_>,
    mark: u32,
    routing: UpdateRouting,
    identity: ReporterIdentity,
    sweep: Option<SweepSetup>,
) -> io::Result<ApplicationWatch<'_>> {
    let inotify = create_inotify(roots.app_slice)?;
    let signals = create_signal_fd()?;
    let mut reporter = RuntimeReporter::new(identity.uid, identity.gid);
    reporter.begin_startup();
    if let Err(error) = routing.ensure_slice(roots.app_slice) {
        log_line(&policy_failure(&format!("the updates slice cannot be created: {error}")));
    }
    // The origin scopes are stopped before anything is adopted:
    // a process joining one is a process this watcher just moved there,
    // and a stop ends every process in the scope it stops.
    // The sweep of the earlier design's scopes runs on its own thread, after this.
    if let Some(setup) = sweep.as_ref()
        && let Some(line) = describe_sweep(&stop_empty_origin_scopes_now(setup))
    {
        log_line(&line);
    }
    if !routing.pid_namespace_known() {
        log_line(&policy_failure(
            "the watcher cannot read its own pid namespace, so a process in a container or a sandbox cannot be told apart and is treated like any other",
        ));
    }
    let events = match subscribe() {
        Ok(socket) => Some(socket),
        Err(error) => {
            log_line(&policy_failure(&format!(
                "the kernel process-event subscription failed: {error}; update entry points are found only by the periodic scan"
            )));
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
        rescan_pending: false,
        reporter,
        identity,
        sweep,
        reported_without_application: Vec::new(),
        guard: None,
    };
    reconcile(&mut watch)?;
    drain_inotify(&watch.inotify)?;
    reconcile(&mut watch)?;
    let outcomes = watch.routing.rescan_with(&SystemClock, &SignalPending { signals: &watch.signals });
    report_outcomes(&mut watch.reporter, &outcomes);
    return Ok(watch);
}

/// Delivers each outcome that needs saying:
/// a failure through the reporter,
/// by its kind,
/// so a person sees it,
/// and a successful move to the log.
fn report_outcomes(reporter: &mut RuntimeReporter, outcomes: &[Outcome]) {
    for outcome in outcomes {
        let Some(line) = describe_outcome(outcome) else {
            continue;
        };
        match outcome.failure_kind() {
            Some(kind) => {
                reporter.report_failure(kind, &line);
            }
            None => log_line(&line),
        }
    }
}

/// Reporting for tests.
#[cfg(test)]
impl ApplicationWatch<'_> {
    /// Reports whether the watcher still holds its process-event subscription.
    pub fn has_process_events(&self) -> bool {
        return self.events.is_some();
    }
}

/// Ends the subscription to process events and drops the socket,
/// and says so when the kernel could not be told.
/// The kernel keeps counting a listener until it is told to ignore,
/// whatever becomes of the socket.
fn stop_process_events(watch: &mut ApplicationWatch<'_>) {
    if let Some(socket) = watch.events.take()
        && let Err(error) = unsubscribe(&socket)
    {
        log_line(&format!("the process-event subscription could not be ended: {error}"));
    }
}

/// Applies the exec events the kernel queued,
/// and falls back to a full scan after an overrun,
/// no sooner than the overrun spacing after the last one,
/// or when the scan interval passed.
/// A shutdown signal that arrives meanwhile ends the work between two processes.
/// A socket that fails to be read is dropped,
/// so a persistent error is reported once instead of spinning the loop,
/// and the periodic scan carries on alone.
fn handle_process_events(watch: &mut ApplicationWatch<'_>) {
    watch.routing.begin_pass();
    let drain_result = watch.events.as_ref().map(drain_exec_events);
    match drain_result {
        Some(Ok(batch)) => {
            if batch.overrun {
                watch.reporter.report_failure("process events dropped", &policy_failure("the kernel dropped process events; every process is rescanned now"));
                watch.rescan_pending = true;
            }
            for pid in batch.pids {
                if (SignalPending { signals: &watch.signals }).stop_requested() {
                    return;
                }
                let outcome = watch.routing.consider(pid);
                report_outcomes(&mut watch.reporter, &[outcome]);
            }
        }
        Some(Err(error)) => {
            stop_process_events(watch);
            watch.reporter.report_failure(
                "process events unreadable",
                &policy_failure(&format!(
                    "reading process events failed: {error}; the socket is dropped, and update entry points are found only by the periodic scan from now on"
                )),
            );
        }
        None => {}
    }
    if rescan_due(watch.rescan_pending, watch.last_full_scan.elapsed()) {
        let outcomes = watch.routing.rescan_with(&SystemClock, &SignalPending { signals: &watch.signals });
        report_outcomes(&mut watch.reporter, &outcomes);
        watch.last_full_scan = Instant::now();
        watch.rescan_pending = false;
    }
}

/// Polls cgroup events plus signals and periodically rescans executable-owned processes.
/// A failure that ends the loop is reported loudly before it is returned,
/// because the watcher then stops enforcing every policy it owns.
pub fn run_application_watch(watch: &mut ApplicationWatch<'_>) -> io::Result<()> {
    watch.reporter.end_startup();
    // The sweep starts once the watcher is committed to run, so an abandoned start sweeps nothing.
    if let Some(setup) = watch.sweep.take() {
        spawn_sweep(setup);
    }
    // So does the guard of the manager's bus name,
    // which keeps a process that is not the user's manager from answering every `systemctl --user` call on the machine.
    let identity = watch.identity;
    match impostor_guard::spawn(identity.uid, identity.gid) {
        Ok(guard) => {
            log_line(&format!("wg-quicker-exempt: guarding {} on the user's session bus", impostor_guard::MANAGER_BUS_NAME));
            // Held by the watch until the watcher ends, and closed with the process if it is killed:
            // the guard ends when its input closes, so no guard outlives its watcher.
            watch.guard = Some(guard);
        }
        Err(error) => {
            watch.reporter.report_failure(
                "impostor manager",
                &format!(
                    "wg-quicker-exempt: {}: the guard could not start ({error}), so a process that takes the manager's bus name would answer every `systemctl --user` call on this machine for as long as it runs; the policy's own calls go over the manager's private socket and are not affected",
                    impostor_guard::LOUD_MARKER
                ),
            );
        }
    }
    let result = poll_until_stopped(watch);
    watch.guard = None;
    stop_process_events(watch);
    if let Err(error) = &result {
        watch.reporter.report_failure(
            "watcher stopped",
            &format!(
                "wg-quicker-exempt: POLICY NOT IN FORCE: socket marking of application and update cgroups, and {POLICY_PROCESS_WATCH}: \
the watcher stopped because {}, and neither is enforced any more; run wg-quicker up again after fixing the cause",
                sanitize(&error.to_string())
            ),
        );
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
                watch.reporter.report_failure(
                    "socket marking rescan failed",
                    &format!(
                        "wg-quicker-exempt: POLICY NOT IN FORCE: socket marking of application and update cgroups: the rescan failed: {}",
                        sanitize(&error.to_string())
                    ),
                );
            }
        }
        handle_process_events(watch);
    }
}
