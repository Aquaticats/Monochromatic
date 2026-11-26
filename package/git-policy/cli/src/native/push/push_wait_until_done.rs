//! What:
//!  The wait of a bounded push:
//! it ends when the run ends,
//! and otherwise when the run stops showing work,
//! writes more output than the cap,
//! or the service is stopped.
//! Why:
//!  A run that shows nothing and holds no connection would hold the service's push of that branch forever,
//!  and every later push of the repository would wait behind it;
//! this loop is where the run is ended instead,
//!  and the cap it is judged by,
//! with the words a wait adds to the attempt's output,
//!  live beside it so that each file stays short enough for the repository's `rust/max-lines` rule.
//!  A run whose connection is open is never stopped for time:
//! it has no ceiling,
//!  which is the `bounded_push` module's own account of the decision.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await Promise.race([exited, stopped, flood, stall]);
//! ```

/// The error a run that cannot be started or read reports.
use crate::auto_push_single_flight::SingleFlightError;
/// The child a wait awaits through its pidfd.
use crate::child_awaited_through_a_pidfd::OwnedChild;
/// The diagnostic log of the wrapper.
use crate::diagnostic_log::warn;
/// How long a run may show nothing before it is stopped,
/// and the window a connected one gets.
use crate::push_deadline_and_patience_settings::Patience;
/// The clock that stops a run on a stall,
/// and the wait it lets pass instead.
use crate::push_progress::{
    Clock,
    WaitReport,
};
/// The causes and reports of a stop.
use crate::stop_cause_and_words::StopCause;
/// The pause between two looks at a running push.
use crate::stop_of_a_push_and_sweep_of_its_locks::POLL_PAUSE;
/// The arm that says the service's shutdown told the stop.
use crate::stop_of_the_service_noticed_by_its_children::StopArm;
/// `File` is the shared anonymous output file of the run.
use std::fs::File;
/// The status a run ended with.
use std::process::ExitStatus;
/// Time spans of the stall and the connected window.
use std::time::Duration;

/// The most output bytes kept from one push;
///  a push that writes more is stopped.
///  Citation:
/// none yet.
/// The owner decided "Keep all lines" for push output,
/// which this cap limits,
/// and no measurement of the
///  size of a push's output is logged;
/// the audit lists it as a memory bound to be measured.
pub const MAX_PUSH_OUTPUT: u64 = 4 * 1024 * 1024;

/// What the line a push's output gains has after the name of the program,
/// when it waited with its connection open.
pub const WAIT_NOTE_PHRASE: &str = "is waiting for the server";

/// Whether an output of this size is a flood:
/// only a size past the cap is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// size > cap
/// ```
pub(crate) fn is_flood(
    size: u64,
    cap: u64,
) -> bool {
    return size > cap;
}

/// What:
///  Whether one of a run's output files has grown past the cap.
/// Why:
///   The wait looks at these between its polls,
/// and a run that ends in the pause between two looks must be judged by the same bound:
/// what it printed past the cap is not all of what it printed.
///  A size that cannot be read counts as past the cap,
/// so nothing unreadable passes for a complete answer either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outputs.some((output) => isFlood(statSize(output)))
/// ```
pub(crate) fn flooded(
    outputs: &[&File],
    cap: u64,
) -> bool {
    for output in outputs {
        let size: u64 = match output.metadata() {
            Ok(metadata) => metadata.len(),
            Err(_) => u64::MAX,
        };
        if is_flood(size, cap) {
            return true;
        }
    }
    return false;
}

/// What a clock keeps as the longest quiet gap:
/// its figure when it reads anything,
/// nothing when it reads nothing.
fn quiet_gap_kept(watched: &Clock) -> Option<Duration> {
    if watched.measures() {
        return Some(watched.longest_quiet());
    }
    return None;
}

/// What:
///  Look at a running read-only query until it ends,
/// stalls or floods its output,
/// and say how quiet it got.
/// Why:
///   `std` has no timed wait,
///  so the parent looks with `try_wait`,
///  which never blocks;
/// the clock reads the signs of work a few times a second.
/// A query that works is waited for however
///  long it takes:
/// it has no wall-clock bound (owner,
/// 2026-11-03:
/// "Stop only on no progress";
/// the coordinator's scope addition
///  to picks 4 and 9 of `doc/planning/cli-git-shipped-limits-audit.md`,
/// open to the owner's veto),
/// and on a system that counts
///  nothing it is never judged stalled.
/// `quiet` receives the longest quiet gap seen,
/// for the ledger,
/// and nothing where the system counts nothing.
/// The clock is the caller's,
/// because what a run may do where nothing can be measured is the caller's decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([exited, stalled, flood]); // never by elapsed time
/// ```
pub async fn wait_until_done(
    child: &mut OwnedChild,
    outputs: &[&File],
    clock: Clock,
    quiet: &mut Option<Duration>,
    stop: Option<&StopArm>,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    return wait_until_done_up_to(child, outputs, clock, MAX_PUSH_OUTPUT, quiet, stop).await;
}

