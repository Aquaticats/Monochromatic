//! What:
//!  The loop of a child watch's task and the writing of its notices:
//! wait for the next mark or the next look at the child's work,
//! print or end the child,
//! repeat.
//! Why:
//!  `child_watch.rs` holds the watch the waiting thread uses and the running of watched steps;
//! the task's loop uses none of that,
//! so it sits in a file of its own and `child_watch.rs` starts it on the runtime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for await (const mark of marksUntilTimeout()) { if (stoppable && clock.stalled()) await stop(pid); else if (ticker.due(elapsed)) await notice(); }
//! ```

use crate::bounded_push::StopCause;
use crate::child_watch_clock_marks_and_stopping::{Mark, Marks, SHORTEST_SLEEP, WatchFlags, WatchTerms, stop_process};
use crate::push_progress::{Clock, SAMPLE_INTERVAL};
use crate::stopped_git_locks::LockLedger;
use crate::wait_progress::{ChildWaitFacts, Ticker, child_wait_line};
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

/// What:
///  The watcher task's loop:
/// wait until the next notice or the next look at the child's work is due,
/// print or end,
/// repeat.
/// Why:
///   Between waits the task awaits the next mark without a timeout,
/// so an idle watch costs nothing,
/// and a dropped
///  watch ends it because its sender is gone.
/// During a wait it wakes at the notice cadence and at the sampling interval of
///  the push meter,
/// whichever is sooner,
/// and at the window itself when that is shorter.
/// The notice is written on the blocking
///  pool,
/// because a terminal that does not take a line must not hold a worker;
/// where no thread can start the notices stop and the
///  judgment goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const mark of marksUntilTimeout()) { if (since === undefined) continue; if (stoppable && clock.stalled()) await stop(pid); else if (ticker.due(elapsed)) await notice(); }
/// ```
pub(crate) async fn watch_loop(mut incoming: UnboundedReceiver<Mark>, flags: WatchFlags, waiting_for: String, pid: u32, terms: WatchTerms, locks: LockLedger) {
    let expired: &AtomicBool = flags.expired.as_ref();
    let stoppable: &AtomicBool = flags.stoppable.as_ref();
    let longest: Arc<AtomicU64> = Arc::clone(&flags.longest);
    let (limit, cadence): (Duration, crate::wait_progress::Cadence) = (terms.limit, terms.cadence);
    let mut notices: Option<Box<dyn Write + Send>> = Some(terms.notices);
    let mut marks: Marks = Marks { since: None, ticker: Ticker::new(cadence), printed: 0, clock: None, pid, window: limit, longest };
    loop {
        let Some(since) = marks.since else {
            match incoming.recv().await {
                Some(mark) => marks.apply(mark, cadence),
                None => return,
            }
            continue;
        };
        let waited: Duration = since.elapsed();
        // A notice that is due goes out before the stall is judged, so a child ended for being quiet was announced.
        if marks.ticker.due(waited) {
            let line: String = child_wait_line(&ChildWaitFacts {
                waiting_for: waiting_for.as_str(),
                pid,
                waited,
                limit,
                limit_note: " (CLI_GIT_CHILD_WAIT_SECONDS changes this)",
                repeat: marks.printed > 0,
                ends_when_quiet: stoppable.load(Ordering::Acquire),
            });
            marks.printed += 1;
            if let Some(sink) = notices.take() {
                notices = write_notice(sink, line).await;
            }
            // The wait may have ended, or another begun, while the line was written.
            while let Ok(mark) = incoming.try_recv() {
                marks.apply(mark, cadence);
            }
            if marks.since != Some(since) {
                continue;
            }
        }
        // The clock is read at every look, a step that may not be run twice included, so its quiet gap is measured even though
        // the step is never ended. Printing can never keep a child that shows no sign of work past its window, because the
        // judgment follows the notice however long the notice took.
        // The ceiling ends the child whatever it shows and whether or not a quiet child may be ended.
        let ceiling: Duration = Duration::from_micros(flags.ceiling.load(Ordering::Acquire));
        if waited >= ceiling {
            flags.ceiling_hit.store(true, Ordering::Release);
            expired.store(true, Ordering::Release);
            marks.keep_gap();
            marks.since = None;
            marks.clock = None;
            stop_process(pid, &locks, &mut incoming).await;
            continue;
        }
        let verdict: Option<StopCause> = marks.clock.as_mut().and_then(Clock::check);
        let quiet: bool = match verdict {
            Some(cause) => matches!(cause, StopCause::Stalled(_) | StopCause::Deadline),
            None => false,
        };
        if quiet && stoppable.load(Ordering::Acquire) {
            // Raised before the child is signalled, so the waiting thread that its end releases reads it as true.
            expired.store(true, Ordering::Release);
            marks.keep_gap();
            marks.since = None;
            marks.clock = None;
            stop_process(pid, &locks, &mut incoming).await;
            continue;
        }
        let until_notice: Duration = marks.ticker.next_in(since.elapsed());
        let until_ceiling: Duration = Duration::from_micros(flags.ceiling.load(Ordering::Acquire)).saturating_sub(since.elapsed());
        let sleep: Duration = until_notice.min(SAMPLE_INTERVAL).min(limit).min(until_ceiling).max(SHORTEST_SLEEP);
        match tokio::time::timeout(sleep, incoming.recv()).await {
            Ok(Some(mark)) => marks.apply(mark, cadence),
            Ok(None) => return,
            Err(_slept) => {}
        }
    }
}

/// What:
///  Write one notice line to `sink` on the blocking pool and hand the sink back;
/// nothing where the write could not run or failed.
/// Why:
///   A notice is advisory,
/// so a failed write is ignored as it always was,
/// and a terminal that never takes the line holds a pool
///  thread,
/// not a worker of the runtime;
/// with no thread to start the notices of this watch stop (the owner,
/// 2026-11-07,
/// "Fail
///  plainly":
/// no inline run of a call that can wait).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sink = await runBlockingJob(() => { sink.write(line + '\n'); return sink; }).catch(() => undefined);
/// ```
async fn write_notice(mut sink: Box<dyn Write + Send>, line: String) -> Option<Box<dyn Write + Send>> {
    return match crate::blocking_job::run_blocking_job(move || {
        // Terminal output error: a notice is advisory; a failed write must not stop the watch.
        let _written = writeln!(sink, "{line}");
        return sink;
    })
    .await
    {
        Ok(kept) => Some(kept),
        Err(error) => {
            crate::diagnostic_log::debug("childWatch", format!("a notice was not written: {error}").as_str());
            None
        }
    };
}
