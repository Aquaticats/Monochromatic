//! What:
//!  The service's own shutdown:
//! a task of the runtime that waits for the stop handler's wake,
//! tells every waiter that the stop came,
//! waits a bounded time for the Git processes and the landing in flight to end,
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
use crate::lazy_runtime::spawn_on_runtime;
use crate::service_timing::LOG_PREFIX;
use crate::stop_of_the_service_noticed_by_its_children::{SETTLE_CEILING, Settled, end_by_the_signal, is_noticing, stopping_signal, tell_the_stop_and_wait_for_the_work, unfinished_work};

/// What:
///  Starts the shutdown task on the process's runtime,
/// or ends the service with a plain line and status 4 when the runtime has no worker to run it.
/// Why:
///  A service that cannot shut down in order would end a stop by the last-resort alarm only,
/// and the acceptor needs the same worker,
/// so a service without one cannot serve at all.
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
    if spawn_on_runtime(shut_down_after_a_stop()).is_none() {
        crate::service_error!("cannot start the shutdown after a stop: the runtime that runs it has no worker thread (the system may refuse threads; check `ulimit -u` and the control group's `pids.max`)");
        std::process::exit(4);
    }
}

/// Waits for a stop,
/// tells it,
/// waits for the work in flight within the bound,
/// and ends the process by the signal.
/// The line it may write never waits for room:
/// a service whose standard error is a full pipe must still end,
/// as the handler's line never waits either (`service_signal.rs`).
async fn shut_down_after_a_stop() {
    let settled: Settled = tell_the_stop_and_wait_for_the_work(SETTLE_CEILING).await;
    if settled == Settled::AtTheBound {
        let line: String = format!(
            "{LOG_PREFIX}stopped: {} pieces of work were still unfinished after {} seconds, so the service ends now and leaves them\n",
            unfinished_work(),
            SETTLE_CEILING.as_secs()
        );
        // A line that cannot be written without waiting is lost: the exit is what matters.
        let _written = send_without_waiting(libc::STDERR_FILENO, line.as_bytes());
    }
    // The marker made ahead for the next batch is the service's own, and `SPEC.md` "Removing the lock and what is made durable"
    // says it "is removed when the service ends, whichever way its loop ends". This shutdown ends the process by the signal, so
    // no destructor of the serve loop runs; the file the landing thread made is removed here, once no landing is in flight, and
    // before the exit.
    crate::service_index_prestage::remove_the_file_made_ahead();
    // A wake without a recorded signal cannot happen (the handler records first); nothing is ended for it.
    let Some(signal) = stopping_signal() else {
        return;
    };
    end_by_the_signal(signal);
}
