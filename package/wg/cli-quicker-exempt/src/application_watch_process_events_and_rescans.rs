//! The process events the kernel sends,
//! the rescan that follows a drop,
//! and how often a full scan runs.

/// The watcher these events belong to.
use super::ApplicationWatch;
/// The report of one outcome.
use super::watch_run_and_poll::report_outcomes;
/// Non-panicking bounded log.
use crate::log_line::log_line;
/// The subscription,
/// and the way it is ended.
use crate::process_events::{
    drain_exec_events,
    unsubscribe,
};
/// The clock of the periodic scan,
/// and the trait that asks whether a shutdown signal arrived.
use crate::routing_support::{
    StopCheck,
    SystemClock,
};
/// The shutdown signals a pass watches.
use crate::signal_descriptor::SignalPending;
/// The words of a policy failure.
use crate::update_report::policy_failure;
/// Timing of the process scan.
use std::time::{
    Duration,
    Instant,
};

/// Periodic process rescan closes browser moves into already-existing cgroups.
pub(super) const RESCAN_INTERVAL_MS: libc::c_int = 250;

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
pub fn rescan_due(
    overrun_pending: bool,
    since_last_scan: Duration,
) -> bool {
    return since_last_scan >= FULL_SCAN_INTERVAL
        || (overrun_pending && since_last_scan >= OVERRUN_RESCAN_SPACING);
}

/// Ends the subscription to process events and drops the socket,
/// and says so when the kernel could not be told.
/// The kernel keeps counting a listener until it is told to ignore,
/// whatever becomes of the socket.
pub(super) fn stop_process_events(watch: &mut ApplicationWatch<'_>) {
    if let Some(socket) = watch.events.take()
        && let Err(error) = unsubscribe(&socket)
    {
        log_line(&format!(
            "the process-event subscription could not be ended: {error}"
        ));
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
pub(super) fn handle_process_events(watch: &mut ApplicationWatch<'_>) {
    watch.routing.begin_pass();
    let drain_result = watch.events.as_ref().map(drain_exec_events);
    match drain_result {
        Some(Ok(batch)) => {
            if batch.overrun {
                watch.reporter.report_failure(
                    "process events dropped",
                    &policy_failure(
                        "the kernel dropped process events; every process is rescanned now",
                    ),
                );
                watch.rescan_pending = true;
            }
            for pid in batch.pids {
                if (SignalPending {
                    signals: &watch.signals,
                })
                .stop_requested()
                {
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
        let outcomes = watch.routing.rescan_with(
            &SystemClock,
            &SignalPending {
                signals: &watch.signals,
            },
        );
        report_outcomes(&mut watch.reporter, &outcomes);
        watch.last_full_scan = Instant::now();
        watch.rescan_pending = false;
    }
}
