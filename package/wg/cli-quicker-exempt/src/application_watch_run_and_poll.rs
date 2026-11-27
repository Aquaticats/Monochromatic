//! The watcher run,
//! and the poll loop behind it.

/// The watcher this run belongs to.
use super::ApplicationWatch;
/// The mark of one cgroup.
use super::mark_reconciliation::reconcile;
/// The process events of one loop turn,
/// and how long a turn waits.
use super::process_events_and_rescans::{
    RESCAN_INTERVAL_MS,
    handle_process_events,
    stop_process_events,
};
/// The directory change notification.
use super::watch_support::drain_inotify;
/// The guard of the manager's bus name.
use crate::impostor_guard;
/// The one-time sweep of the scopes of the earlier design.
use crate::leftover_sweep::spawn_sweep;
/// Non-panicking bounded log.
use crate::log_line::{
    log_line,
    sanitize,
};
/// Delivery of failures after startup.
use crate::runtime_report::RuntimeReporter;
/// The shutdown signal of one loop turn.
use crate::signal_descriptor::consume_signal;
/// The words of the policy.
use crate::update_report::{
    Outcome,
    POLICY_PROCESS_WATCH,
    describe_outcome,
};
/// Polling and syscall failures.
use std::io;
/// Raw descriptor support.
use std::os::fd::AsRawFd;
/// The instants of a loop turn.
use std::time::Instant;

/// Delivers each outcome that needs saying:
/// a failure through the reporter,
/// by its kind,
/// so a person sees it,
/// and a successful move to the log.
pub(super) fn report_outcomes(
    reporter: &mut RuntimeReporter,
    outcomes: &[Outcome],
) {
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
            log_line(&format!(
                "wg-quicker-exempt: guarding {} on the user's session bus",
                impostor_guard::MANAGER_BUS_NAME
            ));
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