/// What:
///  The same wait as `wait_until_done`,
/// with the output ceiling it stops past given by the caller.
/// Why:
///   A query's answer is a report,
/// and the push report's cap bounds it;
/// the second pass of a merge reads the three stages of the files it may merge,
/// up to the
///  pass's own blob limit for each name it asks for,
/// so its ceiling is that sum and not the report's.
/// Nothing else may pass a ceiling above `MAX_PUSH_OUTPUT`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([exited, stalled, flood]); // flood: any watched file past `cap`
/// ```
pub async fn wait_until_done_up_to(
    child: &mut OwnedChild,
    outputs: &[&File],
    clock: Clock,
    cap: u64,
    quiet: &mut Option<Duration>,
    stop: Option<&StopArm>,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    let mut waits: Vec<WaitReport> = Vec::new();
    return wait_with_clock(child, outputs, clock, cap, "git", &mut waits, quiet, stop).await;
}

/// What:
///  The same wait as `wait_until_done` for a push,
/// handing over the wait of a push that is waiting for its remote with the
///  connection open.
/// Why:
///   The push is not stopped while it waits;
/// the caller says so once,
/// in a line,
/// and the wait ends the way any other does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([exited, stalled, flood]); // waits.push(report) when the push begins to wait for the server
/// ```
pub async fn wait_until_done_noting(
    child: &mut OwnedChild,
    outputs: &[&File],
    patience: Patience,
    label: &str,
    waits: &mut Vec<WaitReport>,
    stop: Option<&StopArm>,
    window: Duration,
    minimum_rate: u64,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    let clock: Clock = Clock::start(child.id(), patience)
        .with_connected_window(window)
        .with_minimum_rate(minimum_rate);
    let mut unused: Option<Duration> = None;
    return wait_with_clock(
        child,
        outputs,
        clock,
        MAX_PUSH_OUTPUT,
        label,
        waits,
        &mut unused,
        stop,
    )
    .await;
}

/// The loop of both waits,
/// over a clock the caller made.
async fn wait_with_clock(
    child: &mut OwnedChild,
    outputs: &[&File],
    mut clock: Clock,
    cap: u64,
    label: &str,
    waits: &mut Vec<WaitReport>,
    quiet: &mut Option<Duration>,
    stop: Option<&StopArm>,
) -> Result<Result<ExitStatus, StopCause>, SingleFlightError> {
    // Nothing is kept where the clock reads nothing: its figure would be the elapsed time, not a quiet gap.
    loop {
        // The exit is awaited on the child's pidfd; the pause is the look at the output sizes and the clock, a few times a second.
        if let Ok(waited) = tokio::time::timeout(POLL_PAUSE, child.wait()).await {
            let status: ExitStatus = waited.map_err(SingleFlightError::Start)?;
            *quiet = quiet_gap_kept(&clock);
            return Ok(Ok(status));
        }
        // The service's shutdown told the stop: the wait ends here, and the caller ends the child with its own grace.
        if stop.is_some_and(StopArm::is_told) {
            *quiet = quiet_gap_kept(&clock);
            return Ok(Err(StopCause::ServiceStopped));
        }
        if flooded(outputs, cap) {
            *quiet = quiet_gap_kept(&clock);
            return Ok(Err(StopCause::Flood));
        }
        if let Some(cause) = clock.check() {
            *quiet = quiet_gap_kept(&clock);
            return Ok(Err(cause));
        }
        if let Some(report) = clock.take_wait_note() {
            // Said as the wait begins, so a long wait is read while it lasts (the service's log, or the person's terminal).
            // Tagged with the step that says it, as every diagnostic of a function boundary is: the line's own words carry the
            //  program the run is, so the logger's fixed name stands beside them once and not twice.
            warn("wait_with_clock", &wait_line_words(label, &report));
            waits.push(report);
        }
    }
}

/// What:
///  The words that name a run's wait for its remote,
///  without the name of the program the log line and the attempt's own output each carry once.
/// Why:
///   A wait is written twice:
///  as the diagnostic of the moment it begins,
///  whose line already names this program,
///  and in the attempt's output the person or the journal reads.
///  One sentence serves both,
///  so the two cannot say different things about the same wait,
///  and neither says the program's name twice.
///  A wait that shows nothing the wrapper counts as progress ends at the connected window,
/// which the report carries,
/// so the words name the same limit the run will be stopped by.
///  "Nothing this wrapper counts" and not "nothing":
/// a server that keeps the connection alive sends a keepalive packet now and then,
/// and a trickle below the minimum rate
///  (`push_minimum_rate.rs`) moves bytes that are not progress either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${label} ${WAIT_NOTE_PHRASE}: the connection to the remote is open (silent ${silent} seconds)`
/// ```
pub(crate) fn wait_line_words(
    label: &str,
    report: &WaitReport,
) -> String {
    return format!(
        "{label} {WAIT_NOTE_PHRASE}: the connection to the remote is open, and nothing the wrapper counts as \
         progress has moved for {} seconds (elapsed_seconds={}); the remote may be checking the push or preparing its answer. It is waited \
         for until it answers or the connection closes, and a connection that stays open with nothing the wrapper counts as progress for \
         {} seconds is stopped as a stall.",
        report.silent.as_secs(),
        report.elapsed.as_secs(),
        report.window.as_secs(),
    );
}
