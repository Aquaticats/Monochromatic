//! Watches user application cgroups and retains marker links for selected named and executable-owned targets.

/// The mark each cgroup holds.
mod mark_reconciliation;
/// The process events and the rescans that follow them.
mod process_events_and_rescans;
/// The watcher run and its poll loop.
mod watch_run_and_poll;

/// Whether a full process scan is due,
/// which only a test asks about on its own.
#[cfg(test)]
pub use process_events_and_rescans::rescan_due;
/// The watcher run and its poll loop.
pub use watch_run_and_poll::run_application_watch;

/// Target discovery across named app-slice services and executable-owned processes.
use crate::application_targets::ScanRoots;
/// The guard of the manager's bus name.
use crate::impostor_guard;
/// The one-time sweep of the scopes of the earlier design.
use crate::leftover_sweep::{
    SweepSetup,
    describe_sweep,
};
/// Non-panicking bounded log.
use crate::log_line::log_line;
/// The pass over the origin scopes that must run before the first adoption.
use crate::origin_scope_sweep::stop_empty_origin_scopes_now;
/// The kernel process-event subscription.
use crate::process_events::subscribe;
/// The clock of the scan before the watcher runs.
use crate::routing_support::SystemClock;
/// Delivery of failures after startup.
use crate::runtime_report::RuntimeReporter;
/// Shutdown signal descriptors.
use crate::signal_descriptor::{
    SignalPending,
    create_signal_fd,
};
/// The words of the policy.
use crate::update_report::policy_failure;
/// Policy engine.
use crate::update_routing::UpdateRouting;
/// The mark of one cgroup.
use mark_reconciliation::reconcile;
/// Polling and syscall failures.
use std::io;
/// Raw and owned descriptor support.
use std::os::fd::OwnedFd;
/// Cgroup paths of the marks the watcher holds.
use std::path::PathBuf;
/// Timing of the process scan.
use std::time::Instant;
/// The report of one outcome.
use watch_run_and_poll::report_outcomes;
/// The directory notification and the marker links retained for each target.
#[path = "watch_support.rs"]
mod watch_support;
/// The directory notification,
/// and the links retained for each target.
use watch_support::{
    ActiveAttachment,
    create_inotify,
    drain_inotify,
};

/// Who the failure reporter notifies.
#[derive(Clone, Copy)]
pub struct ReporterIdentity {
    /// User id of the watched user.
    pub uid: u32,
    /// Group id of the watched user.
    pub gid: u32,
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
        log_line(&policy_failure(&format!(
            "the updates slice cannot be created: {error}"
        )));
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
    let outcomes = watch.routing.rescan_with(
        &SystemClock,
        &SignalPending {
            signals: &watch.signals,
        },
    );
    report_outcomes(&mut watch.reporter, &outcomes);
    return Ok(watch);
}

/// Reporting for tests.
#[cfg(test)]
impl ApplicationWatch<'_> {
    /// Reports whether the watcher still holds its process-event subscription.
    pub fn has_process_events(&self) -> bool {
        return self.events.is_some();
    }
}
