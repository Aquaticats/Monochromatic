//! Records of a server process that ended unexpectedly or did not finish starting,
//!  with the last
//! lines it wrote to standard error (`src/logging/stderr_tail.rs`).
//!  The lines go into the log only;
//! no user-facing note shows them.

/// The kept standard-error lines of each server.
use crate::logging::stderr_tail;
/// What:
///  `Duration` is a time span;
///  `Instant` is a point on the runtime's clock.
/// Why:
///  The record of an ended server waits a bounded time for the end of its standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const until = performance.now() + 1000;
/// ```
use tokio::time::{Duration, Instant};

/// Longest wait,
///  after a server process ended,
///  for the end of its standard error,
///  so that the
/// lines it wrote just before it ended are in the record.
///  helix-lsp reads standard output and
/// standard error in separate tasks,
///  so the worker can learn of the end before the last lines
/// arrive;
///  the process is gone,
///  so the end of the stream follows at once unless the machine is
/// very busy or another process still holds the stream open.
pub const SETTLE: Duration = Duration::from_secs(1);

/// Pause between two looks at the stream while the record waits.
const SETTLE_POLL: Duration = Duration::from_millis(10);

/// What:
///  The record of one ended server process,
///  written when this value is dropped.
///  `Drop` is the
///       code Rust runs when a value goes away.
/// Why:
///  The value is dropped when the wait is over,
///  and also when the worker's runtime stops
///      during the wait and discards the waiting task,
///  so the record is written either way.
struct EndRecord {
    /// The server's configured name.
    server: String,
    /// Whether the server had finished starting.
    was_ready: bool,
}

/// Writing the record.
impl Drop for EndRecord {
    /// What:
    ///  Write the warning with the lines kept by now,
    ///  if any.
    /// Why:
    ///  A server that wrote nothing gets the same record without the `stderr_tail` field.
    fn drop(&mut self) {
        let lines = stderr_tail::take(&self.server);
        if lines.is_empty() {
            tracing::warn!(server = %self.server, was_ready = self.was_ready, "language server process ended");
        } else {
            tracing::warn!(server = %self.server, was_ready = self.was_ready, stderr_tail = ?lines, "language server process ended");
        }
    }
}

/// What:
///  Log that a server process ended without being asked to,
///  with its last standard-error
///       lines,
///  once its standard error ended or `SETTLE` passed.
///  `tokio::spawn` runs the wait as
///       its own task,
///  so the worker goes on at once.
/// Why:
///  Session state changes at once;
///  only this record waits for the lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endedUnexpectedly(server: string, wasReady: boolean): void {
///   void (async () => { while (!closed(server) && now() < until) await sleep(10); warn(...); })();
/// }
/// ```
pub(super) fn ended_unexpectedly(server: String, was_ready: bool) {
    ended_unexpectedly_then(server, was_ready, Nothing);
}

/// What:
///  Whoever wants the server's last standard-error lines once they are complete.
///  A type that implements this trait carries its own state into the waiting task,
///  and the task consumes it by calling `after_end` once.
/// Why:
///  A named type carries what the receiver needs without an anonymous function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface AfterEnd { afterEnd(lines: string[]): void }
/// ```
pub(super) trait AfterEnd: Send + 'static {
    /// Take the complete lines.
    fn after_end(self, lines: Vec<String>);
}

/// The receiver that does nothing with the lines.
struct Nothing;

/// Dropping the lines is the whole job of this receiver.
impl AfterEnd for Nothing {
    /// Ignore the lines.
    fn after_end(self, _lines: Vec<String>) {}
}

/// What:
///  `ended_unexpectedly`,
///  and hand a copy of the lines to `then` before the record drains them.
/// Why:
///  The worker states why a start failed from the same lines the record reports,
///  once they are complete;
///  the record still reports them exactly as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endedUnexpectedlyThen(server: string, wasReady: boolean, then: (lines: string[]) => void): void
/// ```
pub(super) fn ended_unexpectedly_then(server: String, was_ready: bool, then: impl AfterEnd) {
    let record = EndRecord { server, was_ready };
    tokio::spawn(async move {
        let started = Instant::now();
        let mut looks: u32 = 0;
        while !stderr_tail::is_closed(&record.server) && started.elapsed() < SETTLE {
            tokio::time::sleep(SETTLE_POLL).await;
            looks += 1;
        }
        if looks > 0 {
            let closed = stderr_tail::is_closed(&record.server);
            tracing::debug!(server = %record.server, waited_ms = started.elapsed().as_millis(), closed, "waited for the end of a server's standard error");
        }
        then.after_end(stderr_tail::lines(&record.server));
        drop(record);
    });
}

/// What:
///  Log the lines a server wrote to standard error before the worker stopped it because it
///       did not finish starting in time;
///  nothing when it wrote none.
///  Follows the worker's error
///       record of the timed-out start.
///  The lines are returned,
///  oldest first,
///  for the reason shown for the failed start.
/// Why:
///  A server that hangs in its start often says why there.
///  A record of its own keeps the
///      start-deadline code as it is.
pub(super) fn stopped_during_start(server: &str) -> Vec<String> {
    let lines = stderr_tail::take(server);
    if !lines.is_empty() {
        tracing::warn!(server = %server, stderr_tail = ?lines, "language server wrote this to standard error before it was stopped");
    }
    return lines;
}

/// Waiting for the last lines,
///  the record when the runtime stops,
///  and the field's absence.
#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
