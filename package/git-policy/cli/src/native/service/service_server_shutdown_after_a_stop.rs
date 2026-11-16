//! What:
//!  The service's own shutdown:
//! a task of the runtime that waits for the stop handler's wake,
//! tells every waiter that the stop came,
//! waits a bounded time for the Git processes and the landing in flight to end,
//! asks for the markers made ahead to be removed and waits for that no longer than its own ceiling,
//! and ends the process by the signal that stopped it.
//! Why:
//!  The owner's pick for open choice 12 (2026-11-07T22:19:43-05:00,
//! verbatim "C:
//! shutdown stops its Gits (Recommended)"):
//! the handler only records the stop (`service_signal.rs`),
//! and this task,
//! which is the service's own code and not a handler,
//! is what stops the Gits,
//! with a bound on every wait.
//! Each waiter ends the one child it owns when it is told (`stop_of_the_service_noticed_by_its_children.rs`),
//! so there is no table of children and no lock.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settled = await tellTheStopAndWait(SETTLE_CEILING); log(settled); process.kill(process.pid, stopSignal);
//! ```

use crate::journald_native::send_without_waiting;
use crate::service_runtime_build_report::{EXIT_NO_WORKER, no_worker_words, start_what_the_service_cannot_serve_without};
use crate::service_sweep_started_at_the_stop::{Swept, TheSweepOfMarkersAtTheStop, start_the_sweep_at_the_stop};
use crate::service_timing::LOG_PREFIX;
use crate::stop_of_the_service_noticed_by_its_children::{SETTLE_CEILING, Settled, end_by_the_signal, is_noticing, stopping_signal, tell_the_stop, unfinished_work, wait_for_the_work};
use crate::wait_ceilings::SWEEP_CEILING;
use std::time::{Duration, Instant};

/// What:
///  Starts the shutdown task on the process's runtime,
/// waiting `RUNTIME_BUILD_PATIENCE` for a build that missed its deadline,
/// or ends the service with a plain line and status 4 when no worker arrived.
/// Why:
///  A service that cannot shut down in order would end a stop by the last-resort alarm only,
/// and the acceptor needs the same worker,
/// so a service without one cannot serve at all;
/// a build that merely outlasted `BUILD_DEADLINE` is waited for rather than given up on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// startOrExit('the shutdown after a stop', () => shutDownAfterAStop());
/// ```
pub(crate) fn start_shutdown_after_a_stop_or_exit() {
    // A process that never prepared to notice a stop (a development run, or a control that serves on a thread of its own) has no
    //  handler to wake a shutdown, and a shutdown with nothing to wait for must not end it.
    if !is_noticing() {
        return;
    }
    if start_what_the_service_cannot_serve_without("the shutdown after a stop", shut_down_after_a_stop()).is_none() {
        crate::service_error!("{}", no_worker_words("the shutdown after a stop"));
        std::process::exit(EXIT_NO_WORKER);
    }
}

/// Waits for a stop,
/// tells it,
/// starts the thread of the sweep the stop ends with,
/// waits for the work in flight within the bound,
/// asks for the removal of the markers made ahead within its own bound,
/// and ends the process by the signal.
/// Every line it writes never waits for room:
/// a service whose standard error is a full pipe must still end,
/// as the handler's line never waits either (`service_signal.rs`).
async fn shut_down_after_a_stop() {
    tell_the_stop().await;
    // The sweep's thread is started here, at the stop, and asked at the end (`service_sweep_started_at_the_stop.rs`):
    //  a thread started where the sweep is asked for would put the start's own wait inside the grace the alarm leaves.
    let sweep: TheSweepOfMarkersAtTheStop = start_the_sweep_at_the_stop();
    let settled: Settled = wait_for_the_work(SETTLE_CEILING).await;
    if settled == Settled::AtTheBound {
        let line: String = format!(
            "{LOG_PREFIX}stopped: {} pieces of work were still unfinished after {} seconds, so the service ends now and leaves them\n",
            unfinished_work(),
            SETTLE_CEILING.as_secs()
        );
        // A line that cannot be written without waiting is lost: the exit is what matters.
        let _written = send_without_waiting(libc::STDERR_FILENO, line.as_bytes());
    }
    // The stop waits for the sweep up to `SWEEP_CEILING` and no longer; a sweep that is keeping up removes two files.
    let began: Instant = Instant::now();
    match sweep.ask(SWEEP_CEILING) {
        Swept::Completely => {}
        Swept::AtTheBound => say_which_removal_was_left(began.elapsed(), None),
        Swept::NoThread => say_which_removal_was_left(Duration::ZERO, Some("no thread could be started for it")),
    }
    // A wake without a recorded signal cannot happen (the handler records first); nothing is ended for it.
    let Some(signal) = stopping_signal() else {
        return;
    };
    end_by_the_signal(signal);
}

/// What:
///  Says, in the service's log,
///  that the removal of the markers made ahead did not finish,
///  which wait it was,
///  how long it waited,
///  and who removes what it did not reach.
/// Why:
///   A stop that gives up the sweep leaves markers in the person's Git directory,
///  and a person or a later reader of the journal must find why they are there without a stack or a guess;
///  the sweep is the only wait of this shutdown that can be given up,
///  so the line names it and its duration rather than saying that something was left.
///  The line goes through the service's own log,
/// and the process waits for that log once,
/// bounded,
/// before it raises:
///  a line that is still in the queue when the signal ends the process is never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceWarn(`stopped: the removal of the markers made ahead was still unfinished after ${ms} ms, …`); writeWhatWaits(WAIT_BEFORE_AN_EXIT)
/// ```
///
/// @param waited - how long the stop waited for the sweep
///
/// @param why - what kept the sweep from running at all, when a thread was refused
///
/// @example
/// ```ts
/// say_which_removal_was_left(began.elapsed(), undefined);
/// ```
fn say_which_removal_was_left(waited: Duration, why: Option<&str>) {
    match why {
        Some(reason) => crate::service_warn!("stopped: the removal of the markers made ahead did not run, because {reason}: they are left where they are, and the next command that takes this index removes them"),
        None => crate::service_warn!("stopped: the removal of the markers made ahead was still unfinished after {} ms: they are left where they are, and the next command that takes this index removes them", waited.as_millis()),
    }
    // The line is written by a thread of the log's own; the end of this process waits for it once, bounded,
    //  as the end of every other command of the wrapper does.
    let _written: bool = crate::service_log_lines_off_the_calling_thread::write_what_waits(crate::service_log_lines_off_the_calling_thread::WAIT_BEFORE_AN_EXIT);
}
